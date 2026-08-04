//! Cross-dimension conformance for the one **transparent control-flow macro**, `cfg_if!`: its arms
//! wrap human-authored items without transforming their identities, so what an adopter writes inside
//! an arm is real, compiled code and every dimension that reads the file must observe it.
//!
//! One fixture body carries all three dimensions' constructs at once — a forbidden `use` (圭表's
//! observation), a forbidden public return type (渾儀's), and a declared seam's only probe (漏刻's) —
//! so the three cannot be pinned on separately-drifting inputs. Each dimension hand-writes its own
//! recognition of the macro (三儀 ⊥ 三儀: the same rule, never a shared scanner), which is exactly
//! why the ledger exists.
//!
//! Each joined after its own measured gap on this fixture shape. 圭表 gained transparency in 0.2.3.
//! 渾儀 handled no `syn::Item::Macro` at all: 圭表 exit 1, 渾儀 **exit 0** on the identical file — an
//! exposure false negative, the one bug class the core contract forbids. 漏刻 skipped a `cfg_if!`
//! body like any foreign macro in both of its passes, which broke two of its three reaction
//! directions in both error directions at once: a typo'd seam and an un-auditable probe inside an arm
//! escaped entirely, while a seam whose only real probe lived in an arm was reported **unprobed** — a
//! false alarm against coverage the adopter actually had.
//!
//! Exit codes are the claim; error wordings stay `errors_conformance.rs`'s concern.

#[path = "support/mod.rs"]
mod support;
use support::{TempFixture, guibiao_exit, hunyi_exit, louke_exit};

const REASON: &str = "conformance: a cfg_if arm's contents are real code in every dimension";

/// The declared seam 漏刻 governs. The fixtures below spell it literally (a `const` cannot be
/// interpolated into one), alongside the mis-typed `"conformance-saem"` that must react as
/// probed-but-undeclared; a mismatch between the two is self-detecting rather than silent — it would
/// flip every exit code these tests assert.
const SEAM: &str = "conformance-seam";

/// All three dimensions' violations in one arm: the `use` 圭表 forbids, the public return type 渾儀
/// forbids, and a probe naming an undeclared seam — 漏刻's own reaction direction, expressed as a
/// violation so the three are pinned the same way rather than one of them asserting absence. The
/// return path is written in full rather than through the arm's own `use`, so 渾儀's reaction measures
/// arm-item observation alone and not, additionally, whether the arm's `use` fed the resolver.
const ARM_VIOLATIONS: &str = "cfg_if::cfg_if! {\n\
                              if #[cfg(unix)] {\n\
                              use crate::forbidden::Thing;\n\
                              pub fn leak() -> crate::forbidden::Thing { crate::forbidden::Thing }\n\
                              pub fn typo(o: u8) { assert_boundary!(\"conformance-saem\", o); }\n\
                              }\n\
                              }\n";

/// The identical three violations written directly in the module — the control. Without it, "all
/// three dimensions react" for the wrapped form could not be distinguished from a fixture that reacts
/// for some unrelated reason, and a clean wrapped result could not be read as a false negative.
const TOP_LEVEL_VIOLATIONS: &str = "use crate::forbidden::Thing;\n\
                                    pub fn leak() -> crate::forbidden::Thing { crate::forbidden::Thing }\n\
                                    pub fn typo(o: u8) { assert_boundary!(\"conformance-saem\", o); }\n";

/// A `cfg_if!` arm carrying no forbidden construct — and carrying the declared seam's **only** probe,
/// which makes this fixture do double duty: it pins that transparency is observation rather than a
/// reaction to the macro's presence (圭表 and 渾儀 stay clean), and that 漏刻 counts an arm's probe as
/// real coverage (a 漏刻 blind to the arm reports the seam unprobed and fails here).
const ARM_CLEAN: &str = "cfg_if::cfg_if! {\n\
                         if #[cfg(unix)] {\n\
                         pub fn fine() -> u8 { 0 }\n\
                         pub fn probed(o: u8) { assert_boundary!(\"conformance-seam\", o); }\n\
                         }\n\
                         }\n";

/// A fixture whose `crate::child` body is `child_body`, with `crate::forbidden` present to be
/// imported and exposed. `lib_body` declares the child (plainly, or from inside an arm).
fn fixture(name: &str, lib_body: &str, child_body: &str) -> TempFixture {
    let fixture = TempFixture::new(name, lib_body);
    let src = fixture.lib().parent().expect("lib.rs has a parent");
    std::fs::write(src.join("forbidden.rs"), "pub struct Thing;\n").expect("write forbidden.rs");
    std::fs::write(src.join("child.rs"), child_body).expect("write child.rs");
    fixture
}

const PLAIN_LIB: &str = "pub mod child;\npub mod forbidden;\n";

/// [`PLAIN_LIB`] plus the declared seam's probe at the crate root, so 漏刻's coverage direction is
/// satisfied and the only finding a violation fixture can produce is its own — never the incidental
/// "declared seam has no probe" that would fire on any fixture lacking one.
const PLAIN_LIB_PROBED: &str = "pub mod child;\npub mod forbidden;\n\
                                pub fn probed(o: u8) { assert_boundary!(\"conformance-seam\", o); }\n";

/// `pub mod child;` declared only inside a `cfg_if!` arm. A dimension blind to the arm never reaches
/// `child.rs` at all — 圭表 would drop it from the reachable set, 渾儀 would call the anchor unknown,
/// and 漏刻 would never scan the file for probes.
const ARM_DECLARED_LIB: &str = "pub mod forbidden;\n\
                                pub fn probed(o: u8) { assert_boundary!(\"conformance-seam\", o); }\n\
                                cfg_if::cfg_if! {\n\
                                if #[cfg(unix)] {\n\
                                pub mod child;\n\
                                }\n\
                                }\n";

/// The measured false negatives, now closed: one file, one `cfg_if!` arm, all three dimensions'
/// forbidden constructs inside it. Before their respective changes 圭表 returned 1 while 渾儀 returned
/// 0, and 漏刻 returned 0 — each of the latter two an escape on the identical input.
#[test]
fn all_three_dimensions_react_to_violations_inside_a_cfg_if_arm() {
    let package = "cfg-if-arm-violations";
    let fixture = fixture(package, PLAIN_LIB_PROBED, ARM_VIOLATIONS);

    assert_eq!(
        guibiao_exit(package, fixture.manifest(), "crate::child", REASON),
        1,
        "圭表: a forbidden `use` inside a cfg_if arm must react"
    );
    assert_eq!(
        hunyi_exit(package, fixture.manifest(), "crate::child", REASON),
        1,
        "渾儀: a forbidden exposure inside a cfg_if arm must react, not pass as an opaque macro"
    );
    assert_eq!(
        louke_exit(fixture.lib(), SEAM, REASON),
        1,
        "漏刻: a probe naming an undeclared seam inside a cfg_if arm must react, not escape with the body"
    );
}

/// The control: the identical constructs at module top level. Establishes that all three boundaries
/// react to this fixture at all, so the wrapped case above measures transparency and nothing else.
#[test]
fn all_three_dimensions_react_to_the_same_violations_at_top_level() {
    let package = "cfg-if-top-level";
    let fixture = fixture(package, PLAIN_LIB_PROBED, TOP_LEVEL_VIOLATIONS);

    assert_eq!(
        guibiao_exit(package, fixture.manifest(), "crate::child", REASON),
        1,
        "圭表: the control forbidden `use` must react"
    );
    assert_eq!(
        hunyi_exit(package, fixture.manifest(), "crate::child", REASON),
        1,
        "渾儀: the control forbidden exposure must react"
    );
    assert_eq!(
        louke_exit(fixture.lib(), SEAM, REASON),
        1,
        "漏刻: the control undeclared-seam probe must react"
    );
}

/// A clean `cfg_if!` arm stays clean in all three dimensions. For 圭表 and 渾儀 that pins transparency
/// as observation rather than a reaction to the macro's presence; for 漏刻 the same fixture is the
/// positive direction, since the arm holds the declared seam's ONLY probe — a 漏刻 blind to the arm
/// reports it unprobed and fails here.
#[test]
fn all_three_dimensions_leave_a_clean_cfg_if_arm_clean() {
    let package = "cfg-if-arm-clean";
    let fixture = fixture(package, PLAIN_LIB, ARM_CLEAN);

    assert_eq!(
        guibiao_exit(package, fixture.manifest(), "crate::child", REASON),
        0,
        "圭表: a clean cfg_if arm must not react"
    );
    assert_eq!(
        hunyi_exit(package, fixture.manifest(), "crate::child", REASON),
        0,
        "渾儀: a clean cfg_if arm must not react"
    );
    assert_eq!(
        louke_exit(fixture.lib(), SEAM, REASON),
        0,
        "漏刻: a probe inside a cfg_if arm must count as real coverage"
    );
}

/// The module-graph half: `child.rs` is reachable only through a `mod` declaration written inside an
/// arm. All three must descend it — a module missed here costs its whole file's observation, not one
/// fact: 圭表 loses its imports, 渾儀 cannot even name it as an anchor, 漏刻 never scans it for probes.
#[test]
fn all_three_dimensions_reach_a_module_declared_only_inside_a_cfg_if_arm() {
    let package = "cfg-if-arm-declared-mod";
    let fixture = fixture(package, ARM_DECLARED_LIB, TOP_LEVEL_VIOLATIONS);

    assert_eq!(
        guibiao_exit(package, fixture.manifest(), "crate::child", REASON),
        1,
        "圭表: an arm-declared module's forbidden `use` must react"
    );
    assert_eq!(
        hunyi_exit(package, fixture.manifest(), "crate::child", REASON),
        1,
        "渾儀: an arm-declared module must resolve as an anchor and its exposure must react"
    );
    assert_eq!(
        louke_exit(fixture.lib(), SEAM, REASON),
        1,
        "漏刻: an arm-declared module's undeclared-seam probe must react"
    );
}
