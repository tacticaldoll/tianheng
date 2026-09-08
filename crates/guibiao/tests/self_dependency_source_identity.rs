//! `is_self_dependency` once matched a dependency edge by NAME ALONE, so a same-named but
//! EXTERNALLY-sourced dependency (a real wrapper/fork/self-comparison pattern, e.g.
//! `foo = { git = "…" }` declared by package `foo`) was wrongly swallowed by the exemption meant
//! only for the genuine null-source self-referential path idiom (`main = { path = "." }`), and
//! every rule built on it — `restrict_dependency_sources_to`, `restrict_dependencies_to([])`,
//! `forbid_dependency_on` — silently read `exit=0 Clean` against it.
//!
//! This suite runs the exact rule constructors the audit names through the real
//! `guibiao::check(&Constitution, &Path)` entry point against a hermetic probe workspace. `cargo
//! metadata --no-deps` never resolves or fetches a dependency graph — verified directly against
//! real cargo: it returns instantly, even offline, for an unreachable git host — so this probe
//! (and the fix it pins) needs no network access and is safe to run in CI.
use std::path::{Path, PathBuf};

use guibiao::{Constitution, CrateBoundary, Outcome, SourceKind, check};

/// A minimal, single-crate probe workspace, decoupled from Tianheng's own workspace via its own
/// `[workspace]` table (the same convention `crates/tianheng/tests/fixtures/*` uses).
struct ProbeWorkspace {
    dir: PathBuf,
    manifest: PathBuf,
}

impl ProbeWorkspace {
    /// Write a probe crate named `name` whose `[dependencies]` table is exactly `dependencies_toml`
    /// (a raw TOML fragment, e.g. `r#"foo = { git = "https://example.invalid/foo.git" }"#`).
    fn new(name: &str, dependencies_toml: &str) -> Self {
        use std::sync::atomic::{AtomicU32, Ordering};
        static COUNTER: AtomicU32 = AtomicU32::new(0);
        let unique = COUNTER.fetch_add(1, Ordering::Relaxed);
        let dir = std::env::temp_dir().join(format!(
            "guibiao-self-dep-source-identity-{name}-{}-{unique}",
            std::process::id()
        ));
        let _ = std::fs::remove_dir_all(&dir);
        xingbiao::claim_scratch(&dir).expect("the fixture root is writable");
        std::fs::create_dir_all(dir.join("src")).expect("create temp src dir");
        let manifest = dir.join("Cargo.toml");
        std::fs::write(
            &manifest,
            format!(
                "[package]\nname = \"{name}\"\nversion = \"0.1.0\"\nedition = \"2021\"\n\n\
                 [dependencies]\n{dependencies_toml}\n\n[workspace]\n"
            ),
        )
        .expect("write Cargo.toml");
        std::fs::write(dir.join("src/lib.rs"), "pub fn hi() {}\n").expect("write lib.rs");
        Self { dir, manifest }
    }

    fn manifest(&self) -> &Path {
        &self.manifest
    }
}

impl Drop for ProbeWorkspace {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.dir);
    }
}

fn assert_violation(outcome: &Outcome, label: &str) {
    match outcome {
        Outcome::Violations(report) => {
            assert_eq!(
                outcome.exit_code(),
                1,
                "{label}: an enforce-severity violation must exit 1"
            );
            assert!(
                !report.violations.is_empty(),
                "{label}: report must carry the violation"
            );
        }
        other => panic!(
            "{label}: expected a Violations outcome for the same-named externally-sourced \
             dependency, got {other:?}"
        ),
    }
}

#[test]
fn restrict_dependency_sources_to_reacts_to_a_same_named_git_dependency() {
    let probe = ProbeWorkspace::new(
        "foo",
        r#"foo = { git = "https://example.invalid/foo.git" }"#,
    );
    let constitution = Constitution::new("repro").boundary(
        CrateBoundary::crate_("foo")
            .restrict_dependency_sources_to([SourceKind::Registry, SourceKind::Path])
            .because("a publishable crate's manifest declares no git dependencies"),
    );

    let outcome = check(&constitution, probe.manifest());
    assert_violation(&outcome, "restrict_dependency_sources_to([Registry, Path])");
}

#[test]
fn restrict_dependencies_to_empty_reacts_to_a_same_named_git_dependency() {
    let probe = ProbeWorkspace::new(
        "foo",
        r#"foo = { git = "https://example.invalid/foo.git" }"#,
    );
    let constitution = Constitution::new("repro").boundary(
        CrateBoundary::crate_("foo")
            .restrict_dependencies_to(Vec::<String>::new())
            .because("this crate forbids every normal dependency"),
    );

    let outcome = check(&constitution, probe.manifest());
    assert_violation(&outcome, "restrict_dependencies_to([])");
}

#[test]
fn forbid_dependency_on_reacts_to_a_same_named_git_dependency() {
    let probe = ProbeWorkspace::new(
        "foo",
        r#"foo = { git = "https://example.invalid/foo.git" }"#,
    );
    let constitution = Constitution::new("repro").boundary(
        CrateBoundary::crate_("foo")
            .forbid_dependency_on(["foo"])
            .because("foo must never depend on a crate named foo"),
    );

    let outcome = check(&constitution, probe.manifest());
    assert_violation(&outcome, "forbid_dependency_on([\"foo\"])");
}

#[test]
fn a_genuine_self_path_dependency_is_still_exempt_through_the_real_entry_point() {
    // The legitimate doctest/dogfooding idiom (`main = { path = "." }`) must still be exempt end
    // to end, not only at the unit level — this is the regression the fix must not break.
    let probe = ProbeWorkspace::new("main", r#"main = { path = "." }"#);
    let constitution = Constitution::new("repro").boundary(
        CrateBoundary::crate_("main")
            .restrict_dependencies_to(Vec::<String>::new())
            .dependency_kind(guibiao::DependencyKind::Normal)
            .because("main forbids every normal dependency except its own self-reference"),
    );

    let outcome = check(&constitution, probe.manifest());
    assert!(
        matches!(outcome, Outcome::Clean(_)),
        "a genuine null-source self-dependency must remain exempt, got {outcome:?}"
    );
}
