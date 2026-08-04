//! Structured contract checks for the catalog's otherwise-unowned boundary families.

use std::path::PathBuf;

use capability_catalog::governance::constitution;
use tianheng::prelude::*;

fn manifest() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("Cargo.toml")
}

#[test]
fn uncovered_public_families_react_through_the_composed_evaluator() {
    let outcome = check_constitution(&constitution(), &manifest());
    assert_eq!(outcome.exit_code(), 1, "catalog must be deliberately red");
    let Outcome::Violations(report) = outcome else {
        panic!("catalog must return structured violations")
    };

    let observed: Vec<_> = report
        .violations
        .iter()
        .map(|violation| {
            (
                violation.kind.as_str(),
                violation.rule.as_str(),
                violation.fact().fact_type(),
                violation.fact().shape(),
                violation.reason.as_str(),
            )
        })
        .collect();

    for expected in [
        (
            "crate",
            "restrict dependency sources to",
            "tianheng.fact/guibiao/dependency-source",
            "declared-source",
            "catalog source metadata must produce its declared source reaction",
        ),
        (
            "module",
            "external crate confined to module",
            "tianheng.fact/guibiao/external-importer",
            "module-path",
            "the external shell dependency stays behind the governance module",
        ),
        (
            "semantic",
            "must only be implemented in the declared location(s)",
            "tianheng.fact/hunyi/trait-impl-site",
            "misplaced-implementation",
            "Command implementations live only under the allowed subtree",
        ),
        (
            "semantic",
            "must not acquire trait",
            "tianheng.fact/hunyi/forbidden-marker-acquisition",
            "impl",
            "marked-domain types remain free of the catalog marker",
        ),
        (
            "semantic",
            "must not expose dyn",
            "tianheng.fact/hunyi/dyn-trait-exposure",
            "public-seam",
            "the catalog dyn family must produce its structured reaction",
        ),
        (
            "semantic",
            "must not expose impl trait",
            "tianheng.fact/hunyi/impl-trait-exposure",
            "public-seam",
            "the catalog impl-trait family must produce its structured reaction",
        ),
        (
            "semantic",
            "must not expose async fn",
            "tianheng.fact/hunyi/async-exposure",
            "async-free-function",
            "the catalog's composed no-existential-leak profile must produce its structured \
             reaction for both the written and the implicit existential signal",
        ),
    ] {
        assert!(
            observed.contains(&expected),
            "missing structured reaction owner for `{expected:?}`: {observed:#?}"
        );
    }
}

/// 0.4.0 closed the false negative where a `cfg_if!` arm's contents were invisible to hunyi's
/// item walk. `crate::marked::CfgGatedMarked` acquires the forbidden marker only inside such an
/// arm (both branches, so the reaction holds on every target); this asserts the specific type is
/// actually named in the reaction, not merely that *a* forbidden-marker violation exists (the
/// broad identity check above already covers `Marked` alone).
#[test]
fn a_cfg_if_wrapped_marker_acquisition_reacts_by_name() {
    let outcome = check_constitution(&constitution(), &manifest());
    let Outcome::Violations(report) = outcome else {
        panic!("catalog must return structured violations")
    };
    let findings: Vec<&str> = report
        .violations
        .iter()
        .map(|v| v.finding.as_str())
        .collect();
    assert!(
        findings.iter().any(|f| f.contains("CfgGatedMarked")),
        "a marker acquired only inside a cfg_if! arm must react: {findings:#?}"
    );
}

/// 0.4.0 closed the false negative where trait-impl-locality treated a `const _: () = { impl … };`
/// body ("const-eval trick") as opaque. `crate::misplaced::Rogue` implements `crate::Command`
/// only behind that wrapper, outside the declared `crate::allowed` subtree; this asserts the
/// specific type is named in the reaction, not merely that *a* locality violation exists (the
/// broad identity check above already covers `Misplaced` alone).
#[test]
fn a_const_eval_trick_wrapped_impl_reacts_by_name() {
    let outcome = check_constitution(&constitution(), &manifest());
    let Outcome::Violations(report) = outcome else {
        panic!("catalog must return structured violations")
    };
    let findings: Vec<&str> = report
        .violations
        .iter()
        .map(|v| v.finding.as_str())
        .collect();
    assert!(
        findings.iter().any(|f| f.contains("Rogue")),
        "an impl written behind the const-eval trick must react: {findings:#?}"
    );
}

/// The `cfg-if = "1"` dependency (`marked.rs`'s cfg_if-wrapped marker-acquisition fixture) adds a
/// SECOND finding under the crate's `restrict_dependency_sources_to` boundary, sharing the
/// identical `(kind, rule, fact_type, shape, reason)` 5-tuple as the catalog's own pre-existing
/// `tianheng` dependency finding asserted above — the composite identity check there cannot tell
/// the two apart, so a future change that quietly stopped flagging `cfg-if` specifically would
/// still satisfy it via the `tianheng` finding alone. Assert `cfg-if` is named by finding text.
#[test]
fn the_cfg_if_dependency_produces_its_own_dependency_source_finding() {
    let outcome = check_constitution(&constitution(), &manifest());
    let Outcome::Violations(report) = outcome else {
        panic!("catalog must return structured violations")
    };
    let findings: Vec<&str> = report
        .violations
        .iter()
        .map(|v| v.finding.as_str())
        .collect();
    assert!(
        findings.contains(&"cfg-if"),
        "the cfg-if dependency must produce its own named dependency-source finding: {findings:#?}"
    );
}

/// The other closed 0.4.0 shape: a malformed forbidden/allowed operand (here a leading `::`) must
/// fail loud as a constitution error (exit 2) instead of silently matching nothing. This declares
/// its own tiny, isolated constitution — deliberately NOT the shared dense one above, since a
/// constitution error stops the whole evaluation and would mask every other family's reaction.
#[test]
fn a_malformed_forbidden_operand_fails_loud_as_a_constitution_error() {
    let malformed = Constitution::new("capability_catalog").forbidden_marker_boundary(
        ForbiddenMarkerBoundary::in_crate("capability_catalog")
            .module("crate::marked")
            .must_not_acquire("::crate::Marker")
            .because("a malformed operand must fail loud, never silently pass"),
    );
    let outcome = check_constitution(&malformed, &manifest());
    assert_eq!(
        outcome.exit_code(),
        2,
        "a leading-`::` operand is a constitution error (exit 2) — distinct from a violation \
         (exit 1) and, above all, from the false-negative silent pass (exit 0) this closes"
    );
    let Outcome::ConstitutionError(message) = outcome else {
        panic!("a malformed forbidden operand must produce Outcome::ConstitutionError")
    };
    assert!(
        message.contains("::crate::Marker"),
        "the constitution error must name the malformed operand verbatim: {message}"
    );
}
