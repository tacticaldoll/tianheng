//! The fixed lifecycle every observation participant implements.
//!
//! 天衡's promise is honesty about what a reaction does **not** see. [`BoundDecl`] types that honesty; this trait
//! makes it a condition of taking part. [`Observer::bounds`] has **no default body**, so a participant cannot be
//! composed into a run without declaring its limits. Before this, the promise was a convention the family kept
//! about itself.
//!
//! # Why no method has a default
//!
//! The enforcement has to land on the **declarer**, not the reader. An enum plus a hand-maintained list is the
//! other shape, and it fails in exactly that way: adding a value breaks the consumer's `match` while every
//! existing declaration keeps its old answer and nobody re-examines it. A method with no default inverts that —
//! adding a **stage** breaks every implementor, family and third-party alike, because a declaration written
//! before a question existed has not answered it.
//!
//! The other half matters as much: adding an **answer** to an existing question must break nothing, which is
//! what `#[non_exhaustive]` on [`Extent`](crate::Extent) and its neighbours provides. Only a new *question*
//! should force re-examination.
//!
//! # Why two methods and not five
//!
//! A third — "identify your boundary kind" — was written and dropped: nothing reacts to it. A
//! [`Violation`](crate::Violation) already carries its own [`BoundaryKind`](crate::BoundaryKind), so an observer
//! restating it would be a second copy of one fact, and two copies can disagree. The same reason
//! [`Extent::demonstrates`](crate::Extent::demonstrates) is derived rather than declared beside its extent.
//!
//! An observation pipeline suggests separate corpus, fact and reaction stages. This family's own law forbids
//! that shape: 三儀 ⊥ 三儀 requires each dimension to implement its lexical hygiene **independently, with no
//! shared scanner**, so no dimension exposes those stages separately and every implementor would collapse them
//! into one call. A lifecycle no implementor honours reads as governance while governing nothing.
//!
//! The same law is why composition is a fan-out and never a pipe: no observer receives another's output. The
//! composed shell composes; the dimensions do not.

use std::path::Path;

use crate::{BoundDecl, Outcome};

/// A participant in one governance run: it observes a workspace, and declares what it does not observe.
///
/// Implement this concretely — the trait names no trait object, and composition introduces none, so an
/// implementor inherits no boxing requirement.
pub trait Observer {
    /// Observe the workspace whose manifest is at `manifest_path`, and return one outcome.
    ///
    /// One call rather than separate corpus, fact and reaction stages — see the module documentation for the law
    /// that makes the split unhonourable. An observer reads what it needs itself; nothing is threaded in from a
    /// sibling.
    ///
    /// Returning [`Outcome::ConstitutionError`] means *this observer cannot judge*, and a composed run treats
    /// that as superseding every violation: a verdict resting on a boundary that could not be evaluated is not a
    /// verdict.
    fn observe(&self, manifest_path: &Path) -> Outcome;

    /// The observation bounds this observer's reaction declares — what it deliberately does not see.
    ///
    /// **No default body, deliberately.** This is the method the protocol exists for: a participant that says
    /// nothing about its limits cannot be written. What it cannot compel is *completeness* — a declared bound of
    /// `observer-protocol` says so, because no reaction can enumerate the limits of a reaction it did not write.
    fn bounds(&self) -> Vec<BoundDecl>;
}
