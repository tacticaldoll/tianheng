//! Which tracked text is a **record** and which is live, in one place because more than one reader asks it.
//!
//! `AGENTS.md` names the records: a commit message, a dated `CHANGELOG.md` section, `docs/history/`. A record
//! is a measurement of its moment, so a figure or a citation inside one is provenance of a decision that was
//! made and stays readable as what it was. Live text is read later against a tree it must be able to address,
//! and is held to it.
//!
//! **The distinction was built twice, one round apart, and the two disagreed about the same document.** The
//! citation reader cut records by dated section; the census sweep read every tracked `.md` with no exemption
//! at all — so one reader treated `CHANGELOG.md`'s dated sections as records and the other refused a figure
//! inside one. A live instance was one paragraph reflow from turning a green tree red over a correct
//! historical record, which is a false refusal rather than a miss. Whichever reader needs this next asks
//! here.
//!
//! A commit message is not a tracked file and needs no row. The two that are files are here, and nothing
//! else: adding a document to escape a refusal is the move this module exists to make visible.

use crate::region::Source;
use std::collections::BTreeSet;

/// Documents that are records in whole, by path prefix.
pub const RECORD_PATHS: [&str; 1] = ["docs/history/"];

/// The document whose **dated** sections are records and whose undated ones are not.
///
/// Exempting the file was wider than `AGENTS.md` says: `## [Unreleased]` is live text by construction, and it
/// was exempt for standing in the same file as the releases below it.
pub const SECTIONED_RECORD: &str = "CHANGELOG.md";

/// Whether every line of the document at `path` is a record.
pub fn is_record_document(path: &str) -> bool {
    RECORD_PATHS.iter().any(|record| path.starts_with(record))
}

/// The one-based lines of `text` that are a record, for the document at `path`.
///
/// Empty for live text, every line for a record document, and the dated sections alone for the sectioned one.
/// Cut by [`crate::sections`], which owns cutting a flat sentinel-delimited document; the date is read by
/// [`crate::reading::date`], which owns the calendar. Neither question is answered again here.
pub fn record_lines(path: &str, text: &str) -> Records {
    if is_record_document(path) {
        return Records::Lines((1..=text.lines().count()).collect());
    }
    if path != SECTIONED_RECORD {
        return Records::Live;
    }
    let source = Source::of(text);
    let mut dated = BTreeSet::new();
    for section in crate::sections::cut(source.prose().numbered_lines(), |line| {
        line.trim_start().starts_with("## [").then_some(())
    }) {
        // **A version heading that carries no readable date is neither answer, and silence chose the wrong
        // one.** Both arms below used to `continue`, so a released section whose date was mistyped or missing
        // read as LIVE — the whole historical record then judged against today's tree, with the diagnostic
        // pointing at an entry inside it rather than at the heading. `[Unreleased]` is the one heading that
        // legitimately carries no date, and it is named rather than inferred.
        let name = section
            .line
            .trim_start()
            .trim_start_matches("## [")
            .split(']')
            .next()
            .expect("`str::split` yields at least one item");
        let Some((_, suffix)) = section.line.split_once("] - ") else {
            if name == "Unreleased" {
                continue;
            }
            return Records::Unreadable(format!(
                "{path}:{} heads a section `{}` with no ` - DATE`, so whether it is a record or live text \
                 cannot be decided — and reading it as live judges a released record against today's tree",
                section.start,
                section.line.trim()
            ));
        };
        if let Err(refusal) = crate::reading::date("changelog section date", suffix.trim()) {
            return Records::Unreadable(format!(
                "{path}:{} heads a section whose date this reader cannot read ({}), so whether it is a \
                 record or live text cannot be decided",
                section.start, refusal.message
            ));
        }
        dated.insert(section.start);
        for (line, _) in &section.body {
            dated.insert(*line);
        }
    }
    Records::Lines(dated)
}

/// Which lines of a document are a record, or that the question could not be decided.
///
/// **The third state is the finding.** With two, a heading this reader could not read fell to *live* — and a
/// released section read as live is a whole historical record judged against today's tree. Absent and
/// unreadable are two facts here as everywhere else in this crate; `capability_subjects::Declared` is the
/// precedent.
#[derive(Debug, PartialEq, Eq)]
pub enum Records {
    /// Every line is live text.
    Live,
    /// These one-based lines are a record; the rest are live.
    Lines(BTreeSet<usize>),
    /// A heading whose kind could not be decided, said in full so a caller refuses rather than guesses.
    Unreadable(String),
}

impl Records {
    /// Whether `line` is a record, treating an undecided document as carrying none.
    ///
    /// For a caller that has already turned [`Records::Unreadable`] into its own refusal and needs the lines
    /// only to finish the pass it was in.
    pub fn contains(&self, line: usize) -> bool {
        matches!(self, Records::Lines(lines) if lines.contains(&line))
    }
}
