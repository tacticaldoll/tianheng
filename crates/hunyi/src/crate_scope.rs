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

/// The extern-resolution context an operand principal-trait match needs.
pub(crate) struct ExternResolution {
    pub(crate) externs: HashSet<String>,
    pub(crate) externs_type: HashSet<String>,
    pub(crate) reexports: ReexportMap,
    pub(crate) extern_renames: ExternRenameMap,
    /// The crate-root rename map with aliases shadowed by a same-named child `mod` removed —
    /// used for a **bare** head, exactly as `module_findings` does (see [`renames_shadowed`]).
    pub(crate) renames_bare: ExternRenameMap,
}

pub(crate) fn extern_resolution(
    src_dir: &Path,
    root_file: &Path,
    crate_package: &str,
    dep_names: &[String],
    items: &[syn::Item],
) -> Result<ExternResolution, String> {
    let externs = external_crate_set(dep_names);
    let externs_type: HashSet<String> = externs
        .difference(&local_type_namespace_names(items))
        .cloned()
        .collect();
    let scan = scan_crate(src_dir, root_file, crate_package, &externs)?;
    let renames_bare = renames_shadowed(&scan.extern_renames, &child_module_names(items));
    Ok(ExternResolution {
        externs,
        externs_type,
        reexports: scan.reexports,
        extern_renames: scan.extern_renames,
        renames_bare,
    })
}

/// Resolve a shape's **principal-trait** path through the same extern-aware ladder
/// signature-coupling uses for an exposed type, minus the type-alias closure. To match
/// `module_findings` exactly: a bare head uses the child-module-shadowed rename map
/// ([`ExternResolution::renames_bare`]) while a leading-`::` head uses the full `extern_renames`;
/// and after the re-export closure both the crate-relative spelling (`crate::Y::T`, via
/// [`apply_crate_root_rename`]) and the bare spelling (`Y::T` from a private `use Y::…;`, via
/// [`apply_bare_alias_rename`] with the child-shadowed map) of a crate-root `extern crate … as`
/// rename are rewritten, so every alias spelling reacts alike (the specs' declared "same resolver
/// ladder … with a crate-root rename applied").
pub(crate) fn resolve_principal(
    path: &syn::Path,
    uses: &UseMap,
    module: &str,
    res: &ExternResolution,
) -> Option<String> {
    let resolved = if path.leading_colon.is_some() {
        extern_verbatim_renamed(path, &res.externs, &res.extern_renames)
    } else {
        resolve_path(path, uses, module, BareFallback::Ignore)
            .or_else(|| extern_verbatim_renamed(path, &res.externs_type, &res.renames_bare))
    };
    resolved.map(|canonical| {
        let canonical = canonicalize_through_reexports(&canonical, &res.reexports);
        let canonical = apply_crate_root_rename(canonical, &res.extern_renames);
        apply_bare_alias_rename(canonical, &res.renames_bare)
    })
}
