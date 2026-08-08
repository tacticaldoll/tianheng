//! `release-coherence`'s declared bounds for the adopter-narrative rule, demonstrated.
//!
//! The capability's reactions are a shell gate and its twin, and `PINNED-BY` resolves only a harness-registered
//! Rust function — so a bound belonging to that gate can be defended by a twin direction and cited by nothing.
//! This file is that citation, the same arrangement `publish_source_integrity.rs` already carries and for the
//! same reason. It exists for these bounds and says so, rather than growing into a second reaction over a
//! surface `scripts/check_release_coherence.sh` already owns.
//!
//! It builds its fixtures through `scripts/lib/coherence_fixture.sh`, the same builder the twin uses. Two
//! constructions of "a repository with a changelog and some machinery" would be the twin-drift class this
//! repository keeps closing, and the whole reason the builder was extracted.

use std::path::{Path, PathBuf};
use std::process::Command;

/// The repository layout, or `None` outside a checkout.
///
/// Split from [`workspace_root`] so the marker discipline can be observed without a test mutating the process
/// environment.
fn locate_layout(root: PathBuf, marker_set: bool) -> Option<PathBuf> {
    if root.join("scripts/lib/coherence_fixture.sh").is_file() {
        return Some(root);
    }
    assert!(
        !marker_set,
        "scripts/lib/coherence_fixture.sh expected under {root:?} but absent while TIANHENG_WORKSPACE_TESTS is \
         set — a governance reaction that quietly does nothing in CI is the shape this family argues against"
    );
    None
}

fn workspace_root() -> Option<PathBuf> {
    locate_layout(
        PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../.."),
        std::env::var_os("TIANHENG_WORKSPACE_TESTS").is_some(),
    )
}

/// Run a command, requiring it to succeed, and return its stdout.
fn must(what: &str, command: &mut Command) -> String {
    let output = command
        .output()
        .unwrap_or_else(|err| panic!("cannot run {what}: {err}"));
    assert!(
        output.status.success(),
        "{what} failed ({}): {}{}",
        output.status,
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    String::from_utf8_lossy(&output.stdout).trim().to_string()
}

/// A scratch root of this test's own, removed first so a killed run leaves nothing behind for the next.
fn scratch(name: &str) -> PathBuf {
    let temp = std::env::temp_dir().join(format!(
        "tianheng-release-coherence-{name}-{}",
        std::process::id()
    ));
    let _ = std::fs::remove_dir_all(&temp);
    std::fs::create_dir_all(&temp).expect("the fixture root is writable");
    temp
}

/// Build a fixture by running `script` with the shared builder sourced, `$1` the scripts directory and `$2` the
/// scratch root. The builder is sourced rather than reimplemented; this function only decides *which* shape.
fn fixture(scripts: &Path, temp: &Path, script: &str) -> String {
    must(
        "the shared coherence fixture builder",
        Command::new("bash")
            .arg("-c")
            .arg(format!(
                r#"set -Eeuo pipefail; . "$1/lib/coherence_fixture.sh"; {script}"#
            ))
            .arg("_")
            .arg(scripts)
            .arg(temp),
    )
}

/// Run the gate over `repo` and return its exit code with the output, for a bound that asserts silence.
fn gate(scripts: &Path, repo: &str) -> (Option<i32>, String) {
    let verdict = Command::new("bash")
        .arg(scripts.join("check_release_coherence.sh"))
        .arg(repo)
        .output()
        .expect("run the release-coherence gate");
    (
        verdict.status.code(),
        format!(
            "{}{}",
            String::from_utf8_lossy(&verdict.stdout),
            String::from_utf8_lossy(&verdict.stderr)
        ),
    )
}

/// `release-coherence/a-dated-release-section-names-a-gate-a-stated-bound`
///
/// `NotAViolation`. A dated section records what was true at that release; rewriting it to satisfy a rule
/// written afterwards would falsify the record, which is why `docs/history/` is left alone too. The bound exists
/// so a reader does not misread the silence as an escape — nine entries in the released `[0.4.0]` name a gate
/// and are meant to keep doing so.
#[test]
fn a_dated_section_naming_a_gate_is_a_stated_bound() {
    let Some(root) = workspace_root() else {
        return;
    };
    let scripts = root.join("scripts");
    let temp = scratch("dated");

    let repo = fixture(
        &scripts,
        &temp,
        r####"repo=$(coherence_fixture_repo "$2" dated)
           coherence_fixture_machinery "$repo"
           python3 - "$repo/CHANGELOG.md" <<'EDIT'
import pathlib, sys
p = pathlib.Path(sys.argv[1])
text = p.read_text()
text = text.replace("## [Unreleased]\n\n", "## [Unreleased]\n\n- An adopter-facing change.\n\n")
text = text.replace("- Release notes.\n",
                    "### Fixed\n- A repair, described by naming `scripts/check_pin_bites.sh`.\n")
p.write_text(text)
EDIT
           coherence_fixture_commit "$repo" 'docs: a dated section names a gate' >/dev/null
           printf '%s\n' "$repo""####,
    );

    let changelog = std::fs::read_to_string(PathBuf::from(&repo).join("CHANGELOG.md"))
        .expect("the fixture's changelog is readable");
    assert!(
        changelog.contains("## [0.2.0] - ") && changelog.contains("`scripts/check_pin_bites.sh`"),
        "the fixture must actually carry the shape this bound is about — a DATED section naming machinery"
    );

    let (code, output) = gate(&scripts, &repo);
    let _ = std::fs::remove_dir_all(&temp);
    assert_eq!(
        code,
        Some(0),
        "the gate must stay silent about a dated section naming machinery — that is the declared bound, and a \
         refusal here would mean the scope had changed and this citation should be retired. Got: {output}"
    );
}

/// `release-coherence/machinery-the-judged-repository-tracks-by-nothing-a-stated-bound`
///
/// `UnderReacts`, owned by the engine. The enumeration is `git ls-files scripts/`, so an untracked `scripts/`
/// reads as absent and a citation of it goes unseen. Closing it means judging worktree content, which this
/// repository's gates are held *not* to do — the larger error, so the blindness is declared instead.
#[test]
fn machinery_tracked_by_nothing_is_a_stated_bound() {
    let Some(root) = workspace_root() else {
        return;
    };
    let scripts = root.join("scripts");
    let temp = scratch("untracked");

    let repo = fixture(
        &scripts,
        &temp,
        r#"repo=$(coherence_fixture_repo "$2" untracked)
           coherence_fixture_development_changelog "$repo" 0.2.0
           coherence_fixture_unreleased_body "$repo" '### Fixed
- A repair, described by naming `scripts/check_pin_bites.sh`.'
           coherence_fixture_commit "$repo" 'docs: name a gate before it is tracked' >/dev/null
           coherence_fixture_machinery "$repo"
           printf '%s\n' "$repo""#,
    );

    let tracked = must(
        "the fixture's tracked scripts/ enumeration",
        Command::new("git").args(["-C", &repo, "ls-files", "scripts/"]),
    );
    assert!(
        tracked.is_empty()
            && PathBuf::from(&repo)
                .join("scripts/check_pin_bites.sh")
                .is_file(),
        "the fixture must hold the machinery in the WORKTREE and not in the index, or it demonstrates nothing"
    );

    let (code, output) = gate(&scripts, &repo);
    let _ = std::fs::remove_dir_all(&temp);
    assert_eq!(
        code,
        Some(0),
        "the gate must stay silent about machinery no commit tracks — that is the declared bound. Got: {output}"
    );
}

/// `release-coherence/a-basename-an-entry-writes-for-another-reason-a-stated-bound`
///
/// `OverReacts`. A word is matched against basenames as well as paths, because the document cites both forms.
/// An entry naming a file of its own whose basename this repository also tracks under `scripts/` is refused,
/// and the entry is innocent. The direction is the safe one — an author meets a refusal to argue with — and
/// narrowing it means deciding which of two files a bare name meant, a judgement about the sentence.
#[test]
fn a_colliding_basename_is_a_stated_bound() {
    let Some(root) = workspace_root() else {
        return;
    };
    let scripts = root.join("scripts");
    let temp = scratch("collide");

    let repo = fixture(
        &scripts,
        &temp,
        r#"repo=$(coherence_fixture_repo "$2" collide)
           coherence_fixture_development_changelog "$repo" 0.2.0
           mkdir -p "$repo/scripts"
           printf '#!/usr/bin/env bash\nexit 0\n' >"$repo/scripts/publish.sh"
           coherence_fixture_unreleased_body "$repo" '### Fixed
- Adopters run their own `publish.sh` after upgrading.'
           coherence_fixture_commit "$repo" 'docs: write a name the repository also tracks' >/dev/null
           printf '%s\n' "$repo""#,
    );

    let (code, output) = gate(&scripts, &repo);
    let _ = std::fs::remove_dir_all(&temp);
    assert_eq!(
        code,
        Some(1),
        "the gate must refuse an innocent entry whose word collides with a tracked basename — that is the \
         declared over-reaction, and silence here would mean the bound had closed. Got: {output}"
    );
    assert!(
        output.contains("publish.sh"),
        "the refusal must name the colliding word, or it demonstrates some other refusal: {output}"
    );
}

/// `release-coherence/a-name-reached-only-through-a-url-a-stated-bound`
///
/// `UnderReacts`, owned by the engine. A word is a maximal run of path characters, so a scheme and host fuse
/// with the path into one run that equals no tracked name. Splitting a URL into its path would make the
/// reaction judge a foreign host's layout as though it were this repository's.
#[test]
fn a_name_reached_only_through_a_url_is_a_stated_bound() {
    let Some(root) = workspace_root() else {
        return;
    };
    let scripts = root.join("scripts");
    let temp = scratch("url");

    let repo = fixture(
        &scripts,
        &temp,
        r#"repo=$(coherence_fixture_repo "$2" url)
           coherence_fixture_development_changelog "$repo" 0.2.0
           coherence_fixture_machinery "$repo"
           coherence_fixture_unreleased_body "$repo" '### Fixed
- See https://github.com/tacticaldoll/tianheng/blob/main/scripts/check_pin_bites.sh for the gate.'
           coherence_fixture_commit "$repo" 'docs: reach a gate only through a URL' >/dev/null
           printf '%s\n' "$repo""#,
    );

    let (code, output) = gate(&scripts, &repo);
    let _ = std::fs::remove_dir_all(&temp);
    assert_eq!(
        code,
        Some(0),
        "the gate must stay silent about a name reached only through a URL — that is the declared bound. \
         Got: {output}"
    );
}

/// `release-coherence/a-heading-inside-a-fenced-code-block-a-stated-bound`
///
/// `UnderReacts`, owned by the engine. The reaction walks the document's line grammar and does not track
/// fences, so a `### ` line inside a fenced block sets the heading in force — and can name the one exempt
/// heading, hiding every entry after it. Latent rather than live: this repository's changelog carries no
/// fenced block at all.
#[test]
fn a_heading_inside_a_fenced_block_is_a_stated_bound() {
    let Some(root) = workspace_root() else {
        return;
    };
    let scripts = root.join("scripts");
    let temp = scratch("fenced");

    let repo = fixture(
        &scripts,
        &temp,
        r####"repo=$(coherence_fixture_repo "$2" fenced)
           coherence_fixture_development_changelog "$repo" 0.2.0
           coherence_fixture_machinery "$repo"
           coherence_fixture_unreleased_body "$repo" '### Fixed
- A repair.

```
### Self-governance
```

- A later repair naming `scripts/check_pin_bites.sh`.'
           coherence_fixture_commit "$repo" 'docs: put a heading inside a fence' >/dev/null
           printf '%s\n' "$repo""####,
    );

    let (code, output) = gate(&scripts, &repo);
    let _ = std::fs::remove_dir_all(&temp);
    assert_eq!(
        code,
        Some(0),
        "the gate must stay silent when a fenced heading reattributes a later entry — that is the declared \
         bound. Got: {output}"
    );
}

/// The marker discipline itself, observed rather than trusted.
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
