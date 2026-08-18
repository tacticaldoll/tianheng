//! The release-coherence readers' failure matrix, for shapes a real manifest cannot legally carry.
//!
//! **Why these are unit tests and the rest of the matrix is not.** The end-to-end matrix builds a fixture
//! repository and runs `judge`, which puts every case through `cargo metadata` — so a case whose whole point
//! is a manifest shape cargo *rejects* cannot be written there. The first draft of the two-halves case tried
//! exactly that: `version.workspace = true, version = "0.2.0"` is a duplicate key, and the direction failed
//! on cargo's parse error rather than on what it meant to observe. A predicate over text is testable as a
//! predicate over text.

use crate::manifest::Quoted;
use crate::refusal::Kind;
use crate::release_coherence_gate::{inline_assignments, is_iso_date, require_internal_pins};

/// The readable values `inline_assignments` found, so a direction asserts what was read rather than a shape.
fn versions_in(value: &str) -> Vec<String> {
    inline_assignments(value, "version")
        .into_iter()
        .map(|found| match found {
            Quoted::Value(version) => version,
            Quoted::Unreadable => "<unreadable>".to_string(),
        })
        .collect()
}

/// Each half of the key recogniser, shown by the shape only that half rejects.
///
/// **The sentence this replaces was refuted by the code it described.** It claimed the delimiter half alone
/// still admits `/version` and the `=`-follows half alone still admits a key ending in `version` — but the
/// delimiter half rejects both of those, so the pair of examples established nothing about either half. One
/// case each, and a run rather than a description.
#[test]
fn each_half_of_the_key_recogniser_rejects_a_shape_of_its_own() {
    // The DELIMITER half. In both of these `version` is glued to the character before it — `/` in a path,
    // `-` in a longer key — so neither opens a key. Without this half both would be read as assignments.
    for glued in [
        r#"{ path = "crates/version-utils", version = "0.2.0" }"#,
        r#"{ rust-version = "1.85", version = "0.2.0" }"#,
    ] {
        assert_eq!(
            versions_in(glued),
            vec!["0.2.0".to_string()],
            "only the assignment is a key: {glued}"
        );
    }

    // The `=`-FOLLOWS half. Here `version` IS preceded by a delimiter — a space inside a string value — so
    // the delimiter half admits it and only this half rejects it. Without it the reader finds two.
    let in_a_value =
        r#"{ path = "crates/xuanji", version = "0.2.0", note = "a version of record" }"#;
    assert_eq!(
        versions_in(in_a_value),
        vec!["0.2.0".to_string()],
        "a delimiter-preceded occurrence that is not an assignment is not a key: {in_a_value}"
    );
}

/// A dated heading's fields are ranged, not merely three digit runs.
///
/// The repair's own standard — *a length test is a parse without its guarantee* — applied one level in: a
/// digit-field test is a parse without a date's guarantee.
#[test]
fn a_dated_suffix_is_a_date_and_not_only_three_digit_runs() {
    for real in ["2026-07-20", "1999-01-01", "2026-12-31"] {
        assert!(is_iso_date(real), "{real} is a date");
    }
    for shaped in ["2026-99-99", "2026-00-10", "0000-00-00", "2026-13-01"] {
        assert!(
            !is_iso_date(shaped),
            "{shaped} has a date's shape and names none"
        );
    }
    for wrong_shape in ["notadate!!", "2026-7-20", "20260720", "2026-07-20-01"] {
        assert!(
            !is_iso_date(wrong_shape),
            "{wrong_shape} is not even the shape"
        );
    }
}

/// Several `path` or several `version` keys in one internal dependency are not this reader's to choose from.
///
/// Here rather than end-to-end for the reason this module's header gives: a duplicate key is a shape cargo
/// rejects outright, so a fixture carrying one fails on the parse rather than on what it means to observe.
///
/// Negative run: with each arm replaced by `continue`, the matching half returned `Ok`.
#[test]
fn several_paths_or_several_versions_in_one_dependency_are_not_chosen_between() {
    let several_paths = "[workspace.dependencies]\n\
                         xuanji = { path = \"crates/xuanji\", path = \"crates/other\", version = \"0.2.0\" }\n";
    let refusal = require_internal_pins(several_paths, "0.2.0")
        .expect_err("two paths name two places and this reader may pick neither");
    assert_eq!(refusal.kind, Kind::CannotJudge, "{}", refusal.message);
    assert!(
        refusal.message.contains("declares 2 `path` keys"),
        "the refusal must say how many paths, got: {}",
        refusal.message
    );

    let several_versions = "[workspace.dependencies]\n\
                            xuanji = { path = \"crates/xuanji\", version = \"0.2.0\", version = \"0.1.0\" }\n";
    let refusal = require_internal_pins(several_versions, "0.2.0")
        .expect_err("two versions are two requirements and this reader may pick neither");
    assert_eq!(refusal.kind, Kind::CannotJudge, "{}", refusal.message);
    assert!(
        refusal.message.contains("declares 2 `version` keys"),
        "the refusal must say how many versions, got: {}",
        refusal.message
    );
}

/// A `path` or a `version` this reader cannot read is a cannot-judge, and the two say which they are.
///
/// Single-quoted TOML strings are legal and this reader does not take them, which is a limit of the reader
/// rather than a fact about the manifest — the distinction `Quoted` exists to keep. The unreadable **path**
/// is the one that matters most: it cannot be answered by skipping the entry, because whether the entry is
/// an internal dependency at all is the thing that could not be read.
///
/// Negative run: with the `Declared::Unreadable` arms replaced by `continue`, the path half reported the
/// vacuity refusal — *found no internal path dependency* — and the version half returned `Ok`.
#[test]
fn a_path_or_a_version_this_reader_cannot_read_is_a_cannot_judge() {
    let path = "[workspace.dependencies]\n\
                xuanji = { path = 'crates/xuanji', version = \"0.2.0\" }\n";
    let refusal = require_internal_pins(path, "0.2.0")
        .expect_err("a path this reader cannot take is not a path it may ignore");
    assert_eq!(refusal.kind, Kind::CannotJudge, "{}", refusal.message);
    assert!(
        refusal
            .message
            .contains("declares a `path` this check cannot read"),
        "got: {}",
        refusal.message
    );

    let version = "[workspace.dependencies]\n\
                   xuanji = { path = \"crates/xuanji\", version = '0.2.0' }\n";
    let refusal = require_internal_pins(version, "0.2.0")
        .expect_err("a version this reader cannot take is not one that satisfies");
    assert_eq!(refusal.kind, Kind::CannotJudge, "{}", refusal.message);
    assert!(
        refusal
            .message
            .contains("declares a version this check cannot read"),
        "got: {}",
        refusal.message
    );
}
