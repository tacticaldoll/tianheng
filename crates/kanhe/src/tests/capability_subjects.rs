//! The subject declaration and the filing join, held to refusing each shape with its own message.

use std::collections::{BTreeMap, BTreeSet};

use crate::capability_subjects::{
    Declared, declaration_offences, join_offences, proposal_capabilities, subject_globs,
};
use crate::refusal::Kind;

fn specs(pairs: &[(&str, &str)]) -> BTreeMap<String, String> {
    pairs
        .iter()
        .map(|(k, v)| (k.to_string(), v.to_string()))
        .collect()
}

fn resolves(_glob: &str) -> Result<Vec<String>, String> {
    Ok(vec!["a/tracked/path.rs".to_string()])
}

fn resolves_nothing(_glob: &str) -> Result<Vec<String>, String> {
    Ok(Vec::new())
}

const WITH_SUBJECT: &str =
    "# c\n\n## Purpose\n\np\n\n## Subject\n\n- `crates/a/src/*.rs`\n\n## Requirements\n";

#[test]
fn a_capability_that_declares_no_subject_is_refused() {
    let offences = declaration_offences(
        &specs(&[("silent", "# c\n\n## Purpose\n\np\n\n## Requirements\n")]),
        resolves,
    );
    assert_eq!(offences.len(), 1);
    assert_eq!(offences[0].kind, Kind::Violation);
    assert!(offences[0].message.contains("declares no `## Subject`"));
}

#[test]
fn a_subject_section_listing_no_glob_is_refused() {
    let offences = declaration_offences(
        &specs(&[(
            "empty",
            "# c\n\n## Purpose\n\np\n\n## Subject\n\nnone yet.\n\n## Requirements\n",
        )]),
        resolves,
    );
    assert_eq!(offences.len(), 1);
    assert_eq!(offences[0].kind, Kind::Violation);
    assert!(offences[0].message.contains("listing no glob"));
}

#[test]
fn a_glob_matching_no_tracked_path_is_refused() {
    let offences = declaration_offences(&specs(&[("dead", WITH_SUBJECT)]), resolves_nothing);
    assert_eq!(offences.len(), 1);
    assert_eq!(offences[0].kind, Kind::Violation);
    assert!(offences[0].message.contains("matches no tracked path"));
}

#[test]
fn an_enumeration_that_fails_is_a_cannot_judge() {
    let offences = declaration_offences(&specs(&[("unreadable", WITH_SUBJECT)]), |_| {
        Err("git exploded".to_string())
    });
    assert_eq!(offences.len(), 1);
    assert_eq!(offences[0].kind, Kind::CannotJudge);
    assert!(offences[0].message.contains("could not resolve"));
}

#[test]
fn a_declared_and_resolving_subject_is_clean() {
    assert!(declaration_offences(&specs(&[("fine", WITH_SUBJECT)]), resolves).is_empty());
}

// --- the filing join ------------------------------------------------------------------------------------

fn claimed(pairs: &[(&str, &[&str])]) -> BTreeMap<String, BTreeSet<String>> {
    pairs
        .iter()
        .map(|(k, v)| (k.to_string(), v.iter().map(|s| s.to_string()).collect()))
        .collect()
}

fn listed(names: &[&str]) -> BTreeSet<String> {
    names.iter().map(|s| s.to_string()).collect()
}

/// The defect this join was written from, reconstructed: a shell wrapper filed under the capability whose
/// subject is this repository's checks.
#[test]
fn a_shell_wrapper_filed_under_the_rust_reaction_capability_is_refused() {
    let offences = join_offences(
        "a-gate-that-matched-no-test",
        &["scripts/publish.sh".to_string()],
        &listed(&["repository-checks"]),
        &claimed(&[
            ("publish-source-integrity", &["scripts/publish.sh"]),
            ("repository-checks", &["crates/kanhe/tests/x.rs"]),
        ]),
    );
    assert_eq!(offences.len(), 1);
    assert_eq!(offences[0].kind, Kind::Violation);
    assert!(offences[0].message.contains("publish-source-integrity"));
    assert!(offences[0].message.contains("scripts/publish.sh"));
}

/// Every claimant, not one. Naming one was the first rule and it could not catch the defect the join was
/// written from, because the two capabilities claiming that file overlap.
#[test]
fn naming_one_of_two_claiming_capabilities_does_not_satisfy_the_join() {
    let offences = join_offences(
        "c",
        &["shared.rs".to_string()],
        &listed(&["second"]),
        &claimed(&[("first", &["shared.rs"]), ("second", &["shared.rs"])]),
    );
    assert_eq!(offences.len(), 1);
    assert!(
        offences[0].message.contains("`first`"),
        "{}",
        offences[0].message
    );
    assert!(
        !offences[0].message.contains("`second` governs"),
        "{}",
        offences[0].message
    );
}

#[test]
fn accounting_for_both_claimants_satisfies_the_join() {
    assert!(
        join_offences(
            "c",
            &["shared.rs".to_string()],
            &listed(&["first", "second"]),
            &claimed(&[("first", &["shared.rs"]), ("second", &["shared.rs"])]),
        )
        .is_empty()
    );
}

/// The declared bound: subjects do not tile the repository, and a file no capability claims is not judged.
#[test]
fn a_file_no_capability_claims_is_not_judged() {
    assert!(
        join_offences(
            "c",
            &["unclaimed.txt".to_string()],
            &BTreeSet::new(),
            &claimed(&[("only", &["something/else.rs"])]),
        )
        .is_empty()
    );
}

#[test]
fn a_proposal_naming_nothing_says_so_rather_than_naming_an_empty_list() {
    let offences = join_offences(
        "c",
        &["governed.rs".to_string()],
        &BTreeSet::new(),
        &claimed(&[("owner", &["governed.rs"])]),
    );
    assert_eq!(offences.len(), 1);
    assert_eq!(offences[0].kind, Kind::Violation);
    assert!(offences[0].message.contains("names no capability"));
}

// --- reading the two documents -------------------------------------------------------------------------

#[test]
fn a_subject_block_ends_at_the_next_section() {
    assert_eq!(
        subject_globs(WITH_SUBJECT),
        Declared::Globs(vec!["crates/a/src/*.rs".to_string()])
    );
    assert_eq!(subject_globs("# c\n\n## Purpose\n\np\n"), Declared::Absent);
}

/// A bullet the reader cannot parse is refused, not quietly left out of the claim.
///
/// The three shapes that used to fall out of a `filter_map`: prose after the closing backtick, no backticks
/// at all, and an unterminated one. Each would have shrunk the capability's declared subject by exactly
/// itself — so `join_offences` would stop seeing every file that bullet claimed, and the capability would
/// govern less than it says while reading as a complete declaration.
#[test]
fn a_subject_bullet_that_does_not_parse_is_refused_rather_than_dropped() {
    for bullet in [
        "- `crates/a/src/*.rs` and everything under it",
        "- crates/a/src/*.rs",
        "- `crates/a/src/*.rs",
    ] {
        let spec = format!("# c\n\n## Subject\n\n{bullet}\n");
        assert_eq!(
            subject_globs(&spec),
            Declared::Unreadable(bullet.to_string()),
            "this bullet was read past instead of refused, which narrows the claim silently"
        );
    }
}

/// The refusal reaches the verdict as a **cannot-judge**, not as a disagreement.
///
/// The section may well claim exactly the right files; this reader cannot say. Reporting it as a violation
/// would send someone to fix a declaration that may be correct, and reporting a shorter glob list would be
/// the silent narrowing itself.
#[test]
fn an_unreadable_subject_bullet_is_a_cannot_judge() {
    let specs = BTreeMap::from([(
        "c".to_string(),
        "# c\n\n## Subject\n\n- `crates/a/*.rs` and more\n".to_string(),
    )]);
    let offences = declaration_offences(&specs, |_| Ok(vec!["crates/a/src/lib.rs".to_string()]));
    assert_eq!(offences.len(), 1, "{offences:?}");
    assert_eq!(offences[0].kind, Kind::CannotJudge);
    assert!(
        offences[0].message.contains("does not understand"),
        "{}",
        offences[0].message
    );
}

#[test]
fn a_capability_named_outside_the_capabilities_section_is_not_read_as_named() {
    let proposal = "## Why\n\nTouching `elsewhere`.\n\n## Capabilities\n\n- `here`: a reason\n\n## Impact\n\n`later`\n";
    assert_eq!(proposal_capabilities(proposal), listed(&["here"]));
}
