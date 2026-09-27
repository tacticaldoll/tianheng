//! The subtree the catalog law allows `Command` implementations under.

/// An implementation where the law allows it, so it does not react.
pub struct Placed;

impl crate::Command for Placed {}
