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

/// `release-coherence/a-dated-release-section-naming-machinery-is-not-observed-a-stated-bound`
///
/// `NotAViolation`. A dated section records what was true at that release; rewriting it to satisfy a rule
/// written afterwards would falsify the record, which is why `docs/history/` is left alone too. The bound exists
/// so a reader does not misread the silence as an escape — five entries in the released `[0.4.0]` name a gate
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

/// `release-coherence/a-gate-named-as-bare-prose-is-not-observed-a-stated-bound`
///
/// `UnderReacts`, owned by the engine. Recognition is by token — a backticked span — so an entry naming a gate
/// as ordinary prose leaks exactly what the rule exists to stop and nothing reacts. Widening to a bare substring
/// would fire on any sentence containing the characters, trading a declared blindness for an undeclared
/// false-positive surface, which is the wrong direction under the Core Contract.
#[test]
fn a_gate_named_as_bare_prose_is_a_stated_bound() {
    let Some(root) = workspace_root() else {
        return;
    };
    let scripts = root.join("scripts");
    let temp = scratch("prose");

    let repo = fixture(
        &scripts,
        &temp,
        r#"repo=$(coherence_fixture_repo "$2" prose)
           coherence_fixture_development_changelog "$repo" 0.2.0
           coherence_fixture_machinery "$repo"
           coherence_fixture_unreleased_body "$repo" '### Fixed
- A repair to the check_pin_bites.sh gate, written as prose rather than as a token.'
           coherence_fixture_commit "$repo" 'docs: name a gate as bare prose' >/dev/null
           printf '%s\n' "$repo""#,
    );

    let changelog = std::fs::read_to_string(PathBuf::from(&repo).join("CHANGELOG.md"))
        .expect("the fixture's changelog is readable");
    assert!(
        changelog.contains(" check_pin_bites.sh gate")
            && !changelog.contains("`check_pin_bites.sh`"),
        "the fixture must name the gate WITHOUT backticks, or it demonstrates the reaction rather than the bound"
    );

    let (code, output) = gate(&scripts, &repo);
    let _ = std::fs::remove_dir_all(&temp);
    assert_eq!(
        code,
        Some(0),
        "the gate must stay silent about a gate named as bare prose — that is the declared bound. Got: {output}"
    );
}

/// `release-coherence/machinery-tracked-by-nothing-is-not-observed-a-stated-bound`
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
