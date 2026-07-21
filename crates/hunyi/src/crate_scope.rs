use std::collections::HashSet;
use std::path::Path;

use serde_json::Value;

use crate::resolve::{
    BareFallback, ExternRenameMap, ReexportMap, UseMap, apply_bare_alias_rename,
    apply_crate_root_rename, canonicalize_through_reexports, extern_verbatim_renamed,
    renames_shadowed, resolve_path, strip_raw,
};
use crate::scan::scan_crate;

/// The sysroot crates: valid extern path heads that never appear in a package's declared
/// `dependencies`, so the external-crate set includes them explicitly (forbidding e.g.
/// `std::process` at a facade seam is legitimate intent, not a lint).
const SYSROOT_CRATES: [&str; 5] = ["std", "core", "alloc", "proc_macro", "test"];

/// The names a crate's declared dependencies are written under **in source**: each
/// dependency's `rename` when present (a Cargo `pkg = { package = "…" }` rename), else its
/// package `name`, normalized `-`→`_` to the Rust path spelling (`async-trait` →
/// `async_trait`). Read from the `cargo metadata --no-deps` package — declared-manifest data,
/// no resolved graph, no network.
pub(crate) fn dependency_names(package: &Value) -> Vec<String> {
    package["dependencies"]
        .as_array()
        .map(|deps| {
            deps.iter()
                .filter_map(|dep| {
                    dep["rename"]
                        .as_str()
                        .or_else(|| dep["name"].as_str())
                        .map(|name| name.replace('-', "_"))
                })
                .collect()
        })
        .unwrap_or_default()
}

/// The **raw** external-crate name set: declared dependency names (already `-`→`_` normalized,
/// rename-aware) ∪ the sysroot crates.
pub(crate) fn external_crate_set(dep_names: &[String]) -> HashSet<String> {
    dep_names
        .iter()
        .cloned()
        .chain(SYSROOT_CRATES.iter().map(|s| s.to_string()))
        .collect()
}

/// The governed module's own **type-namespace** item names — `mod`, `struct`, `enum`, `union`,
/// `trait`, and `type` alias declarations.
pub(crate) fn local_type_namespace_names(items: &[syn::Item]) -> HashSet<String> {
    items
        .iter()
        .filter_map(|item| match item {
            syn::Item::Mod(m) => Some(strip_raw(&m.ident.to_string())),
            syn::Item::Struct(s) => Some(strip_raw(&s.ident.to_string())),
            syn::Item::Enum(e) => Some(strip_raw(&e.ident.to_string())),
            syn::Item::Union(u) => Some(strip_raw(&u.ident.to_string())),
            syn::Item::Trait(t) => Some(strip_raw(&t.ident.to_string())),
            syn::Item::Type(t) => Some(strip_raw(&t.ident.to_string())),
            _ => None,
        })
        .collect()
}

/// The governed module's own child-**module** names.
pub(crate) fn child_module_names(items: &[syn::Item]) -> HashSet<String> {
    items
        .iter()
        .filter_map(|item| match item {
            syn::Item::Mod(m) => Some(strip_raw(&m.ident.to_string())),
            _ => None,
        })
        .collect()
}

/// The CRATE-WIDE extern-resolution context an operand principal-trait match needs: the extern
/// name set and the crate-wide re-export/rename maps. Deliberately excludes anything derived from
/// a specific module's own items (that's [`FileExternScope`], computed per FILE by the caller —
/// see its own doc for why a #[cfg]-split module's several branches must never share one).
pub(crate) struct ExternResolution {
    pub(crate) externs: HashSet<String>,
    pub(crate) reexports: ReexportMap,
    pub(crate) extern_renames: ExternRenameMap,
}

pub(crate) fn extern_resolution(
    src_dir: &Path,
    root_file: &Path,
    crate_package: &str,
    dep_names: &[String],
) -> Result<ExternResolution, String> {
    let externs = external_crate_set(dep_names);
    let scan = scan_crate(src_dir, root_file, crate_package, &externs)?;
    Ok(ExternResolution {
        externs,
        reexports: scan.reexports,
        extern_renames: scan.extern_renames,
    })
}

/// The PER-FILE extern-resolution context an operand principal-trait match needs: derived from
/// exactly one `#[cfg]`-branch's own items, never a cross-branch union. Two mutually-exclusive
/// `#[cfg]` branches are never compiled together, so a shadow set (a child-module name, hence a
/// shrunk extern/rename map) derived from one branch's items must never suppress resolution for a
/// DIFFERENT, mutually-exclusive branch's own principal path (a confirmed false negative, found on
/// a round-7 adversarial review; see `PROJECT.md`'s Decisions — the identical conflation class
/// round 6 fixed for the `use`-map and `module_findings`'s own `externs_type`/`renames_bare`).
pub(crate) struct FileExternScope {
    pub(crate) externs_type: HashSet<String>,
    /// The crate-root rename map with aliases shadowed by a same-named child `mod` **of this
    /// file's own branch** removed — used for a **bare** head, exactly as `module_findings` does
    /// (see [`renames_shadowed`]).
    pub(crate) renames_bare: ExternRenameMap,
}

pub(crate) fn file_extern_scope(
    res: &ExternResolution,
    file_items: &[syn::Item],
) -> FileExternScope {
    let externs_type = res
        .externs
        .difference(&local_type_namespace_names(file_items))
        .cloned()
        .collect();
    let renames_bare = renames_shadowed(&res.extern_renames, &child_module_names(file_items));
    FileExternScope {
        externs_type,
        renames_bare,
    }
}

/// Resolve a shape's **principal-trait** path through the same extern-aware ladder
/// signature-coupling uses for an exposed type, minus the type-alias closure. To match
/// `module_findings` exactly: a bare head uses the child-module-shadowed rename map
/// ([`FileExternScope::renames_bare`]) while a leading-`::` head uses the full `extern_renames`;
/// and after the re-export closure both the crate-relative spelling (`crate::Y::T`, via
/// [`apply_crate_root_rename`]) and the bare spelling (`Y::T` from a private `use Y::…;`, via
/// [`apply_bare_alias_rename`] with the child-shadowed map) of a crate-root `extern crate … as`
/// rename are rewritten, so every alias spelling reacts alike (the specs' declared "same resolver
/// ladder … with a crate-root rename applied"). `file_scope` MUST be the branch that OWNS `path`
/// (the exposure's own file), never a different branch's scope.
pub(crate) fn resolve_principal(
    path: &syn::Path,
    uses: &UseMap,
    module: &str,
    res: &ExternResolution,
    file_scope: &FileExternScope,
) -> Option<String> {
    let resolved = if path.leading_colon.is_some() {
        extern_verbatim_renamed(path, &res.externs, &res.extern_renames)
    } else {
        resolve_path(path, uses, module, BareFallback::Ignore).or_else(|| {
            extern_verbatim_renamed(path, &file_scope.externs_type, &file_scope.renames_bare)
        })
    };
    resolved.map(|canonical| {
        let canonical = canonicalize_through_reexports(&canonical, &res.reexports);
        let canonical = apply_crate_root_rename(canonical, &res.extern_renames);
        apply_bare_alias_rename(canonical, &file_scope.renames_bare)
    })
}
