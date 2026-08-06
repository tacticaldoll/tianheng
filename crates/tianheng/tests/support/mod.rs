//! Shared temp-fixture helper for the cross-dimension conformance suites (`*_conformance.rs`):
//! each feeds the same input through more than one observation dimension's real public entry
//! point, and each independently hand-rolled its own `write_fixture` + manual `remove_dir_all`
//! before this was centralized — the plumbing this file replaces, not the conformance claims
//! themselves (those stay in each suite).
//!
//! Compiled fresh into each `*_conformance.rs` binary via `#[path]`, so a field/method only some
//! callers use (e.g. `lib()`, needed only by `lexical_conformance.rs`) is dead code in the others
//! — allowed here rather than split into per-binary variants.
#![allow(dead_code)]

pub mod region;

use std::path::{Path, PathBuf};

/// A minimal, dependency-free single-crate fixture (so `cargo metadata --no-deps` never touches
/// the network), written under a unique temp directory and cleaned up on drop.
pub struct TempFixture {
    dir: PathBuf,
    manifest: PathBuf,
    lib: PathBuf,
}

impl TempFixture {
    /// Write a fixture crate named `name` with `lib.rs` set to `body`.
    pub fn new(name: &str, body: &str) -> Self {
        let dir = std::env::temp_dir().join(format!(
            "tianheng-conformance-{name}-{}",
            std::process::id()
        ));
        let _ = std::fs::remove_dir_all(&dir);
        let src = dir.join("src");
        std::fs::create_dir_all(&src).expect("create temp src");
        let manifest = dir.join("Cargo.toml");
        std::fs::write(
            &manifest,
            format!("[package]\nname = \"{name}\"\nversion = \"0.0.0\"\nedition = \"2021\"\n"),
        )
        .expect("write Cargo.toml");
        let lib = src.join("lib.rs");
        std::fs::write(&lib, body).expect("write lib.rs");
        Self { dir, manifest, lib }
    }

    pub fn manifest(&self) -> &Path {
        &self.manifest
    }

    pub fn lib(&self) -> &Path {
        &self.lib
    }
}

impl Drop for TempFixture {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.dir);
    }
}

/// 圭表's exit code for a `must_not_import("crate::forbidden")` boundary on `module` — the shared
/// shape every `*_conformance.rs` suite checks 圭表 against, differing only in which module is
/// anchored and which reason each suite states for its own fixture.
pub fn guibiao_exit(package: &str, manifest: &Path, module: &str, reason: &str) -> u8 {
    let constitution = guibiao::Constitution::new(package).boundary(
        guibiao::ModuleBoundary::in_crate(package)
            .module(module)
            .must_not_import("crate::forbidden")
            .because(reason),
    );
    guibiao::check(&constitution, manifest).exit_code()
}

/// 渾儀's exit code for a `must_not_expose("crate::forbidden::Thing")` boundary on `module` — the
/// semantic-dimension twin of [`guibiao_exit`] above.
pub fn hunyi_exit(package: &str, manifest: &Path, module: &str, reason: &str) -> u8 {
    let boundary = hunyi::SignatureBoundary::in_crate(package)
        .module(module)
        .must_not_expose("crate::forbidden::Thing")
        .because(reason);
    hunyi::check(&[boundary], manifest).exit_code()
}

/// 漏刻's exit code for an `only_origins(["o"])` boundary at `seam`, audited over `root` — the
/// runtime-dimension twin of [`guibiao_exit`]/[`hunyi_exit`] above.
pub fn louke_exit(root: &Path, seam: &'static str, reason: &str) -> u8 {
    let boundary = louke::RuntimeBoundary::at(seam)
        .only_origins(["o"])
        .because(reason);
    // These conformance fixtures assert exit codes, never `file` labels, so the anchor only needs to
    // be the fixture's own checkout-equivalent: the directory holding the scanned root, which is
    // what a real caller's `workspace_root` is relative to its members.
    let anchor = root.parent().unwrap_or(root);
    louke::audit_probe_coverage(&[boundary], &[root.to_path_buf()], anchor).exit_code()
}
