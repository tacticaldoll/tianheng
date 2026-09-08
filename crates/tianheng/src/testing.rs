//! Adopter-facing test harness utilities for `cargo test` integration.
//!
//! Provides [`GovernanceTest`] to execute clean reaction assertions, verify workspace member
//! coverage, enforce Markdown projection freshness with `BLESS=1` auto-regeneration, and
//! test fixture reactions.

use std::path::{Path, PathBuf};

use guibiao::check_and_cover;

use crate::{Constitution, Outcome, check_constitution, constitution_markdown};

fn bless_enabled() -> bool {
    std::env::var("BLESS").is_ok_and(|value| value == "1" || value.eq_ignore_ascii_case("true"))
}

fn resolve_relative(path: &Path, base: &Path) -> PathBuf {
    if path.is_absolute() {
        path.to_path_buf()
    } else {
        base.join(path)
    }
}

/// A test harness for asserting architectural governance properties in `cargo test`.
///
/// Wraps a [`Constitution`] and provides fluent assertion methods for workspace governance,
/// coverage completeness, projection freshness, and fixture negative testing.
#[derive(Debug, Clone)]
pub struct GovernanceTest {
    constitution: Constitution,
    manifest_dir: PathBuf,
    explicit_manifest_dir: bool,
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
            explicit_manifest_dir: false,
        }
    }

    /// Explicitly override the manifest directory path.
    pub fn with_manifest_dir(mut self, path: impl Into<PathBuf>) -> Self {
        self.manifest_dir = path.into();
        self.explicit_manifest_dir = true;
        self
    }

    /// Access the wrapped [`Constitution`].
    pub fn constitution(&self) -> &Constitution {
        &self.constitution
    }

    /// Resolve the target manifest path (`Cargo.toml`).
    pub fn manifest_path(&self) -> PathBuf {
        ensure_cargo_toml_path(&self.manifest_dir)
    }

    /// Check if a manifest path exists, enforcing `TIANHENG_WORKSPACE_TESTS` discipline.
    fn check_manifest_exists(&self, manifest: PathBuf, is_explicit: bool) -> Option<PathBuf> {
        if !manifest.exists() {
            if is_explicit || self.explicit_manifest_dir {
                panic!("target manifest at {manifest:?} does not exist");
            }
            assert!(
                std::env::var_os("TIANHENG_WORKSPACE_TESTS").is_none(),
                "manifest expected at {:?} but absent while TIANHENG_WORKSPACE_TESTS is set",
                manifest
            );
            return None;
        }
        Some(manifest)
    }

    /// Helper to resolve the main constitution manifest path.
    fn resolve_manifest(&self) -> Option<PathBuf> {
        self.check_manifest_exists(self.manifest_path(), false)
    }

    /// Helper to resolve a fixture manifest path (absolute or relative to `manifest_dir`).
    fn resolve_fixture_manifest(&self, path: impl AsRef<Path>) -> Option<PathBuf> {
        let target_path = resolve_relative(path.as_ref(), &self.manifest_dir);
        self.check_manifest_exists(ensure_cargo_toml_path(&target_path), true)
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
            matches!(outcome, Outcome::Clean(_)),
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
        let mut coverage = coverage.expect("workspace metadata is readable");
        let semantic_targets: std::collections::HashSet<&str> = self
            .constitution
            .semantic_boundaries()
            .crate_packages()
            .collect();
        coverage
            .uncovered
            .retain(|member| !semantic_targets.contains(member.as_str()));
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

        let root = manifest.parent().unwrap_or_else(|| Path::new("."));

        let projection = constitution_markdown(&self.constitution);
        let expected = if preamble.is_empty() {
            projection
        } else if preamble.ends_with('\n') {
            format!("{preamble}{projection}")
        } else {
            format!("{preamble}\n{projection}")
        };

        assert_projection_matches(root, projection_path, &expected);
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
        let Some(manifest) = self.resolve_fixture_manifest(fixture_manifest_path) else {
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

    /// Alias for [`test_fixture`](Self::test_fixture).
    #[doc(alias = "test_fixture")]
    pub fn assert_violates_fixture(&self, fixture_manifest_path: impl AsRef<Path>) -> &Self {
        self.test_fixture(fixture_manifest_path)
    }
}

/// Helper function to ensure a path targets `Cargo.toml`.
fn ensure_cargo_toml_path(path: &Path) -> PathBuf {
    if path.ends_with("Cargo.toml") {
        path.to_path_buf()
    } else {
        path.join("Cargo.toml")
    }
}

/// Assert that the document at `projection_path` is exactly `expected`, blessing it under `BLESS`.
///
/// The bless-and-diff rule for **any** generated document.
/// [`GovernanceTest::assert_projection_fresh`] delegates here, and a caller rendering its own content — a
/// register of declared observation bounds, a table of gate properties — gets the same rule rather than
/// another copy of it. This repository already carries one implementation in shell and one here, and the
/// mechanism whose whole purpose is to stop documents drifting is a poor place to duplicate.
///
/// A free function rather than a method because the rule has nothing to do with a `Constitution`: requiring
/// one in order to bless an unrelated document would be a dependency invented by the API's shape.
///
/// `projection_path` is resolved relative to `root` when relative.
///
/// # Panics
///
/// Panics if the target cannot be read or written, or if the contents differ while `BLESS` is unset.
pub fn assert_projection_matches(root: &Path, projection_path: impl AsRef<Path>, expected: &str) {
    let target_path = resolve_relative(projection_path.as_ref(), root);

    if bless_enabled() {
        std::fs::write(&target_path, expected).unwrap_or_else(|err| {
            panic!("failed to write blessed projection to {target_path:?}: {err}")
        });
        return;
    }

    let actual = std::fs::read_to_string(&target_path).unwrap_or_else(|err| {
        panic!(
            "failed to read projection file at {target_path:?}: {err}. Run with BLESS=1 to generate."
        );
    });

    assert_eq!(
        actual, expected,
        "projection Markdown at {target_path:?} is out of sync with the code that generates it! Run with BLESS=1 to regenerate."
    );
}

#[cfg(test)]
mod tests {
    use super::*;

    struct TempHarness {
        root: PathBuf,
    }

    impl TempHarness {
        fn new(name: &str) -> Self {
            let root = std::env::temp_dir().join(format!(
                "tianheng-governance-test-{name}-{}",
                std::process::id()
            ));
            let _ = std::fs::remove_dir_all(&root);
            xingbiao::claim_scratch(&root).unwrap();
            std::fs::create_dir_all(root.join("src")).unwrap();
            std::fs::write(
                root.join("Cargo.toml"),
                "[package]\nname = \"fixture\"\nversion = \"0.0.0\"\n",
            )
            .unwrap();
            std::fs::write(root.join("src/lib.rs"), "").unwrap();
            Self { root }
        }
    }

    impl Drop for TempHarness {
        fn drop(&mut self) {
            let _ = std::fs::remove_dir_all(&self.root);
        }
    }

    #[test]
    fn relative_paths_resolve_from_their_callers_base() {
        let base = std::env::temp_dir().join("tianheng-relative-path-base");
        assert_eq!(
            resolve_relative(Path::new("fixtures/violating"), &base),
            base.join("fixtures/violating")
        );

        let absolute = base.join("law.md");
        assert_eq!(resolve_relative(&absolute, Path::new("ignored")), absolute);
    }

    #[test]
    #[ignore = "executed in subprocesses by projection_freshness_covers_every_bless_mode"]
    fn projection_bless_mode_child() {
        let mode = std::env::var("TIANHENG_PROJECTION_TEST_MODE").unwrap();
        let temp = TempHarness::new("projection");
        let harness = GovernanceTest::for_constitution(Constitution::new("fixture"))
            .with_manifest_dir(&temp.root);
        let path = temp.root.join("law.md");
        let live = constitution_markdown(harness.constitution());

        match mode.as_str() {
            "fresh" => {
                std::fs::write(&path, &live).unwrap();
                harness.assert_projection_fresh(&path);
            }
            "stale" => {
                std::fs::write(&path, "stale").unwrap();
                assert!(
                    std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                        harness.assert_projection_fresh(&path);
                    }))
                    .is_err()
                );
                assert_eq!(std::fs::read_to_string(&path).unwrap(), "stale");
            }
            "blessed" => {
                std::fs::write(&path, "stale").unwrap();
                harness.assert_projection_fresh(&path);
                assert_eq!(std::fs::read_to_string(&path).unwrap(), live);
            }
            _ => panic!("unknown projection test mode"),
        }
    }

    #[test]
    fn projection_freshness_covers_every_bless_mode() {
        let exe = std::env::current_exe().unwrap();
        for (bless, mode) in [
            (None, "fresh"),
            (None, "stale"),
            (Some(""), "stale"),
            (Some("0"), "stale"),
            (Some("false"), "stale"),
            (Some("1"), "blessed"),
            (Some("true"), "blessed"),
            (Some("TRUE"), "blessed"),
        ] {
            let mut command = std::process::Command::new(&exe);
            command
                .args([
                    "--exact",
                    "testing::tests::projection_bless_mode_child",
                    "--ignored",
                ])
                .env("TIANHENG_PROJECTION_TEST_MODE", mode);
            match bless {
                Some(value) => {
                    command.env("BLESS", value);
                }
                None => {
                    command.env_remove("BLESS");
                }
            }
            // **The child must be seen to have run, not merely to have exited 0.** `libtest` exits 0 for a
            // filter that selects nothing — measured, `--exact <unknown> --ignored` reports
            // `0 passed; 92 filtered out` and succeeds — so a rename of this module or of the child, or
            // removing its `#[ignore]`, would empty all eight rows and leave this matrix green having run
            // nothing. These eight rows are the only coverage of whether `assert_projection_fresh` overwrites
            // a tracked file across the BLESS spellings, which `COOKBOOK.md` and `README.md` advertise.
            //
            // Same remedy and same sentence as its two siblings: `require_one_pass` in both sanctioned
            // wrappers, and `ran_exactly_one` in `pin_bites` — *a filter matching nothing exits 0 over
            // nothing*.
            let output = command.output().unwrap();
            let log = String::from_utf8_lossy(&output.stdout);
            assert!(
                output.status.success(),
                "projection child failed for BLESS={bless:?}, mode={mode}:\n{log}"
            );
            assert!(
                log.contains("test result: ok. 1 passed"),
                "the projection child did not run for BLESS={bless:?}, mode={mode} — a filter matching \
                 nothing exits 0 over nothing, so this row proved nothing:\n{log}"
            );
        }
    }

    #[test]
    fn workspace_coverage_counts_semantic_targets_but_not_runtime_seams() {
        let temp = TempHarness::new("coverage");
        let semantic = crate::SignatureBoundary::in_crate("fixture")
            .module("crate")
            .must_not_expose("crate::infra")
            .because("the fixture API owns its vocabulary");
        GovernanceTest::for_constitution(Constitution::new("fixture").signature_boundary(semantic))
            .with_manifest_dir(&temp.root)
            .assert_all_workspace_members_covered();

        let runtime = crate::RuntimeBoundary::at("fixture-seam")
            .only_origins(["fixture"])
            .because("only the fixture crosses this seam");
        let runtime_only =
            GovernanceTest::for_constitution(Constitution::new("fixture").runtime(runtime))
                .with_manifest_dir(&temp.root);
        assert!(
            std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                runtime_only.assert_all_workspace_members_covered();
            }))
            .is_err(),
            "a seam carries no crate target and must not cover the fixture package"
        );
    }

    #[test]
    fn missing_explicit_manifest_or_fixture_panics_loudly() {
        let harness = GovernanceTest::for_constitution(Constitution::new("test"))
            .with_manifest_dir("non_existent_directory_xyz");
        assert!(
            std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                harness.assert_clean();
            }))
            .is_err(),
            "missing explicit manifest_dir must panic loudly"
        );

        let harness_fixture = GovernanceTest::for_constitution(Constitution::new("test"));
        assert!(
            std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                harness_fixture.assert_violates_fixture("non_existent_fixture.toml");
            }))
            .is_err(),
            "missing explicit fixture path must panic loudly"
        );
    }
}
