//! The repository shape a publish is allowed to run from, built hermetically.
//!
//! Every step is hermetic for a measured reason: a fixture that inherited the machine's signing
//! configuration once turned an intentionally unsigned tag into a signed one, which made a refusal
//! impossible to demonstrate.

use std::path::{Path, PathBuf};

use super::{commit, write};
use crate::hermetic_git::fixture as run;

/// A repository in the exact shape a publish runs from: `main` pushed to a bare remote, its tip a
/// `chore(release): <version>` snapshot, tagged with a signed annotated tag, worktree clean.
///
/// The caller owns the root and its cleanup, because a builder that also decided lifetime would make a
/// caller's single guard two.
pub struct Fixture {
    /// The working tree a publish would run from.
    pub repo: PathBuf,
    /// The bare remote `main` was pushed to.
    pub remote: PathBuf,
    /// The signing key the annotated tag was made with.
    pub key: PathBuf,
}

/// Build a [`Fixture`] under `root`, in the shape a publish is allowed to run from.
///
/// Every step is hermetic: a fixture that inherited the machine's signing configuration once turned an
/// intentionally unsigned tag into a signed one, which made a refusal impossible to demonstrate.
pub fn build(root: &Path, name: &str, version: &str) -> Fixture {
    let repo = root.join(name);
    let remote = root.join(format!("{name}-origin.git"));
    let key = root.join(format!("{name}-key"));
    std::fs::create_dir_all(&repo).expect("the fixture root is writable");

    run(
        root,
        "ssh-keygen",
        &[
            "-q",
            "-t",
            "ed25519",
            "-N",
            "",
            "-C",
            "fixture",
            "-f",
            &key.display().to_string(),
        ],
    );
    run(
        root,
        "git",
        &["init", "-q", "--bare", &remote.display().to_string()],
    );
    run(&repo, "git", &["init", "-q", "-b", "main"]);
    for (k, v) in [
        ("user.name", "Publish Source Test"),
        ("user.email", "publish-source@example.invalid"),
        ("gpg.format", "ssh"),
        ("commit.gpgsign", "false"),
        ("tag.gpgsign", "false"),
        ("tag.forceSignAnnotated", "false"),
    ] {
        run(&repo, "git", &["config", k, v]);
    }
    run(
        &repo,
        "git",
        &["config", "user.signingkey", &key.display().to_string()],
    );

    write(
        repo.join("Cargo.toml"),
        &format!("[workspace]\nmembers = []\n\n[workspace.package]\nversion = \"{version}\"\n"),
    );
    commit(&repo, &crate::release_subject::canonical(version));
    run(
        &repo,
        "git",
        &[
            "tag",
            "-s",
            &format!("v{version}"),
            "-m",
            &format!("v{version}"),
        ],
    );
    run(
        &repo,
        "git",
        &["remote", "add", "origin", &remote.display().to_string()],
    );
    run(&repo, "git", &["push", "-q", "origin", "main"]);
    // **The tag is pushed too, because the accepted shape is *the tagged commit on the remote's main*.** This
    // pushed `main` alone, so the direction every refusal is measured against was itself the unpushed-tag
    // case, and every tag read in the gate was local. A fixture that cannot express the shape cannot hold the
    // claim; `a_tag_that_is_not_on_the_remote_is_a_violation` is the shape it could not express.
    run(
        &repo,
        "git",
        &["push", "-q", "origin", &format!("refs/tags/v{version}")],
    );

    Fixture { repo, remote, key }
}
