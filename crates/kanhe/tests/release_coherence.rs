//! Repository check: the release commit spine, the version-bearing surfaces, and the changelog.
//!
//! The judgement lives in `support/release_coherence_gate.rs`, shared by this gate and by the fixtures below,
//! because two constructions of "a repository with a changelog" is the twin-drift class this repository keeps
//! closing. It separates a **violation** from a **cannot-judge**, and this matrix asserts which — a matrix
//! reading only "non-zero" was blind to exactly the regression the shell era's shared backstop introduced,
//! where every genuine incoherence was reported as cannot-judge with CI green throughout.

use kanhe::refusal;

use kanhe::release_coherence_gate as gate;

use gate::{
    build_fixture, commit, development_changelog, hermetic, judge, release_changelog,
    workspace_files,
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

fn git(repo: &Path, args: &[&str]) {
    let out = hermetic("git")
        .args(args)
        .current_dir(repo)
        .output()
        .unwrap_or_else(|err| panic!("cannot run git {args:?}: {err}"));
    assert!(
        out.status.success(),
        "git {args:?} failed: {}",
        String::from_utf8_lossy(&out.stderr)
    );
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
        text.replace("## [0.2.1] - 2026-07-20", "## [0.2.1] - notadate!!"),
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
        std::fs::write(&manifest, format!("{text}\n{table}")).expect("write");
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
                "## [0.2.1] - 2026-07-20",
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
        "[package]\nname = 'xuanji'\nversion.workspace = true\nedition = \"2024\"\n",
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
    std::fs::write(&lock, text.replace("name = \"xuanji\"", "name = 'xuanji'")).expect("write");
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

/// `release-coherence/a-heading-inside-a-fenced-code-block-a-stated-bound`
///
/// `UnderReacts`, owned by the engine. The check walks the document's line grammar and does not track
/// fences, so a `### ` line inside a fenced block sets the heading in force — and can name the one exempt
/// heading. Latent rather than live: this repository's changelog carries no fenced block at all.
#[test]
fn a_heading_inside_a_fenced_block_is_a_stated_bound() {
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
    assert!(
        verdict.is_ok(),
        "the check must stay silent when a fenced heading reattributes a later entry. Got: {:?}",
        verdict.err()
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
            "no internal path dependency",
            "internal-pins" as &str,
            "found no internal path dependency",
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
        (
            "no family requirement in any example",
            "example-reqs",
            "found no family dependency requirement",
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
/// A single-quoted literal is valid TOML and is not a form this reader takes. Reporting it as *missing or
/// malformed* sends an operator to look for a version key that is sitting right there, correctly spelled for
/// cargo — the same conflation `Quoted` was introduced to end one reader over.
#[test]
fn a_version_value_this_reader_cannot_read_is_not_one_that_is_absent() {
    let root = scratch("unreadable-version-value");
    let repo = bare(&root, "repo");
    initialised(&repo);
    std::fs::write(
        repo.join("Cargo.toml"),
        "[workspace.package]\nversion = '0.4.0'\n",
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
/// `quoted_value` takes the text between the first pair of quotes and discards what follows, so a
/// trailing comment there was always read correctly. The claim was refuted by a reviewer and is replaced by
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
/// the context is two bare TOML keys. The reason the sibling cfg form is left alone — that a quoted cfg
/// expression is the grammar a line-oriented reader is likeliest to be wrong about — says nothing about this
/// one, and reading it needs no guess: TOML bare keys carry no dot, so the triple runs to the next dot.
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

/// A family pin under a **quoted cfg** target table is still not read, which is the declared bound.
///
/// The control for the direction above: it holds against a reader that stopped where the bound says, and
/// fails against one that admitted every `[target.…]` heading alike. It is not a negative run — it passed
/// before the change too — and it is here because the change it guards is a change to where the reader
/// stops, which only a pair of cases can locate.
#[test]
fn a_family_pin_under_a_quoted_cfg_target_is_the_declared_bound() {
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
    judge(&fixture.repo).expect("a cfg-guarded family pin is a declared bound, not a violation");
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
        format!("{text}alias = {{ package = xuanji, version = \"0.2.0\" }}\n"),
    )
    .expect("write");
    development_changelog(&fixture.repo, "0.2.0", true);
    commit(&fixture.repo, "chore: name a crate unreadably");
    refusal::expect(
        "release-coherence#example-package-value-unreadable",
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
        "release-coherence#example-declares-several-packages",
        &refuse(
            &fixture.repo,
            Kind::CannotJudge,
            "declares 2 `package` keys",
        ),
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
        "release-coherence#example-declares-several-pins",
        &refuse(
            &fixture.repo,
            Kind::CannotJudge,
            "declares 2 `version` keys",
        ),
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
        "[package]\nname = 'xuanji'\nversion.workspace = true\nedition = \"2024\"\n",
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
        format!("{text}tianheng = {{ version = '0.2.0' }}\n"),
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

/// A root manifest declaring no internal path dependency is a check reporting over nothing.
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
    commit(&fixture.repo, "chore: declare no internal path dependency");
    refusal::expect(
        "release-coherence#no-internal-path-dependency-found",
        &refuse(
            &fixture.repo,
            Kind::CannotJudge,
            "found no internal path dependency",
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
/// The second vacuity guard, and the one the first cannot cover: manifests were read and none of them
/// declared a family dependency, so every example pin would be reported over an empty set.
///
/// Negative run: with the guard replaced by `Ok(())`, this fixture passed.
#[test]
fn examples_requiring_no_family_crate_report_over_nothing() {
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
        "release-coherence#no-family-requirement-in-examples",
        &refuse(
            &fixture.repo,
            Kind::CannotJudge,
            "found no family dependency requirement",
        ),
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
        text.replace(
            "name = \"xuanji\"\nversion = \"0.2.1\"",
            "name = \"xuanji\"\nversion = '0.2.1'",
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
