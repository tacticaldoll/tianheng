//! Deliberate forbidden-marker fault.

/// A domain type that must not acquire the catalog marker.
pub struct Marked;

impl crate::Marker for Marked {}

/// A second domain type whose marker acquisition is written only inside a `cfg_if!` arm — 0.4.0
/// closed the false negative where an arm's contents (here, the `impl` itself) were invisible to
/// hunyi's item walk merely because `cfg_if!` wraps them. Both branches acquire the marker so the
/// reaction holds regardless of which one this build compiles (mirrors hunyi's own
/// `cfg_if_if_else_arms_both_expose_forbidden_types` fixture).
pub struct CfgGatedMarked;

cfg_if::cfg_if! {
    if #[cfg(unix)] {
        impl crate::Marker for CfgGatedMarked {}
    } else {
        impl crate::Marker for CfgGatedMarked {}
    }
}
