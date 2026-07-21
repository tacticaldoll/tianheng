//! The crate-wide scan — one fresh whole-crate traversal from the root, descending every
//! file-based and inline module, that collects the `pub use` re-export closure, the resolvable
//! type-alias map, crate-root `extern crate … as` renames, the locally-defined trait paths,
//! every trait-impl site, and every type definition (with its `#[derive]`s). The reaction hearts
//! read the resulting [`CrateScan`]; this is distinct from the single-path descent in
//! `module_resolve` (which does not fit a "nowhere except here" property), reusing only the leaf
//! primitives and the shared resolver.

use std::collections::{HashMap, HashSet};
use std::path::{Path, PathBuf};

use syn::parse::Parser;
use syn::visit::{self, Visit};

use crate::crate_scope::{child_module_names, local_type_namespace_names};
use crate::errors::missing_module_file_error;
use crate::module_resolve::{locate_module_file, read_parse, resolve_module_branches};
use crate::resolve::{
    AliasMap, BareFallback, ExternRenameMap, ReexportMap, UseMap, alias_nominal_target,
    bare_single_segment_ident, collect_reexports, collect_uses, extern_verbatim_renamed,
    resolve_path, strip_raw, type_to_string,
};
use crate::syn_util::{direct_path_value, has_path_attr};

/// One impl site observed in the crate: its enclosing module path, the **real file it was read
/// from** (its own branch's file — never re-resolved afterward from the module string, which
/// misattributes a finding whenever two `#[cfg]`-split branches share one module path), the
/// written trait path, the implemented-for type, and that module's `use`-map (for resolution).
pub(crate) struct ImplSite {
    pub(crate) module: String,
    pub(crate) file: PathBuf,
    pub(crate) trait_path: syn::Path,
    pub(crate) self_ty: syn::Type,
    pub(crate) uses: UseMap,
}

/// One type definition observed in the crate: its canonical path (`module::Name`), the module
/// it is defined in and the real file it was read from (for a forbidden-`derive` finding's source
/// file — its own branch's file, same provenance guarantee as [`ImplSite::file`]), the paths in
/// its `#[derive(...)]`/`#[cfg_attr(_, derive(...))]`, and that module's `use`-map (so a renamed
/// derive macro, `use serde::Serialize as Ser; #[derive(Ser)]`, resolves to its true leaf).
pub(crate) struct TypeDef {
    pub(crate) canonical: String,
    pub(crate) module: String,
    pub(crate) file: PathBuf,
    pub(crate) derives: Vec<syn::Path>,
    pub(crate) uses: UseMap,
}

/// One crate-wide scan: the `pub use` re-export closure, the set of locally-defined trait
/// paths (for anchor verification), every trait-impl site, and every type definition.
pub(crate) struct CrateScan {
    pub(crate) reexports: ReexportMap,
    pub(crate) aliases: AliasMap,
    pub(crate) extern_renames: ExternRenameMap,
    pub(crate) trait_defs: HashSet<String>,
    pub(crate) impls: Vec<ImplSite>,
    pub(crate) type_defs: Vec<TypeDef>,
    /// For each non-generic `type X = <path>;` whose target is a nominal path, the alias's canonical
    /// key (`{module}::X`) mapped to the **landing type** its target resolves to under the same
    /// bare-head `CurrentModule` fallback the impl-self check uses (`type Bar = Real` in `crate::dom`
    /// → `crate::dom::Real`; `type Baz = Vec<u8>` / `= String` → `crate::dom::Vec` / `crate::dom::String`,
    /// neither crate-defined). A `type` alias defines no new type — coherence sees through it — so a
    /// marker impl'd on `Bar` governs a subtree type IFF this landing type is itself a crate-defined
    /// subtree type. The forbidden-marker check consults this to react on `type Bar = Real` while NOT
    /// firing on an alias to a foreign/prelude type (whose marker lands off the governed subtree).
    /// (This is distinct from `aliases`, the exposure closure's resolvable-target map, which does not
    /// record a bare-local-struct target.)
    pub(crate) alias_targets: HashMap<String, String>,
}

/// Collect crate-root `extern crate X as Y;` renames (`Y → X`) into `out`. Crate-root only: such a
/// rename binds `Y` crate-wide via the extern prelude, whereas a module-scoped `extern crate … as`
/// binds only locally (collecting it crate-wide would false-positive on a same-named head elsewhere
/// — a stated bound). `as _` / `X == Y` / `extern crate self as …` are no-ops.
fn collect_crate_root_extern_renames(items: &[syn::Item], out: &mut ExternRenameMap) {
    for item in items {
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
        alias_targets: HashMap::new(),
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
    // crash. The louke probe scanner guards the same hazard; the two dimensions keep parallel copies
    // (三儀 ⊥ 三儀). Seeded with the crate root so a submodule looping back to it is caught too.
    let mut ancestors: HashSet<PathBuf> = HashSet::new();
    ancestors.insert(canonicalize_source(root_file)?);
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
        &mut scan,
    )?;
    Ok(scan)
}

/// Canonicalize a source file path (resolving symlinks) for the ancestor-set cycle guard; an
/// unresolvable path is a scan error ("cannot judge"), never a silent skip.
fn canonicalize_source(file: &Path) -> Result<PathBuf, String> {
    std::fs::canonicalize(file).map_err(|err| {
        format!(
            "cannot canonicalize source file '{}': {err}",
            file.display()
        )
    })
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

/// Resolve a module's direct child `mod` declarations to the `(items, module path, child dir)` each
/// subtree walk recurses into — the single copy of the descent skeleton and its false-negative-
/// critical guards, shared by [`walk_module`], [`collect_subtree`] (`walk_subtree_modules`), and
/// [`walk_unsafe`] (`scan_unsafe_sites`) so a fix to one guard cannot silently diverge across the
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
#[allow(clippy::type_complexity)]
fn resolve_child_modules(
    items: &[syn::Item],
    module: &str,
    child_dir: &Path,
    file_dir: &Path,
    current_file: &Path,
    crate_package: &str,
    ancestors: &HashSet<PathBuf>,
) -> Result<
    Vec<(
        Vec<syn::Item>,
        String,
        PathBuf,
        PathBuf,
        Option<PathBuf>,
        PathBuf,
    )>,
    String,
> {
    let mut children = Vec::new();
    // Deduped by (declared name, resolved file's CANONICAL path): two mutually-exclusive `#[cfg]`
    // arms that both plainly declare the SAME name `mod seg;` (no `#[path]`, so both are found via
    // the identical `locate_module_file` lookup), or that both `#[path]`-remap the SAME name to the
    // identical target, are the same real file compiled twice by neither build — pushing a branch
    // per occurrence would duplicate that file's items in the crate-wide scan
    // (`ImplSite`/`TypeDef`/`UnsafeSite`), inflating one real violation into two apparently-
    // distinct findings whenever a self-type's generic argument is unrenderable and falls back to
    // a positional ordinal that differs between the two scan-Vec positions (escaping the eventual
    // fact-identity dedup). Mirrors `module_resolve.rs::descend`'s own `seen_files` guard, closing
    // the identical gap left open here (found on a round-6 adversarial review; see `PROJECT.md`'s
    // Decisions). Keyed on the NAME too, not the file alone: two DIFFERENT declared names that
    // happen to `#[path]`-remap to the identical file (`#[path="s.rs"] mod a;` / `#[path="s.rs"]
    // mod b;`) are two real, separately-compiled modules — already an existing, tested case — and
    // must never collide with each other's own dedup entry.
    let mut seen_files: HashSet<(String, PathBuf)> = HashSet::new();
    for item in items {
        let syn::Item::Mod(module_item) = item else {
            continue;
        };
        let name = strip_raw(&module_item.ident.to_string());
        let child_module = format!("{module}::{name}");
        // An **unconditional** `#[path = "…"]` remap is now *followed* — its file (or inline body)
        // observed — closing the relocated-module coverage gap (its `unsafe` sites / items were
        // previously dropped, a false negative). rustc resolves a non-inline `#[path]` relative to
        // `file_dir` — the directory a `#[path]` in the current position resolves from: the
        // containing file's own dir at file scope, but with each **enclosing inline `mod`** name
        // accumulated onto it (rustc adds the inline-module chain as directory components, so
        // `mod inline { #[path="p.rs"] mod inner; }` in `a.rs` loads `<a.rs child dir>/inline/p.rs`,
        // never `<a.rs dir>/p.rs`). `child_dir` is the conventional-child base and differs from
        // `file_dir` for a non-mod-rs `name.rs`. A `#[path]`-loaded file is itself mod-rs-like, so
        // ITS children resolve from the loaded file's own directory. An inline
        // `#[path = "dir"] mod x { … }` relocates x's base to `<file_dir>/dir` for BOTH its
        // file-children and any `#[path]` nested in its body — so that becomes the body's `file_dir`.
        if let Some(rel) = direct_path_value(&module_item.attrs) {
            match &module_item.content {
                // Inline body relocated by `#[path = "dir"]`: `<file_dir>/dir` is the base for the
                // body's file-children AND any `#[path]` written inside it, so it is the body's
                // `file_dir` too (not the enclosing `file_dir` — the relocation accumulates). The
                // body's own content still lives in the enclosing file, so `current_file` inherits
                // unchanged.
                Some((_, inner)) => {
                    let relocated = file_dir.join(&rel);
                    children.push((
                        inner.clone(),
                        child_module,
                        relocated.clone(),
                        relocated,
                        None,
                        current_file.to_path_buf(),
                    ))
                }
                None => {
                    let file = file_dir.join(&rel);
                    if !file.is_file() {
                        // An unconditional `#[path]` target must exist (rustc errors otherwise), so
                        // an absent one is a genuine broken reference: fail loud (exit 2), never a
                        // silent skip. A cfg-conditional `#[path]` is the `has_path_attr` skip below.
                        return Err(missing_module_file_error(&child_module, crate_package));
                    }
                    let canon = canonicalize_source(&file)?;
                    if ancestors.contains(&canon) {
                        return Err(module_cycle_error(&child_module, crate_package, &file));
                    }
                    if !seen_files.insert((name.clone(), canon.clone())) {
                        continue;
                    }
                    let parsed = read_parse(&file)?;
                    // mod-rs-like: the loaded file's own directory is the base for both its
                    // conventional children and any nested `#[path]` beneath it.
                    let own_dir = file
                        .parent()
                        .map(Path::to_path_buf)
                        .unwrap_or_else(|| file_dir.to_path_buf());
                    children.push((
                        parsed.items,
                        child_module,
                        own_dir.clone(),
                        own_dir,
                        Some(canon),
                        file,
                    ));
                }
            }
            continue;
        }
        // A `cfg_attr`-wrapped `#[path]` is cfg-conditional: following it cfg-blind could read a
        // file rustc does not compile here, so it stays a stated skip bound — and the conventional
        // file is never governed in its place.
        if has_path_attr(&module_item.attrs) {
            continue;
        }
        let sub_dir = child_dir.join(&name);
        match &module_item.content {
            // Inline `mod x { … }`: descend its lexical items (same file). Its own children — both
            // conventional `mod y;` AND any `#[path]` nested in the body — resolve from `<child_dir>/x`
            // (rustc accumulates the inline-module name as a directory component), so that dir is the
            // body's `file_dir` too, NOT the enclosing `file_dir`. Getting this wrong drops a
            // `#[path]` relocated inside an inline block onto the wrong file — a false negative.
            // The body's own content stays in the enclosing file, so `current_file` inherits
            // unchanged.
            Some((_, inner)) => children.push((
                inner.clone(),
                child_module,
                sub_dir.clone(),
                sub_dir,
                None,
                current_file.to_path_buf(),
            )),
            // File `mod x;`: `<dir>/x.rs` or `<dir>/x/mod.rs`; children under `x/`; the child's own
            // `file_dir` is the located file's directory (`<dir>` for `x.rs`, `<dir>/x` for
            // `x/mod.rs`), which is where a `#[path]` inside it resolves from.
            None => match locate_module_file(child_dir, &name) {
                Some(file) => {
                    // A file already on the current descent path (an ANCESTOR, by canonical
                    // symlink-resolved path) is a genuine module cycle — a symlinked directory or a
                    // circular `#[path]` looping the `mod` graph back on itself. Stop with a scan
                    // error (exit 2 "cannot judge") rather than recursing into a stack overflow. Two
                    // *sibling/cousin* declarations legitimately resolving to one file (e.g.
                    // `#[path="s.rs"] mod a; #[path="s.rs"] mod b;`, which rustc compiles) are NOT a
                    // cycle — the ancestor set, unlike a monotonic whole-tree visited set, does not
                    // misreport them (that would be a false positive on compilable input).
                    let canon = canonicalize_source(&file)?;
                    if ancestors.contains(&canon) {
                        return Err(module_cycle_error(&child_module, crate_package, &file));
                    }
                    if !seen_files.insert((name.clone(), canon.clone())) {
                        continue;
                    }
                    let parsed = read_parse(&file)?;
                    let own_dir = file
                        .parent()
                        .map(Path::to_path_buf)
                        .unwrap_or_else(|| sub_dir.clone());
                    children.push((
                        parsed.items,
                        child_module,
                        sub_dir,
                        own_dir,
                        Some(canon),
                        file,
                    ));
                }
                // A `#[cfg]`-gated module may legitimately have no source file when the feature is
                // off (a standard optional-feature pattern) — a stated coverage bound, not a scan
                // error. A non-cfg missing file is a real scan error: fail loud (exit 2).
                None => {
                    if !has_cfg_attr(&module_item.attrs) {
                        return Err(missing_module_file_error(&child_module, crate_package));
                    }
                }
            },
        }
    }
    Ok(children)
}

// `child_dir` and `file_dir` are distinct module-resolution bases (see `resolve_child_modules`), not
// bundled: they thread the descent by position alongside the crate-scan accumulator and guards.
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
    scan: &mut CrateScan,
) -> Result<(), String> {
    let uses = collect_uses(&items);
    // The re-export closure applies the same per-defining-module child-module shadow the direct
    // head oracle does: a bare `pub use dep::X;` / `pub use wc::X;` head named by this module's own
    // child `mod dep` / `mod wc` is not recorded as the dependency / renamed crate, so a
    // cross-module facade reaching it through this crate-wide map does not mis-canonicalize (the
    // facade-closure FP). `collect_reexports` keeps a leading-`::` head on the raw sets.
    let child_mods = child_module_names(&items);
    collect_reexports(
        &items,
        &module,
        externs,
        &child_mods,
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

    for item in &items {
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
                    module: module.clone(),
                    file: current_file.clone(),
                    trait_path: trait_path.clone(),
                    self_ty: (*impl_item.self_ty).clone(),
                    uses: uses.clone(),
                });
            }
            syn::Item::Struct(i) => {
                push_type_def(&i.attrs, &i.ident, &module, &current_file, &uses, scan)?;
            }
            syn::Item::Enum(i) => {
                push_type_def(&i.attrs, &i.ident, &module, &current_file, &uses, scan)?;
            }
            syn::Item::Union(i) => {
                push_type_def(&i.attrs, &i.ident, &module, &current_file, &uses, scan)?;
            }
            // A non-generic `type X = <nominal path>;` alias: record `{module}::X → target`
            // so the exposure pipeline can follow it to the defining path. The target-resolution
            // ladder is byte-identical to the query site's, so no resolvable target is dropped and
            // no local shadow is misread:
            //   0. a leading-`::` target — an unambiguous extern (raw set, with the crate-root
            //      rename applied), a HARD short-circuit, so `type X = ::serde::Value;` records the
            //      extern even under a local `mod serde`, and `type X = ::<rename>::Foo;` too;
            //   1. `resolve_path(Ignore)` — use-map / `crate`·`self`·`super`;
            //   2. `bare_local_alias_target` — a bare single-segment target naming one of THIS
            //      module's own type aliases recorded as `{module}::{name}` (its canonical alias-map
            //      key), tried BEFORE the extern oracle so a local alias shadows a same-named
            //      dependency (rustc's own resolution); the query-time `canonicalize_through_aliases`
            //      fixpoint then closes a *bare* alias-of-an-alias chain regardless of source order.
            //      Gated to local alias names, so a bare non-alias target (a local struct, a std
            //      prelude type like `String`) is never mis-recorded — no false positive;
            //   3. `extern_verbatim_renamed` — an extern head, incl. a crate-root `extern crate as`
            //      rename (the rename map is pre-collected, so this is order-independent).
            // A generic alias (`type X<T> = …`) or a complex target (`Vec<T>`, `&T`, a
            // tuple/`dyn`/`impl`) is skipped — a stated coverage bound, never a silent claim.
            syn::Item::Type(type_item) if type_item.generics.params.is_empty() => {
                // Record the alias's LANDING type — where its target resolves under the same bare-head
                // `CurrentModule` fallback the impl-self check uses — so the forbidden-marker check can
                // react on an alias to a crate-defined subtree type (`type Bar = Real`) yet stay silent
                // on one to a foreign/prelude type (`type Baz = Vec<u8>` / `= String`), whose marker
                // lands off the governed subtree. Only a nominal `Type::Path` target has a single
                // landing type; a tuple/ref/`dyn` target has none and is skipped (never governed here).
                if let syn::Type::Path(tp) = &*type_item.ty {
                    if let Some(landing) =
                        resolve_path(&tp.path, &uses, &module, BareFallback::CurrentModule)
                    {
                        let alias =
                            format!("{module}::{}", strip_raw(&type_item.ident.to_string()));
                        scan.alias_targets.insert(alias, landing);
                    }
                }
                if let Some(target) = alias_nominal_target(&type_item.ty) {
                    let alias = format!("{module}::{}", strip_raw(&type_item.ident.to_string()));
                    let resolved = if target.leading_colon.is_some() {
                        extern_verbatim_renamed(target, externs, &scan.extern_renames)
                    } else {
                        resolve_path(target, &uses, &module, BareFallback::Ignore)
                            .or_else(|| {
                                bare_local_alias_target(target, &module, &local_alias_names)
                            })
                            .or_else(|| {
                                extern_verbatim_renamed(target, &externs_type, &scan.extern_renames)
                            })
                    };
                    if let Some(resolved) = resolved {
                        if resolved != alias {
                            scan.aliases.insert(alias, resolved);
                        }
                    }
                }
            }
            _ => {}
        }
    }

    for (child_items, child_module, sub_dir, sub_file_dir, opened, child_file) in
        resolve_child_modules(
            &items,
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
/// `resolve_child_modules`'s own policy), a `cfg_attr`-wrapped `#[path]` is the actual stated
/// coverage bound (skipped, since following it cfg-blind could read a file rustc does not compile
/// here), a `#[cfg]`-gated fileless module is tolerated, a non-`#[cfg]` missing module file is a
/// scan error (exit 2), and a symlink module cycle is a scan error (exit 2), never a stack
/// overflow.
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
        ancestors.insert(canonicalize_source(&file)?);
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
    out: &mut Vec<(String, Vec<syn::Item>, PathBuf)>,
) -> Result<(), String> {
    for (child_items, child_module, sub_dir, sub_file_dir, opened, child_file) in
        resolve_child_modules(
            &items,
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

fn has_cfg_attr(attrs: &[syn::Attribute]) -> bool {
    attrs.iter().any(|attr| attr.path().is_ident("cfg"))
}

// --- Unsafe-site scan (`semantic-unsafe-confinement`) -------------------------

/// One `unsafe` site observed in the crate: its enclosing (file) module, the real file it was
/// read from (its own branch's file — the same provenance guarantee as [`ImplSite::file`]), and a
/// stable label (`unsafe block`, `unsafe fn decode`, `unsafe impl Send`, `unsafe trait Zeroable`,
/// `unsafe extern block`). The label is module-qualified at the finding layer for injectivity.
pub(crate) struct UnsafeSite {
    pub(crate) module: String,
    pub(crate) file: PathBuf,
    pub(crate) label: String,
}

/// A `syn::visit::Visit` collector recording every executable-`unsafe` **code site** within the
/// items it is fed: `unsafe fn` (free / inherent / trait-decl / trait-impl method), `unsafe impl`,
/// `unsafe trait`, `unsafe extern` block, and `unsafe {}` expression block (deep in bodies). It is
/// fed a module's items **minus top-level `mod`s** (the walk owns their descent); `visit_item_mod`
/// is left at its **default (recursing)** so a `mod` declared *inside a fn/block body* — which the
/// top-level walk never reaches — is still observed, attributed to the enclosing file module.
#[derive(Default)]
struct UnsafeSiteCollector {
    labels: Vec<String>,
    // Positional discriminator for a self type `type_to_string` cannot render (`_#n`), so two such
    // `unsafe impl`s in one module stay distinct findings rather than masking each other.
    unsafe_impl_ordinal: usize,
    // The enclosing `impl`'s self-type / `trait`'s name during the recursion, so an `unsafe fn`
    // method is owner-qualified (`unsafe fn Foo::m`) — else two same-named `unsafe fn`s on
    // different owners in one module collapse to one finding and a baseline of the first masks the
    // second (a false negative), the same injectivity `unsafe impl` already guards.
    current_owner: Option<String>,
    current_trait: Option<String>,
    // The trait of the enclosing *trait `impl`* (`None` for an inherent impl), so a trait-impl
    // `unsafe fn` is qualified by `<trait for self>` — else `impl Foo { unsafe fn m }` and
    // `impl A for Foo { unsafe fn m }` (same self type), or `impl A for Foo` and `impl B for Foo`
    // (same self type, different trait), collapse to one `unsafe fn Foo::m` and a baseline of one
    // masks the other (a false negative). Self-type alone only separates *different* self types.
    current_impl_trait: Option<String>,
}

/// Render a trait path for an `unsafe impl` label — segment idents joined by `::` (raw-stripped),
/// enough to keep two `unsafe impl`s of different traits distinct. No `quote`.
fn render_trait_path(path: &syn::Path) -> String {
    let lead = if path.leading_colon.is_some() {
        "::"
    } else {
        ""
    };
    let segs: Vec<String> = path
        .segments
        .iter()
        .map(|s| strip_raw(&s.ident.to_string()))
        .collect();
    format!("{lead}{}", segs.join("::"))
}

impl<'ast> Visit<'ast> for UnsafeSiteCollector {
    fn visit_expr_unsafe(&mut self, node: &'ast syn::ExprUnsafe) {
        self.labels.push("unsafe block".to_string());
        visit::visit_expr_unsafe(self, node);
    }

    fn visit_item_fn(&mut self, node: &'ast syn::ItemFn) {
        if node.sig.unsafety.is_some() {
            self.labels.push(format!(
                "unsafe fn {}",
                strip_raw(&node.sig.ident.to_string())
            ));
        }
        visit::visit_item_fn(self, node);
    }

    fn visit_impl_item_fn(&mut self, node: &'ast syn::ImplItemFn) {
        if node.sig.unsafety.is_some() {
            let name = strip_raw(&node.sig.ident.to_string());
            // Qualify by the enclosing impl (set in `visit_item_impl`): a *trait* impl by
            // `<trait for self>`, an inherent impl by its self type alone. Self-type alone only
            // separates *different* self types, so a trait-impl method and an inherent (or
            // other-trait) method with the same name on the *same* self type would otherwise
            // collapse to one finding and a baseline of one mask the other (a false negative).
            let label = match (&self.current_impl_trait, &self.current_owner) {
                (Some(tr), Some(owner)) => format!("unsafe fn <{tr} for {owner}>::{name}"),
                (_, Some(owner)) => format!("unsafe fn {owner}::{name}"),
                (_, None) => format!("unsafe fn {name}"),
            };
            self.labels.push(label);
        }
        visit::visit_impl_item_fn(self, node);
    }

    fn visit_trait_item_fn(&mut self, node: &'ast syn::TraitItemFn) {
        if node.sig.unsafety.is_some() {
            let name = strip_raw(&node.sig.ident.to_string());
            // Qualify by the declaring trait (set in `visit_item_trait`), so two traits each
            // declaring `unsafe fn m` in one module do not collapse to one finding.
            let label = match &self.current_trait {
                Some(owner) => format!("unsafe fn {owner}::{name}"),
                None => format!("unsafe fn {name}"),
            };
            self.labels.push(label);
        }
        visit::visit_trait_item_fn(self, node);
    }

    fn visit_item_impl(&mut self, node: &'ast syn::ItemImpl) {
        // Owner-qualify by the implemented-for type so `unsafe impl Send for Foo` and
        // `unsafe impl Send for Bar` in one module stay distinct findings — else a baseline of
        // the first silently masks the second (a false negative). Lexical (`type_to_string`, no
        // resolution — this is the light walk), mirroring the trait-path rendering above. The `_#n`
        // fallback (an unrenderable self type) is consumed only when needed, so two such impls stay
        // distinct. The same owner also qualifies the impl's inner `unsafe fn` methods.
        let owner = type_to_string(&node.self_ty).unwrap_or_else(|| {
            let label = format!("_#{}", self.unsafe_impl_ordinal);
            self.unsafe_impl_ordinal += 1;
            label
        });
        // The implemented trait (if any), rendered once — reused for the `unsafe impl` label and to
        // qualify the impl's inner `unsafe fn` methods as `<trait for self>` (injectivity above).
        let impl_trait = node
            .trait_
            .as_ref()
            .map(|(_, path, _)| render_trait_path(path));
        if node.unsafety.is_some() {
            let label = match &impl_trait {
                Some(tr) => format!("unsafe impl {tr} for {owner}"),
                None => format!("unsafe impl {owner}"),
            };
            self.labels.push(label);
        }
        let prev_owner = self.current_owner.replace(owner);
        let prev_trait = self.current_impl_trait.take();
        self.current_impl_trait = impl_trait;
        visit::visit_item_impl(self, node);
        self.current_owner = prev_owner;
        self.current_impl_trait = prev_trait;
    }

    fn visit_item_trait(&mut self, node: &'ast syn::ItemTrait) {
        let name = strip_raw(&node.ident.to_string());
        if node.unsafety.is_some() {
            self.labels.push(format!("unsafe trait {name}"));
        }
        let prev = self.current_trait.replace(name);
        visit::visit_item_trait(self, node);
        self.current_trait = prev;
    }

    fn visit_item_foreign_mod(&mut self, node: &'ast syn::ItemForeignMod) {
        if node.unsafety.is_some() {
            self.labels.push("unsafe extern block".to_string());
        }
        visit::visit_item_foreign_mod(self, node);
    }
}

/// Walk the whole crate from its root and collect every `unsafe` site with its enclosing module.
/// Mirrors [`scan_crate`]'s descent (file + inline modules, ancestor-path cycle guard → exit 2, an
/// unconditional `#[path]` followed / a `cfg_attr`-wrapped one skipped as a stated bound, a
/// non-`#[cfg]` missing module file → exit 2, a cfg-gated missing file tolerated). A separate,
/// lighter walk than `scan_crate` (no re-export/alias/type-def resolution).
pub(crate) fn scan_unsafe_sites(
    src_dir: &Path,
    root_file: &Path,
    crate_package: &str,
) -> Result<Vec<UnsafeSite>, String> {
    let root = read_parse(root_file)?;
    let mut sites = Vec::new();
    let mut ancestors: HashSet<PathBuf> = HashSet::new();
    ancestors.insert(canonicalize_source(root_file)?);
    walk_unsafe(
        root.items,
        "crate".to_string(),
        src_dir.to_path_buf(),
        // The crate root is mod-rs-like: its own directory is the `#[path]` base too.
        src_dir.to_path_buf(),
        root_file.to_path_buf(),
        crate_package,
        &ancestors,
        &mut sites,
    )?;
    Ok(sites)
}

#[allow(clippy::too_many_arguments)]
fn walk_unsafe(
    items: Vec<syn::Item>,
    module: String,
    child_dir: PathBuf,
    file_dir: PathBuf,
    current_file: PathBuf,
    crate_package: &str,
    ancestors: &HashSet<PathBuf>,
    sites: &mut Vec<UnsafeSite>,
) -> Result<(), String> {
    // Feed the collector this module's items minus top-level `mod`s (walk-owned); body-nested
    // `mod`s stay in and are caught by the collector's default `visit_item_mod` recursion.
    let mut collector = UnsafeSiteCollector::default();
    for item in &items {
        if matches!(item, syn::Item::Mod(_)) {
            continue;
        }
        collector.visit_item(item);
    }
    for label in collector.labels {
        sites.push(UnsafeSite {
            module: module.clone(),
            file: current_file.clone(),
            label,
        });
    }

    for (child_items, child_module, sub_dir, sub_file_dir, opened, child_file) in
        resolve_child_modules(
            &items,
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
                walk_unsafe(
                    child_items,
                    child_module,
                    sub_dir,
                    sub_file_dir,
                    child_file,
                    crate_package,
                    &child_ancestors,
                    sites,
                )?;
            }
            None => walk_unsafe(
                child_items,
                child_module,
                sub_dir,
                sub_file_dir,
                child_file,
                crate_package,
                ancestors,
                sites,
            )?,
        }
    }
    Ok(())
}
