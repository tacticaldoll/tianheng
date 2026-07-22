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
