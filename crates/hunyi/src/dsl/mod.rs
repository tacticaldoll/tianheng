//! Declaration DSL for 渾儀's semantic boundaries — the builder types each capability
//! exposes (`SemanticBoundary`, `TraitImplBoundary`, `VisibilityBoundary`,
//! `ForbiddenMarkerBoundary`, `DynTraitBoundary`, `ImplTraitBoundary`,
//! `AsyncExposureBoundary`) and their crate/module/boundary draft chains. Pure data and
//! builders — no scan, no resolution, no reaction — re-exported from the crate root so the
//! public paths (`hunyi::SemanticBoundary`, …) stay unchanged. One module per capability
//! family; each family is self-contained (its own draft chain, sharing only `xuanji::Severity`).

mod async_exposure;
mod dyn_trait;
mod forbidden_marker;
mod impl_trait;
mod signature;
mod trait_impl;
mod visibility;

pub use async_exposure::*;
pub use dyn_trait::*;
pub use forbidden_marker::*;
pub use impl_trait::*;
pub use signature::*;
pub use trait_impl::*;
pub use visibility::*;
