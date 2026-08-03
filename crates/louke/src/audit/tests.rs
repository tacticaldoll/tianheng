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

#[test]
fn every_audit_rule_family_has_exact_semantic_identity() {
    let cases = [
        (
            AuditRule::UniqueSeamDeclaration,
            "tianheng.rule/louke/unique-seam-declaration",
        ),
        (
            AuditRule::DeclaredSeamProbed,
            "tianheng.rule/louke/declared-seam-probed",
        ),
        (
            AuditRule::ProbeDeclaredSeam,
            "tianheng.rule/louke/probe-declared-seam",
        ),
        (
            AuditRule::LiteralProbeSeam,
            "tianheng.rule/louke/literal-probe-seam",
        ),
    ];
    for (rule, expected_type) in cases {
        let key = rule.key();
        assert_eq!(key.rule_type(), expected_type);
        assert_eq!(key.fields().count(), 0);
    }
}

/// A unique, self-cleaning temp base directory for an `assert_boundary!`-probe source fixture:
/// write source files under it, then hand its root (or a derived path) to `audit_probe_coverage`
/// — replaces the hand-rolled `temp_dir().join(format!(...))` + manual `remove_dir_all` at both
/// ends that every test in this file otherwise repeated.
struct TempBase(PathBuf);

impl TempBase {
    fn new(label: &str) -> Self {
        let base = std::env::temp_dir().join(format!("louke-{label}-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&base);
        Self(base)
    }

    fn path(&self) -> &Path {
        &self.0
    }

    /// Write a one-file crate dir (`<name>/a.rs`) under this base and return the dir path.
    fn dir(&self, name: &str, body: &str) -> PathBuf {
        let dir = self.0.join(name);
        std::fs::create_dir_all(&dir).unwrap();
        std::fs::write(dir.join("a.rs"), body).unwrap();
        dir
    }

    /// Write a source file at `relative` (relative to this base), creating parent dirs as
    /// needed. Returns the file's absolute path.
    fn source(&self, relative: &str, body: &str) -> PathBuf {
        let path = self.0.join(relative);
        std::fs::create_dir_all(path.parent().unwrap()).unwrap();
        std::fs::write(&path, body).unwrap();
        path
    }

    /// Symlink `target` at `link_rel` (relative to this base).
    #[cfg(unix)]
    fn symlink(&self, target: impl AsRef<Path>, link_rel: &str) {
        std::os::unix::fs::symlink(target, self.0.join(link_rel)).expect("create symlink");
    }

    /// [`audit_probe_coverage`] with THIS base as the label anchor — the shape every real caller
    /// has, a checkout root sitting above every source root it scans (`xingbiao::workspace_root`
    /// for the `tianheng` shell). Since the anchor is now a caller's argument rather than something
    /// the audit derives, this is where the tests state theirs once instead of at each call.
    ///
    /// A test that is *about* the anchor calls [`audit_probe_coverage`] directly with the anchor it
    /// means — `two_member_sets_over_one_checkout_label_a_shared_file_identically` varies the input
    /// set against a fixed anchor, and the absolute-`#[path]` bound tests need a file outside it.
    fn audit(&self, declared: &[RuntimeBoundary], roots: &[PathBuf]) -> Outcome {
        audit_probe_coverage(declared, roots, self.path())
    }

    /// [`TempBase::audit`] with a custom probe-marker list.
    fn audit_with_markers(
        &self,
        declared: &[RuntimeBoundary],
        roots: &[PathBuf],
        markers: &[&str],
    ) -> Outcome {
        audit_probe_coverage_with_markers(declared, roots, self.path(), markers)
    }
}

impl Drop for TempBase {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.0);
    }
}

#[test]
fn root_aware_audit_follows_modules_and_excludes_orphans_and_inline_shadows() {
    let tb = TempBase::new("root-walk");

    let root = tb.source(
        "custom_root.rs",
        "mod adapter; mod nested { mod child; } mod inline { fn live() {} }",
    );
    tb.source(
        "adapter.rs",
        "fn live() { assert_boundary!(\"adapter\", o); } mod deep;",
    );
    tb.source(
        "adapter/deep.rs",
        "fn live() { assert_boundary!(\"deep\", o); }",
    );
    tb.source(
        "nested/child.rs",
        "fn live() { assert_boundary!(\"nested\", o); }",
    );
    tb.source(
        "orphan.rs",
        "fn dead() { assert_boundary!(\"orphan\", o); }",
    );
    tb.source(
        "inline.rs",
        "fn dead() { assert_boundary!(\"inline-shadow\", o); }",
    );

    let outcome = tb.audit(
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
    let mut targets: Vec<_> = violations.iter().map(|v| v.target()).collect();
    targets.sort_unstable();
    assert_eq!(targets, ["inline-shadow", "orphan"]);
}

#[test]
fn a_cfg_attr_gated_missing_module_still_fails_loud_not_tolerated() {
    // `#[cfg_attr(unix, allow(dead_code))] mod gated;` with no `gated.rs` is a REAL rustc compile
    // error (E0583) on every platform: `cfg_attr`'s predicate only gates whether `allow(dead_code)`
    // is applied, never whether the `mod` item itself exists — unlike a bare `#[cfg(...)]`, which
    // genuinely removes the item when its predicate is false. Verified against a real `rustc`
    // invocation of this exact fixture shape (E0583, unconditionally, regardless of the
    // `cfg_attr`'s predicate value). The scanner must not conflate the two: tolerating this would
    // silently skip auditing a module that always compiles and is always genuinely broken.
    let tb = TempBase::new("cfg-attr-not-cfg-tolerant");
    let root = tb.source(
        "lib.rs",
        "#[cfg_attr(unix, allow(dead_code))]\nmod gated;\nfn f() { assert_boundary!(\"a\", o); }",
    );
    let outcome = tb.audit(&[boundary("a", Severity::Enforce)], &[root]);
    assert!(
        matches!(outcome, Outcome::ConstitutionError(ref message) if message.contains("gated")),
        "a cfg_attr-decorated (not cfg-gated) missing module must still fail loud: {outcome:?}"
    );
}

#[test]
fn root_aware_audit_fails_loud_on_an_unresolvable_reachable_module() {
    let tb = TempBase::new("root-missing");
    let root = tb.source("lib.rs", "mod missing;");
    let outcome = tb.audit(&[], &[root]);
    assert!(
        matches!(outcome, Outcome::ConstitutionError(ref message) if message.contains("missing")),
        "a declared source module cannot disappear silently: {outcome:?}"
    );
}

#[test]
fn deeply_nested_blocks_are_a_scan_error_not_a_stack_overflow() {
    // A pathologically nested block chain must not overflow the native stack — a real,
    // malformed or adversarial source nested this deep must fail loud (a scan error) rather
    // than crash the process. Measured crash threshold for this exact recursion under a 2MB
    // test-thread stack: safe at depth 1100, a real SIGABRT stack overflow at depth 1105+; the
    // depth cap this pins is 300, comfortably clear of both that measured line and this test's
    // depth.
    let tb = TempBase::new("scope-depth-cap");
    let depth = 2000;
    let nested = format!("fn f() {{{}{}}}", "{".repeat(depth), "}".repeat(depth));
    let root = tb.source("lib.rs", &nested);
    let outcome = tb.audit(&[], &[root]);
    assert!(
        matches!(outcome, Outcome::ConstitutionError(ref message) if message.contains("depth bound")),
        "scope nesting past the depth cap must be a scan error, not a stack overflow: {outcome:?}"
    );
}

#[test]
fn moderately_nested_blocks_still_observe_a_real_violation() {
    // Control: nesting comfortably under the depth cap is unaffected — a real, deeply (but not
    // pathologically) nested unresolvable module reference must still be observed and fail loud
    // on its own terms, proving the walk actually reaches this depth rather than being narrowed
    // by the fix.
    let tb = TempBase::new("scope-depth-under-cap");
    let depth = 100;
    let nested = format!(
        "fn f() {{{}mod missing;{}}}",
        "{".repeat(depth),
        "}".repeat(depth)
    );
    let root = tb.source("lib.rs", &nested);
    let outcome = tb.audit(&[], &[root]);
    assert!(
        matches!(outcome, Outcome::ConstitutionError(ref message) if message.contains("missing")),
        "a moderately nested missing module must still be observed: {outcome:?}"
    );
}

#[test]
fn a_cfg_gated_module_with_no_file_is_skipped_not_errored() {
    let tb = TempBase::new("cfg-absent");
    // `#[cfg(feature = "never")] mod optional;` with no `optional.rs` is legal Rust when the
    // feature is off; the walk must skip it rather than fail the audit (exit 2), matching 渾儀's
    // cfg-tolerance. The non-cfg `mod present;` resolves normally and carries the only probe, so a
    // clean outcome proves the cfg-gated module was tolerated, not that the walk simply stopped.
    // Its counterpart — a non-cfg `mod missing;` with no file — still fails loud (see
    // `root_aware_audit_fails_loud_on_an_unresolvable_reachable_module`).
    let root = tb.source(
        "lib.rs",
        "#[cfg(feature = \"never\")]\nmod optional;\nmod present;",
    );
    tb.source(
        "present.rs",
        "fn live() { assert_boundary!(\"present\", o); }",
    );
    let outcome = tb.audit(&[boundary("present", Severity::Enforce)], &[root]);
    assert_eq!(
        outcome.exit_code(),
        0,
        "a cfg-gated module with no file is skipped, not errored: {outcome:?}"
    );
}

#[test]
fn root_aware_audit_fails_loud_on_non_utf8_reachable_source() {
    let tb = TempBase::new("root-unreadable");
    let root = tb.source("lib.rs", "mod broken;");
    std::fs::write(tb.path().join("broken.rs"), [0xff, 0xfe]).unwrap();
    let outcome = tb.audit(&[], &[root]);
    assert!(
        matches!(outcome, Outcome::ConstitutionError(ref message) if message.contains("broken.rs")),
        "a selected source that cannot be decoded must fail loud: {outcome:?}"
    );
}

#[test]
fn root_aware_audit_does_not_follow_a_mod_token_inside_a_macro_body() {
    let tb = TempBase::new("root-macro-mod");
    let root = tb.source(
        "lib.rs",
        "macro_rules! generated { () => { mod phantom; } } fn live() {}",
    );
    let outcome = tb.audit(&[], &[root]);
    assert_eq!(outcome, Outcome::Clean, "macro tokens are not live modules");
}

#[test]
fn a_path_substring_in_a_comment_or_attr_does_not_drop_a_reachable_module() {
    let tb = TempBase::new("path-substr");
    // The preamble contains the substring "path" twice — a line comment and an unrelated cfg
    // feature name — but neither is a `#[path]` attribute, so the module must still be followed
    // and its probe seen. A raw-substring detector would misclassify it as relocated and drop
    // the module (a silent coverage false negative).
    let root = tb.source(
        "lib.rs",
        "// fast path to the adapter\n#[cfg(feature = \"fastpath\")]\nmod adapter;",
    );
    tb.source(
        "adapter.rs",
        "fn live() { assert_boundary!(\"adapter\", o); }",
    );
    let outcome = tb.audit(&[boundary("adapter", Severity::Enforce)], &[root]);
    assert_eq!(
        outcome.exit_code(),
        0,
        "a `path` substring in a comment/attr must not drop a reachable module: {outcome:?}"
    );
}

#[test]
fn an_unconditional_path_attribute_is_followed_to_its_target() {
    let tb = TempBase::new("path-attr");
    // A genuine unconditional `#[path = "..."]` relocation (with a `]` inside the literal to
    // exercise bracket matching) is now FOLLOWED to its author-chosen file, and its probe counts;
    // the conventional name (`relocated.rs`) is never consulted. Previously the module was skipped
    // — a coverage false negative for a seam probed only in a relocated file.
    let root = tb.source(
        "lib.rs",
        "#[path = \"generated/a]b.rs\"]\nmod relocated;\nfn live() {}",
    );
    tb.source(
        "generated/a]b.rs",
        "fn inner() { assert_boundary!(\"relocated-seam\", o); }",
    );
    let outcome = tb.audit(&[boundary("relocated-seam", Severity::Enforce)], &[root]);
    assert_eq!(
        outcome.exit_code(),
        0,
        "an unconditional #[path] module is followed and its probe counts: {outcome:?}"
    );
}

#[test]
fn a_semicolon_inside_an_earlier_doc_attributes_string_does_not_hide_a_later_path_attribute() {
    // Round-9 finding: mod_preamble_attrs found where a mod declaration's own attribute preamble
    // begins by scanning BACKWARD from the mod keyword for the nearest raw byte equal to `;`/`{`/`}`
    // -- the only traversal in this file that was not literal/comment-aware (every other walk here
    // routes through skip_literal_or_comment specifically to avoid this class of bug). An EARLIER
    // attribute's own string value containing a bare `;` (ordinary prose, e.g. `#[doc = "Handles A;
    // falls back to B."]`) stopped the old backward scan mid-literal, desyncing the forward
    // attribute walk that followed: it read the string's own closing quote as the OPENER of a bogus
    // new string, swallowing the real `#[path = "..."]` attribute's own `#` inside it. The scanner
    // then never saw the #[path] attribute at all, so it fell back to the conventional (nonexistent)
    // location and failed loud on a module that is genuinely `#[path]`-relocated and compiles fine.
    let tb = TempBase::new("doc-semicolon");
    let root = tb.source("lib.rs", "mod worker;\nfn live() {}");
    tb.source(
        "worker.rs",
        "#[doc = \"Handles A; falls back to B.\"]\n#[path = \"relocated.rs\"]\nmod inner;\n",
    );
    tb.source(
        "relocated.rs",
        "fn f() { assert_boundary!(\"relocated-seam\", o); }",
    );
    let outcome = tb.audit(&[boundary("relocated-seam", Severity::Enforce)], &[root]);
    assert_eq!(
        outcome.exit_code(),
        0,
        "the #[path] attribute must still be found and followed despite the earlier #[doc] \
         attribute's own semicolon: {outcome:?}"
    );
}

#[test]
fn a_brace_delimited_attribute_argument_does_not_hide_an_earlier_path_attribute() {
    // Round-10 finding: round 9's fix made mod_preamble_attrs' backward-then-forward preamble
    // scan literal/comment-aware, but not attribute-group-aware. Its forward pass found `start`
    // by remembering the position just past the LAST raw `;`/`{`/`}` byte seen, with no tracking
    // of nesting -- so a brace-delimited attribute ARGUMENT (`#[foo({ 1 })]`, a valid token tree,
    // not a string literal) sitting between an earlier, real `#[path = "..."]` and the `mod`
    // keyword had its own internal `{`/`}` mistaken for item-boundary terminators, resetting
    // `start` to a point AFTER the real `#[path]` attribute -- reproducing the round-9 bug's exact
    // failure mode (a #[path]-relocated module falsely reported as unresolvable) through a
    // different vector. Fixed by skipping a whole `#[...]` group as one atomic unit (via the same
    // attr_group_end already used by the second, attribute-matching pass) when scanning for the
    // preamble's own start, so its internal bytes are never examined as boundary candidates.
    let tb = TempBase::new("brace-attr");
    let root = tb.source("lib.rs", "mod worker;\nfn live() {}");
    tb.source(
        "worker.rs",
        "#[path = \"relocated.rs\"]\n#[foo({ 1 })]\nmod inner;\n",
    );
    tb.source(
        "relocated.rs",
        "fn f() { assert_boundary!(\"relocated-seam\", o); }",
    );
    let outcome = tb.audit(&[boundary("relocated-seam", Severity::Enforce)], &[root]);
    assert_eq!(
        outcome.exit_code(),
        0,
        "the #[path] attribute must still be found and followed despite the later brace-delimited \
         #[foo({{ 1 }})] attribute argument: {outcome:?}"
    );
}

#[test]
fn path_in_a_non_mod_rs_file_resolves_from_the_containing_files_own_dir() {
    let tb = TempBase::new("path-nonmodrs");
    // rustc ground truth: a non-inline `#[path="bar.rs"]` inside foo.rs (reached via `mod foo;`)
    // resolves to <root>/bar.rs — foo.rs's OWN directory — not <root>/foo/bar.rs. bar.rs probes an
    // UNDECLARED seam, so the rustc-correct verdict is a probed-but-undeclared enforce violation
    // (exit 1). A decoy at foo/bar.rs (the buggy child_base location) has no probe: reading it would
    // return Clean/exit 0 — the forbidden false negative. This pins the corrected containing-file dir.
    let root = tb.source("lib.rs", "mod foo;");
    tb.source("foo.rs", "#[path = \"bar.rs\"]\nmod bar;");
    tb.source(
        "bar.rs",
        "fn inner() { assert_boundary!(\"undeclared-seam\", o); }",
    );
    tb.source("foo/bar.rs", "fn decoy() {}");
    let outcome = tb.audit(&[], &[root]);
    assert_eq!(
        outcome.exit_code(),
        1,
        "a #[path] inside a non-mod.rs file resolves from its own dir (bar.rs, undeclared seam -> \
         exit 1), never the foo/bar.rs decoy (which would be a silent exit-0 FN): {outcome:?}"
    );
}

#[test]
fn path_nested_in_an_inline_block_resolves_from_the_accumulated_dir() {
    let tb = TempBase::new("path-inline");
    // rustc ground truth (rustc 1.96.0): `mod inline { #[path="other.rs"] mod inner; }` at the crate
    // root resolves inner to <root>/inline/other.rs — the base accumulates the inline-module name.
    // inline/other.rs probes an UNDECLARED seam (rustc-correct verdict: exit 1). A decoy at
    // <root>/other.rs (the enclosing file_dir base, no probe) is what threading file_dir UNCHANGED
    // through the inline recursion would have read — returning Clean/exit 0, the forbidden false
    // negative. Pins the accumulated inline base.
    let root = tb.source("lib.rs", "mod inline { #[path = \"other.rs\"] mod inner; }");
    tb.source(
        "inline/other.rs",
        "fn inner() { assert_boundary!(\"undeclared-seam\", o); }",
    );
    tb.source("other.rs", "fn decoy() {}");
    let outcome = tb.audit(&[], &[root]);
    assert_eq!(
        outcome.exit_code(),
        1,
        "a #[path] nested in an inline block resolves from <root>/inline (undeclared seam -> exit 1), \
         never the <root>/other.rs decoy (a silent exit-0 FN): {outcome:?}"
    );
}

#[test]
fn a_hex_escape_in_a_path_literal_decodes_to_the_same_file_syn_reads() {
    let tb = TempBase::new("path-escape");
    // rustc ground truth (rustc 1.96.0): `#[path = "f\x6fo.rs"] mod name;` compiles foo.rs (\x6f =
    // 'o') and ignores a conventional name.rs. 渾儀 decodes the escape via syn's `s.value()`; 漏刻 must
    // decode it identically (twin-drift parity) rather than bail to conventional resolution. foo.rs
    // probes an UNDECLARED seam (rustc-correct verdict exit 1); the name.rs decoy has no probe, so a
    // bail-to-conventional would read it and return Clean/exit 0 — the compound false negative.
    let root = tb.source("lib.rs", "#[path = \"f\\x6fo.rs\"]\nmod name;");
    tb.source(
        "foo.rs",
        "fn inner() { assert_boundary!(\"undeclared-seam\", o); }",
    );
    tb.source("name.rs", "fn decoy() {}");
    let outcome = tb.audit(&[], &[root]);
    assert_eq!(
        outcome.exit_code(),
        1,
        "a \\x escape in a #[path] literal decodes to foo.rs (undeclared seam -> exit 1), matching \
         syn, never the name.rs decoy (a silent exit-0 FN): {outcome:?}"
    );
}

#[test]
fn an_absent_unconditional_path_target_is_a_constitution_error() {
    let tb = TempBase::new("path-absent");
    // An unconditional `#[path]` whose target file is absent is a genuine broken reference (rustc
    // errors too): fail loud (exit 2), never a silent skip that could hide a seam.
    let root = tb.source(
        "lib.rs",
        "#[path = \"missing.rs\"]\nmod relocated;\nfn live() {}",
    );
    let outcome = tb.audit(&[], &[root]);
    assert_eq!(
        outcome.exit_code(),
        2,
        "an absent unconditional #[path] target fails loud (exit 2): {outcome:?}"
    );
}

#[test]
fn a_cfg_attr_path_relocation_with_no_resolution_anywhere_fails_loud() {
    // `#[cfg_attr(unix, path = "...")]`'s relocation target is cfg-conditional, so it is NOT
    // followed cfg-blind (a stated bound — the scan instead falls back to the conventional file,
    // documented on `audit_probe_coverage`). But `cfg_attr` never conditionally REMOVES the `mod`
    // item itself (unlike a bare `#[cfg]`): verified against a real `rustc` build on this (unix)
    // machine — `#[cfg_attr(unix, path = "unix_seam.rs")] mod plat;` with `unix_seam.rs` absent is
    // a genuine compile error (rustc does NOT fall back to a conventional `plat.rs` even when one
    // is present, since `cfg(unix)` is true here and the relocation is authoritative) — so when
    // NEITHER the relocation target NOR the conventional file exists, this must fail loud, never
    // silently tolerate. (A prior version of this test asserted `Outcome::Clean` here; corrected
    // after empirically verifying against `rustc` that this is always a real broken reference.)
    let tb = TempBase::new("cfgattr-path");
    let root = tb.source(
        "lib.rs",
        "#[cfg_attr(unix, path = \"unix_seam.rs\")]\nmod plat;\nfn live() {}",
    );
    let outcome = tb.audit(&[], &[root]);
    assert!(
        matches!(outcome, Outcome::ConstitutionError(ref message) if message.contains("plat")),
        "cfg_attr-wrapped #[path] with no resolution anywhere must fail loud: {outcome:?}"
    );
}

#[test]
fn directory_input_retains_the_recursive_compatibility_corpus() {
    let tb = TempBase::new("dir-compat");
    let dir = tb.dir("legacy", "fn f() { assert_boundary!(\"legacy\", o); }");
    assert_eq!(
        tb.audit(&[boundary("legacy", Severity::Enforce)], &[dir])
            .exit_code(),
        0
    );
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
fn probe_first_arg_with_generic_comma_preserves_full_span() {
    for (src, expected_expr) in [
        (
            "fn f() { assert_boundary!(SEAM::<A, B>, o); }",
            "SEAM::<A, B>",
        ),
        ("fn f() { assert_boundary!(a < b, o); }", "a < b"),
        ("fn f() { assert_boundary!(a < b, c > d); }", "a < b"),
        (
            "fn f() { assert_boundary!(SEAM::<fn() -> A, B>, o); }",
            "SEAM::<fn() -> A, B>",
        ),
        (
            "fn f() { assert_boundary!(SEAM::<{ A > B }, C>, o); }",
            "SEAM::<{ A > B }, C>",
        ),
        (
            "fn f() { assert_boundary!(<Foo<A, B> as Trait<C, D>>::SEAM, o); }",
            "<Foo<A, B> as Trait<C, D>>::SEAM",
        ),
        (
            "fn f() { assert_boundary!(<Foo<A, B> as /* gap */ Trait<C, D>>::SEAM, o); }",
            "<Foo<A, B> as /* gap */ Trait<C, D>>::SEAM",
        ),
        (
            "fn f() { assert_boundary!(<Foo<A, B> as\n    Trait<C, D>>::SEAM, o); }",
            "<Foo<A, B> as\n    Trait<C, D>>::SEAM",
        ),
        (
            "fn f() { assert_boundary!(value as /* gap */ <Foo<A, B> as Trait<C, D>>::Assoc, o); }",
            "value as /* gap */ <Foo<A, B> as Trait<C, D>>::Assoc",
        ),
        (
            "fn f() { assert_boundary!(value as\n    <Foo<A, B> as Trait<C, D>>::Assoc, o); }",
            "value as\n    <Foo<A, B> as Trait<C, D>>::Assoc",
        ),
        (
            "fn f() { assert_boundary!(SEAM::<[Foo<A, B>; 1], C>, o); }",
            "SEAM::<[Foo<A, B>; 1], C>",
        ),
        (
            "fn f() { assert_boundary!(seam:: // comment\n <A, B>, obj); }",
            "seam:: // comment\n <A, B>",
        ),
        (
            "fn f() { assert_boundary!(seam:: /* /* nested */ */ <A, C>, obj); }",
            "seam:: /* /* nested */ */ <A, C>",
        ),
        (
            "fn f() { assert_boundary!(SEAM::<Outer /* comment */ <A, B>, C>, obj); }",
            "SEAM::<Outer /* comment */ <A, B>, C>",
        ),
        (
            "fn f() { assert_boundary!(seam::<Ω <A, B>, C>, obj); }",
            "seam::<Ω <A, B>, C>",
        ),
        (
            "fn f() { assert_boundary!(&<Foo<u8, u16> as Trait<u32, u64>>::SEAM, ()); }",
            "&<Foo<u8, u16> as Trait<u32, u64>>::SEAM",
        ),
        (
            "fn f() { assert_boundary!(*<Foo<A, B> as Trait<C, D>>::SEAM, ()); }",
            "*<Foo<A, B> as Trait<C, D>>::SEAM",
        ),
        (
            "fn f() { assert_boundary!(!<Foo<A, B> as Trait<C, D>>::SEAM, ()); }",
            "!<Foo<A, B> as Trait<C, D>>::SEAM",
        ),
        (
            "fn f() { assert_boundary!(& /* comment */ <Foo<u8, u16> as Trait<u32, u64>>::SEAM, ()); }",
            "& /* comment */ <Foo<u8, u16> as Trait<u32, u64>>::SEAM",
        ),
        (
            "fn f() { assert_boundary!(& /* outer /* nested */ comment */ <Foo<u8, u16> as Trait<u32, u64>>::SEAM, ()); }",
            "& /* outer /* nested */ comment */ <Foo<u8, u16> as Trait<u32, u64>>::SEAM",
        ),
        (
            "fn f() { assert_boundary!(& // comment\n <Foo<u8, u32> as Trait<u64, u128>>::SEAM, ()); }",
            "& // comment\n <Foo<u8, u32> as Trait<u64, u128>>::SEAM",
        ),
        (
            "fn f() { assert_boundary!(&\"a\" < &Foo::<u8, u16>::X, ()); }",
            "&\"a\" < &Foo::<u8, u16>::X",
        ),
        (
            "fn f() { assert_boundary!(&'a' < &'b', ()); }",
            "&'a' < &'b'",
        ),
        (
            "fn f() { assert_boundary!(&b\"a\" < &b\"b\", ()); }",
            "&b\"a\" < &b\"b\"",
        ),
        (
            "fn f() { assert_boundary!(&r#\"a\"# < &r#\"b\"#, ()); }",
            "&r#\"a\"# < &r#\"b\"#",
        ),
    ] {
        let mut probes = Vec::new();
        scan_source(src, "test.rs", &mut probes);
        assert_eq!(probes.len(), 1, "failed for {src}");
        match &probes[0] {
            crate::audit::scan::Probe::Unauditable { expr, .. } => {
                assert_eq!(
                    expr, expected_expr,
                    "first macro arg must match for {src}: got {expr:?}"
                );
            }
            other => panic!("expected Unauditable probe, got {other:?}"),
        }
    }
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
fn an_undecodable_escape_is_unauditable() {
    // The decoder's `None` contract: anything it cannot reproduce EXACTLY reacts loud, never a
    // silently mismatched literal.
    let malformed = [
        "fn f() { assert_boundary!(\"bad\\q\", o); }", // unknown escape
        "fn f() { assert_boundary!(\"trunc\\x\", o); }", // truncated \x
        "fn f() { assert_boundary!(\"hi\\xFF\", o); }", // \x value > 0x7F
        "fn f() { assert_boundary!(\"emptyu\\u{}\", o); }", // \u with no digits
        "fn f() { assert_boundary!(\"leadus\\u{_41}\", o); }", // leading `_` — rustc rejects
    ];
    for src in malformed {
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
fn a_backslash_newline_line_continuation_now_decodes_like_rustc() {
    // Verified against a real `rustc` build: `"a\` + newline + `b"` decodes to `"ab"` (the
    // backslash, the newline, and the continued line's leading whitespace are stripped). This
    // decoder now matches `syn`'s `LitStr::value()` fidelity instead of treating the shape as
    // un-auditable — the fix a v0.2.0..v0.2.1 cross-dimension sweep found missing here (and in
    // 圭表's independent copy).
    let src = "fn f() { assert_boundary!(\"a\\\nb\", o); }";
    let mut probes = Vec::new();
    scan_source(src, "test.rs", &mut probes);
    assert!(
        matches!(&probes[..], [Probe::Literal(seam)] if seam == "ab"),
        "a line continuation must decode to the joined value, matching rustc: {probes:?}"
    );
}

#[test]
fn audit_matches_an_escaped_seam_against_its_escaped_probe() {
    // Regression guard for the both-direction false pair: a declared seam containing a newline
    // (the compiler-decoded `RuntimeBoundary::at("a\n")`) is covered by a source probe written
    // `assert_boundary!("a\n", o)` — the scanner now decodes the probe to the same value.
    let tb = TempBase::new("audit-esc");
    let dir = tb.dir("esc", "fn f() { assert_boundary!(\"a\\n\", o); }");
    let outcome = tb.audit(&[boundary("a\n", Severity::Enforce)], &[dir]);
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
    let tb = TempBase::new("audit-esc2");
    let dir = tb.dir("esc2", "fn f() { assert_boundary!(\"a\\n\", o); }");
    let outcome = tb.audit(&[boundary("a\\n", Severity::Enforce)], &[dir]);
    match outcome {
        Outcome::Violations(report) => {
            assert!(
                report
                    .violations
                    .iter()
                    .any(|v| v.finding.contains("has no configured probe marker")),
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
    let tb = TempBase::new("audit-macro");
    let dir = tb.dir(
        "m",
        "macro_rules! g { () => { assert_boundary!(\"seam\", o); }; }",
    );
    let outcome = tb.audit(&[boundary("seam", Severity::Enforce)], &[dir]);
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
    let tb = TempBase::new("audit-dup");
    let dir = tb.dir("m", "fn f() { assert_boundary!(\"twice\", o); }");
    let outcome = tb.audit(
        &[
            boundary("twice", Severity::Enforce),
            boundary("twice", Severity::Enforce),
        ],
        &[dir],
    );
    match outcome {
        Outcome::Violations(report) => assert!(
            report
                .violations
                .iter()
                .any(|v| v.target() == "twice" && v.finding.contains("declared more than once")),
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
    let tb = TempBase::new("audit");

    // declared + probed match → clean (exit 0)
    let clean = tb.dir("clean", "fn f() { assert_boundary!(\"s\", o); }");
    assert_eq!(
        tb.audit(&[boundary("s", Severity::Enforce)], &[clean])
            .exit_code(),
        0
    );

    // declared but unprobed (enforce) → react (exit 1)
    let unprobed = tb.dir("unprobed", "fn f() {}");
    assert_eq!(
        tb.audit(&[boundary("orphan", Severity::Enforce)], &[unprobed])
            .exit_code(),
        1
    );

    // probed but undeclared (a typo) → react at CI, not a prod panic (exit 1)
    let typo = tb.dir("typo", "fn f() { assert_boundary!(\"ghost\", o); }");
    assert_eq!(tb.audit(&[], &[typo]).exit_code(), 1);
}

#[test]
fn audit_production_violation_separates_target_rule_and_fact_roles() {
    let tb = TempBase::new("audit-structured-identity");
    let dir = tb.dir("unprobed", "fn f() {}");
    let outcome = tb.audit(&[boundary("checkout", Severity::Enforce)], &[dir]);
    let report = match outcome {
        Outcome::Violations(report) => report,
        other => panic!("expected an unprobed-seam violation, got {other:?}"),
    };
    let violation = report.violations.first().expect("one unprobed seam");
    let id = violation.id();
    assert_eq!(violation.target(), "checkout");
    let rule = id.rule_key();
    assert_eq!(rule.rule_type(), "tianheng.rule/louke/declared-seam-probed");
    assert_eq!(rule.fields().count(), 0);
    let fact = id.fact();
    assert_eq!(fact.fact_type(), "tianheng.fact/louke/runtime-seam-audit");
    assert_eq!(fact.shape(), "unprobed-declaration");
    assert_eq!(
        fact.fields().collect::<Vec<_>>(),
        vec![("seam", "checkout")]
    );
}

#[test]
fn a_warn_severity_unprobed_seam_is_advisory_not_a_failure() {
    let tb = TempBase::new("audit-warn");
    let dir = tb.dir("warn", "fn f() {}");
    // A warn boundary with no probe reacts (a Violation) but does not by itself fail CI.
    let outcome = tb.audit(&[boundary("soft", Severity::Warn)], &[dir]);
    assert_eq!(outcome.exit_code(), 0, "warn-only is advisory: {outcome:?}");
    assert!(
        matches!(outcome, Outcome::Violations(_)),
        "it still reports the advisory: {outcome:?}"
    );
}

#[test]
fn coverage_spans_the_workspace_corpus() {
    let tb = TempBase::new("audit-corpus");
    // Declared once; its only probe lives in a *different* member dir.
    let decl_only = tb.dir("crate_a", "fn f() {}");
    let probe_only = tb.dir("crate_b", "fn g() { assert_boundary!(\"shared\", o); }");
    let outcome = tb.audit(
        &[boundary("shared", Severity::Enforce)],
        &[decl_only, probe_only],
    );
    assert_eq!(
        outcome.exit_code(),
        0,
        "the corpus is the union of all dirs: {outcome:?}"
    );
}

#[test]
fn an_unauditable_probe_reacts() {
    let tb = TempBase::new("audit-unaud");
    let dir = tb.dir(
        "unaud",
        "const SEAM: &str = \"s\"; fn f() { assert_boundary!(SEAM, o); }",
    );
    // Even though a boundary "s" is declared, the probe is non-literal → un-auditable → react.
    let outcome = tb.audit(&[boundary("s", Severity::Enforce)], &[dir]);
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
}

/// Un-auditable-probe violations from `outcome` (identified by carrying a `file` — the one
/// runtime violation kind that does; a seam-level runtime violation names a seam, never a file).
fn unauditable_violations(outcome: &Outcome) -> Vec<&crate::Violation> {
    match outcome {
        Outcome::Violations(report) => report
            .violations
            .iter()
            .filter(|v| v.file.is_some())
            .collect(),
        other => panic!("expected violations, got {other:?}"),
    }
}

/// Assert that `violations` carries exactly `expected` DISTINCT `fact()` identities — the
/// count-and-collect shape repeated across the identity-distinctness regression tests below (two
/// textually/lexically distinct probes must never collapse to fewer findings than expected).
/// `message` explains WHY these must stay distinct; the violations themselves are appended for a
/// failing assertion's own debugging.
fn assert_distinct_fact_count(violations: &[&crate::Violation], expected: usize, message: &str) {
    let facts: std::collections::BTreeSet<_> =
        violations.iter().map(|v| v.fact().clone()).collect();
    assert_eq!(facts.len(), expected, "{message}: {violations:?}");
}

#[test]
fn two_distinct_expressions_in_the_same_function_react_separately() {
    let tb = TempBase::new("audit-unaud-two-exprs");
    let dir = tb.dir(
        "two",
        "fn compute_seam() -> &'static str { \"s\" } \
         fn f() { assert_boundary!(SEAM_A, o); assert_boundary!(compute_seam(), o); }",
    );
    let outcome = tb.audit(&[boundary("s", Severity::Enforce)], &[dir]);
    let violations = unauditable_violations(&outcome);
    assert_eq!(
        violations.len(),
        2,
        "two textually distinct non-literal expressions must react separately: {violations:?}"
    );
    assert_distinct_fact_count(&violations, 2, "their identities must be distinct");
}

#[test]
fn same_expression_in_two_different_free_functions_reacts_separately() {
    let tb = TempBase::new("audit-unaud-two-fns");
    let dir = tb.dir(
        "two",
        "fn a() { assert_boundary!(SEAM_A, o); } fn b() { assert_boundary!(SEAM_A, o); }",
    );
    let outcome = tb.audit(&[boundary("s", Severity::Enforce)], &[dir]);
    let violations = unauditable_violations(&outcome);
    assert_eq!(
        violations.len(),
        2,
        "the same expression in two different functions must react separately: {violations:?}"
    );
    assert_distinct_fact_count(
        &violations,
        2,
        "distinguished by enclosing function, not collapsed",
    );
}

#[test]
fn raw_identifier_function_names_keep_probe_owners_distinct() {
    let tb = TempBase::new("audit-unaud-two-raw-ident-fns");
    let dir = tb.dir(
        "two",
        "fn r#type() { assert_boundary!(SEAM_A, o); } \
         fn r#async() { assert_boundary!(SEAM_A, o); }",
    );
    let outcome = tb.audit(&[boundary("s", Severity::Enforce)], &[dir]);
    let violations = unauditable_violations(&outcome);
    assert_eq!(
        violations.len(),
        2,
        "raw identifier names must not collapse to their leading `r`: {violations:?}"
    );
    let owners: std::collections::BTreeSet<_> = violations
        .iter()
        .map(|violation| {
            violation
                .fact()
                .fields()
                .find_map(|(name, value)| (name == "owner").then_some(value.to_string()))
                .expect("unauditable probe identity carries its owner")
        })
        .collect();
    assert_eq!(
        owners,
        ["fn async".to_string(), "fn type".to_string()].into(),
        "raw item names use the scanner's canonical de-prefixed vocabulary"
    );
}

#[test]
fn same_named_nested_functions_in_different_outer_functions_react_separately() {
    let tb = TempBase::new("audit-unaud-two-nested-fns");
    let dir = tb.dir(
        "two",
        "fn outer_a() { fn inner() { assert_boundary!(SEAM_A, o); } } \
         fn outer_b() { fn inner() { assert_boundary!(SEAM_A, o); } }",
    );
    let outcome = tb.audit(&[boundary("s", Severity::Enforce)], &[dir]);
    let violations = unauditable_violations(&outcome);
    assert_eq!(
        violations.len(),
        2,
        "same-named nested functions in distinct outer functions must not collapse: {violations:?}"
    );
    assert_distinct_fact_count(&violations, 2, "lexical owner chains must differ");
}

#[test]
fn same_named_nested_functions_in_equal_closures_react_separately_and_stably() {
    let tb = TempBase::new("audit-unaud-two-closures");
    let dir = tb.dir(
        "two",
        "fn outer() { \
         (|| { fn inner() { assert_boundary!(SEAM_A, o); } })(); \
         (|| { fn inner() { assert_boundary!(SEAM_A, o); } })(); \
         }",
    );
    let before = tb.audit(
        &[boundary("s", Severity::Enforce)],
        std::slice::from_ref(&dir),
    );
    let before_violations = unauditable_violations(&before);
    assert_eq!(
        before_violations.len(),
        2,
        "equal nested functions under distinct closures must not collapse: {before_violations:?}"
    );
    let before_facts: std::collections::BTreeSet<_> = before_violations
        .iter()
        .map(|violation| violation.fact().clone())
        .collect();
    assert_eq!(before_facts.len(), 2, "closure owners must differ");

    std::fs::write(
        dir.join("a.rs"),
        "fn outer() { \
         let unrelated = 1; \
         (|| { fn inner() { assert_boundary!(SEAM_A, o); } })(); \
         (|| { fn inner() { assert_boundary!(SEAM_A, o); } })(); \
         }",
    )
    .unwrap();
    let after = tb.audit(&[boundary("s", Severity::Enforce)], &[dir]);
    let after_facts: std::collections::BTreeSet<_> = unauditable_violations(&after)
        .iter()
        .map(|violation| violation.fact().clone())
        .collect();
    assert_eq!(
        before_facts, after_facts,
        "a differently-shaped unrelated insertion must not re-key closure ownership"
    );
}

#[test]
fn same_named_local_impl_methods_in_different_outer_functions_react_separately() {
    let tb = TempBase::new("audit-unaud-two-local-impls");
    let dir = tb.dir(
        "two",
        "fn outer_a() { struct Local; impl Local { fn probe() { assert_boundary!(SEAM_A, o); } } } \
         fn outer_b() { struct Local; impl Local { fn probe() { assert_boundary!(SEAM_A, o); } } }",
    );
    let outcome = tb.audit(&[boundary("s", Severity::Enforce)], &[dir]);
    let violations = unauditable_violations(&outcome);
    assert_eq!(
        violations.len(),
        2,
        "same-named local impl methods in distinct outer functions must not collapse: {violations:?}"
    );
    assert_distinct_fact_count(
        &violations,
        2,
        "outer lexical owners must qualify local impls",
    );
}

#[test]
fn nested_function_identity_survives_unrelated_item_insertion() {
    let tb = TempBase::new("audit-unaud-nested-stable");
    let dir = tb.dir(
        "same",
        "fn outer() { fn inner() { assert_boundary!(SEAM_A, o); } }",
    );
    let before_fact = unauditable_violations(&tb.audit(
        &[boundary("s", Severity::Enforce)],
        std::slice::from_ref(&dir),
    ))[0]
        .fact()
        .clone();
    std::fs::write(
        dir.join("a.rs"),
        "fn unrelated() {} fn outer() { fn inner() { assert_boundary!(SEAM_A, o); } }",
    )
    .expect("rewrite fixture with unrelated item");
    let after_fact = unauditable_violations(&tb.audit(&[boundary("s", Severity::Enforce)], &[dir]))
        [0]
    .fact()
    .clone();
    assert_eq!(before_fact, after_fact);
}

#[test]
fn same_named_method_in_two_different_impls_reacts_separately() {
    // The regression case an unqualified bare enclosing-fn/impl name would have missed: two
    // owners sharing a method name and an identical expression must not collapse to one finding.
    let tb = TempBase::new("audit-unaud-two-impls");
    let dir = tb.dir(
        "two",
        "struct A; struct B; \
         impl A { fn probe(&self) { assert_boundary!(SEAM_A, o); } } \
         impl B { fn probe(&self) { assert_boundary!(SEAM_A, o); } }",
    );
    let outcome = tb.audit(&[boundary("s", Severity::Enforce)], &[dir]);
    let violations = unauditable_violations(&outcome);
    assert_eq!(
        violations.len(),
        2,
        "same-named method on two different owners must react separately: {violations:?}"
    );
    assert_distinct_fact_count(
        &violations,
        2,
        "distinguished by owner, not collapsed under a bare method name",
    );
}

#[test]
fn same_named_free_fn_in_two_different_inline_mods_reacts_separately() {
    // The regression an adversarial review of this exact change caught empirically: without
    // module-path qualification, two same-named free fns in different inline `mod` blocks of one
    // file collapsed to one finding — the identical false-negative class this change exists to
    // close, just via a dimension (module path) the first cut of `render_owner` didn't track.
    let tb = TempBase::new("audit-unaud-two-inline-mods");
    let dir = tb.dir(
        "two",
        "mod a { pub fn probe() { assert_boundary!(SEAM_A, o); } } \
         mod b { pub fn probe() { assert_boundary!(SEAM_A, o); } }",
    );
    let outcome = tb.audit(&[boundary("s", Severity::Enforce)], &[dir]);
    let violations = unauditable_violations(&outcome);
    assert_eq!(
        violations.len(),
        2,
        "same-named free fn in two different inline mods must react separately: {violations:?}"
    );
    assert_distinct_fact_count(
        &violations,
        2,
        "distinguished by module path, not collapsed under a bare fn name",
    );
}

#[test]
fn same_named_method_in_two_different_trait_impls_reacts_separately() {
    let tb = TempBase::new("audit-unaud-two-trait-impls");
    let dir = tb.dir(
        "two",
        "struct T; trait Foo { fn probe(&self); } trait Bar { fn probe(&self); } \
         impl Foo for T { fn probe(&self) { assert_boundary!(SEAM_A, o); } } \
         impl Bar for T { fn probe(&self) { assert_boundary!(SEAM_A, o); } }",
    );
    let outcome = tb.audit(&[boundary("s", Severity::Enforce)], &[dir]);
    let violations = unauditable_violations(&outcome);
    assert_eq!(
        violations.len(),
        2,
        "same-named method on the same Self type via two different traits must react separately: {violations:?}"
    );
    assert_distinct_fact_count(&violations, 2, "distinguished by trait, not collapsed");
}

#[test]
fn identical_expression_repeated_in_the_same_function_collapses_to_one_violation() {
    let tb = TempBase::new("audit-unaud-dup");
    let dir = tb.dir(
        "dup",
        "fn f() { assert_boundary!(SEAM_A, o); assert_boundary!(SEAM_A, o); }",
    );
    let outcome = tb.audit(&[boundary("s", Severity::Enforce)], &[dir]);
    let violations = unauditable_violations(&outcome);
    assert_eq!(
        violations.len(),
        1,
        "a verbatim-repeated expression in the same scope is a stated bound, not two findings: {violations:?}"
    );
}

#[test]
fn same_expression_in_two_files_reacts_separately() {
    let tb = TempBase::new("audit-unaud-two-files");
    let dir_a = tb.dir("file-a", "fn f() { assert_boundary!(SEAM_A, o); }");
    let dir_b = tb.dir("file-b", "fn f() { assert_boundary!(SEAM_A, o); }");
    let outcome = tb.audit(&[boundary("s", Severity::Enforce)], &[dir_a, dir_b]);
    let violations = unauditable_violations(&outcome);
    assert_eq!(
        violations.len(),
        2,
        "the same expression in two different files must react separately: {violations:?}"
    );
    assert_distinct_fact_count(&violations, 2, "distinguished by file, not collapsed");
}

#[test]
fn a_seam_level_runtime_violation_has_no_file() {
    // A declared-but-never-probed seam names a seam, not a source location, so its `file`
    // is a faithful `None` — distinct from the un-auditable case, which does have a file.
    let tb = TempBase::new("audit-seamnull");
    let dir = tb.dir("unprobed", "fn f() {}");
    let outcome = tb.audit(&[boundary("orphan", Severity::Enforce)], &[dir]);
    let violations = match &outcome {
        Outcome::Violations(report) => &report.violations,
        other => panic!("expected violations, got {other:?}"),
    };
    assert!(
        violations.iter().all(|v| v.file.is_none()),
        "a seam-level runtime violation has no source file: {violations:?}"
    );
}

#[test]
fn finder_repro_nonmodrs_path_base() {
    let tb = TempBase::new("finder-base");
    let root = tb.source("lib.rs", "pub mod app;\n");
    tb.source("app.rs", "#[path = \"relocated.rs\"]\npub mod worker;\n");
    // The REAL compiled target (rustc uses this)
    tb.source(
        "relocated.rs",
        "fn inner() { assert_boundary!(\"relocated-seam\", o); }\n",
    );
    // An orphan at louke's wrong join path (src/app/relocated.rs)
    tb.source("app/relocated.rs", "fn inner() {}\n");
    // Only the REAL target is compiled and has the probe -> must be caught.
    let outcome = tb.audit(&[boundary("relocated-seam", Severity::Enforce)], &[root]);
    assert_eq!(
        outcome.exit_code(),
        0,
        "Coverage should pass as the probe is matched"
    );
}

#[test]
fn finder_repro_fn_orphan() {
    let tb = TempBase::new("finder-fn");
    let root = tb.source("lib.rs", "pub mod app;\n");
    tb.source("app.rs", "#[path = \"relocated.rs\"]\npub mod worker;\n");
    // The REAL compiled target (rustc uses this): has an assert on an UNDECLARED seam -> should react.
    tb.source(
        "relocated.rs",
        "fn inner() { assert_boundary!(\"undeclared-seam\", o); }\n",
    );
    // An orphan at louke's wrong join path (src/app/relocated.rs): louke scans THIS instead. No probe.
    tb.source("app/relocated.rs", "fn inner() {}\n");
    // No declared boundaries: an assert on "undeclared-seam" in the REAL target must be caught.
    let outcome = tb.audit(&[], &[root]);
    assert_eq!(outcome.exit_code(), 1, "Should catch undeclared seam");
}

#[cfg(unix)]
#[test]
fn root_aware_audit_does_not_hang_on_a_symlinked_directory_cycle() {
    // `loop_mod` is a directory symlink back to the base itself, and the root declares `mod
    // loop_mod;` — so each descent generates a NEW, ever-longer LITERAL path
    // (`loop_mod/mod.rs`, `loop_mod/loop_mod/mod.rs`, …) that always canonicalizes to the same
    // real file. Deduping on the literal path alone (as this scanner's `visited` set did before
    // routing through `xingbiao::try_visit`) never recognizes the repeat, so the walk keeps
    // descending — empirically, until the accumulated path trips an OS path-length limit and the
    // scan spuriously fails on a structure that is not actually broken (never a true unbounded
    // hang here, since the OS bounds it first, but still a real, wrong reaction). Canonicalizing
    // closes it in two visits instead, matching 圭表/渾儀's own analogous symlink-cycle tests.
    let tb = TempBase::new("symlink-cycle");
    let root = tb.source(
        "mod.rs",
        "mod loop_mod;\nfn f() { assert_boundary!(\"a\", o); }\n",
    );
    tb.symlink(tb.path(), "loop_mod");
    let outcome = tb.audit(&[boundary("a", Severity::Enforce)], &[root]);
    assert_eq!(
        outcome,
        Outcome::Clean,
        "a real, declared, and probed seam must be covered, not hang or error: {outcome:?}"
    );
}

#[cfg(unix)]
#[test]
fn directory_audit_does_not_hang_on_a_symlinked_directory_cycle() {
    let tb = TempBase::new("dir-symlink-cycle");
    let _src = tb.source("main.rs", "fn f() { assert_boundary!(\"a\", o); }\n");
    tb.symlink(tb.path(), "loop_dir");
    let outcome = tb.audit(
        &[boundary("a", Severity::Enforce)],
        &[tb.path().to_path_buf()],
    );
    assert_eq!(
        outcome,
        Outcome::Clean,
        "directory scan on a cyclic symlinked dir must be covered without hanging or looping: {outcome:?}"
    );
}

#[test]
fn custom_marker_list_recognizes_user_probe_wrapper() {
    let tb = TempBase::new("custom-marker");
    let root = tb.source(
        "main.rs",
        "fn f() { my_custom_seam!(\"custom-seam\", obj); }\n",
    );
    let outcome = tb.audit_with_markers(
        &[boundary("custom-seam", Severity::Enforce)],
        &[root],
        &["assert_boundary", "my_custom_seam"],
    );
    assert_eq!(
        outcome,
        Outcome::Clean,
        "custom registered probe macro wrapper must cover the seam: {outcome:?}"
    );
}

#[test]
fn custom_marker_messages_name_the_configuration_and_actual_matched_marker() {
    let tb = TempBase::new("custom-marker-wording");
    let missing = tb.source("missing.rs", "fn f() {}\n");
    let missing_outcome = tb.audit_with_markers(
        &[boundary("custom-seam", Severity::Enforce)],
        &[missing],
        &["my_custom_seam"],
    );
    let Outcome::Violations(missing_report) = missing_outcome else {
        panic!("expected an unprobed violation");
    };
    assert!(
        missing_report.violations[0]
            .finding
            .contains("no configured probe marker")
    );
    assert!(
        !missing_report.violations[0]
            .finding
            .contains("assert_boundary")
    );

    let unauditable = tb.source("unauditable.rs", "fn f() { my_custom_seam!(SEAM, obj); }\n");
    let unauditable_outcome = tb.audit_with_markers(
        &[boundary("custom-seam", Severity::Enforce)],
        &[unauditable],
        &["my_custom_seam"],
    );
    let Outcome::Violations(unauditable_report) = unauditable_outcome else {
        panic!("expected unprobed and unauditable violations");
    };
    let violation = unauditable_report
        .violations
        .iter()
        .find(|violation| {
            violation.rule
                == "a configured probe marker's seam must be a string literal to be auditable"
        })
        .expect("unauditable violation");
    assert!(violation.finding.contains("my_custom_seam! probe"));
}

#[test]
fn anonymous_scope_header_ignores_delimiters_inside_literals_and_comments() {
    let source = br#"if value == "};" /* ; { } */ {"#;
    let brace = source.len() - 1;
    assert_eq!(
        scan::anonymous_scope_header(source, 0, brace),
        r#"if value == "};" /* ; { } */"#
    );
}

#[test]
fn literal_punctuation_in_anonymous_scopes_keeps_probe_owners_distinct_end_to_end() {
    let tb = TempBase::new("anonymous-owner-literal-punctuation");
    let dir = tb.dir(
        "two",
        r#"
fn outer() {
    if "a;b" == "x" {
        fn probe() { assert_boundary!(SEAM_A, o); }
    }
    if "a{b" == "x" {
        fn probe() { assert_boundary!(SEAM_A, o); }
    }
}
"#,
    );
    let outcome = tb.audit(&[boundary("s", Severity::Enforce)], &[dir]);
    let violations = unauditable_violations(&outcome);
    assert_eq!(
        violations.len(),
        2,
        "literal punctuation must not collapse distinct anonymous owners: {violations:?}"
    );
    let owners: std::collections::BTreeSet<_> = violations
        .iter()
        .map(|violation| {
            violation
                .fact()
                .fields()
                .find_map(|(name, value)| (name == "owner").then_some(value.to_string()))
                .expect("unauditable probe identity carries its owner")
        })
        .collect();
    assert_eq!(
        owners.len(),
        2,
        "the full audit path must retain both literal-distinguished owner headers: {violations:?}"
    );
    assert!(owners.iter().any(|owner| owner.contains(r#"if "a;b""#)));
    assert!(owners.iter().any(|owner| owner.contains(r#"if "a{b""#)));
}

#[test]
fn unregistered_custom_marker_is_ignored_by_audit() {
    let tb = TempBase::new("unregistered-marker");
    let root = tb.source(
        "main.rs",
        "fn f() { unknown_seam!(\"custom-seam\", obj); }\n",
    );
    let outcome = tb.audit_with_markers(
        &[boundary("custom-seam", Severity::Enforce)],
        &[root],
        &["assert_boundary", "my_custom_seam"],
    );
    assert!(
        matches!(outcome, Outcome::Violations(_)),
        "unregistered custom macro wrapper must be ignored and report declared seam unprobed: {outcome:?}"
    );
}

#[test]
fn empty_marker_list_is_constitution_error() {
    let tb = TempBase::new("empty-marker-list");
    let root = tb.source("main.rs", "fn f() { assert_boundary!(\"seam\", obj); }\n");
    let outcome = tb.audit_with_markers(&[boundary("seam", Severity::Enforce)], &[root], &[]);
    assert!(
        matches!(outcome, Outcome::ConstitutionError(_)),
        "empty markers list must be a constitution error: {outcome:?}"
    );
}

#[test]
fn blank_marker_string_is_constitution_error() {
    let tb = TempBase::new("blank-marker");
    let root = tb.source("main.rs", "fn f() { assert_boundary!(\"seam\", obj); }\n");
    let outcome = tb.audit_with_markers(&[boundary("seam", Severity::Enforce)], &[root], &["  "]);
    assert!(
        matches!(outcome, Outcome::ConstitutionError(_)),
        "blank marker string must be a constitution error: {outcome:?}"
    );
}

#[test]
fn invalid_marker_string_is_constitution_error() {
    let tb = TempBase::new("invalid-marker");
    let root = tb.source("main.rs", "fn f() { assert_boundary!(\"seam\", obj); }\n");
    for invalid in [
        "if", "match", "123foo", "foo::bar", "foo-bar", "_", "r#self", "r#super", "r#crate", "r#_",
        "💥", "a💥", "café",
    ] {
        let outcome = tb.audit_with_markers(
            &[boundary("seam", Severity::Enforce)],
            std::slice::from_ref(&root),
            &[invalid],
        );
        assert!(
            matches!(outcome, Outcome::ConstitutionError(_)),
            "invalid marker '{invalid}' must be a constitution error: {outcome:?}"
        );
    }
}

// --- transparent control-flow macro (`cfg_if!`) arms ------------------------
//
// `cfg_if!` wraps human-authored code in arms without transforming identities, so an arm's contents
// are real, compiled code. Skipping such a body like any foreign macro broke two of the audit's
// three reaction directions, measured on ordinary compilable source: a seam whose only probe lived
// in an arm was reported unprobed (a false alarm against real coverage), while a typo'd seam and an
// un-auditable probe inside an arm escaped entirely — the forbidden false negative, and a
// contradiction of `audit_probe_coverage`'s own never-a-silent-skip rule. 圭表 has read these bodies
// since 0.2.3 and 渾儀 joined in 0.3.1; these tests are 漏刻's half of one shared rule.

#[test]
fn a_probe_inside_a_cfg_if_arm_is_counted() {
    let src = "cfg_if::cfg_if! { if #[cfg(unix)] { fn f(o: u8) { assert_boundary!(\"arm\", o); } } }\n\
               fn g(o: u8) { assert_boundary!(\"top\", o); }";
    let mut probes = Vec::new();
    scan_source(src, "test.rs", &mut probes);
    assert_eq!(
        literal_seams(&probes),
        ["arm", "top"],
        "a probe inside a cfg_if arm must count, and the probe after the body must still be seen: {probes:?}"
    );
}

#[test]
fn probes_in_every_cfg_if_arm_are_counted() {
    // if / else-if / else — arms are a cfg-blind union, so all three count. The audit never
    // evaluates `cfg`, exactly as its contract already states for `#[cfg]`-gated probes.
    let src = "cfg_if::cfg_if! {\n\
               if #[cfg(unix)] { fn a(o: u8) { assert_boundary!(\"a\", o); } }\n\
               else if #[cfg(windows)] { fn b(o: u8) { assert_boundary!(\"b\", o); } }\n\
               else { fn c(o: u8) { assert_boundary!(\"c\", o); } }\n\
               }";
    let mut probes = Vec::new();
    scan_source(src, "test.rs", &mut probes);
    assert_eq!(
        literal_seams(&probes),
        ["a", "b", "c"],
        "every arm of an else-if chain must be scanned: {probes:?}"
    );
}

#[test]
fn a_probe_inside_a_nested_cfg_if_is_counted() {
    let src = "cfg_if::cfg_if! { if #[cfg(unix)] {\n\
               cfg_if::cfg_if! { if #[cfg(target_pointer_width = \"64\")] {\n\
               fn f(o: u8) { assert_boundary!(\"inner\", o); } } }\n\
               } }";
    let mut probes = Vec::new();
    scan_source(src, "test.rs", &mut probes);
    assert_eq!(
        literal_seams(&probes),
        ["inner"],
        "a nested cfg_if's arm must be scanned: {probes:?}"
    );
}

#[test]
fn a_paren_delimited_and_unqualified_cfg_if_are_both_transparent() {
    // The invocation's own delimiter is irrelevant, and an adopter who wrote `use cfg_if::cfg_if;`
    // invokes it bare — the name test matches the identifier before `!`, so both spellings are one
    // shape (the same last-segment match 圭表 and 渾儀 apply).
    let src = "cfg_if!( if #[cfg(unix)] { fn f(o: u8) { assert_boundary!(\"paren\", o); } } );\n\
               cfg_if! { if #[cfg(unix)] { fn g(o: u8) { assert_boundary!(\"brace\", o); } } }";
    let mut probes = Vec::new();
    scan_source(src, "test.rs", &mut probes);
    assert_eq!(
        literal_seams(&probes),
        ["paren", "brace"],
        "paren-delimited and unqualified cfg_if invocations must both be transparent: {probes:?}"
    );
}

#[test]
fn a_cfg_if_inside_a_macro_rules_body_is_still_skipped() {
    // Ordering guard: the outer `macro_rules!` body is skipped first, so a transparent invocation
    // written inside a macro TEMPLATE is never reached and its probe stays macro-generated. The
    // macro-definition exclusion is unaffected by transparency.
    let src = "macro_rules! gen { () => { cfg_if::cfg_if! { if #[cfg(unix)] { assert_boundary!(\"dead\", o) } } }; }\n\
               fn f(o: u8) { assert_boundary!(\"live\", o); }";
    let mut probes = Vec::new();
    scan_source(src, "test.rs", &mut probes);
    assert_eq!(
        literal_seams(&probes),
        ["live"],
        "a cfg_if inside a macro_rules template must stay skipped: {probes:?}"
    );
}

#[test]
fn a_foreign_macro_body_inside_a_cfg_if_arm_is_still_skipped() {
    // Transparency is not contagious: the arm is real code, but a foreign macro invoked INSIDE the
    // arm is still macro-generated, so its probe must not count while the arm's own does.
    let src = "cfg_if::cfg_if! { if #[cfg(unix)] {\n\
               fn f(o: u8) { wrap!( assert_boundary!(\"dead\", o) ); assert_boundary!(\"live\", o); }\n\
               } }";
    let mut probes = Vec::new();
    scan_source(src, "test.rs", &mut probes);
    assert_eq!(
        literal_seams(&probes),
        ["live"],
        "a foreign macro body inside a transparent arm must still be skipped: {probes:?}"
    );
}

#[test]
fn a_seam_probed_only_inside_a_cfg_if_arm_is_covered() {
    let tb = TempBase::new("arm-covered");
    let root = tb.source(
        "lib.rs",
        "cfg_if::cfg_if! { if #[cfg(unix)] { pub fn f(o: u8) { assert_boundary!(\"seam\", o); } } }",
    );
    let outcome = tb.audit(&[boundary("seam", Severity::Enforce)], &[root]);
    assert_eq!(
        outcome,
        Outcome::Clean,
        "a seam probed only inside an arm must be covered, not reported unprobed: {outcome:?}"
    );
}

#[test]
fn a_seam_probed_nowhere_is_still_reported_unprobed() {
    // The control for the test above: without it, "Clean" could hold for a fixture whose boundary
    // never reacts at all rather than because the arm's probe was found.
    let tb = TempBase::new("arm-control");
    let root = tb.source("lib.rs", "pub fn f() {}");
    let outcome = tb.audit(&[boundary("seam", Severity::Enforce)], &[root]);
    assert_eq!(
        outcome.exit_code(),
        1,
        "an unprobed seam must still react: {outcome:?}"
    );
}

#[test]
fn a_typod_seam_inside_a_cfg_if_arm_reacts() {
    // Closed false negative #1: probed-but-undeclared never fired inside an arm, so a mis-typed
    // seam — which panics at runtime against a seam nobody declared — passed CI.
    let tb = TempBase::new("arm-typo");
    let root = tb.source(
        "lib.rs",
        "cfg_if::cfg_if! { if #[cfg(unix)] { pub fn f(o: u8) { assert_boundary!(\"seaam\", o); } } }\n\
         pub fn g(o: u8) { assert_boundary!(\"seam\", o); }",
    );
    let outcome = tb.audit(&[boundary("seam", Severity::Enforce)], &[root]);
    assert_eq!(
        outcome.exit_code(),
        1,
        "a typo'd seam inside an arm must react as probed-but-undeclared: {outcome:?}"
    );
}

#[test]
fn an_unauditable_probe_inside_a_cfg_if_arm_reacts_with_its_lexical_owner() {
    // Closed false negative #2, with its identity pinned. The owner qualifies the probe by the
    // anonymous block scopes it genuinely sits in — the invocation's own body and the arm — exactly
    // as it would for a real `if` block's braces. That is 漏刻's existing rule for any anonymous
    // scope, applied unchanged rather than special-cased for arms, and it names the arm in the
    // message, which an adopter reading a violation wants.
    let tb = TempBase::new("arm-unauditable");
    let root = tb.source(
        "lib.rs",
        "const S: &str = \"seam\";\n\
         cfg_if::cfg_if! { if #[cfg(unix)] { pub fn f(o: u8) { assert_boundary!(S, o); } } }",
    );
    let outcome = tb.audit(&[boundary("seam", Severity::Enforce)], &[root]);
    let text = format!("{outcome:?}");
    assert!(
        text.contains("non-literal seam `S`"),
        "an un-auditable probe inside an arm must react, never a silent skip: {text}"
    );
    assert!(
        text.contains("block cfg_if::cfg_if!#1::block if #[cfg(unix)]#1::fn f"),
        "the un-auditable probe's owner must name its real lexical scopes: {text}"
    );
}

#[test]
fn a_module_declared_inside_a_cfg_if_arm_covers_a_seam() {
    // The module-graph half: a dimension blind to the arm never reaches the file at all, so every
    // probe beneath it is invisible — a coverage false negative that costs a whole subtree.
    let tb = TempBase::new("arm-mod");
    let root = tb.source(
        "lib.rs",
        "cfg_if::cfg_if! { if #[cfg(unix)] { pub mod net; } }",
    );
    tb.source(
        "net.rs",
        "pub fn f(o: u8) { assert_boundary!(\"seam\", o); }",
    );
    let outcome = tb.audit(&[boundary("seam", Severity::Enforce)], &[root]);
    assert_eq!(
        outcome,
        Outcome::Clean,
        "an arm-declared module's probe must count: {outcome:?}"
    );
}

#[test]
fn a_module_declared_after_a_cfg_if_body_is_still_reached() {
    // Range-resumption guard: the arm descent must not consume the enclosing walk's cursor past the
    // invocation, or a declaration following the body would be dropped (and its probes with it).
    let tb = TempBase::new("arm-then-mod");
    let root = tb.source(
        "lib.rs",
        "cfg_if::cfg_if! { if #[cfg(unix)] { pub mod first; } }\npub mod second;",
    );
    tb.source(
        "first.rs",
        "pub fn a(o: u8) { assert_boundary!(\"one\", o); }",
    );
    tb.source(
        "second.rs",
        "pub fn b(o: u8) { assert_boundary!(\"two\", o); }",
    );
    let outcome = tb.audit(
        &[
            boundary("one", Severity::Enforce),
            boundary("two", Severity::Enforce),
        ],
        &[root],
    );
    assert_eq!(
        outcome,
        Outcome::Clean,
        "a module declared after a transparent body must still be reached: {outcome:?}"
    );
}

#[test]
fn an_arm_declared_module_with_no_file_is_tolerated() {
    // Arm membership is cfg-conditional: the predicate lives in the macro header, so rustc strips
    // the whole arm and the crate compiles with no such file. 圭表's settled rule, adopted.
    let tb = TempBase::new("arm-absent");
    let root = tb.source(
        "lib.rs",
        "cfg_if::cfg_if! { if #[cfg(unix)] { pub mod unix_impl; } else { pub mod windows_impl; } }",
    );
    tb.source(
        "unix_impl.rs",
        "pub fn f(o: u8) { assert_boundary!(\"seam\", o); }",
    );
    let outcome = tb.audit(&[boundary("seam", Severity::Enforce)], &[root]);
    assert_eq!(
        outcome,
        Outcome::Clean,
        "a fileless arm declaration must be tolerated, not a constitution error: {outcome:?}"
    );
}

#[test]
fn the_same_fileless_declaration_outside_an_arm_still_fails_loud() {
    // The control for the tolerance: an unconditional declaration with no file is a broken
    // reference (exit 2), so the tolerance above is a decision about arms, not a blanket softening.
    let tb = TempBase::new("arm-absent-control");
    let root = tb.source("lib.rs", "pub mod windows_impl;");
    let outcome = tb.audit(&[boundary("seam", Severity::Enforce)], &[root]);
    assert_eq!(
        outcome.exit_code(),
        2,
        "an unconditional fileless declaration must stay a constitution error: {outcome:?}"
    );
}

#[test]
fn arm_membership_is_not_inherited_into_an_inline_module_body() {
    // The tolerance covers the arm's own declarations, not everything beneath them: a bare `#[cfg]`
    // on an outer `mod` does not tolerate an absent file for an inner `mod` either, in any of the
    // three dimensions. Keeping that asymmetry identical across them is the point.
    let tb = TempBase::new("arm-inline-inherit");
    let root = tb.source(
        "lib.rs",
        "cfg_if::cfg_if! { if #[cfg(unix)] { pub mod outer { pub mod missing; } } }",
    );
    let outcome = tb.audit(&[boundary("seam", Severity::Enforce)], &[root]);
    assert_eq!(
        outcome.exit_code(),
        2,
        "arm membership must not be inherited into an inline module body: {outcome:?}"
    );
}

#[test]
fn an_arm_declared_dual_backed_module_is_still_a_constitution_error() {
    // Arm membership makes an ABSENCE tolerable, never an ambiguity resolvable: no predicate value
    // makes two conventional files compile as one module. The same ordering all three dimensions
    // apply to this shape.
    let tb = TempBase::new("arm-dual-backed");
    let root = tb.source(
        "lib.rs",
        "cfg_if::cfg_if! { if #[cfg(unix)] { pub mod child; } }",
    );
    tb.source("child.rs", "pub fn a() {}");
    tb.source("child/mod.rs", "pub fn b() {}");
    let outcome = tb.audit(&[boundary("seam", Severity::Enforce)], &[root]);
    assert_eq!(
        outcome.exit_code(),
        2,
        "an arm-declared dual-backed module must stay a constitution error: {outcome:?}"
    );
}

#[test]
fn a_clean_cfg_if_arm_stays_clean() {
    // Transparency observes contents; it does not react to the macro. A crate using `cfg_if!`
    // cleanly must not acquire a finding merely for using it.
    let tb = TempBase::new("arm-clean");
    let root = tb.source(
        "lib.rs",
        "cfg_if::cfg_if! { if #[cfg(unix)] { pub fn f() -> u8 { 0 } } }\n\
         pub fn g(o: u8) { assert_boundary!(\"seam\", o); }",
    );
    let outcome = tb.audit(&[boundary("seam", Severity::Enforce)], &[root]);
    assert_eq!(
        outcome,
        Outcome::Clean,
        "a clean cfg_if arm must stay clean: {outcome:?}"
    );
}

#[test]
fn a_spaced_transparent_invocation_is_transparent_in_both_passes() {
    // `cfg_if ! { … }` is valid Rust, and both passes look back past whitespace for the name before
    // deciding a `!` opens a macro — so the spaced form must be recognized as transparent too. Were
    // it recognized as a macro but not as transparent, the probe pass would skip its body and the
    // module pass would swallow the arm as an opaque block: the false negative in both halves.
    let src = "cfg_if ! { if #[cfg(unix)] { fn f(o: u8) { assert_boundary!(\"spaced\", o); } } }";
    let mut probes = Vec::new();
    scan_source(src, "test.rs", &mut probes);
    assert_eq!(
        literal_seams(&probes),
        ["spaced"],
        "a spaced transparent invocation must be scanned into: {probes:?}"
    );

    let tb = TempBase::new("spaced-arm-mod");
    let root = tb.source("lib.rs", "cfg_if ! { if #[cfg(unix)] { pub mod net; } }");
    tb.source(
        "net.rs",
        "pub fn f(o: u8) { assert_boundary!(\"seam\", o); }",
    );
    let outcome = tb.audit(&[boundary("seam", Severity::Enforce)], &[root]);
    assert_eq!(
        outcome,
        Outcome::Clean,
        "a module declared inside a spaced transparent invocation's arm must be reached: {outcome:?}"
    );
}

#[test]
fn a_similarly_named_macro_is_not_transparent() {
    // The name gate compares the MAXIMAL identifier run, so `my_cfg_if!` is a different macro and its
    // body stays skipped. A suffix/substring match here would read an arbitrary macro's nested blocks
    // as arms — the false-positive direction the gate exists to prevent.
    let src = "fn f(o: u8) { my_cfg_if!( assert_boundary!(\"dead\", o) ); assert_boundary!(\"live\", o); }";
    let mut probes = Vec::new();
    scan_source(src, "test.rs", &mut probes);
    assert_eq!(
        literal_seams(&probes),
        ["live"],
        "only the exact `cfg_if` name is transparent: {probes:?}"
    );
}

#[test]
fn a_module_declared_inside_a_nested_cfg_if_arm_is_reached() {
    // The module pass's arm descent re-enters the walk, so a nested invocation is covered by the same
    // recursion rather than a second mechanism — pinned here because the probe pass's nesting test
    // exercises a different code path.
    let tb = TempBase::new("nested-arm-mod");
    let root = tb.source(
        "lib.rs",
        "cfg_if::cfg_if! { if #[cfg(unix)] {\n\
         cfg_if::cfg_if! { if #[cfg(target_pointer_width = \"64\")] { pub mod deep; } }\n\
         } }",
    );
    tb.source(
        "deep.rs",
        "pub fn f(o: u8) { assert_boundary!(\"seam\", o); }",
    );
    let outcome = tb.audit(&[boundary("seam", Severity::Enforce)], &[root]);
    assert_eq!(
        outcome,
        Outcome::Clean,
        "a module declared inside a nested cfg_if arm must be reached: {outcome:?}"
    );
}

#[test]
fn twin_arms_declaring_one_module_do_not_double_report_its_probe() {
    // The per-platform shim spelled with ONE file: both arms declare the same module, so the arm
    // descent collects it twice. The walk's canonical-path visit tracking must collapse that to one
    // scan — otherwise a single un-auditable probe beneath it would be reported twice, inflating one
    // real finding into two (the false-positive direction of this change).
    let tb = TempBase::new("twin-arms");
    let root = tb.source(
        "lib.rs",
        "cfg_if::cfg_if! { if #[cfg(unix)] { pub mod imp; } else { pub mod imp; } }",
    );
    tb.source(
        "imp.rs",
        "const S: &str = \"seam\";\npub fn f(o: u8) { assert_boundary!(S, o); }\n\
         pub fn g(o: u8) { assert_boundary!(\"seam\", o); }",
    );
    let outcome = tb.audit(&[boundary("seam", Severity::Enforce)], &[root]);
    let count = match &outcome {
        Outcome::Violations(report) => report.violations.len(),
        other => panic!("expected violations, got {other:?}"),
    };
    assert_eq!(
        count, 1,
        "twin arms declaring one module must yield exactly one un-auditable finding: {outcome:?}"
    );
}

#[test]
fn an_absent_path_remap_target_inside_a_cfg_if_arm_is_tolerated() {
    // The other absence site reads the same gate: a bare `#[cfg]` co-occurring with an unconditional
    // `#[path]` removes the whole item, `#[path]` included, and arm membership expresses the same
    // intent. Written because the delta claims it — an untested spec claim is the shape that rots.
    let tb = TempBase::new("arm-absent-path");
    let root = tb.source(
        "lib.rs",
        "cfg_if::cfg_if! { if #[cfg(windows)] { #[path = \"windows_impl.rs\"] pub mod imp; } }\n\
         pub fn f(o: u8) { assert_boundary!(\"seam\", o); }",
    );
    let outcome = tb.audit(&[boundary("seam", Severity::Enforce)], &[root]);
    assert_eq!(
        outcome,
        Outcome::Clean,
        "an absent unconditional #[path] target inside an arm must be tolerated: {outcome:?}"
    );
}

/// A comment between the `mod` keyword and its name is legal, unremarkable Rust (trivia to rustc)
/// — but a bare whitespace-only skip in that position stopped at the comment's leading `/`, so the
/// identifier scan found nothing there and the whole declaration was never recognized as a `mod` at
/// all. The module and its entire subtree — every probe beneath it — silently vanished from the
/// corpus (exit 0 Clean) instead of reacting to the typo'd seam it actually contains.
#[test]
fn a_comment_between_mod_and_its_name_does_not_drop_the_module() {
    let tb = TempBase::new("comment-before-name");
    let root = tb.source(
        "lib.rs",
        "pub mod /* relocated */ child;\npub fn p(o: u8) { assert_boundary!(\"seam\", o); }",
    );
    tb.source(
        "child.rs",
        "pub fn q(o: u8) { assert_boundary!(\"seaam\", o); }",
    );
    let outcome = tb.audit(&[boundary("seam", Severity::Enforce)], &[root]);
    assert_eq!(
        outcome.exit_code(),
        1,
        "the typo'd seam in the comment-relocated module must react: {outcome:?}"
    );
}

/// The identical shape with the comment AFTER the module's name, before its terminator.
#[test]
fn a_comment_between_the_mod_name_and_its_terminator_does_not_drop_the_module() {
    let tb = TempBase::new("comment-after-name");
    let root = tb.source(
        "lib.rs",
        "pub mod child /* relocated */;\npub fn p(o: u8) { assert_boundary!(\"seam\", o); }",
    );
    tb.source(
        "child.rs",
        "pub fn q(o: u8) { assert_boundary!(\"seaam\", o); }",
    );
    let outcome = tb.audit(&[boundary("seam", Severity::Enforce)], &[root]);
    assert_eq!(
        outcome.exit_code(),
        1,
        "the typo'd seam in the comment-relocated module must react: {outcome:?}"
    );
}

/// The only legal non-inline module form inside a function/block body is one carrying `#[path]`
/// (a bare `mod name;` with no established file-path convention there does not compile) — but the
/// catch-all brace skip treated every non-`mod`, non-arm brace as one opaque unit, so this legal
/// form was never observed: the module and the typo'd seam it contains silently vanished from the
/// corpus (exit 0 Clean).
#[test]
fn a_path_mod_inside_a_function_body_reacts() {
    let tb = TempBase::new("block-scoped-path-mod");
    let root = tb.source(
        "lib.rs",
        "pub fn f() { #[path = \"inner.rs\"] mod inner; }\npub fn p(o: u8) { assert_boundary!(\"seam\", o); }",
    );
    tb.source(
        "inner.rs",
        "pub fn q(o: u8) { assert_boundary!(\"seaam\", o); }",
    );
    let outcome = tb.audit(&[boundary("seam", Severity::Enforce)], &[root]);
    assert_eq!(
        outcome.exit_code(),
        1,
        "the typo'd seam in the block-scoped #[path] module must react: {outcome:?}"
    );
}

/// The identical shape nested one bare block deeper (`{ { #[path] mod inner; } }`), confirming the
/// generalized brace descent is not narrowly scoped to a function's own immediate body.
#[test]
fn a_path_mod_inside_a_nested_bare_block_reacts() {
    let tb = TempBase::new("nested-block-scoped-path-mod");
    let root = tb.source(
        "lib.rs",
        "pub fn f() { { #[path = \"inner.rs\"] mod inner; } }\npub fn p(o: u8) { assert_boundary!(\"seam\", o); }",
    );
    tb.source(
        "inner.rs",
        "pub fn q(o: u8) { assert_boundary!(\"seaam\", o); }",
    );
    let outcome = tb.audit(&[boundary("seam", Severity::Enforce)], &[root]);
    assert_eq!(
        outcome.exit_code(),
        1,
        "the typo'd seam in the nested-block #[path] module must react: {outcome:?}"
    );
}

/// `mod_preamble_attrs` documented a `cfg_attr(path)` tolerance the code never implemented: the
/// attribute-matching pass checked for the exact identifier `cfg`, so `cfg_attr` — a different
/// identifier — matched neither the `path` arm nor the bare-`cfg` arm. A module stacking two
/// `cfg_attr`-wrapped `#[path]` declarations that together cover every platform (both targets
/// present, compiling cleanly on every configuration) was reported a hard constitution error
/// instead of being scanned.
#[test]
fn two_cfg_attr_path_declarations_covering_every_platform_are_scanned_not_erred() {
    let tb = TempBase::new("two-cfg-attr-path");
    let root = tb.source(
        "lib.rs",
        "#[cfg_attr(unix, path = \"u.rs\")]\n#[cfg_attr(not(unix), path = \"w.rs\")]\npub mod plat;\npub fn p(o: u8) { assert_boundary!(\"seam\", o); }",
    );
    tb.source(
        "u.rs",
        "pub fn q(o: u8) { assert_boundary!(\"seaam\", o); }",
    );
    tb.source(
        "w.rs",
        "pub fn r(o: u8) { assert_boundary!(\"seaam\", o); }",
    );
    let outcome = tb.audit(&[boundary("seam", Severity::Enforce)], &[root]);
    assert_eq!(
        outcome.exit_code(),
        1,
        "both cfg_attr(path) targets must be scanned (typo'd seam reacts), never a constitution \
         error on source that compiles on every platform: {outcome:?}"
    );
}

/// The identical shape with clean seams in both cfg_attr(path) targets — confirms the fix reports
/// the boundary satisfied (not merely "not a constitution error").
#[test]
fn two_cfg_attr_path_declarations_covering_every_platform_are_clean_when_probes_match() {
    let tb = TempBase::new("two-cfg-attr-path-clean");
    let root = tb.source(
        "lib.rs",
        "#[cfg_attr(unix, path = \"u.rs\")]\n#[cfg_attr(not(unix), path = \"w.rs\")]\npub mod plat;\npub fn p(o: u8) { assert_boundary!(\"seam\", o); }",
    );
    tb.source("u.rs", "pub fn q(o: u8) { assert_boundary!(\"seam\", o); }");
    tb.source("w.rs", "pub fn r(o: u8) { assert_boundary!(\"seam\", o); }");
    let outcome = tb.audit(&[boundary("seam", Severity::Enforce)], &[root]);
    assert_eq!(
        outcome,
        Outcome::Clean,
        "source compiling cleanly on every platform, with every probe matching the declared seam, \
         must be Clean: {outcome:?}"
    );
}

/// A cfg_attr(path) target that does NOT exist on disk is skipped, not erred, when either the
/// conventional file or another cfg_attr candidate backs the module — the union-observation
/// counterpart of the crate-wide walk's own absence tolerance.
#[test]
fn a_missing_cfg_attr_path_target_is_tolerated_when_the_conventional_file_backs_the_module() {
    let tb = TempBase::new("cfg-attr-path-missing-target-conventional-backs");
    let root = tb.source(
        "lib.rs",
        "#[cfg_attr(windows, path = \"win.rs\")]\npub mod plat;\npub fn p(o: u8) { assert_boundary!(\"seam\", o); }",
    );
    tb.source(
        "plat.rs",
        "pub fn q(o: u8) { assert_boundary!(\"seaam\", o); }",
    );
    let outcome = tb.audit(&[boundary("seam", Severity::Enforce)], &[root]);
    assert_eq!(
        outcome.exit_code(),
        1,
        "the conventional file must still be read and react, even with an absent sibling \
         cfg_attr(path) target: {outcome:?}"
    );
}

/// A cfg_attr(path) remap on an INLINE `mod x { … }` (not the external `mod x;` form) redirects
/// where x's own nested items resolve from — the same union rule applied to a base directory
/// instead of a file existence check, since the inline body itself is always present in source.
#[test]
fn a_cfg_attr_path_remap_on_an_inline_module_redirects_its_nested_items() {
    let tb = TempBase::new("cfg-attr-path-inline-module-redirect");
    let root = tb.source(
        "lib.rs",
        "#[cfg_attr(unix, path = \"unix_dir\")]\npub mod x {\n    pub mod y;\n}\npub fn p(o: u8) { assert_boundary!(\"seam\", o); }",
    );
    tb.source(
        "unix_dir/y.rs",
        "pub fn q(o: u8) { assert_boundary!(\"seaam\", o); }",
    );
    let outcome = tb.audit(&[boundary("seam", Severity::Enforce)], &[root]);
    assert_eq!(
        outcome.exit_code(),
        1,
        "the cfg_attr(path)-remapped directory must be followed for x's nested `mod y;`, not the \
         conventional (nonexistent) `x/y.rs`, and never a constitution error: {outcome:?}"
    );
}

/// The un-auditable-probe fact's identity must not embed a raw, checkout-dependent absolute path:
/// a byte-identical source file scanned from two different absolute locations (the same
/// relocation a different clone path / CI runner produces) must yield the IDENTICAL violation
/// identity, or a baseline recorded in one checkout matches nothing in the other.
#[test]
fn unauditable_probe_identity_is_stable_across_checkout_locations() {
    let tb1 = TempBase::new("probefix1");
    let root1 = tb1.source(
        "src/lib.rs",
        "pub const SEAM: &str = \"seam\";\npub fn go(o: u8) { assert_boundary!(SEAM, o); }",
    );
    let tb2 = TempBase::new("probefix2");
    let root2 = tb2.source(
        "src/lib.rs",
        "pub const SEAM: &str = \"seam\";\npub fn go(o: u8) { assert_boundary!(SEAM, o); }",
    );
    // Each checkout anchors on its OWN base — exactly what a real caller does, since
    // `workspace_root` is that checkout's own directory. The two absolute prefixes differ; the
    // labels, and therefore the identities, must not.
    let outcome1 = tb1.audit(&[boundary("seam", Severity::Enforce)], &[root1]);
    let outcome2 = tb2.audit(&[boundary("seam", Severity::Enforce)], &[root2]);
    let Outcome::Violations(report1) = outcome1 else {
        panic!("expected violations from checkout 1: {outcome1:?}");
    };
    let Outcome::Violations(report2) = outcome2 else {
        panic!("expected violations from checkout 2: {outcome2:?}");
    };
    let ids1: Vec<_> = report1.violations.iter().map(|v| v.id()).collect();
    let ids2: Vec<_> = report2.violations.iter().map(|v| v.id()).collect();
    assert_eq!(
        ids1, ids2,
        "the same source scanned from two different absolute checkout locations must produce \
         identical violation identities, or a baseline recorded in one never matches the other"
    );
    // Non-vacuous: the identity is not merely absent (e.g. both empty) — an unauditable-probe
    // violation genuinely fired, and its `file` field is relative, never the raw absolute path.
    let unauditable = report1
        .violations
        .iter()
        .find(|v| v.rule.contains("string literal"))
        .expect("an unauditable-probe violation must have fired");
    let file = unauditable.file.as_deref().expect("file field must be set");
    assert_eq!(
        file, "src/lib.rs",
        "a scanned file is labeled by its place within the checkout root, not by its own directory \
         — which is what keeps two members' same-named `lib.rs` apart"
    );
    assert!(
        !file.starts_with('/'),
        "the identity's file label must never be a raw absolute path: {file}"
    );
}

/// The second half of identity stability, and the reason the anchor is the caller's argument rather
/// than something derived from `source_inputs`: one checkout, one anchor, but two different member
/// SETS must label a file they share identically.
///
/// A derived longest-common-prefix anchor cannot do this. With every member under `crates/` the
/// derived anchor is `<root>/crates`, labeling a file `a/src/lib.rs`; adding one member outside that
/// prefix (`tools/c`, or an example, or a fixture crate) drops the anchor to `<root>` and relabels
/// the very same file `crates/a/src/lib.rs`. Every recorded baseline entry then goes stale and
/// re-fires as new at once — the exact loss the checkout-relative labeling exists to prevent,
/// reached by adding an unrelated crate instead of by moving the clone.
#[test]
fn two_member_sets_over_one_checkout_label_a_shared_file_identically() {
    let tb = TempBase::new("member-set-stability");
    let shared = tb.source(
        "crates/a/src/lib.rs",
        "pub const SEAM: &str = \"seam\";\npub fn go(o: u8) { assert_boundary!(SEAM, o); }",
    );
    let sibling = tb.source(
        "crates/b/src/lib.rs",
        "pub const SEAM: &str = \"seam\";\npub fn go(o: u8) { assert_boundary!(SEAM, o); }",
    );
    // The added member sits outside the `crates/` prefix the first two share — the shape that moves
    // a derived anchor. The anchor itself (the checkout root) is identical in both calls, as it is
    // for a real caller reading `workspace_root` before and after the member is added.
    let outsider = tb.source(
        "tools/c/src/lib.rs",
        "pub const SEAM: &str = \"seam\";\npub fn go(o: u8) { assert_boundary!(SEAM, o); }",
    );
    let label_of = |roots: &[PathBuf]| -> Vec<String> {
        let outcome = tb.audit(&[boundary("seam", Severity::Enforce)], roots);
        let Outcome::Violations(report) = outcome else {
            panic!("expected violations: {outcome:?}");
        };
        report
            .violations
            .iter()
            .filter(|v| v.rule.contains("string literal"))
            .filter_map(|v| v.file.clone())
            .filter(|file| file.starts_with("crates/a/"))
            .collect()
    };
    let before = label_of(&[shared.clone(), sibling.clone()]);
    let after = label_of(&[shared, sibling, outsider]);
    assert_eq!(
        before,
        vec!["crates/a/src/lib.rs".to_string()],
        "the label must name the file's place in the checkout, whatever else is being scanned"
    );
    assert_eq!(
        before, after,
        "adding a workspace member outside the other members' shared prefix must not relabel their \
         findings — every baseline entry recorded against the old label would otherwise go stale \
         and re-fire as new"
    );
}

/// Multiple workspace-member roots (the real `tianheng` caller's shape, one absolute `src_path`
/// per member from `cargo_metadata`) are each labeled relative to the checkout root the caller
/// anchors on — never a raw absolute path, and distinct members never collide despite sharing a
/// bare `lib.rs` filename.
#[test]
fn multi_root_probe_identity_is_relative_to_the_checkout_anchor() {
    let tb = TempBase::new("multi-root-common-ancestor");
    let root_a = tb.source(
        "crate-a/src/lib.rs",
        "pub const SEAM: &str = \"seam\";\npub fn go(o: u8) { assert_boundary!(SEAM, o); }",
    );
    let root_b = tb.source(
        "crate-b/src/lib.rs",
        "pub const SEAM: &str = \"seam\";\npub fn go(o: u8) { assert_boundary!(SEAM, o); }",
    );
    let outcome = tb.audit(&[boundary("seam", Severity::Enforce)], &[root_a, root_b]);
    let Outcome::Violations(report) = outcome else {
        panic!("expected violations: {outcome:?}");
    };
    let mut files: Vec<&str> = report
        .violations
        .iter()
        .filter(|v| v.rule.contains("string literal"))
        .filter_map(|v| v.file.as_deref())
        .collect();
    files.sort_unstable();
    assert_eq!(
        files,
        vec!["crate-a/src/lib.rs", "crate-b/src/lib.rs"],
        "each member's identity must be relative to the shared checkout root, distinguishing \
         same-named files by their own member path"
    );
}

/// Stated bound (documented in `finding.rs`/`audit.rs`): an ABSOLUTE `#[path = "/…"]` literal whose
/// target does NOT happen to lie under the scanning checkout's own anchor directory has no textual
/// relationship to it (`Path::join` discards the receiver entirely for an absolute joinee), so its
/// identity falls back to the raw absolute path — never silently dropped (the violation still
/// fires), just not relabeled. (When the target DOES happen to lie under the anchor, the label is
/// relative instead, and the identity can still disagree across checkouts — a separate, KNOWN
/// residual gap pinned by `a_nested_absolute_path_literal_still_disagrees_across_checkouts_a_known_residual_gap`
/// below, not silently ignored.) An absolute literal is already a non-portable, machine-specific
/// construct on its own either way.
#[test]
fn an_absolute_path_literal_falls_back_to_the_absolute_label_a_stated_bound() {
    let tb = TempBase::new("abs-path-literal-bound");
    // The target must sit OUTSIDE the anchor for the fallback to be what is under test, so it goes
    // in a separate base rather than a sibling directory inside this one. With the anchor now the
    // caller's checkout root (not the scanned file's own directory), a target *inside* the checkout
    // gets a relative label instead — pinned by the sibling test below, which is a real narrowing
    // of this bound rather than a change of it.
    let outside = TempBase::new("abs-path-literal-outside");
    let target_dir = outside.path().join("shared_outside");
    std::fs::create_dir_all(&target_dir).unwrap();
    let abs_target = target_dir.join("thing.rs");
    std::fs::write(
        &abs_target,
        "pub fn q(o: u8) { assert_boundary!(SEAM_CONST, o); }",
    )
    .unwrap();
    let root = tb.source(
        "crates/foo/src/lib.rs",
        &format!(
            "pub const SEAM_CONST: &str = \"seam\";\n#[path = {:?}]\nmod thing;",
            abs_target.display().to_string()
        ),
    );
    let outcome = tb.audit(&[boundary("seam", Severity::Enforce)], &[root]);
    let Outcome::Violations(report) = outcome else {
        panic!("expected an unauditable-probe violation to fire: {outcome:?}");
    };
    let unauditable = report
        .violations
        .iter()
        .find(|v| v.rule.contains("string literal"))
        .expect("an absolute #[path] target's probe must still react, never silently dropped");
    let file = unauditable.file.as_deref().expect("file field must be set");
    assert_eq!(
        file,
        abs_target.display().to_string(),
        "an absolute #[path] literal's target outside the anchor has no relationship to it, so its \
         label stays the raw absolute path — a documented, deliberate bound, not a silent regression"
    );
}

/// The narrowing the explicit anchor brings to the bound above: an absolute `#[path]` literal whose
/// target lies INSIDE the caller's checkout root is now labeled relative to it, so that identity is
/// checkout-independent like every other. Under the previous derived anchor (the scanned file's own
/// directory) a sibling-tree target like this fell outside it and kept the absolute label; the
/// workspace root contains it, so it no longer does. Only a target genuinely outside the checkout
/// keeps the absolute fallback, and only the cross-checkout case of THIS shape stays open (the
/// residual gap pinned below).
#[test]
fn an_absolute_path_literal_inside_the_anchor_is_labeled_relative_to_it() {
    let tb = TempBase::new("abs-path-literal-inside");
    let abs_target = tb.path().join("crates/shared/src/thing.rs");
    std::fs::create_dir_all(abs_target.parent().unwrap()).unwrap();
    std::fs::write(
        &abs_target,
        "pub fn q(o: u8) { assert_boundary!(SEAM_CONST, o); }",
    )
    .unwrap();
    let root = tb.source(
        "crates/foo/src/lib.rs",
        &format!(
            "pub const SEAM_CONST: &str = \"seam\";\n#[path = {:?}]\nmod thing;",
            abs_target.display().to_string()
        ),
    );
    let outcome = tb.audit(&[boundary("seam", Severity::Enforce)], &[root]);
    let Outcome::Violations(report) = outcome else {
        panic!("expected an unauditable-probe violation to fire: {outcome:?}");
    };
    let unauditable = report
        .violations
        .iter()
        .find(|v| v.rule.contains("string literal"))
        .expect("an absolute #[path] target's probe must still react, never silently dropped");
    let file = unauditable.file.as_deref().expect("file field must be set");
    assert_eq!(
        file, "crates/shared/src/thing.rs",
        "a target under the checkout root is labeled relative to it, not left absolute"
    );
}

/// KNOWN, DEFERRED residual gap (see `BACKLOG.md`'s DESIGN-BREAKING decision index and PR #157's
/// commit body, which recorded this as an explicit non-goal): when an absolute `#[path]` literal's target
/// happens to be textually nested under a GIVEN checkout's own anchor, `strip_prefix` succeeds by
/// pure text match — producing a clean, relative-LOOKING label — even though the literal itself is
/// fixed text that does not move with the checkout. The identical hardcoded literal scanned from a
/// DIFFERENT checkout (where it no longer shares the anchor's prefix) falls back to the full
/// absolute path instead, so the two checkouts still disagree — reproducing the very
/// checkout-dependent-identity problem this whole fix exists to close, just for this one
/// deliberately out-of-scope construct. Pinned here (not silently left untested) so a future fix
/// has a failing case to work against, and so this test itself fails loud if that future fix
/// changes this behavior without updating the assertion.
#[test]
fn a_nested_absolute_path_literal_still_disagrees_across_checkouts_a_known_residual_gap() {
    let tb_a = TempBase::new("nested-abs-checkout-a");
    let tb_b = TempBase::new("nested-abs-checkout-b");
    let abs_target = tb_a
        .path()
        .join("crates/foo/src/nested_under_anchor/thing.rs");
    std::fs::create_dir_all(abs_target.parent().unwrap()).unwrap();
    std::fs::write(
        &abs_target,
        "pub fn q(o: u8) { assert_boundary!(SEAM_CONST, o); }",
    )
    .unwrap();
    // The identical hardcoded literal (checkout a's own absolute path) is committed into BOTH
    // checkouts' source, exactly as a real clone would carry it verbatim.
    let lib_body = format!(
        "pub const SEAM_CONST: &str = \"seam\";\n#[path = {:?}]\nmod thing;",
        abs_target.display().to_string()
    );
    let root_a = tb_a.source("crates/foo/src/lib.rs", &lib_body);
    let root_b = tb_b.source("crates/foo/src/lib.rs", &lib_body);

    // Each checkout anchors on its own root, as a real caller does. The literal names checkout a's
    // path, so it lies under a's anchor and outside b's — which is the whole gap.
    let Outcome::Violations(report_a) =
        tb_a.audit(&[boundary("seam", Severity::Enforce)], &[root_a])
    else {
        panic!("expected a violation from checkout a");
    };
    let Outcome::Violations(report_b) =
        tb_b.audit(&[boundary("seam", Severity::Enforce)], &[root_b])
    else {
        panic!("expected a violation from checkout b");
    };
    let id_a = report_a
        .violations
        .iter()
        .find_map(|v| v.rule.contains("string literal").then(|| v.id()));
    let id_b = report_b
        .violations
        .iter()
        .find_map(|v| v.rule.contains("string literal").then(|| v.id()));
    assert_ne!(
        id_a, id_b,
        "this pins the KNOWN residual gap: a nested absolute #[path] literal's identity still \
         differs across checkouts (checkout a's own anchor happens to make it relative; \
         checkout b's does not) — if this ever starts passing with equal IDs, the gap has been \
         fixed and this test's assertion (and the design.md/CHANGELOG note describing it as open) \
         should be updated together"
    );
}
