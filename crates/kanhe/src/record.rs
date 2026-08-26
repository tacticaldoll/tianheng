//! Which tracked text is a **record** and which is live, in one place because two readers ask it.
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
pub fn record_lines(path: &str, text: &str) -> BTreeSet<usize> {
    if is_record_document(path) {
        return (1..=text.lines().count()).collect();
    }
    if path != SECTIONED_RECORD {
        return BTreeSet::new();
    }
    let source = Source::of(text);
    let mut dated = BTreeSet::new();
    for section in crate::sections::cut(source.prose().numbered_lines(), |line| {
        line.trim_start().starts_with("## [").then_some(())
    }) {
        let Some((_, suffix)) = section.line.split_once("] - ") else {
            continue;
        };
        if crate::reading::date("changelog section date", suffix.trim()).is_err() {
            continue;
        }
        dated.insert(section.start);
        for (line, _) in &section.body {
            dated.insert(*line);
        }
    }
    dated
}
