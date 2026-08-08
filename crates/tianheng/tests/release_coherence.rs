//! Self-governance reaction: the release commit spine, the version-bearing surfaces, and the changelog.
//!
//! The judgement lives in `support/release_coherence_gate.rs`, shared by this gate and by the fixtures below,
//! because two constructions of "a repository with a changelog" is the twin-drift class this repository keeps
//! closing. It separates a **violation** from a **cannot-judge**, and this matrix asserts which — a matrix
//! reading only "non-zero" was blind to exactly the regression the shell era's shared backstop introduced,
//! where every genuine incoherence was reported as cannot-judge with CI green throughout.

#[path = "support/release_coherence_gate.rs"]
mod gate;

use gate::{
    Kind, build_fixture, commit, development_changelog, hermetic, judge, release_changelog,
    workspace_files,
};
use std::path::{Path, PathBuf};

fn locate_layout(root: PathBuf, marker_set: bool) -> Option<PathBuf> {
    if root.join("CHANGELOG.md").is_file() {
        return Some(root);
    }
    assert!(
        !marker_set,
        "CHANGELOG.md expected under {root:?} but absent while TIANHENG_WORKSPACE_TESTS is set — a governance \
         reaction that quietly does nothing in CI is the shape this family argues against"
    );
    None
}

fn workspace_root() -> Option<PathBuf> {
    locate_layout(
        PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../.."),
        std::env::var_os("TIANHENG_WORKSPACE_TESTS").is_some(),
    )
}

fn scratch(name: &str) -> PathBuf {
    let root = std::env::temp_dir().join(format!(
        "tianheng-release-coherence-{name}-{}",
        std::process::id()
    ));
    let _ = std::fs::remove_dir_all(&root);
    std::fs::create_dir_all(&root).expect("the fixture root is writable");
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
        repo.join("scripts/check_pin_bites.sh"),
        "#!/usr/bin/env bash\nexit 0\n",
    )
    .expect("write");
}

fn refuse(repo: &Path, kind: Kind, needle: &str) {
    let refusal = judge(repo).expect_err(&format!("expected a refusal containing {needle:?}"));
    assert_eq!(refusal.kind, kind, "{}", refusal.message);
    assert!(
        refusal.message.contains(needle),
        "expected a refusal containing {needle:?}, got: {}",
        refusal.message
    );
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
    refuse(&repo, Kind::CannotJudge, "release history is unavailable");
    let _ = std::fs::remove_dir_all(&root);
}

#[test]
fn a_malformed_release_subject_is_a_violation() {
    let root = scratch("malformed");
    let fixture = build_fixture(&root, "malformed", "0.2.0");
    development_changelog(&fixture.repo, "0.2.0", true);
    commit(&fixture.repo, "release: next");
    refuse(
        &fixture.repo,
        Kind::Violation,
        "malformed release history subject",
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
    refuse(
        &fixture.repo,
        Kind::Violation,
        "is older than latest release 0.2.0",
    );
    let _ = std::fs::remove_dir_all(&root);
}

#[test]
fn development_with_no_release_narrative_is_a_violation() {
    let root = scratch("empty-development");
    let fixture = build_fixture(&root, "empty-development", "0.2.0");
    development_changelog(&fixture.repo, "0.2.0", false);
    commit(&fixture.repo, "chore: omit release note");
    refuse(
        &fixture.repo,
        Kind::Violation,
        "requires adopter-facing release narrative",
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
    refuse(
        &fixture.repo,
        Kind::Violation,
        "must inherit version.workspace = true",
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
    refuse(&fixture.repo, Kind::Violation, "is pinned to 0.1.0");
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
    refuse(&fixture.repo, Kind::Violation, "requires xuanji = \"0.9\"");
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
    refuse(
        &fixture.repo,
        Kind::Violation,
        "Cargo.lock package xuanji is 0.2.0",
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
    refuse(&fixture.repo, Kind::Violation, "repeats a heading");
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
    refuse(&fixture.repo, Kind::Violation, "carries no `### Migration`");
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

// --- adopter narrative names no self-governance machinery ---------------------------------------------------

#[test]
fn an_adopter_heading_naming_a_gate_is_a_violation() {
    let root = scratch("adopter-names-path");
    let fixture = build_fixture(&root, "adopter-names-path", "0.2.0");
    development_changelog(&fixture.repo, "0.2.0", true);
    with_machinery(&fixture.repo);
    unreleased_body(
        &fixture.repo,
        "### Fixed\n- A repair naming `scripts/check_pin_bites.sh`.",
    );
    commit(&fixture.repo, "docs: name a gate under an adopter heading");
    refuse(
        &fixture.repo,
        Kind::Violation,
        "names this repository's own machinery",
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
        "### Self-governance\n- A repair naming `scripts/check_pin_bites.sh`.",
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
            "### Fixed\n- A repair naming `check_pin_bites.sh`.",
        ),
        (
            "inside-a-command",
            "### Fixed\n- Run `bash scripts/check_pin_bites.sh --fix` to repair.",
        ),
        (
            "nested-span",
            "### Fixed\n- A repair naming `` `scripts/check_pin_bites.sh` `` in a nested span.",
        ),
        (
            "wrapped-span",
            "### Fixed\n- A repair naming `scripts/check_pin_bites.sh\n  ` across a wrap.",
        ),
        (
            "link-target",
            "### Fixed\n- A repair naming [the gate](scripts/check_pin_bites.sh).",
        ),
        (
            "unquoted-prose",
            "### Fixed\n- A repair to the check_pin_bites.sh gate, written as prose.",
        ),
        (
            "sentence-end",
            "### Fixed\n- A repair to scripts/check_pin_bites.sh.",
        ),
        (
            "the-directory",
            "### Fixed\n- A repair described by naming `scripts/` and nothing in it.",
        ),
    ] {
        let root = scratch(name);
        let fixture = build_fixture(&root, name, "0.2.0");
        development_changelog(&fixture.repo, "0.2.0", true);
        with_machinery(&fixture.repo);
        unreleased_body(&fixture.repo, body);
        commit(&fixture.repo, "docs: name a gate");
        refuse(
            &fixture.repo,
            Kind::Violation,
            "names this repository's own machinery",
        );
        let _ = std::fs::remove_dir_all(&root);
    }
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

/// Refuse to accept silence from a reaction that is not reacting at all.
///
/// Every bound below asserts SILENCE, and silence has more than one cause. Adversarial review found the sharp
/// case by widening a scope AND blinding an enumerator at once: a bound was then plainly false and its pin
/// stayed green, because a dead reaction is silent about everything. Only a live control reaches that.
fn assert_reaction_is_live(root: &Path) {
    let fixture = build_fixture(root, "live-control", "0.2.0");
    development_changelog(&fixture.repo, "0.2.0", true);
    with_machinery(&fixture.repo);
    unreleased_body(
        &fixture.repo,
        "### Fixed\n- A repair naming `scripts/check_pin_bites.sh`.",
    );
    commit(&fixture.repo, "docs: the live control");
    let verdict = judge(&fixture.repo);
    assert!(
        verdict.is_err(),
        "the control must be refused, or the silence a pin is about to assert says nothing — a reaction that \
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
            "### Fixed\n- A repair naming `scripts/check_pin_bites.sh`.\n",
        ),
    )
    .expect("write");
    commit(&fixture.repo, "docs: a dated section names a gate");
    let verdict = judge(&fixture.repo);
    let _ = std::fs::remove_dir_all(&root);
    assert!(
        verdict.is_ok(),
        "the reaction must stay silent about a dated section naming machinery — that is the declared bound. \
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
        "### Fixed\n- A repair naming `scripts/check_pin_bites.sh`.",
    );
    commit(&fixture.repo, "docs: name a gate before it is tracked");
    with_machinery(&fixture.repo); // written, never added
    let verdict = judge(&fixture.repo);
    let _ = std::fs::remove_dir_all(&root);
    assert!(
        verdict.is_ok(),
        "the reaction must stay silent about machinery no commit tracks. Got: {:?}",
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
        verdict.expect_err("the over-reaction is the declared bound; silence would close it");
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
        "### Fixed\n- A repair to the scripts and to scripts/lib, written without a trailing slash.",
    );
    commit(&fixture.repo, "docs: name a directory without its slash");
    let verdict = judge(&fixture.repo);
    let _ = std::fs::remove_dir_all(&root);
    assert!(
        verdict.is_ok(),
        "the reaction must stay silent about a directory named without its trailing slash. Got: {:?}",
        verdict.err()
    );
}

/// `release-coherence/a-name-reached-only-through-a-url-a-stated-bound`
///
/// `UnderReacts`, owned by the engine. A word is a maximal run of path characters, so a scheme and host fuse
/// with the path into one run that equals no tracked name; splitting a URL would make the reaction judge a
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
        "### Fixed\n- See https://github.com/tacticaldoll/tianheng/blob/main/scripts/check_pin_bites.sh for it.",
    );
    commit(&fixture.repo, "docs: reach a gate only through a URL");
    let verdict = judge(&fixture.repo);
    let _ = std::fs::remove_dir_all(&root);
    assert!(
        verdict.is_ok(),
        "the reaction must stay silent about a name reached only through a URL. Got: {:?}",
        verdict.err()
    );
}

/// `release-coherence/a-heading-inside-a-fenced-code-block-a-stated-bound`
///
/// `UnderReacts`, owned by the engine. The reaction walks the document's line grammar and does not track
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
        "### Fixed\n- A repair.\n\n```\n### Self-governance\n```\n\n- A later repair naming `scripts/check_pin_bites.sh`.",
    );
    commit(&fixture.repo, "docs: put a heading inside a fence");
    let verdict = judge(&fixture.repo);
    let _ = std::fs::remove_dir_all(&root);
    assert!(
        verdict.is_ok(),
        "the reaction must stay silent when a fenced heading reattributes a later entry. Got: {:?}",
        verdict.err()
    );
}

#[test]
fn an_absent_layout_is_loud_when_the_workspace_marker_is_set() {
    let absent = std::env::temp_dir().join("tianheng-release-coherence-absent");
    let _ = std::fs::remove_dir_all(&absent);
    assert!(locate_layout(absent.clone(), false).is_none());
    assert!(
        std::panic::catch_unwind(|| locate_layout(absent, true)).is_err(),
        "an absent layout must fail loudly under TIANHENG_WORKSPACE_TESTS rather than skip"
    );
}
