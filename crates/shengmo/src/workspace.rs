//! Where the repository is, for the law and every reaction that reads it.
//!
//! One definition. Before this crate existed the probe was copied into 14 targets and its
//! absent-layout direction into 11 of them, which is why `TIANHENG_WORKSPACE_TESTS` came to mean
//! two different things: a marker with fourteen definitions has none.

use std::path::PathBuf;

/// The Tianheng workspace manifest. `None` when it is absent — e.g. inside a published
/// `.crate` tarball, which has no workspace root — so the self-governance gate SKIPS rather
/// than fails when the crate is tested standalone. In the repo the path exists, so the gate
/// runs for real.
pub fn manifest() -> Option<PathBuf> {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../Cargo.toml");
    if path.exists() {
        return Some(path);
    }
    // Absent. CI sets TIANHENG_WORKSPACE_TESTS=1 so a missing manifest (a checkout/layout
    // regression) fails LOUD rather than silently skipping the dogfood gate; without the env
    // (e.g. a packaged .crate tested standalone) the absence is legitimate, so skip.
    assert!(
        std::env::var_os("TIANHENG_WORKSPACE_TESTS").is_none(),
        "workspace manifest expected but absent while TIANHENG_WORKSPACE_TESTS is set — \
         the self-governance gate must not silently skip in CI"
    );
    None
}

/// The repository root — the parent of the workspace manifest. Reuses [`manifest`]'s
/// repo-only discipline verbatim: `None` (skip) outside a checkout, fail-loud under
/// `TIANHENG_WORKSPACE_TESTS`.
pub fn root() -> Option<PathBuf> {
    manifest().map(|m| {
        m.parent()
            .expect("the workspace manifest has a parent directory")
            .to_path_buf()
    })
}
