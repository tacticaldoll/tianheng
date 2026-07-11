//! A small library governed by 渾儀 (hunyi)'s **unsafe-confinement** capability: `unsafe` may
//! appear **only under** a declared subtree (`crate::ffi`), the auditability boundary of a layered
//! crate. It governs *where* `unsafe` lives, not *whether* it may exist — the non-compiler-
//! expressible complement of `#![forbid(unsafe_code)]`.
//!
//! - `crate::ffi` holds real, **confined** `unsafe` (a raw-pointer read behind a safe wrapper) —
//!   the allowed location.
//! - `crate::net` deliberately contains a **stray** `unsafe` block — a leak outside the ffi
//!   subtree. `tests/reaction.rs` asserts the reaction; the `demo` binary renders it.
pub mod ffi;
pub mod governance;
pub mod net;
