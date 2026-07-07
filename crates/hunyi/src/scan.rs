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

use crate::crate_scope::{child_module_names, local_type_namespace_names};
use crate::errors::missing_module_file_error;
use crate::module_resolve::{locate_module_file, read_parse};
use crate::resolve::{
    AliasMap, BareFallback, ExternRenameMap, ReexportMap, UseMap, alias_nominal_target,
    collect_reexports, collect_uses, extern_verbatim_renamed, resolve_path, strip_raw,
};
use crate::syn_util::has_path_attr;

/// One impl site observed in the crate: its enclosing module path, the written trait
/// path, the implemented-for type, and that module's `use`-map (for resolution).
pub(crate) struct ImplSite {
    pub(crate) module: String,
    pub(crate) trait_path: syn::Path,
    pub(crate) self_ty: syn::Type,
    pub(crate) uses: UseMap,
}

/// One type definition observed in the crate: its canonical path (`module::Name`), the module
/// it is defined in (for a forbidden-`derive` finding's source file), the paths in its
/// `#[derive(...)]`/`#[cfg_attr(_, derive(...))]`, and that module's `use`-map (so a renamed
/// derive macro, `use serde::Serialize as Ser; #[derive(Ser)]`, resolves to its true leaf).
pub(crate) struct TypeDef {
    pub(crate) canonical: String,
    pub(crate) module: String,
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
    if target.leading_colon.is_some() || target.segments.len() != 1 {
        return None;
    }
    let seg = &target.segments[0];
    if !matches!(seg.arguments, syn::PathArguments::None) {
        return None;
    }
    let name = strip_raw(&seg.ident.to_string());
    local_alias_names
        .contains(&name)
        .then(|| format!("{module}::{name}"))
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
    // already-visited canonical file is that cycle: "cannot judge" (exit 2), never a crash. The
    // louke probe scanner guards the same hazard; the two dimensions keep parallel copies (三儀 ⊥
    // 三儀). Seeded with the crate root so a submodule symlinking back to it is caught too.
    let mut visited: HashSet<PathBuf> = HashSet::new();
    visited.insert(canonicalize_source(root_file)?);
    walk_module(
        root.items,
        "crate".to_string(),
        src_dir.to_path_buf(),
        crate_package,
        externs,
        &mut visited,
        &mut scan,
    )?;
    Ok(scan)
}

/// Canonicalize a source file path (resolving symlinks) for the visited-set cycle guard; an
/// unresolvable path is a scan error ("cannot judge"), never a silent skip.
fn canonicalize_source(file: &Path) -> Result<PathBuf, String> {
    std::fs::canonicalize(file).map_err(|err| {
        format!(
            "cannot canonicalize source file '{}': {err}",
            file.display()
        )
    })
}

fn walk_module(
    items: Vec<syn::Item>,
    module: String,
    child_dir: PathBuf,
    crate_package: &str,
    externs: &HashSet<String>,
    visited: &mut HashSet<PathBuf>,
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
                    trait_path: trait_path.clone(),
                    self_ty: (*impl_item.self_ty).clone(),
                    uses: uses.clone(),
                });
            }
            syn::Item::Struct(i) => {
                push_type_def(&i.attrs, &i.ident, &module, &uses, scan)?;
            }
            syn::Item::Enum(i) => {
                push_type_def(&i.attrs, &i.ident, &module, &uses, scan)?;
            }
            syn::Item::Union(i) => {
                push_type_def(&i.attrs, &i.ident, &module, &uses, scan)?;
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

    for item in items {
        if let syn::Item::Mod(module_item) = item {
            // A `#[path]`-remapped module is located off the conventional path; not
            // observed (a stated coverage bound), never a silent claim of cleanliness.
            if has_path_attr(&module_item.attrs) {
                continue;
            }
            let name = strip_raw(&module_item.ident.to_string());
            let child_module = format!("{module}::{name}");
            match module_item.content {
                // Inline `mod x { … }`: descend its lexical items; file-children under `x/`.
                Some((_, inner)) => {
                    walk_module(
                        inner,
                        child_module,
                        child_dir.join(&name),
                        crate_package,
                        externs,
                        visited,
                        scan,
                    )?;
                }
                // File `mod x;`: `<dir>/x.rs` or `<dir>/x/mod.rs`; children under `x/`.
                None => match locate_module_file(&child_dir, &name) {
                    Some(file) => {
                        // A file already visited (by canonical, symlink-resolved path) is a module
                        // cycle — a symlinked directory looping the `mod` graph back on itself.
                        // Stop with a scan error (exit 2 "cannot judge") rather than recursing
                        // forever into a stack overflow.
                        if !visited.insert(canonicalize_source(&file)?) {
                            return Err(format!(
                                "cannot judge module '{child_module}' in package '{crate_package}': \
                                 its source file '{}' forms a module cycle (a symlink loop)",
                                file.display()
                            ));
                        }
                        let parsed = read_parse(&file)?;
                        walk_module(
                            parsed.items,
                            child_module,
                            child_dir.join(&name),
                            crate_package,
                            externs,
                            visited,
                            scan,
                        )?;
                    }
                    // A `#[cfg]`-gated module may legitimately have no source file when the
                    // feature is off (a standard optional-feature pattern) — a stated
                    // coverage bound, not a scan error. A non-cfg missing file is a real
                    // scan error: fail loud (exit 2), never a silent pass.
                    None => {
                        if !has_cfg_attr(&module_item.attrs) {
                            return Err(missing_module_file_error(&child_module, crate_package));
                        }
                    }
                },
            }
        }
    }
    Ok(())
}

/// Record a type definition with its derive paths into the scan.
fn push_type_def(
    attrs: &[syn::Attribute],
    ident: &syn::Ident,
    module: &str,
    uses: &UseMap,
    scan: &mut CrateScan,
) -> Result<(), String> {
    let name = strip_raw(&ident.to_string());
    let derives = extract_derives(attrs)?;
    scan.type_defs.push(TypeDef {
        canonical: format!("{module}::{name}"),
        module: module.to_string(),
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
