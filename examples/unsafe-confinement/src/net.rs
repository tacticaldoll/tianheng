//! The networking module — ordinary, safe application code that must NOT reach for `unsafe`.
//!
//! THE DELIBERATE VIOLATION: `peek` opens a raw-pointer `unsafe` block here, outside the declared
//! `crate::ffi` subtree. `UnsafeBoundary` reads the AST and reacts — the confinement is breached.
//! (The right repair is to move the raw access behind the audited boundary in `crate::ffi`.)

/// A stray raw-pointer read outside the confined ffi subtree — the deliberate leak.
///
/// # Safety
///
/// `pointer` must be valid and aligned for a read of one `u8`.
pub unsafe fn peek(pointer: *const u8) -> u8 {
    // SAFETY: the caller guarantees `pointer` is valid for a read of one byte. (The point is the
    // *location*: this `unsafe` sits outside the allowed `crate::ffi` subtree, so it reacts.)
    unsafe { *pointer }
}
