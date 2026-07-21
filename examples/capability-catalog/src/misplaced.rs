//! Deliberate trait-locality and external-confinement faults.

use tianheng::Outcome;

/// An implementation outside the law's allowed implementation subtree.
pub struct Misplaced;

impl crate::Command for Misplaced {}

/// Mentioning Tianheng here violates its external-crate confinement.
pub fn leaks(_: Outcome) {}
