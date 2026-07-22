//! 渾儀 (Húnyí) — the **semantic** observation dimension of Tianheng.
//!
//! Where the gnomon 圭表 observes *imports*, 渾儀 observes *meaning* via the AST (`syn`):
//! does a module's **public API expose** a forbidden type? That is the complement of
//! import-governance — a type imported for internal use is fine, but a type named in a `pub`
//! signature or alias chain is observed.
//!
//! Declare a [`SemanticBoundary`] in Rust, [`check`] it against a Cargo workspace, and get
//! an [`Outcome`]. The heavy `syn` parser is quarantined to this crate, keeping the functional
//! core dependency-light (`self_governance.rs`).
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
    Baseline, BoundaryKind, Finding, Outcome, Polarity, Report, RuleKey, Severity,
    StructuredFactIdentity, Violation, ViolationId, apply_baseline,
};

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
pub(crate) use exposure::module_findings;
#[cfg(test)]
pub(crate) use forbidden_marker::forbidden_marker_findings;
#[cfg(test)]
pub(crate) use impl_trait::{impl_trait_module_findings, impl_trait_operand_module_findings};
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
/// boundaries; [`check_all`] evaluates them all with a single `cargo metadata` read.
#[derive(Debug, Clone, Default)]
pub struct SemanticBoundaries {
    /// Exposure boundaries (`semantic-signature-coupling`).
    pub signature: Vec<SemanticBoundary>,
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

impl SemanticBoundaries {
    /// Whether no semantic boundary of any kind is declared.
    pub fn is_empty(&self) -> bool {
        self.signature.is_empty()
            && self.trait_impl.is_empty()
            && self.visibility.is_empty()
            && self.forbidden_marker.is_empty()
            && self.dyn_trait.is_empty()
            && self.impl_trait.is_empty()
            && self.async_exposure.is_empty()
            && self.unsafe_confinement.is_empty()
    }
}

// --- Composition: evaluate every capability with a single metadata read ------

/// Evaluate every declared semantic capability against `metadata` into the one accumulator, in a
/// fixed order; the first constitution error short-circuits. Split out so [`check_all`] keeps the
/// single-read + exit-2-supersedes contract with plain `?`, not eight repeated error blocks.
fn eval_all(
    metadata: &Value,
    boundaries: &SemanticBoundaries,
    violations: &mut Vec<Violation>,
) -> Result<(), String> {
    eval_into(metadata, &boundaries.signature, check_boundary, violations)?;
    eval_into(
        metadata,
        &boundaries.trait_impl,
        check_trait_impl_boundary,
        violations,
    )?;
    eval_into(
        metadata,
        &boundaries.visibility,
        check_visibility_boundary,
        violations,
    )?;
    eval_into(
        metadata,
        &boundaries.forbidden_marker,
        check_forbidden_marker_boundary,
        violations,
    )?;
    eval_into(
        metadata,
        &boundaries.dyn_trait,
        check_dyn_trait_boundary,
        violations,
    )?;
    eval_into(
        metadata,
        &boundaries.impl_trait,
        check_impl_trait_boundary,
        violations,
    )?;
    eval_into(
        metadata,
        &boundaries.async_exposure,
        check_async_exposure_boundary,
        violations,
    )?;
    eval_into(
        metadata,
        &boundaries.unsafe_confinement,
        check_unsafe_boundary,
        violations,
    )?;
    Ok(())
}

/// Evaluate every declared semantic boundary against the workspace with a **single**
/// `cargo metadata` read, merging all findings into one outcome. A constitution error on any
/// boundary supersedes (exit 2). The per-capability `check`/`check_trait_impl_locality`/
/// `check_visibility` entries remain for direct use; the shell composes via this.
pub fn check_all(boundaries: &SemanticBoundaries, manifest_path: &Path) -> Outcome {
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
