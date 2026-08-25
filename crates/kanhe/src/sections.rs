//! Cutting a prose document into flat, sentinel-delimited sections — once, rather than per reader.
//!
//! **Five readers in this crate each rewrote the same skeleton.** Walk the lines; a sentinel line opens a
//! section; the next sentinel closes it; content before the first sentinel belongs to none. What differed was
//! never the skeleton — it was which line counts as a sentinel, and each reader carried its own state for the
//! skeleton anyway: an `inside: bool`, a `section: String` cursor, a `split` on one spelling followed by a
//! `split_once` on another, and a bare line count.
//!
//! **The predicate is a parameter and the level is not, which is a correction rather than a generalisation.**
//! An earlier design for this module admitted only a heading *level* (`##` / `###`), on the reasoning that
//! anything wider becomes a Markdown parser. Read against the callers, that interface serves none of them:
//! `release_coherence_gate`'s three readers open on `## [`-prefixed lines and take the name from the part
//! before `" - "`, while `capability_subjects`'s two open on one exact heading and close on any `## `. Those
//! are different *predicates* at the same level, so a level cannot tell them apart. What the parameter buys is
//! the skeleton; what it does not buy is block structure.
//!
//! **Flat, and that is the contract rather than a limitation to widen later.** A section runs to the next
//! sentinel, full stop. A **nested** delimiter — shell's `case`…`esac`, a Rust block — is not this module's
//! subject, and handing one to it reproduces a defect this repository has already paid for:
//! `wrapper_parser::parser_arms` bounded a nested `case` with a boolean, the inner `esac` closed the outer
//! read, and every arm after it left the map unannounced. Markdown headings do not nest, which is why a
//! boolean was *right* in `release_coherence_gate`'s readers and wrong in that one. The same shape, two grammars, opposite
//! verdicts — so the grammar is stated here instead of being inferred from whichever caller arrives next.
//!
//! Cuts a [`Prose`] region, never a bare `&str`: a fenced `## [Unreleased]` is not a
//! section, and every reader this replaces could be told otherwise.

use crate::region::Prose;

/// One flat section: the sentinel that opened it, where it opened, and the lines it holds until the next
/// sentinel.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Section {
    /// What the predicate named this section — the caller's own vocabulary, not a normalised form.
    pub name: String,
    /// The one-based line the sentinel sits on, in the original document.
    pub start: usize,
    /// The lines between this sentinel and the next, each with its own original position.
    pub body: Vec<(usize, String)>,
}

/// Cut `prose` at every line `sentinel` names, in document order.
///
/// Lines before the first sentinel belong to no section and are dropped: every reader this replaces already
/// skipped a document's preamble, one of them with an explicit `if section.is_empty() { continue }`. A
/// document whose first sentinel never appears yields an empty vector, which is the honest answer rather than
/// one section holding everything.
///
/// **Repeats are yielded, never merged.** Two sections the predicate names identically are two entries — that
/// is what lets a caller ask [`crate::selection::the_only`] and be refused. Collapsing them here would answer
/// *how many* on the caller's behalf, which is the habit `selection`'s own header exists to end.
pub fn cut(prose: Prose<'_>, sentinel: impl Fn(&str) -> Option<String>) -> Vec<Section> {
    let mut out: Vec<Section> = Vec::new();
    for (number, line) in prose.numbered_lines() {
        if let Some(name) = sentinel(&line) {
            out.push(Section {
                name,
                start: number,
                body: Vec::new(),
            });
            continue;
        }
        if let Some(open) = out.last_mut() {
            open.body.push((number, line));
        }
    }
    out
}
