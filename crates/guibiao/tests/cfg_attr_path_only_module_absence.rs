//! Regression for a `pub mod imp;` declaration backed ONLY by one or more
//! `#[cfg_attr(pred, path = "...")]` remaps (no direct `#[path]`, no plain conventional file):
//! before this fix, 圭表's own module-boundary reachability walk required a plain conventional
//! file (`imp.rs` / `imp/mod.rs`) to exist even when a `cfg_attr(path)` candidate resolved to a
//! real on-disk file, hard-erroring (exit 2, "source file could not be located") on a
//! declaration that compiles cleanly under real rustc on every platform. 渾儀 and 漏刻 already
//! tolerate the identical shape in their own crate-wide walkers; this suite pins 圭表's own
//! module-boundary walk to the same rule (三儀 ⊥ 三儀: the same rule, not the same function),
//! through the real `guibiao::check(&Constitution, &Path)` entry point against a hermetic probe
//! workspace.
use std::path::{Path, PathBuf};

use guibiao::{Constitution, ModuleBoundary, Outcome, check};

/// A minimal, single-crate probe workspace, decoupled from Tianheng's own workspace via its own
/// `[workspace]` table (the same convention `crates/guibiao/tests/*` already uses).
struct ProbeWorkspace {
    dir: PathBuf,
    manifest: PathBuf,
}

impl ProbeWorkspace {
    fn new(name: &str, lib_rs: &str, extra_files: &[(&str, &str)]) -> Self {
        use std::sync::atomic::{AtomicU32, Ordering};
        static COUNTER: AtomicU32 = AtomicU32::new(0);
        let unique = COUNTER.fetch_add(1, Ordering::Relaxed);
        let dir = std::env::temp_dir().join(format!(
            "guibiao-cfg-attr-path-only-{name}-{}-{unique}",
            std::process::id()
        ));
        let _ = std::fs::remove_dir_all(&dir);
        xingbiao::claim_scratch(&dir).expect("the fixture root is writable");
        std::fs::create_dir_all(dir.join("src")).expect("create temp src dir");
        let manifest = dir.join("Cargo.toml");
        std::fs::write(
            &manifest,
            format!(
                "[package]\nname = \"{name}\"\nversion = \"0.1.0\"\nedition = \"2021\"\n\n[workspace]\n"
            ),
        )
        .expect("write Cargo.toml");
        std::fs::write(dir.join("src/lib.rs"), lib_rs).expect("write lib.rs");
        for (path, contents) in extra_files {
            let target = dir.join("src").join(path);
            if let Some(parent) = target.parent() {
                std::fs::create_dir_all(parent).expect("create extra file parent dir");
            }
            std::fs::write(target, contents).expect("write extra file");
        }
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

fn assert_violation_in(outcome: &Outcome, label: &str, expect_file_suffix: &str) {
    match outcome {
        Outcome::Violations(report) => {
            assert_eq!(
                outcome.exit_code(),
                1,
                "{label}: an enforce-severity violation must exit 1"
            );
            assert!(
                report.violations.iter().any(|v| v
                    .file
                    .as_deref()
                    .is_some_and(|f| f.ends_with(expect_file_suffix))),
                "{label}: expected a violation attributed to a file ending in \
                 '{expect_file_suffix}', got {:?}",
                report.violations
            );
        }
        other => panic!("{label}: expected Violations, got {other:?}"),
    }
}

/// Asserts a violation exists for EACH of `expect_file_suffixes` — used where both platform
/// targets must be independently confirmed scanned (real union-scan), not merely "a violation
/// happened somewhere": each fixture importing a DIFFERENT forbidden member keeps the two
/// occurrences from collapsing into one structured violation identity.
fn assert_violation_in_each(outcome: &Outcome, label: &str, expect_file_suffixes: &[&str]) {
    match outcome {
        Outcome::Violations(report) => {
            assert_eq!(
                outcome.exit_code(),
                1,
                "{label}: an enforce-severity violation must exit 1"
            );
            for suffix in expect_file_suffixes {
                assert!(
                    report
                        .violations
                        .iter()
                        .any(|v| v.file.as_deref().is_some_and(|f| f.ends_with(suffix))),
                    "{label}: expected a violation attributed to a file ending in '{suffix}', \
                     got {:?}",
                    report.violations
                );
            }
        }
        other => panic!("{label}: expected Violations, got {other:?}"),
    }
}

/// The 0.3.1 audit trigger shape reconstructed verbatim: a single `pub mod imp;` decorated with
/// TWO STACKED `#[cfg_attr(.., path = ..)]` attributes, one per platform, each naming a
/// different real file that exists on disk — jointly exhaustive (`unix` / `not(unix)`), so every
/// real rustc build compiles cleanly through exactly one of the two targets and NEVER needs a
/// plain `imp.rs` / `imp/mod.rs`, which does not exist here.
#[test]
fn stacked_cfg_attr_path_targets_with_no_plain_file_are_governed_not_hard_errored() {
    let lib_rs = r#"
pub mod forbidden {
    pub struct Thing;
    pub struct OtherThing;
}
#[cfg_attr(unix, path = "unix_imp.rs")]
#[cfg_attr(not(unix), path = "other_imp.rs")]
pub mod imp;
"#;
    let probe = ProbeWorkspace::new(
        "stackedcfgattr",
        lib_rs,
        &[
            (
                "unix_imp.rs",
                "use crate::forbidden::Thing;\npub fn x() -> Thing { Thing }\n",
            ),
            (
                "other_imp.rs",
                "use crate::forbidden::OtherThing;\npub fn y() -> OtherThing { OtherThing }\n",
            ),
        ],
    );
    let constitution = Constitution::new("repro").boundary(
        ModuleBoundary::in_crate("stackedcfgattr")
            .module("crate")
            .must_not_import("crate::forbidden")
            .because("audit-seam repro: two stacked cfg_attr(path) targets, no plain file"),
    );

    let outcome = check(&constitution, probe.manifest());
    // Both platform targets exist and each imports a DIFFERENT forbidden member, so both must
    // show up as distinct violations — confirming a real union-scan (both files read), not merely
    // "a violation happened somewhere" (which a single-file read would also produce). On unix a
    // real build compiles `unix_imp.rs`; the scanner is cfg-blind and reads `other_imp.rs` too,
    // same policy as every other `cfg_attr(path)` union already pinned in `reachability/tests.rs`.
    assert_violation_in_each(
        &outcome,
        "stacked cfg_attr(path), no plain file",
        &["unix_imp.rs", "other_imp.rs"],
    );
}

/// Control (a): a SINGLE `cfg_attr(path)` target, no plain file. Must be tolerated identically —
/// the fix is not specific to "stacked".
#[test]
fn single_cfg_attr_path_target_with_no_plain_file_is_governed_not_hard_errored() {
    let lib_rs = r#"
pub mod forbidden {
    pub struct Thing;
}
#[cfg_attr(unix, path = "unix_imp.rs")]
pub mod imp;
"#;
    let probe = ProbeWorkspace::new(
        "singlecfgattr",
        lib_rs,
        &[(
            "unix_imp.rs",
            "use crate::forbidden::Thing;\npub fn x() -> Thing { Thing }\n",
        )],
    );
    let constitution = Constitution::new("repro").boundary(
        ModuleBoundary::in_crate("singlecfgattr")
            .module("crate")
            .must_not_import("crate::forbidden")
            .because("audit-seam repro control: single cfg_attr(path), no plain file"),
    );

    let outcome = check(&constitution, probe.manifest());
    assert_violation_in(
        &outcome,
        "single cfg_attr(path), no plain file",
        "unix_imp.rs",
    );
}

/// Control (b): a direct unconditional `#[path]` plus a `cfg_attr` fallback must keep working,
/// ensuring both candidates remain observed.
#[test]
fn direct_path_plus_cfg_attr_fallback_still_works() {
    let lib_rs = r#"
pub mod forbidden {
    pub struct Thing;
}
#[path = "unix_imp.rs"]
#[cfg_attr(not(unix), path = "other_imp.rs")]
pub mod imp;
"#;
    let probe = ProbeWorkspace::new(
        "directpluscond",
        lib_rs,
        &[
            (
                "unix_imp.rs",
                "use crate::forbidden::Thing;\npub fn x() -> Thing { Thing }\n",
            ),
            (
                "other_imp.rs",
                "use crate::forbidden::Thing;\npub fn y() -> Thing { Thing }\n",
            ),
        ],
    );
    let constitution = Constitution::new("repro").boundary(
        ModuleBoundary::in_crate("directpluscond")
            .module("crate")
            .must_not_import("crate::forbidden")
            .because("audit-seam repro control: direct path + cfg_attr fallback"),
    );

    let outcome = check(&constitution, probe.manifest());
    assert_violation_in(
        &outcome,
        "direct #[path] + cfg_attr fallback",
        "other_imp.rs",
    );
}

/// Corner case the fix must NOT widen: a `cfg_attr(path)` target that itself does not exist on
/// disk, and no plain conventional file either, and no bare `#[cfg]` — every candidate absent —
/// must remain a genuine constitution error (exit 2), matching hunyi's own
/// `!has_backing_source && !cfg_conditional` boundary for the identical shape.
#[test]
fn cfg_attr_path_target_absent_with_no_plain_file_still_hard_errors() {
    let lib_rs = r#"
pub mod forbidden {
    pub struct Thing;
}
#[cfg_attr(windows, path = "windows_only_imp.rs")]
pub mod imp;
"#;
    // Deliberately do NOT create `windows_only_imp.rs`, `imp.rs`, or `imp/mod.rs`.
    let probe = ProbeWorkspace::new("absentcfgattr", lib_rs, &[]);
    let constitution = Constitution::new("repro").boundary(
        ModuleBoundary::in_crate("absentcfgattr")
            .module("crate")
            .must_not_import("crate::forbidden")
            .because("audit-seam repro control: cfg_attr(path) target absent, no plain file"),
    );

    let outcome = check(&constitution, probe.manifest());
    match outcome {
        Outcome::ConstitutionError(msg) => {
            assert!(
                msg.contains("crate::imp") && msg.contains("could not be located"),
                "expected the missing-plain-file constitution error, got: {msg}"
            );
        }
        other => panic!(
            "cfg_attr(path) target absent + no plain file: expected ConstitutionError, got {other:?}"
        ),
    }
}

/// Corner case the fix must NOT widen: both conventional forms present stays an ambiguity error
/// even when a resolvable `cfg_attr(path)` candidate also exists on the same declaration.
#[test]
fn both_conventional_forms_present_stays_an_ambiguity_error_alongside_a_resolved_cfg_attr() {
    let lib_rs = r#"
pub mod forbidden {
    pub struct Thing;
}
#[cfg_attr(unix, path = "unix_imp.rs")]
pub mod imp;
"#;
    let probe = ProbeWorkspace::new(
        "dualformcfgattr",
        lib_rs,
        &[
            ("unix_imp.rs", "use crate::forbidden::Thing;\n"),
            ("imp.rs", "// conventional flat form\n"),
        ],
    );
    std::fs::create_dir_all(probe.dir.join("src/imp")).expect("create imp dir");
    std::fs::write(
        probe.dir.join("src/imp/mod.rs"),
        "// conventional nested form\n",
    )
    .expect("write imp/mod.rs");

    let constitution = Constitution::new("repro").boundary(
        ModuleBoundary::in_crate("dualformcfgattr")
            .module("crate")
            .must_not_import("crate::forbidden")
            .because("audit-seam repro control: both conventional forms present"),
    );

    let outcome = check(&constitution, probe.manifest());
    match outcome {
        Outcome::ConstitutionError(msg) => {
            assert!(
                msg.contains("resolves to both"),
                "expected the dual-backed ambiguity constitution error, got: {msg}"
            );
        }
        other => {
            panic!("both conventional forms present: expected ConstitutionError, got {other:?}")
        }
    }
}

/// The same rule for an **inline** `mod x { … }`, where a `cfg_attr(path)` names the base directory
/// the body's own file-form children resolve from rather than a file to read.
///
/// The spec already required this for an **unconditional** `#[path]` and 圭表 already honored it; only
/// the conditional form was missed, and it failed in the loud direction: the walk looked for the
/// conventional `src/x/y.rs` and reported a constitution error on a crate that builds cleanly, so an
/// adopter using this idiom could not run `check` at all. Reproduced against the real entry point
/// before the fix, with the unconditional form as the control.
#[test]
fn a_conditional_remap_on_an_inline_mod_relocates_its_child_base() {
    let lib_rs = r#"
pub mod forbidden {
    pub struct Thing;
}
#[cfg_attr(unix, path = "unix_dir")]
pub mod x {
    pub mod y;
}
"#;
    let probe = ProbeWorkspace::new(
        "inlinecondbase",
        lib_rs,
        &[(
            "unix_dir/y.rs",
            "use crate::forbidden::Thing;\npub fn f() -> Thing { Thing }\n",
        )],
    );
    let constitution = Constitution::new("repro").boundary(
        ModuleBoundary::in_crate("inlinecondbase")
            .module("crate")
            .must_not_import("crate::forbidden")
            .because("a conditional remap on an inline mod relocates its child base"),
    );

    let outcome = check(&constitution, probe.manifest());
    assert_violation_in(&outcome, "cfg_attr(path) on an inline mod", "unix_dir/y.rs");
}

/// The union half, and the reason a candidate is a candidate rather than the base: the scanner does
/// not evaluate `cfg`, so both platform bases are descended and the children beneath each are
/// observed. Each fixture imports a DIFFERENT forbidden member, so a single-base read could not
/// produce both violations.
#[test]
fn every_present_conditional_base_of_an_inline_mod_is_descended() {
    let lib_rs = r#"
pub mod forbidden {
    pub struct Thing;
    pub struct OtherThing;
}
#[cfg_attr(unix, path = "unix_dir")]
#[cfg_attr(not(unix), path = "other_dir")]
pub mod x {
    pub mod y;
}
"#;
    let probe = ProbeWorkspace::new(
        "inlinecondunion",
        lib_rs,
        &[
            (
                "unix_dir/y.rs",
                "use crate::forbidden::Thing;\npub fn f() -> Thing { Thing }\n",
            ),
            (
                "other_dir/y.rs",
                "use crate::forbidden::OtherThing;\npub fn g() -> OtherThing { OtherThing }\n",
            ),
        ],
    );
    let constitution = Constitution::new("repro").boundary(
        ModuleBoundary::in_crate("inlinecondunion")
            .module("crate")
            .must_not_import("crate::forbidden")
            .because("every present conditional base of an inline mod is descended"),
    );

    let outcome = check(&constitution, probe.manifest());
    assert_violation_in_each(
        &outcome,
        "two conditional bases on one inline mod",
        &["unix_dir/y.rs", "other_dir/y.rs"],
    );
}

/// The tolerance must not become a silent pass. When NO candidate base exists and neither does the
/// conventional one, the body's `mod y;` is a reference broken on every platform — rustc errors on it
/// — so the conventional base is descended anyway and the missing child still fails loud.
#[test]
fn an_inline_mod_whose_every_conditional_base_is_absent_still_fails_loud() {
    let lib_rs = r#"
pub mod forbidden {
    pub struct Thing;
}
#[cfg_attr(unix, path = "absent_dir")]
pub mod x {
    pub mod y;
}
"#;
    let probe = ProbeWorkspace::new("inlinecondabsent", lib_rs, &[]);
    let constitution = Constitution::new("repro").boundary(
        ModuleBoundary::in_crate("inlinecondabsent")
            .module("crate")
            .must_not_import("crate::forbidden")
            .because("an absent base is tolerated, a broken child reference is not"),
    );

    match check(&constitution, probe.manifest()) {
        Outcome::ConstitutionError(msg) => {
            assert!(
                msg.contains("could not be located"),
                "expected the missing-module constitution error, got: {msg}"
            );
        }
        other => panic!("every base absent: expected ConstitutionError, got {other:?}"),
    }
}
