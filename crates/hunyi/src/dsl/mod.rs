//! Declaration DSL for 渾儀's semantic boundaries — the builder types each capability
//! exposes (`SignatureBoundary`, `TraitImplBoundary`, `VisibilityBoundary`,
//! `ForbiddenMarkerBoundary`, `DynTraitBoundary`, `ImplTraitBoundary`,
//! `AsyncExposureBoundary`) and their crate/module/boundary draft chains. Pure data and
//! builders — no scan, no resolution, no reaction — re-exported from the crate root so the
//! public paths (`hunyi::SignatureBoundary`, …) stay unchanged. One module per capability
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

/// The 5 accessor/builder methods every boundary family's `$boundary`/`$draft` pair defines
/// byte-identically: `$boundary::crate_package/with_anchor/anchor/severity`, and
/// `$draft::warn`. Each family keeps its own rule-specific fields, transition verbs, and
/// multiplicity methods separate — only this shared accessor surface is factored.
macro_rules! boundary_common {
    ($boundary:ty, $draft:ty) => {
        impl $boundary {
            /// The crate this boundary governs.
            pub fn crate_package(&self) -> &str {
                &self.crate_package
            }

            /// Attach a durable governance anchor (e.g. `"ADR-014"`) — a stable pointer into the
            /// project's governance, distinct from the free-text `reason`. Optional; a boundary
            /// with none projects and reacts exactly as before.
            pub fn with_anchor(mut self, anchor: &str) -> Self {
                self.anchor = Some(anchor.to_string());
                self
            }

            /// The durable governance anchor recorded with the boundary, if any.
            pub fn anchor(&self) -> Option<&str> {
                self.anchor.as_deref()
            }

            /// The boundary's severity (`enforce` or `warn`).
            pub fn severity(&self) -> xuanji::Severity {
                self.severity
            }
        }

        impl $draft {
            /// Make this an advisory (`warn`) boundary: violations are reported but do not fail
            /// the reaction — the first rung of adoption.
            pub fn warn(mut self) -> Self {
                self.severity = xuanji::Severity::Warn;
                self
            }
        }
    };
}
pub(crate) use boundary_common;

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
