//! Infrastructure — the outer layer. The domain must not reach in here.

/// A stand-in for a real infrastructural resource (a DB pool, an HTTP client, …).
pub struct DbPool;

impl DbPool {
    /// Open the pool.
    pub fn open() -> Self {
        DbPool
    }
}
