//! The repository shape `release_coherence_gate` judges, built hermetically.
//!
//! The compare-link constant comes from that gate rather than being spelled again here: it is the
//! authority on the link shape it accepts, and a second spelling would be two facts that must agree.

use std::path::{Path, PathBuf};

use super::{commit, write};
use crate::hermetic_git::fixture as run;
use crate::release_coherence_gate::COMPARE;

/// A repository in the shape `release_coherence_gate` reads, built hermetically.
///
/// A fixture that inherits the judged machine cannot demonstrate a refusal, because the shape it builds is not
/// the shape it named — measured on the sibling publish gate, where ambient signing configuration turned an
/// intentionally unsigned tag into a signed one.
pub struct Fixture {
    /// The fixture repository's working tree.
    pub repo: PathBuf,
}

/// Write a workspace manifest, its members and a matching `Cargo.lock`, all naming one version.
pub fn workspace_files(repo: &Path, version: &str) {
    write(
        repo.join("Cargo.toml"),
        &format!(
            "[workspace]\nmembers = [\"crates/xuanji\", \"crates/tianheng\", \"crates/renamed-dir\"]\n\n\
             [workspace.package]\nversion = \"{version}\"\n\n\
             [workspace.dependencies]\nxuanji = {{ path = \"crates/xuanji\", version = \"{version}\" }}\n"
        ),
    );
    // `xuanji` publishes and `tianheng` does not, so a fixture exercises both sides of the criterion the
    // machinery corpus reads from the manifests. Each member carries a `src/lib.rs`, because a workspace
    // cargo cannot load is one this gate cannot enumerate — the fixture is a real workspace or it is not
    // evidence about one.
    for (package, publishes) in [("xuanji", true), ("tianheng", false)] {
        let publish = if publishes { "" } else { "publish = false\n" };
        write(
            repo.join(format!("crates/{package}/Cargo.toml")),
            &format!(
                "[package]\nname = \"{package}\"\nversion.workspace = true\nedition = \"2024\"\n{publish}"
            ),
        );
        write(repo.join(format!("crates/{package}/src/lib.rs")), "");
    }
    // **A member whose directory is not its package name.** Without it, the fixture's two sides agree by
    // construction — every member sits at `crates/<name>/` — so a corpus that derived the directory from the
    // package name would pass every row here while being wrong about any workspace that does not. It is
    // unpublished, so its files must reach the machinery set: if the derivation regresses, this member
    // contributes nothing and a changelog naming its gate reports clean.
    write(
        repo.join("crates/renamed-dir/Cargo.toml"),
        "[package]\nname = \"machinery-under-another-name\"\nversion.workspace = true\n\
         edition = \"2024\"\npublish = false\n",
    );
    write(repo.join("crates/renamed-dir/src/lib.rs"), "");
    write(
        repo.join("crates/renamed-dir/tests/renamed_gate.rs"),
        "#[test]\nfn t() {}\n",
    );
    let minor = version.rsplit_once('.').map(|(h, _)| h).unwrap_or(version);
    // The example package the fixture carries, named through a binding like the members above rather than
    // as one path literal: a literal here reads as a reference into *this* repository, which the reference
    // gate then reports as stale — the path belongs to the fixture, not to the tree being judged.
    let example = "adopter";
    write(
        repo.join(format!("examples/{example}/Cargo.toml")),
        &format!(
            "[package]\nname = \"{example}\"\nversion = \"0.0.0\"\nedition = \"2024\"\n\n\
             [dependencies]\nxuanji = \"{minor}\"\n"
        ),
    );
    write(
        repo.join("Cargo.lock"),
        &format!(
            "version = 4\n\n[[package]]\nname = \"tianheng\"\nversion = \"{version}\"\n\n\
             [[package]]\nname = \"xuanji\"\nversion = \"{version}\"\n\n\
             [[package]]\nname = \"machinery-under-another-name\"\nversion = \"{version}\"\n"
        ),
    );
}

/// Write a changelog in the development shape: an `[Unreleased]` section, carrying an adopter-facing item
/// only when asked, so its absence can be refused.
pub fn development_changelog(repo: &Path, version: &str, with_item: bool) {
    let item = if with_item {
        "- An adopter-facing change.\n\n"
    } else {
        ""
    };
    write(
        repo.join("CHANGELOG.md"),
        &format!(
            "# Changelog\n\n## [Unreleased]\n\n{item}[Unreleased]: {COMPARE}/v{version}...HEAD\n"
        ),
    );
}

/// Write a changelog in the release shape: a dated section for `version`, with the link block naming
/// `previous`.
pub fn release_changelog(repo: &Path, version: &str, previous: &str) {
    // The same day the fixture's commits carry, from the one owner — this section and those commits are the
    // two halves `release-coherence` compares.
    let day = crate::hermetic_git::FIXTURE_DAY;
    write(
        repo.join("CHANGELOG.md"),
        &format!(
            "# Changelog\n\n## [Unreleased]\n\n## [{version}] - {day}\n\n- Release notes.\n\n\
             [Unreleased]: {COMPARE}/v{version}...HEAD\n[{version}]: {COMPARE}/v{previous}...v{version}\n"
        ),
    );
}

/// A repository released at `version` over a `0.1.0` predecessor.
pub fn build(root: &Path, name: &str, version: &str) -> Fixture {
    let repo = root.join(name);
    std::fs::create_dir_all(&repo).expect("the fixture root is writable");
    run(&repo, "git", &["init", "-q", "-b", "main"]);
    run(
        &repo,
        "git",
        &["config", "user.name", "Release Coherence Test"],
    );
    run(
        &repo,
        "git",
        &["config", "user.email", "release-coherence@example.invalid"],
    );
    run(&repo, "git", &["config", "commit.gpgsign", "false"]);

    workspace_files(&repo, "0.1.0");
    release_changelog(&repo, "0.1.0", "0.0.0");
    // The predecessor uses the retired subject to prove the migration can read the existing spine.
    commit(&repo, "release: 0.1.0");

    workspace_files(&repo, version);
    release_changelog(&repo, version, "0.1.0");
    commit(&repo, &crate::release_subject::canonical(version));

    Fixture { repo }
}
