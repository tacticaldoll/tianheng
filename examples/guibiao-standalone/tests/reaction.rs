//! 圭表-standalone reactions, asserted as runnable proof. These ride the *simplest* example
//! (per the BACKLOG plan) because each needs only one boundary + one violation: the adoption
//! ladder (severity and baseline axes) and the identity ⊥ metadata stability contract.
use std::path::{Path, PathBuf};

use guibiao::{
    apply_baseline, check, Baseline, Constitution, CrateBoundary, ModuleBoundary, Outcome, Report,
};
use hexagonal_demo::governance::constitution;

fn manifest() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("Cargo.toml")
}

fn report_of(outcome: Outcome) -> Report {
    match outcome {
        Outcome::Violations(report) => report,
        other => panic!("expected violations, got {other:?}"),
    }
}

/// The core reaction: the `domain → infra` import trips the enforce boundary → exit 1.
#[test]
fn the_import_violation_reacts_with_exit_1() {
    assert_eq!(check(&constitution(), &manifest()).exit_code(), 1);
}

/// 圭表 also governs the *feature* surface of a declared dependency, not just its name. This
/// dogfoods `forbid_feature` end-to-end through the published `check` surface, on the crate's real
/// declared edge: the adopter's law pins `guibiao` to `default-features = false` (keep the footprint
/// minimal), but `Cargo.toml` declares `guibiao = "0.1"` with defaults on, so the declared `default`
/// feature trips the enforce boundary. Kept as its own constitution so the example's core teaching
/// (the one module boundary above) stays a single, clean message.
#[test]
fn a_forbidden_dependency_feature_reacts() {
    let law = Constitution::new("hexagonal_demo").boundary(
        CrateBoundary::crate_("hexagonal_demo")
            .forbid_feature("guibiao", "default")
            .because("pin guibiao to default-features = false — keep the adopter's footprint minimal"),
    );
    let report = report_of(check(&law, &manifest()));
    assert!(
        report
            .violations
            .iter()
            .any(|v| v.finding == "guibiao/default"),
        "the finding names the dependency and the offending feature (guibiao/default), got {:?}",
        report.violations.iter().map(|v| &v.finding).collect::<Vec<_>>(),
    );
    assert_eq!(
        Outcome::Violations(report).exit_code(),
        1,
        "declaring guibiao with default features on trips the forbid-feature boundary",
    );
}

/// Adoption ladder, axis 1 — severity. The same boundary at `.warn()` is *reported* but does
/// not gate: exit 0. Warn is the gentle entry point.
#[test]
fn warn_severity_reports_without_gating() {
    let warned = Constitution::new("hexagonal_demo").boundary(
        ModuleBoundary::in_crate("hexagonal_demo")
            .module("crate::domain")
            .must_not_import("crate::infra")
            .warn()
            .because("the domain stays pure"),
    );
    let outcome = check(&warned, &manifest());
    assert_eq!(outcome.exit_code(), 0, "warn must not gate");
    assert!(
        !report_of(outcome).violations.is_empty(),
        "warn still reports the drift"
    );
}

/// Adoption ladder, axis 2 — baseline. Grandfathering the existing violation → exit 0.
#[test]
fn baseline_grandfathers_the_existing_violation() {
    let mut report = report_of(check(&constitution(), &manifest()));
    let baseline = Baseline::of(&report);
    apply_baseline(&mut report, &baseline);
    assert_eq!(
        Outcome::Violations(report).exit_code(),
        0,
        "a fully-baselined report is green"
    );
}

/// …but a violation *absent* from the baseline still reacts → exit 1. What is grandfathered is
/// grandfathered; what is new is enforced.
#[test]
fn a_violation_absent_from_the_baseline_still_reacts() {
    let mut report = report_of(check(&constitution(), &manifest()));
    apply_baseline(&mut report, &Baseline::of(&Report::new(vec![])));
    assert_eq!(
        Outcome::Violations(report).exit_code(),
        1,
        "an un-baselined violation reacts"
    );
}

/// Identity ⊥ presentation/metadata. Version-2 `ViolationId = (target, rule, finding_key)` excludes
/// human finding text and `file`, so relocating the code keeps the baseline matching.
#[test]
fn moving_the_file_does_not_churn_the_baseline() {
    let report = report_of(check(&constitution(), &manifest()));
    let baseline = Baseline::of(&report);
    let relocated = report.violations[0]
        .clone()
        .with_file(Some("src/somewhere_else.rs".to_string()));
    assert!(
        baseline.contains(&relocated),
        "file is metadata, not identity — moving code must not turn a known violation new"
    );
}
