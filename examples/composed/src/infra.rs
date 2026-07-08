//! Infrastructure — the outer layer neither the domain nor the public API may reach.

/// A stand-in for an internal infrastructural resource.
pub struct DbPool;

impl DbPool {
    /// Open the pool.
    pub fn open() -> Self {
        DbPool
    }
}
