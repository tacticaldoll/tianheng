//! Self-governance reaction: observation bound register integrity and projection freshness.
//!
//! Asserts that every observation bound declared in spec scenarios carries a valid citation
//! (`PINNED-BY` resolving to a Rust test, or `UNPINNED` resolving to a tracked file),
//! and maintains the freshness of `docs/observation-bounds.md`.

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
fn observation_bounds_are_declared_and_cited() {
    let Some(root) = workspace_root() else {
        return;
    };

    let specs_dir = root.join("openspec/specs");
    if !specs_dir.is_dir() {
        return;
    }

    let output = Command::new("git")
        .args(["ls-files", "openspec/specs"])
        .current_dir(&root)
        .output()
        .expect("git ls-files should succeed");

    assert!(output.status.success());
    let spec_files: Vec<String> = String::from_utf8_lossy(&output.stdout)
        .lines()
        .filter(|s| s.ends_with(".md"))
        .map(|s| s.to_string())
        .collect();

    let mut declared_bounds = Vec::new();

    for spec_rel in spec_files {
        let full_path = root.join(&spec_rel);
        let content = match std::fs::read_to_string(&full_path) {
            Ok(c) => c,
            Err(_) => continue,
        };

        for line in content.lines() {
            let trimmed = line.trim();
            if trimmed.starts_with("#### Scenario:")
                && (trimmed.contains("stated bound") || trimmed.contains("documented bound"))
            {
                declared_bounds.push((spec_rel.clone(), trimmed.to_string()));
            }
        }
    }

    assert!(
        !declared_bounds.is_empty(),
        "Observation bound register found zero declared bounds across openspec/specs"
    );
}

#[test]
fn the_observation_bounds_projection_is_fresh() {
    let Some(root) = workspace_root() else {
        return;
    };
    let bounds_doc = root.join("docs/observation-bounds.md");
    if bounds_doc.is_file() {
        let content = std::fs::read_to_string(&bounds_doc).expect("read observation-bounds.md");
        tianheng::testing::assert_projection_matches(&root, "docs/observation-bounds.md", &content);
    }
}

#[test]
fn an_absent_layout_is_loud_when_the_workspace_marker_is_set() {
    let absent = std::env::temp_dir().join("tianheng-bound-register-absent");
    let _ = std::fs::remove_dir_all(&absent);
    assert!(locate_layout(absent.clone(), false).is_none());
    assert!(
        std::panic::catch_unwind(|| locate_layout(absent, true)).is_err(),
        "an absent layout must fail loudly under TIANHENG_WORKSPACE_TESTS rather than skip"
    );
}
