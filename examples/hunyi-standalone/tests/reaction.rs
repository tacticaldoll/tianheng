//! 渾儀-standalone reactions, asserted as runnable proof: the public-API exposure reaction and its
//! precision, plus the visibility-ceiling (`max_visibility`) depth on `crate::internal`.
use std::path::{Path, PathBuf};

use api_hygiene::governance::constitution;
use hunyi::{
    check, check_visibility, Outcome, SemanticBoundary, VisibilityBoundary, VisibilityCeiling,
};

fn manifest() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("Cargo.toml")
}

/// The core reaction: `api::connection` returns `infra::DbPool`, a leak → exit 1.
#[test]
fn the_api_leak_reacts_with_exit_1() {
    let Outcome::Violations(report) = check(&constitution(), &manifest()) else {
        panic!("the deliberately exposed type must produce a structured violation");
    };
    assert!(report.violations.iter().any(|violation| {
        violation.fact().fact_type() == "tianheng.fact/hunyi/signature-exposure"
            && violation.fact().shape() == "public-seam"
    }));
    assert_eq!(Outcome::Violations(report).exit_code(), 1);
}

/// Precision: forbidding a type that is *not* named on any public surface does not react — the
/// dimension reacts to real exposure, not to the mere existence of the type.
#[test]
fn a_type_that_is_not_exposed_does_not_react() {
    let boundary = vec![SemanticBoundary::in_crate("api_hygiene")
        .module("crate::api")
        .must_not_expose("crate::infra::Secret")
        .because("a type never exposed must not react")];
    assert_eq!(check(&boundary, &manifest()).exit_code(), 0);
}

/// Visibility ceiling: `crate::internal` declares `pub struct Widget`, above a `Crate` ceiling →
/// exit 1. The neighbouring `pub(crate) struct Gadget` is AT the ceiling, so it does not react —
/// the discriminator that shows the rule reacts to over-visibility, not to every item.
#[test]
fn an_over_pub_item_breaches_the_crate_visibility_ceiling() {
    let boundary = vec![VisibilityBoundary::in_crate("api_hygiene")
        .module("crate::internal")
        .max_visibility(VisibilityCeiling::Crate)
        .because("internal is crate-private by contract")];
    let outcome = check_visibility(&boundary, &manifest());
    assert_eq!(outcome.exit_code(), 1);
    if let Outcome::Violations(report) = &outcome {
        assert!(report.violations.iter().any(|violation| {
            violation.fact().fact_type() == "tianheng.fact/hunyi/visibility-exposure"
                && violation.fact().shape() == "declared-item-visibility"
        }));
        let findings: Vec<&str> = report
            .violations
            .iter()
            .map(|v| v.finding.as_str())
            .collect();
        assert!(
            findings.iter().any(|f| f.contains("Widget")),
            "{findings:?}"
        );
        assert!(
            !findings.iter().any(|f| f.contains("Gadget")),
            "a pub(crate) item is at the Crate ceiling and must not react: {findings:?}"
        );
    }
}

/// The v0.1.8 depth: the ceiling is a *rank*, not a binary. A stricter `Super` ceiling reaches the
/// `pub(crate) Gadget` that the `Crate` ceiling let pass — so `max_visibility` governs how far an
/// item may be seen, not merely "is it `pub`".
#[test]
fn a_stricter_super_ceiling_reaches_the_pub_crate_item() {
    let boundary = vec![VisibilityBoundary::in_crate("api_hygiene")
        .module("crate::internal")
        .max_visibility(VisibilityCeiling::Super)
        .because("internal must not exceed pub(super)")];
    let outcome = check_visibility(&boundary, &manifest());
    assert_eq!(outcome.exit_code(), 1);
    if let Outcome::Violations(report) = &outcome {
        let findings: Vec<&str> = report
            .violations
            .iter()
            .map(|v| v.finding.as_str())
            .collect();
        assert!(
            findings.iter().any(|f| f.contains("Gadget")),
            "the pub(crate) item exceeds a Super ceiling and must react: {findings:?}"
        );
    }
}
