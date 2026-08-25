//! The cut's own failure matrix: what it yields, what it drops, and what it refuses to decide.
//!
//! Written because the module landed with only *indirect* coverage — the release gate's directions exercise
//! it through `judge`, which cannot say whether a preamble was dropped or a repeat was merged, only whether
//! the verdict came out the same. A component's contract is not a consequence of its callers agreeing.

use crate::region::Source;
use crate::sections::{Section, cut};

/// The predicate `release_coherence_gate` uses, restated here so this matrix does not depend on its private
/// spelling: a `## [` line, named by the part before any ` - `.
fn version_section(line: &str) -> Option<String> {
    line.starts_with("## [").then(|| {
        line.split_once(" - ")
            .map_or(line, |(head, _)| head)
            .trim_end()
            .to_string()
    })
}

/// Lines before the first sentinel belong to no section, and a document with no sentinel yields nothing.
///
/// Both halves, because one alone passes for the wrong reason: an implementation that opened an implicit
/// section at line 1 would satisfy "no sentinel yields nothing" only if it also dropped the preamble, and one
/// that dropped the preamble but opened an implicit section would satisfy neither honestly.
#[test]
fn a_preamble_belongs_to_no_section_and_no_sentinel_yields_nothing() {
    let with_preamble =
        Source::of("# Changelog\n\nsome intro prose\n\n## [0.1.0] - 2020-01-01\n- a thing\n");
    let sections = cut(with_preamble.prose().numbered_lines(), version_section);
    assert_eq!(sections.len(), 1, "{sections:?}");
    assert_eq!(sections[0].name, "## [0.1.0]");
    assert_eq!(
        sections[0].body,
        vec![(6, "- a thing".to_string())],
        "the preamble is not in any body, and the section's body starts at the document's line 6"
    );

    let no_sentinel = Source::of("# Changelog\n\nnothing but prose\n");
    assert_eq!(
        cut(no_sentinel.prose().numbered_lines(), version_section),
        Vec::new(),
        "an empty vector, not one section holding the whole document — a document with no sentinel declares \
         no section, and answering otherwise would invent one"
    );
}

/// Two sections the predicate names identically are two entries, so a caller can be refused for the count.
///
/// Merging them here would answer *how many* on the caller's behalf. `require_changelog_state` asks exactly
/// that question about `## [Unreleased]` and must be able to see two.
#[test]
fn a_repeated_name_yields_two_sections_rather_than_one() {
    let doubled = Source::of(
        "## [Unreleased]\n- one\n## [0.1.0] - 2020-01-01\n- two\n## [Unreleased]\n- three\n",
    );
    let sections = cut(doubled.prose().numbered_lines(), version_section);
    assert_eq!(
        sections
            .iter()
            .filter(|s| s.name == "## [Unreleased]")
            .count(),
        2,
        "{sections:?}"
    );
    assert_eq!(
        crate::selection::the_only(
            "[Unreleased] section",
            sections.iter().filter(|s| s.name == "## [Unreleased]")
        )
        .map(|s| s.start)
        .map_err(|r| r.message.contains("found 2")),
        Err(true),
        "and the count reaches `selection::the_only`, which is what refuses it"
    );
}

/// A sentinel inside a fenced block is not a sentinel, because the cut reads prose.
///
/// This is the misread the whole migration exists to close: a fenced `## [Unreleased]` used to open a section
/// in every reader that walked the document's lines.
#[test]
fn a_fenced_sentinel_opens_no_section() {
    let fenced = Source::of(
        "## [0.1.0] - 2020-01-01\n\
         - a real entry\n\
         \n\
         ```\n\
         ## [Unreleased]\n\
         ```\n\
         - still the same section\n",
    );
    let sections = cut(fenced.prose().numbered_lines(), version_section);
    assert_eq!(sections.len(), 1, "{sections:?}");
    assert_eq!(
        sections[0].body.last(),
        Some(&(7, "- still the same section".to_string())),
        "the fenced heading opened nothing, so line 7 is still the first section's — and its position \
         survives the fence rather than being renumbered"
    );
}

/// The section carries the sentinel line as written, not only the name the predicate derived from it.
///
/// The derived name is lossy for this predicate by design — it drops the ` - DATE` suffix — so a reader
/// asking *which date* has to read the line. Given a dated section and an undated one, because a reader that
/// returned the name for both would pass a fixture holding only the undated form.
#[test]
fn a_section_keeps_the_sentinel_line_the_document_wrote() {
    // The dated form is invented rather than copied from `CHANGELOG.md`, as every sibling fixture here
    // already does. A fixture mirroring the live release heading reads as a quotation of it, and a tree-wide
    // edit of that heading — which the release ritual performs — would silently rewrite this input. That
    // very edit was attempted while preparing `0.5.0` and caught by an assertion, not by the fixture.
    let both = Source::of("## [Unreleased]\n- pending\n## [0.1.0] - 2020-01-01\n- shipped\n");
    let sections = cut(both.prose().numbered_lines(), version_section);
    let names: Vec<&str> = sections.iter().map(|s| s.name.as_str()).collect();
    let lines: Vec<&str> = sections.iter().map(|s| s.line.as_str()).collect();
    assert_eq!(names, vec!["## [Unreleased]", "## [0.1.0]"]);
    assert_eq!(
        lines,
        vec!["## [Unreleased]", "## [0.1.0] - 2020-01-01"],
        "the dated section's line keeps its date where its name cannot; the undated one is identical in both, \
         which is why the fixture carries one of each"
    );
}

/// Any table heading closes the block, so a foreign table's keys are never the previous block's.
///
/// **This is the structural half of a rule that used to be a rule.** `require_lock_versions` walked
/// `Cargo.lock` with `name`, `version` and `source` as function-level state and called a `close` closure on
/// *every* table header — because `[[patch.unused]]`, which cargo writes whenever a `[patch]` section exists,
/// carries all three of those keys. Read as ordinary content they overwrote the block above, so the last
/// member's version was replaced before it was filed and the workspace lookup reported that member absent
/// from a lock recording it. The call on the foreign header was what stopped it, and deleting that one call
/// brought the defect back.
///
/// With the predicate answering *is this a heading* before *is it mine*, a foreign table's body belongs to no
/// `[[package]]` at all, so the state cannot bleed whether it is a local or not. The negative run for this
/// direction is a predicate that recognises only `[[package]]` as a boundary — which is exactly the shape the
/// `close` call existed to patch.
#[test]
fn a_foreign_table_heading_still_closes_the_block() {
    let lock = Source::of(
        "[[package]]\n\
         name = \"member\"\n\
         version = \"0.5.0\"\n\
         \n\
         [[patch.unused]]\n\
         name = \"some-patched-crate\"\n\
         version = \"9.9.9\"\n\
         source = \"registry+https://example.invalid/index\"\n",
    );
    let blocks = cut(lock.toml().numbered_lines(), |line| {
        crate::manifest::is_table(line).then(|| line.trim() == "[[package]]")
    });

    let mine: Vec<&Section<bool>> = blocks.iter().filter(|block| block.name).collect();
    assert_eq!(mine.len(), 1, "{blocks:?}");
    let body: Vec<&str> = mine[0].body.iter().map(|(_, l)| l.as_str()).collect();
    assert!(
        body.iter().any(|l| l.contains("\"member\"")),
        "the package's own keys are its own: {body:?}"
    );
    assert!(
        !body
            .iter()
            .any(|l| l.contains("9.9.9") || l.contains("some-patched-crate")),
        "a `[[patch.unused]]` table carries its own name and version, and none of them belong to the block \
         above it: {body:?}"
    );
    // The foreign table is a block of its own rather than dropped, which is what makes it *not* the previous
    // one's — a reader that discarded it entirely would pass this by never having read it.
    assert_eq!(blocks.len(), 2, "{blocks:?}");
}
