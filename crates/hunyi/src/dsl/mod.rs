//! Declaration DSL for 渾儀's semantic boundaries — the builder types each capability
//! exposes (`SemanticBoundary`, `TraitImplBoundary`, `VisibilityBoundary`,
//! `ForbiddenMarkerBoundary`, `DynTraitBoundary`, `ImplTraitBoundary`,
//! `AsyncExposureBoundary`) and their crate/module/boundary draft chains. Pure data and
//! builders — no scan, no resolution, no reaction — re-exported from the crate root so the
//! public paths (`hunyi::SemanticBoundary`, …) stay unchanged. One module per capability
//! family; each family is self-contained, sharing only the small canonical path/set encoding used
//! by their `xuanji::RuleKey` values.

mod async_exposure;
mod dyn_trait;
mod forbidden_marker;
mod impl_trait;
mod signature;
mod trait_impl;
mod unsafe_confinement;
mod visibility;

pub use async_exposure::*;
pub use dyn_trait::*;
pub use forbidden_marker::*;
pub use impl_trait::*;
pub use signature::*;
pub use trait_impl::*;
pub use unsafe_confinement::*;
pub use visibility::*;

fn canonical_path(value: &str) -> String {
    value
        .split("::")
        .map(|segment| segment.strip_prefix("r#").unwrap_or(segment))
        .collect::<Vec<_>>()
        .join("::")
}

fn canonical_path_set(values: &[String]) -> String {
    let mut values: Vec<String> = values.iter().map(|value| canonical_path(value)).collect();
    values.sort_unstable();
    values.dedup();
    serde_json::to_string(&values).expect("a list of canonical paths always serializes")
}
