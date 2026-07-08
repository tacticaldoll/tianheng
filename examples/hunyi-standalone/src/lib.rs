//! A small library governed by 渾儀 (hunyi) **alone** — the semantic dimension that reads the
//! AST and reacts when a module's **public API exposes** a forbidden type. The complement of
//! 圭表: a type imported for internal use is fine; a type named in a `pub` signature is a leak.
//!
//! Here `crate::api` deliberately leaks `crate::infra::DbPool` through a `pub fn` return type.
//! `tests/reaction.rs` asserts the reaction; the `demo` binary renders it.
pub mod api;
pub mod governance;
pub mod infra;
