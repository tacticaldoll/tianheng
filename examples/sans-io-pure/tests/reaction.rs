//! `sans_io_pure` reactions, asserted as runnable proof: the profile folds a 圭表 clock boundary
//! and a subtree-scoped 渾儀 async boundary into one declaration, and BOTH axes react on the
//! kernel — including a `pub async fn` one module *below* the anchor, which only the subtree scope
//! catches.
use std::path::{Path, PathBuf};

use sans_io_kernel::governance::constitution;
use tianheng::prelude::*;
use tianheng::{check, check_async_exposure};

fn manifest() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("Cargo.toml")
}

/// The 圭表 half: `kernel::stamp` reads `std::time::SystemTime::now()` inline → exit 1. Asserted on
/// the boundary the profile actually emitted (the static side of the composition).
#[test]
fn the_clock_axis_reacts() {
    let c = constitution();
    let Outcome::Violations(report) = check(c.static_boundaries(), &manifest()) else {
        panic!("the inline clock read must produce a structured violation");
    };
    assert!(report.violations.iter().any(|violation| {
        violation.fact().fact_type() == "tianheng.fact/guibiao/inline-path"
            && violation.fact().shape() == "path-in-module"
    }));
    assert_eq!(Outcome::Violations(report).exit_code(), 1);
}

/// The 渾儀 half: `kernel::inner::fetch` is a `pub async fn` in a submodule → exit 1. Asserted on
/// the async boundary the profile emitted, which is subtree-scoped (`including_submodules`).
#[test]
fn the_async_axis_reacts_across_the_subtree() {
    let c = constitution();
    let async_boundaries = &c.semantic_boundaries().async_exposure;
    let Outcome::Violations(report) = check_async_exposure(async_boundaries, &manifest()) else {
        panic!("the subtree async seam must produce a structured violation");
    };
    assert!(report.violations.iter().any(|violation| {
        violation.fact().fact_type() == "tianheng.fact/hunyi/async-exposure"
            && matches!(
                violation.fact().shape(),
                "async-free-function" | "async-inherent-method" | "async-trait-method"
            )
    }));
    assert_eq!(Outcome::Violations(report).exit_code(), 1);
}

/// The discriminator that makes the subtree scope load-bearing: a **seam-only** async boundary on
/// `crate::kernel` (without `including_submodules`) does NOT see the submodule's `async fn` — so it
/// passes (exit 0). `sans_io_pure` opts into the subtree scope precisely to close this gap.
#[test]
fn a_seam_only_async_boundary_would_miss_the_submodule() {
    let seam_only = vec![AsyncExposureBoundary::in_crate("sans_io_kernel")
        .module("crate::kernel")
        .must_not_expose_async_fn()
        .because("seam-only: governs crate::kernel's own items, not its submodules")];
    assert_eq!(check_async_exposure(&seam_only, &manifest()).exit_code(), 0);
}
