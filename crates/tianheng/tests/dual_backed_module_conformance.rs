//! Cross-dimension conformance for resolving a plain `mod child;` to its conventional source file —
//! `child.rs` or `child/mod.rs` — across all four outcomes of that lookup, in all three dimensions.
//!
//! The ambiguous outcome (BOTH forms present) is the reason this suite exists: it is a genuine rustc
//! compile error (E0761) for a live declaration, independent of any `#[cfg]`. 漏刻's own
//! `resolve_external_module` (`crates/louke/src/audit/scan/probes.rs`) has always hard-errored on it; 圭表's
//! `reachability` gained the same reaction in 0.2.3, when this suite pinned the two of them; 渾儀's
//! `locate_module_file` silently returned the first form it probed until it joined them here, so an
//! item written in the unselected form escaped observation entirely.
//!
//! Each dimension hand-writes its own resolution (三儀 ⊥ 三儀: no shared scanner code), so the four
//! states are exercised **exhaustively** rather than sampled — that exhaustiveness is what makes the
//! ledger, not a shared implementation in a substrate crate, the drift reaction for this convention.
//! The unambiguous states are pinned too, not only the errors: without them "all three exit 2" could
//! hold for a fixture that no dimension can resolve at all.
//!
//! Exit codes are the claim. Error *wordings* are deliberately not compared: 圭表's and 漏刻's own
//! messages for this shape already differ from each other (a quoted full module path vs a backticked
//! bare name; a trailing rule clause present in one and absent in the other), so there is no twin to
//! be. Wording agreement stays `errors_conformance.rs`'s concern.

#[path = "support/mod.rs"]
mod support;
use support::{TempFixture, guibiao_exit, hunyi_exit, louke_exit};

const REASON: &str = "conformance: a plain `mod` resolves to exactly one file in every dimension";

/// Clean in every dimension: no `use` for 圭表, no forbidden type in a public signature for 渾儀, and
/// 漏刻's declared seam probe present so the runtime dimension is clean whenever the module resolves.
const CLEAN_CHILD: &str = "pub fn probed(o: u8) { assert_boundary!(\"conformance-seam\", o); }\n";

/// The clean body plus the exposure 渾儀's boundary forbids. Used for the **nested** form of the
/// dual-backed fixture so that tree *is* the measured false negative rather than merely resembling
/// it: with a clean `child.rs` beside it, 渾儀 read only the flat form and returned exit 0 on exactly
/// this input. A reverted fix therefore fails the assertion with the original defect, not a proxy.
const LEAKY_CHILD: &str = "pub fn probed(o: u8) { assert_boundary!(\"conformance-seam\", o); }\n\
                           pub fn leak() -> crate::forbidden::Thing { crate::forbidden::Thing }\n";

/// A fixture whose `crate::child` is backed by the requested conventional forms — the four
/// combinations of `flat` (`child.rs`) and `nested` (`child/mod.rs`) being the lookup's whole input
/// space. `None` omits that form entirely.
fn fixture(name: &str, flat: Option<&str>, nested: Option<&str>) -> TempFixture {
    let fixture = TempFixture::new(name, "pub mod child;\npub mod forbidden;\n");
    let src = fixture.lib().parent().expect("lib.rs has a parent");
    std::fs::write(src.join("forbidden.rs"), "pub struct Thing;\n").expect("write forbidden.rs");
    if let Some(body) = flat {
        std::fs::write(src.join("child.rs"), body).expect("write child.rs");
    }
    if let Some(body) = nested {
        std::fs::create_dir_all(src.join("child")).expect("mkdir child");
        std::fs::write(src.join("child").join("mod.rs"), body).expect("write child/mod.rs");
    }
    fixture
}

/// Both conventional forms at once: every dimension must refuse to judge (exit 2), never resolve to
/// one of the two and scan it as though the other were absent.
#[test]
fn all_three_dimensions_agree_a_dual_backed_module_is_a_scan_error() {
    let package = "dual-backed-both";
    let fixture = fixture(package, Some(CLEAN_CHILD), Some(LEAKY_CHILD));

    assert_eq!(
        guibiao_exit(package, fixture.manifest(), "crate::child", REASON),
        2,
        "圭表: a dual-backed module must be a constitution error (cannot judge)"
    );
    assert_eq!(
        hunyi_exit(package, fixture.manifest(), "crate::child", REASON),
        2,
        "渾儀: a dual-backed module must be a constitution error, never a first-form pick"
    );
    assert_eq!(
        louke_exit(fixture.lib(), "conformance-seam", REASON),
        2,
        "漏刻: a dual-backed module must be a constitution error (cannot judge)"
    );
}

/// Neither conventional form, with no `#[cfg]` gate to make the absence legitimate: every dimension
/// must refuse to judge rather than quietly dropping the module from what it observes.
#[test]
fn all_three_dimensions_agree_an_unconditionally_absent_module_is_a_scan_error() {
    let package = "dual-backed-neither";
    let fixture = fixture(package, None, None);

    assert_eq!(
        guibiao_exit(package, fixture.manifest(), "crate::child", REASON),
        2,
        "圭表: an unconditionally absent module file must be a constitution error"
    );
    assert_eq!(
        hunyi_exit(package, fixture.manifest(), "crate::child", REASON),
        2,
        "渾儀: an unconditionally absent module file must be a constitution error"
    );
    assert_eq!(
        louke_exit(fixture.lib(), "conformance-seam", REASON),
        2,
        "漏刻: an unconditionally absent module file must be a constitution error"
    );
}

/// Exactly `child.rs`: every dimension resolves it and finds the module clean. Pins that the
/// ambiguity reaction did not swallow the unambiguous flat form.
#[test]
fn all_three_dimensions_agree_a_flat_only_module_resolves_clean() {
    let package = "dual-backed-flat";
    let fixture = fixture(package, Some(CLEAN_CHILD), None);

    assert_eq!(
        guibiao_exit(package, fixture.manifest(), "crate::child", REASON),
        0,
        "圭表: a `child.rs`-only module resolves and is clean"
    );
    assert_eq!(
        hunyi_exit(package, fixture.manifest(), "crate::child", REASON),
        0,
        "渾儀: a `child.rs`-only module resolves and is clean"
    );
    assert_eq!(
        louke_exit(fixture.lib(), "conformance-seam", REASON),
        0,
        "漏刻: a `child.rs`-only module resolves and its declared seam is probed"
    );
}

/// Exactly `child/mod.rs`: the same agreement for the nested form, which 渾儀 previously reached only
/// when the flat form happened to be absent.
#[test]
fn all_three_dimensions_agree_a_nested_only_module_resolves_clean() {
    let package = "dual-backed-nested";
    let fixture = fixture(package, None, Some(CLEAN_CHILD));

    assert_eq!(
        guibiao_exit(package, fixture.manifest(), "crate::child", REASON),
        0,
        "圭表: a `child/mod.rs`-only module resolves and is clean"
    );
    assert_eq!(
        hunyi_exit(package, fixture.manifest(), "crate::child", REASON),
        0,
        "渾儀: a `child/mod.rs`-only module resolves and is clean"
    );
    assert_eq!(
        louke_exit(fixture.lib(), "conformance-seam", REASON),
        0,
        "漏刻: a `child/mod.rs`-only module resolves and its declared seam is probed"
    );
}
