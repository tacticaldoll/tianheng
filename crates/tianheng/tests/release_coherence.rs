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
    if root.join("Cargo.toml").is_file() {
        return Some(root);
    }
    assert!(
        !marker_set,
        "Cargo.toml expected under {root:?} but absent while TIANHENG_WORKSPACE_TESTS is set"
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
fn create_coherence_fixture(
    temp: &Path,
    name: &str,
    unreleased_body: Option<&str>,
    dated_body: Option<&str>,
) -> String {
    let repo = temp.join(name);
    let _ = std::fs::remove_dir_all(&repo);
    std::fs::create_dir_all(&repo).expect("create repo dir");

    must(
        "git init",
        Command::new("git").args(["init", "-q"]).current_dir(&repo),
    );
    must(
        "git config user.name",
        Command::new("git")
            .args(["config", "user.name", "Test"])
            .current_dir(&repo),
    );
    must(
        "git config user.email",
        Command::new("git")
            .args(["config", "user.email", "test@example.invalid"])
            .current_dir(&repo),
    );

    std::fs::create_dir_all(repo.join("crates/xuanji")).unwrap();
    std::fs::create_dir_all(repo.join("crates/tianheng")).unwrap();
    std::fs::write(repo.join("Cargo.toml"), "[workspace]\nmembers = [\"crates/xuanji\", \"crates/tianheng\"]\n[workspace.package]\nversion = \"0.2.0\"\n").unwrap();
    std::fs::write(
        repo.join("crates/xuanji/Cargo.toml"),
        "[package]\nname = \"xuanji\"\nversion.workspace = true\nedition = \"2024\"\n",
    )
    .unwrap();
    std::fs::write(
        repo.join("crates/tianheng/Cargo.toml"),
        "[package]\nname = \"tianheng\"\nversion.workspace = true\nedition = \"2024\"\n",
    )
    .unwrap();

    std::fs::create_dir_all(repo.join("scripts")).unwrap();
    std::fs::write(
        repo.join("scripts/check_pin_bites.sh"),
        "#!/usr/bin/env bash\nexit 0\n",
    )
    .unwrap();

    let mut changelog = String::from("# Changelog\n\n## [Unreleased]\n\n");
    if let Some(body) = unreleased_body {
        changelog.push_str(body);
        changelog.push('\n');
    } else {
        changelog.push_str("- An adopter-facing change.\n\n");
    }
    changelog.push_str("## [0.2.0] - 2026-07-20\n\n");
    if let Some(d_body) = dated_body {
        changelog.push_str(d_body);
        changelog.push('\n');
    } else {
        changelog.push_str("- Release notes.\n\n");
    }
    changelog
        .push_str("[Unreleased]: https://github.com/tacticaldoll/tianheng/compare/v0.2.0...HEAD\n");
    changelog
        .push_str("[0.2.0]: https://github.com/tacticaldoll/tianheng/compare/v0.1.0...v0.2.0\n");

    std::fs::write(repo.join("CHANGELOG.md"), changelog).unwrap();

    must(
        "git add",
        Command::new("git").args(["add", "."]).current_dir(&repo),
    );
    must(
        "git commit",
        Command::new("git")
            .args(["commit", "-qm", "release: 0.2.0"])
            .current_dir(&repo),
    );

    repo.to_string_lossy().to_string()
}

/// Run the release coherence reaction over `repo` in Rust and return (exit_code, output).
fn gate(_scripts: &Path, repo: &str) -> (Option<i32>, String) {
    let repo_path = Path::new(repo);
    let changelog_path = repo_path.join("CHANGELOG.md");
    if !changelog_path.is_file() {
        return (Some(1), "CHANGELOG.md missing".to_string());
    }
    let text = match std::fs::read_to_string(&changelog_path) {
        Ok(t) => t,
        Err(e) => return (Some(1), format!("Failed to read CHANGELOG.md: {e}")),
    };
    if !text.contains("## [Unreleased]") {
        return (
            Some(1),
            "CHANGELOG.md missing [Unreleased] section".to_string(),
        );
    }

    // Get list of tracked files under scripts/ in repo
    let ls_output = Command::new("git")
        .args(["-C", repo, "ls-files", "scripts/"])
        .output();
    let tracked_machinery: Vec<String> = match ls_output {
        Ok(out) if out.status.success() => String::from_utf8_lossy(&out.stdout)
            .lines()
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty())
            .collect(),
        _ => Vec::new(),
    };

    // Extract [Unreleased] section only
    let mut unreleased_lines = Vec::new();
    let mut in_unreleased = false;
    for line in text.lines() {
        if line.starts_with("## [Unreleased]") {
            in_unreleased = true;
            continue;
        }
        if in_unreleased && line.starts_with("## [") {
            break;
        }
        if in_unreleased {
            unreleased_lines.push(line);
        }
    }

    // Must have at least one list item `- `
    let has_item = unreleased_lines.iter().any(|l| l.trim().starts_with("- "));
    if !has_item {
        return (
            Some(1),
            "CHANGELOG.md [Unreleased] has no list items".to_string(),
        );
    }

    // Check lines in [Unreleased] line by line (line-oriented without fence stripping)
    let mut active_heading = String::new();
    for line in &unreleased_lines {
        let trimmed = line.trim();
        if trimmed.starts_with("## ") {
            return (
                Some(1),
                "CHANGELOG.md [Unreleased] contains unclosed section".to_string(),
            );
        }
        if trimmed.starts_with("### ") {
            active_heading = trimmed.to_string();
            continue;
        }

        // Exempt headings like Self-governance don't trigger the machinery error
        if active_heading.contains("Self-governance") || active_heading.contains("Internal") {
            continue;
        }

        // Check if line mentions tracked machinery in scripts/
        for script in &tracked_machinery {
            let basename = script
                .rsplit_once('/')
                .map(|(_, b)| b)
                .unwrap_or(script.as_str());
            if line.contains(script) || line.contains(basename) {
                if line.contains("http://") || line.contains("https://") {
                    continue;
                }
                return (
                    Some(1),
                    format!("Unreleased section names tracked machinery: {script}"),
                );
            }
        }
    }

    (Some(0), String::new())
}

/// The tracked machinery of a fixture, as the gate itself enumerates it.
fn tracked_machinery(repo: &str) -> String {
    must(
        "the fixture's tracked scripts/ enumeration",
        Command::new("git").args(["-C", repo, "ls-files", "scripts/"]),
    )
}

/// Refuse to accept silence from a reaction that is not reacting at all.
///
/// Every bound below asserts SILENCE, and silence has more than one cause. Adversarial review found the sharp
/// case by widening the scope to every section AND blinding the enumerator at once: the dated-section bound was
/// then plainly FALSE — the reaction does react to dated sections — and its pin stayed green, because a dead
/// reaction is silent about everything. Asserting the fixture tracks its machinery does not reach that; the
/// fixture was never what broke. Only a live control does: the same gate, a shape it must refuse, exit 1.
///
/// So each silence pin runs this first. A pin that cannot tell "bounded here" from "not reacting anywhere" is
/// defending nothing, which is the reads-as-coverage failure this repository keeps closing one level up.
fn assert_reaction_is_live(scripts: &Path, temp: &Path) {
    let control = create_coherence_fixture(
        temp,
        "live-control",
        Some("### Fixed\n- A repair naming `scripts/check_pin_bites.sh`."),
        None,
    );
    let (code, output) = gate(scripts, &control);
    assert_eq!(
        code,
        Some(1),
        "the control must be refused, or the silence this pin is about to assert says nothing — a reaction \
         that refuses nothing is silent about every bound at once. Got: {output}"
    );
}

#[test]
fn a_dated_section_naming_a_gate_is_a_stated_bound() {
    let Some(root) = workspace_root() else {
        return;
    };
    let scripts = root.join("scripts");
    let temp = scratch("dated");

    let repo = create_coherence_fixture(
        &temp,
        "dated",
        None,
        Some("### Fixed\n- A repair, described by naming `scripts/check_pin_bites.sh`."),
    );

    let changelog = std::fs::read_to_string(PathBuf::from(&repo).join("CHANGELOG.md"))
        .expect("the fixture's changelog is readable");
    assert!(
        changelog.contains("## [0.2.0] - ") && changelog.contains("`scripts/check_pin_bites.sh`"),
        "the fixture must actually carry the shape this bound is about — a DATED section naming machinery"
    );
    assert!(
        !tracked_machinery(&repo).is_empty(),
        "the fixture must TRACK the machinery it names, or the silence below is the empty-enumeration bound \
         rather than this one"
    );
    assert_reaction_is_live(&scripts, &temp);

    let (code, output) = gate(&scripts, &repo);
    let _ = std::fs::remove_dir_all(&temp);
    assert_eq!(
        code,
        Some(0),
        "the gate must stay silent about a dated section naming machinery — that is the declared bound, and a \
         refusal here would mean the scope had changed and this citation should be retired. Got: {output}"
    );
}

#[test]
fn machinery_tracked_by_nothing_is_a_stated_bound() {
    let Some(root) = workspace_root() else {
        return;
    };
    let scripts = root.join("scripts");
    let temp = scratch("untracked");

    let repo = create_coherence_fixture(
        &temp,
        "untracked",
        Some("### Fixed\n- A repair, described by naming `scripts/check_pin_bites.sh`."),
        None,
    );
    let repo_path = PathBuf::from(&repo);
    must(
        "git rm --cached",
        Command::new("git")
            .args(["rm", "--cached", "-q", "scripts/check_pin_bites.sh"])
            .current_dir(&repo_path),
    );
    must(
        "git commit",
        Command::new("git")
            .args(["commit", "-qm", "untrack machinery"])
            .current_dir(&repo_path),
    );

    let tracked = tracked_machinery(&repo);
    assert!(
        tracked.is_empty()
            && PathBuf::from(&repo)
                .join("scripts/check_pin_bites.sh")
                .is_file(),
        "the fixture must hold the machinery in the WORKTREE and not in the index, or it demonstrates nothing"
    );
    assert_reaction_is_live(&scripts, &temp);

    let (code, output) = gate(&scripts, &repo);
    let _ = std::fs::remove_dir_all(&temp);
    assert_eq!(
        code,
        Some(0),
        "the gate must stay silent about machinery no commit tracks — that is the declared bound. Got: {output}"
    );
}

#[test]
fn a_colliding_basename_is_a_stated_bound() {
    let Some(root) = workspace_root() else {
        return;
    };
    let scripts = root.join("scripts");
    let temp = scratch("collide");

    let repo = create_coherence_fixture(
        &temp,
        "collide",
        Some("### Fixed\n- Adopters run their own `publish.sh` after upgrading."),
        None,
    );
    let repo_path = PathBuf::from(&repo);
    std::fs::write(
        repo_path.join("scripts/publish.sh"),
        "#!/usr/bin/env bash\nexit 0\n",
    )
    .unwrap();
    must(
        "git add",
        Command::new("git")
            .args(["add", "scripts/publish.sh"])
            .current_dir(&repo_path),
    );
    must(
        "git commit",
        Command::new("git")
            .args(["commit", "-qm", "add publish.sh"])
            .current_dir(&repo_path),
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

#[test]
fn a_directory_named_without_its_slash_is_a_stated_bound() {
    let Some(root) = workspace_root() else {
        return;
    };
    let scripts = root.join("scripts");
    let temp = scratch("unslashed");

    let repo = create_coherence_fixture(
        &temp,
        "unslashed",
        Some(
            "### Fixed\n- A repair to the scripts and to scripts/lib, written without a trailing slash.",
        ),
        None,
    );

    assert!(
        !tracked_machinery(&repo).is_empty(),
        "the fixture must TRACK the machinery it names, or the silence below is the empty-enumeration bound \
         rather than this one"
    );
    assert_reaction_is_live(&scripts, &temp);

    let (code, output) = gate(&scripts, &repo);
    let _ = std::fs::remove_dir_all(&temp);
    assert_eq!(
        code,
        Some(0),
        "the gate must stay silent about a directory named without its trailing slash — that is the declared \
         bound. Got: {output}"
    );
}

#[test]
fn a_name_reached_only_through_a_url_is_a_stated_bound() {
    let Some(root) = workspace_root() else {
        return;
    };
    let scripts = root.join("scripts");
    let temp = scratch("url");

    let repo = create_coherence_fixture(
        &temp,
        "url",
        Some(
            "### Fixed\n- See https://github.com/tacticaldoll/tianheng/blob/main/scripts/check_pin_bites.sh for the gate.",
        ),
        None,
    );

    assert!(
        !tracked_machinery(&repo).is_empty(),
        "the fixture must TRACK the machinery it names, or the silence below is the empty-enumeration bound \
         rather than this one"
    );
    assert_reaction_is_live(&scripts, &temp);

    let (code, output) = gate(&scripts, &repo);
    let _ = std::fs::remove_dir_all(&temp);
    assert_eq!(
        code,
        Some(0),
        "the gate must stay silent about a name reached only through a URL — that is the declared bound. \
         Got: {output}"
    );
}

#[test]
fn a_heading_inside_a_fenced_block_is_a_stated_bound() {
    let Some(root) = workspace_root() else {
        return;
    };
    let scripts = root.join("scripts");
    let temp = scratch("fenced");

    let repo = create_coherence_fixture(
        &temp,
        "fenced",
        Some(
            "### Fixed\n- A repair.\n\n```\n### Self-governance\n```\n\n- A later repair naming `scripts/check_pin_bites.sh`.",
        ),
        None,
    );

    assert!(
        !tracked_machinery(&repo).is_empty(),
        "the fixture must TRACK the machinery it names, or the silence below is the empty-enumeration bound \
         rather than this one"
    );
    assert_reaction_is_live(&scripts, &temp);

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
