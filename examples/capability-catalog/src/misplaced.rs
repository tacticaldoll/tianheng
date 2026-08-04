//! Deliberate trait-locality and external-confinement faults.

use tianheng::Outcome;

/// An implementation outside the law's allowed implementation subtree.
pub struct Misplaced;

impl crate::Command for Misplaced {}

/// A second misplaced implementation, written behind the "const-eval trick" idiom
/// (`const _: () = { impl … };`, commonly used to force a compile-time trait assertion). 0.4.0
/// closed the false negative where trait-impl-locality treated this body as opaque: the `impl`
/// still binds to `Rogue`'s own coherence set regardless of the const wrapper, so it is real,
/// externally callable API the moment `Rogue` is module-level — and it must be observed exactly
/// like `Misplaced` above.
pub struct Rogue;

const _: () = {
    impl crate::Command for Rogue {}
};

/// Mentioning Tianheng here violates its external-crate confinement.
pub fn leaks(_: Outcome) {}
