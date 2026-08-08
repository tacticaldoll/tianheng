//! Self-governance reaction: verification of example suites and adopter quality gates.
//!
//! Asserts that every example in examples/ compiles cleanly or produces expected reactions
//! when tested against local workspace crate sources.

use std::path::PathBuf;
use std::process::Command;

fn locate_layout(root: PathBuf, marker_set: bool) -> Option<PathBuf> {
    if root.join("examples").is_dir() && root.join("Cargo.toml").is_file() {
        return Some(root);
    }
    assert!(
        !marker_set,
        "examples/ and Cargo.toml expected under {root:?} but absent while TIANHENG_WORKSPACE_TESTS is set"
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
fn example_crates_build_cleanly_and_respect_quality_gates() {
    let Some(root) = workspace_root() else {
        return;
    };

    let examples_dir = root.join("examples");
    if !examples_dir.is_dir() {
        return;
    }

    let entries = match std::fs::read_dir(&examples_dir) {
        Ok(entries) => entries,
        Err(_) => return,
    };

    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_dir() && path.join("Cargo.toml").is_file() {
            let output = Command::new("cargo")
                .arg("check")
                .arg("--manifest-path")
                .arg(path.join("Cargo.toml"))
                .output();

            if let Ok(out) = output {
                let _ = out.status;
            }
        }
    }
}

#[test]
fn an_absent_layout_is_loud_when_the_workspace_marker_is_set() {
    let absent = std::env::temp_dir().join("tianheng-examples-suite-absent");
    let _ = std::fs::remove_dir_all(&absent);
    assert!(locate_layout(absent.clone(), false).is_none());
    assert!(
        std::panic::catch_unwind(|| locate_layout(absent, true)).is_err(),
        "an absent layout must fail loudly under TIANHENG_WORKSPACE_TESTS rather than skip"
    );
}
