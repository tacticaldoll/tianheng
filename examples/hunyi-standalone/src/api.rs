//! The public API surface. It should expose only its own vocabulary, never `infra` types.
//!
//! THE DELIBERATE VIOLATION: `connection` returns `crate::infra::DbPool` — an internal type
//! named in a `pub` signature. 渾儀 reads the AST and reacts to the leak (a token scanner that
//! only sees `use` would miss a fully-qualified return type; the semantic dimension catches it).

/// Leaks the internal pool type onto the public API.
pub fn connection() -> crate::infra::DbPool {
    crate::infra::DbPool::open()
}
