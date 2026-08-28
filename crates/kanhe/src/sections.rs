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
//! `release_coherence_gate`'s changelog reader opens on `## [`-prefixed lines and takes the name from the
//! part before `" - "`; its manifest and lock readers open on a bracketed TOML table, which is not a heading
//! at all; and `capability_subjects`'s readers open on one exact heading and close on any `## `. Those are
//! different *predicates*, and two of them are not even in the same document format, so a heading level
//! cannot tell them apart. What the parameter buys is
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
//! **Takes numbered lines rather than one region kind, and that is what keeps this to one skeleton.** The
//! readers it replaces span two grammars: Markdown headings over a [`Prose`](crate::region::Prose) region, and TOML table headings
//! over a [`toml`](crate::region::Source::toml) one. Both hand out `(position, line)`; both then need
//! *sentinel opens, next sentinel closes, preamble belongs to none*. Writing a second `cut` for the second
//! grammar would be this module's own subject performed on itself — the skeleton is the shared thing, and the
//! grammar is a parameter exactly as the predicate is. The line type is generic for the same reason and no
//! other: `Prose` hands out owned text because excising a comment span builds a new string, and `Executed`
//! hands out borrowed slices because cutting a tail comment does not. Requiring one of them would push a
//! `.map()` onto every call site of the other kind.
//!
//! A region either way, never a bare `&str`. On the Markdown side a fenced `## [Unreleased]` is not a section;
//! on the TOML side a `[table]` inside a string or behind a `#` is not a table. Every reader this replaces
//! could be told otherwise.

/// One flat section: the sentinel that opened it, where it opened, and the lines it holds until the next
/// sentinel.
///
/// **Generic in what the predicate answers, defaulting to a name.** A Markdown heading names its section with
/// a string; a TOML table heading answers a *classification* — `[dependencies]`, `[dependencies.NAME]` and
/// anything else are three different things to the reader walking them, and collapsing those into a string
/// would put the caller back to re-deciding from the text. `T` is whatever the predicate returns, and the
/// default keeps every existing call site spelled `Section`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Section<T = String> {
    /// What the predicate named this section — the caller's own vocabulary, not a normalised form.
    pub name: T,
    /// The sentinel line as the document wrote it.
    ///
    /// **A name a predicate derives can be lossy, and one of them is.** `release_coherence_gate`'s predicate
    /// drops a ` - DATE` suffix, so a section it names `## [0.5.0]` was written with a date after that
    /// heading, and a reader asking *which date* has nothing to ask. The date is deliberately not spelled
    /// here: it is a live value in `CHANGELOG.md`, and a doc comment naming it goes stale on the day of the
    /// cut — which is when nobody is reading doc comments. Carrying the line makes the cut lossless: a
    /// caller that wants the derived name takes [`Section::name`], and one that wants what was written takes
    /// this. Keeping only the name would push every such caller back onto its own line walk, which is the
    /// habit this module exists to end.
    pub line: String,
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
pub fn cut<T, S: AsRef<str> + Into<String>>(
    lines: impl Iterator<Item = (usize, S)>,
    sentinel: impl Fn(&str) -> Option<T>,
) -> Vec<Section<T>> {
    let mut out: Vec<Section<T>> = Vec::new();
    for (number, line) in lines {
        if let Some(name) = sentinel(line.as_ref()) {
            out.push(Section {
                name,
                line: line.into(),
                start: number,
                body: Vec::new(),
            });
            continue;
        }
        if let Some(open) = out.last_mut() {
            open.body.push((number, line.into()));
        }
    }
    out
}
