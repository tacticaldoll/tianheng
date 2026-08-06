//! The participant's reactions, asserted as runnable proof.
//!
//! Every assertion here is bound to the participant's **own** contribution, never to the exit code alone: the
//! module boundary reacts on its own, so `exit_code() == 1` would hold with the participant contributing
//! nothing at all. That is the shape of false negative this file exists to refuse.

use std::path::PathBuf;

use house_rules::governance::{constitution, manifest, participant, verdict};
use house_rules::observer::ModuleHeaderObserver;
use tianheng::prelude::*;

/// The house rule reacts, with the participant's own structured identity — not a family fact type.
#[test]
fn the_missing_module_header_reacts_with_the_participant_s_own_identity() {
    let Outcome::Violations(report) = participant().observe(&manifest()) else {
        panic!("the file with no `//!` header must produce a structured violation");
    };
    let house_rule: Vec<&Violation> = report
        .violations
        .iter()
        .filter(|violation| violation.fact().fact_type() == "house-rules.fact/module-header")
        .collect();
    assert_eq!(
        house_rule.len(),
        1,
        "exactly one file in `src/` lacks a header: {report:?}"
    );
    assert_eq!(house_rule[0].fact().shape(), "missing-header");
    assert!(
        house_rule[0].finding.ends_with("undocumented.rs"),
        "the finding names the offending file: {:?}",
        house_rule[0].finding
    );
}

/// **The composition, not the exit code.** Both contributions must be present in one verdict: the family
/// dimension's `module` violation and the participant's own. Asserting exit 1 alone would pass while either
/// one silently stopped contributing, since each reacts by itself.
#[test]
fn one_verdict_carries_both_the_dimension_s_finding_and_the_participant_s() {
    let outcome = verdict();
    assert_eq!(outcome.exit_code(), 1);
    let Outcome::Violations(report) = &outcome else {
        panic!("the composed run must report violations: {outcome:?}");
    };
    assert!(
        report
            .violations
            .iter()
            .any(|violation| violation.fact().fact_type() == "tianheng.fact/guibiao/imported-path"),
        "圭表's contribution is missing, so the fold dropped the dimension: {report:?}"
    );
    assert!(
        report
            .violations
            .iter()
            .any(|violation| violation.fact().fact_type() == "house-rules.fact/module-header"),
        "the participant's contribution is missing, so joining a run did nothing: {report:?}"
    );
}

/// Precision: the participant reacts to a *missing* header, not to every file. Every other file in `src/`
/// has one, so the count above being exactly one is the discriminator — and a subtree where every file is
/// documented is clean rather than merely quieter.
#[test]
fn a_subtree_whose_files_all_carry_headers_is_clean() {
    let bin = ModuleHeaderObserver::reading(["src/bin"]);
    assert_eq!(bin.observe(&manifest()).exit_code(), 0);
}

/// A subtree the participant was told to read and cannot is **exit 2**, never a quiet pass. An outsider
/// joining a run inherits the family's contract: the one forbidden bug is reporting clean because the look
/// failed.
#[test]
fn a_subtree_that_cannot_be_read_cannot_judge() {
    let absent = ModuleHeaderObserver::reading(["src/no-such-subtree"]);
    let outcome = absent.observe(&manifest());
    assert_eq!(outcome.exit_code(), 2, "{outcome:?}");
    assert!(matches!(outcome, Outcome::ConstitutionError(_)));
}

/// The bounds are **computed**, one per configured subtree, so the declaration set depends on configuration
/// rather than on a literal written in advance. This is what `BoundId`'s owned-or-borrowed form is for, and
/// no declaration inside the family exercises it — every family bound is a literal.
#[test]
fn the_declared_bounds_are_built_from_the_configuration() {
    let two = ModuleHeaderObserver::reading(["src", "src/bin"]);
    let ids: Vec<String> = two
        .bounds()
        .iter()
        .map(|bound| bound.id().as_str().to_string())
        .collect();
    assert_eq!(
        ids,
        vec![
            "house-rules/a-file-nested-below-src-is-out-of-reach".to_string(),
            "house-rules/a-file-nested-below-src/bin-is-out-of-reach".to_string(),
        ],
        "one bound per governed subtree, named after it"
    );
    assert!(
        participant().bounds().len() == 1,
        "and a participant reading one subtree declares one"
    );
}

/// The bound this participant declares is the truth about it: a file nested below the governed subtree is
/// never read. Declaring a limit that does not exist would be worse than declaring none.
#[test]
fn a_file_nested_below_src_is_out_of_reach() {
    let nested = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("src/bin/demo.rs");
    assert!(nested.is_file(), "the fixture for this bound must exist");
    let Outcome::Violations(report) = participant().observe(&manifest()) else {
        panic!("the participant must react at all for this bound to mean anything");
    };
    assert!(
        !report
            .violations
            .iter()
            .any(|violation| violation.finding.contains("bin")),
        "nothing below `src/` may appear in the findings: {report:?}"
    );
}

/// The static half is a real boundary of its own, so the example is not teaching a participant beside an
/// inert declaration.
#[test]
fn the_declared_module_boundary_reacts_on_its_own() {
    assert_eq!(
        check_constitution(&constitution(), &manifest()).exit_code(),
        1
    );
}
