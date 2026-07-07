//! `::`-delimited path containment and leaf/self-type helpers shared by every capability's
//! subtree / forbidden / allowed test — the single home of the containment rule, so no copy
//! drifts to a bare `starts_with` that would admit a sibling (a false positive on the allowed
//! side, a false negative on the forbidden side). Name resolution itself lives in
//! [`crate::resolve`].

use std::collections::{HashMap, HashSet};

use crate::resolve::{
    BareFallback, ReexportMap, UseMap, canonicalize_through_reexports, resolve_path, strip_raw,
};

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

/// Resolve an `impl`'s self-type to the canonical path of the type it **lands on** — its
/// definition — or `None` when it is not a placeable nominal path (a reference/tuple/complex
/// shape — a stated bound). For a `Type::Path` (incl. a generic `Wrapper<T>`, governed by the
/// outer `Wrapper`), the leading path resolves via the impl module's `use`s / current-module,
/// then is followed through the re-export (`pub use` facade) and type-alias closures to the
/// definition it denotes: `impl M for crate::facade::T` where `crate::facade` re-exports `T`, and
/// `impl M for Bar` where `type Bar = Real`, both land on the real definition (to coherence a
/// re-export/alias denotes the same type, so the marker genuinely lands there).
///
/// The canonicalization is folded in **here** so a self-type is canonical *by construction*: a
/// caller cannot resolve a self-type and forget to close the re-export/alias hop (the sibling
/// capabilities' shared-canonicalizer discipline, made structural at the one self-type resolver).
/// The two maps are interleaved to a fixpoint — a re-export of an alias, or an alias of a
/// re-export, both terminate (each distinct path is visited once). `alias_targets` carries the
/// `CurrentModule`-fallback landing, so an alias to a bare local struct (`type Bar = Real`) is
/// caught — which the `Ignore`-built exposure alias map deliberately does not, the reason this is
/// not the exposure canonicalizer. A defining path is never a key in either map (an alias/re-export
/// name cannot clash with a definition in its module), so the fixpoint never over-follows past a
/// definition — dropping the old inline `defined`-stop changes no landing.
pub(crate) fn resolve_self_type(
    self_ty: &syn::Type,
    uses: &UseMap,
    module: &str,
    alias_targets: &HashMap<String, String>,
    reexports: &ReexportMap,
) -> Option<String> {
    let base = match self_ty {
        syn::Type::Path(tp) => resolve_path(&tp.path, uses, module, BareFallback::CurrentModule)?,
        _ => return None,
    };
    let mut landing = base;
    let mut seen = HashSet::new();
    while seen.insert(landing.clone()) {
        let via_reexport = canonicalize_through_reexports(&landing, reexports);
        if via_reexport != landing {
            landing = via_reexport;
            continue;
        }
        if let Some(next) = alias_targets.get(&landing) {
            landing = next.clone();
            continue;
        }
        break;
    }
    Some(landing)
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
