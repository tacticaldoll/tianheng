//! 渾儀 (Húnyí) — the **semantic** observation dimension of Tianheng.
//!
//! Where the gnomon 圭表 observes *imports*, 渾儀 observes *meaning* via the AST (`syn`):
//! does a module's **public API expose** a forbidden type? That is the complement of
//! import-governance — a type imported for internal use is fine, but a type named in a `pub`
//! signature or alias chain is observed.
//!
//! Declare a [`SignatureBoundary`] in Rust, [`check`] it against a Cargo workspace, and get
//! an [`Outcome`]. The heavy `syn` parser is quarantined to this crate, keeping the functional
//! core dependency-light (`crates/shengmo/src/law.rs`).
//!
//! Govern by reaction, not instruction.
//!
//! **Layout.** Each semantic capability is a self-contained reaction module
//! (`check_<cap>` → `check_<cap>_boundary` → `<cap>_findings`); [`check_all`] composes the eight
//! with a single `cargo metadata` read. The shared reaction spine lives in the `driver` module
//! and the canonical rule labels in `rules`, below every capability so none depends on another.

#![forbid(unsafe_code)]
#![deny(missing_docs)]

use std::path::Path;

use serde_json::Value;

// The reaction model is the shared 璇璣 crate, re-exported so a consumer can stay on
// hunyi's surface; these names are also used internally below.
pub use xuanji::{
    Baseline, BoundDecl, BoundId, BoundaryKind, Defence, Demonstrates, Extent, FactGranularity,
    Finding, Observer, Outcome, Owner, Polarity, Reached, Report, RuleKey, ScanDepth, Severity,
    StructuredFactIdentity, Violation, ViolationId, apply_baseline,
};

mod bounds;
pub use bounds::observation_bounds;

mod observer;
pub use observer::SemanticObserver;

mod dsl;
pub use dsl::*;

// The canonical rule labels — one source per rule, re-exported so the 天衡 shell's `list`
// projections stay on hunyi's surface (`hunyi::SIGNATURE_RULE`, …).
mod rules;
pub use rules::*;

// The cargo-metadata reads live in 星表 (`xingbiao`), the shared substrate below the 三儀 — the
// static and semantic dimensions read the workspace through one reader, not two drifting twins.

// Already-decomposed helper substrates: resolution, collection, scanning, emission, errors, …
mod collect;
mod containment;
mod crate_scope;
mod driver;
mod emit;
mod errors;
mod file_scope;
mod finding;
mod module_resolve;
mod resolve;
mod scan;
mod shape_scan;
mod syn_util;

// The eight semantic capabilities, each a self-contained reaction (check → check_boundary →
// findings). Their public `check_*` entries and crate-internal `*_findings` hearts are
// re-exported at the crate root so both the shell and the tests keep their existing paths.
mod async_exposure;
mod dyn_trait;
mod exposure;
mod forbidden_marker;
mod impl_trait;
mod trait_impl;
mod unsafe_confinement;
mod visibility;

pub use async_exposure::check_async_exposure;
pub use dyn_trait::check_dyn_trait;
pub use exposure::check;
pub use forbidden_marker::check_forbidden_marker;
pub use impl_trait::check_impl_trait;
pub use trait_impl::check_trait_impl_locality;
pub use unsafe_confinement::check_unsafe_confinement;
pub use visibility::check_visibility;

// The pure-heart `*_findings` entries stay crate-internal; the test suite pulls the crate root via
// `use super::*`, so re-export them here for tests only (they are called in-module by each
// capability's `check_*_boundary`, so a non-test build never reaches them through the root).
#[cfg(test)]
pub(crate) use async_exposure::{async_exposure_module_findings, async_exposure_subtree_findings};
#[cfg(test)]
pub(crate) use dyn_trait::{dyn_module_findings, dyn_operand_module_findings};
#[cfg(test)]
pub(crate) use forbidden_marker::forbidden_marker_findings;
#[cfg(test)]
pub(crate) use impl_trait::{
    impl_trait_module_findings, impl_trait_operand_module_findings,
    impl_trait_operand_subtree_findings, impl_trait_subtree_findings,
};
#[cfg(test)]
pub(crate) use trait_impl::trait_impl_findings;
#[cfg(test)]
pub(crate) use unsafe_confinement::unsafe_findings;
#[cfg(test)]
pub(crate) use visibility::visibility_findings;

use crate::async_exposure::check_async_exposure_boundary;
use crate::driver::{eval_into, outcome_from, read_metadata};
use crate::dyn_trait::check_dyn_trait_boundary;
use crate::exposure::check_boundary;
use crate::forbidden_marker::check_forbidden_marker_boundary;
use crate::impl_trait::check_impl_trait_boundary;
use crate::trait_impl::check_trait_impl_boundary;
use crate::unsafe_confinement::check_unsafe_boundary;
use crate::visibility::check_visibility_boundary;

// --- The 渾儀 dimension's boundary set ----------------------------------------

/// The 渾儀 (semantic) dimension's boundaries, gathered so the shell takes the dimension as
/// one unit rather than one parameter per capability. Each field is one capability's
/// boundaries; [`check_all`] evaluates every non-empty bundle with a single `cargo metadata` read.
#[derive(Debug, Clone, Default)]
pub struct SemanticBoundaries {
    /// Exposure boundaries (`semantic-signature-coupling`).
    pub signature: Vec<SignatureBoundary>,
    /// Impl-locality boundaries (`semantic-trait-impl-locality`).
    pub trait_impl: Vec<TraitImplBoundary>,
    /// Visibility boundaries (`semantic-visibility-boundary`).
    pub visibility: Vec<VisibilityBoundary>,
    /// Forbidden-marker boundaries (`semantic-forbidden-marker`).
    pub forbidden_marker: Vec<ForbiddenMarkerBoundary>,
    /// Dyn-trait exposure boundaries (`semantic-dyn-trait-boundary`).
    pub dyn_trait: Vec<DynTraitBoundary>,
    /// Impl-trait (existential) exposure boundaries (`semantic-impl-trait-boundary`).
    pub impl_trait: Vec<ImplTraitBoundary>,
    /// Async-fn (implicit existential) exposure boundaries (`semantic-async-exposure-boundary`).
    pub async_exposure: Vec<AsyncExposureBoundary>,
    /// Unsafe-confinement boundaries (`semantic-unsafe-confinement`).
    pub unsafe_confinement: Vec<UnsafeBoundary>,
}

/// One capability's boundaries, its `crate_package` accessor, and its `check_*_boundary`
/// reaction, behind one dyn-safe surface — so [`SemanticBoundaries::is_empty`],
/// [`SemanticBoundaries::crate_packages`], and [`eval_all`] each enumerate the eight capabilities
/// via one loop over [`SemanticBoundaries::capability_sets`] rather than independently
/// hand-enumerating all eight (the drift risk a ninth capability's addition would otherwise
/// carry: three lists to remember, not one).
trait CapabilitySet<'a> {
    fn is_empty(&self) -> bool;
    fn crate_packages(&self) -> Vec<&'a str>;
    fn eval(&self, metadata: &Value, violations: &mut Vec<Violation>) -> Result<(), String>;
}

/// The generic `CapabilitySet` implementation every capability's field instantiates: its own
/// boundary slice, its boundary type's `crate_package` accessor, and its `check_*_boundary`
/// reaction — all plain `fn` items, so no per-capability trait impl is needed. `'a` names the
/// borrow of the owning `SemanticBoundaries`, carried through to `crate_packages`' return so a
/// caller can collect it after this value (or its dyn wrapper) has been dropped.
struct Capability<'a, B> {
    boundaries: &'a [B],
    crate_package: fn(&B) -> &str,
    check: fn(&Value, &B, &mut Vec<Violation>) -> Result<(), String>,
}

impl<'a, B> CapabilitySet<'a> for Capability<'a, B> {
    fn is_empty(&self) -> bool {
        self.boundaries.is_empty()
    }

    fn crate_packages(&self) -> Vec<&'a str> {
        self.boundaries
            .iter()
            .map(|boundary| (self.crate_package)(boundary))
            .collect()
    }

    fn eval(&self, metadata: &Value, violations: &mut Vec<Violation>) -> Result<(), String> {
        eval_into(metadata, self.boundaries, self.check, violations)
    }
}

impl SemanticBoundaries {
    /// The eight declared capabilities as one dyn-safe list, in the fixed evaluation order
    /// [`eval_all`] shares. The single enumeration point [`is_empty`](Self::is_empty),
    /// [`crate_packages`](Self::crate_packages), and [`eval_all`] each loop over.
    fn capability_sets(&self) -> Vec<Box<dyn CapabilitySet<'_> + '_>> {
        vec![
            Box::new(Capability {
                boundaries: &self.signature,
                crate_package: SignatureBoundary::crate_package,
                check: check_boundary,
            }),
            Box::new(Capability {
                boundaries: &self.trait_impl,
                crate_package: TraitImplBoundary::crate_package,
                check: check_trait_impl_boundary,
            }),
            Box::new(Capability {
                boundaries: &self.visibility,
                crate_package: VisibilityBoundary::crate_package,
                check: check_visibility_boundary,
            }),
            Box::new(Capability {
                boundaries: &self.forbidden_marker,
                crate_package: ForbiddenMarkerBoundary::crate_package,
                check: check_forbidden_marker_boundary,
            }),
            Box::new(Capability {
                boundaries: &self.dyn_trait,
                crate_package: DynTraitBoundary::crate_package,
                check: check_dyn_trait_boundary,
            }),
            Box::new(Capability {
                boundaries: &self.impl_trait,
                crate_package: ImplTraitBoundary::crate_package,
                check: check_impl_trait_boundary,
            }),
            Box::new(Capability {
                boundaries: &self.async_exposure,
                crate_package: AsyncExposureBoundary::crate_package,
                check: check_async_exposure_boundary,
            }),
            Box::new(Capability {
                boundaries: &self.unsafe_confinement,
                crate_package: UnsafeBoundary::crate_package,
                check: check_unsafe_boundary,
            }),
        ]
    }

    /// Whether no semantic boundary of any kind is declared.
    pub fn is_empty(&self) -> bool {
        self.capability_sets().iter().all(|set| set.is_empty())
    }

    /// The target crate package of every declared semantic boundary, across all capabilities.
    ///
    /// Centralizes crate-target enumeration for composed consumers such as workspace coverage, so
    /// adding a capability cannot require a second hand-maintained list in the shell.
    pub fn crate_packages(&self) -> impl Iterator<Item = &str> {
        self.capability_sets()
            .into_iter()
            .flat_map(|set| set.crate_packages())
            .collect::<Vec<_>>()
            .into_iter()
    }
}

// --- Composition: evaluate every declared capability with a single metadata read ------

/// Evaluate every declared semantic capability against `metadata` into the one accumulator, in a
/// fixed order (shared with [`SemanticBoundaries::capability_sets`]); the first constitution error
/// short-circuits. Split out so [`check_all`] keeps the single-read + exit-2-supersedes contract
/// with plain `?`, not eight repeated error blocks.
fn eval_all(
    metadata: &Value,
    boundaries: &SemanticBoundaries,
    violations: &mut Vec<Violation>,
) -> Result<(), String> {
    for set in boundaries.capability_sets() {
        set.eval(metadata, violations)?;
    }
    Ok(())
}

/// Evaluate every declared semantic boundary against the workspace with a **single**
/// `cargo metadata` read, merging all findings into one outcome. A constitution error on any
/// boundary supersedes (exit 2). An empty bundle returns [`Outcome::Clean`] before metadata is
/// read. The per-capability `check`/`check_trait_impl_locality`/`check_visibility` entries remain
/// for direct use; the shell and [`SemanticObserver`] compose via this.
///
/// `Clean` for an empty bundle is not the vacuous pass a composed run of **no** observer would be. Here a
/// participant was composed and declares nothing for this dimension, which a static-only adoption does
/// deliberately, so there is nothing to observe; refusing would make that adoption's every run exit 2.
/// `observer-protocol` states the asymmetry and why unifying it fails in both directions.
pub fn check_all(boundaries: &SemanticBoundaries, manifest_path: &Path) -> Outcome {
    if boundaries.is_empty() {
        return Outcome::Clean;
    }
    let metadata = match read_metadata(manifest_path) {
        Ok(metadata) => metadata,
        Err(outcome) => return outcome,
    };
    let mut violations = Vec::new();
    match eval_all(&metadata, boundaries, &mut violations) {
        Ok(()) => outcome_from(violations),
        Err(error) => Outcome::ConstitutionError(error),
    }
}

#[cfg(test)]
mod tests;
