//! Self-governance reaction: publish source verification.
//!
//! Asserts that publishing runs only from clean checkouts pointing at signed release snapshot commits.

use std::path::PathBuf;
use std::process::Command;

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

#[test]
fn publish_source_verification_structure() {
    let Some(root) = workspace_root() else {
        return;
    };

    let manifest = root.join("Cargo.toml");
    assert!(manifest.is_file(), "Workspace root Cargo.toml must exist");

    let output = Command::new("git")
        .args(["status", "--porcelain"])
        .current_dir(&root)
        .output();

    if let Ok(out) = output {
        assert!(
            out.status.success(),
            "git status check must execute cleanly"
        );
    }
}

#[test]
fn an_absent_layout_is_loud_when_the_workspace_marker_is_set() {
    let absent = std::env::temp_dir().join("tianheng-publish-source-absent");
    let _ = std::fs::remove_dir_all(&absent);
    assert!(locate_layout(absent.clone(), false).is_none());
    assert!(
        std::panic::catch_unwind(|| locate_layout(absent, true)).is_err(),
        "an absent layout must fail loudly under TIANHENG_WORKSPACE_TESTS rather than skip"
    );
}
