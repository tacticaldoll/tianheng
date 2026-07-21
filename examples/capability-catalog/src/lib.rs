//! Deliberately violating source for the capability-catalog contract checks.

pub mod governance;
pub mod marked;
pub mod misplaced;
pub mod shapes;

/// A local architectural command whose implementations are confined by the catalog law.
pub trait Command {}

/// A marker the catalog law forbids types under `marked` from acquiring.
pub trait Marker {}
