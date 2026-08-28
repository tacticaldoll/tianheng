//! This crate's law: one boundary 三儀 owns, and one house rule it does not.
//!
//! The composed run is the point. `Run::over(...).observe(...)` folds a family dimension and an outside
//! participant into **one verdict** with one exit code, so an adopter's own rule is not a second gate bolted
//! beside 天衡 with its own reporting and its own idea of what a failure is.

use std::path::{Path, PathBuf};

use tianheng::prelude::*;

use crate::observer::ModuleHeaderObserver;

/// The subtree this crate's house rule governs.
pub const GOVERNED_SUBTREE: &str = "src";

/// This crate's manifest.
pub fn manifest() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("Cargo.toml")
}

/// The 圭表 half: `crate::api` must not reach into `crate::infra`.
///
/// Deliberately violated — `api.rs` imports it — so the composed verdict has a contribution from a family
/// dimension as well as from the participant. A run where only one of the two reacted would pass a test bound
/// to the exit code while proving nothing about composition.
pub fn constitution() -> Constitution {
    Constitution::new("house-rules").boundary(
        ModuleBoundary::in_crate("house_rules")
            .module("crate::api")
            .must_not_import("crate::infra")
            .because("the api layer talks to infra through a seam, not directly"),
    )
}

/// The participant, reading this crate's governed subtree.
pub fn participant() -> ModuleHeaderObserver {
    ModuleHeaderObserver::reading([GOVERNED_SUBTREE])
}

/// One verdict from a family dimension and an outside participant, folded in that order.
///
/// Assembly order is part of 天衡's contract — it decides which cannot-judge is reported when more than one
/// participant cannot judge — so an adopter composing their own participant is choosing that order too.
pub fn verdict() -> Outcome {
    let manifest = manifest();
    Run::over(&manifest)
        .observe(StaticObserver::new(
            constitution().static_boundaries().clone(),
        ))
        .observe(participant())
        .verdict()
}
