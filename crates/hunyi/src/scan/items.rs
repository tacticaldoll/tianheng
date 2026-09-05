//! Item traversal and crate-wide scan logic.

use std::collections::HashSet;
use std::path::{Path, PathBuf};

use syn::parse::Parser;

use super::types::*;
use crate::collect::type_param_names;
use crate::crate_scope::local_type_namespace_names;
use crate::errors::{dual_backed_module_error, missing_module_file_error};
use crate::module_resolve::{ModuleFile, locate_module_file, read_parse, resolve_module_branches};
use crate::resolve::{
    AliasMap, BareFallback, ExternRenameMap, ReexportMap, UseMap, alias_nominal_targets,
    bare_single_segment_ident, collect_reexports, collect_uses, extern_verbatim_renamed,
    resolve_path_all, strip_raw,
};
use crate::syn_util::{
    FlatItem, cfg_attr_path_values, child_module_decls, direct_path_value,
    flatten_transparent_macro_items, flatten_with_body_nested_impls, has_cfg_attr,
};
/// Collect crate-root `extern crate X as Y;` renames (`Y → X`) into `out`. Crate-root only: such a
/// rename binds `Y` crate-wide via the extern prelude, whereas a module-scoped `extern crate … as`
/// binds only locally (collecting it crate-wide would false-positive on a same-named head elsewhere
/// — a stated bound). `as _` / `X == Y` / `extern crate self as …` are no-ops.
fn collect_crate_root_extern_renames(items: &[syn::Item], out: &mut ExternRenameMap) {
    // Flattened here too, not only in the walkers: a platform-branching crate root may well write
    // its `extern crate X as Y;` inside a `cfg_if!` arm, and a rename missed here silently
    // mis-resolves every head that uses the alias (a false negative one hop from observation).
    for item in flatten_transparent_macro_items(items) {
        if let syn::Item::ExternCrate(ec) = item {
            if let Some((_, rename)) = &ec.rename {
                let alias = strip_raw(&rename.to_string());
                let real = strip_raw(&ec.ident.to_string());
                if alias != "_" && alias != real && real != "self" {
                    out.insert(alias, real);
                }
            }
        }
    }
}

/// A bare single-segment alias target (`type X = Inner`) whose ident names a non-generic type
/// alias in the *current* module resolves to that alias's canonical key `{module}::{ident}`, so the
/// query fixpoint can follow a bare alias-of-an-alias chain (order-independent). `None` for a
/// leading-`::` / multi-segment / generic-argument-bearing path, or a name that is not a local
/// alias — leaving a bare non-alias target (a local struct, a std prelude type like `String`)
/// unresolved, matching the exposure query's `Ignore` policy for a bare non-alias head (no
/// mis-record, so no false positive even under a boundary forbidding the module's own path).
fn bare_local_alias_target(
    target: &syn::Path,
    module: &str,
    local_alias_names: &HashSet<String>,
) -> Option<String> {
    bare_single_segment_ident(target)
        .filter(|name| local_alias_names.contains(name))
        .map(|name| format!("{module}::{name}"))
}

/// Walk the whole crate from its root, descending every file-based and inline module,
/// collecting re-exports, trait definitions, and trait-impl sites. This is a fresh
/// whole-crate traversal (the single-path `descend` does not fit a "nowhere except
/// here" property); it reuses only the leaf primitives and the shared resolver.
pub(crate) fn scan_crate(
    src_dir: &Path,
    root_file: &Path,
    crate_package: &str,
    externs: &HashSet<String>,
) -> Result<CrateScan, String> {
    let root = read_parse(root_file)?;
    let mut scan = CrateScan {
        reexports: ReexportMap::new(),
        aliases: AliasMap::new(),
        extern_renames: ExternRenameMap::new(),
        trait_defs: HashSet::new(),
        impls: Vec::new(),
        type_defs: Vec::new(),
        alias_targets: AliasMap::new(),
    };
    // Pre-collect crate-root `extern crate X as Y;` renames BEFORE the walk, so the rename map is
    // complete before any alias-target or re-export-closure resolution — every source-order
    // (forward-reference) hazard is eliminated (an alias or re-export preceding the `extern crate`
    // in root source order still resolves). Renames are crate-root-only (they bind crate-wide via
    // the extern prelude; a module-scoped one is a stated bound), so one root scan suffices.
    collect_crate_root_extern_renames(&root.items, &mut scan.extern_renames);
    // Every source file read during the walk, by its canonicalized (symlink-resolved) path. A
    // file-backed `mod x;` is located through the live filesystem, which follows symlinks, so a
    // cyclic symlinked module directory (`src/foo/foo -> src/foo`) would otherwise recurse forever
    // and stack-overflow (SIGABRT) — neither exit 0/1 nor the contract's exit 2. Re-reaching an
    // canonical file already on the descent path is that cycle: "cannot judge" (exit 2), never a
    // crash. 圭表 keeps a parallel canonicalizing guard on its own module-boundary walk (三儀 ⊥ 三儀);
    // 漏刻's probe scanner uses `xingbiao::try_visit` for the same canonicalizing guard.
    // Seeded with the crate root so a submodule looping back to it is caught too.
    let mut ancestors: HashSet<PathBuf> = HashSet::new();
    ancestors.insert(xingbiao::canonicalize_or_fail(root_file)?);
    walk_module(
        root.items,
        "crate".to_string(),
        src_dir.to_path_buf(),
        // The crate root is mod-rs-like: its own directory (`src_dir`, the root file's parent) is the
        // base for both its conventional children and any `#[path]` written in it.
        src_dir.to_path_buf(),
        root_file.to_path_buf(),
        crate_package,
        externs,
        &ancestors,
        0,
        &mut scan,
    )?;
    Ok(scan)
}

/// A module whose source file loops the current descent path back on itself — a symlinked module
/// directory or a circular `#[path]` (rustc's "circular modules"). Diagnosed as "cannot judge"
/// (exit 2) rather than recursing into a stack overflow; never a silent pass.
fn module_cycle_error(module: &str, crate_package: &str, file: &Path) -> String {
    format!(
        "cannot judge module '{module}' in package '{crate_package}': its source file '{}' forms a \
         module cycle (a symlink loop or a circular `#[path]`)",
        file.display()
    )
}

/// A DoS backstop — native call-stack recursion depth, which grows by one on every recursive
/// descent into a child module (inline `mod { … }` nesting AND file-backed `mod x;` descent alike,
/// since both cost one stack frame per level; unlike the symlink/`#[path]`-cycle guard above,
/// `ancestors` alone cannot bound this, because an inline child never opens a new file and so never
/// grows `ancestors`). Past the bound, refuse to recurse further rather than risking an
/// uncontrolled native stack overflow — worse than the contract's own exit-2 "cannot judge", which
/// at least reports why; never a silent pass either way.
///
/// Chosen empirically, not guessed: `walk_module`'s own per-frame footprint (several owned
/// `HashSet`/`String`/`PathBuf` clones per level) overflowed a 2MB test-thread's stack somewhere
/// between 80 and 90 levels of genuine recursion in a from-scratch measurement (see
/// `a_deeply_nested_acyclic_module_tree_is_a_scan_error_not_a_stack_overflow`'s own history) — an
/// order of magnitude below what a naive guess (512, matching `use_scan.rs`'s much cheaper
/// string-based `MAX_USE_NEST_DEPTH`) would have allowed. 32 keeps a wide safety margin below that
/// measured line (real stack-size variance across platforms/threads considered), while still
/// comfortably exceeding any real crate's module nesting depth.
const MAX_MODULE_DEPTH: usize = 32;

/// Shared by all three walkers ([`walk_module`], [`collect_subtree`], `unsafe_sites::walk_unsafe`) so the
/// bound and its wording cannot silently diverge between them (the twin-drift bug class
/// `resolve_child_modules`'s own doc comment names for its guards).
pub(super) fn check_module_depth(
    depth: usize,
    module: &str,
    crate_package: &str,
) -> Result<(), String> {
    if depth >= MAX_MODULE_DEPTH {
        return Err(format!(
            "cannot judge module '{module}' in package '{crate_package}': module nesting exceeds \
             the depth bound ({MAX_MODULE_DEPTH}) this scanner supports without risking a native \
             stack overflow"
        ));
    }
    Ok(())
}

/// The two views of a module's items each of the three walkers needs — the plain list its own
/// observation reads (with every recovered body-nested `impl` folded in), and the arm-membership-
/// carrying list [`resolve_child_modules`] needs for its absence tolerance — from ONE flattening
/// pass. See [`flatten_with_body_nested_impls`] for why `flat` (the second element) is returned
/// untouched rather than itself extended with the recovered impls.
pub(super) fn flatten_for_walk(items: &[syn::Item]) -> (Vec<syn::Item>, Vec<FlatItem>) {
    let (flat, nested_impls) = flatten_with_body_nested_impls(items);
    let mut plain: Vec<syn::Item> = flat.iter().map(|f| f.item.clone()).collect();
    plain.extend(nested_impls);
    (plain, flat)
}

/// Resolve a module's direct child `mod` declarations to the `(items, module path, child dir)` each
/// subtree walk recurses into — the single copy of the descent skeleton and its false-negative-
/// critical guards, shared by [`walk_module`], [`collect_subtree`] (`walk_subtree_modules`), and
/// `unsafe_sites::walk_unsafe` (`scan_unsafe_sites`) so a fix to one guard cannot silently diverge across the
/// three (the twin-drift bug class). Owns: the `#[path]` policy (an **unconditional** `#[path = "…"]`
/// is followed to its author-chosen file/body; a `cfg_attr`-wrapped `#[path]` stays a cfg-conditional
/// skip bound), the inline-vs-file dispatch, the symlink module-cycle guard (a re-reached canonical
/// file is exit 2, never a stack overflow), and the `#[cfg]`-tolerance / non-cfg-missing-file guard
/// (exit 2).
///
/// Children are returned in source order; each caller does its own per-module work, then recurses
/// over them, extending `ancestors` with the child's opened file (see below). `ancestors` is the set
/// of source files on the current descent path (root → this module's file) — NOT a monotonic
/// whole-tree set — so a re-reached file is diagnosed as a cycle only when it loops the path back on
/// itself, never when two sibling/cousin modules legitimately share one `#[path]` target. An inline
/// module's body is cloned (callers borrow their items).
// Each child is `(items, module path, child_dir, file_dir, opened_file, current_file)`: `child_dir`
// is the base for the child's own conventional `mod y;`, `file_dir` the directory a `#[path]`
// written in the child resolves from (they differ for a non-mod-rs `name.rs`, and both accumulate
// an enclosing inline-`mod` name); `opened_file` is the canonical path of the new source file this
// child opened (`Some` for a file-based / `#[path]`-file child, `None` for an inline body that
// stays in the parent's file) — the caller unions it into `ancestors` before recursing.
// `current_file` is the literal (non-canonicalized) path of the file the child's OWN items live
// in — the same file `opened_file` names for a file-based child, or the caller's own
// `current_file` inherited unchanged for an inline body — so a caller that attributes each finding
// to its real source file (rather than a single first-branch file for the whole module) always has
// it in hand, never re-resolved afterward from the module string alone (which misattributes a
// finding once two `#[cfg]`-split branches share one module path). A named struct would obscure
// the by-position destructuring at the three call sites; the shape is documented here.
/// `(items, module path, child_dir, file_dir, opened_file, current_file)` for one resolved child —
/// see `resolve_child_modules`'s own doc for what each position means.
type ChildEntry = (
    Vec<syn::Item>,
    String,
    PathBuf,
    PathBuf,
    Option<PathBuf>,
    PathBuf,
);

/// Canonicalize `file`, check it against the descent-path cycle guard and the crate-wide dedup
/// guard, then read+parse it — the identical "resolve and load a module file" sequence all three
/// file-loading sites in [`resolve_direct_path_child`]/[`resolve_conventional_child`] share.
/// `Ok(None)` means the file was already visited by another branch (the dedup guard) and
/// contributes nothing new; each caller builds its own [`ChildEntry`] from the returned
/// items/canonical path, since the base directories a caller assigns differ (a mod-rs-like
/// `#[path]`-loaded file uses its own directory for BOTH tuple positions; a conventional
/// `<dir>/name.rs` uses the pre-established `<child_dir>/name` for one of them) — that assembly
/// is deliberately NOT folded in here, so this helper never has to guess which shape a caller
/// wants.
fn load_child_file(
    file: &Path,
    child_module: &str,
    crate_package: &str,
    ancestors: &HashSet<PathBuf>,
    seen_files: &mut HashSet<(String, PathBuf)>,
    name: &str,
) -> Result<Option<(Vec<syn::Item>, PathBuf)>, String> {
    let canon = xingbiao::canonicalize_or_fail(file)?;
    if ancestors.contains(&canon) {
        return Err(module_cycle_error(child_module, crate_package, file));
    }
    if !seen_files.insert((name.to_string(), canon.clone())) {
        return Ok(None);
    }
    let parsed = read_parse(file)?;
    Ok(Some((parsed.items, canon)))
}

/// The **unconditional** `#[path = "…"]` remap case: its file (or inline body) is *followed* and
/// observed — closing the relocated-module coverage gap (its `unsafe` sites / items were
/// previously dropped, a false negative). rustc resolves a non-inline `#[path]` relative to
/// `file_dir` — the directory a `#[path]` in the current position resolves from: the containing
/// file's own dir at file scope, but with each **enclosing inline `mod`** name accumulated onto it
/// (rustc adds the inline-module chain as directory components, so
/// `mod inline { #[path="p.rs"] mod inner; }` in `a.rs` loads `<a.rs child dir>/inline/p.rs`, never
/// `<a.rs dir>/p.rs`). A `#[path]`-loaded file is itself mod-rs-like, so ITS children resolve from
/// the loaded file's own directory. An inline `#[path = "dir"] mod x { … }` relocates x's base to
/// `<file_dir>/dir` for BOTH its file-children and any `#[path]` nested in its body — so that
/// becomes the body's `file_dir`.
#[allow(clippy::too_many_arguments)]
fn resolve_direct_path_child(
    rel: &str,
    module_item: &syn::ItemMod,
    name: &str,
    child_module: &str,
    file_dir: &Path,
    current_file: &Path,
    crate_package: &str,
    cfg_conditional: bool,
    ancestors: &HashSet<PathBuf>,
    seen_files: &mut HashSet<(String, PathBuf)>,
    children: &mut Vec<ChildEntry>,
) -> Result<(), String> {
    match &module_item.content {
        // Inline body relocated by `#[path = "dir"]`: `<file_dir>/dir` is the base for the body's
        // file-children AND any `#[path]` written inside it, so it is the body's `file_dir` too
        // (not the enclosing `file_dir` — the relocation accumulates). The body's own content
        // still lives in the enclosing file, so `current_file` inherits unchanged.
        Some((_, inner)) => {
            let relocated = file_dir.join(rel);
            children.push((
                inner.clone(),
                child_module.to_string(),
                relocated.clone(),
                relocated,
                None,
                current_file.to_path_buf(),
            ));
        }
        None => {
            let file = file_dir.join(rel);
            if !xingbiao::is_regular_file(&file)? {
                // An unconditional `#[path]` target must exist (rustc errors otherwise), so an
                // absent one is a genuine broken reference: fail loud (exit 2), never a silent
                // skip. A `cfg_attr`-wrapped `#[path]` is the union `resolve_conventional_child`
                // handles, not this unconditional branch at all (`direct_path_value` never
                // matches it). A BARE `#[cfg(pred)]` co-occurring with this unconditional
                // `#[path]` removes the whole item when `pred` is false (verified against a real
                // rustc build: `#[cfg(windows)] #[path = "…"] mod x;` compiles cleanly on a
                // non-windows host with the target entirely absent) — tolerate exactly like the
                // plain-missing-file case elsewhere in this walker.
                if cfg_conditional {
                    return Ok(());
                }
                return Err(missing_module_file_error(child_module, crate_package));
            }
            let Some((items, canon)) = load_child_file(
                &file,
                child_module,
                crate_package,
                ancestors,
                seen_files,
                name,
            )?
            else {
                return Ok(());
            };
            // mod-rs-like: the loaded file's own directory is the base for both its conventional
            // children and any nested `#[path]` beneath it.
            let own_dir = file
                .parent()
                .map(Path::to_path_buf)
                .unwrap_or_else(|| file_dir.to_path_buf());
            children.push((
                items,
                child_module.to_string(),
                own_dir.clone(),
                own_dir,
                Some(canon),
                file,
            ));
        }
    }
    Ok(())
}

/// The no-`#[path]` case: a `cfg_attr`-wrapped `#[path]` is cfg-conditional on which file
/// compiles, but — unlike a bare `#[cfg]` — `cfg_attr` never removes the `mod` item itself, so
/// cfg-blind observation must union every candidate the predicate could select, never skip the
/// module outright. An INLINE body is unaffected by `#[path]`/`cfg_attr(path)` at all (rustc
/// ignores it for an inline `mod`; the body always compiles) and is unconditionally descended,
/// exactly like the no-attribute case. A FILE module's conventional file and its `cfg_attr`
/// target are both read when present, as separate sources for the same module name (mirroring the
/// per-platform-pair `seen_files` union for two plain declarations of the same name).
#[allow(clippy::too_many_arguments)]
fn resolve_conventional_child(
    module_item: &syn::ItemMod,
    name: &str,
    child_module: &str,
    child_dir: &Path,
    file_dir: &Path,
    current_file: &Path,
    crate_package: &str,
    cfg_conditional: bool,
    ancestors: &HashSet<PathBuf>,
    seen_files: &mut HashSet<(String, PathBuf)>,
    children: &mut Vec<ChildEntry>,
) -> Result<(), String> {
    let cfg_attr_targets = cfg_attr_path_values(&module_item.attrs);
    let sub_dir = child_dir.join(name);
    match &module_item.content {
        // Inline `mod x { … }`: descend its lexical items (same file). Its own children — both
        // conventional `mod y;` AND any `#[path]` nested in the body — resolve from `<child_dir>/x`
        // (rustc accumulates the inline-module name as a directory component), so that dir is the
        // body's `file_dir` too, NOT the enclosing `file_dir`. Getting this wrong drops a
        // `#[path]` relocated inside an inline block onto the wrong file — a false negative.
        // The body's own content stays in the enclosing file, so `current_file` inherits
        // unchanged.
        // A `cfg_attr(…, path = "dir")` on this inline header names a CANDIDATE base per platform
        // predicate (the unconditional form is dispatched to `resolve_direct_path_child` before this
        // point, so only the conditional one reaches here). Every candidate is unioned with the
        // conventional base, cfg-blind: `syn` does not evaluate `cfg`, so preferring one would drop
        // every child beneath the other. A candidate is descended only when it EXISTS as a directory
        // — recursing into an absent one would fail loud on the body's other, unrelated nested items
        // solely because one platform's directory is missing — and when none exists the conventional
        // base is descended anyway, so a nested reference broken on every platform still fails loud.
        // Not following it made 渾儀 exit 2 on source that compiles cleanly, while this crate's own
        // FILE-form arm below already followed the same attribute; 圭表 and 漏刻 state the same rule
        // for the identical shape (三儀 ⊥ 三儀: the same rule, hand-written per dimension).
        Some((_, inner)) => {
            let bases: Vec<PathBuf> = cfg_attr_targets
                .iter()
                .map(|rel| file_dir.join(rel))
                .chain(std::iter::once(sub_dir.clone()))
                .collect();
            // An unreadable base is not an absent one; dropping it takes its subtree with it.
            let mut bases: Vec<PathBuf> = {
                let mut kept = Vec::new();
                for base in bases {
                    if xingbiao::is_directory(&base)? {
                        kept.push(base);
                    }
                }
                kept
            };
            bases.sort();
            bases.dedup();
            if bases.is_empty() {
                bases.push(sub_dir);
            }
            for base in bases {
                children.push((
                    inner.clone(),
                    child_module.to_string(),
                    base.clone(),
                    base,
                    None,
                    current_file.to_path_buf(),
                ));
            }
        }
        // File `mod x;`: `<dir>/x.rs` or `<dir>/x/mod.rs`; children under `x/`; the child's own
        // `file_dir` is the located file's directory (`<dir>` for `x.rs`, `<dir>/x` for
        // `x/mod.rs`), which is where a `#[path]` inside it resolves from.
        None => {
            let mut has_backing_source = false;
            // Every `cfg_attr(path)` target this declaration carries (a module may stack more
            // than one, each gated by its own predicate for a different platform), whichever
            // exists — read alongside the conventional file below, never in place of it: cfg-blind
            // observation cannot know which one this build actually compiles.
            for rel in &cfg_attr_targets {
                let file = file_dir.join(rel);
                if xingbiao::is_regular_file(&file)? {
                    has_backing_source = true;
                    if let Some((items, canon)) = load_child_file(
                        &file,
                        child_module,
                        crate_package,
                        ancestors,
                        seen_files,
                        name,
                    )? {
                        let own_dir = file
                            .parent()
                            .map(Path::to_path_buf)
                            .unwrap_or_else(|| file_dir.to_path_buf());
                        children.push((
                            items,
                            child_module.to_string(),
                            own_dir.clone(),
                            own_dir,
                            Some(canon),
                            file,
                        ));
                    }
                }
            }
            match locate_module_file(child_dir, name)? {
                ModuleFile::One(file) => {
                    // A file already on the current descent path (an ANCESTOR, by canonical
                    // symlink-resolved path) is a genuine module cycle — a symlinked directory or a
                    // circular `#[path]` looping the `mod` graph back on itself. Stop with a scan
                    // error (exit 2 "cannot judge") rather than recursing into a stack overflow. Two
                    // *sibling/cousin* declarations legitimately resolving to one file (e.g.
                    // `#[path="s.rs"] mod a; #[path="s.rs"] mod b;`, which rustc compiles) are NOT a
                    // cycle — the ancestor set, unlike a monotonic whole-tree visited set, does not
                    // misreport them (that would be a false positive on compilable input).
                    if let Some((items, canon)) = load_child_file(
                        &file,
                        child_module,
                        crate_package,
                        ancestors,
                        seen_files,
                        name,
                    )? {
                        let own_dir = file
                            .parent()
                            .map(Path::to_path_buf)
                            .unwrap_or_else(|| sub_dir.clone());
                        children.push((
                            items,
                            child_module.to_string(),
                            sub_dir,
                            own_dir,
                            Some(canon),
                            file,
                        ));
                    }
                }
                // BOTH conventional forms present is unresolvable under every predicate value — no
                // `#[cfg]` makes two files compile as one module — so unlike an absence it is never
                // a legitimate configuration, and it reacts regardless of any gate. Erroring out of
                // the whole walk rather than dropping this one module matches 圭表's own
                // `resolve_plain_sources`: a crate whose module graph cannot be resolved cannot be
                // judged, and excluding the module would hide every import beneath it.
                ModuleFile::Ambiguous { flat, nested } => {
                    return Err(dual_backed_module_error(
                        child_module,
                        name,
                        crate_package,
                        &flat,
                        &nested,
                    ));
                }
                // A `#[cfg]`-gated module may legitimately have no source file when the feature is
                // off (a standard optional-feature pattern) — a stated coverage bound, not a scan
                // error. A non-cfg missing conventional file is tolerated too when the `cfg_attr`
                // target above already backs this module on some other build; only when NEITHER
                // candidate exists, and this declaration is not otherwise cfg-conditional, is the
                // module truly unbacked on every configuration — a real scan error (exit 2).
                ModuleFile::Absent => {
                    if !has_backing_source && !cfg_conditional {
                        return Err(missing_module_file_error(child_module, crate_package));
                    }
                }
            }
        }
    }
    Ok(())
}

// `child_dir` and `file_dir` are distinct module-resolution bases (see `child_dir` and `file_dir`), not
// bundled: they thread the descent by position alongside the crate-scan accumulator and guards.
pub(super) fn resolve_child_modules(
    items: &[FlatItem],
    module: &str,
    child_dir: &Path,
    file_dir: &Path,
    current_file: &Path,
    crate_package: &str,
    ancestors: &HashSet<PathBuf>,
) -> Result<Vec<ChildEntry>, String> {
    let mut children = Vec::new();
    // Deduped by (declared name, resolved file's CANONICAL path): two mutually-exclusive `#[cfg]`
    // arms that both plainly declare the SAME name `mod seg;` (no `#[path]`, so both are found via
    // the identical `locate_module_file` lookup), or that both `#[path]`-remap the SAME name to the
    // identical target, are the same real file compiled twice by neither build — pushing a branch
    // per occurrence would duplicate that file's items in the crate-wide scan
    // (`ImplSite`/`TypeDef`/`UnsafeSite`), inflating one real violation into two apparently-
    // distinct findings whenever a self-type's generic argument is unrenderable and falls back to
    // a positional ordinal that differs between the two scan-Vec positions (escaping the eventual
    // fact-identity dedup). Mirrors `module_resolve.rs::descend`'s own `seen_files` guard.
    // Keyed on the NAME too, not the file alone: two DIFFERENT declared names that
    // happen to `#[path]`-remap to the identical file (`#[path="s.rs"] mod a;` / `#[path="s.rs"]
    // mod b;`) are two real, separately-compiled modules — already an existing, tested case — and
    // must never collide with each other's own dedup entry.
    let mut seen_files: HashSet<(String, PathBuf)> = HashSet::new();
    // Takes items with transparent-macro (`cfg_if!`) arms flattened in, so a `mod` declared only
    // inside an arm is descended like any other — 圭表 already observes such a declaration
    // (`declared_modules_observes_mod_inside_cfg_if_macro_body`), and a module invisible here costs
    // the whole subtree beneath it its observation (its `unsafe` sites, markers, and trait impls).
    // The parameter is `&[FlatItem]`, not `&[syn::Item]`, because arm MEMBERSHIP is load-bearing
    // here (the absence tolerance below) and flattening erases it: handed an already-flattened
    // plain list, this walker would report a legitimately fileless arm-declared module as exit 2.
    // The type makes that mistake impossible rather than a comment asking callers not to make it.
    for flat in items {
        let syn::Item::Mod(module_item) = &flat.item else {
            continue;
        };
        let name = strip_raw(&module_item.ident.to_string());
        let child_module = format!("{module}::{name}");
        // May this declaration legitimately have no source file on this build? Its own bare
        // `#[cfg]`, or arm membership (every `cfg_if!` arm is gated by a predicate in the macro
        // header). The same rule `module_resolve::descend` applies, matching 圭表.
        let cfg_conditional = flat.in_transparent_arm || has_cfg_attr(&module_item.attrs);
        if let Some(rel) = direct_path_value(&module_item.attrs) {
            resolve_direct_path_child(
                &rel,
                module_item,
                &name,
                &child_module,
                file_dir,
                current_file,
                crate_package,
                cfg_conditional,
                ancestors,
                &mut seen_files,
                &mut children,
            )?;
            continue;
        }
        resolve_conventional_child(
            module_item,
            &name,
            &child_module,
            child_dir,
            file_dir,
            current_file,
            crate_package,
            cfg_conditional,
            ancestors,
            &mut seen_files,
            &mut children,
        )?;
    }
    Ok(children)
}

// `child_dir` and `file_dir` are distinct module-resolution bases (see `resolve_child_modules`), not
// bundled: they thread the descent by position alongside the crate-scan accumulator and guards.
/// Record this module's own facts into `scan` from a single flattened items pass: trait
/// definitions, trait-impl sites, type definitions, and resolvable type-alias targets (including
/// the forbidden-marker alias-landing map). Pulled out of `walk_module` as the "this module's own
/// observation" phase, distinct from the child-descent phase that follows it.
#[allow(clippy::too_many_arguments)]
fn record_module_facts(
    items: &[syn::Item],
    module: &str,
    current_file: &Path,
    uses: &UseMap,
    externs: &HashSet<String>,
    externs_type: &HashSet<String>,
    local_alias_names: &HashSet<String>,
    scan: &mut CrateScan,
) -> Result<(), String> {
    for item in items {
        match item {
            syn::Item::Trait(trait_item) => {
                scan.trait_defs.insert(format!(
                    "{module}::{}",
                    strip_raw(&trait_item.ident.to_string())
                ));
            }
            // Trait impls only (`impl Trait for Type`); inherent impls carry no `trait_`.
            syn::Item::Impl(impl_item) if impl_item.trait_.is_some() => {
                let (_, trait_path, _) = impl_item.trait_.as_ref().expect("trait_ is Some");
                scan.impls.push(ImplSite {
                    module: module.to_string(),
                    file: current_file.to_path_buf(),
                    trait_path: trait_path.clone(),
                    self_ty: (*impl_item.self_ty).clone(),
                    uses: uses.clone(),
                    type_params: type_param_names(&impl_item.generics),
                });
            }
            syn::Item::Struct(i) => {
                push_type_def(&i.attrs, &i.ident, module, current_file, uses, scan)?;
            }
            syn::Item::Enum(i) => {
                push_type_def(&i.attrs, &i.ident, module, current_file, uses, scan)?;
            }
            syn::Item::Union(i) => {
                push_type_def(&i.attrs, &i.ident, module, current_file, uses, scan)?;
            }
            // A non-generic `type X = <nominal path>;` alias: record `{module}::X → target`
            // so the exposure pipeline can follow it to the defining path. The target-resolution
            // ladder is byte-identical to the query site's, so no resolvable target is dropped and
            // no local shadow is misread:
            //   0. a leading-`::` target — an unambiguous extern (raw set, with the crate-root
            //      rename applied), a HARD short-circuit, so `type X = ::serde::Value;` records the
            //      extern even under a local `mod serde`, and `type X = ::<rename>::Foo;` too;
            //   1. `resolve_path_all(Ignore)` — use-map (every cfg-branch candidate) /
            //      `crate`·`self`·`super`;
            //   2. `bare_local_alias_target` — a bare single-segment target naming one of THIS
            //      module's own type aliases recorded as `{module}::{name}` (its canonical alias-map
            //      key), tried BEFORE the extern oracle so a local alias shadows a same-named
            //      dependency (rustc's own resolution); the query-time `expand_canonical_paths`
            //      fixpoint then closes a *bare* alias-of-an-alias chain regardless of source order.
            //      Gated to local alias names, so a bare non-alias target (a local struct, a std
            //      prelude type like `String`) is never mis-recorded — no false positive;
            //   3. `extern_verbatim_renamed` — an extern head, incl. a crate-root `extern crate as`
            //      rename (the rename map is pre-collected, so this is order-independent).
            // A generic alias (`type X<T> = …`) or a complex target (`Vec<T>`, `&T`, a
            // tuple/`dyn`/`impl`) is skipped — a stated coverage bound, never a silent claim.
            syn::Item::Type(type_item) => {
                if !type_item.generics.params.is_empty() {
                    // Stated bound: generic type aliases (`type X<T> = …`) are intentionally skipped.
                    continue;
                }
                // Record the alias's LANDING type — where its target resolves under the same bare-head
                // `CurrentModule` fallback the impl-self check uses — so the forbidden-marker check can
                // react on an alias to a crate-defined subtree type (`type Bar = Real`) yet stay silent
                // on one to a foreign/prelude type (`type Baz = Vec<u8>` / `= String`), whose marker
                // lands off the governed subtree. Only a nominal `Type::Path` target has a single
                // landing type; a tuple/ref/`dyn` target has none and is skipped (never governed here).
                if let syn::Type::Path(tp) = &*type_item.ty {
                    let landings =
                        resolve_path_all(&tp.path, uses, module, BareFallback::CurrentModule);
                    if !landings.is_empty() {
                        let alias =
                            format!("{module}::{}", strip_raw(&type_item.ident.to_string()));
                        let entry = scan.alias_targets.entry(alias).or_default();
                        for landing in landings {
                            if !entry.contains(&landing) {
                                entry.push(landing);
                            }
                        }
                    }
                }
                let mut targets = Vec::new();
                alias_nominal_targets(&type_item.ty, &mut targets);
                for target in targets {
                    let alias = format!("{module}::{}", strip_raw(&type_item.ident.to_string()));
                    let resolved_list: Vec<String> = if target.leading_colon.is_some() {
                        extern_verbatim_renamed(target, externs, &scan.extern_renames)
                            .into_iter()
                            .collect()
                    } else {
                        let use_candidates =
                            resolve_path_all(target, uses, module, BareFallback::Ignore);
                        if !use_candidates.is_empty() {
                            use_candidates
                        } else {
                            bare_local_alias_target(target, module, local_alias_names)
                                .or_else(|| {
                                    extern_verbatim_renamed(
                                        target,
                                        externs_type,
                                        &scan.extern_renames,
                                    )
                                })
                                .into_iter()
                                .collect()
                        }
                    };
                    for resolved in resolved_list {
                        if resolved != alias {
                            let entry = scan.aliases.entry(alias.clone()).or_default();
                            if !entry.contains(&resolved) {
                                entry.push(resolved);
                            }
                        }
                    }
                }
            }
            _ => {}
        }
    }
    Ok(())
}

#[allow(clippy::too_many_arguments)]
fn walk_module(
    items: Vec<syn::Item>,
    module: String,
    child_dir: PathBuf,
    file_dir: PathBuf,
    current_file: PathBuf,
    crate_package: &str,
    externs: &HashSet<String>,
    ancestors: &HashSet<PathBuf>,
    depth: usize,
    scan: &mut CrateScan,
) -> Result<(), String> {
    check_module_depth(depth, &module, crate_package)?;
    // ONE flattening pass, two views. `flat` retains arm membership for the child resolution below;
    // `items` is the plain list this module's own observation reads — a re-export, trait definition,
    // trait impl, or type definition written inside an arm is a real declaration of this module, and
    // the crate-wide maps built here feed every capability's resolution, so a miss is not one lost
    // fact but a mis-resolution everywhere downstream.
    let (items, flat) = flatten_for_walk(&items);
    let uses = collect_uses(&items);
    // The re-export closure applies the same per-defining-module child-module shadow the direct
    // head oracle does: a bare `pub use dep::X;` / `pub use wc::X;` head named by this module's own
    // child `mod dep` / `mod wc` is not recorded as the dependency / renamed crate, so a
    // cross-module facade reaching it through this crate-wide map does not mis-canonicalize. That
    // exclusion is now computed PER re-export item (via `flat`'s own arm/`#[cfg]` tag, inside
    // `collect_reexports`), not once over this module's whole child-module set: a `mod` that is
    // provably mutually exclusive with a SPECIFIC `pub use` in `flat` (a different `cfg_if!` arm,
    // or a syntactic `#[cfg]` negation) must not shadow that item's own head even though both live
    // in this same file (see `collect_reexports`'s own doc).
    // `collect_reexports` keeps a leading-`::` head on the raw sets regardless.
    let child_mod_decls = child_module_decls(&flat);
    collect_reexports(
        &flat,
        &module,
        externs,
        &child_mod_decls,
        &scan.extern_renames,
        &mut scan.reexports,
    );
    // Alias targets resolve in the same per-module shadow as type positions: a bare head naming
    // a local child module (`mod serde` + `type X = serde::Foo`) is local, not the dependency.
    let externs_type: HashSet<String> = externs
        .difference(&local_type_namespace_names(&items))
        .cloned()
        .collect();
    // This module's own non-generic type-alias names — the only bare single-segment targets the
    // alias-collection ladder resolves against the current module (a bare intermediate in an
    // alias-of-an-alias chain, always same-module). Gating to these names keeps a bare non-alias
    // target (a local struct, or a std prelude type like `String`) from being mis-recorded as
    // `{module}::{name}` — which would false-positive under a boundary forbidding the module's own
    // path. Computed once here so the check is order-independent within the module.
    let local_alias_names: HashSet<String> = items
        .iter()
        .filter_map(|it| match it {
            syn::Item::Type(t) if t.generics.params.is_empty() => {
                Some(strip_raw(&t.ident.to_string()))
            }
            _ => None,
        })
        .collect();

    record_module_facts(
        &items,
        &module,
        &current_file,
        &uses,
        externs,
        &externs_type,
        &local_alias_names,
        scan,
    )?;

    for (child_items, child_module, sub_dir, sub_file_dir, opened, child_file) in
        resolve_child_modules(
            &flat,
            &module,
            &child_dir,
            &file_dir,
            &current_file,
            crate_package,
            ancestors,
        )?
    {
        // Extend the ancestor path with the child's own file (an inline body stays in the parent's
        // file, so it inherits `ancestors` unchanged); each sibling branches from the SAME parent
        // path, so a file shared across siblings is never mistaken for a cycle.
        match opened {
            Some(canon) => {
                let mut child_ancestors = ancestors.clone();
                child_ancestors.insert(canon);
                walk_module(
                    child_items,
                    child_module,
                    sub_dir,
                    sub_file_dir,
                    child_file,
                    crate_package,
                    externs,
                    &child_ancestors,
                    depth + 1,
                    scan,
                )?;
            }
            None => walk_module(
                child_items,
                child_module,
                sub_dir,
                sub_file_dir,
                child_file,
                crate_package,
                externs,
                ancestors,
                depth + 1,
                scan,
            )?,
        }
    }
    Ok(())
}

/// Walk the anchored module's whole subtree — the module itself and every descendant (file-based
/// `mod x;` and inline `mod x { … }` alike) — returning each module's path and the items it owns.
/// The subtree analogue of [`crate::module_resolve::resolve_module_items_with_files`]: where that
/// returns one module's items, this returns every module at or below the anchor, so a reaction can
/// observe a "nowhere under here" property (e.g. no public `async fn` anywhere beneath a
/// sans-I/O kernel).
///
/// Inherits the crate walk's guards, so a subtree reaction never silently under-reacts: an
/// **unconditional** `#[path]`-remapped module is followed like any other descendant (matching
/// `resolve_child_modules`'s own policy), a `cfg_attr`-wrapped `#[path]` module is observed via the
/// identical union `resolve_child_modules` applies (an inline body regardless of the attribute, a
/// file module's conventional file and its `cfg_attr` target both read when they exist on disk),
/// a `#[cfg]`-gated fileless module is tolerated, a non-`#[cfg]` missing module file is a scan
/// error (exit 2), and a symlink module cycle is a scan error (exit 2), never a stack overflow.
///
/// When the anchor (or any segment on the path to it) was reached through a mutually-exclusive
/// `#[cfg]` split, [`resolve_module_branches`] keeps every surviving branch's own items paired
/// with its own directories — the subtree walk runs `collect_subtree` **once per branch**, each
/// seeded with only that branch's own ancestor file, and merges every branch's results. Using
/// `resolve_module_root`'s single, first-branch-only directory pair together with its *unioned*
/// items here would resolve a non-first branch's own child against the wrong directory, silently
/// dropping it — a real false negative found on adversarial review.
pub(crate) fn walk_subtree_modules(
    src_dir: &Path,
    root_file: &Path,
    module: &str,
    crate_package: &str,
) -> Result<Vec<(String, Vec<syn::Item>, PathBuf)>, String> {
    let branches = resolve_module_branches(src_dir, root_file, module, crate_package)?;
    let mut out: Vec<(String, Vec<syn::Item>, PathBuf)> = Vec::new();
    for (items, file, child_dir, file_dir) in branches {
        // Seed the ancestor path with THIS branch's own file, so a descendant looping back to it
        // is caught — the same discipline `scan_crate` applies from the crate root. Never a set
        // shared across branches: two mutually-exclusive `#[cfg]` arms' own files are never
        // simultaneously open in any real build, so one arm's file must never gate the other's.
        let mut ancestors: HashSet<PathBuf> = HashSet::new();
        ancestors.insert(xingbiao::canonicalize_or_fail(&file)?);
        // This branch's own `path_base` IS the base a `#[path]` written in it resolves from —
        // used AS-IS, never re-derived as `file.parent()`: for an inline-module branch,
        // `path_base` is its accumulated directory, which differs from the *enclosing* file's own
        // directory (the inline body stays in the parent's file, but its own `#[path]`s and
        // conventional children do not resolve from the parent's directory) — re-deriving it here
        // silently substituted the wrong base and could hard-error or, worse, silently observe
        // the wrong (uncompiled) file in the subtree walk.
        collect_subtree(
            items,
            module.to_string(),
            child_dir,
            file_dir,
            file,
            crate_package,
            &ancestors,
            0,
            &mut out,
        )?;
    }
    Ok(out)
}

/// Recurse the subtree from one module: descend each child `mod` (mirroring [`walk_module`]'s
/// descent and its guards), then record this module's own `(path, items, file)` — `file` the real
/// file this module's own branch was resolved from, so a caller attributes each finding to the
/// file that actually produced it rather than re-resolving from the module string afterward (which
/// misattributes a finding once two `#[cfg]`-split branches share one module path). The order of
/// `out` is unspecified — a subtree reaction sorts its findings — so recording after descent is
/// fine.
#[allow(clippy::too_many_arguments)]
fn collect_subtree(
    items: Vec<syn::Item>,
    module: String,
    child_dir: PathBuf,
    file_dir: PathBuf,
    current_file: PathBuf,
    crate_package: &str,
    ancestors: &HashSet<PathBuf>,
    depth: usize,
    out: &mut Vec<(String, Vec<syn::Item>, PathBuf)>,
) -> Result<(), String> {
    check_module_depth(depth, &module, crate_package)?;
    // Flattened before the items are recorded in `out`: a subtree reaction (`including_submodules`)
    // observes each module's item list directly, so an arm item missing here is a false negative in
    // the subtree scope even though the same item reacts at the anchor itself. `flat` keeps arm
    // membership for the child resolution.
    let (items, flat) = flatten_for_walk(&items);
    for (child_items, child_module, sub_dir, sub_file_dir, opened, child_file) in
        resolve_child_modules(
            &flat,
            &module,
            &child_dir,
            &file_dir,
            &current_file,
            crate_package,
            ancestors,
        )?
    {
        match opened {
            Some(canon) => {
                let mut child_ancestors = ancestors.clone();
                child_ancestors.insert(canon);
                collect_subtree(
                    child_items,
                    child_module,
                    sub_dir,
                    sub_file_dir,
                    child_file,
                    crate_package,
                    &child_ancestors,
                    depth + 1,
                    out,
                )?;
            }
            None => collect_subtree(
                child_items,
                child_module,
                sub_dir,
                sub_file_dir,
                child_file,
                crate_package,
                ancestors,
                depth + 1,
                out,
            )?,
        }
    }
    out.push((module, items, current_file));
    Ok(())
}

/// Record a type definition with its derive paths into the scan.
fn push_type_def(
    attrs: &[syn::Attribute],
    ident: &syn::Ident,
    module: &str,
    file: &Path,
    uses: &UseMap,
    scan: &mut CrateScan,
) -> Result<(), String> {
    let name = strip_raw(&ident.to_string());
    let derives = extract_derives(attrs)?;
    scan.type_defs.push(TypeDef {
        canonical: format!("{module}::{name}"),
        module: module.to_string(),
        file: file.to_path_buf(),
        derives,
        uses: uses.clone(),
    });
    Ok(())
}

/// Extract the derive paths from a type's `#[derive(...)]` and `#[cfg_attr(_, derive(...))]`
/// attributes (the latter read cfg-agnostically). A `derive` whose arguments fail to parse is
/// a scan error (exit 2) — "cannot judge" is never a silent skip.
fn extract_derives(attrs: &[syn::Attribute]) -> Result<Vec<syn::Path>, String> {
    let mut out = Vec::new();
    for attr in attrs {
        if attr.path().is_ident("derive") {
            out.extend(parse_derive_paths(&attr.meta)?);
        } else if attr.path().is_ident("cfg_attr") {
            let metas = attr
                .parse_args_with(meta_list_parser())
                .map_err(|e| format!("cannot parse #[cfg_attr(...)]: {e}"))?;
            extract_derives_from_cfg_metas(&metas, &mut out)?;
        }
    }
    Ok(out)
}

fn meta_list_parser() -> impl Parser<Output = syn::punctuated::Punctuated<syn::Meta, syn::Token![,]>>
{
    syn::punctuated::Punctuated::<syn::Meta, syn::Token![,]>::parse_terminated
}

/// Parse the comma-separated paths of a `derive(...)` meta-list (empty `#[derive]`/non-list
/// yields none).
fn parse_derive_paths(meta: &syn::Meta) -> Result<Vec<syn::Path>, String> {
    let parser = syn::punctuated::Punctuated::<syn::Path, syn::Token![,]>::parse_terminated;
    match meta {
        syn::Meta::List(list) => Ok(list
            .parse_args_with(parser)
            .map_err(|e| format!("cannot parse derive(...): {e}"))?
            .into_iter()
            .collect()),
        _ => Ok(Vec::new()),
    }
}

/// Extract derives from a `cfg_attr`'s metas: the first is the cfg predicate (skipped); the
/// rest are conditionally-applied attributes — a `derive(...)`, or a **nested** `cfg_attr(...)`
/// recursed into (so `#[cfg_attr(a, cfg_attr(b, derive(X)))]` still yields `X`).
fn extract_derives_from_cfg_metas(
    metas: &syn::punctuated::Punctuated<syn::Meta, syn::Token![,]>,
    out: &mut Vec<syn::Path>,
) -> Result<(), String> {
    for meta in metas.iter().skip(1) {
        if let syn::Meta::List(list) = meta {
            if list.path.is_ident("derive") {
                out.extend(parse_derive_paths(meta)?);
            } else if list.path.is_ident("cfg_attr") {
                let inner = list
                    .parse_args_with(meta_list_parser())
                    .map_err(|e| format!("cannot parse nested #[cfg_attr(...)]: {e}"))?;
                extract_derives_from_cfg_metas(&inner, out)?;
            }
        }
    }
    Ok(())
}

// --- Unsafe-site scan (`semantic-unsafe-confinement`) -------------------------
