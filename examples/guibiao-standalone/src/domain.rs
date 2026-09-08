//! Domain — the pure core. It should depend on ports, never on `infra`.
//!
//! THE DELIBERATE VIOLATION: this `use` reaches into `crate::infra`, which the constitution in
//! [`crate::governance`] forbids. 圭表 observes it on the source `use` scan and reacts.
use crate::infra::DbPool;

/// A domain operation that (wrongly) grabs infrastructure directly.
pub fn bootstrap() -> DbPool {
    DbPool::open()
}
