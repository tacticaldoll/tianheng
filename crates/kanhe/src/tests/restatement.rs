//! The restatement judgement's failure matrix: what it refuses, and what it over-refuses.

use crate::region::Source;
use crate::restatement::{
    comment_block_copies_allowlist, comment_restates_the_declaration, document_offences,
};

/// `self-law-projection/a-doc-example-of-the-dependency-dsl-is-refused-a-stated-bound`
///
/// The recognizer reads a comment's **text**, never its purpose, so a rustdoc example teaching the re-exported
/// DSL is refused exactly as a restatement of the shell's own declaration would be. That is the safe direction —
/// a false positive is a sentence to rewrite, where the false negative would be a copied declaration nothing
/// governs — and the shell publishes this DSL, so the shape is live even with no instance in the tree today.
///
/// The control matters as much: a comment discussing the boundary without naming the call is accepted, so this
/// shows a limit of reading text rather than a recognizer that refuses every comment.
#[test]
fn a_doc_example_of_the_dependency_dsl_is_refused() {
    assert!(
        comment_restates_the_declaration(
            "/// CrateBoundary::crate_(\"x\").restrict_dependencies_to([\"y\"])"
        ),
        "a doc example of the DSL is refused, though it restates nothing about this shell — the declared \
         false refusal"
    );
    assert!(
        comment_restates_the_declaration(
            "// the shell's own restrict_dependencies_to(guibiao, hunyi) list"
        ),
        "the control in the other direction: a real restatement is what the check is for"
    );
    assert!(
        !comment_restates_the_declaration(
            "// the shell's dependency allowlist, see AGENTS.self-law.md"
        ),
        "and a comment explaining the boundary without naming the call is accepted, so the refusals above \
         are a limit of reading text rather than a recognizer that refuses everything"
    );
}

/// `self-law-projection/a-comment-naming-every-member-for-another-reason-is-refused-a-stated-bound`
///
/// The block check asks whether every allowlist member appears, never why, so a block naming them for a
/// different purpose — a crate-level note on what the shell composes, say — reads the same as a copied census.
/// Kept over-reacting rather than taught to read intent: the alternative is a heuristic over prose, which this
/// repository has measured and rejected elsewhere.
#[test]
fn a_comment_naming_every_member_for_another_reason_is_refused() {
    let allowlist: Vec<String> = ["guibiao", "hunyi", "louke", "serde_json"]
        .into_iter()
        .map(str::to_string)
        .collect();
    assert!(
        comment_block_copies_allowlist(
            "//! The shell composes guibiao, hunyi and louke, and serializes its report with serde_json.\n",
            &allowlist
        ),
        "a block naming the members for another reason is refused — the declared false refusal"
    );
    assert!(
        !comment_block_copies_allowlist(
            "//! The shell composes guibiao, hunyi and louke.\n",
            &allowlist
        ),
        "the control: naming some of them is accepted, so the refusal above is about the full set and not \
         about mentioning a crate at all"
    );
}

// --- the census a document must not reproduce ---------------------------------------------------------

fn lists() -> Vec<(String, Vec<String>)> {
    vec![
        (
            "guibiao".to_string(),
            ["serde_json", "xuanji", "xingbiao"]
                .map(str::to_string)
                .to_vec(),
        ),
        ("xuanji".to_string(), vec!["serde_json".to_string()]),
    ]
}

#[test]
fn a_block_naming_a_crate_and_every_member_of_its_allowlist_is_refused() {
    let source =
        Source::of("- `guibiao` depends on `xuanji`, `xingbiao`, and `serde_json` only.\n");
    let offences = document_offences("PROBE.md", source.prose(), &lists());
    assert_eq!(offences.len(), 1, "{offences:?}");
    assert!(offences[0].contains("guibiao"), "{offences:?}");
}

/// `self-law-projection`'s bound: a membership reproduced without naming the crate it governs is not
/// observed. Reading it as one would refuse any paragraph listing the same crate names for another reason.
#[test]
fn a_block_naming_the_members_but_not_the_crate_is_not_observed() {
    assert!(
        document_offences(
            "PROBE.md",
            Source::of("- The bases are `xuanji`, `xingbiao`, and `serde_json`.\n").prose(),
            &lists(),
        )
        .is_empty()
    );
}

/// The same bound's other half: an allowlist of one cannot be told from a mention of that crate.
#[test]
fn a_single_member_allowlist_is_not_observed() {
    let source = Source::of("- `xuanji` needs `serde_json`.\n");
    assert!(document_offences("PROBE.md", source.prose(), &lists()).is_empty());
}

/// A list is read one item at a time, so a census assembled across separate entries is not one block.
#[test]
fn a_census_spread_across_separate_items_is_not_one_block() {
    assert!(
        document_offences(
            "PROBE.md",
            Source::of("- `guibiao` reads through `xingbiao`.\n- It also uses `xuanji`.\n- And `serde_json`.\n")
                .prose(),
            &lists(),
        )
        .is_empty()
    );
}

/// A block's reported line is the document's own, across a fence that prose drops.
///
/// **The start cannot be derived from the blank line before it.** The old reader counted its own lines and
/// set `block_start = number + 1` on a blank, which names *the line after this one* — true only while every
/// following line survives. Prose drops a fenced block entirely, so that arithmetic points into the fence and
/// the offence cites a line the reader cannot find. Taking the start from the first line the block actually
/// holds is right in both readings.
///
/// The fixture puts the fence *between* the blank and the block, which is the only arrangement where the two
/// rules disagree: with the block opening on a list item both answers are the current line, so a fixture
/// built that way would pass either way.
#[test]
fn a_blocks_reported_line_survives_a_fence_above_it() {
    let source = Source::of(
        "intro\n\
         \n\
         ```text\n\
         a fenced sample\n\
         ```\n\
         `guibiao` depends on `xuanji`, `xingbiao`, and `serde_json` only.\n",
    );
    let offences = document_offences("PROBE.md", source.prose(), &lists());
    assert_eq!(offences.len(), 1, "{offences:?}");
    assert!(
        offences[0].contains("PROBE.md:6"),
        "the block opens on the document's line 6; deriving it from the blank line at 2 names line 3, which \
         is inside the fence prose just dropped. Got: {}",
        offences[0]
    );
}
