//! Deliberate static-item fault.

use std::cell::Cell;

/// A process-wide counter the catalog law forbids this module from holding.
pub static COUNTER: std::sync::atomic::AtomicU32 = std::sync::atomic::AtomicU32::new(0);

thread_local! {
    /// Per-thread scratch state the catalog law forbids this module from holding.
    pub static SCRATCH: Cell<u32> = const { Cell::new(0) };
}
