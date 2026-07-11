//! An internal module whose items are meant to stay crate-private — governed by 渾儀's
//! **visibility ceiling** (`max_visibility`), the depth complement of the exposure rules.
//!
//! THE DELIBERATE VIOLATION: `Widget` is declared `pub`, so it can escape the crate — above a
//! `Crate` ceiling. `max_visibility(VisibilityCeiling::Crate)` reacts (its sugar is
//! `must_not_declare_pub`). The neighbouring `pub(crate)` item is at the ceiling, so it passes.

/// Leaks past the crate boundary — the deliberate over-visibility (`pub`, above the `Crate`
/// ceiling).
pub struct Widget;

/// Correctly crate-private — at the `Crate` ceiling, so it does not react.
// It exists to be *observed* by the visibility ceiling, not called; `allow(dead_code)` keeps the
// example warning-clean without a contrived use that would obscure the point.
#[allow(dead_code)]
pub(crate) struct Gadget;
