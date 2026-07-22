//! Cross-scanner conformance matrix — 圭表 (`guibiao`) and 漏刻 (`louke`) each hand-roll their own
//! lexical hygiene (comment/string/macro-body skipping) independently, by design (三儀 ⊥ 三儀: no
//! shared scanner code). Each dimension's own test suite pins its OWN handling of tricky lexical
//! cases, but nothing had previously fed the SAME literal source snippet to both and asserted they
//! agree — so a lexical fix landing in one dimension could silently remain absent in its sibling
//! (BACKLOG: "圭表 and 漏刻 have accumulated related lexical repairs around module/path handling and
//! nested block comments, but no executable parity ledger says where their neutral token behavior
//! should agree").
//!
//! This is that ledger: each case below writes ONE fixture source file containing both a
//! guibiao-relevant construct (a `use` the boundary forbids) and a louke-relevant construct (an
//! `assert_boundary!` probe), wrapped in the SAME tricky lexical shape, and asserts both dimensions
//! agree on whether it is real code or inert (comment / string / macro-generated). Pinning parity,
// not deciding extraction (PROJECT.md's judgment-neutral-parsing-primitive direction stays gated on
//! a third forcing event); a genuine future divergence is a separate false-negative closure, not a
//! reason to weaken this ledger.

use std::path::Path;

use guibiao::{Constitution as GnomonConstitution, ModuleBoundary, Outcome as GnomonOutcome};
use louke::{RuntimeBoundary, audit_probe_coverage};

#[path = "support/mod.rs"]
mod support;
use support::TempFixture;

fn guibiao_forbids_forbidden(package: &str, manifest: &Path) -> GnomonOutcome {
    let constitution = GnomonConstitution::new(package).boundary(
        ModuleBoundary::in_crate(package)
            .module("crate")
            .must_not_import("crate::forbidden")
            .because("conformance: the hidden `use` must not be observed if it is inert"),
    );
    guibiao::check(&constitution, manifest)
}

/// `audit_probe_coverage` reacts on TWO independent axes, so declaring `"conformance-seam"`
/// distinguishes "no real probe found" from "a real probe found" unambiguously: a probe hidden
/// inside inert text leaves the declared seam unprobed (exit 1, `unprobed_seam`); a real probe
/// (declared or not) satisfies it (exit 0) — never the reverse, so the exit code alone pins which
/// case fired without needing a second, differently-configured check.
fn louke_sees_a_real_probe(root: &Path) -> bool {
    let boundary = RuntimeBoundary::at("conformance-seam")
        .only_origins(["o"])
        .because("conformance: a real probe must satisfy this declared seam");
    audit_probe_coverage(&[boundary], &[root.to_path_buf()]).exit_code() == 0
}

fn assert_both_agree(name: &str, body: &str, expect_real: bool) {
    let fixture = TempFixture::new(name, body);
    let guibiao_outcome = guibiao_forbids_forbidden(name, fixture.manifest());
    let louke_sees_real = louke_sees_a_real_probe(fixture.lib());

    assert_eq!(
        guibiao_outcome.exit_code() == 1,
        expect_real,
        "圭表 disagreed on whether the hidden `use` is real: {guibiao_outcome:?}"
    );
    assert_eq!(
        louke_sees_real, expect_real,
        "漏刻 disagreed on whether the hidden probe is real (coverage-satisfied = {louke_sees_real})"
    );
}

#[test]
fn both_dimensions_skip_a_nested_block_comment() {
    assert_both_agree(
        "nested-comment",
        "/* outer /* inner */ still a comment \
         use crate::forbidden::Thing; \
         fn f() { assert_boundary!(\"conformance-seam\", o); } */\n\
         pub fn f() {}\n",
        false,
    );
}

#[test]
fn both_dimensions_see_through_a_nested_block_comment_to_real_content_after_it() {
    // The comment closes correctly (nesting tracked), so real content AFTER it is still observed
    // — the inverse check: a scanner that mis-tracks nesting depth could either swallow this real
    // content (a false negative) or leak the commented-out content as if real (a false positive).
    assert_both_agree(
        "nested-comment-real",
        "/* outer /* inner */ still a comment */\n\
         pub mod real { pub fn f() { use crate::forbidden::Thing; assert_boundary!(\"conformance-seam\", o); } }\n",
        true,
    );
}

#[test]
fn both_dimensions_skip_a_macro_body_regardless_of_delimiter() {
    // `[]`/`()` bodies, not only `{}` — 漏刻's own history names this as a fix 圭表 needed to
    // independently take too (PROJECT.md: "漏刻's nested-comment/non-`()`-delimiter fixes that 圭表
    // never took" is the exact divergence class this ledger exists to catch).
    assert_both_agree(
        "macro-bracket-body",
        "some_macro![ use crate::forbidden::Thing; fn f() { assert_boundary!(\"conformance-seam\", o); } ];\n\
         pub fn f() {}\n",
        false,
    );
    assert_both_agree(
        "macro-paren-body",
        "some_macro!( use crate::forbidden::Thing; fn f() { assert_boundary!(\"conformance-seam\", o); } );\n\
         pub fn f() {}\n",
        false,
    );
}

#[test]
fn both_dimensions_treat_a_raw_string_as_inert_text() {
    // A raw string's contents look exactly like a real `use`/probe, but must never be mistaken
    // for one by either scanner — the raw-string-vs-real-code boundary is exactly the kind of
    // lexical primitive both dimensions independently re-derive.
    assert_both_agree(
        "raw-string",
        "pub fn f() -> &'static str {\n    r#\"use crate::forbidden::Thing; assert_boundary!(\"conformance-seam\", o);\"#\n}\n",
        false,
    );
}
