//! The funnel as one inspectable reaction: the same unified Constitution the CLI consumes returns
//! one Outcome containing each source-observed dimension's fault, while the declared runtime seam's
//! probe coverage stays clean.
use std::path::{Path, PathBuf};

use composed_app::governance::constitution;
use tianheng::{check_constitution, BoundaryKind, Outcome};

fn manifest() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("Cargo.toml")
}

/// 圭表 and 渾儀 contribute to one report; 漏刻's declared seam is correctly probed.
#[test]
fn the_composed_constitution_returns_every_source_observed_fault() {
    let Outcome::Violations(report) = check_constitution(&constitution(), &manifest()) else {
        panic!("the deliberately violating composed fixture must return a report");
    };
    assert!(
        report
            .violations
            .iter()
            .any(|violation| violation.kind == BoundaryKind::Module),
        "the report must include the static module-import fault",
    );
    assert!(
        report
            .violations
            .iter()
            .any(|violation| violation.kind == BoundaryKind::Semantic),
        "the report must include the semantic API-exposure fault",
    );
    assert!(
        report
            .violations
            .iter()
            .all(|violation| violation.kind != BoundaryKind::Runtime),
        "the declared runtime seam is probed, so its CI face stays clean",
    );
}
