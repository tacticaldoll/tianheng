//! Infrastructure — an internal type that must not surface on the public API.

/// A stand-in for an internal resource whose existence is an implementation detail.
pub struct DbPool;

impl DbPool {
    /// Open the pool.
    pub fn open() -> Self {
        DbPool
    }
}
