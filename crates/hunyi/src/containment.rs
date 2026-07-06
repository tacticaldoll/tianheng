//! `::`-delimited path containment and leaf/self-type helpers shared by every capability's
//! subtree / forbidden / allowed test — the single home of the containment rule, so no copy
//! drifts to a bare `starts_with` that would admit a sibling (a false positive on the allowed
//! side, a false negative on the forbidden side). Name resolution itself lives in
//! [`crate::resolve`].

use crate::resolve::{BareFallback, UseMap, resolve_path, strip_raw};

/// Sibling-safe `::`-path containment: `path` equals `prefix` or sits strictly beneath it
/// (`crate::a` contains `crate::a::b`, never the sibling `crate::ab`). The single home of the
/// containment rule every capability's subtree/forbidden/allowed test shares, so no copy can
/// drift to a bare `starts_with` that would admit a sibling — a false positive on the allowed
/// side, a false negative on the forbidden side.
fn path_within(path: &str, prefix: &str) -> bool {
    path == prefix || path.starts_with(&format!("{prefix}::"))
}

/// A canonical path is under `subtree` — [`path_within`] read with subtree-containment naming at
/// the call site (`crate::a` contains `crate::a::b`, never the sibling `crate::ab`).
pub(crate) fn under_subtree(canonical: &str, subtree: &str) -> bool {
    path_within(canonical, subtree)
}

/// The leaf identifier of a `::`-delimited path string, raw-canonicalized (`r#Trait` → `Trait`) so
/// a declared marker written with a raw identifier compares equal to the observed [`path_leaf`],
/// which strips it. (Trait names are never keywords, so this is defensive symmetry, not a live gap.)
pub(crate) fn leaf_of(path: &str) -> &str {
    let leaf = path.rsplit("::").next().unwrap_or(path);
    leaf.strip_prefix("r#").unwrap_or(leaf)
}

/// The leaf identifier of a `syn::Path` (raw-canonicalized).
pub(crate) fn path_leaf(path: &syn::Path) -> String {
    path.segments
        .last()
        .map(|s| strip_raw(&s.ident.to_string()))
        .unwrap_or_default()
}

/// Resolve an `impl`'s self-type to the canonical path of its definition, or `None` when it
/// is not a placeable nominal path (a reference/tuple/complex shape — a stated bound). For a
/// `Type::Path` (incl. a generic `Wrapper<T>`, governed by the outer `Wrapper`), the leading
/// path resolves via the impl module's `use`s / current-module / re-exports.
pub(crate) fn resolve_self_type(
    self_ty: &syn::Type,
    uses: &UseMap,
    module: &str,
) -> Option<String> {
    match self_ty {
        syn::Type::Path(tp) => resolve_path(&tp.path, uses, module, BareFallback::CurrentModule),
        _ => None,
    }
}

/// `::`-delimited containment: a canonical path is forbidden when it equals a forbidden
/// entry or sits beneath it (so `crate::infra` matches `crate::infra::db::Pool` but never
/// the sibling `crate::infrastructure`).
pub(crate) fn matches_forbidden(canonical: &str, forbidden: &[String]) -> bool {
    forbidden.iter().any(|entry| path_within(canonical, entry))
}

/// `::`-delimited containment at allowed-vs-location polarity: a module location is
/// allowed when it equals an allowed entry or sits beneath it (so `crate::commands`
/// allows `crate::commands::greet` but never the sibling `crate::commandeer`).
pub(crate) fn matches_allowed(location: &str, allowed: &[String]) -> bool {
    allowed.iter().any(|entry| path_within(location, entry))
}
