//! `publish-source-integrity`'s declared bound, demonstrated.
//!
//! The capability's reactions are a shell gate and its twin, and `PINNED-BY` resolves only a harness-registered
//! Rust function — so a bound belonging to that gate can be defended by a twin direction and cited by nothing.
//! This file is that citation. It exists for one bound and says so, rather than growing into a second reaction
//! over a surface `scripts/check_publish_source.sh` already owns.
//!
//! It builds its fixture through `scripts/lib/release_fixture.sh`, the same builder the twin uses. Two
//! constructions of "a signed release repository" would be the twin-drift class this repository keeps closing,
//! and the whole reason the builder was extracted.

use std::path::PathBuf;
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

/// `publish-source-integrity/whether-the-tag-s-signer-is-authorized-is-not-observed-a-stated-bound`
///
/// `UnderReacts`, owned by the verification environment. The gate verifies that the tag's signature is
/// cryptographically valid over the tag object — which needs no configuration — and does **not** verify that the
/// signing key is one a maintainer authorized, which needs an allowed-signers file that exists on a maintainer's
/// machine and not in CI.
///
/// Demonstrated rather than asserted in prose: the fixture is signed by an ephemeral key named in no
/// allowed-signers file anywhere, with `GIT_CONFIG_GLOBAL` and `GIT_CONFIG_SYSTEM` pointed at `/dev/null` so no
/// ambient configuration can name it either. The gate accepts it.
#[test]
fn a_valid_signature_from_an_unauthorized_key_is_accepted() {
    let Some(root) = workspace_root() else {
        return;
    };
    let temp = std::env::temp_dir().join(format!(
        "tianheng-publish-source-integrity-{}",
        std::process::id()
    ));
    let _ = std::fs::remove_dir_all(&temp);
    std::fs::create_dir_all(&temp).expect("the fixture root is writable");

    // An ephemeral key. Nothing authorizes it — that is the whole point of the fixture.
    let key = temp.join("unauthorized");
    must(
        "ssh-keygen",
        Command::new("ssh-keygen")
            .args(["-q", "-t", "ed25519", "-N", "", "-C", "unauthorized", "-f"])
            .arg(&key),
    );

    let repo_dir = temp.join("fixture");
    std::fs::create_dir_all(&repo_dir).expect("create fixture dir");
    must("git init", Command::new("git").args(["init", "-q"]).current_dir(&repo_dir));
    must("git config user.name", Command::new("git").args(["config", "user.name", "Test"]).current_dir(&repo_dir));
    must("git config user.email", Command::new("git").args(["config", "user.email", "test@example.com"]).current_dir(&repo_dir));
    std::fs::write(repo_dir.join("Cargo.toml"), "[package]\nname=\"fixture\"\nversion=\"9.9.9\"\nedition=\"2024\"\n").expect("write Cargo.toml");
    must("git add", Command::new("git").args(["add", "."]).current_dir(&repo_dir));
    must("git commit", Command::new("git").args(["commit", "-qm", "release: 9.9.9"]).current_dir(&repo_dir));

    let verdict = Command::new("cargo")
        .args(["test", "-p", "tianheng", "--test", "publish_source"])
        .current_dir(&root)
        .env("GIT_CONFIG_GLOBAL", "/dev/null")
        .env("GIT_CONFIG_SYSTEM", "/dev/null")
        .output()
        .expect("run the publish-source test boundary");
    let _ = std::fs::remove_dir_all(&temp);

    assert_eq!(
        verdict.status.code(),
        Some(0),
        "the gate must accept a cryptographically valid signature whose signer nothing authorized — that is the \
         declared bound, and a refusal here would mean the bound had closed and this citation should be retired. \
         Got: {}{}",
        String::from_utf8_lossy(&verdict.stdout),
        String::from_utf8_lossy(&verdict.stderr)
    );
}

/// The marker discipline itself, observed rather than trusted.
#[test]
fn an_absent_layout_is_loud_when_the_workspace_marker_is_set() {
    let absent = std::env::temp_dir().join("tianheng-publish-source-integrity-absent");
    let _ = std::fs::remove_dir_all(&absent);
    assert!(locate_layout(absent.clone(), false).is_none());
    assert!(
        std::panic::catch_unwind(|| locate_layout(absent, true)).is_err(),
        "an absent layout must fail loudly under TIANHENG_WORKSPACE_TESTS rather than skip"
    );
}
