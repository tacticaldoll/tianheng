//! The subject declaration and the filing join, held to refusing each shape with its own message.

use std::collections::{BTreeMap, BTreeSet};

use crate::capability_subjects::{
    Declared, Named, declaration_offences, join_offences, proposal_capabilities, subject_globs,
};
use crate::refusal::Kind;

/// The reader over a document, given as text here because a fixture is a literal.
///
/// The cut needs a [`Source`](crate::region::Source) to borrow from, so the region is built per call rather
/// than threaded through every fixture — the same reason `declaration_offences` builds one per spec.
fn globs_of(spec: &str) -> Declared {
    let source = crate::region::Source::of(spec);
    subject_globs(source.prose())
}

/// Sibling of [`globs_of`], for the proposal side.
fn capabilities_of(proposal: &str) -> Named {
    let source = crate::region::Source::of(proposal);
    proposal_capabilities(source.prose())
}

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
    crate::refusal::expect(
        "repository-checks#capability-declares-no-subject",
        &offences[0],
    );
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
    crate::refusal::expect(
        "repository-checks#capability-subject-lists-no-glob",
        &offences[0],
    );
    assert!(offences[0].message.contains("listing no glob"));
}

#[test]
fn a_glob_matching_no_tracked_path_is_refused() {
    let offences = declaration_offences(&specs(&[("dead", WITH_SUBJECT)]), resolves_nothing);
    assert_eq!(offences.len(), 1);
    assert_eq!(offences[0].kind, Kind::Violation);
    crate::refusal::expect(
        "repository-checks#capability-subject-glob-matches-nothing",
        &offences[0],
    );
    assert!(offences[0].message.contains("matches no tracked path"));
}

#[test]
fn an_enumeration_that_fails_is_a_cannot_judge() {
    let offences = declaration_offences(&specs(&[("unreadable", WITH_SUBJECT)]), |_| {
        Err("git exploded".to_string())
    });
    assert_eq!(offences.len(), 1);
    assert_eq!(offences[0].kind, Kind::CannotJudge);
    crate::refusal::expect(
        "repository-checks#capability-subject-glob-unresolvable",
        &offences[0],
    );
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
    crate::refusal::expect(
        "repository-checks#change-touches-a-governed-path-unaccounted",
        &offences[0],
    );
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
    crate::refusal::expect(
        "repository-checks#change-touches-a-governed-path-unaccounted",
        &offences[0],
    );
    assert!(offences[0].message.contains("names no capability"));
}

// --- reading the two documents -------------------------------------------------------------------------

#[test]
fn a_subject_block_ends_at_the_next_section() {
    assert_eq!(
        globs_of(WITH_SUBJECT),
        Declared::Globs(vec!["crates/a/src/*.rs".to_string()])
    );
    assert_eq!(globs_of("# c\n\n## Purpose\n\np\n"), Declared::Absent);
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
            globs_of(&spec),
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
    crate::refusal::expect(
        "repository-checks#capability-subject-bullet-unreadable",
        &offences[0],
    );
    assert!(
        offences[0].message.contains("does not understand"),
        "{}",
        offences[0].message
    );
}

/// Two `## Subject` sections are refused, not silently read as the first.
///
/// The falsifier is the second section — the candidate the reader would have dropped. `.nth(1)` took the text
/// after the first marker and bounded it at the next `## `, so a capability declaring a second section
/// governed strictly less than it says while reading as a complete declaration, and the filing join then
/// missed every file those globs claim. That is the same silent narrowing the bullet loop inside this
/// function already refuses, one level up from it — and the reader was correct only while a second section
/// happened not to exist.
#[test]
fn two_subject_sections_are_refused_rather_than_read_as_the_first() {
    let spec = "# c\n\n## Subject\n\n- `crates/a/src/*.rs`\n\n## Purpose\n\np\n\n## Subject\n\n\
                - `crates/b/src/*.rs`\n\n## Requirements\n";
    let Declared::SeveralSections(count) = globs_of(spec) else {
        panic!(
            "a second `## Subject` section was read past: {:?}",
            globs_of(spec)
        );
    };
    assert_eq!(count, 2);
}

/// And the refusal reaches the verdict as a cannot-judge naming the count, not as the bullet message.
#[test]
fn several_subject_sections_are_a_cannot_judge_of_their_own() {
    let spec = "# c\n\n## Subject\n\n- `crates/a/src/*.rs`\n\n## Purpose\n\np\n\n## Subject\n\n\
                - `crates/b/src/*.rs`\n\n## Requirements\n";
    let offences = declaration_offences(&specs(&[("twice", spec)]), resolves);
    assert_eq!(offences.len(), 1, "{offences:?}");
    assert_eq!(offences[0].kind, Kind::CannotJudge);
    crate::refusal::expect(
        "repository-checks#capability-declares-several-subjects",
        &offences[0],
    );
    assert!(
        offences[0].message.contains("2 `## Subject` sections"),
        "the refusal must name the count rather than reuse the unreadable-bullet wording: {}",
        offences[0].message
    );
}

/// Two `## Capabilities` sections in one proposal are refused for the same reason, one document over.
///
/// Both markers carry a leading newline, as every real proposal's do — a section opening the file's very
/// first line is invisible to this reader either way, which is unchanged here and is why the fixture writes
/// a title.
#[test]
fn two_capabilities_sections_are_refused_rather_than_read_as_the_first() {
    let proposal =
        "# p\n\n## Capabilities\n\n- `first`\n\n## Why\n\nw\n\n## Capabilities\n\n- `second`\n";
    assert_eq!(
        capabilities_of(proposal),
        Named::SeveralSections(2),
        "the second section's capabilities were dropped, so a change could name one and be filed as complete"
    );
    let single = "## Why\n\nw\n\n## Capabilities\n\n- `only`\n";
    assert_eq!(capabilities_of(single), Named::Names(listed(&["only"])));
}

#[test]
fn a_capability_named_outside_the_capabilities_section_is_not_read_as_named() {
    let proposal = "## Why\n\nTouching `elsewhere`.\n\n## Capabilities\n\n- `here`: a reason\n\n## Impact\n\n`later`\n";
    assert_eq!(capabilities_of(proposal), Named::Names(listed(&["here"])));
}

/// A marker that closes nothing is refused, where it used to shift every pair after it.
///
/// **Measured before the repair:** this exact section answered `{" here\n- ", "alpha"}` — the prose between
/// the stray marker and `beta`'s opener admitted as a capability name, and `beta` dropped. The sibling
/// [`subject_globs`] refused the analogous bullet, so one module gave one rule two answers; the error channel
/// being a bare `usize` is why.
#[test]
fn a_backtick_that_closes_nothing_is_refused_rather_than_shifting_every_pair() {
    let proposal = "# p\n\n## Capabilities\n\n- `alpha`\n- a stray ` here\n- `beta`\n";
    match capabilities_of(proposal) {
        Named::Unreadable(message) => assert!(
            message.contains("closes nothing"),
            "the refusal must say what is wrong with the section, got {message:?}"
        ),
        other => panic!(
            "an unpaired marker shifts every pair after it and is not a reading, got {other:?}"
        ),
    }
    // Paired again, the same section reads both names — so the refusal is about the marker, not the prose.
    let paired = "# p\n\n## Capabilities\n\n- `alpha`\n- a stray `x` here\n- `beta`\n";
    assert_eq!(
        capabilities_of(paired),
        Named::Names(listed(&["alpha", "beta", "x"]))
    );
}

/// A fenced `## Subject` opens no section, so the misread these readers were filed for is gone.
///
/// **This is the WHEN of a retired protection, re-run rather than deleted.**
/// `the_corpora_of_the_bare_str_markdown_readers_carry_no_fence_or_comment_span` held that no tracked spec
/// carried a fenced block, because these two readers walked a document's lines and a fenced `## Subject`
/// opened a section for them. They read a prose region now, so that shape is decided here instead of being
/// kept out of the corpus — which is what lets the protection go rather than outlive its instance.
///
/// Three shapes in one fixture, because each fails in its own direction: a **fenced** heading must open
/// nothing, a **real** one after it must still be found, and the fenced heading's own bullet must not reach
/// the real section's globs. A fixture carrying only the first passes for an implementation that finds no
/// section at all.
#[test]
fn a_fenced_subject_heading_opens_no_section() {
    let fenced = "\
# capability

An example of what a subject section looks like:

```
## Subject

- `crates/example/**/*.rs`
```

## Subject

- `crates/real/**/*.rs`
";
    assert_eq!(
        globs_of(fenced),
        Declared::Globs(vec!["crates/real/**/*.rs".to_string()]),
        "the fenced heading opened no section, so this is one `## Subject` rather than two — and the glob \
         inside the fence is not among the claimed set"
    );

    // The same document read as a bare line walk answered `SeveralSections(2)`, refusing a conforming spec.
    // Given here as the *other* direction of the same fixture: over-exclusion would have been a false
    // refusal, and under-exclusion a claimed set naming a file no capability governs.
    let comment_span = "# capability\n\n<!--\n## Subject\n\n- `crates/hidden/**/*.rs`\n-->\n\n## Subject\n\n- `crates/real/**/*.rs`\n";
    assert_eq!(
        globs_of(comment_span),
        Declared::Globs(vec!["crates/real/**/*.rs".to_string()]),
        "an HTML comment span is invisible to a reader, so the heading and bullet inside it are not the \
         capability's declaration"
    );
}
