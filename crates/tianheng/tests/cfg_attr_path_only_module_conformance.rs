//! Cross-dimension conformance for a module declared ONLY via one or more resolved
//! `#[cfg_attr(pred, path = "...")]` remaps — no plain conventional file (`imp.rs` /
//! `imp/mod.rs`) and no unconditional `#[path]`.
//!
//! 渾儀's `has_backing_source` walk (`crates/hunyi/src/scan.rs`) and 漏刻's own copy
//! (`crates/louke/src/audit/scan.rs`) already tolerated this exact shape, treating a resolved
//! `cfg_attr(path)` candidate as legitimate grounds for the plain file's own absence. 圭表's own
//! module-boundary reachability walk (`crates/guibiao/src/module_scan/reachability/walk.rs`) did
//! not: it hard-errored ("source file could not be located", exit 2) on a declaration that
//! compiles cleanly under real rustc on every platform — a cross-dimension divergence closed by
//! PR #164 (`fix(guibiao): tolerate a resolved cfg_attr(path) candidate backing a module with no
//! plain file`). PR #164's own suite (`crates/guibiao/tests/cfg_attr_path_only_module_absence.rs`)
//! already pins 圭表 alone against the audited trigger and its corner cases; nothing had
//! previously fed the SAME shape to all three dimensions' real public entry points and asserted
//! they agree — this is that ledger, mirroring `dual_backed_module_conformance.rs`'s practice of
//! pinning every relevant outcome of a module-resolution rule, not only the happy one.
//!
//! Each dimension hand-writes its own resolution (三儀 ⊥ 三儀: no shared scanner code). Four
//! states are exercised: a stacked (per-platform) `cfg_attr(path)`-only declaration carrying a
//! violation, a single non-stacked `cfg_attr(path)`-only declaration carrying a violation (the fix
//! is not specific to "stacked"), a clean `cfg_attr(path)`-only declaration whose only probe lives
//! inside the remapped file, and the boundary the tolerance must NOT widen — every candidate
//! absent (no plain file, no resolved `cfg_attr(path)` target, no bare `#[cfg]`) — which must stay
//! a genuine constitution error (exit 2) in all three dimensions, never a silently-governed orphan.
//!
//! Exit codes are the claim; error wordings stay `errors_conformance.rs`'s concern.

#[path = "support/mod.rs"]
mod support;
use support::{TempFixture, guibiao_exit, hunyi_exit, louke_exit};

const REASON: &str =
    "conformance: a cfg_attr(path)-only module is governed exactly like a file-backed one";

/// The declared seam 漏刻 governs; spelled literally in the fixture bodies below (a `const` cannot
/// be interpolated into a macro-call string), alongside the mis-typed `"conformance-saem"` that
/// must react as probed-but-undeclared — a mismatch between the two is self-detecting (it would
/// flip every exit code these tests assert), matching `cfg_if_transparency_conformance.rs`'s
/// practice.
const SEAM: &str = "conformance-seam";

/// The forbidden target every violation-bearing fixture imports/exposes.
const FORBIDDEN_MOD: &str = "pub mod forbidden { pub struct Thing; }\n";

/// A top-level probe for the REAL declared seam, present in every violation fixture below so
/// 漏刻's own "declared-but-unprobed" direction never fires as an incidental confound — the only
/// finding a violation fixture can produce is the one embedded in the remapped module itself.
const TOP_LEVEL_PROBED: &str =
    "pub fn probed(o: u8) { assert_boundary!(\"conformance-seam\", o); }\n";

/// All three dimensions' violations in one file: the `use` 圭表 forbids, the public return type
/// 渾儀 forbids, and a probe naming an undeclared seam — 漏刻's own reaction direction expressed as
/// a violation, matching `cfg_if_transparency_conformance.rs`'s combined-fixture practice.
const IMP_VIOLATIONS: &str = "use crate::forbidden::Thing;\n\
                              pub fn leak() -> crate::forbidden::Thing { crate::forbidden::Thing }\n\
                              pub fn typo(o: u8) { assert_boundary!(\"conformance-saem\", o); }\n";

/// A harmless stub for the platform target that carries no content of interest — present only so
/// BOTH stacked candidates resolve to a real file, proving the tolerance is not accidentally keyed
/// to which one happens to hold the violation.
const IMP_STUB: &str = "pub fn noop() {}\n";

/// Clean, and carrying the declared seam's ONLY probe — proves 漏刻 counts a probe living inside a
/// `cfg_attr(path)`-only module as real coverage, not merely that the module resolves without
/// erroring (mirroring `cfg_if_transparency_conformance.rs`'s `ARM_CLEAN` double duty).
const IMP_CLEAN: &str = "pub fn probed(o: u8) { assert_boundary!(\"conformance-seam\", o); }\n";

/// A fixture whose crate root is `lib_body`, with `files` written alongside it in `src/`.
fn fixture(name: &str, lib_body: &str, files: &[(&str, &str)]) -> TempFixture {
    let fixture = TempFixture::new(name, lib_body);
    let src = fixture.lib().parent().expect("lib.rs has a parent");
    for (path, contents) in files {
        std::fs::write(src.join(path), contents).expect("write fixture file");
    }
    fixture
}

/// The audited PR #164 trigger reconstructed verbatim: TWO stacked, jointly-exhaustive
/// `#[cfg_attr(.., path = ..)]` remaps on one `pub mod imp;`, both targets present on disk, no
/// plain `imp.rs`/`imp/mod.rs`. All three dimensions must read the violating target and react, not
/// hard-error on the declaration's own absent conventional file.
#[test]
fn all_three_dimensions_govern_a_stacked_cfg_attr_path_only_module() {
    let package = "cfg-attr-path-only-stacked";
    let lib = format!(
        "{FORBIDDEN_MOD}{TOP_LEVEL_PROBED}\
         #[cfg_attr(unix, path = \"imp_unix.rs\")]\n\
         #[cfg_attr(not(unix), path = \"imp_other.rs\")]\n\
         pub mod imp;\n"
    );
    let fixture = fixture(
        package,
        &lib,
        &[("imp_unix.rs", IMP_VIOLATIONS), ("imp_other.rs", IMP_STUB)],
    );

    assert_eq!(
        guibiao_exit(package, fixture.manifest(), "crate::imp", REASON),
        1,
        "圭表: a stacked cfg_attr(path)-only module's forbidden `use` must react"
    );
    assert_eq!(
        hunyi_exit(package, fixture.manifest(), "crate::imp", REASON),
        1,
        "渾儀: a stacked cfg_attr(path)-only module's forbidden exposure must react"
    );
    assert_eq!(
        louke_exit(fixture.lib(), SEAM, REASON),
        1,
        "漏刻: a stacked cfg_attr(path)-only module's undeclared-seam probe must react"
    );
}

/// Control: a SINGLE, non-stacked `cfg_attr(path)`-only remap, same absence of a plain file. The
/// fix (and this ledger) is not specific to the stacked shape.
#[test]
fn all_three_dimensions_govern_a_single_cfg_attr_path_only_module() {
    let package = "cfg-attr-path-only-single";
    let lib = format!(
        "{FORBIDDEN_MOD}{TOP_LEVEL_PROBED}#[cfg_attr(unix, path = \"imp_unix.rs\")]\npub mod imp;\n"
    );
    let fixture = fixture(package, &lib, &[("imp_unix.rs", IMP_VIOLATIONS)]);

    assert_eq!(
        guibiao_exit(package, fixture.manifest(), "crate::imp", REASON),
        1,
        "圭表: a single cfg_attr(path)-only module's forbidden `use` must react"
    );
    assert_eq!(
        hunyi_exit(package, fixture.manifest(), "crate::imp", REASON),
        1,
        "渾儀: a single cfg_attr(path)-only module's forbidden exposure must react"
    );
    assert_eq!(
        louke_exit(fixture.lib(), SEAM, REASON),
        1,
        "漏刻: a single cfg_attr(path)-only module's undeclared-seam probe must react"
    );
}

/// The positive direction: a clean `cfg_attr(path)`-only module stays clean, and 漏刻 counts the
/// probe living inside its remapped file as real coverage — proving the boundary case below pins a
/// real regression, not a scanner that merely rejects everything it cannot conventionally resolve.
#[test]
fn all_three_dimensions_leave_a_clean_cfg_attr_path_only_module_clean() {
    let package = "cfg-attr-path-only-clean";
    let lib = format!("{FORBIDDEN_MOD}#[cfg_attr(unix, path = \"imp_unix.rs\")]\npub mod imp;\n");
    let fixture = fixture(package, &lib, &[("imp_unix.rs", IMP_CLEAN)]);

    assert_eq!(
        guibiao_exit(package, fixture.manifest(), "crate::imp", REASON),
        0,
        "圭表: a clean cfg_attr(path)-only module must not react"
    );
    assert_eq!(
        hunyi_exit(package, fixture.manifest(), "crate::imp", REASON),
        0,
        "渾儀: a clean cfg_attr(path)-only module must not react"
    );
    assert_eq!(
        louke_exit(fixture.lib(), SEAM, REASON),
        0,
        "漏刻: a probe inside a cfg_attr(path)-only module must count as real coverage"
    );
}

/// The boundary the tolerance must NOT widen: the `cfg_attr(path)` target is absent, there is no
/// plain conventional file, and the declaration carries no bare `#[cfg]` either — every candidate
/// is absent on every configuration, so this is a genuine, unrecoverable compile error under real
/// rustc. All three dimensions must still refuse to judge (exit 2), never silently govern an orphan.
#[test]
fn all_three_dimensions_agree_every_candidate_absent_stays_a_scan_error() {
    let package = "cfg-attr-path-only-absent";
    let lib = format!(
        "{FORBIDDEN_MOD}#[cfg_attr(windows, path = \"windows_only_imp.rs\")]\npub mod imp;\n"
    );
    // Deliberately does NOT create `windows_only_imp.rs`, `imp.rs`, or `imp/mod.rs`.
    let fixture = fixture(package, &lib, &[]);

    assert_eq!(
        guibiao_exit(package, fixture.manifest(), "crate::imp", REASON),
        2,
        "圭表: every candidate absent must stay a constitution error"
    );
    assert_eq!(
        hunyi_exit(package, fixture.manifest(), "crate::imp", REASON),
        2,
        "渾儀: every candidate absent must stay a constitution error"
    );
    assert_eq!(
        louke_exit(fixture.lib(), SEAM, REASON),
        2,
        "漏刻: every candidate absent must stay a constitution error"
    );
}
