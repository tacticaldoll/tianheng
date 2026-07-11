//! The kernel that is *supposed* to be sans-I/O: no ambient clock, a synchronous public API.
//! It breaks both axes, so `sans_io_pure` reacts on each.
//!
//! THE 圭表 VIOLATION: `stamp` reads the ambient clock inline (`std::time::SystemTime::now()`) —
//! time should be *injected*, not read. `sans_io_pure`'s clock half (`must_not_call_inline
//! "std::time" ending_with "now"`) reacts.

/// Reads the ambient wall clock inline — the deliberate clock-read violation.
pub fn stamp() -> std::time::SystemTime {
    std::time::SystemTime::now()
}

/// A kernel submodule (its `async fn` violation, and why the subtree scope is needed, live at its
/// own definition site).
pub mod inner;
