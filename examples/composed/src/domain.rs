//! Domain — the pure core. THE STATIC FAULT: it imports `infra` (圭表 reacts on the `use` scan).
use crate::infra::DbPool;

/// A domain operation that wrongly reaches for infrastructure directly.
pub fn bootstrap() -> DbPool {
    DbPool::open()
}
