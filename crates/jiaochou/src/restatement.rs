//! Whether authored text restates a declaration the law already owns.
//!
//! The membership of an allowlist belongs to the declaration and its projection; a second copy in
//! prose is a source of truth that goes stale with no reaction to say so.

use std::path::Path;

use tianheng::{Boundary, Rule};

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

/// Every declared dependency allowlist, keyed by the crate it governs.
///
/// Produced from the live constitution rather than listed, so a crate added to the law is read here without
/// anyone remembering this file.
pub fn allowlists(boundaries: &[Boundary]) -> Vec<(String, Vec<String>)> {
    boundaries
        .iter()
        .filter_map(|boundary| match boundary {
            Boundary::Crate(crate_boundary) => match crate_boundary.rule() {
                Rule::RestrictDependenciesTo { allowed, .. } => {
                    Some((crate_boundary.target().package.clone(), allowed.to_vec()))
                }
                _ => None,
            },
            _ => None,
        })
        .collect()
}

/// Whether a block names the crate whose allowlist is being checked.
fn names_the_crate(block: &str, crate_name: &str) -> bool {
    block
        .split(|character: char| {
            !(character.is_ascii_alphanumeric() || character == '_' || character == '-')
        })
        .any(|token| token == crate_name)
}

/// Every contiguous block of `text` that names every member of some declared allowlist.
///
/// A block is what a reader takes in at once — a paragraph, a bullet, a comment run — so the question is
/// whether the census appears together, not whether the words appear at all in a long document.
pub fn document_offences(
    path: &str,
    text: &str,
    allowlists: &[(String, Vec<String>)],
) -> Vec<String> {
    let mut offences = Vec::new();
    let mut block = String::new();
    let mut block_start = 1usize;
    let mut number = 0usize;
    let flush = |block: &str, start: usize, offences: &mut Vec<String>| {
        if block.trim().is_empty() {
            return;
        }
        for (crate_name, allowlist) in allowlists {
            // A census is a list. An allowlist of one degenerates into "this block mentions that crate
            // name", which is not a restatement of anything — measured, widening without this floor read
            // every paragraph naming `serde_json` as a copy of 璇璣's declaration. And the block must name
            // the crate it governs, or it is a list of names rather than a claim about that crate's
            // dependencies. What both cost is a declared bound: a two-member allowlist reproduced without
            // naming its crate is not observed.
            if allowlist.len() < 2 || !names_the_crate(block, crate_name) {
                continue;
            }
            if comment_block_copies_allowlist(block, allowlist) {
                offences.push(format!(
                    "  {path}:{start} names every member of `{crate_name}`'s live dependency allowlist \
                     ({}) in one block; the membership is owned by the declaration and rendered by \
                     `AGENTS.self-law.md` — point there rather than restating it",
                    allowlist.join(", ")
                ));
            }
        }
    };
    for line in text.lines() {
        number += 1;
        // A blank line ends a block, and so does the start of a list item: a reader takes in one bullet,
        // not the whole list, and treating a list as one block reported a census that no single entry
        // carries — measured, it named four crates' allowlists against one architecture section.
        let trimmed = line.trim_start();
        let starts_an_item = trimmed.starts_with("- ") || trimmed.starts_with("* ");
        if line.trim().is_empty() || starts_an_item {
            flush(&block, block_start, &mut offences);
            block.clear();
            block_start = number;
        }
        if line.trim().is_empty() {
            block_start = number + 1;
        } else {
            block.push_str(line);
            block.push('\n');
        }
    }
    flush(&block, block_start, &mut offences);
    offences
}
