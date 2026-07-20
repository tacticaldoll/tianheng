use super::*;
use crate::{Outcome, RuntimeBoundary, Severity};
use std::path::{Path, PathBuf};

// A declared boundary for a seam, severity-parameterized (declarations are now objects,
// not source-scanned — so the audit tests construct them directly).
fn boundary(seam: &'static str, severity: Severity) -> RuntimeBoundary {
    let draft = RuntimeBoundary::at(seam).only_origins(["o"]);
    let draft = if severity == Severity::Warn {
        draft.warn()
    } else {
        draft
    };
    draft.because("r")
}

fn literal_seams(probes: &[Probe]) -> Vec<String> {
    probes
        .iter()
        .filter_map(|p| match p {
            Probe::Literal(s) => Some(s.clone()),
            Probe::Unauditable { .. } => None,
        })
        .collect()
}

// Write a one-file crate dir under a unique base and return it.
fn write_dir(base: &Path, name: &str, body: &str) -> PathBuf {
    let dir = base.join(name);
    std::fs::create_dir_all(&dir).unwrap();
    std::fs::write(dir.join("a.rs"), body).unwrap();
    dir
}

fn write_source(base: &Path, relative: &str, body: &str) -> PathBuf {
    let path = base.join(relative);
    std::fs::create_dir_all(path.parent().unwrap()).unwrap();
    std::fs::write(&path, body).unwrap();
    path
}

#[test]
fn root_aware_audit_follows_modules_and_excludes_orphans_and_inline_shadows() {
    let base = std::env::temp_dir().join(format!("louke-root-walk-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&base);

    let root = write_source(
        &base,
        "custom_root.rs",
        "mod adapter; mod nested { mod child; } mod inline { fn live() {} }",
    );
    write_source(
        &base,
        "adapter.rs",
        "fn live() { assert_boundary!(\"adapter\", o); } mod deep;",
    );
    write_source(
        &base,
        "adapter/deep.rs",
        "fn live() { assert_boundary!(\"deep\", o); }",
    );
    write_source(
        &base,
        "nested/child.rs",
        "fn live() { assert_boundary!(\"nested\", o); }",
    );
    write_source(
        &base,
        "orphan.rs",
        "fn dead() { assert_boundary!(\"orphan\", o); }",
    );
    write_source(
        &base,
        "inline.rs",
        "fn dead() { assert_boundary!(\"inline-shadow\", o); }",
    );

    let outcome = audit_probe_coverage(
        &[
            boundary("adapter", Severity::Enforce),
            boundary("deep", Severity::Enforce),
            boundary("nested", Severity::Enforce),
            boundary("orphan", Severity::Enforce),
            boundary("inline-shadow", Severity::Enforce),
        ],
        &[root],
    );
    let violations = match outcome {
        Outcome::Violations(report) => report.violations,
        other => panic!("orphan and inline shadow must stay unprobed: {other:?}"),
    };
    let mut targets: Vec<_> = violations.iter().map(|v| v.target.as_str()).collect();
    targets.sort_unstable();
    assert_eq!(targets, ["inline-shadow", "orphan"]);

    let _ = std::fs::remove_dir_all(base);
}

#[test]
fn root_aware_audit_fails_loud_on_an_unresolvable_reachable_module() {
    let base = std::env::temp_dir().join(format!("louke-root-missing-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&base);
    let root = write_source(&base, "lib.rs", "mod missing;");
    let outcome = audit_probe_coverage(&[], &[root]);
    assert!(
        matches!(outcome, Outcome::ConstitutionError(ref message) if message.contains("missing")),
        "a declared source module cannot disappear silently: {outcome:?}"
    );
    let _ = std::fs::remove_dir_all(base);
}

#[test]
fn root_aware_audit_fails_loud_on_non_utf8_reachable_source() {
    let base = std::env::temp_dir().join(format!("louke-root-unreadable-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&base);
    let root = write_source(&base, "lib.rs", "mod broken;");
    std::fs::write(base.join("broken.rs"), [0xff, 0xfe]).unwrap();
    let outcome = audit_probe_coverage(&[], &[root]);
    assert!(
        matches!(outcome, Outcome::ConstitutionError(ref message) if message.contains("broken.rs")),
        "a selected source that cannot be decoded must fail loud: {outcome:?}"
    );
    let _ = std::fs::remove_dir_all(base);
}

#[test]
fn root_aware_audit_does_not_follow_a_mod_token_inside_a_macro_body() {
    let base = std::env::temp_dir().join(format!("louke-root-macro-mod-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&base);
    let root = write_source(
        &base,
        "lib.rs",
        "macro_rules! generated { () => { mod phantom; } } fn live() {}",
    );
    let outcome = audit_probe_coverage(&[], &[root]);
    assert_eq!(outcome, Outcome::Clean, "macro tokens are not live modules");
    let _ = std::fs::remove_dir_all(base);
}

#[test]
fn directory_input_retains_the_recursive_compatibility_corpus() {
    let base = std::env::temp_dir().join(format!("louke-dir-compat-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&base);
    let dir = write_dir(
        &base,
        "legacy",
        "fn f() { assert_boundary!(\"legacy\", o); }",
    );
    assert_eq!(
        audit_probe_coverage(&[boundary("legacy", Severity::Enforce)], &[dir]).exit_code(),
        0
    );
    let _ = std::fs::remove_dir_all(base);
}

#[test]
fn scan_collects_only_literal_probes_skipping_comments_and_strings() {
    let src = r#"
            fn setup() { louke::install([RuntimeBoundary::at("domain-entry").only_origins(["app::domain"]).because("x")], []); }
            fn used() { assert_boundary!("domain-entry", obj); }
            // a comment mentioning assert_boundary!("ignored-comment") must not count
            let s = "assert_boundary!(\"ignored-string\", x)";
        "#;
    let mut probes = Vec::new();
    scan_source(src, "test.rs", &mut probes);
    let literals: Vec<&str> = probes
        .iter()
        .filter_map(|p| match p {
            Probe::Literal(s) => Some(s.as_str()),
            Probe::Unauditable { .. } => None,
        })
        .collect();
    // The `RuntimeBoundary::at` declaration is no longer scanned (declarations are objects).
    assert_eq!(
        literals,
        vec!["domain-entry"],
        "{probes:?} should hold only the real probe"
    );
    assert!(
        !literals.contains(&"ignored-comment") && !literals.contains(&"ignored-string"),
        "markers in comments/strings must not count: {literals:?}"
    );
    assert!(
        !probes
            .iter()
            .any(|p| matches!(p, Probe::Unauditable { .. })),
        "no un-auditable probe in this fixture"
    );
}

#[test]
fn scan_flags_a_non_literal_seam_probe_as_unauditable() {
    let src = r#"
            const SEAM: &str = "domain-entry";
            fn used() { assert_boundary!(SEAM, obj); }
            fn ok() { assert_boundary!("explicit", obj); }
        "#;
    let mut probes = Vec::new();
    scan_source(src, "test.rs", &mut probes);
    assert!(
        probes
            .iter()
            .any(|p| matches!(p, Probe::Unauditable { .. })),
        "a const-seam probe must be flagged un-auditable: {probes:?}"
    );
    assert!(
        probes
            .iter()
            .any(|p| matches!(p, Probe::Literal(s) if s == "explicit")),
        "the literal probe is still captured: {probes:?}"
    );
}

#[test]
fn a_comment_between_bang_and_paren_does_not_drop_the_probe() {
    // The dangerous false negative: a probe must still be seen with a comment between `!`
    // and `(`, else an undeclared/typo seam there would escape Direction B and panic in prod.
    for src in [
        "fn f() { assert_boundary! /* x */ (\"c-seam\", o); }",
        "fn f() { assert_boundary! // c\n (\"c-seam\", o); }",
    ] {
        let mut probes = Vec::new();
        scan_source(src, "test.rs", &mut probes);
        assert!(
            probes
                .iter()
                .any(|p| matches!(p, Probe::Literal(s) if s == "c-seam")),
            "a comment between ! and ( must not drop the probe: {probes:?}"
        );
    }
}

#[test]
fn an_identifier_ending_in_the_marker_is_not_a_probe() {
    // `my_assert_boundary!` / `xassert_boundary!` are unrelated user macros — a left word
    // boundary keeps them from being mis-counted (a false probe that could mask coverage).
    let src = "fn f() { my_assert_boundary!(\"prefixed\", o); xassert_boundary!(\"fp\", o); }";
    let mut probes = Vec::new();
    scan_source(src, "test.rs", &mut probes);
    assert!(
        probes.is_empty(),
        "an embedded marker must not count as a probe: {probes:?}"
    );
}

#[test]
fn a_non_ascii_prefixed_lookalike_is_not_a_probe() {
    // `Ωassert_boundary` is ONE identifier (Ω is XID_Start), so its `!` is a foreign macro, not our
    // probe. The non-ASCII byte before `assert_boundary` must count as an identifier byte or the
    // left word boundary would be wrongly satisfied and the foreign macro miscounted as coverage.
    let src = "fn f() { Ωassert_boundary!(\"seam\", o); }";
    let mut probes = Vec::new();
    scan_source(src, "test.rs", &mut probes);
    assert!(
        literal_seams(&probes).is_empty(),
        "a non-ASCII-prefixed lookalike macro must not be captured: {probes:?}"
    );
}

#[test]
fn a_probe_with_a_gap_before_the_bang_is_captured() {
    // `ident ! (…)` with whitespace or a comment between the name and `!` is valid Rust
    // (`println !("x")` compiles), so a probe written that way must still count — a contiguous-only
    // marker match silently dropped it (a false negative: seam falsely reported unprobed, and a
    // typo'd seam never caught as probed-but-undeclared).
    for src in [
        "fn f() { assert_boundary !(\"live\", o); }",
        "fn f() { assert_boundary/* gap */!(\"live\", o); }",
        "fn f() { assert_boundary\n        !(\"live\", o); }",
    ] {
        let mut probes = Vec::new();
        scan_source(src, "test.rs", &mut probes);
        assert_eq!(
            literal_seams(&probes),
            ["live"],
            "a probe with a gap before `!` must be captured: {src:?} -> {probes:?}"
        );
    }
}

#[test]
fn a_probe_inside_a_spaced_foreign_macro_body_is_not_counted() {
    // Symmetric with the gap-tolerant probe marker: a foreign macro invoked with whitespace before
    // its `!` (`wrap !( … )`, valid Rust) is recognized as a macro, so a probe lexically inside its
    // body is skipped (macro-generated / dead), not miscounted as coverage. The real probe after
    // the body still counts.
    let src = "fn f() { wrap !( assert_boundary!(\"dead\", o) ); assert_boundary!(\"live\", o); }";
    let mut probes = Vec::new();
    scan_source(src, "test.rs", &mut probes);
    assert_eq!(
        literal_seams(&probes),
        ["live"],
        "the probe inside the spaced foreign macro body must not count: {probes:?}"
    );
}

#[test]
fn a_keyword_before_the_bang_is_not_a_macro_body() {
    // `return !( … )` is unary negation in expression position, NOT a macro invocation (a macro
    // name is never a keyword). Its parenthesized operand must be scanned, not skipped as a macro
    // body, so a probe inside it still counts. Guards the whitespace-lookback against misreading a
    // keyword `!` as a foreign macro.
    let src = "fn f() -> bool { return !( { assert_boundary!(\"live\", o); true } ); }";
    let mut probes = Vec::new();
    scan_source(src, "test.rs", &mut probes);
    assert_eq!(
        literal_seams(&probes),
        ["live"],
        "a probe inside a keyword-negated group must still count: {probes:?}"
    );
}

#[test]
fn a_raw_identifier_macro_whose_word_is_a_keyword_is_skipped() {
    // `r#async!(…)` is a macro invocation whose raw-identifier name escapes the keyword `async` —
    // it IS a macro, so a probe inside its body is macro-generated and must not count. The keyword
    // guard must recognize the `r#` prefix and not mistake the escaped run for the bare keyword.
    let src =
        "fn f() { r#async!( assert_boundary!(\"dead\", o) ); assert_boundary!(\"live\", o); }";
    let mut probes = Vec::new();
    scan_source(src, "test.rs", &mut probes);
    assert_eq!(
        literal_seams(&probes),
        ["live"],
        "a probe inside a raw-identifier-named macro body must not count: {probes:?}"
    );
}

#[test]
fn a_raw_string_seam_is_an_auditable_literal() {
    // A raw-string seam is a traceable literal — parse its value, do not mis-flag it.
    let src =
        "fn f() { assert_boundary!(r#\"raw-seam\"#, o); assert_boundary!(r\"plain-raw\", o); }";
    let mut probes = Vec::new();
    scan_source(src, "test.rs", &mut probes);
    assert!(
        probes
            .iter()
            .any(|p| matches!(p, Probe::Literal(s) if s == "raw-seam")),
        "r#\"…\"# seam value must be captured: {probes:?}"
    );
    assert!(
        probes
            .iter()
            .any(|p| matches!(p, Probe::Literal(s) if s == "plain-raw")),
        "r\"…\" seam value must be captured: {probes:?}"
    );
    assert!(
        !probes
            .iter()
            .any(|p| matches!(p, Probe::Unauditable { .. })),
        "a raw-string seam is auditable, not un-auditable: {probes:?}"
    );
}

#[test]
fn capture_probe_decodes_string_escapes_to_the_compiler_value() {
    // The seam must be compared by the value the COMPILER produces, not the raw source bytes —
    // the declared set is `RuntimeBoundary::seam()`, already decoded. Each `assert_boundary!`
    // in this fixture carries a plain-string seam written with an escape; the decoded value is
    // what the runtime seam actually is. (The fixture is a raw string, so the `\n` etc. below
    // reach the scanner as backslash-escapes, exactly as a programmer would write them.)
    let src = r##"fn f() {
            assert_boundary!("a\n", o);
            assert_boundary!("t\ta", o);
            assert_boundary!("cr\r", o);
            assert_boundary!("back\\slash", o);
            assert_boundary!("nul\0", o);
            assert_boundary!("q\"q", o);
            assert_boundary!("hex\x41", o);
            assert_boundary!("u\u{2764}", o);
            assert_boundary!("us\u{2_764}", o);
        }"##;
    let mut probes = Vec::new();
    scan_source(src, "test.rs", &mut probes);
    let seams = literal_seams(&probes);
    // Right-hand sides are ordinary Rust literals: the compiler decodes them, so this asserts
    // decoded == decoded (the scanner must match the compiler, not the raw bytes).
    for expected in [
        "a\n",
        "t\ta",
        "cr\r",
        "back\\slash",
        "nul\0",
        "q\"q",
        "hex\x41",
        "u\u{2764}",
        "us\u{2_764}",
    ] {
        assert!(
            seams.iter().any(|s| s == expected),
            "decoded seam {expected:?} missing from {seams:?}"
        );
    }
    assert!(
        !probes
            .iter()
            .any(|p| matches!(p, Probe::Unauditable { .. })),
        "well-formed escaped seams are auditable, not un-auditable: {probes:?}"
    );
}

#[test]
fn an_undecodable_escape_or_line_continuation_is_unauditable() {
    // The decoder's `None` contract: anything it cannot reproduce EXACTLY reacts loud, never a
    // silently mismatched literal. A backslash-newline line continuation is the reachable
    // real-source case (it compiles); the malformed forms are the defensive backstop.
    let line_continuation = "fn f() { assert_boundary!(\"a\\\nb\", o); }";
    let malformed = [
        "fn f() { assert_boundary!(\"bad\\q\", o); }", // unknown escape
        "fn f() { assert_boundary!(\"trunc\\x\", o); }", // truncated \x
        "fn f() { assert_boundary!(\"hi\\xFF\", o); }", // \x value > 0x7F
        "fn f() { assert_boundary!(\"emptyu\\u{}\", o); }", // \u with no digits
        "fn f() { assert_boundary!(\"leadus\\u{_41}\", o); }", // leading `_` — rustc rejects
    ];
    for src in std::iter::once(line_continuation).chain(malformed) {
        let mut probes = Vec::new();
        scan_source(src, "test.rs", &mut probes);
        assert!(
            probes
                .iter()
                .any(|p| matches!(p, Probe::Unauditable { .. })),
            "an un-decodable escape must be un-auditable, never a literal: {src:?} -> {probes:?}"
        );
        assert!(
            !probes.iter().any(|p| matches!(p, Probe::Literal(_))),
            "an un-decodable escape must not yield a (mismatched) literal: {src:?} -> {probes:?}"
        );
    }
}

#[test]
fn audit_matches_an_escaped_seam_against_its_escaped_probe() {
    // Regression guard for the both-direction false pair: a declared seam containing a newline
    // (the compiler-decoded `RuntimeBoundary::at("a\n")`) is covered by a source probe written
    // `assert_boundary!("a\n", o)` — the scanner now decodes the probe to the same value.
    let base = std::env::temp_dir().join(format!("louke-audit-esc-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&base);
    let dir = write_dir(&base, "esc", "fn f() { assert_boundary!(\"a\\n\", o); }");
    let outcome = audit_probe_coverage(&[boundary("a\n", Severity::Enforce)], &[dir]);
    let _ = std::fs::remove_dir_all(&base);
    assert_eq!(
        outcome.exit_code(),
        0,
        "an escaped seam whose probe decodes to the same value is covered: {outcome:?}"
    );
}

#[test]
fn audit_reacts_when_a_declaration_and_probe_decode_differently() {
    // The false-negative closure: declaring `at("a\\n")` (three chars: a, backslash, n) while
    // the only probe is `"a\n"` (two chars: a, newline) is a real runtime mismatch — the probe
    // would panic on an undeclared seam. Comparing raw bytes counted it covered; decoding
    // catches it. Expect BOTH directions: declared-unprobed and probed-undeclared.
    let base = std::env::temp_dir().join(format!("louke-audit-esc2-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&base);
    let dir = write_dir(&base, "esc2", "fn f() { assert_boundary!(\"a\\n\", o); }");
    let outcome = audit_probe_coverage(&[boundary("a\\n", Severity::Enforce)], &[dir]);
    let _ = std::fs::remove_dir_all(&base);
    match outcome {
        Outcome::Violations(report) => {
            assert!(
                report
                    .violations
                    .iter()
                    .any(|v| v.finding.contains("has no assert_boundary! probe")),
                "the 3-char declared seam must be reported unprobed: {:?}",
                report.violations
            );
            assert!(
                report
                    .violations
                    .iter()
                    .any(|v| v.finding.contains("undeclared seam")),
                "the 2-char decoded probe must be reported undeclared: {:?}",
                report.violations
            );
        }
        other => panic!("expected a decode-mismatch reaction, got {other:?}"),
    }
}

#[test]
fn an_escape_free_or_raw_seam_is_unchanged_by_the_decoder() {
    // No baseline/behavior churn for the common case: an escape-free plain seam and a raw-string
    // seam decode to themselves, so coverage is exactly as before.
    let mut probes = Vec::new();
    scan_source(
        r##"fn f() { assert_boundary!("domain-entry", o); assert_boundary!(r"raw\n", o); }"##,
        "t.rs",
        &mut probes,
    );
    let mut seams = literal_seams(&probes);
    seams.sort_unstable();
    assert_eq!(
        seams,
        ["domain-entry", "raw\\n"],
        "escape-free and raw seams are verbatim (raw keeps its backslash-n): {probes:?}"
    );
}

#[test]
fn a_raw_or_byte_string_does_not_desync_the_scanner() {
    // A raw string with an inner `"` must not swallow a later real probe, and a probe
    // marker inside a byte string must not be counted.
    let src = r####"
            let x = r#"he said "hi""#;
            fn f() { assert_boundary!("real-seam", o); }
            let y = b"assert_boundary!(\"bytestr\", z)";
        "####;
    let mut probes = Vec::new();
    scan_source(src, "test.rs", &mut probes);
    let literals: Vec<&str> = probes
        .iter()
        .filter_map(|p| match p {
            Probe::Literal(s) => Some(s.as_str()),
            Probe::Unauditable { .. } => None,
        })
        .collect();
    assert!(
        literals.contains(&"real-seam"),
        "a raw string must not desync and swallow a later probe: {literals:?}"
    );
    assert!(
        !literals.contains(&"bytestr"),
        "a marker inside a byte string must not count: {literals:?}"
    );
}

#[test]
fn a_probe_inside_a_macro_body_is_not_counted() {
    // A probe inside a `macro_rules!` body or another macro invocation body is
    // macro-generated / dead: it must NOT count as coverage (the audit's forbidden FN). A real
    // probe AFTER the macro body must still be captured.
    for src in [
        "macro_rules! m { () => { assert_boundary!(\"dead\", o); }; }\n\
             fn f() { assert_boundary!(\"live\", o); }",
        // whitespace between `macro_rules` and `!` (valid, if unformatted, Rust): the name-skip
        // must still recognise the keyword or the body is walked and its probe wrongly counted.
        "macro_rules ! spaced { () => { assert_boundary!(\"dead\", o); }; }\n\
             fn f() { assert_boundary!(\"live\", o); }",
        "fn f() { some_macro! { let _ = 1; assert_boundary!(\"dead\", o) }; assert_boundary!(\"live\", o); }",
        // nested + mixed delimiters, with a `}` inside a string and a `}` inside a char
        "fn f() { wrap!( [ { let s = \"}}}\"; let c = '}'; assert_boundary!(\"dead\", o) } ] ); assert_boundary!(\"live\", o); }",
    ] {
        let mut probes = Vec::new();
        scan_source(src, "t.rs", &mut probes);
        let seams = literal_seams(&probes);
        assert_eq!(
            seams,
            ["live"],
            "only the real probe outside the macro body counts: {src:?} -> {probes:?}"
        );
    }
}

#[test]
fn a_probe_inside_a_raw_ident_named_macro_body_is_not_counted() {
    // A `macro_rules!` name may be a raw identifier (`r#async`, `r#try`): the name-skip run must
    // span the `#` of `r#…` (an ident byte is alphanumeric/`_` only, so `#` needs its own arm) or
    // it would bail at the `#`, miss the body delimiter, scan the body, and falsely count the
    // `assert_boundary!` inside it — a reintroduced false negative. Guards that arm against removal.
    let src = "macro_rules! r#async { () => { assert_boundary!(\"dead\", o); }; }\n\
               fn f() { assert_boundary!(\"live\", o); }";
    let mut probes = Vec::new();
    scan_source(src, "t.rs", &mut probes);
    assert_eq!(
        literal_seams(&probes),
        ["live"],
        "a probe inside a raw-identifier-named macro body must not count: {probes:?}"
    );
}

#[test]
fn operators_and_keyword_glued_bang_are_not_macro_bodies() {
    // `!=`, unary `!`, and a keyword glued to `!` (valid Rust: `if!cond {…}`) must NOT be
    // treated as macro invocations — else a probe inside the real block would be swallowed (a
    // reintroduced false negative). The probe inside each block must still be captured.
    for src in [
        "fn f() { if!cond { assert_boundary!(\"live\", o); } }",
        "fn f() { while!x { assert_boundary!(\"live\", o); } }",
        "fn f() { let _ = a != b; if !flag { assert_boundary!(\"live\", o); } }",
    ] {
        let mut probes = Vec::new();
        scan_source(src, "t.rs", &mut probes);
        assert_eq!(
            literal_seams(&probes),
            ["live"],
            "a keyword-glued `!` / operator must not skip the real block: {src:?} -> {probes:?}"
        );
    }
}

#[test]
fn a_declared_seam_probed_only_inside_a_macro_body_reacts_unprobed() {
    // End-to-end: a declared seam whose ONLY probe is inside a macro body is never enforced at
    // runtime, so the audit must report it declared-but-unprobed (exit 1 at enforce), not
    // silently covered.
    let base = std::env::temp_dir().join(format!("louke-audit-macro-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&base);
    let dir = write_dir(
        &base,
        "m",
        "macro_rules! g { () => { assert_boundary!(\"seam\", o); }; }",
    );
    let outcome = audit_probe_coverage(&[boundary("seam", Severity::Enforce)], &[dir]);
    let _ = std::fs::remove_dir_all(&base);
    assert_eq!(
        outcome.exit_code(),
        1,
        "a seam probed only inside a macro body must react as unprobed: {outcome:?}"
    );
}

#[test]
fn a_probe_inside_a_nested_block_comment_is_not_counted() {
    // Rust block comments nest, so this entire span is ONE comment and the probe is
    // commented out. A non-depth-aware scan would leave comment mode at the inner `*/`
    // and wrongly count "s" as probed — the forbidden false negative (the seam would be
    // reported covered while never enforced).
    let mut probes = Vec::new();
    scan_source(
        r#"/* outer /* inner */ assert_boundary!("s", o); */"#,
        "t.rs",
        &mut probes,
    );
    assert!(
        probes.is_empty(),
        "a probe inside a nested block comment must not count: {probes:?}"
    );
}

#[test]
fn a_real_probe_after_a_nested_block_comment_is_still_counted() {
    // The depth fix must not over-eat: `/* a /* b */ c */` is a complete (nested) comment,
    // and the probe that follows is real code and MUST count.
    let mut probes = Vec::new();
    scan_source(
        r#"/* a /* b */ c */ assert_boundary!("real", o);"#,
        "t.rs",
        &mut probes,
    );
    assert_eq!(
        literal_seams(&probes),
        ["real"],
        "a real probe after a closed nested comment must count: {probes:?}"
    );
}

#[test]
fn a_brace_or_bracket_delimited_probe_is_captured() {
    // Rust macros accept `{ }` and `[ ]` identically to `( )`; such a probe is real and
    // must be audited, not silently dropped (a drop would let a typo'd seam escape the
    // undeclared-seam check — a false negative).
    let mut probes = Vec::new();
    scan_source(
        "fn f() { assert_boundary!{\"brace\", o}; assert_boundary![\"bracket\", o]; }",
        "t.rs",
        &mut probes,
    );
    let mut seams = literal_seams(&probes);
    seams.sort_unstable();
    assert_eq!(
        seams,
        ["brace", "bracket"],
        "brace/bracket-delimited probes must be captured: {probes:?}"
    );
}

#[test]
fn audit_reacts_to_a_duplicate_declared_seam() {
    // A seam declared twice is a constitution error: prod `install` fails loud on it, and the
    // CI face must react too (enforce) so it surfaces before a running binary. Probe the seam
    // so the ONLY finding is the duplicate, not a declared-unprobed gap.
    let base = std::env::temp_dir().join(format!("louke-audit-dup-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&base);
    let dir = write_dir(&base, "m", "fn f() { assert_boundary!(\"twice\", o); }");
    let outcome = audit_probe_coverage(
        &[
            boundary("twice", Severity::Enforce),
            boundary("twice", Severity::Enforce),
        ],
        &[dir],
    );
    let _ = std::fs::remove_dir_all(&base);
    match outcome {
        Outcome::Violations(report) => assert!(
            report
                .violations
                .iter()
                .any(|v| v.target == "twice" && v.finding.contains("declared more than once")),
            "a duplicate declared seam must react: {:?}",
            report.violations
        ),
        other => panic!("expected a duplicate-seam violation, got {other:?}"),
    }
}

#[test]
fn a_nested_comment_between_bang_and_paren_does_not_drop_the_probe() {
    // skip_trivia shares the depth-aware skip with scan_source, so a NESTED comment between
    // `!` and `(` must be skipped whole; otherwise it desyncs and misses the real probe.
    let mut probes = Vec::new();
    scan_source(
        r#"fn f() { assert_boundary! /* a /* b */ c */ ("nested-trivia", o); }"#,
        "t.rs",
        &mut probes,
    );
    assert_eq!(
        literal_seams(&probes),
        ["nested-trivia"],
        "a probe after a nested comment between ! and ( must be captured: {probes:?}"
    );
}

#[test]
fn audit_probe_coverage_reacts_both_directions() {
    let base = std::env::temp_dir().join(format!("louke-audit-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&base);

    // declared + probed match → clean (exit 0)
    let clean = write_dir(&base, "clean", "fn f() { assert_boundary!(\"s\", o); }");
    assert_eq!(
        audit_probe_coverage(&[boundary("s", Severity::Enforce)], &[clean]).exit_code(),
        0
    );

    // declared but unprobed (enforce) → react (exit 1)
    let unprobed = write_dir(&base, "unprobed", "fn f() {}");
    assert_eq!(
        audit_probe_coverage(&[boundary("orphan", Severity::Enforce)], &[unprobed]).exit_code(),
        1
    );

    // probed but undeclared (a typo) → react at CI, not a prod panic (exit 1)
    let typo = write_dir(&base, "typo", "fn f() { assert_boundary!(\"ghost\", o); }");
    assert_eq!(audit_probe_coverage(&[], &[typo]).exit_code(), 1);

    let _ = std::fs::remove_dir_all(&base);
}

#[test]
fn a_warn_severity_unprobed_seam_is_advisory_not_a_failure() {
    let base = std::env::temp_dir().join(format!("louke-audit-warn-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&base);
    let dir = write_dir(&base, "warn", "fn f() {}");
    // A warn boundary with no probe reacts (a Violation) but does not by itself fail CI.
    let outcome = audit_probe_coverage(&[boundary("soft", Severity::Warn)], &[dir]);
    assert_eq!(outcome.exit_code(), 0, "warn-only is advisory: {outcome:?}");
    assert!(
        matches!(outcome, Outcome::Violations(_)),
        "it still reports the advisory: {outcome:?}"
    );
    let _ = std::fs::remove_dir_all(&base);
}

#[test]
fn coverage_spans_the_workspace_corpus() {
    let base = std::env::temp_dir().join(format!("louke-audit-corpus-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&base);
    // Declared once; its only probe lives in a *different* member dir.
    let decl_only = write_dir(&base, "crate_a", "fn f() {}");
    let probe_only = write_dir(
        &base,
        "crate_b",
        "fn g() { assert_boundary!(\"shared\", o); }",
    );
    let outcome = audit_probe_coverage(
        &[boundary("shared", Severity::Enforce)],
        &[decl_only, probe_only],
    );
    assert_eq!(
        outcome.exit_code(),
        0,
        "the corpus is the union of all dirs: {outcome:?}"
    );
    let _ = std::fs::remove_dir_all(&base);
}

#[test]
fn an_unauditable_probe_reacts() {
    let base = std::env::temp_dir().join(format!("louke-audit-unaud-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&base);
    let dir = write_dir(
        &base,
        "unaud",
        "const SEAM: &str = \"s\"; fn f() { assert_boundary!(SEAM, o); }",
    );
    // Even though a boundary "s" is declared, the probe is non-literal → un-auditable → react.
    let outcome = audit_probe_coverage(&[boundary("s", Severity::Enforce)], &[dir]);
    assert_eq!(
        outcome.exit_code(),
        1,
        "an un-auditable probe must react: {outcome:?}"
    );
    // The un-auditable violation carries the offending source file (the probe scan
    // captured it): a genuine observation, not a dishonest null.
    let violations = match &outcome {
        Outcome::Violations(report) => &report.violations,
        other => panic!("expected violations, got {other:?}"),
    };
    let file = violations
        .iter()
        .find_map(|v| v.file.as_deref())
        .expect("the un-auditable-probe violation carries its source file");
    assert!(
        file.ends_with("a.rs"),
        "file names the probe's source: {file}"
    );
    let _ = std::fs::remove_dir_all(&base);
}

#[test]
fn a_seam_level_runtime_violation_has_no_file() {
    // A declared-but-never-probed seam names a seam, not a source location, so its `file`
    // is a faithful `None` — distinct from the un-auditable case, which does have a file.
    let base = std::env::temp_dir().join(format!("louke-audit-seamnull-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&base);
    let dir = write_dir(&base, "unprobed", "fn f() {}");
    let outcome = audit_probe_coverage(&[boundary("orphan", Severity::Enforce)], &[dir]);
    let violations = match &outcome {
        Outcome::Violations(report) => &report.violations,
        other => panic!("expected violations, got {other:?}"),
    };
    assert!(
        violations.iter().all(|v| v.file.is_none()),
        "a seam-level runtime violation has no source file: {violations:?}"
    );
    let _ = std::fs::remove_dir_all(&base);
}
