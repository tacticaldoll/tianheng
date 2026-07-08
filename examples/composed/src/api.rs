//! Public API. THE SEMANTIC FAULT: `connection` returns `infra::DbPool`, leaking an internal
//! type onto the public surface (渾儀 reads the AST and reacts).

/// Leaks the internal pool type onto the public API.
pub fn connection() -> crate::infra::DbPool {
    crate::infra::DbPool::open()
}
