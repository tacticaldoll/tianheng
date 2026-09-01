//! Repository check: the release commit spine, the version-bearing surfaces, and the changelog.
//!
//! The judgement lives in `support/release_coherence_gate.rs`, shared by this gate and by the fixtures below,
//! because two constructions of "a repository with a changelog" is the twin-drift class this repository keeps
//! closing. It separates a **violation** from a **cannot-judge**, and this matrix asserts which — a matrix
//! reading only "non-zero" was blind to exactly the regression the shell era's shared backstop introduced,
//! where every genuine incoherence was reported as cannot-judge with CI green throughout.

use kanhe::hermetic_git::FIXTURE_DAY;
use kanhe::refusal;

/// A day that is not [`FIXTURE_DAY`], for the direction that needs the two to disagree.
///
/// Spelled rather than computed, because date arithmetic would be a second implementation of the calendar
/// in a file whose subject is a date — and the pair is legible as it stands: the day after.
const A_DIFFERENT_DAY: &str = "2026-07-21";

use kanhe::release_coherence_gate as gate;

use gate::{
    build_fixture, commit, development_changelog, judge, release_changelog, workspace_files,
};
use refusal::Kind;
use std::path::{Path, PathBuf};

fn workspace_root() -> Option<PathBuf> {
    shengmo::workspace::locate(
        PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../.."),
        |root| root.join("CHANGELOG.md").is_file(),
        shengmo::workspace::marker_set(),
    )
}

fn scratch(name: &str) -> PathBuf {
    let root = std::env::temp_dir().join(format!(
        "tianheng-release-coherence-{name}-{}",
        std::process::id()
    ));
    let _ = std::fs::remove_dir_all(&root);
    xingbiao::claim_scratch(&root).expect("the fixture root is writable");
    root
}

/// Every `git` this file runs goes through the fixture builder, dates and all.
///
/// **A second command helper stood beside the shared one and predated it.** This was
/// `hermetic("git")` plus an assert — which is `hermetic_git::fixture` minus the fixed dates — and four
/// commit-creating paths went through it, including the one the date direction adds. So every amended
/// fixture HEAD carried a wall-clock committer date while the constant beside it existed to stop exactly
/// that. It is the fourth instance of the class `hermetic_git`'s own header is about, and the one the date
/// extraction walked past.
fn git(repo: &Path, args: &[&str]) {
    kanhe::hermetic_git::fixture(repo, "git", args);
}

/// Rewrite `[Unreleased]`'s single item, so a direction can place content under a heading of its choosing.
fn unreleased_body(repo: &Path, body: &str) {
    let path = repo.join("CHANGELOG.md");
    let text = std::fs::read_to_string(&path).expect("read the fixture changelog");
    std::fs::write(
        &path,
        text.replace("- An adopter-facing change.\n", &format!("{body}\n")),
    )
    .expect("write the fixture changelog");
}

fn with_machinery(repo: &Path) {
    std::fs::create_dir_all(repo.join("scripts")).expect("create scripts/");
    std::fs::write(
        repo.join("scripts/check_fixture_gate.sh"),
        "#!/usr/bin/env bash\nexit 0\n",
    )
    .expect("write");
    // Machinery where the machinery actually moved to: a gate inside the fixture's **unpublished** member.
    // Without it the corpus derived from the manifests would be exercised only by `scripts/`, which is the
    // enumeration this change replaced.
    std::fs::create_dir_all(repo.join("crates/tianheng/tests"))
        .expect("create the member's tests/");
    std::fs::write(
        repo.join("crates/tianheng/tests/fixture_gate.rs"),
        "#[test]\nfn t() {}\n",
    )
    .expect("write");
}

/// A refusal of `kind` saying `needle`, returned so the direction can cite the site it came from.
///
/// **The site, and not only the message.** A needle is a phrase inside a rendered message: it cannot tell a
/// branch that was never exercised from one whose wording moved, which is what the refusal register replaces
/// with a citation compared by running. The needle stays, because what the operator is told is the whole of
/// what a refusal delivers.
fn refuse(repo: &Path, kind: Kind, needle: &str) -> refusal::Refusal {
    let refusal = judge(repo).expect_err(&format!("expected a refusal containing {needle:?}"));
    assert_eq!(refusal.kind, kind, "{}", refusal.message);
    assert!(
        refusal.message.contains(needle),
        "expected a refusal containing {needle:?}, got: {}",
        refusal.message
    );
    refusal
}

/// The gate, over this repository.
#[test]
fn the_release_surfaces_are_coherent() {
    let Some(root) = workspace_root() else {
        return;
    };
    match judge(&root) {
        Ok(report) => eprintln!("{report}"),
        Err(refusal) => panic!(
            "release coherence ({:?}): {}",
            refusal.kind, refusal.message
        ),
    }
}

// --- the failure matrix -------------------------------------------------------------------------------------

#[test]
fn a_snapshot_is_coherent() {
    let root = scratch("snapshot");
    let fixture = build_fixture(&root, "snapshot", "0.2.0");
    let verdict = judge(&fixture.repo);
    let _ = std::fs::remove_dir_all(&root);
    assert!(verdict.is_ok(), "{:?}", verdict.err());
}

/// A release section dated on a day other than its own release commit is a violation.
///
/// **The value, not only the shape.** `is_iso_date` was hardened twice — parsed rather than counted, then
/// ranged rather than digit-tested — and each step asked a sharper question about the shape while the value
/// went unasked. Three releases carried a section date equal to their `release: X.Y.Z` commit's date because
/// someone remembered; the fourth was prepared with a date four days behind the day it would be cut on, and
/// nothing said so.
///
/// Only at the snapshot: before the release commit exists there is nothing to be dated against, and a date
/// written during preparation is an intent. The control is `a_snapshot_is_coherent`, which is the same
/// fixture with the date left agreeing.
#[test]
fn a_release_section_dated_away_from_its_commit_is_a_violation() {
    let root = scratch("date-disagrees");
    let fixture = build_fixture(&root, "date-disagrees", "0.2.0");
    let path = fixture.repo.join("CHANGELOG.md");
    let text = std::fs::read_to_string(&path).expect("the fixture changelog is readable");
    std::fs::write(
        &path,
        text.replace(
            &format!("## [0.2.0] - {FIXTURE_DAY}"),
            &format!("## [0.2.0] - {A_DIFFERENT_DAY}"),
        ),
    )
    .expect("the fixture changelog is writable");
    // Amended rather than committed on top, so the release commit stays HEAD and the state stays Snapshot.
    git(&fixture.repo, &["add", "."]);
    git(&fixture.repo, &["commit", "-q", "--amend", "--no-edit"]);
    let verdict = judge(&fixture.repo);
    let _ = std::fs::remove_dir_all(&root);
    let refusal = verdict.expect_err("a section dated away from its commit must be refused");
    refusal::expect(
        "release-coherence#release-date-disagrees-with-its-commit",
        &refusal,
    );
    assert!(
        refusal.message.contains(A_DIFFERENT_DAY) && refusal.message.contains(FIXTURE_DAY),
        "the refusal names both dates so an operator can see which to change: {}",
        refusal.message
    );
}

#[test]
fn development_with_release_notes_is_coherent() {
    let root = scratch("development");
    let fixture = build_fixture(&root, "development", "0.2.0");
    development_changelog(&fixture.repo, "0.2.0", true);
    commit(&fixture.repo, "docs: describe pending work");
    let verdict = judge(&fixture.repo);
    let _ = std::fs::remove_dir_all(&root);
    assert!(verdict.is_ok(), "{:?}", verdict.err());
}

#[test]
fn a_release_ready_tree_is_coherent() {
    let root = scratch("ready");
    let fixture = build_fixture(&root, "ready", "0.2.0");
    workspace_files(&fixture.repo, "0.2.1");
    release_changelog(&fixture.repo, "0.2.1", "0.2.0");
    commit(&fixture.repo, "chore: prepare release");
    let verdict = judge(&fixture.repo);
    let _ = std::fs::remove_dir_all(&root);
    assert!(verdict.is_ok(), "{:?}", verdict.err());
}

/// A renamed family dependency is resolved by its `package` field, not by the key it was given.
///
/// **The stale pin sits beside a correct one, deliberately.** The aggregate `requirements` counter is
/// satisfied by any example declaring any readable family pin, so a fixture carrying only the renamed entry
/// would be refused by the vacuity guard rather than by the rule — passing for the wrong reason. Both
/// entries live in one manifest so the guard cannot mask the miss.
///
/// Negative run: with identity taken from the key alone, `alias` matches no family crate, the entry is
/// skipped, and `judge` returns `Ok` over an example requiring `xuanji = "0.0.1"` against workspace `0.2.0`.
#[test]
fn a_renamed_family_dependency_is_resolved_by_its_package_field() {
    let root = scratch("renamed-dep");
    let fixture = build_fixture(&root, "renamed-dep", "0.2.0");
    let manifest = fixture.repo.join("examples/adopter/Cargo.toml");
    let text = std::fs::read_to_string(&manifest).expect("read");
    std::fs::write(
        &manifest,
        format!("{text}alias = {{ package = \"xuanji\", version = \"0.0.1\" }}\n"),
    )
    .expect("write");
    development_changelog(&fixture.repo, "0.2.0", true);
    commit(&fixture.repo, "chore: rename a family dependency");
    refusal::expect(
        "release-coherence#example-pin-disagrees",
        &refuse(&fixture.repo, Kind::Violation, "xuanji (as `alias`)"),
    );
    let _ = std::fs::remove_dir_all(&root);
}

/// The section dated for the workspace version is adopter-facing while the release is still being written.
///
/// Negative run: reading `## [Unreleased]` alone, this returns `Ok` — and in release-ready state that section
/// is required to be **empty**, so the check has no subject at all during preparation.
#[test]
fn a_dated_section_for_the_pending_release_is_adopter_facing() {
    let root = scratch("pending-dated");
    let fixture = build_fixture(&root, "pending-dated", "0.2.0");
    with_machinery(&fixture.repo);
    workspace_files(&fixture.repo, "0.2.1");
    release_changelog(&fixture.repo, "0.2.1", "0.2.0");
    let path = fixture.repo.join("CHANGELOG.md");
    let text = std::fs::read_to_string(&path).expect("read");
    std::fs::write(
        &path,
        text.replace(
            "- Release notes.\n",
            "### Fixed\n- A repair naming `scripts/check_fixture_gate.sh`.\n",
        ),
    )
    .expect("write");
    commit(&fixture.repo, "chore: prepare release");
    refusal::expect(
        "release-coherence#adopter-entry-names-own-machinery",
        &refuse(
            &fixture.repo,
            Kind::Violation,
            "names this repository's own machinery",
        ),
    );
    let _ = std::fs::remove_dir_all(&root);
}

/// The dated heading's suffix is parsed as a date, not counted as ten characters.
///
/// Negative run: under the length test, `notadate!!` is exactly ten characters, so the heading satisfied
/// *CHANGELOG carries dated release notes* and `judge` returned `Ok`.
#[test]
fn a_dated_heading_whose_suffix_is_not_a_date_is_a_violation() {
    let root = scratch("not-a-date");
    let fixture = build_fixture(&root, "not-a-date", "0.2.0");
    workspace_files(&fixture.repo, "0.2.1");
    release_changelog(&fixture.repo, "0.2.1", "0.2.0");
    let path = fixture.repo.join("CHANGELOG.md");
    let text = std::fs::read_to_string(&path).expect("read");
    std::fs::write(
        &path,
        text.replace(
            &format!("## [0.2.1] - {FIXTURE_DAY}"),
            "## [0.2.1] - notadate!!",
        ),
    )
    .expect("write");
    commit(&fixture.repo, "chore: prepare release");
    refusal::expect(
        "release-coherence#dated-release-notes-missing",
        &refuse(
            &fixture.repo,
            Kind::Violation,
            "missing dated release notes for 0.2.1",
        ),
    );
    let _ = std::fs::remove_dir_all(&root);
}

/// A member whose own name carries `version` still has its pin read.
///
/// Negative run: `split("version").nth(1)` cuts at the first occurrence on the whole line — inside the
/// dependency's own name — so the pin read as absent and the gate answered *has no version pin*, a false
/// refusal over a correctly pinned manifest.
#[test]
fn a_member_whose_name_carries_version_still_reads_its_pin() {
    let root = scratch("version-in-name");
    let fixture = build_fixture(&root, "version-in-name", "0.2.0");
    let manifest = fixture.repo.join("Cargo.toml");
    let text = std::fs::read_to_string(&manifest).expect("read");
    std::fs::write(
        &manifest,
        text.replace(
            "xuanji = { path = \"crates/xuanji\", version = \"0.2.0\" }",
            "xuanji = { path = \"crates/xuanji\", version = \"0.2.0\" }\n\
             version-utils = { path = \"crates/version-utils\", version = \"0.2.0\" }",
        ),
    )
    .expect("write");
    development_changelog(&fixture.repo, "0.2.0", true);
    commit(
        &fixture.repo,
        "chore: add a member whose name carries the word",
    );
    let verdict = judge(&fixture.repo);
    let _ = std::fs::remove_dir_all(&root);
    assert!(
        verdict.is_ok(),
        "a pin on a line whose dependency name carries `version` must still be read: {:?}",
        verdict.err()
    );
}

/// Cargo's **detailed table** form is a dependency declaration, renamed or not.
///
/// `[dependencies.alias]` with its own `package` and `version` lines names no family crate on any single
/// line, so a reader keyed on `<crate> = …` entries saw nothing at all. The stale entry sits beside a correct
/// inline pin so the aggregate `requirements` counter cannot refuse this fixture for the wrong reason.
///
/// Negative run: without the heading-tracking reader both rows return `Ok`, over an example requiring
/// `xuanji = "0.0.1"` against workspace `0.2.0`.
#[test]
fn a_detailed_dependency_table_is_read_renamed_or_not() {
    for (label, table, expected) in [
        (
            "renamed",
            "[dependencies.alias]\npackage = \"xuanji\"\nversion = \"0.0.1\"\n",
            "xuanji (as `alias`)",
        ),
        (
            "plainly named",
            "[dependencies.xuanji]\nversion = \"0.0.1\"\n",
            "xuanji",
        ),
    ] {
        let root = scratch(&format!("detailed-{}", label.replace(' ', "-")));
        let fixture = build_fixture(&root, "detailed", "0.2.0");
        let manifest = fixture.repo.join("examples/adopter/Cargo.toml");
        let text = std::fs::read_to_string(&manifest).expect("read");
        // **The example's own inline entry goes, so the table under test is the only declaration of it.**
        // Left in place, the plainly-named row declares `xuanji` twice and cargo refuses the manifest — the
        // fixture stopped being about the detailed table at all. The hand-rolled reader read lines and never
        // met the collision.
        let without_inline = text.replace("xuanji = \"0.2\"", "");
        assert_ne!(
            without_inline, text,
            "{label}: the example's inline entry must be the one this table replaces"
        );
        std::fs::write(&manifest, format!("{without_inline}\n{table}")).expect("write");
        development_changelog(&fixture.repo, "0.2.0", true);
        commit(&fixture.repo, "chore: a detailed dependency table");
        let verdict = judge(&fixture.repo);
        let _ = std::fs::remove_dir_all(&root);
        let refusal =
            verdict.expect_err(&format!("{label}: a stale detailed table must be refused"));
        assert_eq!(
            refusal.kind,
            Kind::Violation,
            "{label}: {}",
            refusal.message
        );
        assert!(
            refusal.message.contains(expected),
            "{label}: the refusal must name the crate it is about: {}",
            refusal.message
        );
    }
}

/// A dependency table whose heading spells its name in escapes is the table cargo reads.
///
/// **Cargo decodes, so this reads.** Put to `cargo metadata` rather than reasoned about: a manifest whose only
/// declaration of `serde` sits under `[target.x86_64-unknown-linux-gnu."\u0064ependencies"]` reports that
/// dependency with that target, and `["dep\u0065ndencies"]` reports it too -- an escape is not a prefix trick
/// and can sit anywhere in the name. A reader answering *undecidable* for a backslash left every pin in such a
/// table unread, and the ordinary pin beside it kept the aggregate non-vacuity guard satisfied, so the run
/// came back clean.
///
/// The stale pin here is therefore the whole point: it is reachable only through the escaped heading, while the
/// fixture's own correct pin holds the counter above zero.
///
/// The `an escaped-quote cfg target` row is the claim `bounds.rs` was already making with nothing holding it
/// to account -- that the
/// cfg shapes this reader meets, *a bare predicate, one carrying spaces, one carrying escaped quotes*, are all
/// classified. A review found the escaped-quote one unpinned, and reaching it through the whole gate is what
/// pins it: cargo reads `serde` under that target, measured, so a pin there is a pin.
///
/// Negative run, per row rather than for the test: with `unquoted` reporting undecodability instead of
/// decoding, `an escaped name` returns `Ok` -- a clean release over a manifest requiring `xuanji = "0.0.1"`
/// against workspace `0.2.0`. `an escaped-quote cfg target` passes either way, measured by running it first
/// under the same perturbation: an undecodable segment left an empty one in the joined name, and the split past the
/// target context happened to land after it. So that row guards a claim that was true for a reason nobody
/// chose, which is what *unpinned* meant here -- and it now holds the reason the prose gives instead.
#[test]
fn an_escaped_dependency_table_heading_is_read_as_the_table_cargo_reads() {
    for (label, table) in [
        (
            "an escaped name",
            "[target.x86_64-unknown-linux-gnu.\"\\u0064ependencies\"]\nxuanji = \"0.0.1\"\n",
        ),
        (
            "an escaped-quote cfg target",
            "[target.\"cfg(feature = \\\"x\\\")\".dependencies]\nxuanji = \"0.0.1\"\n",
        ),
    ] {
        let root = scratch(&format!("escaped-heading-{}", label.replace(' ', "-")));
        let fixture = build_fixture(&root, "escaped-heading", "0.2.0");
        let manifest = fixture.repo.join("examples/adopter/Cargo.toml");
        let text = std::fs::read_to_string(&manifest).expect("read");
        std::fs::write(&manifest, format!("{text}\n{table}")).expect("write");
        development_changelog(&fixture.repo, "0.2.0", true);
        commit(&fixture.repo, "chore: an escaped dependency table heading");
        let verdict = judge(&fixture.repo);
        let _ = std::fs::remove_dir_all(&root);
        let refusal = verdict.expect_err(&format!("{label}: a stale pin under it must be refused"));
        assert_eq!(
            refusal.kind,
            Kind::Violation,
            "{label}: {}",
            refusal.message
        );
        assert!(
            refusal.message.contains("xuanji"),
            "{label}: the refusal must name the crate it is about: {}",
            refusal.message
        );
    }
}

/// A key spelled after a family crate outside a dependency table is not a version requirement.
///
/// The other direction of the same cause: the reader looked at no heading, so `[features]` — whose values are
/// arrays, not versions — was read as a source of pins. Left open, a feature named after a family crate would
/// be refused for a version it never declared.
///
/// Negative run: without the heading-tracking reader this is a cannot-judge, *requires xuanji with a version
/// this check cannot read (\[\])*.
#[test]
fn a_feature_named_after_a_family_crate_is_not_a_pin() {
    let root = scratch("feature-named");
    let fixture = build_fixture(&root, "feature-named", "0.2.0");
    let manifest = fixture.repo.join("examples/adopter/Cargo.toml");
    let text = std::fs::read_to_string(&manifest).expect("read");
    std::fs::write(&manifest, format!("{text}\n[features]\nxuanji = []\n")).expect("write");
    development_changelog(&fixture.repo, "0.2.0", true);
    commit(&fixture.repo, "chore: a feature named after a family crate");
    let verdict = judge(&fixture.repo);
    let _ = std::fs::remove_dir_all(&root);
    assert!(
        verdict.is_ok(),
        "a `[features]` key is not a dependency and must not be read as one: {:?}",
        verdict.err()
    );
}

/// A table whose dot is spelled as an escape is one key, and one key is not a dependency table.
///
/// The other direction of the same defect: `["dependencies\u002Exuanji"]` decodes to the text
/// `dependencies.xuanji`, and a reader that joined its segments back with dots read it as the detailed table
/// `[dependencies.xuanji]` and applied the pin rule to it. Measured, cargo reads it as one unknown top-level
/// key and no dependency at all -- put to it as this fixture's own heading, not as a shape resembling it --
/// so refusing here would stop a release over a manifest cargo builds. That is a defect; the Core Contract's
/// *one forbidden bug* is the other direction, a real violation that silently passes.
///
/// Negative run: replacing the segment match with the joined-name classifier it replaced makes this a
/// violation -- *example adopter requires xuanji = "0.0.1"; this check admits only "0.2.0" or "0.2"*.
/// Perturbing the **cut** instead leaves it green, which is worth saying: the defect lived in the join,
/// and a negative run aimed at the wrong half of the repair reports a guard that is not one.
#[test]
fn a_table_whose_dot_is_escaped_is_one_key_and_not_a_dependency_table() {
    let root = scratch("escaped-separator");
    let fixture = build_fixture(&root, "escaped-separator", "0.2.0");
    let manifest = fixture.repo.join("examples/adopter/Cargo.toml");
    let text = std::fs::read_to_string(&manifest).expect("read");
    std::fs::write(
        &manifest,
        format!(
            "{text}\n{}",
            "[\"dependencies\\u002Exuanji\"]\nversion = \"0.0.1\"\n"
        ),
    )
    .expect("write");
    development_changelog(&fixture.repo, "0.2.0", true);
    commit(&fixture.repo, "chore: a table whose dot is escaped");
    let verdict = judge(&fixture.repo);
    let _ = std::fs::remove_dir_all(&root);
    assert!(
        verdict.is_ok(),
        "one literal key is no dependency table to cargo, and must not be read as one: {:?}",
        verdict.err()
    );
}

/// A dated heading's fields are ranged, not merely digits.
///
/// Negative run: reading only three all-digit fields of widths 4/2/2, `2026-99-99` satisfied *CHANGELOG
/// carries dated release notes* — the same shortfall as the length test that preceded it, one level in.
#[test]
fn a_dated_heading_whose_fields_are_out_of_range_is_a_violation() {
    for impossible in ["2026-99-99", "2026-00-10", "0000-00-00"] {
        let root = scratch(&format!("range-{impossible}"));
        let fixture = build_fixture(&root, "range", "0.2.0");
        workspace_files(&fixture.repo, "0.2.1");
        release_changelog(&fixture.repo, "0.2.1", "0.2.0");
        let path = fixture.repo.join("CHANGELOG.md");
        let text = std::fs::read_to_string(&path).expect("read");
        std::fs::write(
            &path,
            text.replace(
                &format!("## [0.2.1] - {FIXTURE_DAY}"),
                &format!("## [0.2.1] - {impossible}"),
            ),
        )
        .expect("write");
        commit(&fixture.repo, "chore: prepare release");
        refusal::expect(
            "release-coherence#dated-release-notes-missing",
            &refuse(
                &fixture.repo,
                Kind::Violation,
                "missing dated release notes for 0.2.1",
            ),
        );
        let _ = std::fs::remove_dir_all(&root);
    }
}

#[test]
fn a_shallow_history_cannot_be_judged() {
    let root = scratch("shallow");
    let repo = root.join("shallow");
    std::fs::create_dir_all(&repo).expect("create");
    git(&repo, &["init", "-q", "-b", "main"]);
    git(&repo, &["config", "user.name", "T"]);
    git(&repo, &["config", "user.email", "t@example.invalid"]);
    git(&repo, &["config", "commit.gpgsign", "false"]);
    workspace_files(&repo, "0.2.0");
    development_changelog(&repo, "0.2.0", true);
    commit(&repo, "chore: initial import");
    refusal::expect(
        "release-coherence#release-history-shallow",
        &refuse(&repo, Kind::CannotJudge, "release history is unavailable"),
    );
    let _ = std::fs::remove_dir_all(&root);
}

#[test]
fn a_malformed_release_subject_is_a_violation() {
    let root = scratch("malformed");
    let fixture = build_fixture(&root, "malformed", "0.2.0");
    development_changelog(&fixture.repo, "0.2.0", true);
    commit(&fixture.repo, "release: next");
    refusal::expect(
        "release-coherence#release-history-version-malformed",
        &refuse(
            &fixture.repo,
            Kind::Violation,
            "malformed release history subject",
        ),
    );
    let _ = std::fs::remove_dir_all(&root);
}

#[test]
fn a_regressed_workspace_version_is_a_violation() {
    let root = scratch("regression");
    let fixture = build_fixture(&root, "regression", "0.2.0");
    workspace_files(&fixture.repo, "0.1.9");
    development_changelog(&fixture.repo, "0.1.9", true);
    commit(&fixture.repo, "chore: regress version");
    refusal::expect(
        "release-coherence#workspace-version-behind-latest-release",
        &refuse(
            &fixture.repo,
            Kind::Violation,
            "is older than latest release 0.2.0",
        ),
    );
    let _ = std::fs::remove_dir_all(&root);
}

#[test]
fn development_with_no_release_narrative_is_a_violation() {
    let root = scratch("empty-development");
    let fixture = build_fixture(&root, "empty-development", "0.2.0");
    development_changelog(&fixture.repo, "0.2.0", false);
    commit(&fixture.repo, "chore: omit release note");
    refusal::expect(
        "release-coherence#unreleased-has-no-adopter-narrative",
        &refuse(
            &fixture.repo,
            Kind::Violation,
            "requires adopter-facing release narrative",
        ),
    );
    let _ = std::fs::remove_dir_all(&root);
}

#[test]
fn a_manifest_that_does_not_inherit_the_workspace_version_is_a_violation() {
    let root = scratch("no-inherit");
    let fixture = build_fixture(&root, "no-inherit", "0.2.0");
    std::fs::write(
        fixture.repo.join("crates/xuanji/Cargo.toml"),
        "[package]\nname = \"xuanji\"\nversion = \"0.2.0\"\nedition = \"2024\"\n",
    )
    .expect("write");
    development_changelog(&fixture.repo, "0.2.0", true);
    commit(&fixture.repo, "chore: pin a member directly");
    refusal::expect(
        "release-coherence#member-does-not-inherit-workspace-version",
        &refuse(
            &fixture.repo,
            Kind::Violation,
            "must inherit version.workspace = true",
        ),
    );
    let _ = std::fs::remove_dir_all(&root);
}

#[test]
fn an_internal_pin_that_disagrees_is_a_violation() {
    let root = scratch("stale-pin");
    let fixture = build_fixture(&root, "stale-pin", "0.2.0");
    let manifest = fixture.repo.join("Cargo.toml");
    let text = std::fs::read_to_string(&manifest).expect("read");
    std::fs::write(
        &manifest,
        text.replace("version = \"0.2.0\" }", "version = \"0.1.0\" }"),
    )
    .expect("write");
    development_changelog(&fixture.repo, "0.2.0", true);
    commit(&fixture.repo, "chore: stale the internal pin");
    refusal::expect(
        "release-coherence#internal-pin-disagrees",
        &refuse(&fixture.repo, Kind::Violation, "is pinned to 0.1.0"),
    );
    let _ = std::fs::remove_dir_all(&root);
}

#[test]
fn an_example_pin_the_workspace_version_does_not_satisfy_is_a_violation() {
    let root = scratch("example-pin");
    let fixture = build_fixture(&root, "example-pin", "0.2.0");
    std::fs::write(
        fixture.repo.join("examples/adopter/Cargo.toml"),
        "[package]\nname = \"adopter\"\nversion = \"0.0.0\"\nedition = \"2024\"\n\n[dependencies]\nxuanji = \"0.9\"\n",
    )
    .expect("write");
    development_changelog(&fixture.repo, "0.2.0", true);
    commit(&fixture.repo, "chore: stale an example pin");
    refusal::expect(
        "release-coherence#example-pin-disagrees",
        &refuse(&fixture.repo, Kind::Violation, "requires xuanji = \"0.9\""),
    );
    let _ = std::fs::remove_dir_all(&root);
}

/// A lock file this parser cannot read is a cannot-judge, not a missing package.
///
/// **A diagnosis this reader could not give before.** The hand-rolled walker found no `[[package]]` blocks in
/// an unparseable lock and reported *Cargo.lock is missing workspace package xuanji* — a violation naming a
/// package that may be sitting right there, in a file cargo cannot read either. Reading the document says
/// which fact was met.
///
/// Negative run: with the parse failure folded into an empty entry set, this reports the missing-package
/// violation instead.
#[test]
fn a_lock_file_this_parser_cannot_read_cannot_be_judged() {
    let root = scratch("lock-unparseable");
    let fixture = build_fixture(&root, "lock-unparseable", "0.2.0");
    workspace_files(&fixture.repo, "0.2.1");
    release_changelog(&fixture.repo, "0.2.1", "0.2.0");
    std::fs::write(
        fixture.repo.join("Cargo.lock"),
        "version = 4\n\n[[package]\nname = \"xuanji\"\n",
    )
    .expect("write");
    commit(&fixture.repo, "chore: leave the lock unparseable");
    let refusal = refuse(
        &fixture.repo,
        Kind::CannotJudge,
        "not a lock file this parser can read",
    );
    refusal::expect("release-coherence#lock-unreadable", &refusal);
    let _ = std::fs::remove_dir_all(&root);
}

/// The lockfile direction must reach EVERY workspace package, not only the first.
#[test]
fn a_stale_lock_entry_for_the_second_package_is_a_violation() {
    let root = scratch("stale-lock");
    let fixture = build_fixture(&root, "stale-lock", "0.2.0");
    workspace_files(&fixture.repo, "0.2.1");
    release_changelog(&fixture.repo, "0.2.1", "0.2.0");
    let lock = fixture.repo.join("Cargo.lock");
    let text = std::fs::read_to_string(&lock).expect("read");
    std::fs::write(
        &lock,
        text.replace(
            "name = \"xuanji\"\nversion = \"0.2.1\"",
            "name = \"xuanji\"\nversion = \"0.2.0\"",
        ),
    )
    .expect("write");
    commit(&fixture.repo, "chore: leave the second package stale");
    refusal::expect(
        "release-coherence#lock-package-version-disagrees",
        &refuse(
            &fixture.repo,
            Kind::Violation,
            "Cargo.lock package xuanji is 0.2.0",
        ),
    );
    let _ = std::fs::remove_dir_all(&root);
}

#[test]
fn a_release_section_repeating_a_heading_is_a_violation() {
    let root = scratch("duplicate-heading");
    let fixture = build_fixture(&root, "duplicate-heading", "0.2.0");
    development_changelog(&fixture.repo, "0.2.0", true);
    unreleased_body(
        &fixture.repo,
        "### Changed\n- An adopter-facing change.\n\n### Changed\n- A second block of the same name.",
    );
    commit(&fixture.repo, "chore: split one section in two");
    refusal::expect(
        "release-coherence#changelog-section-repeats-a-heading",
        &refuse(&fixture.repo, Kind::Violation, "repeats a heading"),
    );
    let _ = std::fs::remove_dir_all(&root);
}

#[test]
fn a_break_with_nowhere_to_read_what_to_do_is_a_violation() {
    let root = scratch("breaking");
    let fixture = build_fixture(&root, "breaking", "0.2.0");
    development_changelog(&fixture.repo, "0.2.0", true);
    unreleased_body(
        &fixture.repo,
        "### Changed\n- **BREAKING** a change with nowhere to read what to do.",
    );
    commit(&fixture.repo, "chore: mark a break with no migration");
    refusal::expect(
        "release-coherence#breaking-without-migration-section",
        &refuse(&fixture.repo, Kind::Violation, "carries no `### Migration`"),
    );
    let _ = std::fs::remove_dir_all(&root);
}

/// The control for the direction above: the same break WITH the section is coherent.
#[test]
fn a_break_with_its_migration_is_coherent() {
    let root = scratch("breaking-ok");
    let fixture = build_fixture(&root, "breaking-ok", "0.2.0");
    development_changelog(&fixture.repo, "0.2.0", true);
    unreleased_body(
        &fixture.repo,
        "### Changed\n- **BREAKING** a change.\n\n### Migration\n- Regenerate the baseline.",
    );
    commit(&fixture.repo, "chore: mark a break and say what to do");
    let verdict = judge(&fixture.repo);
    let _ = std::fs::remove_dir_all(&root);
    assert!(verdict.is_ok(), "{:?}", verdict.err());
}

/// `release-coherence/prose-about-the-marker-is-read-as-a-marker-a-stated-bound`
///
/// `OverReacts`, owned by the engine. The classifier asks whether a release section *contains* `**BREAKING**`,
/// so a section that merely **discusses** the marker is required to carry a `### Migration` it does not owe.
/// Shown rather than described: the body below announces nothing and marks nothing, and the refusal arrives
/// anyway.
///
/// Recognising the marker at an entry's start instead was considered and declined. Over-reaction is the safe
/// direction — the Core Contract forbids exactly one bug, and it is the false negative — while a positional
/// matcher buys a false-negative risk in the floor: a real break whose marker sits anywhere but the entry's
/// first token would stop being observed at all. A false refusal an author argues with beats a break nobody is
/// told about, so the reaction keeps its reach and the cost is declared here.
#[test]
fn prose_about_the_marker_is_read_as_a_marker_a_stated_bound() {
    let root = scratch("breaking-prose");
    let fixture = build_fixture(&root, "breaking-prose", "0.2.0");
    development_changelog(&fixture.repo, "0.2.0", true);
    unreleased_body(
        &fixture.repo,
        "### Changed\n- A diagnostic whose exit code does not move, so it earns no **BREAKING** mark.",
    );
    commit(&fixture.repo, "chore: discuss the marker without using it");
    refusal::expect(
        "release-coherence#breaking-without-migration-section",
        &refuse(&fixture.repo, Kind::Violation, "carries no `### Migration`"),
    );
    let _ = std::fs::remove_dir_all(&root);
}

// --- adopter narrative names no self-governance machinery ---------------------------------------------------

#[test]
fn an_adopter_heading_naming_a_gate_is_a_violation() {
    let root = scratch("adopter-names-path");
    let fixture = build_fixture(&root, "adopter-names-path", "0.2.0");
    development_changelog(&fixture.repo, "0.2.0", true);
    with_machinery(&fixture.repo);
    unreleased_body(
        &fixture.repo,
        "### Fixed\n- A repair naming `scripts/check_fixture_gate.sh`.",
    );
    commit(&fixture.repo, "docs: name a gate under an adopter heading");
    refusal::expect(
        "release-coherence#adopter-entry-names-own-machinery",
        &refuse(
            &fixture.repo,
            Kind::Violation,
            "names this repository's own machinery",
        ),
    );
    let _ = std::fs::remove_dir_all(&root);
}

/// The same entry under the self-governance heading, so the refusal above is about the heading it sat under.
#[test]
fn the_same_entry_under_the_self_governance_heading_is_coherent() {
    let root = scratch("self-governance");
    let fixture = build_fixture(&root, "self-governance", "0.2.0");
    development_changelog(&fixture.repo, "0.2.0", true);
    with_machinery(&fixture.repo);
    unreleased_body(
        &fixture.repo,
        "### Self-governance\n- A repair naming `scripts/check_fixture_gate.sh`.",
    );
    commit(&fixture.repo, "docs: name a gate where it belongs");
    let verdict = judge(&fixture.repo);
    let _ = std::fs::remove_dir_all(&root);
    assert!(verdict.is_ok(), "{:?}", verdict.err());
}

/// Every form the word rule reaches. Three of them — a span carrying a command, a padded double-backtick
/// span, and a span wrapped across a source line — passed clean under an earlier rule that compared whole
/// backticked spans, and every one is a shape this repository's own changelog already uses.
#[test]
fn a_gate_named_in_any_word_form_is_a_violation() {
    for (name, body) in [
        (
            "bare-basename",
            "### Fixed\n- A repair naming `check_fixture_gate.sh`.",
        ),
        (
            "inside-a-command",
            "### Fixed\n- Run `bash scripts/check_fixture_gate.sh --fix` to repair.",
        ),
        (
            "nested-span",
            "### Fixed\n- A repair naming `` `scripts/check_fixture_gate.sh` `` in a nested span.",
        ),
        (
            "wrapped-span",
            "### Fixed\n- A repair naming `scripts/check_fixture_gate.sh\n  ` across a wrap.",
        ),
        (
            "link-target",
            "### Fixed\n- A repair naming [the gate](scripts/check_fixture_gate.sh).",
        ),
        (
            "unquoted-prose",
            "### Fixed\n- A repair to the check_fixture_gate.sh gate, written as prose.",
        ),
        (
            "sentence-end",
            "### Fixed\n- A repair to scripts/check_fixture_gate.sh.",
        ),
        (
            "the-directory",
            "### Fixed\n- A repair described by naming `scripts/` and nothing in it.",
        ),
        // A gate that lives in an **unpublished crate** rather than under `scripts/`. This is the row the
        // corpus could not see while it enumerated one directory: the window that deleted fourteen shell
        // gates moved the machinery here, and the specification's own scenario named a path like this one
        // while the enumeration still resolved only the old address.
        (
            "a-crates-resident-gate",
            "### Fixed\n- A repair naming `crates/tianheng/tests/fixture_gate.rs`.",
        ),
        (
            "a-crates-resident-basename",
            "### Fixed\n- A repair naming `fixture_gate.rs`.",
        ),
        // **A member whose directory is not its package name.** The row the fixture could not carry while
        // both sides agreed by construction: every member sat at `crates/<name>/`, so a corpus deriving the
        // directory from the package name passed every row above while being wrong about any workspace that
        // does not. `crates/renamed-dir/` holds `machinery-under-another-name`, which publishes nothing.
        (
            "a-member-whose-directory-is-not-its-name",
            "### Fixed\n- A repair naming `crates/renamed-dir/tests/renamed_gate.rs`.",
        ),
    ] {
        let root = scratch(name);
        let fixture = build_fixture(&root, name, "0.2.0");
        development_changelog(&fixture.repo, "0.2.0", true);
        with_machinery(&fixture.repo);
        unreleased_body(&fixture.repo, body);
        commit(&fixture.repo, "docs: name a gate");
        refusal::expect(
            "release-coherence#adopter-entry-names-own-machinery",
            &refuse(
                &fixture.repo,
                Kind::Violation,
                "names this repository's own machinery",
            ),
        );
        let _ = std::fs::remove_dir_all(&root);
    }
}

/// The subject every other fixture could not carry: a repository reached by a path that is not canonical.
///
/// The live call site above passes `PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..")`, which renders
/// with its `..` components intact, while `cargo metadata` reports canonical paths. Deriving the member prefix
/// from the caller's spelling made **all eight** members fail to resolve: machinery collapsed to the two
/// `scripts/` files, `published` stayed empty, and two `continue`s meant nothing said so. Measured on this
/// repository with one changelog entry naming a member's file — the pre-fix gate reported `48 passed`, the
/// fixed one refuses.
///
/// Every fixture root is a clean absolute, which is exactly why the whole matrix stayed green over a corpus
/// that contained no member at all. This row spells the same repository the way the live caller spells it.
#[test]
fn a_repository_reached_through_a_non_canonical_path_still_resolves_its_members() {
    let root = scratch("non-canonical-root");
    let fixture = build_fixture(&root, "non-canonical-root", "0.2.0");
    development_changelog(&fixture.repo, "0.2.0", true);
    with_machinery(&fixture.repo);
    unreleased_body(
        &fixture.repo,
        "### Fixed\n- A repair naming `crates/renamed-dir/tests/renamed_gate.rs`.",
    );
    commit(&fixture.repo, "docs: name a member's gate");

    let indirect = fixture.repo.join("crates").join("..");
    let verdict = judge(&indirect);
    let _ = std::fs::remove_dir_all(&root);
    let refusal = verdict.expect_err(
        "a member's file named in adopter-facing prose is machinery however the root is spelled",
    );
    assert!(
        refusal.message.contains("renamed_gate.rs"),
        "the refusal must name the member's file; collapsing to `scripts/` is the defect this row exists for: \
         {}",
        refusal.message
    );
}

/// A `[lib]` name written **before** `[package]` does not become the package's name.
///
/// The read took the first line whose trimmed start was `name` anywhere in the manifest, which is right only
/// while `[package]` precedes every other name-bearing table — a premise TOML does not impose and nothing
/// stated. The multiplicity is not hypothetical: `crates/tianheng/Cargo.toml` carries three `name` keys
/// (`[package]`, `[lib]`, `[[bin]]`), and the old read was correct there by their order and by the three
/// values agreeing.
///
/// The second name here is the one the old reader would have taken, and it is a name no lock entry has — so
/// under the old read this fixture reports a missing lock entry for `wrong_name` while `xuanji`'s real entry
/// goes unexamined.
#[test]
fn a_lib_name_before_the_package_table_is_not_the_package_name() {
    let root = scratch("lib-name-first");
    let fixture = build_fixture(&root, "lib-name-first", "0.2.0");
    std::fs::write(
        fixture.repo.join("crates/xuanji/Cargo.toml"),
        "[lib]
name = \"wrong_name\"\n\n[package]\nname = \"xuanji\"\nversion.workspace = true\n\
         edition = \"2024\"\n",
    )
    .expect("write");
    development_changelog(&fixture.repo, "0.2.0", true);
    commit(&fixture.repo, "chore: order the tables the other way");
    let verdict = judge(&fixture.repo);
    let _ = std::fs::remove_dir_all(&root);
    assert!(
        verdict.is_ok(),
        "the `[package]` table names this crate whatever order the tables are written in; taking `[lib]`'s \
         name leaves the real package unexamined. Got: {:?}",
        verdict.err()
    );
}

/// A `[package]` name this reader cannot read is a cannot-judge, not a package that is silently absent.
///
/// Single-quoted values are valid TOML and `first_string_value` reads only double quotes. Both readers that
/// consumed the old `Option` treated `None` as *not a package*: one skipped the lock-version comparison for
/// it, the other dropped it from the family every example pin is checked against. Neither said anything.
#[test]
fn a_package_name_this_reader_cannot_read_is_a_cannot_judge() {
    let root = scratch("unreadable-name");
    let fixture = build_fixture(&root, "unreadable-name", "0.2.0");
    std::fs::write(
        fixture.repo.join("crates/xuanji/Cargo.toml"),
        // **This WHEN moved when a real parser replaced the hand-rolled reader.** It was `name = 'xuanji'` — a
        // single-quoted string, legal TOML the old reader declined — and the parser takes it. What still
        // reaches the site is a `name` that is not a string at all.
        "[package]\nname = { workspace = true }\nversion.workspace = true\nedition = \"2024\"\n",
    )
    .expect("write");
    commit(&fixture.repo, "chore: quote the name the other way");
    refuse(&fixture.repo, Kind::CannotJudge, "cannot read");
    let _ = std::fs::remove_dir_all(&root);
}

/// A `Cargo.lock` name this reader cannot read is a cannot-judge, not a package that is not there.
///
/// Single-quoted values are valid TOML. The read defaulted an unreadable name to the empty string, which the
/// `!name.is_empty()` guard then took as *no package here* — so that entry's version never reached the map,
/// and the workspace lookup either reported it missing or matched a stale one recorded under the previous
/// name.
///
/// **Release-ready, deliberately.** During development this repository tolerates lockfile drift by design, so
/// the reader is never reached and the real tree cannot exercise it — measured: perturbing the live
/// `Cargo.lock` leaves the gate green for that reason alone, which is a subject outside the reader's corpus
/// rather than evidence about it.
#[test]
fn a_lock_name_this_reader_cannot_read_is_a_cannot_judge() {
    let root = scratch("lock-name-unreadable");
    let fixture = build_fixture(&root, "lock-name-unreadable", "0.2.0");
    workspace_files(&fixture.repo, "0.2.1");
    release_changelog(&fixture.repo, "0.2.1", "0.2.0");
    let lock = fixture.repo.join("Cargo.lock");
    let text = std::fs::read_to_string(&lock).expect("read the fixture lock");
    // **This WHEN moved when a real parser replaced the hand-rolled walker.** It was `name = 'xuanji'` — a
    // single-quoted string, legal TOML the old reader declined — and the parser takes it. What still reaches
    // the site is a `name` that is not a string at all.
    std::fs::write(
        &lock,
        text.replace("name = \"xuanji\"", "name = [\"xuanji\"]"),
    )
    .expect("write");
    commit(&fixture.repo, "chore: quote a lock name the other way");
    refusal::expect(
        "release-coherence#lock-package-name-unreadable",
        &refuse(&fixture.repo, Kind::CannotJudge, "cannot read"),
    );
    let _ = std::fs::remove_dir_all(&root);
}

/// An example pin this reader cannot read is a cannot-judge, not a requirement that is absent.
///
/// The key is already known to name a family crate when the pin is read, so failing to read its version is a
/// limit of this reader and not a fact about the example. Skipping it left that example unexamined while the
/// aggregate `requirements` counter stayed non-zero on the strength of the other examples — the partial case
/// an aggregate guard is exactly unable to see.
#[test]
fn an_example_pin_this_reader_cannot_read_is_a_cannot_judge() {
    let root = scratch("pin-unreadable");
    let fixture = build_fixture(&root, "pin-unreadable", "0.2.0");
    let example = "adopter";
    let manifest = fixture.repo.join(format!("examples/{example}/Cargo.toml"));
    let text = std::fs::read_to_string(&manifest).expect("read the fixture example manifest");
    // Only the pin's quoting moves. Re-quoting the whole manifest would take the package name with it and
    // trip a different reader first, so the run would be red for a reason other than the one under test.
    let single = text
        .replace("xuanji = \"", "xuanji = '")
        .replace("\"\n", "'\n");
    std::fs::write(&manifest, single).expect("write");
    development_changelog(&fixture.repo, "0.2.0", true);
    commit(&fixture.repo, "chore: quote an example pin the other way");
    refuse(&fixture.repo, Kind::CannotJudge, "cannot read");
    let _ = std::fs::remove_dir_all(&root);
}

/// A registry entry sharing a workspace member's name is not that member's lock entry.
///
/// The map was single-valued and keyed on the name alone, so the first entry won and everything after it was
/// dropped. Two entries under one name is ordinary in a lock — two versions of one crate, or a member sharing
/// a name with something fetched — and `source` is what tells them apart: a workspace member has none.
///
/// The registry entry is written **first** and carries a version the workspace does not have, so under the
/// old read it wins and the gate reports a version disagreement that is not one. The member's own entry, with
/// the right version, sat second and was discarded.
#[test]
fn a_registry_entry_sharing_a_members_name_is_not_the_members_entry() {
    let root = scratch("lock-name-shared");
    let fixture = build_fixture(&root, "lock-name-shared", "0.2.0");
    workspace_files(&fixture.repo, "0.2.1");
    release_changelog(&fixture.repo, "0.2.1", "0.2.0");
    let lock = fixture.repo.join("Cargo.lock");
    let text = std::fs::read_to_string(&lock).expect("read the fixture lock");
    let decoy = "version = 4\n\n[[package]]\nname = \"xuanji\"\nversion = \"9.9.9\"\n\
                 source = \"registry+https://example.invalid/index\"\n";
    std::fs::write(&lock, text.replacen("version = 4\n\n", decoy, 1)).expect("write");
    commit(
        &fixture.repo,
        "chore: add a registry entry sharing a member's name",
    );
    let verdict = judge(&fixture.repo);
    let _ = std::fs::remove_dir_all(&root);
    assert!(
        verdict.is_ok(),
        "the source-less entry is the workspace member's, whichever order the blocks are written in; \
         comparing against the registry entry reports a disagreement that is not one. Got: {:?}",
        verdict.err()
    );
}

/// Two source-less entries under one name is a cannot-judge, not a member picked by position.
///
/// The opposite answer to the direction above, and the reason selecting by `source` is not enough on its own:
/// if two entries both lack a source, which is the workspace member is genuinely undecided, and choosing
/// either would be deciding it by the order the file happens to be written in.
#[test]
fn two_source_less_entries_under_one_name_cannot_be_judged() {
    let root = scratch("lock-name-ambiguous");
    let fixture = build_fixture(&root, "lock-name-ambiguous", "0.2.0");
    workspace_files(&fixture.repo, "0.2.1");
    release_changelog(&fixture.repo, "0.2.1", "0.2.0");
    let lock = fixture.repo.join("Cargo.lock");
    let text = std::fs::read_to_string(&lock).expect("read the fixture lock");
    let twin = "version = 4\n\n[[package]]\nname = \"xuanji\"\nversion = \"9.9.9\"\n";
    std::fs::write(&lock, text.replacen("version = 4\n\n", twin, 1)).expect("write");
    commit(&fixture.repo, "chore: add a second source-less entry");
    refusal::expect(
        "release-coherence#lock-several-sourceless-entries",
        &refuse(&fixture.repo, Kind::CannotJudge, "with no source"),
    );
    let _ = std::fs::remove_dir_all(&root);
}

/// A table that is not `[[package]]` does not absorb the package block above it.
///
/// The block boundary was `[[package]]` **alone**, so every other table header read as ordinary content while
/// the block above it stayed open. `[[patch.unused]]` — which cargo writes whenever a `[patch]` section
/// exists — carries its own `name`, `version` and `source`, and those overwrote the open block's: the last
/// member's version was replaced before it was ever filed, so that member vanished from the map and the
/// workspace lookup reported `Cargo.lock is missing workspace package …` for a lock that holds it. A **false
/// accusation**, which is the class this module's own header says it exists to prevent.
///
/// Written at the END of the file, where cargo writes it, so the block it absorbs is the fixture's last
/// member rather than a position chosen to make the point.
#[test]
fn a_non_package_table_does_not_absorb_the_block_above_it() {
    let root = scratch("lock-foreign-table");
    let fixture = build_fixture(&root, "lock-foreign-table", "0.2.0");
    workspace_files(&fixture.repo, "0.2.1");
    release_changelog(&fixture.repo, "0.2.1", "0.2.0");
    let lock = fixture.repo.join("Cargo.lock");
    let text = std::fs::read_to_string(&lock).expect("read the fixture lock");
    std::fs::write(
        &lock,
        format!(
            "{text}\n[[patch.unused]]\nname = \"some-patched-crate\"\nversion = \"9.9.9\"\n\
             source = \"registry+https://example.invalid/index\"\n"
        ),
    )
    .expect("write");
    commit(&fixture.repo, "chore: add a patch.unused table to the lock");
    let verdict = judge(&fixture.repo);
    let _ = std::fs::remove_dir_all(&root);
    assert!(
        verdict.is_ok(),
        "a `[[patch.unused]]` table carries its own name and version; reading them into the package block \
         above it drops that member and reports it absent from a lock that records it. Got: {:?}",
        verdict.err()
    );
}

/// A commented-out internal pin is not a pin.
///
/// The three manifest readers took raw lines. `# xuanji = { path = "crates/xuanji" }` satisfies every
/// predicate the internal-pin filter applies — it contains `path`, `"crates/`, and `=` — so it was counted
/// as a declared pin and then refused for carrying no version: `internal dependency # xuanji has no version
/// pin`. A **false refusal**, in front of the release gate, over text that declares nothing. It also
/// inflated the `pins == 0` vacuity guard, so a manifest whose only "pins" were commentary would have read
/// as a manifest that had been checked.
///
/// The sibling reader four hundred lines up had stripped `#` by hand the whole time, so one file read one
/// corpus two ways. All three go through `region` now.
#[test]
fn a_commented_out_internal_pin_is_not_a_pin() {
    let root = scratch("commented-pin");
    let fixture = build_fixture(&root, "commented-pin", "0.2.0");
    workspace_files(&fixture.repo, "0.2.1");
    release_changelog(&fixture.repo, "0.2.1", "0.2.0");
    let manifest = fixture.repo.join("Cargo.toml");
    let text = std::fs::read_to_string(&manifest).expect("read the fixture manifest");
    std::fs::write(
        &manifest,
        format!("{text}\n# hunyi = {{ path = \"crates/hunyi\" }}\n"),
    )
    .expect("write");
    commit(&fixture.repo, "chore: comment out an internal pin");
    let verdict = judge(&fixture.repo);
    let _ = std::fs::remove_dir_all(&root);
    assert!(
        verdict.is_ok(),
        "a commented-out dependency declares nothing, so refusing it names a disagreement no manifest \
         makes. Got: {:?}",
        verdict.err()
    );
}

/// An inherit line with a glued comment still inherits.
///
/// `version.workspace = true#c` is legal TOML: the grammar allows zero whitespace before a comment. So this
/// member inherits, and a reader that refuses it refuses a valid manifest.
///
/// **What this doc said, and what happened to it.** It was written to defend a decision to keep two manifest
/// readers out of `region`, on the ground that `toml()`'s token-start rule read that `#` as content. Three
/// commits later `toml()` stopped using the token-start rule — it lexes strings and cuts where TOML cuts —
/// and both readers were converted. The measurement was right about the rule of the day; the conclusion it
/// carried is gone, and this paragraph outlived it by one file. The commit that reversed the decision swept
/// `CHANGELOG.md` for the superseded conclusion and did not sweep the test whose own subject the reversal
/// changed.
///
/// **What holds it now is a different edit.** Reverting `toml()` to `Rule::TokenStart("#")` turns this red,
/// which is the same one-line negative run under a rule the reader no longer has a choice about. The
/// direction this replaced asserted the same fact against **its own copy** of the predicate and called
/// nothing in the gate, so no edit to the product could turn it at all — the question that separates the two
/// being the cheap one: *which change to the product makes this red?*
#[test]
fn an_inherit_line_with_a_glued_comment_still_inherits() {
    let root = scratch("glued-comment");
    let fixture = build_fixture(&root, "glued-comment", "0.2.0");
    workspace_files(&fixture.repo, "0.2.1");
    release_changelog(&fixture.repo, "0.2.1", "0.2.0");
    let member = fixture.repo.join("crates/xuanji/Cargo.toml");
    let text = std::fs::read_to_string(&member).expect("read the member manifest");
    std::fs::write(
        &member,
        text.replace("version.workspace = true", "version.workspace = true#c"),
    )
    .expect("write");
    commit(&fixture.repo, "chore: glue a comment to the inherit line");
    let verdict = judge(&fixture.repo);
    let _ = std::fs::remove_dir_all(&root);
    assert!(
        verdict.is_ok(),
        "TOML allows zero whitespace before a comment, so this member still inherits; refusing it would be \
         a false refusal introduced by reading the line under a rule written for a different language. \
         Got: {:?}",
        verdict.err()
    );
}

/// A member whose only inherit line is commented out is refused.
///
/// The other direction of the same predicate, through the same entry point: a comment declares nothing, so
/// the member does not inherit and the gate says which one.
#[test]
fn a_member_whose_only_inherit_line_is_commented_out_is_refused() {
    let root = scratch("commented-inherit");
    let fixture = build_fixture(&root, "commented-inherit", "0.2.0");
    workspace_files(&fixture.repo, "0.2.1");
    release_changelog(&fixture.repo, "0.2.1", "0.2.0");
    let member = fixture.repo.join("crates/xuanji/Cargo.toml");
    let text = std::fs::read_to_string(&member).expect("read the member manifest");
    std::fs::write(
        &member,
        text.replace("version.workspace = true", "# version.workspace = true"),
    )
    .expect("write");
    commit(&fixture.repo, "chore: comment out the inherit line");
    refusal::expect(
        "release-coherence#member-does-not-inherit-workspace-version",
        &refuse(
            &fixture.repo,
            Kind::Violation,
            "must inherit version.workspace",
        ),
    );
    let _ = std::fs::remove_dir_all(&root);
}

/// A basename the enumerator does not resolve is not machinery, however much it looks like a gate.
#[test]
fn a_basename_the_enumerator_does_not_resolve_is_coherent() {
    let root = scratch("unresolved");
    let fixture = build_fixture(&root, "unresolved", "0.2.0");
    development_changelog(&fixture.repo, "0.2.0", true);
    with_machinery(&fixture.repo);
    unreleased_body(
        &fixture.repo,
        "### Fixed\n- A repair in a tool named `check_something_the_repository_does_not_track.sh`.",
    );
    commit(
        &fixture.repo,
        "docs: name a file no scripts/ entry resolves",
    );
    let verdict = judge(&fixture.repo);
    let _ = std::fs::remove_dir_all(&root);
    assert!(verdict.is_ok(), "{:?}", verdict.err());
}

// --- declared bounds ----------------------------------------------------------------------------------------

/// Refuse to accept silence from a check that is not running at all.
///
/// Every bound below asserts SILENCE, and silence has more than one cause. Adversarial review found the sharp
/// case by widening a scope AND blinding an enumerator at once: a bound was then plainly false and its pin
/// stayed green, because a dead check is silent about everything. Only a live control reaches that.
fn assert_reaction_is_live(root: &Path) {
    let fixture = build_fixture(root, "live-control", "0.2.0");
    development_changelog(&fixture.repo, "0.2.0", true);
    with_machinery(&fixture.repo);
    unreleased_body(
        &fixture.repo,
        "### Fixed\n- A repair naming `scripts/check_fixture_gate.sh`.",
    );
    commit(&fixture.repo, "docs: the live control");
    let verdict = judge(&fixture.repo);
    assert!(
        verdict.is_err(),
        "the control must be refused, or the silence a pin is about to assert says nothing — a check that \
         refuses nothing is silent about every bound at once"
    );
}

/// `release-coherence/a-dated-release-section-names-a-gate-a-stated-bound`
///
/// `UnderReacts`, owned by the engine. The leak is real — an adopter reading a dated section meets entries
/// naming files they can never run — and what is refused is the *repair*: rewriting a dated section to satisfy
/// a rule written afterwards would falsify the record, the reason `docs/history/` is left alone too.
#[test]
fn a_dated_section_naming_a_gate_is_a_stated_bound() {
    let root = scratch("bound-dated");
    assert_reaction_is_live(&root);
    let fixture = build_fixture(&root, "dated", "0.2.0");
    with_machinery(&fixture.repo);
    let path = fixture.repo.join("CHANGELOG.md");
    let text = std::fs::read_to_string(&path).expect("read");
    std::fs::write(
        &path,
        text.replace(
            "## [Unreleased]\n",
            "## [Unreleased]\n\n- An adopter-facing change.\n",
        )
        .replace(
            "- Release notes.\n",
            "### Fixed\n- A repair naming `scripts/check_fixture_gate.sh`.\n",
        ),
    )
    .expect("write");
    commit(&fixture.repo, "docs: a dated section names a gate");
    let verdict = judge(&fixture.repo);
    let _ = std::fs::remove_dir_all(&root);
    assert!(
        verdict.is_ok(),
        "the check must stay silent about a dated section naming machinery — that is the declared bound. \
         Got: {:?}",
        verdict.err()
    );
}

/// `release-coherence/machinery-the-judged-repository-tracks-by-nothing-a-stated-bound`
///
/// `UnderReacts`, owned by the engine. The enumeration is `git ls-files scripts/`, so an untracked `scripts/`
/// reads as absent; closing this means judging worktree content, which this repository's gates are held not
/// to do — the larger error.
#[test]
fn machinery_tracked_by_nothing_is_a_stated_bound() {
    let root = scratch("bound-untracked");
    assert_reaction_is_live(&root);
    let fixture = build_fixture(&root, "untracked", "0.2.0");
    development_changelog(&fixture.repo, "0.2.0", true);
    unreleased_body(
        &fixture.repo,
        "### Fixed\n- A repair naming `scripts/check_fixture_gate.sh`.",
    );
    commit(&fixture.repo, "docs: name a gate before it is tracked");
    with_machinery(&fixture.repo); // written, never added
    let verdict = judge(&fixture.repo);
    let _ = std::fs::remove_dir_all(&root);
    assert!(
        verdict.is_ok(),
        "the check must stay silent about machinery no commit tracks. Got: {:?}",
        verdict.err()
    );
}

/// `release-coherence/a-basename-an-entry-writes-for-another-reason-a-stated-bound`
///
/// `OverReacts`. A word is matched against basenames as well as paths, because the document cites both forms.
/// An entry naming a file of its own whose basename the repository also tracks is refused, and the entry is
/// innocent — the safe direction, and narrowing it means deciding which of two files a bare name meant.
#[test]
fn a_colliding_basename_is_a_stated_bound() {
    let root = scratch("bound-collide");
    let fixture = build_fixture(&root, "collide", "0.2.0");
    development_changelog(&fixture.repo, "0.2.0", true);
    std::fs::create_dir_all(fixture.repo.join("scripts")).expect("create");
    std::fs::write(
        fixture.repo.join("scripts/publish.sh"),
        "#!/usr/bin/env bash\nexit 0\n",
    )
    .expect("write");
    unreleased_body(
        &fixture.repo,
        "### Fixed\n- Adopters run their own `publish.sh` after upgrading.",
    );
    commit(
        &fixture.repo,
        "docs: write a name the repository also tracks",
    );
    let verdict = judge(&fixture.repo);
    let _ = std::fs::remove_dir_all(&root);
    let refusal =
        verdict.expect_err("the false refusal is the declared bound; silence would close it");
    assert!(
        refusal.message.contains("publish.sh"),
        "the refusal must name the colliding word: {}",
        refusal.message
    );
}

/// `release-coherence/a-directory-named-without-its-trailing-slash-a-stated-bound`
///
/// `UnderReacts`, owned by the engine. Directories are derived slash-terminated; the unslashed form is a word
/// indistinguishable from prose — `scripts` is an English plural this repository's own changelog uses as one.
#[test]
fn a_directory_named_without_its_slash_is_a_stated_bound() {
    let root = scratch("bound-unslashed");
    assert_reaction_is_live(&root);
    let fixture = build_fixture(&root, "unslashed", "0.2.0");
    development_changelog(&fixture.repo, "0.2.0", true);
    with_machinery(&fixture.repo);
    unreleased_body(
        &fixture.repo,
        "### Fixed\n- A repair to the scripts and to a shared library, written without a trailing slash.",
    );
    commit(&fixture.repo, "docs: name a directory without its slash");
    let verdict = judge(&fixture.repo);
    let _ = std::fs::remove_dir_all(&root);
    assert!(
        verdict.is_ok(),
        "the check must stay silent about a directory named without its trailing slash. Got: {:?}",
        verdict.err()
    );
}

/// `release-coherence/a-name-reached-only-through-a-url-a-stated-bound`
///
/// `UnderReacts`, owned by the engine. A word is a maximal run of path characters, so a scheme and host fuse
/// with the path into one run that equals no tracked name; splitting a URL would make the check judge a
/// foreign host's layout as though it were this repository's.
#[test]
fn a_name_reached_only_through_a_url_is_a_stated_bound() {
    let root = scratch("bound-url");
    assert_reaction_is_live(&root);
    let fixture = build_fixture(&root, "url", "0.2.0");
    development_changelog(&fixture.repo, "0.2.0", true);
    with_machinery(&fixture.repo);
    unreleased_body(
        &fixture.repo,
        "### Fixed\n- See https://github.com/tacticaldoll/tianheng/blob/main/scripts/check_fixture_gate.sh for it.",
    );
    commit(&fixture.repo, "docs: reach a gate only through a URL");
    let verdict = judge(&fixture.repo);
    let _ = std::fs::remove_dir_all(&root);
    assert!(
        verdict.is_ok(),
        "the check must stay silent about a name reached only through a URL. Got: {:?}",
        verdict.err()
    );
}

/// A `### ` line inside a fence does not set the heading in force, so it cannot exempt a later entry.
///
/// **A retired bound, with its own WHEN re-run rather than deleted.** It was declared under
/// `release-coherence` as *a heading inside a fenced code block*, an `UnderReacts` owned by the
/// engine — named in words here rather than by its id, because a retired id resolving to nothing is what
/// `every_bare_bound_reference_resolves_to_a_declared_bound` refuses, and it is right to: an id that points
/// nowhere reads exactly like an undeclared bound. The check walked the document's line grammar without
/// tracking fences, so a fenced
/// `### Self-governance` set the exempt heading and every entry after it went unreported — a false negative,
/// latent only because this repository's changelog carried no fenced block.
///
/// The bound's stated cost was *a second, stateful reading of a document this gate reads once*, and that
/// premise is what retired it rather than a decision to pay the cost. `region::Prose` already tracks fences,
/// for every reader in the crate, so the gate reads a region instead of text and the second reading is one
/// that already existed. The fixture below is the bound's own WHEN, unchanged; only the THEN moved.
#[test]
fn a_heading_inside_a_fenced_block_does_not_reattribute_a_later_entry() {
    let root = scratch("bound-fenced");
    assert_reaction_is_live(&root);
    let fixture = build_fixture(&root, "fenced", "0.2.0");
    development_changelog(&fixture.repo, "0.2.0", true);
    with_machinery(&fixture.repo);
    unreleased_body(
        &fixture.repo,
        "### Fixed\n- A repair.\n\n```\n### Self-governance\n```\n\n- A later repair naming `scripts/check_fixture_gate.sh`.",
    );
    commit(&fixture.repo, "docs: put a heading inside a fence");
    let verdict = judge(&fixture.repo);
    let _ = std::fs::remove_dir_all(&root);
    let refusal = verdict.expect_err(
        "a fenced `### Self-governance` no longer sets the heading in force, so the entry naming machinery \
         after it is reported rather than exempt",
    );
    refusal::expect(
        "release-coherence#adopter-entry-names-own-machinery",
        &refusal,
    );
}

// --- the changelog surfaces and the vacuity guards, which nothing covered ----------------------------------

#[test]
fn two_unreleased_sections_are_a_violation() {
    let root = scratch("two-unreleased");
    let fixture = build_fixture(&root, "two-unreleased", "0.2.0");
    development_changelog(&fixture.repo, "0.2.0", true);
    let path = fixture.repo.join("CHANGELOG.md");
    let text = std::fs::read_to_string(&path).expect("read");
    std::fs::write(
        &path,
        text.replace("## [Unreleased]\n", "## [Unreleased]\n\n## [Unreleased]\n"),
    )
    .expect("write");
    commit(&fixture.repo, "chore: grow a second unreleased section");
    refusal::expect(
        "release-coherence#unreleased-section-not-exactly-one",
        &refuse(
            &fixture.repo,
            Kind::Violation,
            "exactly one [Unreleased] section",
        ),
    );
    let _ = std::fs::remove_dir_all(&root);
}

#[test]
fn a_snapshot_whose_unreleased_carries_an_item_is_a_violation() {
    let root = scratch("snapshot-unreleased");
    let fixture = build_fixture(&root, "snapshot-unreleased", "0.2.0");
    let path = fixture.repo.join("CHANGELOG.md");
    let text = std::fs::read_to_string(&path).expect("read");
    std::fs::write(
        &path,
        text.replace(
            "## [Unreleased]\n",
            "## [Unreleased]\n\n- A leftover item.\n",
        ),
    )
    .expect("write");
    git(&fixture.repo, &["add", "."]);
    git(
        &fixture.repo,
        &["commit", "-q", "--amend", "-m", "release: 0.2.0"],
    );
    refusal::expect(
        "release-coherence#unreleased-not-empty-in-state",
        &refuse(
            &fixture.repo,
            Kind::Violation,
            "must be empty in snapshot state",
        ),
    );
    let _ = std::fs::remove_dir_all(&root);
}

#[test]
fn a_release_with_no_dated_notes_is_a_violation() {
    let root = scratch("no-dated-notes");
    let fixture = build_fixture(&root, "no-dated-notes", "0.2.0");
    workspace_files(&fixture.repo, "0.2.1");
    development_changelog(&fixture.repo, "0.2.1", false);
    commit(&fixture.repo, "chore: prepare without notes");
    refusal::expect(
        "release-coherence#dated-release-notes-missing",
        &refuse(
            &fixture.repo,
            Kind::Violation,
            "missing dated release notes for 0.2.1",
        ),
    );
    let _ = std::fs::remove_dir_all(&root);
}

#[test]
fn an_unreleased_comparison_link_that_does_not_start_at_the_version_is_a_violation() {
    let root = scratch("bad-unreleased-link");
    let fixture = build_fixture(&root, "bad-unreleased-link", "0.2.0");
    development_changelog(&fixture.repo, "0.2.0", true);
    let path = fixture.repo.join("CHANGELOG.md");
    let text = std::fs::read_to_string(&path).expect("read");
    std::fs::write(&path, text.replace("v0.2.0...HEAD", "v0.1.0...HEAD")).expect("write");
    commit(&fixture.repo, "chore: point the link at the wrong version");
    refusal::expect(
        "release-coherence#unreleased-comparison-link-wrong",
        &refuse(
            &fixture.repo,
            Kind::Violation,
            "comparison link must start at v0.2.0",
        ),
    );
    let _ = std::fs::remove_dir_all(&root);
}

#[test]
fn a_dated_comparison_link_that_does_not_start_at_the_previous_release_is_a_violation() {
    let root = scratch("bad-dated-link");
    let fixture = build_fixture(&root, "bad-dated-link", "0.2.0");
    workspace_files(&fixture.repo, "0.2.1");
    release_changelog(&fixture.repo, "0.2.1", "0.0.9");
    commit(
        &fixture.repo,
        "chore: point the release link at the wrong predecessor",
    );
    refusal::expect(
        "release-coherence#release-comparison-link-wrong",
        &refuse(&fixture.repo, Kind::Violation, "must start at v0.2.0"),
    );
    let _ = std::fs::remove_dir_all(&root);
}

#[test]
fn a_lockfile_missing_a_workspace_package_is_a_violation() {
    let root = scratch("lock-missing");
    let fixture = build_fixture(&root, "lock-missing", "0.2.0");
    workspace_files(&fixture.repo, "0.2.1");
    release_changelog(&fixture.repo, "0.2.1", "0.2.0");
    let lock = fixture.repo.join("Cargo.lock");
    let text = std::fs::read_to_string(&lock).expect("read");
    std::fs::write(
        &lock,
        text.replace(
            "\n[[package]]\nname = \"xuanji\"\nversion = \"0.2.1\"\n",
            "\n",
        ),
    )
    .expect("write");
    commit(&fixture.repo, "chore: drop a package from the lockfile");
    refusal::expect(
        "release-coherence#lock-missing-workspace-package",
        &refuse(
            &fixture.repo,
            Kind::Violation,
            "Cargo.lock is missing workspace package xuanji",
        ),
    );
    let _ = std::fs::remove_dir_all(&root);
}

#[test]
fn an_internal_dependency_with_no_version_pin_is_a_violation() {
    let root = scratch("unpinned-internal");
    let fixture = build_fixture(&root, "unpinned-internal", "0.2.0");
    let manifest = fixture.repo.join("Cargo.toml");
    let text = std::fs::read_to_string(&manifest).expect("read");
    std::fs::write(
        &manifest,
        text.replace(
            "xuanji = { path = \"crates/xuanji\", version = \"0.2.0\" }",
            "xuanji = { path = \"crates/xuanji\" }",
        ),
    )
    .expect("write");
    development_changelog(&fixture.repo, "0.2.0", true);
    commit(&fixture.repo, "chore: drop the internal pin");
    refusal::expect(
        "release-coherence#internal-pin-absent",
        &refuse(&fixture.repo, Kind::Violation, "has no version pin"),
    );
    let _ = std::fs::remove_dir_all(&root);
}

#[test]
fn a_snapshot_whose_version_disagrees_with_its_subject_is_a_violation() {
    let root = scratch("snapshot-mismatch");
    let fixture = build_fixture(&root, "snapshot-mismatch", "0.2.0");
    workspace_files(&fixture.repo, "0.3.0");
    git(&fixture.repo, &["add", "."]);
    git(
        &fixture.repo,
        &["commit", "-q", "--amend", "-m", "release: 0.2.0"],
    );
    refuse(
        &fixture.repo,
        Kind::Violation,
        "release snapshot subject is 0.2.0 but workspace version is 0.3.0",
    );
    let _ = std::fs::remove_dir_all(&root);
}

#[test]
fn a_release_subject_with_no_space_is_a_violation() {
    let root = scratch("no-space-subject");
    let fixture = build_fixture(&root, "no-space-subject", "0.2.0");
    development_changelog(&fixture.repo, "0.2.0", true);
    commit(&fixture.repo, "release:0.3.0");
    refusal::expect(
        "release-coherence#release-history-subject-malformed",
        &refuse(
            &fixture.repo,
            Kind::Violation,
            "malformed release history subject",
        ),
    );
    let _ = std::fs::remove_dir_all(&root);
}

/// The vacuity guards — the direction this judgement's own doc-comment argues for, and which nothing covered.
///
/// Each removes the thing an enumeration counts, so the loop that judges it would otherwise iterate nothing
/// and report clean. That is the reads-as-coverage failure one level up.
#[test]
fn every_enumeration_refuses_rather_than_reporting_clean_over_nothing() {
    for (_shape, wreck, needle) in [
        (
            "no dependency on a family crate",
            "internal-pins" as &str,
            "found no dependency on a family crate",
        ),
        // Two sites once produced one needle — an unreadable directory and a readable empty one — so the
        // direction could not say which fired, and only the first was ever reached.
        (
            "an unreadable examples directory",
            "examples",
            "found no enumerable directory at",
        ),
        (
            "an examples directory holding no manifest",
            "examples-empty",
            "found no example manifests",
        ),
        // Per example since the counter moved inside the loop: the aggregate could not see one example
        // going unexamined beside siblings that parsed, which is the read this enumeration now covers.
        (
            "an example requiring no family crate",
            "example-reqs",
            "declares no family dependency requirement",
        ),
    ] {
        let root = scratch(wreck);
        let fixture = build_fixture(&root, wreck, "0.2.0");
        development_changelog(&fixture.repo, "0.2.0", true);
        match wreck {
            "internal-pins" => {
                let manifest = fixture.repo.join("Cargo.toml");
                let text = std::fs::read_to_string(&manifest).expect("read");
                let cut = text
                    .lines()
                    .filter(|l| !l.contains("path = \"crates/"))
                    .collect::<Vec<_>>()
                    .join("\n");
                std::fs::write(&manifest, cut).expect("write");
            }
            "examples" => {
                std::fs::remove_dir_all(fixture.repo.join("examples")).expect("remove examples");
            }
            "examples-empty" => {
                std::fs::remove_dir_all(fixture.repo.join("examples")).expect("remove examples");
                std::fs::create_dir_all(fixture.repo.join("examples")).expect("recreate empty");
            }
            "example-reqs" => {
                std::fs::write(
                    fixture.repo.join("examples/adopter/Cargo.toml"),
                    "[package]\nname = \"adopter\"\nversion = \"0.0.0\"\nedition = \"2024\"\n",
                )
                .expect("write");
            }
            _ => unreachable!(),
        }
        commit(&fixture.repo, "chore: empty an enumeration");
        refuse(&fixture.repo, Kind::CannotJudge, needle);
        let _ = std::fs::remove_dir_all(&root);
    }
}

// --- the inputs this judgement cannot read ---------------------------------------------------------------

/// A bare directory, before anything has been laid out in it.
fn bare(root: &Path, name: &str) -> PathBuf {
    let repo = root.join(name);
    std::fs::create_dir_all(&repo).expect("the fixture root is writable");
    repo
}

fn initialised(repo: &Path) {
    git(repo, &["init", "-q", "-b", "main"]);
    git(repo, &["config", "user.name", "T"]);
    git(repo, &["config", "user.email", "t@example.invalid"]);
    git(repo, &["config", "commit.gpgsign", "false"]);
}

#[test]
fn a_root_without_a_manifest_cannot_be_judged() {
    let root = scratch("no-manifest");
    let repo = bare(&root, "repo");
    refusal::expect(
        "release-coherence#repository-root-has-no-manifest",
        &refuse(&repo, Kind::CannotJudge, "has no Cargo.toml"),
    );
    let _ = std::fs::remove_dir_all(&root);
}

#[test]
fn a_root_without_a_changelog_cannot_be_judged() {
    let root = scratch("no-changelog");
    let repo = bare(&root, "repo");
    std::fs::write(repo.join("Cargo.toml"), "[workspace]\n").expect("write");
    refusal::expect(
        "release-coherence#repository-root-has-no-changelog",
        &refuse(&repo, Kind::CannotJudge, "has no CHANGELOG.md"),
    );
    let _ = std::fs::remove_dir_all(&root);
}

/// Not a git worktree at all — a different fact from a history too shallow to read.
#[test]
fn a_root_that_is_not_a_worktree_cannot_be_judged() {
    let root = scratch("no-worktree");
    let repo = bare(&root, "repo");
    std::fs::write(repo.join("Cargo.toml"), "[workspace]\n").expect("write");
    std::fs::write(repo.join("CHANGELOG.md"), "# Changelog\n").expect("write");
    refusal::expect(
        "release-coherence#git-unrunnable",
        &refuse(&repo, Kind::CannotJudge, "has no git history"),
    );
    let _ = std::fs::remove_dir_all(&root);
}

#[test]
fn a_manifest_with_no_workspace_version_cannot_be_judged() {
    let root = scratch("no-version");
    let repo = bare(&root, "repo");
    initialised(&repo);
    std::fs::write(
        repo.join("Cargo.toml"),
        "[workspace.package]\nedition = \"2024\"\n",
    )
    .expect("write");
    std::fs::write(repo.join("CHANGELOG.md"), "# Changelog\n").expect("write");
    refusal::expect(
        "release-coherence#workspace-version-absent",
        &refuse(
            &repo,
            Kind::CannotJudge,
            "workspace version is missing or malformed",
        ),
    );
    let _ = std::fs::remove_dir_all(&root);
}

/// A comment on the table heading does not close the table before it opens.
///
/// `[workspace.package] # …` is legal TOML. Read as a raw line it fails an equality against
/// `[workspace.package]`, then matches "starts with `[`" and *closes* the scan — so the version is reported
/// absent over a manifest that declares one. The refusal expected here is the **next** phase's, which is what
/// says the version was read: this fixture has no commit, so the release history is what cannot be read.
#[test]
fn a_commented_table_heading_still_opens_the_workspace_package_table() {
    let root = scratch("commented-table-heading");
    let repo = bare(&root, "repo");
    initialised(&repo);
    std::fs::write(
        repo.join("Cargo.toml"),
        "[workspace.package] # the version every member inherits\nversion = \"0.2.0\"\n",
    )
    .expect("write");
    std::fs::write(repo.join("CHANGELOG.md"), "# Changelog\n").expect("write");
    refusal::expect(
        "release-coherence#release-history-unreadable",
        &refuse(
            &repo,
            Kind::CannotJudge,
            "could not read the release history",
        ),
    );
    let _ = std::fs::remove_dir_all(&root);
}

/// A trailing comment on the version line does not become part of the version.
///
/// The release-prep spelling: `version = "0.3.0"  # bumped for the window`. Read as a raw line, the value
/// carries the comment with it and no longer parses as a semantic version — a false cannot-judge over a legal
/// manifest, at the one moment someone is most likely to annotate that line. As above, the refusal expected
/// is the next phase's.
#[test]
fn a_trailing_comment_on_the_version_line_still_reads_the_version() {
    let root = scratch("commented-version-value");
    let repo = bare(&root, "repo");
    initialised(&repo);
    std::fs::write(
        repo.join("Cargo.toml"),
        "[workspace.package]\nversion = \"0.3.0\"  # bumped for the release window\n",
    )
    .expect("write");
    std::fs::write(repo.join("CHANGELOG.md"), "# Changelog\n").expect("write");
    refusal::expect(
        "release-coherence#release-history-unreadable",
        &refuse(
            &repo,
            Kind::CannotJudge,
            "could not read the release history",
        ),
    );
    let _ = std::fs::remove_dir_all(&root);
}

/// A value this reader cannot read is not a value that is absent.
///
/// Reporting an unreadable value as *missing or malformed* sends an operator to look for a version key that
/// is sitting right there — the same conflation `Quoted` was introduced to end one reader over.
///
/// **This WHEN moved when a real parser replaced the hand-rolled reader.** It was a single-quoted literal,
/// valid TOML the old reader declined; the parser takes it, so that shape now reports the version it declares
/// and the limitation is gone rather than declared. What still reaches this site is a value that is not a
/// string at all — here the catalog declaring that it inherits, which is the table that declares the catalog
/// inheriting from itself. The site is kept because its WHEN was rerun against the new reader.
#[test]
fn a_version_value_this_reader_cannot_read_is_not_one_that_is_absent() {
    let root = scratch("unreadable-version-value");
    let repo = bare(&root, "repo");
    initialised(&repo);
    std::fs::write(
        repo.join("Cargo.toml"),
        "[workspace.package]\nversion = { workspace = true }\n",
    )
    .expect("write");
    std::fs::write(repo.join("CHANGELOG.md"), "# Changelog\n").expect("write");
    refusal::expect(
        "release-coherence#workspace-version-unreadable",
        &refuse(
            &repo,
            Kind::CannotJudge,
            "declares a workspace version this check cannot read",
        ),
    );
    let _ = std::fs::remove_dir_all(&root);
}

/// A repository with no commit at all: the release history cannot be read, which is not a shallow clone.
#[test]
fn a_repository_with_no_commit_cannot_have_its_history_read() {
    let root = scratch("no-commit");
    let repo = bare(&root, "repo");
    initialised(&repo);
    std::fs::write(
        repo.join("Cargo.toml"),
        "[workspace.package]\nversion = \"0.2.0\"\n",
    )
    .expect("write");
    std::fs::write(repo.join("CHANGELOG.md"), "# Changelog\n").expect("write");
    refusal::expect(
        "release-coherence#release-history-unreadable",
        &refuse(
            &repo,
            Kind::CannotJudge,
            "could not read the release history",
        ),
    );
    let _ = std::fs::remove_dir_all(&root);
}

/// A tracked path that is a directory where a file is expected: the read fails rather than returning empty.
#[test]
fn a_lockfile_that_is_a_directory_cannot_be_read() {
    let root = scratch("lock-directory");
    let fixture = build_fixture(&root, "lock-directory", "0.2.0");
    std::fs::remove_file(fixture.repo.join("Cargo.lock")).expect("remove the lockfile");
    std::fs::create_dir(fixture.repo.join("Cargo.lock")).expect("put a directory in its place");
    refusal::expect(
        "release-coherence#changelog-or-manifest-unreadable",
        &refuse(
            &fixture.repo,
            Kind::CannotJudge,
            "could not read Cargo.lock",
        ),
    );
    let _ = std::fs::remove_dir_all(&root);
}

#[test]
fn an_absent_crate_directory_cannot_be_enumerated() {
    let root = scratch("no-crates");
    let fixture = build_fixture(&root, "no-crates", "0.2.0");
    std::fs::remove_dir_all(fixture.repo.join("crates")).expect("remove crates/");
    refusal::expect(
        "release-coherence#directory-not-enumerable",
        &refuse(
            &fixture.repo,
            Kind::CannotJudge,
            "found no enumerable directory at",
        ),
    );
    let _ = std::fs::remove_dir_all(&root);
}

/// The directory is there and holds no manifest — a different read from the directory being absent.
#[test]
fn a_crate_directory_holding_no_manifest_cannot_be_enumerated() {
    let root = scratch("empty-crates");
    let fixture = build_fixture(&root, "empty-crates", "0.2.0");
    std::fs::remove_dir_all(fixture.repo.join("crates")).expect("remove crates/");
    std::fs::create_dir(fixture.repo.join("crates")).expect("recreate it empty");
    refusal::expect(
        "release-coherence#no-crate-manifests-found",
        &refuse(
            &fixture.repo,
            Kind::CannotJudge,
            "found no workspace crate manifests under crates/",
        ),
    );
    let _ = std::fs::remove_dir_all(&root);
}

/// A member manifest that is a file and is still not readable **as text**.
///
/// A directory in its place is skipped, because the enumeration asks `is_file()` first — measured, not
/// assumed: the first attempt put a directory there and the judgement sailed past to a later check. Invalid
/// UTF-8 is the shape that satisfies `is_file()` and fails the read, and it needs no permission games that a
/// run as root would defeat.
#[test]
fn a_member_manifest_that_is_not_text_cannot_be_read() {
    let root = scratch("manifest-not-text");
    let fixture = build_fixture(&root, "manifest-not-text", "0.2.0");
    let manifest = fixture.repo.join("crates/xuanji/Cargo.toml");
    std::fs::write(&manifest, [0x5b, 0x70, 0xff, 0xfe, 0x5d])
        .expect("write bytes that are not UTF-8");
    refuse(&fixture.repo, Kind::CannotJudge, "could not read");
    let _ = std::fs::remove_dir_all(&root);
}

/// The machinery enumeration reads the index; an index git cannot parse cannot be enumerated.
#[test]
fn machinery_that_cannot_be_enumerated_cannot_be_judged() {
    let root = scratch("unreadable-index");
    let fixture = build_fixture(&root, "unreadable-index", "0.2.0");
    std::fs::write(fixture.repo.join(".git/index"), b"not an index").expect("corrupt the index");
    refusal::expect(
        "release-coherence#directory-listing-unreadable",
        &refuse(&fixture.repo, Kind::CannotJudge, "could not enumerate"),
    );
    let _ = std::fs::remove_dir_all(&root);
}

/// An example manifest that exists and cannot be read is a cannot-judge, not a directory holding none.
///
/// Skipping both alike let the remaining readable examples satisfy the counters this judgement reasons from,
/// so a run reported clean over the very manifest it could not read. Invalid UTF-8 is the shape that satisfies
/// `is_file()` and fails the read, needing no permission games a run as root would defeat.
#[test]
fn an_example_manifest_that_is_not_text_cannot_be_read() {
    let root = scratch("example-not-text");
    let fixture = build_fixture(&root, "example-not-text", "0.2.0");
    let manifest = fixture.repo.join("examples/adopter/Cargo.toml");
    assert!(manifest.is_file(), "the fixture builds an example manifest");
    std::fs::write(&manifest, [0x5b, 0x70, 0xff, 0xfe, 0x5d])
        .expect("write bytes that are not UTF-8");
    refusal::expect(
        "release-coherence#example-manifest-unreadable",
        &refuse(
            &fixture.repo,
            Kind::CannotJudge,
            "could not read the example manifest",
        ),
    );
    let _ = std::fs::remove_dir_all(&root);
}

/// A directory under `examples/` holding no manifest is still skipped — absence is not a failed read.
#[test]
fn an_example_directory_holding_no_manifest_is_skipped() {
    let root = scratch("example-no-manifest");
    let fixture = build_fixture(&root, "example-no-manifest", "0.2.0");
    std::fs::create_dir_all(fixture.repo.join("examples/notes")).expect("create");
    std::fs::write(fixture.repo.join("examples/notes/README.md"), "prose\n").expect("write");
    development_changelog(&fixture.repo, "0.2.0", true);
    commit(
        &fixture.repo,
        "docs: add a directory that is not an example crate",
    );
    let verdict = judge(&fixture.repo);
    let _ = std::fs::remove_dir_all(&root);
    assert!(
        verdict.is_ok(),
        "a directory with no Cargo.toml was treated as a failed read: {:?}",
        verdict.err()
    );
}

/// A glued comment cannot supply a version pin the manifest does not declare.
///
/// TOML admits zero whitespace before `#`, so `{ path = "crates/xuanji" }#, version = "0.2.0"` declares a
/// dependency with **no** version. The token-start rule read that `#` as content, found `version` in the
/// commented tail, and passed the pin — a false pass in front of `cargo publish`, where the crate would then
/// be rejected by the registry for the pin the gate had just certified.
///
/// The sibling of `an_inherit_line_with_a_glued_comment_still_inherits`: the same blindness, the opposite
/// direction of error, which is why the rule had to become TOML's rather than either predicate's.
#[test]
fn a_glued_comment_cannot_supply_an_internal_version_pin() {
    let root = scratch("glued-pin");
    let fixture = build_fixture(&root, "glued-pin", "0.2.0");
    let manifest = fixture.repo.join("Cargo.toml");
    let text = std::fs::read_to_string(&manifest).expect("read");
    std::fs::write(
        &manifest,
        text.replace(
            "xuanji = { path = \"crates/xuanji\", version = \"0.2.0\" }",
            "xuanji = { path = \"crates/xuanji\" }#, version = \"0.2.0\"",
        ),
    )
    .expect("write");
    development_changelog(&fixture.repo, "0.2.0", true);
    commit(&fixture.repo, "chore: glue the pin into a comment");
    refusal::expect(
        "release-coherence#internal-pin-absent",
        &refuse(&fixture.repo, Kind::Violation, "has no version pin"),
    );
    let _ = std::fs::remove_dir_all(&root);
}

/// A `[package]` heading carrying a trailing comment still opens the package table.
///
/// The measured difference between reading raw manifest lines and reading executed ones in `package_name`,
/// and it is the table heading rather than the `name` value. `[package] # the repository checks` fails
/// `trimmed == "[package]"`, so the whole table is skipped, no `name` is found, and the manifest reports
/// `Absent` — which `require_example_pins` turns into `cannot_judge`, refusing to judge a release over a
/// perfectly legal manifest.
///
/// Recorded because the entry claiming this conversion's benefit named the wrong one: it said
/// `name = "kanhe" # the repository checks` had been answering `Unreadable`. It had not.
/// `quoted_value`, the reader of the time, took the text between the first pair of quotes and discarded
/// what followed, so a trailing comment there was always read correctly. The claim was refuted by a reviewer and is replaced by
/// the direction the measurement actually supports.
#[test]
fn a_package_heading_with_a_trailing_comment_still_opens_the_table() {
    let root = scratch("commented-heading");
    let fixture = build_fixture(&root, "commented-heading", "0.2.0");
    let member = fixture.repo.join("crates/xuanji/Cargo.toml");
    let text = std::fs::read_to_string(&member).expect("read the member manifest");
    std::fs::write(
        &member,
        text.replacen("[package]", "[package] # the repository checks", 1),
    )
    .expect("write");
    development_changelog(&fixture.repo, "0.2.0", true);
    commit(&fixture.repo, "chore: comment the package heading");
    let verdict = judge(&fixture.repo);
    let _ = std::fs::remove_dir_all(&root);
    assert!(
        verdict.is_ok(),
        "a comment after `[package]` is a comment; the table it heads is still the package table, and \
         refusing to judge the release over it is a false refusal. Got: {:?}",
        verdict.err()
    );
}

/// A tab is whitespace to TOML, so an inherit line spelled with one still inherits.
///
/// The repair that routed this reader through `region::toml()` dropped the `trim()` the hand-rolled version
/// had, leaving `line.replace(' ', "")` — which removes `%x20` and not `%x09`, while TOML's `wschar` is both.
/// So `\tversion.workspace = true` stopped matching and its member was refused with *must inherit
/// version.workspace = true*: a false refusal in front of the release gate over a legal, cargo-accepted
/// manifest. The same class and the same direction as the defect the repair had just closed, reintroduced by
/// the repair.
///
/// Two spellings, because the tab can sit on either side of the content and only one of them was reachable
/// from the comment work: the indent, and the gap before a comment that `toml_head` correctly leaves in the
/// head. Of the five manifest readers in this file the other four trim, and this is the only one comparing a
/// whole line — the predicate an omitted whitespace class hurts most.
#[test]
fn an_inherit_line_spelled_with_tabs_still_inherits() {
    for (label, spelling) in [
        ("tab-indent", "\tversion.workspace = true"),
        ("tab-before-comment", "version.workspace = true\t# c"),
    ] {
        let root = scratch(label);
        let fixture = build_fixture(&root, label, "0.2.0");
        workspace_files(&fixture.repo, "0.2.1");
        release_changelog(&fixture.repo, "0.2.1", "0.2.0");
        let member = fixture.repo.join("crates/xuanji/Cargo.toml");
        let text = std::fs::read_to_string(&member).expect("read the member manifest");
        std::fs::write(&member, text.replace("version.workspace = true", spelling)).expect("write");
        commit(&fixture.repo, "chore: spell the inherit line with a tab");
        let verdict = judge(&fixture.repo);
        let _ = std::fs::remove_dir_all(&root);
        assert!(
            verdict.is_ok(),
            "`{label}`: a tab is TOML whitespace, so this member inherits and refusing it is a false \
             refusal. Got: {:?}",
            verdict.err()
        );
    }
}

/// A family pin under a **bare-triple** target table is read like any other dependency.
///
/// `[target.x86_64-unknown-linux-gnu.dependencies]` is a dependency table with a context in front of it, and
/// the context is two bare TOML keys. This was the first of the target forms to be read, back when the cfg
/// siblings were left alone on the reasoning that a quoted cfg expression is the grammar a line-oriented
/// reader is likeliest to be wrong about. None of them are left alone now — the heading arrives as keys, and
/// a selector is one key whatever it spells — so what this row holds is the plainest form of the context,
/// beside the cfg forms its two siblings hold.
///
/// Negative run: before the reader learned the context, this returned `Ok` — the stale pin sat in a table the
/// heading test classified as `Other` and no dependency was read from it at all.
#[test]
fn a_family_pin_under_a_target_triple_is_read() {
    let root = scratch("target-triple");
    let fixture = build_fixture(&root, "target-triple", "0.2.0");
    let manifest = fixture.repo.join("examples/adopter/Cargo.toml");
    let text = std::fs::read_to_string(&manifest).expect("read");
    std::fs::write(
        &manifest,
        format!("{text}\n[target.x86_64-unknown-linux-gnu.dependencies]\nxuanji = \"0.0.1\"\n"),
    )
    .expect("write");
    development_changelog(&fixture.repo, "0.2.0", true);
    commit(
        &fixture.repo,
        "chore: depend on a family crate for one target",
    );
    refusal::expect(
        "release-coherence#example-pin-disagrees",
        &refuse(
            &fixture.repo,
            Kind::Violation,
            "requires xuanji = \"0.0.1\"",
        ),
    );
    let _ = std::fs::remove_dir_all(&root);
}

/// A family pin under a **quoted cfg** target table is read, which is where the bound moved to.
///
/// **This direction's prose used to say the opposite of its own assertion.** It asserted the pin was *not*
/// read, as the declared bound then said. Converging the four heading readers onto one that unquotes each
/// segment closed that: quoting alone hides nothing now, and this direction went red — which was the bound's
/// own WHEN, re-run, and the measurement that narrowed it.
///
/// A bound then remained for a cfg expression carrying a **dot**, and it is retired too: holding the heading
/// as keys leaves the expression one key whatever it contains, so there is no dot for the context step to
/// land inside. `a_pin_under_a_cfg_target_carrying_a_dot_is_read` is what retired it. These three target rows
/// no longer locate where the reader stops; they hold that it does not stop.
///
/// **This prose was itself the leftover, twice.** It said the opposite of its own assertion once, and then
/// described a retired bound as live — in a file the retiring commit had edited, a few lines above a
/// direction asserting the opposite. A review found it. What would have caught it is in `AGENTS.md` now: a
/// retirement's sweep takes the pinning test's own name as a grep seed, because prose that *describes* a
/// bound never mentions its id and no bijection can see it.
#[test]
fn a_family_pin_under_a_quoted_cfg_target_is_observed() {
    let root = scratch("target-cfg");
    let fixture = build_fixture(&root, "target-cfg", "0.2.0");
    let manifest = fixture.repo.join("examples/adopter/Cargo.toml");
    let text = std::fs::read_to_string(&manifest).expect("read");
    std::fs::write(
        &manifest,
        format!("{text}\n[target.'cfg(unix)'.dependencies]\nxuanji = \"0.0.1\"\n"),
    )
    .expect("write");
    development_changelog(&fixture.repo, "0.2.0", true);
    commit(&fixture.repo, "chore: depend on a family crate under a cfg");
    refusal::expect(
        "release-coherence#example-pin-disagrees",
        &refuse(&fixture.repo, Kind::Violation, "requires xuanji"),
    );
    let _ = std::fs::remove_dir_all(&root);
}

/// A family pin under a cfg target whose expression carries a **dot** is read, which retires a declared bound.
///
/// **This is the bound's own WHEN, written into the tree after the change rather than reasoned about.** The
/// bound said such a pin went unobserved because stepping past the target context split the heading at its
/// first dot, landing inside the expression. That was true of a reader holding the heading as one dotted
/// string; it is not true of one holding segments, where the expression is a single key whatever it contains.
/// Both spellings cargo accepts are here, and cargo reads `serde` under each -- measured, the dependency
/// arrives with target `cfg(target_os = "l.x")`.
///
/// Negative run: with the quote-aware cut replaced by `split('.')`, each row returns `Ok` -- measured one at a
/// time, since the loop stops at its first failure -- so the pin goes unread exactly as the bound described,
/// over a manifest requiring `xuanji = "0.0.1"` against workspace `0.2.0`.
#[test]
fn a_pin_under_a_cfg_target_carrying_a_dot_is_read() {
    for (label, table) in [
        (
            "literal-quoted",
            "[target.'cfg(target_os = \"l.x\")'.dependencies]\nxuanji = \"0.0.1\"\n",
        ),
        (
            "basic-quoted",
            "[target.\"cfg(target_os = \\\"l.x\\\")\".dependencies]\nxuanji = \"0.0.1\"\n",
        ),
    ] {
        let root = scratch(&format!("cfg-dot-{label}"));
        let fixture = build_fixture(&root, "cfg-dot", "0.2.0");
        let manifest = fixture.repo.join("examples/adopter/Cargo.toml");
        let text = std::fs::read_to_string(&manifest).expect("read");
        std::fs::write(&manifest, format!("{text}\n{table}")).expect("write");
        development_changelog(&fixture.repo, "0.2.0", true);
        commit(
            &fixture.repo,
            "chore: a pin under a cfg target carrying a dot",
        );
        let verdict = judge(&fixture.repo);
        let _ = std::fs::remove_dir_all(&root);
        let refusal = verdict.expect_err(&format!("{label}: the pin under it must be read"));
        assert_eq!(
            refusal.kind,
            Kind::Violation,
            "{label}: {}",
            refusal.message
        );
        assert!(
            refusal.message.contains("xuanji"),
            "{label}: the refusal must name the crate: {}",
            refusal.message
        );
    }
}

/// A workspace table is not a dependency of the package whose manifest carries it.
///
/// **`[workspace.dependencies]` is a catalog, and a catalog is an offer rather than a requirement.** Measured:
/// a package declaring `[workspace.dependencies] xuanji = "0.5"` beside `[dependencies] serde_json = "1"`
/// reports exactly one dependency to `cargo metadata`, and it is not `xuanji`. A member takes the offer up by
/// writing `xuanji = { workspace = true }`; the table alone makes nobody depend on anything.
///
/// The consumers read one unqualified list from the same reader, and only the root's wanted the catalog. The
/// per-example guard exists to refuse an example that declares **no** family dependency at all -- and a
/// catalog entry counted toward it, so an example could satisfy the guard with a table cargo does not read as
/// a dependency. That is the false negative the guard was written against, arriving through the reader
/// beneath it. The sibling `[workspace.dev-dependencies]` and `[workspace.target.<triple>.dependencies]`
/// carry no meaning at all: measured, a member inheriting from either fails to load.
///
/// The first rows are the false-refusal direction and the last is the false-negative one. The root's own use
/// of the catalog needs no row here: the fixture root pins the family through `[workspace.dependencies]`, so
/// every direction in this file that reaches `judge` is already reading it.
///
/// Negative run, measured per case because a loop stops at its first failure: with the catalog admitted to
/// what an example *requires*, the first row becomes a violation naming `xuanji`, and the last case -- run on
/// its own with the rows emptied -- returns `ok release coherence`, the example counting a requirement it
/// never declared.
#[test]
fn a_workspace_table_is_not_a_dependency_of_the_package_carrying_it() {
    for (label, table) in [
        ("dependencies", "[workspace.dependencies]"),
        ("dev-dependencies", "[workspace.dev-dependencies]"),
        (
            "target",
            "[workspace.target.x86_64-unknown-linux-gnu.dependencies]",
        ),
    ] {
        let root = scratch(&format!("workspace-table-{label}"));
        let fixture = build_fixture(&root, "workspace-table", "0.2.0");
        let manifest = fixture.repo.join("examples/adopter/Cargo.toml");
        let text = std::fs::read_to_string(&manifest).expect("read");
        std::fs::write(&manifest, format!("{text}\n{table}\nxuanji = \"0.0.1\"\n")).expect("write");
        development_changelog(&fixture.repo, "0.2.0", true);
        commit(&fixture.repo, "chore: a workspace table in an example");
        let verdict = judge(&fixture.repo);
        let _ = std::fs::remove_dir_all(&root);
        assert!(
            verdict.is_ok(),
            "{label}: cargo reads no dependency of `adopter` from this table, so a version in it is not a \
             pin of `adopter`: {:?}",
            verdict.err()
        );
    }

    // The other direction: an example whose only family mention is a catalog declares no family dependency.
    let root = scratch("catalog-is-not-a-requirement");
    let fixture = build_fixture(&root, "catalog-is-not-a-requirement", "0.2.0");
    std::fs::create_dir_all(fixture.repo.join("examples/catalogue")).expect("create");
    std::fs::write(
        fixture.repo.join("examples/catalogue/Cargo.toml"),
        "[package]\nname = \"catalogue\"\nversion = \"0.0.0\"\nedition = \"2024\"\n\n\
         [dependencies]\nserde_json = \"1\"\n\n[workspace.dependencies]\nxuanji = \"0.2\"\n",
    )
    .expect("write");
    development_changelog(&fixture.repo, "0.2.0", true);
    commit(
        &fixture.repo,
        "chore: an example offering what it does not require",
    );
    let refusal = refuse(
        &fixture.repo,
        Kind::CannotJudge,
        "example catalogue declares no",
    );
    refusal::expect(
        "release-coherence#example-requires-no-family-crate",
        &refusal,
    );
    let _ = std::fs::remove_dir_all(&root);
}

/// An example accepting the offer in its own catalog is held to the catalog's version, not called versionless.
///
/// **Every example in this repository is its own workspace root** — the root manifest's own comment says so,
/// and `exclude` keeps them out of the main workspace — so `workspace = true` beside
/// `[workspace.dependencies] xuanji = "…"` in one example manifest is a shape cargo resolves. Measured: it
/// resolves the inline, dotted and detailed spellings alike, and resolves to the catalog's requirement even
/// when a local `version` sits in the same inline table, so the catalog is *the* answer rather than one of two.
///
/// The reader read the inline table, found no `version` key in it, and filed `Declared::Absent` — the state
/// meaning *a path-only or git-only dependency that nothing holds to a version*. So an example whose pin is
/// held exactly was refused for having no pin: the false-refusal direction, and outside the spec's absent-pin
/// scenario, which is written for a dependency nothing holds.
///
/// **Every spelling is here because recognising one and missing another is what this file's history is
/// made of.** The same equivalence is what `an_unjudged_dotted_tail_declares_as_its_inline_spelling_does`
/// holds for the keys this reader already judged.
///
/// Negative run: with the offer unrecognised — the pin read from the local `version` key alone — this reports
/// the absent-pin violation *example adopter requires xuanji with no version, so nothing holds it to the
/// workspace version 0.2.0*, which is the arm every held row reaches. Observed on the first case the run
/// reached; a loop stops at its first failure, so the rows after it were not separately measured.
#[test]
fn an_example_inheriting_from_its_own_catalog_is_held_to_the_catalog_version() {
    for (spelling, accepts) in [
        (
            "inline",
            // Single braces: this is a `format!` **argument**, not the format string, so `{{` would reach the
            // manifest doubled. It did, and the row passed anyway while the inline-key scanner matched a raw
            // substring — a fixture that did not plant its own shape, passing because the reader was loose.
            "[dependencies]\nxuanji = { workspace = true }\n",
        ),
        ("dotted", "[dependencies]\nxuanji.workspace = true\n"),
        ("detailed", "[dependencies.xuanji]\nworkspace = true\n"),
    ] {
        for (label, catalog, stale) in [
            ("the workspace version", "0.2.0", false),
            ("the minor series", "0.2", false),
            ("a stale catalog", "0.0.1", true),
        ] {
            let root = scratch(&format!("inherits-{spelling}-{}", label.replace(' ', "-")));
            let fixture = build_fixture(&root, "inherits", "0.2.0");
            std::fs::write(
                fixture.repo.join("examples/adopter/Cargo.toml"),
                format!(
                    "[workspace]\n[package]\nname = \"adopter\"\nversion = \"0.0.0\"\n\
                     edition = \"2024\"\n\n[workspace.dependencies]\nxuanji = \"{catalog}\"\n\n{accepts}"
                ),
            )
            .expect("write");
            development_changelog(&fixture.repo, "0.2.0", true);
            commit(
                &fixture.repo,
                "chore: inherit a family pin from the example's own catalog",
            );
            let verdict = judge(&fixture.repo);
            let _ = std::fs::remove_dir_all(&root);
            if stale {
                let refusal = verdict.expect_err(&format!(
                    "{spelling}/{label}: a stale catalog is still a stale pin"
                ));
                assert_eq!(
                    refusal.kind,
                    Kind::Violation,
                    "{spelling}/{label}: {}",
                    refusal.message
                );
                assert!(
                    refusal.message.contains("xuanji"),
                    "{spelling}/{label}: the refusal must name the crate: {}",
                    refusal.message
                );
            } else {
                assert!(
                    verdict.is_ok(),
                    "{spelling}/{label}: cargo holds this dependency to `{catalog}`, so the pin is neither \
                     absent nor unreadable: {:?}",
                    verdict.err()
                );
            }
        }
    }
}

/// An example inheriting what no catalog beside it offers is not judged.
///
/// Measured: `cargo metadata` refuses a manifest whose dependency takes `workspace = true` while its catalog
/// declares no such crate — *failed to parse manifest* — so this is a file nothing builds, and the reader says
/// which of the two it met rather than reporting a pin it never found.
#[test]
fn an_example_inheriting_what_no_catalog_offers_is_not_judged() {
    let root = scratch("inherits-nothing");
    let fixture = build_fixture(&root, "inherits-nothing", "0.2.0");
    std::fs::write(
        fixture.repo.join("examples/adopter/Cargo.toml"),
        "[workspace]\n[package]\nname = \"adopter\"\nversion = \"0.0.0\"\nedition = \"2024\"\n\n\
         [workspace.dependencies]\nserde_json = \"1\"\n\n[dependencies]\nxuanji = { workspace = true }\n",
    )
    .expect("write");
    development_changelog(&fixture.repo, "0.2.0", true);
    commit(&fixture.repo, "chore: inherit what nothing offers");
    let refusal = refuse(
        &fixture.repo,
        Kind::CannotJudge,
        "requires xuanji from the workspace catalog",
    );
    refusal::expect(
        "release-coherence#example-inherits-what-no-catalog-offers",
        &refusal,
    );
    let _ = std::fs::remove_dir_all(&root);
}

/// An inline field whose key cannot be decoded is not an absent one, and does not read as a clean pin.
///
/// **The undecodable state existed and a `filter_map` erased it.** The inline-table reader answers with the
/// values it could attribute to the key it was asked about, so a field it could not decode simply vanished:
/// measured, `xuanji = { version = "0.2", "\q" = true }` kept the readable `0.2`, matched the workspace
/// minor series, and reported a clean release over a manifest `cargo metadata` refuses to parse — *missing
/// escaped value*, measured under cargo 1.96.0.
///
/// The examples check runs `cargo metadata` per example and would fail on that file in the same run, which
/// bounds the damage; a compensating control in another gate is not this gate answering, and the Core
/// Contract's one forbidden bug is a real violation that silently passes. The dotted spelling already reported
/// the fields the line could have carried as unreadable.
///
/// **One record has three producers, and the state was wired into them one at a time.** An inline table's
/// fields, a dotted key's tail and a detailed table's body all build the same `Detailed`; each was closed in
/// its own round, the last of them after a review went looking for the sibling rather than the site. The
/// detailed body was the worst of the three, because it scanned the line once per watched key — so an
/// undecodable key was filtered out four times over, leaving a readable identity beside a readable pin. A row
/// below reaches each producer.
///
/// **The outer key is deliberately *not* a family crate, which is the shape the first version of this
/// direction could not see.** That version used `xuanji` as the key, so identity classification succeeded and
/// the unreadable pin was reached — masking the path where it is not reached at all: the first repair left
/// `package` falling back to the key, so a non-family alias was skipped as *not a family dependency* **before**
/// the pin was read, and the fixture's own correct pin satisfied the per-example counter. A review found both
/// the defect and the masking.
///
/// Negative run: with the undecodable field ignored, this returns `Ok`; with only the version and path made
/// unreadable and identity left to fall back, it returns `Ok` too, which is the row this one adds.
#[test]
fn an_inline_field_that_cannot_be_decoded_is_not_a_clean_pin() {
    for (label, entry) in [
        (
            "inline",
            "alias = { version = \"0.2\", \"\\q\" = \"xuanji\" }",
        ),
        ("dotted", "alias.\"\\q\" = \"xuanji\""),
        // The third producer of one record: a detailed table's body, which scanned the line once per
        // watched key and so filtered an undecodable one out four times over.
        (
            "detailed",
            "[dependencies.alias]\npackage = \"xuanji\"\nversion = \"0.2\"\n\"\\q\" = true",
        ),
        // Structure beneath a value this reader judges: cargo answers *cannot extend value of type string
        // with a dotted key*, and discarding it as unrelated kept the readable pin.
        (
            "detailed dotted subfield",
            "[dependencies.alias]\npackage = \"xuanji\"\nversion = \"0.2\"\nversion.extra = true",
        ),
    ] {
        let root = scratch(&format!("undecodable-field-{label}"));
        let fixture = build_fixture(&root, "undecodable-field", "0.2.0");
        let manifest = fixture.repo.join("examples/adopter/Cargo.toml");
        let text = std::fs::read_to_string(&manifest).expect("read");
        std::fs::write(&manifest, format!("{text}{entry}\n")).expect("write");
        development_changelog(&fixture.repo, "0.2.0", true);
        commit(&fixture.repo, "chore: a field this reader cannot decode");
        let verdict = judge(&fixture.repo);
        let _ = std::fs::remove_dir_all(&root);
        let refusal = verdict.expect_err(&format!(
            "{label}: a manifest cargo will not load is not a clean pin"
        ));
        assert_eq!(
            refusal.kind,
            Kind::CannotJudge,
            "{label}: an undecodable key is not a disagreement, it is an unread one: {}",
            refusal.message
        );
        // **The site, not only the class.** Asserting the class alone let the first version of this pass
        // while the gate said *a `package` value this check cannot read* — about a dependency declaring no
        // `package` key at all, sending an operator to look for a key that is not there. A review read the
        // emitted diagnostic rather than the exit class and found it; every sibling direction here asserts
        // the site.
        refusal::expect("release-coherence#manifest-unparseable", &refusal);
        assert!(
            refusal
                .message
                .contains("a manifest this parser cannot read"),
            "{label}: the refusal names what it could not read: {}",
            refusal.message
        );
    }
}

/// Two sections claiming one version are not judged, and the first of them does not answer.
///
/// **The fourth reader in this crate to be asked *how many*, and the first that selects from a document.**
/// The dated section was taken with `.find()`, so a changelog carrying a stale `## [0.2.1]` dated years
/// earlier ahead of the correct one reported *ok release coherence* — measured end to end. Two sections
/// carrying the *same* date passed too. At the snapshot the same selection inverts: the stale date would be
/// compared against the release commit and the refusal would name the wrong line.
///
/// The `[Unreleased]` reader counts its sections and refuses any count but one, with a comment saying its
/// every arm assumes exactly one exists. The same assumption was made here and never checked, and nothing
/// declared it — not the spec, which was written in the singular throughout, not the observation bounds, not
/// the backlog. The release-coherence spec now carries *Two sections claim one version*, which is what these
/// rows pin.
///
/// **The count is over the sections that claim the version, not over the ones whose suffix parsed.** The
/// malformed rows are the reason: a heading left behind or a typo'd date is the likelier sibling, and
/// counting after the date filter left one survivor and reported clean on both of them.
///
/// Negative run, both halves: with the count removed and the section taken by `.find()`, every row reports
/// clean; with the count kept but taken *after* the date filter, the two well-formed rows still refuse and
/// the two malformed ones report clean.
#[test]
fn two_dated_sections_for_one_version_are_not_judged() {
    for (label, stale) in [
        ("an older date first", "## [0.2.1] - 2020-01-01"),
        ("the same date twice", "## [0.2.1] - 2026-08-28"),
        // **The sibling need not parse as a date.** A first version of this counted the candidates that had
        // already passed the date test, so a malformed heading left one survivor and reported clean — and a
        // heading left behind, or a typo'd date, is the likelier mistake than a second well-formed one.
        ("a suffix that is not a date", "## [0.2.1] - notadate!!"),
        ("no suffix at all", "## [0.2.1]"),
    ] {
        let root = scratch(&format!("two-dated-{}", label.replace(' ', "-")));
        let fixture = build_fixture(&root, "two-dated", "0.2.0");
        workspace_files(&fixture.repo, "0.2.1");
        release_changelog(&fixture.repo, "0.2.1", "0.2.0");
        let path = fixture.repo.join("CHANGELOG.md");
        let text = std::fs::read_to_string(&path).expect("read");
        let heading = text
            .lines()
            .find(|line| line.starts_with("## [0.2.1] - "))
            .expect("the release changelog carries a dated section")
            .to_string();
        // The stale section goes **before** the correct one, which is what makes `.find()` answer from it.
        std::fs::write(
            &path,
            text.replacen(
                &heading,
                &format!("{stale}\n\n- An adopter-facing change.\n\n{heading}"),
                1,
            ),
        )
        .expect("write");
        commit(&fixture.repo, "chore: two dated sections for one version");
        let refusal = refuse(&fixture.repo, Kind::CannotJudge, "sections for 0.2.1");
        refusal::expect("release-coherence#several-release-sections", &refusal);
        let _ = std::fs::remove_dir_all(&root);
    }
}

/// A `[[package]]` block writing `version` before `name` records the version all the same.
///
/// **An ordering premise nothing stated.** The version arm was guarded on a name already read, so a block
/// written the other way round dropped the version, recorded nothing, and reached *Cargo.lock is missing
/// workspace package xuanji* — exit 1, about a lock that records it two lines apart. Cargo writes `name`
/// first, so no lock it writes fires this; nothing said so, in a comment, a spec clause, an observation bound
/// or the backlog entry covering this reader, which addressed its key decoding and not key order.
///
/// The guard is gone and the block-level test decides, which is safe against the shape the guard appeared to
/// defend: a top-level `version = 4` sits outside every block and is dropped by the block filter. This
/// fixture carries that line, which is what makes the row evidence rather than an assertion.
///
/// Negative run: with the guard restored, this is that violation.
#[test]
fn a_lock_block_writing_version_before_name_still_records_it() {
    let root = scratch("lock-order");
    let fixture = build_fixture(&root, "lock-order", "0.2.0");
    // Release-ready, because the lock reader runs only in that phase and in the snapshot — a development
    // fixture never reaches it, which is what a first version of this direction asserted `Ok` past.
    workspace_files(&fixture.repo, "0.2.1");
    release_changelog(&fixture.repo, "0.2.1", "0.2.0");
    let lock = fixture.repo.join("Cargo.lock");
    let text = std::fs::read_to_string(&lock).expect("read");
    let target = "[[package]]\nname = \"xuanji\"\nversion = \"0.2.1\"";
    // **The mutation is asserted before it is written.** `str::replace` over an absent target is a no-op, so
    // a fixture whose format drifts leaves the coherent lock in place and this direction passes having
    // perturbed nothing — the shape a review named after the 0.5.0 preparation produced two of them.
    assert_eq!(
        text.matches(target).count(),
        1,
        "the fixture lock must carry exactly one xuanji block in the order this direction reverses"
    );
    std::fs::write(
        &lock,
        text.replace(
            target,
            "[[package]]\nversion = \"0.2.1\"\nname = \"xuanji\"",
        ),
    )
    .expect("write");
    commit(
        &fixture.repo,
        "chore: a lock block written the other way round",
    );
    let verdict = judge(&fixture.repo);
    let _ = std::fs::remove_dir_all(&root);
    assert!(
        verdict.is_ok(),
        "the lock records this member's version; which of two keys a block writes first is not a fact \
         about it: {:?}",
        verdict.err()
    );
}

/// A dependency key whose **name** carries a dot is one key, and cargo builds it.
///
/// **The row that tells the two readings apart.** `xuanji."version.extra" = true` is a single key literally
/// named `version.extra`; `xuanji.version.extra = true` is two keys, structure beneath a string. Measured
/// under cargo 1.96.0: the first builds, the second is refused as *cannot extend value of type string with a
/// dotted key*. A reader that joins the decoded segments with dots and splits them back apart answers the
/// same for both, and refused the manifest cargo accepts — the same collapse the table-heading reader keeps
/// segments to avoid, on the side that had joined them.
///
/// Negative run: with the tail joined and re-split, this is a cannot-judge naming a field it cannot decode.
#[test]
fn a_dependency_key_whose_name_carries_a_dot_is_one_key() {
    let root = scratch("dotted-name");
    let fixture = build_fixture(&root, "dotted-name", "0.2.0");
    let manifest = fixture.repo.join("examples/adopter/Cargo.toml");
    let text = std::fs::read_to_string(&manifest).expect("read");
    std::fs::write(
        &manifest,
        // **The dotted form replaces the example's own entry rather than being appended beside it.** Appended,
        // `xuanji` is already a string and `xuanji.version` cannot extend it — a document cargo refuses too,
        // so the fixture stopped being about the thing it names. The hand-rolled reader never noticed because
        // it read lines rather than a document.
        text.replace(
            "xuanji = \"0.2\"",
            "xuanji.version = \"0.2\"\nxuanji.\"version.extra\" = true",
        ),
    )
    .expect("write");
    development_changelog(&fixture.repo, "0.2.0", true);
    commit(
        &fixture.repo,
        "chore: a dependency key whose name carries a dot",
    );
    let verdict = judge(&fixture.repo);
    let _ = std::fs::remove_dir_all(&root);
    assert!(
        verdict.is_ok(),
        "cargo builds this manifest — the key is named `version.extra`, not structure beneath `version`: {:?}",
        verdict.err()
    );
}

/// A family crate the catalog offers without a `path` is a violation, where it used to pass the gate.
///
/// **The subject was *where a dependency points*, and the defect is a family crate that points nowhere
/// local.** Measured under cargo 1.96.0 on a synthetic workspace: a catalog entry `xuanji = "0.4.0"` beside a
/// local member `xuanji 0.9.0` gives the inheriting member `registry+…#xuanji@0.4.0` — the registry crate,
/// with the member sitting unused — and `cargo package` on a `git` dependency carrying a `version` drops the
/// source and records the version alone. Either way the published requirement is that version, and deleting
/// one `path = …` is the whole of the edit that gets there. Measured before this repair: the same fixture
/// answered *ok release coherence*.
///
/// The correct sibling stays, so the vacuity floor cannot be what refuses: one path dependency satisfies the
/// count while the pathless one is dropped from the subject in silence.
///
/// Negative run: with the subject selected by `path` again, this returns `Ok`.
#[test]
fn a_family_crate_offered_with_no_path_is_a_violation() {
    let root = scratch("pathless-family");
    let fixture = build_fixture(&root, "pathless-family", "0.2.0");
    let manifest = fixture.repo.join("Cargo.toml");
    let text = std::fs::read_to_string(&manifest).expect("read");
    std::fs::write(
        &manifest,
        text.replace(
            "xuanji = { path = \"crates/xuanji\", version = \"0.2.0\" }",
            "xuanji = { path = \"crates/xuanji\", version = \"0.2.0\" }\ntianheng = \"0.5\"",
        ),
    )
    .expect("write");
    development_changelog(&fixture.repo, "0.2.0", true);
    commit(&fixture.repo, "chore: offer a family crate with no path");
    let verdict = judge(&fixture.repo);
    let _ = std::fs::remove_dir_all(&root);
    let refusal = verdict.expect_err(
        "members inheriting this entry build against the registry crate, and the requirement it publishes is \
         whatever it says",
    );
    refusal::expect("release-coherence#internal-path-absent", &refusal);
    assert_eq!(refusal.kind, Kind::Violation, "{}", refusal.message);
    assert!(
        refusal.message.contains("tianheng"),
        "the refusal names the crate it could not hold: {}",
        refusal.message
    );
}

/// A family crate's path is compared against the member's own directory, spelling by spelling.
///
/// **A prefix test decided this, and it was wrong in both directions at once.** `starts_with("crates/")`
/// refused `./crates/xuanji` — which cargo resolves to the member, measured: `path=…/crates/xuanji`,
/// `source=None` — and accepted `crates/../vendor/xuanji`, which cargo resolves to `vendor/xuanji`, outside
/// the workspace. One review found each. The question is not whether the text begins with `crates/` but
/// whether it names the directory this member lives in, and the member is what answers it — a directory that
/// cannot be derived from the package name, since this fixture keeps `machinery-under-another-name` under
/// `crates/renamed-dir`.
///
/// **What each spelling earns.** A spelling of the member's own directory is clean; a path naming some other
/// directory is a violation, whether it leaves the workspace or lands on a different member; a path this
/// reader will not name a directory for is a cannot-judge that says **which** reason it met, rather than a
/// guess or a sentence enumerating the other reasons.
///
/// Negative run, measured rather than described: with the prefix test deciding first, the row that offers
/// `crates/xuanji` for `tianheng` — another member's directory — reports `ok release coherence`. That is the
/// row no prefix could ever have answered, and it is the first one the perturbation reaches.
#[test]
fn a_family_crate_path_is_compared_against_the_members_own_directory() {
    enum Answer {
        Clean,
        Violation,
        /// The reason the refusal must name. **Asserted, because the message was the defect**: three causes
        /// once reached one arm that enumerated two of them, and `.` was told it was absolute or carried a
        /// `..`. A direction checking only the site cannot see that.
        CannotJudge(&'static str),
    }
    for (label, spelling, answer) in [
        ("bare", "crates/tianheng", Answer::Clean),
        ("a leading dot-slash", "./crates/tianheng", Answer::Clean),
        ("a doubled separator", "crates//tianheng", Answer::Clean),
        ("a trailing separator", "crates/tianheng/", Answer::Clean),
        (
            "an interior dot segment",
            "crates/./tianheng",
            Answer::Clean,
        ),
        (
            "outside the workspace",
            "vendor/tianheng",
            Answer::Violation,
        ),
        // The directory of a real member of this fixture, under another crate's name.
        (
            "another member's directory",
            "crates/xuanji",
            Answer::Violation,
        ),
        (
            "a dot-dot leaving the crates directory",
            "crates/../vendor/tianheng",
            Answer::CannotJudge("carries a `..` segment"),
        ),
        (
            "a leading dot-dot",
            "../crates/tianheng",
            Answer::CannotJudge("carries a `..` segment"),
        ),
        (
            "absolute",
            "/opt/crates/tianheng",
            Answer::CannotJudge("is absolute"),
        ),
        // The third cause. Measured under cargo 1.96.0 with dependency resolution on: `path = "."` fails
        // with *failed to get `xuanji` as a dependency*, so it is a manifest nothing builds — and it is
        // neither absolute nor a traversal, which is what the one-message arm used to tell an operator.
        (
            "the manifest's own directory",
            ".",
            Answer::CannotJudge("names no directory"),
        ),
        (
            "the manifest's own directory with a separator",
            "./",
            Answer::CannotJudge("names no directory"),
        ),
        ("empty", "", Answer::CannotJudge("names no directory")),
    ] {
        let root = scratch(&format!("member-dir-{}", label.replace(' ', "-")));
        let fixture = build_fixture(&root, "member-dir", "0.2.0");
        let manifest = fixture.repo.join("Cargo.toml");
        let text = std::fs::read_to_string(&manifest).expect("read");
        std::fs::write(
            &manifest,
            text.replace(
                "xuanji = { path = \"crates/xuanji\", version = \"0.2.0\" }",
                &format!(
                    "xuanji = {{ path = \"crates/xuanji\", version = \"0.2.0\" }}\ntianheng = {{ path = \"{spelling}\", version = \"0.2.0\" }}"
                ),
            ),
        )
        .expect("write");
        development_changelog(&fixture.repo, "0.2.0", true);
        commit(&fixture.repo, "chore: spell a member path");
        let verdict = judge(&fixture.repo);
        let _ = std::fs::remove_dir_all(&root);
        match answer {
            Answer::Clean => assert!(
                verdict.is_ok(),
                "{label}: cargo resolves {spelling} to this workspace's tianheng: {:?}",
                verdict.err()
            ),
            Answer::Violation => {
                let refusal = verdict.expect_err(&format!(
                    "{label}: {spelling} is not the member's directory"
                ));
                refusal::expect(
                    "release-coherence#internal-path-names-another-directory",
                    &refusal,
                );
                assert_eq!(
                    refusal.kind,
                    Kind::Violation,
                    "{label}: {}",
                    refusal.message
                );
                assert!(
                    refusal.message.contains("crates/tianheng"),
                    "{label}: the refusal names where the member actually is: {}",
                    refusal.message
                );
            }
            Answer::CannotJudge(reason) => {
                let refusal = verdict.expect_err(&format!(
                    "{label}: {spelling} is not this reader's to resolve"
                ));
                refusal::expect("release-coherence#internal-path-unresolvable", &refusal);
                assert_eq!(
                    refusal.kind,
                    Kind::CannotJudge,
                    "{label}: {}",
                    refusal.message
                );
                assert!(
                    refusal.message.contains(reason),
                    "{label}: the refusal names the cause it met rather than a sibling's: {}",
                    refusal.message
                );
            }
        }
    }
}

/// A case alias of a member's directory is refused — a stated bound.
///
/// **The answer is the same on every host, and only right where the filesystem is case-sensitive.** On this
/// repository's CI `CRATES/TIANHENG` names a directory that does not exist, so the violation is correct. On a
/// case-insensitive volume cargo resolves it to the member and the same violation is an over-reaction. This
/// direction observes the answer; the bound declares what that answer costs elsewhere.
///
/// **Kept rather than closed, and the reason is what the bound records.** Directory identity is the volume's
/// rule, not the string's, so deciding it needs the filesystem — and this reader is handed no repository.
/// Canonicalizing to obtain one would make `..` resolvable too, turning a refusal that stops in front of an
/// operator into an answer and moving three other verdicts with it.
///
/// Negative run: with the comparison made case-insensitively, this reports `ok release coherence` — which is
/// the answer a case-insensitive host wants and the wrong one here.
#[test]
fn a_case_alias_of_a_member_directory_is_a_stated_bound() {
    let root = scratch("case-alias");
    let fixture = build_fixture(&root, "case-alias", "0.2.0");
    let manifest = fixture.repo.join("Cargo.toml");
    let text = std::fs::read_to_string(&manifest).expect("read");
    std::fs::write(
        &manifest,
        text.replace(
            "xuanji = { path = \"crates/xuanji\", version = \"0.2.0\" }",
            "xuanji = { path = \"crates/xuanji\", version = \"0.2.0\" }\ntianheng = { path = \"CRATES/TIANHENG\", version = \"0.2.0\" }",
        ),
    )
    .expect("write");
    development_changelog(&fixture.repo, "0.2.0", true);
    commit(&fixture.repo, "chore: offer a member through a case alias");
    let verdict = judge(&fixture.repo);
    let _ = std::fs::remove_dir_all(&root);
    let refusal = verdict.expect_err(
        "on a case-sensitive filesystem CRATES/TIANHENG is not a directory this workspace has",
    );
    refusal::expect(
        "release-coherence#internal-path-names-another-directory",
        &refusal,
    );
    assert_eq!(refusal.kind, Kind::Violation, "{}", refusal.message);
    assert!(
        refusal.message.contains("CRATES/TIANHENG") && refusal.message.contains("crates/tianheng"),
        "the refusal names what was written and where the member is: {}",
        refusal.message
    );
}

/// Editing at a release snapshot is development, not a snapshot with a dirty tree.
///
/// **The state was read from the commit while everything else read the worktree.** `release_spine` decided
/// `Snapshot` on `head == release_commit` alone; every other reader takes its content through
/// `std::fs::read_to_string`. The first change of a new cycle falls between those two sources: sitting on the
/// release commit, the author writes the `[Unreleased]` entry that `Development` **requires**, and it is
/// judged in `Snapshot`, where `[Unreleased]` must be **empty**. Two rules, both real, and no tree satisfies
/// them at once — the author's only way out is to commit, which is what moves `head`.
///
/// Measured on this repository: `release/0.5.1`'s first change could not pass the Definition of Done until it
/// was committed, and passed immediately afterwards with nothing else altered.
///
/// A release snapshot is an unmodified **checkout** of one. This writes the entry without committing and
/// requires the answer an author can act on.
///
/// Negative run: with the state read from the commit alone, this refuses with *[Unreleased] must be empty in
/// snapshot state*.
#[test]
fn editing_at_a_release_snapshot_is_development() {
    let root = scratch("snapshot-edited");
    let fixture = build_fixture(&root, "snapshot-edited", "0.2.0");
    release_changelog(&fixture.repo, "0.2.0", "0.1.0");
    // The release commit has to carry a change, as its sibling direction records; what it carries is beside
    // the point.
    std::fs::write(fixture.repo.join("NOTES.md"), "prepared\n").expect("write");
    commit(&fixture.repo, "release: 0.2.0");
    // The next cycle's first edit, uncommitted — `head` is still the release commit.
    development_changelog(&fixture.repo, "0.2.0", true);
    let verdict = judge(&fixture.repo);
    let _ = std::fs::remove_dir_all(&root);
    let ok =
        verdict.expect("an edited checkout of a release is the next cycle, not a dirty snapshot");
    assert!(
        ok.contains("development: 0.2.0"),
        "the state follows the tree the gate reads, not the commit it sits on: {ok}"
    );
}

/// A crate whose `[package]` name is a single-quoted string is read, not refused.
///
/// **The row a negative run demanded.** Migrating `package_name` to a real parser made this readable, and
/// restoring the old double-quote-only rule broke nothing in the corpus — so the improvement was unguarded
/// and could have been reverted in silence. Two directions had asserted the refusal; both had their WHEN
/// moved to a `name` that is no string at all, which leaves nobody observing the new answer.
///
/// Measured under cargo 1.96.0: a single-quoted `name` is legal TOML that cargo resolves. Refusing it made
/// `require_example_pins` answer *cannot judge* over a manifest cargo builds.
///
/// Negative run: with the double-quote-only rule restored, this row refuses instead of judging the pin.
#[test]
fn a_single_quoted_package_name_is_read() {
    let root = scratch("single-quoted-name");
    let fixture = build_fixture(&root, "single-quoted-name", "0.2.0");
    // The member's own manifest, spelled the legal way this reader used to decline. A stale example pin sits
    // beside it, so the verdict is about that pin being judged at all rather than about reading nothing.
    std::fs::write(
        fixture.repo.join("crates/xuanji/Cargo.toml"),
        "[package]NLname = QQxuanjiQQNLversion.workspace = trueNLedition = QQ2024QQNL"
            .replace("NL", "\n")
            .replace("QQ", "'"),
    )
    .expect("write");
    let example = fixture.repo.join("examples/adopter/Cargo.toml");
    let text = std::fs::read_to_string(&example).expect("read");
    std::fs::write(&example, {
        let staled = text.replace("xuanji = \"0.2\"", "xuanji = \"0.0.1\"");
        assert_ne!(staled, text, "the example's pin must actually be staled");
        staled
    })
    .expect("write");
    development_changelog(&fixture.repo, "0.2.0", true);
    commit(
        &fixture.repo,
        "chore: name the package with a single-quoted string",
    );
    let verdict = judge(&fixture.repo);
    let _ = std::fs::remove_dir_all(&root);
    let refusal = verdict.expect_err("the stale example pin is judged, so the name was read");
    assert_eq!(refusal.kind, Kind::Violation, "{}", refusal.message);
    assert!(
        refusal.message.contains("xuanji"),
        "the crate was named, so its pin could be judged: {}",
        refusal.message
    );
}

/// A stale internal pin behind a quoted tail is refused, where it used to pass the gate.
///
/// **The first false negative found in three rounds of review, and the one that mattered most.** The
/// dependency reader split its key on the first *raw* dot, so `xuanji."path" = "xuanji"` read as a dependency
/// with **no path** — and `require_internal_pins` treats a dependency with no path as external and skips it.
/// Measured under cargo 1.96.0: that manifest is a path dependency with requirement `^0.5`, so a non-exact
/// internal pin reached the release gate and passed. The aggregate `pins == 0` floor cannot catch it — one
/// bare pin elsewhere satisfies the count, which is the aggregate-counter defect the sibling example check
/// records having fixed.
///
/// It stands in front of `cargo publish`, where a version is yankable and never replaceable, which is why the
/// review that found it said to act on this row before any other.
///
/// Negative run: with the raw dot split restored, this returns `Ok` — a clean release over a root pinning an
/// internal dependency at `0.5` while the workspace is at `0.2.0`.
#[test]
fn a_stale_internal_pin_behind_a_quoted_tail_is_refused() {
    let root = scratch("quoted-tail-pin");
    let fixture = build_fixture(&root, "quoted-tail-pin", "0.2.0");
    let manifest = fixture.repo.join("Cargo.toml");
    let text = std::fs::read_to_string(&manifest).expect("read");
    std::fs::write(
        &manifest,
        // **The correct pin stays, which is what makes this the false negative rather than the vacuity
        // floor.** `require_internal_pins` refuses when it finds no internal path dependency at all; one
        // correct pin satisfies that count, and the stale one behind the quoted tail is then dropped from its
        // subject in silence. That is the aggregate-counter shape the sibling example check records fixing.
        text.replace(
            "xuanji = { path = \"crates/xuanji\", version = \"0.2.0\" }",
            "xuanji = { path = \"crates/xuanji\", version = \"0.2.0\" }\n\
             tianheng.\"path\" = \"crates/tianheng\"\ntianheng.version = \"0.5\"",
        ),
    )
    .expect("write");
    development_changelog(&fixture.repo, "0.2.0", true);
    commit(
        &fixture.repo,
        "chore: pin an internal dependency behind a quoted tail",
    );
    let verdict = judge(&fixture.repo);
    let _ = std::fs::remove_dir_all(&root);
    let refusal =
        verdict.expect_err("a path dependency pinned at 0.5 against a workspace at 0.2.0 is stale");
    assert!(
        refusal.message.contains("tianheng"),
        "the refusal must name the dependency: {}",
        refusal.message
    );
}

/// Every spelling of the inherit line that cargo honours is read as inheriting.
///
/// **Measured under cargo 1.96.0, each resolving the member at `0.5.0`:** `version.workspace = true`,
/// `version = { workspace = true }`, `\"version\".workspace = true` and `'version'.workspace = true`. The
/// recogniser was whitespace-stripped string equality against the first, so the other three answered *does not
/// inherit* — a `violation_at`, exit 1, over manifests cargo reads. A false refusal is a defect, and the
/// narrowness was declared nowhere: not the spec, not
/// `docs/observation-bounds.md`, not the doc above the call.
///
/// It asks the shared key reader. A dotted head naming `version` reports its **tail**, so this asks whether
/// the field is `workspace`; the inline form is a `version` whose value carries the offer. Both comparisons are
/// of **decoded names**, which they were not when this direction was first written: the tail was joined raw, so
/// `version."workspace" = true` and its literal and escaped siblings were still refused, and a review measured
/// all four. The rows below are what that costs — every spelling cargo honours, enumerated once, against a
/// reader that no longer compares spellings at all.
///
/// Negative run, measured a row at a time: with the string equality restored the inline-table row is
/// *workspace package xuanji must inherit version.workspace = true*, and with the tail joined raw the
/// `quoted tail` row is the same refusal.
#[test]
fn every_inherit_spelling_cargo_honours_is_read_as_inheriting() {
    for (label, line) in [
        ("dotted", "version.workspace = true"),
        ("inline table", "version = { workspace = true }"),
        ("quoted key", "\"version\".workspace = true"),
        ("literal key", "'version'.workspace = true"),
        // The tail, which the head-only decode left raw for a round.
        ("quoted tail", "version.\"workspace\" = true"),
        ("literal tail", "version.'workspace' = true"),
        ("escaped tail", "version.\"\\u0077orkspace\" = true"),
        ("quoted inner key", "version = { \"workspace\" = true }"),
    ] {
        let root = scratch(&format!("inherit-{}", label.replace(' ', "-")));
        let fixture = build_fixture(&root, "inherit", "0.2.0");
        let manifest = fixture.repo.join("crates/xuanji/Cargo.toml");
        let text = std::fs::read_to_string(&manifest).expect("read");
        std::fs::write(&manifest, text.replace("version.workspace = true", line)).expect("write");
        development_changelog(&fixture.repo, "0.2.0", true);
        commit(&fixture.repo, "chore: spell the inherit line another way");
        let verdict = judge(&fixture.repo);
        let _ = std::fs::remove_dir_all(&root);
        assert!(
            verdict.is_ok(),
            "{label}: cargo resolves this member at the workspace version through this spelling: {:?}",
            verdict.err()
        );
    }
}

/// A member whose `[package]` name is spelled in quotes is read under that name, not the directory's.
///
/// **Measured: `[package]` with `\"name\" = \"xuanji\"` names `xuanji` to `cargo metadata`.** The reader matched
/// the key's raw text, so it answered `Absent`, and that state has two consumers. One says *declares no
/// `[package]` name*, about a manifest that declares one — the loud one, and what this direction observes. The
/// other falls back to the **directory** the manifest sits in, so a member is compared under an identity its
/// manifest never gave; that one is silent, which is why the reader was repaired rather than the message.
///
/// Reached through the whole gate rather than by widening the reader's visibility, which is the rule this
/// file's sibling states for `require_internal_pins`.
///
/// Negative run: with the raw-text key match restored, this is a cannot-judge — *crates/xuanji/Cargo.toml
/// declares no `[package]` name, so whether an example pins it cannot be decided*.
#[test]
fn a_member_whose_package_name_is_quoted_is_read_under_that_name() {
    let root = scratch("quoted-package-name");
    let fixture = build_fixture(&root, "quoted-package-name", "0.2.0");
    let manifest = fixture.repo.join("crates/xuanji/Cargo.toml");
    let text = std::fs::read_to_string(&manifest).expect("read");
    std::fs::write(
        &manifest,
        text.replace("name = \"xuanji\"", "\"name\" = \"xuanji\""),
    )
    .expect("write");
    development_changelog(&fixture.repo, "0.2.0", true);
    commit(&fixture.repo, "chore: spell a package name in quotes");
    let verdict = judge(&fixture.repo);
    let _ = std::fs::remove_dir_all(&root);
    assert!(
        verdict.is_ok(),
        "cargo names this package `xuanji`; a quoted key is not an absent one: {:?}",
        verdict.err()
    );
}

/// Assignment-shaped text inside a quoted value is string data, not a table key.
///
/// **The scanner read key boundaries and not lexical state.** It required the byte before the key to be a
/// delimiter — `{`, `,`, space, tab — which a string containing `, workspace = true` supplies exactly as an
/// inline table does. Measured: cargo reads such a manifest without complaint, resolving the dependency at
/// its declared version with a path whose directory name happens to carry that text. The gate answered *a
/// version this check cannot read*: a false refusal, and one whose sentence points an operator at a version
/// that is perfectly readable.
///
/// The blindness was in the shared scanner, so every key it reads had it — `version` and `path` since long
/// before `workspace` joined them. Both rows are here for that reason: one plants the shape in a path and
/// looks for the offer, the other plants a second `version` and looks for *several*.
///
/// Negative run: with the scanner matching a key anywhere its neighbouring byte allows, the first row is a
/// cannot-judge — *requires xuanji with a version this check cannot read* — and the second is *declares 2
/// `version` keys*. Measured a row at a time.
#[test]
fn assignment_shaped_text_inside_a_value_is_not_a_key() {
    for (label, entry) in [
        (
            "an offer inside a path",
            "xuanji = { path = \"deps, workspace = true\", version = \"0.2.0\" }",
        ),
        (
            "a version inside a path",
            "xuanji = { path = \"deps, version = 9\", version = \"0.2.0\" }",
        ),
    ] {
        let root = scratch(&format!("value-not-a-key-{}", label.replace(' ', "-")));
        let fixture = build_fixture(&root, "value-not-a-key", "0.2.0");
        let manifest = fixture.repo.join("examples/adopter/Cargo.toml");
        let text = std::fs::read_to_string(&manifest).expect("read");
        // **Replaces the example's own entry rather than being appended beside it.** Appended, the manifest
        // declares `xuanji` twice and cargo refuses it — so the fixture stopped being about the thing it
        // names. The hand-rolled reader never noticed, because it read lines rather than a document.
        let composed = text.replace("xuanji = \"0.2\"", entry);
        assert_ne!(
            composed, text,
            "{label}: the entry must replace the example's own"
        );
        std::fs::write(&manifest, composed).expect("write");
        development_changelog(&fixture.repo, "0.2.0", true);
        commit(
            &fixture.repo,
            "chore: assignment-shaped text inside a value",
        );
        let verdict = judge(&fixture.repo);
        let _ = std::fs::remove_dir_all(&root);
        assert!(
            verdict.is_ok(),
            "{label}: cargo reads this dependency at `0.2.0`; the text inside the quotes is a directory \
             name, not a table key: {:?}",
            verdict.err()
        );
    }
}

/// Two `workspace` keys in one dependency are malformed, not emphatic.
///
/// The predicate read the values and not how many there were, so `{ workspace = true, workspace = true }`
/// answered *inherits* — duplicate keys, which TOML itself rejects and cargo refuses to parse, resolving to a
/// clean release through a catalog entry that happened to match. A review found it. `version` and `path` each
/// have a state for several declarations; this one had none, and now the cardinality is read before the value.
///
/// Negative run: with the count discarded — `all` over the values, as it was — this returns `Ok`, the pin
/// resolved from the catalog as though one key had been written.
#[test]
fn two_workspace_keys_in_one_dependency_are_not_one_inheritance() {
    let root = scratch("two-offers");
    let fixture = build_fixture(&root, "two-offers", "0.2.0");
    std::fs::write(
        fixture.repo.join("examples/adopter/Cargo.toml"),
        "[workspace]\n[package]\nname = \"adopter\"\nversion = \"0.0.0\"\nedition = \"2024\"\n\n\
         [workspace.dependencies]\nxuanji = \"0.2\"\n\n\
         [dependencies]\nxuanji = { workspace = true, workspace = true }\n",
    )
    .expect("write");
    development_changelog(&fixture.repo, "0.2.0", true);
    commit(&fixture.repo, "chore: two workspace keys in one dependency");
    let refusal = refuse(&fixture.repo, Kind::CannotJudge, "duplicate key");
    refusal::expect("release-coherence#manifest-unparseable", &refusal);
    let _ = std::fs::remove_dir_all(&root);
}

/// A catalog entry whose identity this reader cannot resolve stops the inheriting example.
///
/// The entry might be the one being inherited, and *might be* is not an answer — skipping it is how a stale
/// pin would reach a release through the catalog rather than through the dependency. A quoted key is the
/// cheapest spelling of an unresolvable identity, and it is the same one the sibling
/// `a_dependency_key_this_reader_cannot_decode_is_refused_rather_than_skipped` refuses one level out.
#[test]
fn a_catalog_entry_whose_identity_is_unresolvable_stops_the_inheriting_example() {
    let root = scratch("catalog-unresolvable");
    let fixture = build_fixture(&root, "catalog-unresolvable", "0.2.0");
    std::fs::write(
        fixture.repo.join("examples/adopter/Cargo.toml"),
        // **This WHEN moved when a real parser replaced the hand-rolled reader.** The catalog entry was
        // written under a quoted key, `"xuanji" = "0.2"`, which the old reader could not decode and the
        // parser resolves. What still cannot be resolved is an entry whose `package` is no string.
        "[workspace]\n[package]\nname = \"adopter\"\nversion = \"0.0.0\"\nedition = \"2024\"\n\n\
         [workspace.dependencies]\nalias = { package = 5, version = \"0.2\" }\n\n\
         [dependencies]\nxuanji = { workspace = true }\n",
    )
    .expect("write");
    development_changelog(&fixture.repo, "0.2.0", true);
    commit(&fixture.repo, "chore: a catalog entry under a quoted key");
    let refusal = refuse(
        &fixture.repo,
        Kind::CannotJudge,
        "names a crate this check cannot resolve",
    );
    refusal::expect(
        "release-coherence#example-catalog-entry-unresolvable",
        &refusal,
    );
    let _ = std::fs::remove_dir_all(&root);
}

/// A catalog entry that itself takes the offer is named rather than followed.
///
/// A catalog inheriting from itself is a manifest cargo refuses to parse, and following it is a loop with no
/// end. The arm exists because the resolved entry's own pin is a `Declared` like any other and this is one of
/// its states — reachable from text, so it is answered rather than left to a branch nothing reaches.
#[test]
fn a_catalog_entry_that_itself_inherits_is_named_rather_than_followed() {
    let root = scratch("catalog-self-inherits");
    let fixture = build_fixture(&root, "catalog-self-inherits", "0.2.0");
    std::fs::write(
        fixture.repo.join("examples/adopter/Cargo.toml"),
        "[workspace]\n[package]\nname = \"adopter\"\nversion = \"0.0.0\"\nedition = \"2024\"\n\n\
         [workspace.dependencies]\nxuanji = { workspace = true }\n\n[dependencies]\nxuanji = { workspace = true }\n",
    )
    .expect("write");
    development_changelog(&fixture.repo, "0.2.0", true);
    commit(
        &fixture.repo,
        "chore: a catalog entry that inherits from itself",
    );
    let refusal = refuse(
        &fixture.repo,
        Kind::CannotJudge,
        "whose own entry takes its version from the catalog",
    );
    refusal::expect("release-coherence#example-catalog-entry-inherits", &refusal);
    let _ = std::fs::remove_dir_all(&root);
}

/// A `package` value this reader cannot read stops the check, and says so as itself.
///
/// Negative run: against the reader that carried a package identity as a `String` with the empty string for
/// its two failure states, this refused with *package identity this check cannot read* — the same sentence it
/// produced for a dependency declaring several `package` keys, which is a different fact.
#[test]
fn an_example_whose_package_value_is_unreadable_is_not_judged() {
    let root = scratch("package-unreadable");
    let fixture = build_fixture(&root, "package-unreadable", "0.2.0");
    let manifest = fixture.repo.join("examples/adopter/Cargo.toml");
    let text = std::fs::read_to_string(&manifest).expect("read");
    std::fs::write(
        &manifest,
        // **This WHEN moved when a real parser replaced the hand-rolled reader.** It was a bare word,
        // `package = xuanji`, which is not TOML at all — the parser refuses the document, so the site this
        // direction observes would go unobserved. A `package` that is a value but no string still reaches it.
        format!("{text}alias = {{ package = 5, version = \"0.2.0\" }}\n"),
    )
    .expect("write");
    development_changelog(&fixture.repo, "0.2.0", true);
    commit(&fixture.repo, "chore: name a crate unreadably");
    refusal::expect(
        "release-coherence#dependency-package-value-unreadable",
        &refuse(
            &fixture.repo,
            Kind::CannotJudge,
            "`package` value this check cannot read",
        ),
    );
    let _ = std::fs::remove_dir_all(&root);
}

/// Several `package` keys in one dependency is not one this reader may choose from — and not the same fact
/// as one it cannot read.
///
/// Negative run: both states were the empty string before, so this refused with the *cannot read* sentence.
/// The distinction is the one its sibling field `pin` was given in this same window while this one was left
/// as a sentinel.
#[test]
fn an_example_declaring_several_package_keys_is_not_judged() {
    let root = scratch("package-several");
    let fixture = build_fixture(&root, "package-several", "0.2.0");
    let manifest = fixture.repo.join("examples/adopter/Cargo.toml");
    let text = std::fs::read_to_string(&manifest).expect("read");
    std::fs::write(
        &manifest,
        format!(
            "{text}alias = {{ package = \"xuanji\", package = \"hunyi\", version = \"0.2.0\" }}\n"
        ),
    )
    .expect("write");
    development_changelog(&fixture.repo, "0.2.0", true);
    commit(&fixture.repo, "chore: name two crates at once");
    refusal::expect(
        "release-coherence#manifest-unparseable",
        &refuse(&fixture.repo, Kind::CannotJudge, "duplicate key"),
    );
    let _ = std::fs::remove_dir_all(&root);
}

/// An example requiring a family crate with no version at all is a violation, not an unreadable pin.
///
/// The branch shipped in the 0.5.0 window with no direction over it. It is legal cargo — a path-only dependency
/// declares no version — so nothing about it is hypothetical.
///
/// Negative run: with the `Pin::Absent` arm replaced by `continue`, this passed; restored, it refuses.
#[test]
fn an_example_requiring_a_family_crate_with_no_version_is_refused() {
    let root = scratch("pin-absent");
    let fixture = build_fixture(&root, "pin-absent", "0.2.0");
    let manifest = fixture.repo.join("examples/adopter/Cargo.toml");
    let text = std::fs::read_to_string(&manifest).expect("read");
    std::fs::write(
        &manifest,
        format!("{text}tianheng = {{ path = \"../../crates/tianheng\" }}\n"),
    )
    .expect("write");
    development_changelog(&fixture.repo, "0.2.0", true);
    commit(&fixture.repo, "chore: require a family crate by path alone");
    refusal::expect(
        "release-coherence#example-pin-absent",
        &refuse(
            &fixture.repo,
            Kind::Violation,
            "requires tianheng with no version",
        ),
    );
    let _ = std::fs::remove_dir_all(&root);
}

/// Several `version` keys in one dependency is not this reader's to choose from either.
///
/// The other branch the 0.5.0 window shipped unexercised, and the shape its own module header names as the
/// example of what a real manifest cannot legally carry.
///
/// Negative run: with the `Pin::Several` arm replaced by `continue`, this passed; restored, it refuses.
#[test]
fn an_example_declaring_several_version_keys_is_not_judged() {
    let root = scratch("pin-several");
    let fixture = build_fixture(&root, "pin-several", "0.2.0");
    let manifest = fixture.repo.join("examples/adopter/Cargo.toml");
    let text = std::fs::read_to_string(&manifest).expect("read");
    std::fs::write(
        &manifest,
        format!(
            "{text}machinery-under-another-name = {{ version = \"0.2.0\", version = \"0.1.0\" }}\n"
        ),
    )
    .expect("write");
    development_changelog(&fixture.repo, "0.2.0", true);
    commit(&fixture.repo, "chore: require one crate at two versions");
    refusal::expect(
        "release-coherence#manifest-unparseable",
        &refuse(&fixture.repo, Kind::CannotJudge, "duplicate key"),
    );
    let _ = std::fs::remove_dir_all(&root);
}

/// An internal pin written as a detailed table is read, not refused for the shape of its first line.
///
/// `[workspace.dependencies.xuanji]` with `path` and `version` on their own lines is what cargo writes, and
/// the line-oriented loop this replaced selected any line carrying `path`, `"crates/` and `=` — so the
/// **path** line was split at its `=` and `path` became the dependency's name, while the `version` line,
/// carrying neither `path` nor `"crates/`, was never read at all.
///
/// Negative run: before the migration this refused with *internal dependency path has no version pin*, a
/// false refusal in front of the release gate over a manifest cargo reads correctly.
#[test]
fn an_internal_pin_written_as_a_detailed_table_is_read() {
    let root = scratch("internal-detailed");
    let fixture = build_fixture(&root, "internal-detailed", "0.2.0");
    let manifest = fixture.repo.join("Cargo.toml");
    let text = std::fs::read_to_string(&manifest).expect("read");
    std::fs::write(
        &manifest,
        text.replace(
            "[workspace.dependencies]\nxuanji = { path = \"crates/xuanji\", version = \"0.2.0\" }\n",
            "[workspace.dependencies.xuanji]\npath = \"crates/xuanji\"\nversion = \"0.2.0\"\n",
        ),
    )
    .expect("write");
    development_changelog(&fixture.repo, "0.2.0", true);
    commit(
        &fixture.repo,
        "chore: write an internal pin as a detailed table",
    );
    judge(&fixture.repo).expect("a detailed internal dependency table is one cargo writes");
    let _ = std::fs::remove_dir_all(&root);
}

/// And a stale pin in that same form is still a violation, so the migration did not buy its silence.
///
/// The control for the direction above: without it, a reader that had simply stopped seeing detailed tables
/// would satisfy both.
#[test]
fn a_stale_internal_pin_in_a_detailed_table_is_a_violation() {
    let root = scratch("internal-detailed-stale");
    let fixture = build_fixture(&root, "internal-detailed-stale", "0.2.0");
    let manifest = fixture.repo.join("Cargo.toml");
    let text = std::fs::read_to_string(&manifest).expect("read");
    std::fs::write(
        &manifest,
        text.replace(
            "[workspace.dependencies]\nxuanji = { path = \"crates/xuanji\", version = \"0.2.0\" }\n",
            "[workspace.dependencies.xuanji]\npath = \"crates/xuanji\"\nversion = \"0.0.1\"\n",
        ),
    )
    .expect("write");
    development_changelog(&fixture.repo, "0.2.0", true);
    commit(&fixture.repo, "chore: leave an internal pin behind");
    refusal::expect(
        "release-coherence#internal-pin-disagrees",
        &refuse(&fixture.repo, Kind::Violation, "is pinned to 0.0.1"),
    );
    let _ = std::fs::remove_dir_all(&root);
}

/// A workspace version that is present and not a version is not an absent one.
///
/// Negative run: this branch had no direction. With it replaced by `Ok`, the fixture reached the changelog
/// checks and refused there instead, naming a surface the operator had not touched.
#[test]
fn a_workspace_version_that_is_not_a_version_cannot_be_judged() {
    let root = scratch("version-malformed");
    let repo = root.join("repo");
    std::fs::create_dir_all(&repo).expect("create");
    initialised(&repo);
    std::fs::write(
        repo.join("Cargo.toml"),
        "[workspace.package]\nversion = \"banana\"\n",
    )
    .expect("write");
    std::fs::write(repo.join("CHANGELOG.md"), "# Changelog\n").expect("write");
    refusal::expect(
        "release-coherence#workspace-version-malformed",
        &refuse(&repo, Kind::CannotJudge, "missing or malformed: banana"),
    );
    let _ = std::fs::remove_dir_all(&root);
}

/// A crate manifest declaring no package name stops the example-pin check rather than shrinking its family.
///
/// A family member that drops out of the family is one every example may then pin at any version at all,
/// which is the partial case an aggregate vacuity guard is exactly unable to see.
///
/// Negative run: with the arm replaced by a `continue`, the fixture passed — the remaining members satisfied
/// the counters and the nameless one went unchecked.
#[test]
fn a_crate_manifest_declaring_no_package_name_stops_the_example_check() {
    let root = scratch("crate-nameless");
    let fixture = build_fixture(&root, "crate-nameless", "0.2.0");
    std::fs::write(
        fixture.repo.join("crates/xuanji/Cargo.toml"),
        "[package]\nversion.workspace = true\nedition = \"2024\"\n",
    )
    .expect("write");
    development_changelog(&fixture.repo, "0.2.0", true);
    commit(&fixture.repo, "chore: leave a member unnamed");
    refusal::expect(
        "release-coherence#crate-package-name-absent",
        &refuse(
            &fixture.repo,
            Kind::CannotJudge,
            "declares no `[package]` name",
        ),
    );
    let _ = std::fs::remove_dir_all(&root);
}

/// And a package name this reader cannot take is not an absent one.
///
/// Single-quoted TOML strings are legal and this reader does not read them — a limit of the reader rather
/// than a fact about the manifest, which is the distinction `Quoted` exists to keep.
///
/// Negative run: with the arm replaced by a `continue`, the fixture passed.
#[test]
fn a_crate_package_name_this_reader_cannot_take_stops_the_example_check() {
    let root = scratch("crate-unreadable-name");
    let fixture = build_fixture(&root, "crate-unreadable-name", "0.2.0");
    std::fs::write(
        fixture.repo.join("crates/xuanji/Cargo.toml"),
        // **This WHEN moved when a real parser replaced the hand-rolled reader.** It was `name = 'xuanji'` — a
        // single-quoted string, legal TOML the old reader declined — and the parser takes it. What still
        // reaches the site is a `name` that is not a string at all.
        "[package]\nname = { workspace = true }\nversion.workspace = true\nedition = \"2024\"\n",
    )
    .expect("write");
    development_changelog(&fixture.repo, "0.2.0", true);
    commit(&fixture.repo, "chore: name a member unreadably");
    refusal::expect(
        "release-coherence#crate-package-name-unreadable",
        &refuse(
            &fixture.repo,
            Kind::CannotJudge,
            "declares a `[package]` name this check cannot read",
        ),
    );
    let _ = std::fs::remove_dir_all(&root);
}

/// An example pin this reader cannot take is not one that satisfies the workspace version.
///
/// Negative run: with the `Declared::Unreadable` arm replaced by a `continue`, the entry was skipped and the
/// remaining example requirement satisfied the vacuity guard, so the fixture passed.
#[test]
fn an_example_pin_this_reader_cannot_take_is_not_one_that_satisfies() {
    let root = scratch("example-pin-unreadable");
    let fixture = build_fixture(&root, "example-pin-unreadable", "0.2.0");
    let manifest = fixture.repo.join("examples/adopter/Cargo.toml");
    let text = std::fs::read_to_string(&manifest).expect("read");
    std::fs::write(
        &manifest,
        // **This WHEN moved when a real parser replaced the hand-rolled reader.** It was a single-quoted
        // version, legal TOML the old reader declined; the parser takes it. What still cannot be taken as a
        // requirement is a value that is no string.
        format!("{text}tianheng = {{ version = 5 }}\n"),
    )
    .expect("write");
    development_changelog(&fixture.repo, "0.2.0", true);
    commit(&fixture.repo, "chore: pin a family crate unreadably");
    refusal::expect(
        "release-coherence#example-pin-unreadable",
        &refuse(
            &fixture.repo,
            Kind::CannotJudge,
            "with a version this check cannot read",
        ),
    );
    let _ = std::fs::remove_dir_all(&root);
}

/// A root manifest declaring no dependency on a family crate is a check reporting over nothing.
///
/// The vacuity guard: the declaration form is cargo's to change, and a reader that found none would
/// otherwise pass every release while judging no pin at all.
///
/// Negative run: with the guard replaced by `Ok(())`, this fixture passed — a release with no internal pin
/// checked at all, reported clean.
#[test]
fn a_root_manifest_with_no_internal_path_dependency_reports_over_nothing() {
    let root = scratch("no-internal-pin");
    let fixture = build_fixture(&root, "no-internal-pin", "0.2.0");
    let manifest = fixture.repo.join("Cargo.toml");
    let text = std::fs::read_to_string(&manifest).expect("read");
    std::fs::write(
        &manifest,
        text.replace(
            "xuanji = { path = \"crates/xuanji\", version = \"0.2.0\" }\n",
            "serde_json = \"1\"\n",
        ),
    )
    .expect("write");
    development_changelog(&fixture.repo, "0.2.0", true);
    commit(
        &fixture.repo,
        "chore: declare no dependency on a family crate",
    );
    refusal::expect(
        "release-coherence#no-internal-family-dependency-found",
        &refuse(
            &fixture.repo,
            Kind::CannotJudge,
            "found no dependency on a family crate",
        ),
    );
    let _ = std::fs::remove_dir_all(&root);
}

/// An `examples/` that holds no manifest at all is the layout having changed, not every example passing.
///
/// Its sibling direction covers one example directory holding no manifest while others do; this covers the
/// aggregate reaching zero, which is what the counter exists for.
///
/// Negative run: with the guard replaced by `Ok(())`, this fixture passed.
#[test]
fn an_examples_directory_holding_no_manifest_at_all_reports_over_nothing() {
    let root = scratch("no-example-manifests");
    let fixture = build_fixture(&root, "no-example-manifests", "0.2.0");
    std::fs::remove_file(fixture.repo.join("examples/adopter/Cargo.toml")).expect("remove");
    std::fs::write(
        fixture.repo.join("examples/adopter/README.md"),
        "no manifest\n",
    )
    .expect("write");
    development_changelog(&fixture.repo, "0.2.0", true);
    commit(&fixture.repo, "chore: leave examples/ carrying no manifest");
    refusal::expect(
        "release-coherence#no-example-manifests-found",
        &refuse(
            &fixture.repo,
            Kind::CannotJudge,
            "found no example manifests under examples/",
        ),
    );
    let _ = std::fs::remove_dir_all(&root);
}

/// Examples that require no family crate are examples this check has nothing to say about.
///
/// The second vacuity guard, and the one the first cannot cover: an example was read and declared no family
/// dependency, so its pins would be reported over an empty set.
///
/// **Counted per example since this was written.** The guard was an aggregate over every example, which is
/// the shape that cannot see a partial read: seven examples parsing kept it non-zero while an eighth went
/// unexamined. This fixture carries one example and reaches the same refusal either way — which is why it
/// could not have caught the aggregate's own hole, and why the sibling below exists.
///
/// Negative run: with the guard replaced by `Ok(())`, this fixture passed.
#[test]
fn an_example_requiring_no_family_crate_reports_over_nothing() {
    let root = scratch("no-family-requirement");
    let fixture = build_fixture(&root, "no-family-requirement", "0.2.0");
    std::fs::write(
        fixture.repo.join("examples/adopter/Cargo.toml"),
        "[package]\nname = \"adopter\"\nversion = \"0.0.0\"\nedition = \"2024\"\n\n\
         [dependencies]\nserde_json = \"1\"\n",
    )
    .expect("write");
    development_changelog(&fixture.repo, "0.2.0", true);
    commit(
        &fixture.repo,
        "chore: require no family crate from any example",
    );
    refusal::expect(
        "release-coherence#example-requires-no-family-crate",
        &refuse(
            &fixture.repo,
            Kind::CannotJudge,
            "declares no family dependency requirement",
        ),
    );
    let _ = std::fs::remove_dir_all(&root);
}

/// The case the aggregate guard could not see: one example fine, one declaring nothing.
///
/// **This is the falsifier the sibling above cannot be.** With the counter outside the loop, `adopter`'s own
/// correct pin kept it non-zero and `bare` was never examined — the partial read that let a renamed key and
/// then a quoted key each reach a release as clean, through the one door that stays open when the reader
/// misses a declaration rather than misreading it. Two examples, and the second is the one the aggregate
/// would have carried past.
///
/// Negative run: with the counter hoisted back out of the loop, this fixture passed and named nothing.
#[test]
fn an_example_declaring_nothing_is_refused_though_its_sibling_is_fine() {
    let root = scratch("bare-beside-a-good-one");
    let fixture = build_fixture(&root, "bare-beside-a-good-one", "0.2.0");
    // `adopter` keeps the pin the fixture builds it with, so the aggregate counter would stay non-zero.
    std::fs::create_dir_all(fixture.repo.join("examples/bare")).expect("create");
    std::fs::write(
        fixture.repo.join("examples/bare/Cargo.toml"),
        "[package]\nname = \"bare\"\nversion = \"0.0.0\"\nedition = \"2024\"\n\n\
         [dependencies]\nserde_json = \"1\"\n",
    )
    .expect("write");
    development_changelog(&fixture.repo, "0.2.0", true);
    commit(
        &fixture.repo,
        "chore: add an example requiring no family crate",
    );
    let refusal = refuse(&fixture.repo, Kind::CannotJudge, "example bare declares no");
    refusal::expect(
        "release-coherence#example-requires-no-family-crate",
        &refusal,
    );
    assert!(
        refusal.message.contains("bare"),
        "the refusal must name WHICH example declared nothing, or an operator cannot find it among the \
         siblings that are fine: {}",
        refusal.message
    );
    let _ = std::fs::remove_dir_all(&root);
}

/// And a version it cannot take is not a version that disagrees.
///
/// Negative run: with the arm replaced by a `continue`, the entry was filed with no version and the member
/// reported missing from the lock — the same wrong sentence its sibling above produced.
#[test]
fn a_lock_version_this_reader_cannot_take_stops_the_comparison() {
    let root = scratch("lock-version-unread");
    let fixture = build_fixture(&root, "lock-version-unreadable", "0.2.0");
    with_machinery(&fixture.repo);
    workspace_files(&fixture.repo, "0.2.1");
    release_changelog(&fixture.repo, "0.2.1", "0.2.0");
    let lock = fixture.repo.join("Cargo.lock");
    let text = std::fs::read_to_string(&lock).expect("read");
    std::fs::write(
        &lock,
        // The same moved WHEN: a single-quoted version is read now, so what stops the comparison is a
        // version that is no string.
        text.replace(
            "name = \"xuanji\"\nversion = \"0.2.1\"",
            "name = \"xuanji\"\nversion = 3",
        ),
    )
    .expect("write");
    commit(&fixture.repo, "release: 0.2.1");
    refusal::expect(
        "release-coherence#lock-version-unreadable",
        &refuse(
            &fixture.repo,
            Kind::CannotJudge,
            "records a version for xuanji that this check cannot read",
        ),
    );
    let _ = std::fs::remove_dir_all(&root);
}

/// A release snapshot naming one version while the workspace declares another.
///
/// HEAD *is* the release commit, so the tree claims to be that release; the version surfaces say otherwise.
/// Nothing downstream can be judged after that, because which of the two the release is has no answer.
///
/// Negative run: with the arm replaced by `State::Snapshot`, the fixture went on to be judged as a snapshot
/// of the version its own release commit does not name.
#[test]
fn a_release_snapshot_naming_another_version_is_a_violation() {
    let root = scratch("snapshot-disagrees");
    let fixture = build_fixture(&root, "snapshot-disagrees", "0.2.0");
    release_changelog(&fixture.repo, "0.2.0", "0.1.0");
    // The release commit has to carry a change, and what it carries is beside the point: what this observes
    // is the subject of the commit HEAD sits on against the version the surfaces declare.
    std::fs::write(fixture.repo.join("NOTES.md"), "prepared\n").expect("write");
    commit(&fixture.repo, "release: 0.3.0");
    refusal::expect(
        "release-coherence#release-snapshot-version-disagrees",
        &refuse(
            &fixture.repo,
            Kind::Violation,
            "release snapshot subject is 0.3.0 but workspace version is 0.2.0",
        ),
    );
    let _ = std::fs::remove_dir_all(&root);
}

/// A crate manifest that is not text stops the enumeration rather than shrinking it.
///
/// The sibling of the example-manifest direction, on the corpus that decides the family: a manifest skipped
/// here is a member that drops out of every downstream comparison while the counters stay satisfied.
///
/// Negative run: with the read replaced by a skip, the fixture passed — one member's pins, lock entry and
/// version inheritance all unjudged, reported clean.
#[test]
fn a_crate_manifest_that_is_not_text_cannot_be_read() {
    let root = scratch("crate-not-text");
    let fixture = build_fixture(&root, "crate-not-text", "0.2.0");
    std::fs::write(
        fixture.repo.join("crates/xuanji/Cargo.toml"),
        [0x66, 0x6f, 0xff, 0xfe],
    )
    .expect("write");
    development_changelog(&fixture.repo, "0.2.0", true);
    commit(
        &fixture.repo,
        "chore: write a member manifest that is not text",
    );
    refusal::expect(
        "release-coherence#crate-manifest-unreadable",
        &refuse(&fixture.repo, Kind::CannotJudge, "could not read"),
    );
    let _ = std::fs::remove_dir_all(&root);
}

/// `cargo metadata` failing over a repository whose surfaces this gate has already read.
///
/// **Not a broken tool.** The line readers judge version surfaces, pins and the lock, and none of them
/// resolves a member's path dependencies — so a member declaring `{ path = "../nope" }` passes every one of
/// them and cargo refuses the workspace. That is a defect in the judged repository, which makes this a
/// refusal about the subject: its fixture is the defect it names, and it was declared unheld on the reading
/// that only a broken tool reaches it.
///
/// Negative run: with the arm replaced by an empty corpus, the gate went on to judge the machinery set over
/// a workspace cargo cannot load, and passed.
#[test]
fn a_metadata_failure_the_subject_caused_is_reported() {
    let root = scratch("metadata-subject");
    let fixture = build_fixture(&root, "metadata-subject", "0.2.0");
    let manifest = fixture.repo.join("crates/xuanji/Cargo.toml");
    let text = std::fs::read_to_string(&manifest).expect("read");
    std::fs::write(
        &manifest,
        format!("{text}\n[dependencies]\nabsent = {{ path = \"../nowhere\" }}\n"),
    )
    .expect("write");
    development_changelog(&fixture.repo, "0.2.0", true);
    commit(&fixture.repo, "chore: depend on a path that is not there");
    refusal::expect(
        "release-coherence#cargo-metadata-failed",
        &refuse(&fixture.repo, Kind::CannotJudge, "cargo metadata failed"),
    );
    let _ = std::fs::remove_dir_all(&root);
}

/// A workspace whose members carry no tracked file at all.
///
/// **A repository shape, not a broken tool.** cargo loads the workspace from the worktree while git tracks
/// nothing under the members, so the enumeration that derives the machinery corpus finds no file to
/// classify. It was declared unheld on a reason that does not fit the table it sat in — *a repository shape
/// rather than a release surface* is an argument about what is worth reacting to, and that table's criterion
/// is that only a broken tool reaches the branch.
///
/// Negative run: with the guard replaced by `Ok(())`, this passed — the machinery corpus derived from no
/// member at all, and an adopter-facing entry naming a gate would be judged against it.
#[test]
fn a_workspace_whose_members_are_untracked_reports_over_nothing() {
    let root = scratch("members-untracked");
    let fixture = build_fixture(&root, "members-untracked", "0.2.0");
    development_changelog(&fixture.repo, "0.2.0", true);
    commit(&fixture.repo, "chore: prepare");
    // Through git directly, because the fixture's `commit` stages everything and would put the members
    // straight back.
    git(&fixture.repo, &["rm", "-r", "--cached", "-q", "crates"]);
    git(
        &fixture.repo,
        &["commit", "-qm", "chore: stop tracking every member"],
    );
    refusal::expect(
        "release-coherence#no-tracked-file-for-any-member",
        &refuse(
            &fixture.repo,
            Kind::CannotJudge,
            "no tracked file was found",
        ),
    );
    let _ = std::fs::remove_dir_all(&root);
}

/// The unpublished members contributing nothing is refused, where a published sibling used to cover for it.
///
/// **A vacuity guard measuring a wider set than the one it protects — the third of that shape in this file.**
/// The machinery set is drawn from the **unpublished** members plus `scripts/`, and the floor counted every
/// member's tracked paths, published ones included. One tracked file under `crates/xuanji` therefore kept the
/// counter non-zero while the machinery set was `scripts/` alone — the state the floor's own message
/// described. The check would then run against `scripts/` alone and report clean over a nearly-empty subject.
///
/// **Reachable, and reported as run rather than as read.** A review found this by reading and judged the
/// state unreachable through `build_fixture`. It is reachable: untrack the two unpublished members and leave
/// the published one alone, which is what this does. `cargo metadata` still names all three, because it reads
/// the filesystem rather than the index.
///
/// Negative run: with the floor restored to `enumerated == 0`, this reports `ok release coherence` — one
/// tracked file under the published member covering for a subject that is empty.
#[test]
fn unpublished_members_contributing_nothing_is_refused() {
    let root = scratch("machinery-empty");
    let fixture = build_fixture(&root, "machinery-empty", "0.2.0");
    development_changelog(&fixture.repo, "0.2.0", true);
    commit(&fixture.repo, "chore: prepare");
    // Through git directly, because the fixture's `commit` stages everything and would put them back. Only
    // the unpublished members go: `crates/xuanji` stays tracked, and it is what kept the old floor silent.
    git(
        &fixture.repo,
        &[
            "rm",
            "-r",
            "--cached",
            "-q",
            "crates/tianheng",
            "crates/renamed-dir",
        ],
    );
    git(
        &fixture.repo,
        &[
            "commit",
            "-qm",
            "chore: stop tracking the unpublished members",
        ],
    );
    let refusal = refuse(
        &fixture.repo,
        Kind::CannotJudge,
        "contributed no tracked file",
    );
    refusal::expect(
        "release-coherence#no-machinery-from-unpublished-members",
        &refusal,
    );
    assert!(
        refusal.message.contains("crates/tianheng")
            && refusal.message.contains("crates/renamed-dir"),
        "the refusal names the members that were expected to contribute: {}",
        refusal.message
    );
    let _ = std::fs::remove_dir_all(&root);
}

/// A dotted key whose tail this reader does not judge is still a declared dependency.
///
/// **A review read the ordering as a hygiene defect and it is the opposite.** `declared_dependencies` creates
/// a dotted key's record BEFORE matching the tail, so `tianheng.git = "…"` — legal cargo, and a tail this
/// reader does not judge — yields a record carrying no pin. Read alone that looks like a dependency the
/// manifest never declared. Read beside `tianheng = { git = "…" }` it is the only answer that keeps the two
/// spellings equal, because the inline reader takes its key before reading any field.
///
/// What the proposed repair would cost is not tidiness. Deferring the insert until the tail is recognised
/// makes the dotted form declare NOTHING, the entry drops out before `example-pin-absent` is reached, and an
/// example requiring a family crate with no version passes — the false-negative shape this same function
/// already closed twice, once for a renamed key and once for a quoted one.
///
/// Negative run, with the insert moved below the `match`: the dotted half reports `no refusal` where it must
/// refuse, and the inline half still refuses — the two spellings disagreeing, which is what this direction
/// exists to make impossible.
#[test]
fn an_unjudged_dotted_tail_declares_as_its_inline_spelling_does() {
    for (label, line) in [
        (
            "dotted",
            "tianheng.git = \"https://example.invalid/tianheng\"\n",
        ),
        (
            "inline",
            "tianheng = { git = \"https://example.invalid/tianheng\" }\n",
        ),
    ] {
        let root = scratch(&format!("unjudged-tail-{label}"));
        let fixture = build_fixture(&root, &format!("unjudged-tail-{label}"), "0.2.0");
        let manifest = fixture.repo.join("examples/adopter/Cargo.toml");
        let text = std::fs::read_to_string(&manifest).expect("read");
        std::fs::write(&manifest, format!("{text}{line}")).expect("write");
        development_changelog(&fixture.repo, "0.2.0", true);
        commit(
            &fixture.repo,
            "chore: require a family crate through a tail this reader does not judge",
        );
        refusal::expect(
            "release-coherence#example-pin-absent",
            &refuse(
                &fixture.repo,
                Kind::Violation,
                "requires tianheng with no version",
            ),
        );
        let _ = std::fs::remove_dir_all(&root);
    }
}
