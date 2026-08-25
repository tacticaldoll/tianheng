//! Whether authored text restates a declaration the law already owns.
//!
//! The membership of an allowlist belongs to the declaration and its projection; a second copy in
//! prose is a source of truth that goes stale with no check to say so.

use std::path::Path;

use tianheng::{Boundary, Rule};

/// Whether one comment line restates the shell's dependency declaration.
///
/// Named and taking a line, so the shape it refuses — and the shape it over-refuses — can be shown by giving it
/// text rather than by editing the shell until it trips. Its false refusal is a declared bound of
/// `self-law-projection`, pinned by `a_doc_example_of_the_dependency_dsl_is_refused`.
pub fn comment_restates_the_declaration(line: &str) -> bool {
    line.contains("restrict_dependencies_to(")
}

/// Whether one contiguous comment block names every member of the live allowlist.
///
/// Its false refusal is likewise declared, pinned by
/// `a_comment_naming_every_member_for_another_reason_is_refused`: the question it answers is whether the
/// members all appear, never why, so a block naming them for a different purpose reads the same as a copy.
pub fn comment_block_copies_allowlist(block: &str, allowlist: &[String]) -> bool {
    !allowlist.is_empty() && allowlist.iter().all(|member| names(block, member))
}

/// Whether `block` carries `word` as a **whole token**, under this module's one tokenizing rule.
///
/// It lived twice, byte-for-byte: once inside [`comment_block_copies_allowlist`]'s `all`, once as
/// [`names_the_crate`]'s whole body. Both decide the same question about the same corpus — does this comment
/// name that identifier — so a change to what counts as a token (admitting `.`, or dropping ascii-only) had
/// to be made in both or the two would disagree about the same block.
///
/// Deliberately **not** shared with `bound_register_parse`'s tokenizers, which look alike and are not: that
/// reader splits on unicode alphanumerics and admits `-` but not `_`, because it is reading English prose
/// rather than Rust identifiers. One predicate per question, not one predicate per resemblance.
fn names(block: &str, word: &str) -> bool {
    block
        .split(|character: char| {
            !(character.is_ascii_alphanumeric() || character == '_' || character == '-')
        })
        .any(|token| token == word)
}

/// Refuse a contiguous line-comment block that restates the whole of a declared allowlist.
///
/// A copied allowlist is a second declaration that nothing holds to the first, so it drifts silently. The
/// refusal names the source and the block's first line.
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
         also refuses a block naming them for another reason — a declared false refusal of \
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
    names(block, crate_name)
}

/// Every contiguous block of `text` that names every member of some declared allowlist.
///
/// A block is what a reader takes in at once — a paragraph, a bullet, a comment run — so the question is
/// whether the census appears together, not whether the words appear at all in a long document.
pub fn document_offences(
    path: &str,
    prose: crate::region::Prose<'_>,
    allowlists: &[(String, Vec<String>)],
) -> Vec<String> {
    let mut offences = Vec::new();
    let mut block = String::new();
    // `0` is *no block open*. The start is the first line the block actually holds, taken from the document
    // rather than derived from the blank line before it: prose drops a fenced block's lines entirely, so
    // "the line after this blank one" is not a position the reader can find once a fence follows.
    let mut block_start = 0usize;
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
    for (number, line) in prose.numbered_lines() {
        // A blank line ends a block, and so does the start of a list item: a reader takes in one bullet,
        // not the whole list, and treating a list as one block reported a census that no single entry
        // carries — measured, it named four crates' allowlists against one architecture section.
        let trimmed = line.trim_start();
        let starts_an_item = trimmed.starts_with("- ") || trimmed.starts_with("* ");
        if line.trim().is_empty() || starts_an_item {
            flush(&block, block_start, &mut offences);
            block.clear();
            block_start = 0;
        }
        if !line.trim().is_empty() {
            if block_start == 0 {
                block_start = number;
            }
            block.push_str(&line);
            block.push('\n');
        }
    }
    flush(&block, block_start, &mut offences);
    offences
}
