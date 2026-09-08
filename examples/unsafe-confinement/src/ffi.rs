//! The FFI subtree — the ONE place `unsafe` is allowed to live. All raw-pointer / foreign-boundary
//! work is confined here, behind explicit boundary functions, so an auditor reviews `unsafe` in
//! exactly one module. `UnsafeBoundary` does not react to `unsafe` here (it is under the declared
//! subtree).

/// Read the byte a raw pointer refers to — confined `unsafe`, the allowed case. (Never called in
/// the reaction: 渾儀 reads the AST, it does not execute the code.)
///
/// # Safety
///
/// `pointer` must be valid and aligned for a read of one `u8`.
pub unsafe fn read(pointer: *const u8) -> u8 {
    // SAFETY: the caller guarantees `pointer` is valid for a read of one byte.
    unsafe { *pointer }
}
