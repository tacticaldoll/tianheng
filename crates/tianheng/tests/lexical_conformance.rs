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
use louke::{Outcome as LoukeOutcome, RuntimeBoundary, audit_probe_coverage};

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

/// What 漏刻 read the fixture's probe as: a declared seam's probe it saw, or none, so the seam is unprobed.
#[derive(Debug, PartialEq, Eq)]
enum LoukeReading {
    Real,
    Inert,
}

/// `audit_probe_coverage` over a declared `"conformance-seam"`: clean where a real probe satisfies it, and exactly the
/// declared seam's unprobed violation where none does. Any other outcome — a scan or constitution error above all, or
/// a violation of another rule — is neither reading, so it panics carrying the whole outcome rather than counting as
/// "no probe".
fn louke_reading(root: &Path) -> LoukeReading {
    let boundary = RuntimeBoundary::at("conformance-seam")
        .only_origins(["o"])
        .because("conformance: a real probe must satisfy this declared seam");
    // The anchor just needs to contain the scanned root the way a real caller's workspace root contains its members.
    let anchor = root.parent().unwrap_or(root);
    match audit_probe_coverage(&[boundary], &[root.to_path_buf()], anchor) {
        LoukeOutcome::Clean(_) => LoukeReading::Real,
        LoukeOutcome::Violations(report)
            if report.violations.len() == 1
                && report.violations[0].rule == "every declared runtime seam must be probed" =>
        {
            LoukeReading::Inert
        }
        other => panic!("漏刻 neither saw the probe nor reported the seam unprobed: {other:?}"),
    }
}

fn assert_both_agree(name: &str, body: &str, expect_real: bool) {
    // 圭表 refuses a forbidden module the crate does not declare, so every fixture declares it,
    // empty and ahead of the case's own lexical shape.
    let body = format!("pub mod forbidden {{}}\n{body}");
    let fixture = TempFixture::new(name, &body);
    let guibiao_outcome = guibiao_forbids_forbidden(name, fixture.manifest());
    let louke_sees_real = louke_reading(fixture.lib()) == LoukeReading::Real;

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

#[test]
fn both_dimensions_separate_tokens_at_every_pattern_white_space_character() {
    // A vertical tab and a left-to-right mark are whitespace to rustc (`Pattern_White_Space`), so a
    // `use` and a probe they stand inside are real code; an ASCII-only reading glues the mark into a name
    // or reads the tab as punctuation.
    assert_both_agree(
        "pattern-white-space",
        "use crate::forbidden::\u{200E}Thing;\npub fn f(o: u8) {\u{b}assert_boundary!(\"conformance-seam\", o); }\n",
        true,
    );
}

/// The one measured divergence this ledger declares rather than closes. C string literals arrive in edition 2021: in
/// 2018 `cr#"x"` is `cr`, `#` and a string, so the `use` and the probe before a later `"#` are real code, and 圭表,
/// which reads each target in its edition, sees the `use`. 漏刻 reads a probe from source roots alone, with no edition
/// to read them in, and takes the `r#"` after `c` for a raw string in every edition, so the probe is not seen — the
/// declared bound
/// `runtime-origin-assertion/a-raw-string-after-an-identifier-character-is-read-as-one-in-every-edition-a-stated-bound`.
/// rustc 1.96.0 builds the shape in edition 2018 and refuses it in 2021.
#[test]
fn louke_reads_a_raw_string_after_an_identifier_character_in_every_edition() {
    let body = "pub mod forbidden {}\nmacro_rules! m { ($($t:tt)*) => {}; }\nm!(cr#\"x\");\nuse crate::forbidden::Thing;\n\
                pub fn f(o: u8) { assert_boundary!(\"conformance-seam\", o); }\nm!(\"#\");\n";
    let fixture = TempFixture::new("c-string-2018", body);
    fixture.write(
        "Cargo.toml",
        "[package]\nname = \"c-string-2018\"\nversion = \"0.0.0\"\nedition = \"2018\"\n",
    );
    let guibiao_outcome = guibiao_forbids_forbidden("c-string-2018", fixture.manifest());
    assert_eq!(
        guibiao_outcome.exit_code(),
        1,
        "圭表 reads the `use` as real code: {guibiao_outcome:?}"
    );
    assert_eq!(
        louke_reading(fixture.lib()),
        LoukeReading::Inert,
        "漏刻 now sees the probe: the declared bound no longer holds and should be retired"
    );
}
