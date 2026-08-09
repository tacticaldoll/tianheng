//! Whether authored text restates a declaration the law already owns.
//!
//! The membership of an allowlist belongs to the declaration and its projection; a second copy in
//! prose is a source of truth that goes stale with no reaction to say so.

use std::path::Path;

/// Whether one comment line restates the shell's dependency declaration.
///
/// Named and taking a line, so the shape it refuses — and the shape it over-refuses — can be shown by giving it
/// text rather than by editing the shell until it trips. Its over-reaction is a declared bound of
/// `self-law-projection`, pinned by [`a_doc_example_of_the_dependency_dsl_is_refused`].
pub fn comment_restates_the_declaration(line: &str) -> bool {
    line.contains("restrict_dependencies_to(")
}

/// Whether one contiguous comment block names every member of the live allowlist.
///
/// Its over-reaction is likewise declared, pinned by
/// [`a_comment_naming_every_member_for_another_reason_is_refused`]: the question it answers is whether the
/// members all appear, never why, so a block naming them for a different purpose reads the same as a copy.
pub fn comment_block_copies_allowlist(block: &str, allowlist: &[String]) -> bool {
    !allowlist.is_empty()
        && allowlist.iter().all(|member| {
            block
                .split(|character: char| {
                    !(character.is_ascii_alphanumeric() || character == '_' || character == '-')
                })
                .any(|token| token == member)
        })
}

pub fn assert_comment_block_does_not_copy_allowlist(
    source: &Path,
    block_start: usize,
    block: &str,
    allowlist: &[String],
) {
    assert!(
        !comment_block_copies_allowlist(block, allowlist),
        "{}:{} names every live shell dependency allowlist member ({}) inside one line-comment block; \
         refer to AGENTS.self-law.md instead. This asks whether the members all appear, never why, so it \
         also refuses a block naming them for another reason — a declared over-reaction of \
         `self-law-projection`, not a case to work around",
        source.display(),
        block_start,
        allowlist.join(", ")
    );
}
