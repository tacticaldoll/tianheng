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
///
/// Transparent-macro arms are flattened so platform-branching renames are observed.
fn collect_crate_root_extern_renames(items: &[syn::Item], out: &mut ExternRenameMap) {
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
///
/// Pre-collects crate-root `extern crate X as Y;` renames before the walk to eliminate
/// forward-reference order hazards. Traverses modules with ancestor path tracking to detect
/// cyclic module loops without stack overflowing.
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
    collect_crate_root_extern_renames(&root.items, &mut scan.extern_renames);
    let mut ancestors: HashSet<PathBuf> = HashSet::new();
    ancestors.insert(xingbiao::canonicalize_or_fail(root_file)?);
    walk_module(
        root.items,
        "crate".to_string(),
        src_dir.to_path_buf(),
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
/// `(items, module path, child_dir, file_dir, opened_file, current_file)` for one resolved child.
///
/// `child_dir` is the base for conventional child modules; `file_dir` is the base for `#[path]` remaps;
/// `opened_file` is the canonical path of a new source file opened by this child (`None` for inline bodies);
/// `current_file` is the literal path of the file containing the child's items.
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
        Some((_, inner)) => {
            let bases: Vec<PathBuf> = cfg_attr_targets
                .iter()
                .map(|rel| file_dir.join(rel))
                .chain(std::iter::once(sub_dir.clone()))
                .collect();
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
        None => {
            let mut has_backing_source = false;
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
                ModuleFile::Ambiguous { flat, nested } => {
                    return Err(dual_backed_module_error(
                        child_module,
                        name,
                        crate_package,
                        &flat,
                        &nested,
                    ));
                }
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
    let mut seen_files: HashSet<(String, PathBuf)> = HashSet::new();
    for flat in items {
        let syn::Item::Mod(module_item) = &flat.item else {
            continue;
        };
        let name = strip_raw(&module_item.ident.to_string());
        let child_module = format!("{module}::{name}");
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

/// Record this module's own facts into `scan` from a single flattened items pass: trait
/// definitions, trait-impl sites, type definitions, and resolvable type-alias targets (including
/// the forbidden-marker alias-landing map). Pulled out of `walk_module` as the "this module's own
/// observation" phase, distinct from the child-descent phase that follows it.
///
/// Records non-generic type alias targets and landing types for marker containment,
/// resolving leading `::`, use-map, bare local aliases, and extern crates.
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
            syn::Item::Type(type_item) => {
                if !type_item.generics.params.is_empty() {
                    continue;
                }
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

/// Walk one module: flatten transparent macros, collect re-exports with per-defining-module
/// child-module shadowing, record module facts, and recurse into child modules.
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
    let (items, flat) = flatten_for_walk(&items);
    let uses = collect_uses(&items);
    let child_mod_decls = child_module_decls(&flat);
    collect_reexports(
        &flat,
        &module,
        externs,
        &child_mod_decls,
        &scan.extern_renames,
        &mut scan.reexports,
    );
    let externs_type: HashSet<String> = externs
        .difference(&local_type_namespace_names(&items))
        .cloned()
        .collect();
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
        let mut ancestors: HashSet<PathBuf> = HashSet::new();
        ancestors.insert(xingbiao::canonicalize_or_fail(&file)?);
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
///
/// **Both names are read through [`crate::syn_util::is_builtin_name`]**, so a raw-identifier
/// spelling is the built-in it spells. Measured against rustc 1.96.0, edition 2021,
/// `--crate-type lib`: `#[r#derive(Clone)]`, `#[r#cfg_attr(unix, derive(Clone))]` and
/// `#[cfg_attr(unix, r#derive(Clone))]` each apply the derive. Compared as written, all three
/// were invisible — and a derive this reader does not see is a marker `forbidden_marker` cannot
/// refuse, which is the false negative the Core Contract forbids. This reader was outside the
/// sweep that closed the same spelling for `path` and `cfg` because that sweep took one file as
/// its corpus and the claim was about the crate.
fn extract_derives(attrs: &[syn::Attribute]) -> Result<Vec<syn::Path>, String> {
    let mut out = Vec::new();
    for attr in attrs {
        if crate::syn_util::is_builtin_name(attr.path(), "derive") {
            out.extend(parse_derive_paths(&attr.meta)?);
        } else if crate::syn_util::is_builtin_name(attr.path(), "cfg_attr") {
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
            if crate::syn_util::is_builtin_name(&list.path, "derive") {
                out.extend(parse_derive_paths(meta)?);
            } else if crate::syn_util::is_builtin_name(&list.path, "cfg_attr") {
                let inner = list
                    .parse_args_with(meta_list_parser())
                    .map_err(|e| format!("cannot parse nested #[cfg_attr(...)]: {e}"))?;
                extract_derives_from_cfg_metas(&inner, out)?;
            }
        }
    }
    Ok(())
}
