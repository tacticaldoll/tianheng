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
    // The last two are the residue the field-range version admitted and this one does not: a day the
    // calendar does not have, and the leap rule in the direction that catches a century.
    for shaped in [
        "2026-99-99",
        "2026-00-10",
        "0000-00-00",
        "2026-13-01",
        "2026-02-31",
        "2026-04-31",
        "1900-02-29",
    ] {
        assert!(
            !is_iso_date(shaped),
            "{shaped} has a date's shape and names none"
        );
    }
    // And the day the leap rule does admit, so the refusals above are the calendar rather than a narrower
    // month.
    assert!(is_iso_date("2024-02-29"), "2024-02-29 is a date");
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
    crate::refusal::expect(
        "release-coherence#dependency-declares-several-paths",
        &refusal,
    );
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
    crate::refusal::expect("release-coherence#internal-pin-several", &refusal);
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
    crate::refusal::expect("release-coherence#dependency-path-unreadable", &refusal);
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
    crate::refusal::expect("release-coherence#internal-pin-unreadable", &refusal);
    assert_eq!(refusal.kind, Kind::CannotJudge, "{}", refusal.message);
    assert!(
        refusal
            .message
            .contains("declares a version this check cannot read"),
        "got: {}",
        refusal.message
    );
}

/// A dependency key this reader cannot decode is refused, not skipped.
///
/// **The falsifier is the stale pin that got away.** TOML admits a quoted key and cargo decodes it —
/// measured, `"serde_json" = "1"` resolves to a dependency named `serde_json` — so `"xuanji" = "0.0.1"` is a
/// real family requirement whose raw spelling matches no family member. Before this it was dropped by the
/// `!family.contains(…)` filter, and the aggregate requirement counter stayed non-zero on the strength of the
/// second example here, so the judgement reported **clean** over a stale pin. That is the false-negative
/// direction the Core Contract forbids, reached through the same door the `Named` arm's own comment already
/// describes for a rename.
///
/// The second example is load-bearing rather than decoration: with only the quoted one present the counter
/// would reach zero and the existing vacuity guard would refuse for its own reason, which is not this one.
#[test]
fn a_dependency_key_this_reader_cannot_decode_is_refused_rather_than_skipped() {
    let root = std::env::temp_dir().join(format!("kanhe-quoted-key-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&root);
    xingbiao::claim_scratch(&root).expect("the scratch root is writable");

    let write = |dir: &str, body: &str| {
        let at = root.join("examples").join(dir);
        std::fs::create_dir_all(&at).expect("the example directory is writable");
        std::fs::write(at.join("Cargo.toml"), body).expect("the example manifest is writable");
    };
    // The quoted key carries a stale pin; the bare one is correct and keeps the counter non-zero.
    write(
        "quoted",
        "[package]\nname = \"ex-quoted\"\n\n[dependencies]\n\"xuanji\" = \"0.0.1\"\n",
    );
    write(
        "bare",
        "[package]\nname = \"ex-bare\"\n\n[dependencies]\nxuanji = \"0.5.0\"\n",
    );

    let manifests = [(
        "crates/xuanji/Cargo.toml".to_string(),
        "[package]\nname = \"xuanji\"\n".to_string(),
    )];

    let refusal = super::super::release_coherence_gate::require_example_pins(
        &root, &manifests, "0.5.0",
    )
    .expect_err(
        "a key this reader cannot decode names some crate, and which one is what it cannot say — so the \
         entry can neither be matched against the family nor passed over. Passed over, the stale \
         \"0.0.1\" reaches a release as clean",
    );
    crate::refusal::expect(
        "release-coherence#example-dependency-key-unreadable",
        &refusal,
    );
    assert_eq!(refusal.kind, Kind::CannotJudge);
    assert!(
        refusal.message.contains("not a bare TOML key"),
        "the refusal must name what it could not decode, got: {}",
        refusal.message
    );

    let _ = std::fs::remove_dir_all(&root);
}

/// The TOML escape for `x`, built rather than typed.
///
/// **Typed into a literal it silently becomes the decoded value**, and that is measured rather than feared:
/// writing these two directions produced a plain `crates/xuanji` four times in a row, each time reading as
/// though it carried the escape. Naming the sequence puts the substitution where a reader sees it.
const ESCAPED_X: &str = "\\u0078";

/// The TOML escape for `j`, for the reason [`ESCAPED_X`] records.
const ESCAPED_J: &str = "\\u006A";

/// An escaped path is refused, and only one of the two positions was ever a false negative.
///
/// **Cargo decodes the escape and this reader decodes none.** Measured on cargo 1.96.0 against a scratch
/// workspace: a `path` of `crates/` + [`ESCAPED_X`] + `uanji` resolves the member at `crates/xuanji`.
///
/// **The two positions are not two instances of one defect, and an earlier version of this comment said they
/// were.** Measured by removing the backslash branch and running this direction:
///
/// - **inside the prefix** (`cr` + [`ESCAPED_X`] + `tes/xuanji`) — the raw source does not begin `crates/`,
///   so `starts_with` did not select the entry, `continue` took it, and the ordinary sibling kept the
///   vacuity counter non-zero so *found no internal path dependency* never fired. The stale `0.0.1` reached
///   a release as clean. **This is the false-negative regression direction.**
/// - **after the prefix** (`crates/` + [`ESCAPED_X`] + `uanji`) — the raw source still begins `crates/`, so
///   the entry WAS selected and its version WAS compared. Measured: the old reader answered
///   `release-coherence#internal-pin-disagrees`, naming *internal dependency xuanji is pinned to 0.0.1;
///   expected 0.5.0*. Not clean, and not a missed check. **This position is coverage of the uniform
///   fail-closed rule, not evidence of the old silence** — and `require_internal_pins` never resolves a
///   crate identity from a path, so nothing here was ever "compared against a name no crate has".
#[test]
fn an_escaped_path_is_refused_and_an_ordinary_sibling_does_not_cover_for_it() {
    // Ordered as the doc comment reads them: the regression direction first, then the uniform-rule one.
    for path in [
        format!("cr{ESCAPED_X}tes/xuanji"),
        format!("crates/{ESCAPED_X}uanji"),
    ] {
        let manifest = format!(
            "[workspace.dependencies]\n\
             xingbiao = {{ path = \"crates/xingbiao\", version = \"0.5.0\" }}\n\
             xuanji = {{ path = \"{path}\", version = \"0.0.1\" }}\n"
        );
        let refusal = require_internal_pins(&manifest, "0.5.0").expect_err(
            "cargo resolves this path and this reader cannot, so whether the entry is an internal \
             dependency is what could not be read",
        );
        crate::refusal::expect("release-coherence#dependency-path-unreadable", &refusal);
        assert_eq!(refusal.kind, Kind::CannotJudge, "{}", refusal.message);
    }
}

/// An escaped renamed package is refused, and an ordinary family dependency does not cover for it.
///
/// `release-coherence` requires a renamed dependency to be resolved by its `package` identity. Measured on
/// cargo 1.96.0: a `package` of `xuan` + [`ESCAPED_J`] + `i` reads as `xuanji`. Before `quoted_value` refused
/// a backslash, the `family.contains(&package)` filter compared the undecoded source, found no family crate,
/// and took the `continue` — while the ordinary sibling kept `requirements_here` non-zero, so the per-example
/// vacuity guard stayed silent and the stale `0.0.1` reached a release as clean.
#[test]
fn an_escaped_renamed_package_is_refused_and_an_ordinary_sibling_does_not_cover_for_it() {
    let root = std::env::temp_dir().join(format!("kanhe-escaped-rename-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&root);
    xingbiao::claim_scratch(&root).expect("the scratch root is writable");

    let at = root.join("examples").join("escaped");
    std::fs::create_dir_all(&at).expect("the example directory is writable");
    // **Both entries in ONE example, which is what makes the guard blind.** `requirements_here` is counted
    // per example, so an escaped entry alone in its own example leaves that counter at zero and the vacuity
    // guard catches it — measured: with the two split across two examples this direction passed under the
    // perturbation and was a restatement rather than a guard. Beside an ordinary family dependency in the
    // same manifest the counter is non-zero and the escaped entry's silence is invisible.
    std::fs::write(
        at.join("Cargo.toml"),
        format!(
            "[package]\nname = \"ex-escaped\"\n\n[dependencies]\n\
             xingbiao = \"0.5.0\"\n\
             alias = {{ package = \"xuan{ESCAPED_J}i\", version = \"0.0.1\" }}\n"
        ),
    )
    .expect("the example manifest is writable");

    let manifests = [
        (
            "crates/xuanji/Cargo.toml".to_string(),
            "[package]\nname = \"xuanji\"\n".to_string(),
        ),
        (
            "crates/xingbiao/Cargo.toml".to_string(),
            "[package]\nname = \"xingbiao\"\n".to_string(),
        ),
    ];

    let refusal = super::super::release_coherence_gate::require_example_pins(
        &root, &manifests, "0.5.0",
    )
    .expect_err(
        "cargo reads this package as a family crate and this reader cannot, so the entry can \
                 neither be matched against the family nor passed over",
    );
    assert_eq!(refusal.kind, Kind::CannotJudge, "{}", refusal.message);
    // The SITE, not only the kind. Without it, a refusal the vacuity guard produced reads as this
    // direction's evidence — which is exactly how its first version passed under the perturbation.
    crate::refusal::expect(
        "release-coherence#example-package-value-unreadable",
        &refusal,
    );

    let _ = std::fs::remove_dir_all(&root);
}

/// An internal pin taking the workspace offer is refused, because this manifest **is** the workspace.
///
/// A dependency in the root's own catalog spelling `workspace = true` would inherit from the catalog it sits
/// in. Cargo refuses a manifest that inherits what its catalog does not declare, and a catalog inheriting
/// from itself declares nothing — so this is undecidable rather than absent, and it stops in front of an
/// operator instead of being reported as a missing pin.
#[test]
fn an_internal_pin_taking_the_workspace_offer_is_refused() {
    let manifest = "\
[workspace.dependencies]
xuanji = { path = \"crates/xuanji\", workspace = true }
";
    let refusal = require_internal_pins(manifest, "0.5.0").expect_err(
        "a pin inheriting from the catalog it sits in is not a pin this reader can read",
    );
    assert_eq!(refusal.kind, Kind::CannotJudge, "{}", refusal.message);
    crate::refusal::expect("release-coherence#internal-pin-inherited", &refusal);
}

/// A detailed dependency table is filed from its own body, not from wherever the next heading falls.
///
/// **The `Option` that held a half-built record is what this closes.** `declared_dependencies` carried a
/// `pending: Option<Detailed>` flushed at the next heading, because a `[dependencies.NAME]` table's `package`,
/// `version` and `path` arrive across separate lines and the record could only be filed once a following
/// heading proved the table over. Every table now carries its own body, so `Detailed` is built and filed
/// inside one iteration — no half-built record to hold, and no boundary to remember to flush at.
///
/// Read through `require_internal_pins` rather than through the reader itself, so nothing's visibility is
/// widened for a test: the pin verdict is what a wrong boundary changes, and it is already reachable.
///
/// **A foreign table sits between two detailed ones, which is the arrangement that separates the two
/// implementations.** A record surviving a heading folds `[package.metadata.docs.rs]`'s `version` into the
/// table above it, and the pin then disagrees with the workspace version. Two detailed tables in a row would
/// pass either way, since the second heading flushes the first correctly.
#[test]
fn a_detailed_table_is_filed_from_its_own_body() {
    let manifest = "\
[dependencies.xuanji]
path = \"crates/xuanji\"
version = \"0.5.0\"

[package.metadata.docs.rs]
version = \"9.9.9\"

[dependencies.xingbiao]
path = \"crates/xingbiao\"
version = \"0.5.0\"
";
    require_internal_pins(manifest, "0.5.0").expect(
        "both internal dependencies pin the workspace version; the `9.9.9` between them is a docs.rs key and \
         belongs to neither table",
    );
}

/// A dotted internal pin is read, and a stale one in that form is refused.
///
/// **The gate passed a stale pin written this way.** `require_internal_pins` selects on **path**, and the
/// per-line reading gave the `.path` line a path with no version and the `.version` line a version with no
/// path — so neither was internal to it. Measured before the repair: four correct inline siblings plus a stale
/// dotted pair answered `Ok(())`, where the same staleness written inline is a violation. Not a shape nobody
/// writes: `version.workspace = true` is that spelling in every member's `[package]` table.
///
/// **Each case carries an ordinary inline sibling, so the vacuity counter cannot cover for it.**
/// `require_internal_pins` returns early when it finds no internal pins at all, and a fixture holding only the
/// dotted pair would take that arm — passing for having read nothing rather than for reading correctly.
///
/// **Both directions, because the repair has two ways to be wrong.** Reading nothing leaves the miss; reading
/// per line produces a *false refusal* — measured, the naive form reports `xuanji.path is pinned to
/// crates/xuanji; expected 0.5.0`, the path read as the requirement, over a manifest cargo accepts. That is a
/// defect in its own right — though the Core Contract's *one forbidden bug* is the opposite direction, a real
/// violation that silently passes — so the correct case is asserted first and the stale one after it.
#[test]
fn a_dotted_internal_pin_is_read_and_a_stale_one_refused() {
    let sibling = "xingbiao = { path = \"crates/xingbiao\", version = \"0.5.0\" }\n";

    let correct = format!(
        "[workspace.dependencies]\n{sibling}xuanji.path = \"crates/xuanji\"\nxuanji.version = \"0.5.0\"\n"
    );
    require_internal_pins(&correct, "0.5.0").expect(
        "a dotted internal pin at the workspace version is read and passes; refusing it would be a false \
         refusal over a manifest cargo accepts",
    );

    let stale = format!(
        "[workspace.dependencies]\n{sibling}xuanji.path = \"crates/xuanji\"\nxuanji.version = \"0.4.0\"\n"
    );
    let refusal = require_internal_pins(&stale, "0.5.0")
        .expect_err("a stale dotted pin is the defect this direction exists for");
    crate::refusal::expect("release-coherence#internal-pin-disagrees", &refusal);
    assert!(
        refusal.message.contains("xuanji") && refusal.message.contains("0.4.0"),
        "the refusal names the dependency by its head key and the pin it found, not the dotted line: {}",
        refusal.message
    );

    // A tail this reader does not judge is ignored exactly as its inline counterpart is, rather than
    // becoming a record of its own.
    let with_features = format!(
        "[workspace.dependencies]\n{sibling}xuanji.path = \"crates/xuanji\"\nxuanji.version = \"0.5.0\"\n\
         xuanji.features = [\"serde\"]\n"
    );
    require_internal_pins(&with_features, "0.5.0")
        .expect("`features` is not a field this reader judges, dotted or inline");
}
