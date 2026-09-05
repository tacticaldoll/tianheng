//! Cross-dimension conformance for a module declared ONLY via one or more resolved
//! `#[cfg_attr(pred, path = "...")]` remaps — no plain conventional file (`imp.rs` /
//! `imp/mod.rs`) and no unconditional `#[path]`.
//!
//! 渾儀's `has_backing_source` walk (`crates/hunyi/src/scan/items.rs`) and 漏刻's own copy
//! (`crates/louke/src/audit/scan/probes.rs`) already tolerated this exact shape, treating a resolved
//! `cfg_attr(path)` candidate as legitimate grounds for the plain file's own absence. 圭表's own
//! module-boundary reachability walk (`crates/guibiao/src/module_scan/reachability/walk.rs`) did
//! not: it hard-errored ("source file could not be located", exit 2) on a declaration that
//! compiles cleanly under real rustc on every platform — a cross-dimension divergence since closed.
//! `crates/guibiao/tests/cfg_attr_path_only_module_absence.rs`
//! already pins 圭表 alone against the audited trigger and its corner cases; nothing had
//! previously fed the SAME shape to all three dimensions' real public entry points and asserted
//! they agree — this is that ledger, mirroring `dual_backed_module_conformance.rs`'s practice of
//! pinning every relevant outcome of a module-resolution rule, not only the happy one.
//!
//! Each dimension hand-writes its own resolution (三儀 ⊥ 三儀: no shared scanner code). Five
//! states are exercised: a stacked (per-platform) `cfg_attr(path)`-only declaration carrying a
//! violation, a single non-stacked `cfg_attr(path)`-only declaration carrying a violation (the fix
//! is not specific to "stacked"), a clean `cfg_attr(path)`-only declaration whose only probe lives
//! inside the remapped file, and the boundary the tolerance must NOT widen — every candidate
//! absent (no plain file, no resolved `cfg_attr(path)` target, no bare `#[cfg]`) — which must stay
//! a genuine constitution error (exit 2) in all three dimensions, never a silently-governed orphan.
//!
//! The fifth is the SIBLING shape, added after it drifted the same way: a `cfg_attr(path)` on an
//! **inline** `mod x { mod y; }`, where the remap names the base directory the body's file-form
//! children resolve from rather than a file to read. 漏刻's specification already required it and 圭表
//! did not — the identical divergence this ledger exists for, one shape over, found because nothing
//! fed THIS shape to all three entry points either. Closed by
//! `fix(guibiao): follow a conditional path remap on an inline mod to its children's base`; pinned
//! here so the class is closed rather than the instance.
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

/// The audited trigger reconstructed verbatim: TWO stacked, jointly-exhaustive
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

/// Two `path` remaps NESTED INSIDE ONE `cfg_attr`, with the violating target declared SECOND.
///
/// **The sibling direction above stacks two ATTRIBUTES, one `path` each, and that is a different
/// axis.** A reader taking only the first `path` in a span still passes it, because each span holds
/// exactly one. Measured against rustc (edition 2021, `--crate-type lib`) this declaration compiles
/// cleanly on Linux with only the second target on disk and neither the first nor a plain `imp.rs`
/// present, so a dimension answering the first alone resolves nothing: with no conventional file to
/// fall back to it reports the declaration unresolvable — exit 2 over valid code — and with one it
/// governs the wrong file and the probe goes unseen.
///
/// 圭表 already unioned every `path =` across nested groups. 渾儀 and 漏刻 each took the first, so
/// this shape is where the three parted, and the violating target is declared second on purpose:
/// first-declared would pass on a reader that never looks past it.
#[test]
fn all_three_dimensions_read_every_path_nested_in_one_cfg_attr() {
    let package = "cfg-attr-path-only-nested";
    let lib = format!(
        "{FORBIDDEN_MOD}{TOP_LEVEL_PROBED}\
         #[cfg_attr(unix, cfg_attr(target_os = \"none\", path = \"imp_never.rs\"), \
         cfg_attr(not(target_os = \"none\"), path = \"imp_here.rs\"))]\n\
         pub mod imp;\n"
    );
    // Only the SECOND target exists, as rustc admits: the first names a target this build has no
    // file for, and the declaration still compiles.
    let fixture = fixture(package, &lib, &[("imp_here.rs", IMP_VIOLATIONS)]);

    assert_eq!(
        guibiao_exit(package, fixture.manifest(), "crate::imp", REASON),
        1,
        "圭表: the second `path` nested in one cfg_attr must be read"
    );
    assert_eq!(
        hunyi_exit(package, fixture.manifest(), "crate::imp", REASON),
        1,
        "渾儀: the second `path` nested in one cfg_attr must be read"
    );
    assert_eq!(
        louke_exit(fixture.lib(), SEAM, REASON),
        1,
        "漏刻: the second `path` nested in one cfg_attr must be read"
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

/// 渾儀 refuses a module target it cannot READ, rather than tolerating it as absent.
///
/// **`is_file()` answers `false` for two different facts**, and the cfg tolerance is what an ABSENCE is
/// owed — so a target this reader merely could not stat was swallowed by it, with whatever the subtree
/// holds going unobserved. 漏刻 was repaired first; this is why the other two moved with it, since a
/// window arguing the three must agree about module resolution cannot leave two of them disagreeing about
/// what `false` means.
///
/// **The boundary governs a module that resolves either way, and that is the whole design of this
/// fixture.** A first version pointed it at the unreadable module itself and passed before the repair as
/// well as after: tolerating the module makes it unknown, an unknown boundary target is a constitution
/// error, and the exit code is `2` for a reason that has nothing to do with reading. Measured — `guibiao=2
/// hunyi=2` both before and after. Governing `crate::readable` instead makes the tolerated run **clean**,
/// so the two states differ.
///
/// Measured as uid 1000, which is why this refuses to skip in silence under `TIANHENG_WORKSPACE_TESTS`:
/// root bypasses mode bits entirely and would make it vacuous.
///
/// Negative run, with only 渾儀's reader reverted:
///
/// ```text
/// assertion `left == right` failed: 渾儀: an unreadable target is not an absent one
///   left: 0
///  right: 2
/// ```
///
/// Clean, over a subtree it could not open.
///
/// **圭表 is not asserted here, and that is a declared gap rather than an omission.** Its reader takes the
/// same criterion, but measured on this fixture it exits `2` **before and after** — loud for a reason this
/// change does not touch — so an assertion over it would pass either way and say nothing. No fixture has
/// been found that separates the two states for 圭表, so its repair travels on symmetry with the two that
/// were seen to fail, and `BACKLOG.md` carries the search. 漏刻's own directions hold its half.
#[test]
#[cfg(unix)]
fn a_module_target_that_cannot_be_read_is_not_tolerated_as_absent() {
    use std::os::unix::fs::PermissionsExt;

    let package = "cfg-attr-path-only-unreadable";
    // `readable` is what the boundary governs and it holds nothing forbidden, so a run that tolerates the
    // unreadable sibling is CLEAN. `imp` is cfg-gated — the arm that is owed a real absence.
    let lib = format!(
        "{FORBIDDEN_MOD}{TOP_LEVEL_PROBED}pub mod readable {{ pub fn ok() {{}} }}\n\
         #[cfg(feature = \"gated\")]\npub mod imp;\n"
    );
    let fixture = fixture(package, &lib, &[]);

    let dir = fixture
        .lib()
        .parent()
        .expect("the lib has a directory")
        .join("imp");
    std::fs::create_dir_all(&dir).expect("create the module directory");
    std::fs::write(dir.join("mod.rs"), IMP_VIOLATIONS).expect("write the module file");
    let restore = std::fs::metadata(&dir)
        .expect("read the mode")
        .permissions();
    std::fs::set_permissions(&dir, std::fs::Permissions::from_mode(0o000)).expect("restrict it");

    let restricted = !dir.join("mod.rs").is_file();
    let exits =
        restricted.then(|| hunyi_exit(package, fixture.manifest(), "crate::readable", REASON));
    std::fs::set_permissions(&dir, restore).expect("restore the mode");

    let Some(hunyi) = exits else {
        assert!(
            std::env::var_os("TIANHENG_WORKSPACE_TESTS").is_none(),
            "mode 000 did not restrict the target — running as root would make this direction vacuous"
        );
        return;
    };
    assert_eq!(hunyi, 2, "渾儀: an unreadable target is not an absent one");
}

/// The sibling shape: a `cfg_attr(path)` on an **inline** `mod x { … }`, naming the base directory
/// the body's own file-form children resolve from. 圭表 followed only the *unconditional* `#[path]`
/// form here and reported a missing-module constitution error on source that compiles cleanly, while
/// 漏刻's specification already required the conditional form — the same one-dimension divergence the
/// file-form cases above record, on the shape nothing had fed to all three entry points.
///
/// All three must observe the child beneath the remapped base and react to what it carries. The two
/// static dimensions govern `crate::imp::inner` directly — the module that actually holds the
/// violation — rather than its parent, so neither dimension's own default `ScanDepth` decides the
/// answer: what is under test is whether that module is REACHED at all, not how far a depth reaches.
#[test]
fn all_three_dimensions_agree_a_conditional_remap_on_an_inline_mod_relocates_its_child_base() {
    let package = "cfg-attr-inline-base";
    let lib = format!(
        "{FORBIDDEN_MOD}{TOP_LEVEL_PROBED}\
         #[cfg_attr(unix, path = \"unix_dir\")]\n\
         #[cfg_attr(not(unix), path = \"other_dir\")]\n\
         pub mod imp {{\n    pub mod inner;\n}}\n"
    );
    let fixture = TempFixture::new(package, &lib);
    let src = fixture.lib().parent().expect("lib.rs has a parent");
    // Both platform bases exist, so the union is real rather than keyed to whichever one happens to
    // hold the violation — the same practice the stacked file-form case above uses.
    for dir in ["unix_dir", "other_dir"] {
        std::fs::create_dir_all(src.join(dir)).expect("create remapped base dir");
    }
    std::fs::write(src.join("unix_dir/inner.rs"), IMP_VIOLATIONS).expect("write violating child");
    std::fs::write(src.join("other_dir/inner.rs"), IMP_STUB).expect("write stub child");

    assert_eq!(
        guibiao_exit(package, fixture.manifest(), "crate::imp::inner", REASON),
        1,
        "圭表: the child beneath a conditional inline-mod base must be observed and react"
    );
    assert_eq!(
        hunyi_exit(package, fixture.manifest(), "crate::imp::inner", REASON),
        1,
        "渾儀: the child beneath a conditional inline-mod base must be observed and react"
    );
    assert_eq!(
        louke_exit(fixture.lib(), SEAM, REASON),
        1,
        "漏刻: the undeclared-seam probe beneath a conditional inline-mod base must react"
    );
}

/// A **raw-identifier** spelling of the remap is the same remap to all three dimensions.
///
/// `r#` changes an identifier's lexical spelling and not the name it spells, so `r#path` names the built-in
/// `path` attribute and `r#cfg_attr` names `cfg_attr`. Measured against rustc 1.96.0, edition 2021,
/// `--crate-type lib`: `#[cfg_attr(unix, r#path = "imp_unix.rs")] pub mod plat;` compiles with only
/// `imp_unix.rs` on disk, and the nested `#[cfg_attr(unix, r#cfg_attr(target_os = "linux", path =
/// "imp_linux.rs"))]` compiles with only `imp_linux.rs`.
///
/// 圭表 and 漏刻 each say so in their own byte scanners, in nearly the same words — *a raw identifier is ONE
/// segment*. 渾儀 reads its attribute names through `syn`, whose `Path::is_ident` compares the ident as
/// written: proc-macro2's `PartialEq<str>` requires the compared string to carry `r#` when the ident is raw,
/// so `r#path` was not `path` to it and the remap was invisible. The crate had `strip_raw` already and
/// applied it to module identifiers, not to attribute names.
#[test]
fn all_three_dimensions_read_a_raw_identifier_spelling_of_a_cfg_attr_path() {
    let package = "cfg-attr-path-only-raw-ident";
    let lib = format!(
        "{FORBIDDEN_MOD}{TOP_LEVEL_PROBED}#[cfg_attr(unix, r#path = \"imp_unix.rs\")]\npub mod imp;\n"
    );
    let fixture = fixture(package, &lib, &[("imp_unix.rs", IMP_VIOLATIONS)]);

    assert_eq!(
        guibiao_exit(package, fixture.manifest(), "crate::imp", REASON),
        1,
        "圭表: `r#path` names the built-in `path`, so the remapped module's forbidden `use` must react"
    );
    assert_eq!(
        hunyi_exit(package, fixture.manifest(), "crate::imp", REASON),
        1,
        "渾儀: `r#path` names the built-in `path`, so the remapped module's forbidden exposure must react"
    );
    assert_eq!(
        louke_exit(fixture.lib(), SEAM, REASON),
        1,
        "漏刻: `r#path` names the built-in `path`, so the remapped module's undeclared seam must react"
    );
}

/// The same, one nesting level in: a raw-identifier `cfg_attr` **wrapping** the remap.
///
/// The outer attribute is spelled plainly, so this is the applied-meta position all three readers descend
/// into — and the segment they have to recognise there is `r#cfg_attr`. 圭表's own scanner records having
/// been measured on exactly this spelling while closing a qualified look-alike; this asserts the three
/// agree about it rather than each recording it alone.
#[test]
fn all_three_dimensions_read_a_raw_identifier_cfg_attr_wrapping_a_path() {
    let package = "cfg-attr-path-only-raw-nested";
    let lib = format!(
        "{FORBIDDEN_MOD}{TOP_LEVEL_PROBED}\
         #[cfg_attr(unix, r#cfg_attr(not(target_os = \"none\"), path = \"imp_unix.rs\"))]\n\
         pub mod imp;\n"
    );
    let fixture = fixture(package, &lib, &[("imp_unix.rs", IMP_VIOLATIONS)]);

    assert_eq!(
        guibiao_exit(package, fixture.manifest(), "crate::imp", REASON),
        1,
        "圭表: a raw-identifier `cfg_attr` is the built-in, so its applied `path` must be read"
    );
    assert_eq!(
        hunyi_exit(package, fixture.manifest(), "crate::imp", REASON),
        1,
        "渾儀: a raw-identifier `cfg_attr` is the built-in, so its applied `path` must be read"
    );
    assert_eq!(
        louke_exit(fixture.lib(), SEAM, REASON),
        1,
        "漏刻: a raw-identifier `cfg_attr` is the built-in, so its applied `path` must be read"
    );
}

/// The half the Core Contract forbids: a conventional file **beside** the raw-identifier remap.
///
/// `all_three_dimensions_read_a_raw_identifier_spelling_of_a_cfg_attr_path` and its nested sibling leave
/// 渾儀 exiting `2` on source rustc compiles, which is loud. This one is
/// the silent shape. `imp.rs` exists and is clean; the remap names `imp_unix.rs`, which carries every
/// violation. Measured against rustc 1.96.0: with both present it is the **remapped** file rustc compiles.
///
/// A reader that misses the remap therefore governs `imp.rs` — a file the build does not contain — reports
/// clean, and leaves whatever the build does contain unobserved. That is a false negative in the one
/// dimension, produced by a spelling of the governed code, which is what *no spelling, alias, re-export,
/// `cfg` arm, or macro form escapes observation* forbids.
#[test]
fn a_conventional_file_beside_a_raw_identifier_remap_does_not_hide_the_remapped_one() {
    let package = "cfg-attr-path-raw-ident-beside-conventional";
    let lib = format!(
        "{FORBIDDEN_MOD}{TOP_LEVEL_PROBED}#[cfg_attr(unix, r#path = \"imp_unix.rs\")]\npub mod imp;\n"
    );
    let fixture = fixture(
        package,
        &lib,
        &[("imp_unix.rs", IMP_VIOLATIONS), ("imp.rs", IMP_STUB)],
    );

    assert_eq!(
        guibiao_exit(package, fixture.manifest(), "crate::imp", REASON),
        1,
        "圭表: the clean conventional file does not hide the remapped one's forbidden `use`"
    );
    assert_eq!(
        hunyi_exit(package, fixture.manifest(), "crate::imp", REASON),
        1,
        "渾儀: the clean conventional file does not hide the remapped one's forbidden exposure"
    );
    assert_eq!(
        louke_exit(fixture.lib(), SEAM, REASON),
        1,
        "漏刻: the clean conventional file does not hide the remapped one's undeclared seam"
    );
}
