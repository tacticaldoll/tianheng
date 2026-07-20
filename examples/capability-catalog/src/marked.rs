//! Deliberate forbidden-marker fault.

/// A domain type that must not acquire the catalog marker.
pub struct Marked;

impl crate::Marker for Marked {}
