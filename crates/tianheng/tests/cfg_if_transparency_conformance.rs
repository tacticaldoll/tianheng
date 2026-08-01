//! Cross-dimension conformance for the one **transparent control-flow macro**, `cfg_if!`: its arms
//! wrap human-authored items without transforming their identities, so what an adopter writes inside
//! an arm is real, compiled code and every dimension that reads the file must observe it.
//!
//! One fixture body carries both dimensions' constructs at once — a forbidden `use` (圭表's
//! observation) and a forbidden public return type (渾儀's) — so the two cannot be pinned on
//! separately-drifting inputs. 圭表 gained this transparency in 0.2.3; 渾儀 handled no
//! `syn::Item::Macro` at all until this suite joined it, and the measured gap was exactly this
//! fixture: 圭表 exit 1, 渾儀 **exit 0** on the identical file, which is an exposure false negative —
//! the one bug class the core contract forbids.
//!
//! **漏刻 is deliberately absent.** Its own scanner has no parser and reads macro bodies by byte
//! (`foreign_macro_body_end`, called in two independent passes — module-declaration collection and
//! probe scanning), so giving it the same transparency needs 圭表's brace-kind model in both places:
//! a different cost class, sequenced as its own change with its own spike. Until that lands this
//! ledger pins two of three dimensions, stated here rather than left to look like an oversight. When
//! it lands, 漏刻 joins these same tests rather than getting a suite of its own.
//!
//! Exit codes are the claim; error wordings stay `errors_conformance.rs`'s concern.

use std::path::Path;

use guibiao::{Constitution as GnomonConstitution, ModuleBoundary};
use hunyi::SemanticBoundary;

#[path = "support/mod.rs"]
mod support;
use support::TempFixture;

const REASON: &str = "conformance: a cfg_if arm's contents are real code in every dimension";

/// Both dimensions' violations in one arm: the `use` 圭表 forbids, and the public return type 渾儀
/// forbids. The return path is written in full rather than through the arm's own `use`, so 渾儀's
/// reaction measures arm-item observation alone and not, additionally, whether the arm's `use` fed
/// the resolver.
const ARM_VIOLATIONS: &str = "cfg_if::cfg_if! {\n\
                              if #[cfg(unix)] {\n\
                              use crate::forbidden::Thing;\n\
                              pub fn leak() -> crate::forbidden::Thing { crate::forbidden::Thing }\n\
                              }\n\
                              }\n";

/// The identical two violations written directly in the module — the control. Without it, "both
/// dimensions react" for the wrapped form could not be distinguished from a fixture that reacts for
/// some unrelated reason, and a clean wrapped result could not be read as a false negative.
const TOP_LEVEL_VIOLATIONS: &str = "use crate::forbidden::Thing;\n\
                                    pub fn leak() -> crate::forbidden::Thing { crate::forbidden::Thing }\n";

/// A `cfg_if!` arm carrying neither construct. Pins that transparency is observation, not a reaction
/// to the mere presence of the macro: a crate using `cfg_if!` cleanly stays clean in both dimensions.
const ARM_CLEAN: &str = "cfg_if::cfg_if! {\n\
                         if #[cfg(unix)] {\n\
                         pub fn fine() -> u8 { 0 }\n\
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

/// `pub mod child;` declared only inside a `cfg_if!` arm. A dimension blind to the arm never reaches
/// `child.rs` at all — 圭表 would drop it from the reachable set, 渾儀 would call the anchor unknown.
const ARM_DECLARED_LIB: &str = "pub mod forbidden;\n\
                                cfg_if::cfg_if! {\n\
                                if #[cfg(unix)] {\n\
                                pub mod child;\n\
                                }\n\
                                }\n";

fn guibiao_exit(package: &str, manifest: &Path) -> u8 {
    let constitution = GnomonConstitution::new(package).boundary(
        ModuleBoundary::in_crate(package)
            .module("crate::child")
            .must_not_import("crate::forbidden")
            .because(REASON),
    );
    guibiao::check(&constitution, manifest).exit_code()
}

fn hunyi_exit(package: &str, manifest: &Path) -> u8 {
    let boundary = SemanticBoundary::in_crate(package)
        .module("crate::child")
        .must_not_expose("crate::forbidden::Thing")
        .because(REASON);
    hunyi::check(&[boundary], manifest).exit_code()
}

/// The measured false negative, now closed: one file, one `cfg_if!` arm, both dimensions' forbidden
/// constructs inside it. Before this change 圭表 returned 1 and 渾儀 returned 0 on exactly this input.
#[test]
fn both_dimensions_react_to_violations_inside_a_cfg_if_arm() {
    let package = "cfg-if-arm-violations";
    let fixture = fixture(package, PLAIN_LIB, ARM_VIOLATIONS);

    assert_eq!(
        guibiao_exit(package, fixture.manifest()),
        1,
        "圭表: a forbidden `use` inside a cfg_if arm must react"
    );
    assert_eq!(
        hunyi_exit(package, fixture.manifest()),
        1,
        "渾儀: a forbidden exposure inside a cfg_if arm must react, not pass as an opaque macro"
    );
}

/// The control: the identical constructs at module top level. Establishes that both boundaries react
/// to this fixture at all, so the wrapped case above measures transparency and nothing else.
#[test]
fn both_dimensions_react_to_the_same_violations_at_top_level() {
    let package = "cfg-if-top-level";
    let fixture = fixture(package, PLAIN_LIB, TOP_LEVEL_VIOLATIONS);

    assert_eq!(
        guibiao_exit(package, fixture.manifest()),
        1,
        "圭表: the control forbidden `use` must react"
    );
    assert_eq!(
        hunyi_exit(package, fixture.manifest()),
        1,
        "渾儀: the control forbidden exposure must react"
    );
}

/// A clean `cfg_if!` arm stays clean in both dimensions — transparency observes contents, it does not
/// react to the macro.
#[test]
fn both_dimensions_leave_a_clean_cfg_if_arm_clean() {
    let package = "cfg-if-arm-clean";
    let fixture = fixture(package, PLAIN_LIB, ARM_CLEAN);

    assert_eq!(
        guibiao_exit(package, fixture.manifest()),
        0,
        "圭表: a clean cfg_if arm must not react"
    );
    assert_eq!(
        hunyi_exit(package, fixture.manifest()),
        0,
        "渾儀: a clean cfg_if arm must not react"
    );
}

/// The module-graph half: `child.rs` is reachable only through a `mod` declaration written inside an
/// arm. Both dimensions must descend it — a module missed here costs its whole file's observation,
/// not one fact.
#[test]
fn both_dimensions_reach_a_module_declared_only_inside_a_cfg_if_arm() {
    let package = "cfg-if-arm-declared-mod";
    let fixture = fixture(package, ARM_DECLARED_LIB, TOP_LEVEL_VIOLATIONS);

    assert_eq!(
        guibiao_exit(package, fixture.manifest()),
        1,
        "圭表: an arm-declared module's forbidden `use` must react"
    );
    assert_eq!(
        hunyi_exit(package, fixture.manifest()),
        1,
        "渾儀: an arm-declared module must resolve as an anchor and its exposure must react"
    );
}
