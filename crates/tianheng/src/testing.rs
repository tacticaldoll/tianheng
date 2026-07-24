//! Adopter-facing test harness utilities for `cargo test` integration.
//!
//! Provides [`GovernanceTest`] to execute clean reaction assertions, verify workspace member
//! coverage, enforce Markdown projection freshness with `BLESS=1` auto-regeneration, and
//! test fixture reactions.

use std::path::{Path, PathBuf};

use guibiao::check_and_cover;

use crate::{Constitution, Outcome, check_constitution, constitution_markdown};

/// A test harness for asserting architectural governance properties in `cargo test`.
///
/// Wraps a [`Constitution`] and provides fluent assertion methods for workspace governance,
/// coverage completeness, projection freshness, and fixture negative testing.
#[derive(Debug, Clone)]
pub struct GovernanceTest {
    constitution: Constitution,
    manifest_dir: PathBuf,
}

impl GovernanceTest {
    /// Begin a governance test harness for the given [`Constitution`].
    ///
    /// Resolves the manifest directory from `CARGO_MANIFEST_DIR` by default.
    pub fn for_constitution(constitution: Constitution) -> Self {
        let manifest_dir = match std::env::var_os("CARGO_MANIFEST_DIR") {
            Some(dir) => PathBuf::from(dir),
            None => PathBuf::from("."),
        };
        Self {
            constitution,
            manifest_dir,
        }
    }

    /// Explicitly override the manifest directory path.
    pub fn with_manifest_dir(mut self, path: impl Into<PathBuf>) -> Self {
        self.manifest_dir = path.into();
        self
    }

    /// Access the wrapped [`Constitution`].
    pub fn constitution(&self) -> &Constitution {
        &self.constitution
    }

    /// Resolve the target manifest path (`Cargo.toml`).
    pub fn manifest_path(&self) -> PathBuf {
        if self.manifest_dir.ends_with("Cargo.toml") {
            self.manifest_dir.clone()
        } else {
            self.manifest_dir.join("Cargo.toml")
        }
    }

    /// Helper to resolve target manifest path and handle missing manifest under packaged tests.
    fn resolve_manifest(&self) -> Option<PathBuf> {
        self.resolve_manifest_from(&self.manifest_dir)
    }

    /// Helper to resolve a specific manifest path and handle missing manifest under packaged tests.
    fn resolve_manifest_from(&self, path: impl AsRef<Path>) -> Option<PathBuf> {
        let target_path = if path.as_ref().is_absolute() {
            path.as_ref().to_path_buf()
        } else {
            self.manifest_dir.join(path.as_ref())
        };

        let manifest = if target_path.ends_with("Cargo.toml") {
            target_path
        } else {
            target_path.join("Cargo.toml")
        };

        if !manifest.exists() {
            assert!(
                std::env::var_os("TIANHENG_WORKSPACE_TESTS").is_none(),
                "manifest expected at {:?} but absent while TIANHENG_WORKSPACE_TESTS is set",
                manifest
            );
            return None;
        }
        Some(manifest)
    }

    /// Assert that the constitution returns no violations (`Outcome::Clean`).
    ///
    /// # Panics
    ///
    /// Panics with a formatted report if any boundary violation or constitution error occurs.
    pub fn assert_clean(&self) -> &Self {
        let Some(manifest) = self.resolve_manifest() else {
            return self;
        };

        let outcome = check_constitution(&self.constitution, &manifest);
        assert!(
            matches!(outcome, Outcome::Clean),
            "architectural violations or errors detected:\n{outcome:?}"
        );
        self
    }

    /// Assert that every member crate in the workspace is targeted by at least one boundary.
    ///
    /// Prevents vacuous test passes where a misspelled target or missing crate escapes governance.
    ///
    /// # Panics
    ///
    /// Panics if any workspace member has no targeting boundary, or if zero members are observed.
    pub fn assert_all_workspace_members_covered(&self) -> &Self {
        let Some(manifest) = self.resolve_manifest() else {
            return self;
        };

        let (_, coverage) = check_and_cover(self.constitution.static_boundaries(), &manifest);
        let coverage = coverage.expect("workspace metadata is readable");
        assert!(
            coverage.total > 0,
            "coverage observed zero workspace members — empty uncovered set would pass vacuously"
        );
        assert!(
            coverage.uncovered.is_empty(),
            "workspace members escape governance (no boundary targets them): {:?}",
            coverage.uncovered
        );
        self
    }

    /// Assert that the Markdown projection at `projection_path` matches the generated constitution doc.
    ///
    /// If `BLESS=1` or `BLESS=true` is set in the environment, overwrites the target file with
    /// the rendered Markdown projection when a mismatch occurs.
    ///
    /// # Panics
    ///
    /// Panics if the target file cannot be read/written or if contents mismatch while `BLESS` is unset.
    pub fn assert_projection_fresh(&self, projection_path: impl AsRef<Path>) -> &Self {
        self.assert_projection_fresh_with_preamble(projection_path, "")
    }

    /// Assert that the Markdown projection at `projection_path` matches the given preamble plus
    /// the generated constitution doc.
    ///
    /// If `BLESS=1` or `BLESS=true` is set in the environment, overwrites the target file with
    /// the rendered Markdown projection when a mismatch occurs.
    pub fn assert_projection_fresh_with_preamble(
        &self,
        projection_path: impl AsRef<Path>,
        preamble: &str,
    ) -> &Self {
        let Some(manifest) = self.resolve_manifest() else {
            return self;
        };

        let target_path = if projection_path.as_ref().is_absolute() {
            projection_path.as_ref().to_path_buf()
        } else {
            let root = manifest.parent().unwrap_or_else(|| Path::new("."));
            root.join(projection_path.as_ref())
        };

        let projection = constitution_markdown(&self.constitution);
        let expected = if preamble.is_empty() {
            projection
        } else if preamble.ends_with('\n') {
            format!("{preamble}{projection}")
        } else {
            format!("{preamble}\n{projection}")
        };

        let is_bless = match std::env::var("BLESS") {
            Ok(val) => val == "1" || val.eq_ignore_ascii_case("true"),
            Err(_) => false,
        };

        if is_bless {
            std::fs::write(&target_path, &expected).unwrap_or_else(|err| {
                panic!("failed to write blessed projection to {target_path:?}: {err}")
            });
            return self;
        }

        let actual = std::fs::read_to_string(&target_path).unwrap_or_else(|err| {
            panic!(
                "failed to read projection file at {target_path:?}: {err}. Run with BLESS=1 to generate."
            );
        });

        assert_eq!(
            actual, expected,
            "projection Markdown at {target_path:?} is out of sync with code constitution! Run with BLESS=1 to regenerate."
        );

        self
    }

    /// Assert that evaluating the constitution against a violating fixture manifest yields boundary violations.
    ///
    /// Evaluates `check_constitution` against `fixture_manifest_path` and asserts that the outcome
    /// is [`Outcome::Violations`]. A [`Outcome::ConstitutionError`] or [`Outcome::Clean`] will panic.
    ///
    /// # Panics
    ///
    /// Panics if fixture evaluation returns [`Outcome::Clean`] or [`Outcome::ConstitutionError`].
    pub fn test_fixture(&self, fixture_manifest_path: impl AsRef<Path>) -> &Self {
        self.assert_violates_fixture(fixture_manifest_path)
    }

    /// Assert that evaluating the constitution against a violating fixture manifest yields boundary violations.
    ///
    /// Evaluates `check_constitution` against `fixture_manifest_path` and asserts that the outcome
    /// is [`Outcome::Violations`]. A [`Outcome::ConstitutionError`] or [`Outcome::Clean`] will panic.
    ///
    /// # Panics
    ///
    /// Panics if fixture evaluation returns [`Outcome::Clean`] or [`Outcome::ConstitutionError`].
    #[doc(alias = "test_fixture")]
    pub fn assert_violates_fixture(&self, fixture_manifest_path: impl AsRef<Path>) -> &Self {
        let Some(manifest) = self.resolve_manifest_from(fixture_manifest_path) else {
            return self;
        };

        let outcome = check_constitution(&self.constitution, &manifest);
        assert!(
            matches!(outcome, Outcome::Violations(_)),
            "expected a boundary violation for fixture at {:?}, got: {:?}",
            manifest,
            outcome
        );
        self
    }
}
