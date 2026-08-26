//! Which text is a record: the cut two readers now share.

use crate::record::{Records, is_record_document, record_lines};

/// A record document is a record in whole, and a live one carries none.
#[test]
fn a_record_document_is_a_record_in_whole() {
    assert!(is_record_document(
        "docs/history/published-artifact-provenance.md"
    ));
    assert!(!is_record_document("BACKLOG.md"));
    assert!(!is_record_document("CHANGELOG.md"));

    let text = "one\ntwo\nthree\n";
    assert_eq!(
        record_lines("docs/history/anything.md", text),
        Records::Lines((1..=3).collect()),
        "every line of a record document is a record"
    );
    assert_eq!(
        record_lines("BACKLOG.md", text),
        Records::Live,
        "a live document carries no record lines"
    );
}

/// A dated changelog section is a record; an undated one, and a heading naming no real date, are not.
///
/// **The date is the calendar's, not a separator's.** The first version of this cut tested the heading for
/// `] - ` — so `## [0.6.0] - TBD`, a section being prepared, became a record and everything under it went
/// unread. The second ranged the fields, which still admitted a day the calendar does not have. Both are here
/// as inputs that must NOT be records.
#[test]
fn a_dated_section_is_a_record_and_the_date_is_the_calendar_s() {
    let text = "\
## [Unreleased]
live one
## [0.6.0] - TBD
placeholder one
## [0.5.0] - 2026-02-31
impossible one
## [0.4.0] - 2026-08-04
recorded one
";
    // The placeholder and the impossible date are now UNDECIDED rather than silently live: a version
    // heading carrying no readable date is neither a record nor live text, and reading it as live puts a
    // released record in front of today's tree.
    let undecided = record_lines("CHANGELOG.md", text);
    assert!(
        matches!(undecided, Records::Unreadable(_)),
        "a version heading with no readable date is undecided, got {undecided:?}"
    );
    // With only real dates and `[Unreleased]`, the cut is the dated section and nothing else.
    let clean = "\
## [Unreleased]
live one
## [0.4.0] - 2026-08-04
recorded one
";
    assert_eq!(
        record_lines("CHANGELOG.md", clean),
        Records::Lines([3usize, 4].into_iter().collect()),
        "the dated section is the record and nothing else is"
    );
    assert_eq!(
        record_lines("BACKLOG.md", text),
        Records::Live,
        "the same headings in a document that is not the sectioned record carry no records"
    );
}
