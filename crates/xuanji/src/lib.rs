//! 璇璣 (Xuánjī) — the shared **reaction model** of Tianheng, the 底 the whole stack
//! turns on.
//!
//! The jade pivot of the armillary sphere, the instrument of celestial measure: the
//! dimension-agnostic vocabulary [`Severity`], [`BoundaryKind`], [`Violation`],
//! [`Report`], [`Baseline`], and [`Outcome`] (each a finding's shape; [`ViolationId`] is
//! `Violation`'s baseline identity). Every observation
//! dimension — the static 圭表 (`guibiao`), semantic 渾儀 (`hunyi`), and runtime 漏刻
//! (`louke`) — expresses its findings in these types, so a dimension may reuse the reaction
//! vocabulary without depending on another dimension's engine.
//!
//! This crate carries the JSON (de)serialization that is **intrinsic** to its types: a
//! [`Baseline`] *is* a generated JSON snapshot, and a [`Violation`] has a canonical JSON
//! shape. `serde_json` is its only dependency; it renders **no verdict** — it holds the
//! *measure*, never the react itself.
//!
//! Govern by reaction, not instruction.

#![forbid(unsafe_code)]
#![deny(missing_docs)]

mod baseline;
mod finding;
mod identity;
mod model;
mod util;
mod violation;

#[cfg(test)]
mod tests;

pub use baseline::{Baseline, BaselineEntry, ViolationId, apply_baseline};
pub use finding::Finding;
pub use identity::{RuleKey, StructuredFactIdentity};
pub use model::{BoundaryKind, Outcome, Polarity, ScanDepth, Severity};
pub use util::pretty_json;
pub use violation::{Report, Violation};
