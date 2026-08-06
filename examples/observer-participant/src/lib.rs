//! A crate with a house rule of its own, governed in one run with 天衡's dimensions.
//!
//! Two faults are deliberate and neither is a bug to fix: `api` imports `infra`, which 圭表 reacts to, and
//! `undocumented.rs` opens with no `//!` header, which this crate's own participant reacts to. The example
//! exists to show one exit code covering both.
pub mod api;
pub mod governance;
pub mod infra;
pub mod observer;
pub mod undocumented;
