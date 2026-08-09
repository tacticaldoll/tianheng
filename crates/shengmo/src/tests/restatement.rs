//! The restatement judgement's failure matrix: what it refuses, and what it over-refuses.

use crate::restatement::{comment_block_copies_allowlist, comment_restates_the_declaration};

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
         over-reaction"
    );
    assert!(
        comment_restates_the_declaration(
            "// the shell's own restrict_dependencies_to(guibiao, hunyi) list"
        ),
        "the control in the other direction: a real restatement is what the reaction is for"
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
        "a block naming the members for another reason is refused — the declared over-reaction"
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
