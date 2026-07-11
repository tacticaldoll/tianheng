//! A kernel submodule. Its public API must stay synchronous like the rest of the kernel.
//!
//! THE 渾儀 VIOLATION: `fetch` is a `pub async fn` — async belongs at the edges, not in the pure
//! kernel. Because `sans_io_pure`'s async half descends the whole `crate::kernel` subtree
//! (`including_submodules`), this reacts even though it sits one module below the anchor.

/// A public `async fn` inside the kernel subtree — the deliberate async-exposure violation.
pub async fn fetch() {}
