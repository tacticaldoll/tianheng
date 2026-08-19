//! Where the repository is, for the law and every gate that reads it.
//!
//! One definition of the **scaffolding**; the *prerequisite* stays with each gate, because that part
//! genuinely differs — the changelog check needs `CHANGELOG.md`, the register needs `openspec/specs/`,
//! the examples suite needs `examples/`. Measured before merging them: the fourteen copies this replaces
//! agreed on every line except that one, so collapsing further would have erased a real distinction while
//! collapsing less leaves `TIANHENG_WORKSPACE_TESTS` meaning whatever each copy decided.
//!
//! The discipline the scaffolding carries: a gate that cannot find its subject **skips** outside a
//! checkout — a packaged tarball has no workspace root, and failing there would be wrong — and **fails
//! loudly** when the marker says a repository was expected. A governance gate that quietly does nothing
//! in CI is the shape this family argues against.

use std::path::{Path, PathBuf};

/// Set where a repository is expected, so an absent layout is a defect rather than a skip.
///
/// **One owner within its reach**, which `crates/kanhe/tests/one_spelling.rs` holds — including the
/// regeneration commands the generated documents carry in their headers, where a rename would otherwise
/// leave every projection telling a reader a command that silently skips the gate it names.
///
/// What it cannot reach: the sites in `tianheng`, `louke` and `xuanji`. Those are published crates and cannot
/// depend on this one, because this one depends on `tianheng` and the edge would close a cycle. A rename
/// therefore has to be made in those crates by hand, and no check can see them — a fact about the dependency
/// graph rather than a site anyone declined to fix. **No count is given**: nothing enumerates that set, so a
/// figure here would be a census with no producer, and the one that stood here had already drifted.
pub const MARKER: &str = "TIANHENG_WORKSPACE_TESTS";

/// Whether the marker says this run must find a repository.
pub fn marker_set() -> bool {
    std::env::var_os(MARKER).is_some()
}

/// `root` when `present` finds what the caller needs there; `None` to skip; a panic under the marker.
///
/// Taking the marker as an argument rather than reading it keeps the direction below able to exercise both
/// answers without touching the process environment, which a parallel test run shares.
pub fn locate(root: PathBuf, present: impl Fn(&Path) -> bool, marker_set: bool) -> Option<PathBuf> {
    if present(&root) {
        return Some(root);
    }
    assert!(
        !marker_set,
        "the repository layout expected under {root:?} is absent while {MARKER} is set — a governance \
         gate that quietly does nothing in CI is the shape this family argues against"
    );
    None
}

/// The Tianheng workspace manifest, for a caller whose prerequisite is the workspace itself.
pub fn manifest() -> Option<PathBuf> {
    root().map(|root| root.join("Cargo.toml"))
}

/// The repository root, requiring only the workspace manifest.
pub fn root() -> Option<PathBuf> {
    locate(
        PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../.."),
        |root| root.join("Cargo.toml").is_file(),
        marker_set(),
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Both answers of the one direction eleven copies each asserted separately.
    #[test]
    fn an_absent_layout_skips_without_the_marker_and_is_loud_with_it() {
        let absent = std::env::temp_dir().join("shengmo-workspace-absent-probe");
        let _ = std::fs::remove_dir_all(&absent);

        assert!(
            locate(
                absent.clone(),
                |root| root.join("Cargo.toml").is_file(),
                false
            )
            .is_none(),
            "outside a checkout a gate must skip: a packaged tarball has no workspace root"
        );
        assert!(
            std::panic::catch_unwind(|| locate(
                absent,
                |root| root.join("Cargo.toml").is_file(),
                true
            ))
            .is_err(),
            "under the marker an absent layout must fail loudly rather than skip"
        );
    }

    /// A present layout is returned, so the direction above is about absence rather than about `locate`
    /// never answering.
    #[test]
    fn a_present_layout_is_returned() {
        let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        assert_eq!(
            locate(root.clone(), |root| root.join("Cargo.toml").is_file(), true),
            Some(root)
        );
    }
}
