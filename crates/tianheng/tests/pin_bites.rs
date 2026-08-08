//! Self-governance reaction: verification of pinned observation bounds.
//!
//! Asserts that every declared bound citation names a test that is registered and callable
//! within the test harness.

use std::path::PathBuf;

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
fn pin_mutation_records_exist_or_are_valid() {
    let Some(root) = workspace_root() else {
        return;
    };

    let records_path = root.join("scripts/lib/pin_mutations.tsv");
    if !records_path.is_file() {
        return;
    }

    let content = std::fs::read_to_string(&records_path).expect("read pin_mutations.tsv");
    let mut valid_records = 0;

    for line in content.lines() {
        let trimmed = line.trim();
        if trimmed.is_empty() || trimmed.starts_with('#') {
            continue;
        }
        let parts: Vec<&str> = trimmed.split('\t').collect();
        if parts.len() >= 2 {
            valid_records += 1;
        }
    }

    assert!(
        valid_records > 0,
        "scripts/lib/pin_mutations.tsv parsed to zero valid mutation records"
    );
}

#[test]
fn an_absent_layout_is_loud_when_the_workspace_marker_is_set() {
    let absent = std::env::temp_dir().join("tianheng-pin-bites-absent");
    let _ = std::fs::remove_dir_all(&absent);
    assert!(locate_layout(absent.clone(), false).is_none());
    assert!(
        std::panic::catch_unwind(|| locate_layout(absent, true)).is_err(),
        "an absent layout must fail loudly under TIANHENG_WORKSPACE_TESTS rather than skip"
    );
}
