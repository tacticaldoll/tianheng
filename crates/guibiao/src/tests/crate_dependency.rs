use super::helpers::*;

#[test]
pub(super) fn must_not_be_imported_by_ignores_external_imports() {
    let (result, violations) = run_module_check(
        "inbound-external",
        &[
            ("lib.rs", "pub mod internal;\npub mod http;\n"),
            ("internal.rs", "// protected\n"),
            ("http.rs", "use serde::Deserialize;\n"),
        ],
        protect_internal_from("crate::http"),
    );
    assert!(result.is_ok(), "{result:?}");
    assert!(
        violations.is_empty(),
        "externals are out of scope: {violations:?}"
    );
}

#[test]
pub(super) fn must_not_be_imported_by_crate_forbids_every_outside_importer() {
    let (result, violations) = run_module_check(
        "inbound-x-crate",
        &[
            ("lib.rs", "pub mod internal;\npub mod http;\n"),
            ("internal.rs", "// protected\n"),
            ("http.rs", "use crate::internal::Secret;\n"),
        ],
        protect_internal_from("crate"),
    );
    assert!(result.is_ok(), "{result:?}");
    // Forbidding importer `crate` means nobody outside internal's own subtree may
    // import it; crate::http violates, internal's own files stay clean.
    assert_eq!(violations.len(), 1, "{violations:?}");
    assert_eq!(violations[0].finding, "crate::http");
}

#[test]
pub(super) fn must_not_be_imported_by_on_the_crate_root_is_a_constitution_error() {
    let (result, _violations) = run_module_check(
        "inbound-m-crate",
        &[("lib.rs", "pub mod http;\n"), ("http.rs", "// nothing\n")],
        ModuleBoundary::in_crate("x")
            .module("crate")
            .must_not_be_imported_by("crate::http")
            .because("the crate root cannot be protected this way"),
    );
    let err = result.expect_err("protecting `crate` must be a constitution error");
    assert_eq!(err, must_not_be_imported_by_on_crate_error("x"));
}

#[test]
pub(super) fn must_not_be_imported_by_dedups_multiple_imports_from_one_importer() {
    let (result, violations) = run_module_check(
        "inbound-dedup",
        &[
            ("lib.rs", "pub mod internal;\npub mod http;\n"),
            ("internal.rs", "// protected\n"),
            (
                "http.rs",
                "use crate::internal::A;\nuse crate::internal::B;\n",
            ),
        ],
        protect_internal_from("crate::http"),
    );
    assert!(result.is_ok(), "{result:?}");
    assert_eq!(
        violations.len(),
        1,
        "one offending importer module yields one violation: {violations:?}"
    );
    assert_eq!(violations[0].finding, "crate::http");
}

#[test]
pub(super) fn must_not_be_imported_by_honors_warn_severity() {
    let (result, violations) = run_module_check(
        "inbound-warn",
        &[
            ("lib.rs", "pub mod internal;\npub mod http;\n"),
            ("internal.rs", "// protected\n"),
            ("http.rs", "use crate::internal::Secret;\n"),
        ],
        ModuleBoundary::in_crate("x")
            .module("crate::internal")
            .must_not_be_imported_by("crate::http")
            .warn()
            .because("internal should be private"),
    );
    assert!(result.is_ok(), "{result:?}");
    assert_eq!(violations.len(), 1, "{violations:?}");
    assert_eq!(violations[0].severity, Severity::Warn);
}

#[test]
pub(super) fn must_not_be_imported_by_projects_its_importer() {
    let constitution = Constitution::new("p").boundary(
        ModuleBoundary::in_crate("app")
            .module("crate::internal")
            .must_not_be_imported_by("crate::http")
            .because("internal is private to its layer"),
    );

    let text = constitution_text(&constitution);
    assert!(
        text.contains("must not be imported by crate::http"),
        "{text}"
    );

    let doc: serde_json::Value = serde_json::from_str(&constitution_json(&constitution)).unwrap();
    assert_eq!(doc["format"], "tianheng.constitution/declared-boundaries");
    assert_eq!(
        doc["boundaries"][0]["rule"],
        "module must not be imported by"
    );
    assert_eq!(doc["boundaries"][0]["target"], "crate::internal");
    // The declared forbidden importer projects as `importer`; no `forbidden`/`only`.
    assert_eq!(doc["boundaries"][0]["importer"], "crate::http");
    assert!(doc["boundaries"][0]["forbidden"].is_null());
    assert!(doc["boundaries"][0]["only"].is_null());
}

#[test]
pub(super) fn must_not_be_imported_by_unknown_protected_module_is_a_constitution_error() {
    // The protected-module validation must fire for the inbound rule too: an unknown
    // `m` is exit 2 before any scan, never a silent clean.
    let (result, _violations) = run_module_check(
        "inbound-unknown-m",
        &[("lib.rs", "pub mod http;\n"), ("http.rs", "// nothing\n")],
        ModuleBoundary::in_crate("x")
            .module("crate::nope")
            .must_not_be_imported_by("crate::http")
            .because("typo target"),
    );
    let err = result.expect_err("an unknown protected module is a constitution error");
    assert_eq!(err, unknown_module_error("crate::nope", "x"));
}

#[test]
pub(super) fn must_not_be_imported_by_inline_protected_module_is_a_constitution_error() {
    let (result, _violations) = run_module_check(
        "inbound-inline-m",
        &[
            (
                "lib.rs",
                "pub mod kernel { pub struct K; }\npub mod http;\n",
            ),
            ("http.rs", "// nothing\n"),
        ],
        ModuleBoundary::in_crate("x")
            .module("crate::kernel")
            .must_not_be_imported_by("crate::http")
            .because("inline target"),
    );
    let err = result.expect_err("an inline protected module is a constitution error");
    assert_eq!(
        err,
        inline_module_target_error("crate::kernel", "x", "kernel")
    );
}

#[test]
pub(super) fn must_not_be_imported_by_matches_a_raw_identifier_importer() {
    // The forbidden importer is declared with a raw identifier; the importing file's
    // module canonicalizes to the same path, so the violation still fires (guards the
    // canonicalization lockstep against a false negative).
    let (result, violations) = run_module_check(
        "inbound-rawid-importer",
        &[
            ("lib.rs", "pub mod internal;\npub mod r#async;\n"),
            ("internal.rs", "// protected\n"),
            ("async.rs", "use crate::internal::Secret;\n"),
        ],
        protect_internal_from("crate::r#async"),
    );
    assert!(result.is_ok(), "{result:?}");
    assert_eq!(violations.len(), 1, "{violations:?}");
    assert_eq!(violations[0].finding, "crate::async");
}

#[test]
pub(super) fn must_not_be_imported_by_protects_a_raw_identifier_module() {
    let (result, violations) = run_module_check(
        "inbound-rawid-protected",
        &[
            ("lib.rs", "pub mod r#type;\npub mod http;\n"),
            ("type.rs", "// protected\n"),
            ("http.rs", "use crate::r#type::Thing;\n"),
        ],
        ModuleBoundary::in_crate("x")
            .module("crate::r#type")
            .must_not_be_imported_by("crate::http")
            .because("type is private"),
    );
    assert!(result.is_ok(), "{result:?}");
    assert_eq!(violations.len(), 1, "{violations:?}");
    assert_eq!(violations[0].target(), "crate::type");
    assert_eq!(violations[0].finding, "crate::http");
}

#[test]
pub(super) fn must_not_be_imported_by_flags_a_mod_rs_backed_importer() {
    let (result, violations) = run_module_check(
        "inbound-modrs",
        &[
            ("lib.rs", "pub mod internal;\npub mod http;\n"),
            ("internal.rs", "// protected\n"),
            ("http/mod.rs", "use crate::internal::Secret;\n"),
        ],
        protect_internal_from("crate::http"),
    );
    assert!(result.is_ok(), "{result:?}");
    assert_eq!(violations.len(), 1, "{violations:?}");
    assert_eq!(violations[0].finding, "crate::http");
}

#[test]
pub(super) fn must_not_be_imported_by_orders_multiple_offenders_deterministically() {
    let (result, violations) = run_module_check(
        "inbound-multi",
        &[
            (
                "lib.rs",
                "pub mod internal;\npub mod zeta;\npub mod alpha;\n",
            ),
            ("internal.rs", "// protected\n"),
            ("zeta.rs", "use crate::internal::Secret;\n"),
            ("alpha.rs", "use crate::internal::Secret;\n"),
        ],
        protect_internal_from("crate"),
    );
    assert!(result.is_ok(), "{result:?}");
    let findings: Vec<&str> = violations.iter().map(|v| v.finding.as_str()).collect();
    assert_eq!(
        findings,
        ["crate::alpha", "crate::zeta"],
        "multiple offenders are sorted deterministically"
    );
}

#[test]
pub(super) fn must_not_be_imported_by_dedups_an_importer_backed_by_two_reachable_sources() {
    // One importer module can be backed by two REACHABLE sources, so the same importer would push
    // `crate::inner` twice and the spec's dedup must collapse it to one.
    //
    // The shape is the additive, cfg-blind one `module-boundary` describes: a `mod inner;` and an
    // inline `mod inner { … }` for the same name, which in valid source arises only under
    // mutually-exclusive `#[cfg]` (a same-scope duplicate is a compile error). The scanner does not
    // evaluate `cfg`, so it observes both.
    //
    // This replaces a fixture that claimed a lib+bin package's `lib.rs` and `main.rs` both sit at
    // module `crate`. They do not — each compiled root is its OWN module graph, and each denotes
    // `crate` within its own, which is why every root is governed separately and carries its
    // compilation unit as an identity role (pinned at the real resolution in
    // `tests/per_target_corpus.rs`). That fixture also could not have
    // caught its own premise being wrong: with dedup collapsing the count to one either way, "two
    // sources deduplicated" and "one source scanned" produce the identical assertion — the trap
    // `AGENTS.md` names. So this test asserts the two facts SEPARATELY, and the first of them is what
    // makes it distinguishing.
    //
    // (a) Only the INLINE source imports the protected module; the file source is clean. A run that
    //     did not read the inline body would report ZERO violations here.
    let (inline_only_result, inline_only) = run_module_check(
        "inbound-inline-source-observed",
        &[
            (
                "lib.rs",
                "#[cfg(unix)]\npub mod inner;\n#[cfg(not(unix))]\npub mod inner { use crate::internal::Secret; }\npub mod internal;\n",
            ),
            ("inner.rs", "// clean\n"),
            ("internal.rs", "// protected\n"),
        ],
        protect_internal_from("crate::inner"),
    );
    assert!(inline_only_result.is_ok(), "{inline_only_result:?}");
    assert_eq!(
        inline_only.len(),
        1,
        "the inline source must be observed on its own — zero here would mean it was never read: \
         {inline_only:?}"
    );
    assert_eq!(inline_only[0].finding, "crate::inner");

    // (b) BOTH sources import it: the count must stay one — the dedup guarantee itself.
    let (result, violations) = run_module_check(
        "inbound-two-reachable-sources",
        &[
            (
                "lib.rs",
                "#[cfg(unix)]\npub mod inner;\n#[cfg(not(unix))]\npub mod inner { use crate::internal::Secret; }\npub mod internal;\n",
            ),
            ("inner.rs", "use crate::internal::Secret;\n"),
            ("internal.rs", "// protected\n"),
        ],
        protect_internal_from("crate::inner"),
    );
    assert!(result.is_ok(), "{result:?}");
    assert_eq!(
        violations.len(),
        1,
        "one offending importer module, even when backed by two reachable sources: {violations:?}"
    );
    assert_eq!(violations[0].finding, "crate::inner");
    // The collapsed inbound violation still carries a representative file (the inbound path also
    // collects (key, file) before de-duplication), so the two-source case is locked on the inbound
    // rule, not only the outbound one.
    assert!(
        violations[0].file.is_some(),
        "the surviving inbound violation carries a representative file"
    );
}

#[test]
pub(super) fn must_not_import_dedups_a_finding_across_subtree_files() {
    // `crate::kernel` and `crate::kernel::sub` are two DIFFERENT modules, and each importing the
    // forbidden path is a separate drift event: two violations, distinguished by their importing
    // module.
    //
    // This test previously asserted one, on the stated reason that "the governed module's subtree can
    // span more than one file". That reason describes ONE module backed by two files; it does not cover
    // two distinct modules, and collapsing those meant baselining the first silently masked the second
    // — a real violation accepted without ever being seen. The inbound rules had always qualified by
    // importer; the outbound rules now do too, so the two families are symmetric.
    let (result, violations) = run_module_check(
        "dedup-mni-subtree",
        &[
            ("lib.rs", "pub mod kernel;\n"),
            ("kernel.rs", "pub mod sub;\nuse crate::forbidden::X;\n"),
            ("kernel/sub.rs", "use crate::forbidden::X;\n"),
        ],
        ModuleBoundary::in_crate("x")
            .module("crate::kernel")
            .must_not_import("crate::forbidden")
            .because("kernel must not import forbidden"),
    );
    assert!(result.is_ok(), "{result:?}");
    assert_eq!(
        violations.len(),
        2,
        "one violation per (importing module, import path): {violations:?}"
    );
    let importers: Vec<String> = violations
        .iter()
        .map(|v| {
            v.id()
                .fact()
                .fields()
                .find(|(name, _)| *name == "importer")
                .map(|(_, value)| value.to_string())
                .expect("the outbound fact carries its importing module")
        })
        .collect();
    assert!(
        importers.iter().any(|i| i == "crate::kernel")
            && importers.iter().any(|i| i == "crate::kernel::sub"),
        "the two importing modules must be the discriminator: {importers:?}"
    );
    assert!(
        violations
            .iter()
            .all(|v| v.finding == "crate::forbidden::X")
    );
}

#[test]
pub(super) fn must_not_import_flags_ancestor_glob_hazard() {
    // When a boundary forbids `crate::a::b`, importing an ancestor wildcard (`use crate::a::*;`)
    // brings `crate::a::b` into scope and MUST trigger a Glob Hazard violation (fail-closed).
    let (result, violations) = run_module_check(
        "glob-hazard-ancestor",
        &[
            ("lib.rs", "pub mod app;\npub mod a;\n"),
            ("app.rs", "use crate::a::*;"),
            ("a.rs", "pub mod b;"),
            ("a/b.rs", "pub struct Secret;"),
        ],
        ModuleBoundary::in_crate("x")
            .module("crate::app")
            .must_not_import("crate::a::b")
            .because("app must not import b"),
    );
    assert!(result.is_ok(), "{result:?}");
    assert_eq!(
        violations.len(),
        1,
        "importing an ancestor module wildcard violates a descendant forbidden boundary: {violations:?}"
    );
    assert_eq!(violations[0].finding, "crate::a");
}

#[test]
pub(super) fn must_not_import_allows_plain_ancestor_module_import() {
    // When a boundary forbids `crate::a::b`, importing the parent module without a wildcard
    // (`use crate::a;`) does NOT bring `crate::a::b` into scope and MUST stay clean.
    let (result, violations) = run_module_check(
        "plain-ancestor-import",
        &[
            ("lib.rs", "pub mod app;\npub mod a;\n"),
            ("app.rs", "use crate::a;"),
            ("a.rs", "pub mod b;"),
            ("a/b.rs", "pub struct Secret;"),
        ],
        ModuleBoundary::in_crate("x")
            .module("crate::app")
            .must_not_import("crate::a::b")
            .because("app must not import b"),
    );
    assert!(result.is_ok(), "{result:?}");
    assert!(
        violations.is_empty(),
        "plain non-glob ancestor import must remain clean: {violations:?}"
    );
}

#[test]
pub(super) fn must_not_import_flags_midpath_super_import() {
    // When a grouped use or nested import contains a mid-path `super` (e.g. `use crate::a::b::{super::secret::X}`),
    // it resolves to `crate::a::secret::X` and MUST trigger a violation when `crate::a::secret` is forbidden.
    let (result, violations) = run_module_check(
        "midpath-super-import",
        &[
            ("lib.rs", "pub mod app;\npub mod a;\n"),
            ("app.rs", "use crate::a::b::{super::secret::X};"),
            ("a.rs", "pub mod b;\npub mod secret;\n"),
            ("a/b.rs", "pub struct Dummy;"),
            ("a/secret.rs", "pub struct X;"),
        ],
        ModuleBoundary::in_crate("x")
            .module("crate::app")
            .must_not_import("crate::a::secret")
            .because("app must not import secret"),
    );
    assert!(result.is_ok(), "{result:?}");
    assert_eq!(
        violations.len(),
        1,
        "mid-path super import must be normalized and flagged as a violation: {violations:?}"
    );
    assert_eq!(violations[0].finding, "crate::a::secret::X");
}

#[test]
pub(super) fn inline_symbol_confinement_flags_midpath_super_call() {
    // Both direct calls (crate::a::b::super::secret::helper()) and group-imported alias calls
    // with mid-path super must normalize to crate::a::secret::helper and react when crate::a::secret is forbidden.
    let (result, violations) = run_module_check(
        "symbol-midpath-super",
        &[
            ("lib.rs", "pub mod app;\npub mod a;\n"),
            (
                "app.rs",
                "use crate::a::b::{super::secret::helper as h};\npub fn run() { h(); crate::a::b::super::secret::helper(); }",
            ),
            ("a.rs", "pub mod b;\npub mod secret;\n"),
            ("a/b.rs", "pub fn dummy() {}"),
            ("a/secret.rs", "pub fn helper() {}"),
        ],
        ModuleBoundary::in_crate("x")
            .module("crate::app")
            .must_not_call_inline("crate::a::secret")
            .because("app must not call inline symbol in secret"),
    );
    assert!(result.is_ok(), "{result:?}");
    assert_eq!(
        violations.len(),
        1,
        "inline symbol calls with mid-path super must normalize and react: {violations:?}"
    );
    assert_eq!(
        violations[0].finding,
        "crate::a::secret::helper in crate::app"
    );
}

#[test]
pub(super) fn restrict_imports_to_keeps_two_importing_modules_distinct() {
    // The outbound dual of the inbound rules' long-standing importer qualification: `crate::kernel`
    // and `crate::kernel::sub` are two modules, so each reaching outward is its own drift event. One
    // violation here would mean baselining the first masks the second.
    let (result, violations) = run_module_check(
        "dedup-rit-subtree",
        &[
            ("lib.rs", "pub mod kernel;\n"),
            ("kernel.rs", "pub mod sub;\nuse crate::io::Sink;\n"),
            ("kernel/sub.rs", "use crate::io::Sink;\n"),
        ],
        restrict_kernel_to_types("crate::kernel", &["crate::types"]),
    );
    assert!(result.is_ok(), "{result:?}");
    assert_eq!(
        violations.len(),
        2,
        "one violation per (importing module, import path): {violations:?}"
    );
    assert!(violations.iter().all(|v| v.finding == "crate::io::Sink"));
}

#[test]
pub(super) fn outbound_dedup_collapses_a_repeated_pair_but_keeps_distinct_ones() {
    // The dedup key is the (importing module, import path) PAIR. `crate::kernel` imports X twice — the
    // same pair, so it collapses to one — while `crate::kernel::sub` importing X is a different pair and
    // stays, as does its Y. Result: three violations over two importers, not one per path.
    let (result, violations) = run_module_check(
        "dedup-distinct",
        &[
            ("lib.rs", "pub mod kernel;\n"),
            (
                "kernel.rs",
                "pub mod sub;\nuse crate::forbidden::X;\nuse crate::forbidden::X as Dup;\n",
            ),
            (
                "kernel/sub.rs",
                "use crate::forbidden::X;\nuse crate::forbidden::Y;\n",
            ),
        ],
        ModuleBoundary::in_crate("x")
            .module("crate::kernel")
            .must_not_import("crate::forbidden")
            .because("kernel must not import forbidden"),
    );
    assert!(result.is_ok(), "{result:?}");
    let findings: Vec<&str> = violations.iter().map(|v| v.finding.as_str()).collect();
    assert_eq!(
        findings,
        [
            "crate::forbidden::X",
            "crate::forbidden::X",
            "crate::forbidden::Y"
        ],
        "the repeated pair collapsed; the other importer's X and Y stayed: {violations:?}"
    );
    // And no two violations share an identity. Compared on the REAL `ViolationId`, not on
    // `(target, rule, finding)`: two of these legitimately share all three and differ only in the
    // fact's importing module, so the old proxy would now report a collision that does not exist —
    // and, worse, would have passed while a real collapse was happening.
    let mut ids: Vec<String> = violations.iter().map(|v| format!("{:?}", v.id())).collect();
    let before = ids.len();
    ids.sort();
    ids.dedup();
    assert_eq!(ids.len(), before, "no duplicate violation identities");
}

#[test]
pub(super) fn restrict_imports_to_does_not_flag_an_over_popped_super() {
    // `crate::a` over-pops with `super::super`; the path names no internal module, so
    // it must not be observed — and must not be mistaken for an outward edge that the
    // allowlist would flag (the regression this guards).
    let (result, violations) = run_module_check(
        "restrict-super-overflow",
        &[
            ("lib.rs", "pub mod a;\n"),
            ("a.rs", "use super::super::other::X;\n"),
        ],
        restrict_kernel_to_types("crate::a", &["crate::types"]),
    );
    assert!(result.is_ok(), "{result:?}");
    assert!(
        violations.is_empty(),
        "an over-popped super is not an outward edge: {violations:?}"
    );
}

#[test]
pub(super) fn baseline_round_trips_through_json() {
    let report = one_enforce_violation();
    let json = Baseline::of(&report).to_json();
    let parsed = Baseline::from_json(&json).expect("a written baseline parses");
    assert!(
        parsed.contains(&report.violations[0]),
        "round-trip must preserve the violation identity"
    );
}

#[test]
pub(super) fn from_json_rejects_malformed_and_unknown_version() {
    assert!(Baseline::from_json("not json").is_err());
    assert!(Baseline::from_json(r#"{"version":3,"violations":[]}"#).is_err());
    assert!(
        Baseline::from_json(r#"{"violations":[]}"#).is_err(),
        "a missing version must be an error, not a silent empty baseline"
    );
}

#[test]
pub(super) fn a_baselined_enforce_violation_does_not_fail() {
    let mut report = one_enforce_violation();
    let baseline = Baseline::of(&report);
    apply_baseline(&mut report, &baseline);
    assert!(report.violations[0].baselined);
    assert_eq!(
        Outcome::Violations(report).exit_code(),
        0,
        "a fully baselined run must not fail"
    );
}

#[test]
pub(super) fn a_new_enforce_violation_fails_against_a_baseline() {
    let baseline = Baseline::of(&Report::new(vec![Violation::new(
        BoundaryKind::Crate,
        test_id("core", "deny external dependencies", "other"),
        "deny external dependencies",
        "other",
        "core must stay dependency-light".to_string(),
        Severity::Enforce,
    )]));
    let mut report = one_enforce_violation();
    apply_baseline(&mut report, &baseline);
    assert!(
        !report.violations[0].baselined,
        "serde is not in the baseline"
    );
    assert_eq!(Outcome::Violations(report).exit_code(), 1);
}

#[test]
pub(super) fn stale_finds_entries_with_no_current_match() {
    let report = one_enforce_violation();
    let baseline = Baseline::of(&Report::new(vec![Violation::new(
        BoundaryKind::Crate,
        test_id("core", "deny external dependencies", "gone"),
        "deny external dependencies",
        "gone",
        "core must stay dependency-light".to_string(),
        Severity::Enforce,
    )]));
    let stale = baseline.stale(&report);
    assert_eq!(stale.len(), 1);
    assert_eq!(stale[0].finding, "gone");
}

#[test]
pub(super) fn report_json_projects_a_violation_with_its_kind() {
    let json = report_json(&Outcome::Violations(one_enforce_violation()), &[], None);
    let doc: serde_json::Value = serde_json::from_str(&json).expect("valid JSON");
    assert_eq!(doc["outcome"], "violations");
    assert_eq!(doc["exit_code"], 1);
    let violation = &doc["violations"][0];
    assert_eq!(doc["format"], "tianheng.reaction/structured-facts");
    assert_eq!(violation["kind"], "crate");
    assert_eq!(violation["finding"], "serde");
    assert_eq!(
        violation["fact"]["type"],
        "tianheng.fact/guibiao/dependency"
    );
    assert_eq!(violation["fact"]["shape"], "dependency-edge");
    assert_eq!(violation["fact"]["fields"]["package"], "serde");
    assert!(violation["rule_key"].is_object());
    assert_eq!(violation["severity"], "enforce");
    assert_eq!(violation["baselined"], false);
    // `reason` is the repair hint; there is no separate field.
    assert!(violation["reason"].as_str().is_some_and(|r| !r.is_empty()));
    assert!(doc.get("repair_hint").is_none());
}

#[test]
pub(super) fn report_json_renders_clean_and_constitution_error() {
    let clean: serde_json::Value = serde_json::from_str(&report_json(
        &Outcome::Clean(Subject::nothing_declared()),
        &[],
        None,
    ))
    .unwrap();
    assert_eq!(clean["outcome"], "clean");
    assert_eq!(clean["exit_code"], 0);
    assert_eq!(clean["violations"].as_array().unwrap().len(), 0);
    assert!(clean.get("coverage").is_none(), "no coverage when None");

    let error: serde_json::Value = serde_json::from_str(&report_json(
        &Outcome::ConstitutionError("boom".into()),
        &[],
        None,
    ))
    .unwrap();
    assert_eq!(error["outcome"], "constitution_error");
    assert_eq!(error["exit_code"], 2);
    assert_eq!(error["error"], "boom");
}

#[test]
pub(super) fn report_json_reflects_baseline_and_stale_in_gate() {
    let mut report = one_enforce_violation();
    let baseline = Baseline::of(&report);
    apply_baseline(&mut report, &baseline);
    // A baseline entry that no current violation matches is stale.
    let stale_baseline = Baseline::from_json(
        r#"{"format":"tianheng.baseline/structured-facts","violations":[{
            "target":"core","rule":"deny external dependencies","finding":"gone",
            "rule_key":{"type":"tianheng.rule/test/policy","fields":{"policy":"deny external dependencies"}},
            "fact":{"type":"tianheng.fact/guibiao/dependency","shape":"declared-dependency","fields":{"kind":"normal","package":"gone"}}
        }]}"#,
    )
    .unwrap();
    let stale: Vec<BaselineEntry> = stale_baseline
        .stale(&Report::empty())
        .into_iter()
        .cloned()
        .collect();
    let doc: serde_json::Value =
        serde_json::from_str(&report_json(&Outcome::Violations(report), &stale, None)).unwrap();
    assert_eq!(doc["exit_code"], 0, "a fully baselined run does not fail");
    assert_eq!(doc["violations"][0]["baselined"], true);
    assert_eq!(doc["stale_baseline"][0]["finding"], "gone");
    assert!(doc["stale_baseline"][0]["fact"].is_object());
}

#[test]
pub(super) fn stale_policy_is_one_pure_exit_code_source_for_runner_and_projection() {
    let baseline = Baseline::from_json(
        r#"{"format":"tianheng.baseline/structured-facts","violations":[{
            "target":"core","rule":"old rule","finding":"gone",
            "rule_key":{"type":"tianheng.rule/test/old","fields":{}},
            "fact":{"type":"tianheng.fact/test/old","shape":"gone","fields":{}}
        }]}"#,
    )
    .unwrap();
    let stale: Vec<BaselineEntry> = baseline.entries().cloned().collect();

    assert_eq!(
        stale_policy(&Outcome::Clean(Subject::nothing_declared()), &stale, true),
        StalePolicy {
            stale_disallowed: true,
            exit_code: 1,
        }
    );
    assert_eq!(
        stale_policy(
            &Outcome::ConstitutionError("cannot judge".into()),
            &stale,
            true
        )
        .exit_code,
        2,
        "stale policy never masks a constitution error"
    );
    assert_eq!(
        stale_policy(&Outcome::Clean(Subject::nothing_declared()), &stale, false).exit_code,
        0
    );
}

#[test]
pub(super) fn report_json_includes_coverage_when_present() {
    let coverage = Coverage {
        total: 3,
        uncovered: vec!["memory".to_string()],
    };
    let doc: serde_json::Value = serde_json::from_str(&report_json(
        &Outcome::Clean(Subject::nothing_declared()),
        &[],
        Some(&coverage),
    ))
    .unwrap();
    assert_eq!(doc["coverage"]["workspace_crates"], 3);
    assert_eq!(doc["coverage"]["uncovered"][0], "memory");
}

#[test]
pub(super) fn external_classification_treats_any_non_null_source_as_external() {
    // A path/internal dep has a null `source`; registry, git, and alternative
    // (sparse) registry deps all have a non-null source and must be classified
    // external. The sparse case is the regression guard: a fixed `registry+`/
    // `git+` prefix list would silently pass an alternative `sparse+` registry.
    let package = serde_json::json!({
        "dependencies": [
            { "name": "internal", "source": null, "kind": null },
            {
                "name": "crates_io",
                "source": "registry+https://github.com/rust-lang/crates.io-index",
                "kind": null
            },
            { "name": "git_dep", "source": "git+https://example.com/x", "kind": null },
            { "name": "alt_sparse", "source": "sparse+https://my.registry/index/", "kind": null },
            {
                "name": "a_dev",
                "source": "registry+https://github.com/rust-lang/crates.io-index",
                "kind": "dev"
            },
        ]
    });
    assert_eq!(
        external_dependencies(&package, DependencyKind::Normal),
        vec![
            "alt_sparse".to_string(),
            "crates_io".to_string(),
            "git_dep".to_string(),
        ],
        "every non-null-source normal dep is external (incl. a sparse alt \
             registry); the null-source internal dep and the dev dep are excluded",
    );
}

#[test]
pub(super) fn a_crate_violation_reports_no_file() {
    // A crate-dependency violation is an edge in the dependency graph (a manifest
    // relation), not a source line, so its `file` is a faithful `None`.
    let metadata = serde_json::json!({
        "packages": [{
            "name": "core",
            "dependencies": [
                {
                    "name": "serde",
                    "source": "registry+https://github.com/rust-lang/crates.io-index",
                    "kind": null
                }
            ],
        }]
    });
    let boundary = CrateBoundary::crate_("core")
        .deny_external_dependencies()
        .because("core stays dependency-light");
    let mut violations = Vec::new();
    let result = check_crate_boundary(&metadata, &[], &boundary, &mut violations);
    assert!(result.is_ok(), "{result:?}");
    assert_eq!(violations.len(), 1, "{violations:?}");
    assert_eq!(violations[0].kind, BoundaryKind::Crate);
    assert_eq!(violations[0].finding, "serde");
    assert!(
        violations[0].file.is_none(),
        "a crate-dependency violation has no single source file"
    );
}

#[test]
pub(super) fn dedup_keeps_the_more_severe_of_duplicate_violations() {
    // The same crate rule declared once `warn` and once `enforce` on one crate — a plausible
    // mid-promotion state — flags the same dependency twice with equal `(target, rule, finding)`
    // identity but different severity. Deduping must keep the ENFORCE reaction: keeping the
    // first-seen `warn` would collapse to an advisory and drop exit-1 to exit-0 (a false
    // negative). Verified in both declaration orders (the fix is order-independent).
    let metadata = serde_json::json!({
        "packages": [{
            "name": "core",
            "dependencies": [
                { "name": "serde", "source": "registry+x", "kind": null }
            ],
        }]
    });
    let warn = || {
        CrateBoundary::crate_("core")
            .deny_external_dependencies()
            .warn()
            .because("observing before enforcing")
    };
    let enforce = || {
        CrateBoundary::crate_("core")
            .deny_external_dependencies()
            .because("core stays dependency-light")
    };
    for (first, second) in [(warn(), enforce()), (enforce(), warn())] {
        let constitution = Constitution::new("mid-promotion")
            .boundary(first)
            .boundary(second);
        let outcome = evaluate(&constitution, &metadata);
        let Outcome::Violations(report) = &outcome else {
            panic!("expected violations, got {outcome:?}");
        };
        assert_eq!(
            report.violations.len(),
            1,
            "duplicates collapse to one: {report:?}"
        );
        assert_eq!(
            report.violations[0].severity,
            Severity::Enforce,
            "the more severe reaction is kept"
        );
        assert_eq!(
            outcome.exit_code(),
            1,
            "an enforce violation fails the reaction"
        );
    }
}

#[test]
pub(super) fn dependency_kind_selects_which_table_is_observed() {
    // `serde` is a normal dep; `proptest` is a dev-dep; `cc` is a build-dep.
    let package = serde_json::json!({
        "dependencies": [
            { "name": "serde", "source": "registry+x", "kind": null },
            { "name": "proptest", "source": "registry+x", "kind": "dev" },
            { "name": "cc", "source": "registry+x", "kind": "build" },
        ]
    });
    let deny = Rule::DenyExternalDependencies { allowed: vec![] };
    // Default (normal) sees only serde; dev sees only proptest; build only cc. The dev/build
    // findings carry a kind suffix so the same dep name in two tables stays a distinct finding.
    assert_eq!(
        deny.findings(&package, &[], DependencyKind::Normal),
        vec!["serde".to_string()]
    );
    assert_eq!(
        deny.findings(&package, &[], DependencyKind::Dev),
        vec!["proptest (dev)".to_string()]
    );
    assert_eq!(
        deny.findings(&package, &[], DependencyKind::Build),
        vec!["cc (build)".to_string()]
    );
}

#[test]
pub(super) fn the_same_dep_in_two_tables_yields_distinct_findings() {
    // The one forbidden bug for the dependency family: `serde` from a git source in BOTH the
    // normal and the dev table, governed by same-rule boundaries differing only by kind, must
    // not collapse to one `(target, rule, finding)` — else baselining the normal violation
    // masks a new dev one. The kind suffix keeps them distinct.
    let package = serde_json::json!({
        "dependencies": [
            { "name": "serde", "source": "git+https://x", "kind": null },
            { "name": "serde", "source": "git+https://x", "kind": "dev" },
        ]
    });
    let rule = Rule::RestrictDependencySourcesTo {
        allowed: vec![SourceKind::Registry],
    };
    let normal = rule.findings(&package, &[], DependencyKind::Normal);
    let dev = rule.findings(&package, &[], DependencyKind::Dev);
    assert_eq!(normal, vec!["serde".to_string()]);
    assert_eq!(dev, vec!["serde (dev)".to_string()]);
    assert_ne!(normal, dev, "same dep in two tables must not collide");
}

#[test]
pub(super) fn workspace_member_names_are_the_no_deps_packages() {
    // With `--no-deps`, `packages` is exactly the workspace members.
    let metadata = serde_json::json!({
        "packages": [ { "name": "core" }, { "name": "adapters" } ]
    });
    assert_eq!(
        workspace_member_names(&metadata),
        Members::Read(vec!["adapters".to_string(), "core".to_string()]),
    );
}

/// An unreadable membership is not an empty one, in either of the two ways it can be unreadable.
///
/// **Both consumers read empty as *nothing to govern*.** Coverage computed `total = 0` with an empty
/// uncovered list and rendered it as complete coverage over a membership it never read; the evaluation
/// refused, but with the sentence for a workspace that genuinely declares no member — the wrong fact
/// about the wrong thing. This crate already states the rule on `workspace_member_src_dirs`: *an
/// unreadable workspace is a constitution error, never a silent empty set*.
#[test]
pub(super) fn an_unreadable_membership_is_not_an_empty_one() {
    // No `packages` array at all.
    let absent = serde_json::json!({ "workspace_root": "/w" });
    let Members::Unreadable(why) = workspace_member_names(&absent) else {
        panic!("metadata carrying no `packages` array cannot be read as a membership");
    };
    assert!(why.contains("no `packages` array"), "got: {why}");

    // A package this reader cannot name: dropping it would shrink the set the workspace rule compares
    // against, so every unlisted member of a partly-unreadable set would read as governed.
    let unnamed = serde_json::json!({
        "packages": [ { "name": "core" }, { "version": "0.1.0" } ]
    });
    let Members::Unreadable(why) = workspace_member_names(&unnamed) else {
        panic!("a package whose name cannot be read leaves the membership incomplete");
    };
    assert!(why.contains("`name` is absent"), "got: {why}");

    // And a workspace that genuinely declares none is read, not refused — the third state stays its own.
    assert_eq!(
        workspace_member_names(&serde_json::json!({ "packages": [] })),
        Members::Read(vec![])
    );
}

/// The two consumers of an unreadable membership, exercised through the consumers.
///
/// **The reader refusing is not the same fact as the consumers honouring the refusal.** The direction
/// above holds `workspace_member_names`, which an `enum` already forces every caller to match on — but
/// matching is not choosing the right arm, and a later edit mapping `Unreadable` to an empty membership
/// or to a fabricated coverage would leave it green. So this one calls what the callers call: `evaluate`
/// for the outcome and `coverage_of` for the advisory, over metadata neither can be handed through
/// `check_and_cover`, whose only entry point spawns `cargo metadata` and therefore cannot be given a
/// membership to fail on.
#[test]
pub(super) fn both_consumers_of_an_unreadable_membership_refuse() {
    let constitution = Constitution::new("w").boundary(
        CrateBoundary::crate_("core")
            .restrict_dependencies_to(["serde_json"])
            .because("a boundary is needed for coverage to have a denominator"),
    );

    for (label, metadata) in [
        (
            "no `packages` array",
            serde_json::json!({ "workspace_root": "/w" }),
        ),
        (
            "a package this reader cannot name",
            serde_json::json!({ "packages": [ { "name": "core" }, { "version": "0.1.0" } ] }),
        ),
    ] {
        match crate::evaluate(&constitution, &metadata) {
            Outcome::ConstitutionError(why) => assert!(
                why.contains("membership") || why.contains("workspace members"),
                "{label}: the error must name what could not be read, got: {why}"
            ),
            other => panic!("{label}: an unreadable membership must refuse, got {other:?}"),
        }
        assert!(
            crate::coverage_of(&metadata, &constitution).is_none(),
            "{label}: coverage over a membership that was never read is coverage over nothing"
        );
    }

    // The control: a membership that IS readable reaches both consumers, so the assertions above are
    // about the unreadable state rather than about a constitution that refuses everything.
    let read = serde_json::json!({ "packages": [ { "name": "core" } ] });
    assert!(
        !matches!(
            crate::evaluate(&constitution, &read),
            Outcome::ConstitutionError(_)
        ),
        "a readable membership must not refuse"
    );
    assert!(
        crate::coverage_of(&read, &constitution).is_some(),
        "a readable membership must produce coverage"
    );
}

#[test]
pub(super) fn workspace_rule_flags_only_unlisted_workspace_members() {
    // Deps: two workspace members (core, adapters), one external (serde), and one
    // path dependency that is NOT a workspace member (outside).
    let package = serde_json::json!({
        "dependencies": [
            { "name": "core", "source": null, "kind": null },
            { "name": "adapters", "source": null, "kind": null },
            {
                "name": "serde",
                "source": "registry+https://github.com/rust-lang/crates.io-index",
                "kind": null
            },
            { "name": "outside", "source": null, "kind": null },
        ]
    });
    let workspace = vec!["core".to_string(), "adapters".to_string()];

    // Restrict to [core]: adapters is an unlisted workspace member → flagged;
    // serde (external) and outside (path, non-member) are ignored.
    let restrict = Rule::RestrictWorkspaceDependenciesTo {
        allowed: vec!["core".to_string()],
    };
    assert_eq!(
        restrict.findings(&package, &workspace, DependencyKind::Normal),
        vec!["adapters".to_string()],
    );

    // Empty allowlist forbids every workspace member, still ignoring external and
    // the non-member path dependency.
    let forbid_all = Rule::RestrictWorkspaceDependenciesTo { allowed: vec![] };
    assert_eq!(
        forbid_all.findings(&package, &workspace, DependencyKind::Normal),
        vec!["adapters".to_string(), "core".to_string()],
    );
}

#[test]
pub(super) fn workspace_rule_never_flags_a_crates_own_self_referential_dev_dependency() {
    // Round-11 finding: Cargo genuinely permits (and real projects use, e.g. a doctest/
    // dogfooding pattern) a crate declaring itself as a `[dev-dependencies]` path dependency
    // on itself (`main = { path = "." }`), and `cargo metadata --no-deps` emits this edge
    // verbatim (verified against real cargo). `workspace_member_names` trivially includes the
    // crate's own name, and `dependencies()` matches by bare package name with no
    // self-exclusion — so before this fix, a `forbid_all_workspace_dependencies` /
    // `restrict_workspace_dependencies_to` boundary declared on that crate flagged its own
    // legitimate self-dev-dependency as an "unlisted workspace dependency", even though a
    // self-dependency can never be an inter-crate layering violation (there is no OTHER crate
    // to leak across a boundary to).
    let package = serde_json::json!({
        "name": "main",
        "dependencies": [
            { "name": "main", "source": null, "kind": "dev" },
        ]
    });
    let workspace = vec!["main".to_string()];
    let forbid_all = Rule::RestrictWorkspaceDependenciesTo { allowed: vec![] };
    assert_eq!(
        forbid_all.findings(&package, &workspace, DependencyKind::Dev),
        Vec::<String>::new(),
        "a crate's own self-referential dev-dependency must never be flagged as an unlisted \
         workspace dependency"
    );
}

#[test]
pub(super) fn no_dependency_rule_ever_flags_a_crates_own_self_referential_dependency() {
    // A crate's self-referential dependency (Cargo's legal `[dev-dependencies] main = { path = "." }`
    // doctest/dogfooding pattern) is never a cross-crate concern. The exclusion is at the shared
    // observation source (`cargo_metadata.rs::is_self_dependency`), so every dependency rule
    // (`ForbidDependencyOn`, `RestrictDependenciesTo`, `RestrictDependencySourcesTo`, etc.) is
    // protected against false positives consistently.
    let package = serde_json::json!({
        "name": "main",
        "dependencies": [
            { "name": "main", "source": null, "kind": "dev", "features": ["x"] },
        ]
    });
    let workspace = vec!["main".to_string()];

    assert_eq!(
        Rule::ForbidDependencyOn {
            crates: vec!["main".to_string()]
        }
        .findings(&package, &workspace, DependencyKind::Dev),
        Vec::<String>::new(),
        "ForbidDependencyOn must not flag the crate's own self-dependency"
    );
    assert_eq!(
        Rule::RestrictDependenciesTo {
            allowed: vec!["serde".to_string()]
        }
        .findings(&package, &workspace, DependencyKind::Dev),
        Vec::<String>::new(),
        "RestrictDependenciesTo must not flag the crate's own self-dependency"
    );
    assert_eq!(
        Rule::RestrictDependencySourcesTo {
            allowed: vec![SourceKind::Registry]
        }
        .findings(&package, &workspace, DependencyKind::Dev),
        Vec::<String>::new(),
        "RestrictDependencySourcesTo must not flag the crate's own self-dependency's Path source"
    );
    assert_eq!(
        Rule::RestrictFeaturesOf {
            crate_: "main".to_string(),
            allowed: vec![]
        }
        .findings(&package, &workspace, DependencyKind::Dev),
        Vec::<String>::new(),
        "RestrictFeaturesOf must not observe the crate's own self-dependency's declared features"
    );
    assert_eq!(
        Rule::ForbidFeaturesOf {
            crate_: "main".to_string(),
            forbidden: vec!["x".to_string()]
        }
        .findings(&package, &workspace, DependencyKind::Dev),
        Vec::<String>::new(),
        "ForbidFeaturesOf must not observe the crate's own self-dependency's declared features"
    );
}

#[test]
pub(super) fn every_crate_rule_still_flags_a_same_named_but_externally_sourced_dependency() {
    // `is_self_dependency` once matched by NAME ALONE, so a package `foo` depending on a *different*,
    // externally-sourced package that merely shares its own name (a real wrapper/fork/
    // self-comparison pattern — verified against real cargo: `foo = { git = "…" }` reads
    // `{"name":"foo","source":"git+…"}`, no error) was wrongly swallowed by the identical
    // self-dependency exemption meant only for the genuine null-source path idiom
    // (`main = { path = "." }`). Every rule sharing the `dependencies()` /
    // `dependencies_with_disallowed_source()` observation must react to this edge exactly as it
    // would to any other same-shaped external dependency, not silently exempt it.
    let package = serde_json::json!({
        "name": "foo",
        "dependencies": [
            {
                "name": "foo",
                "source": "git+https://example.invalid/foo.git",
                "kind": null,
                "features": ["x"]
            },
        ]
    });
    let workspace = vec!["foo".to_string()];

    assert_eq!(
        Rule::ForbidDependencyOn {
            crates: vec!["foo".to_string()]
        }
        .findings(&package, &workspace, DependencyKind::Normal),
        vec!["foo".to_string()],
        "ForbidDependencyOn must flag a same-named externally-sourced dependency"
    );
    assert_eq!(
        Rule::RestrictDependenciesTo {
            allowed: Vec::<String>::new()
        }
        .findings(&package, &workspace, DependencyKind::Normal),
        vec!["foo".to_string()],
        "RestrictDependenciesTo([]) must flag a same-named externally-sourced dependency"
    );
    assert_eq!(
        Rule::RestrictDependencySourcesTo {
            allowed: vec![SourceKind::Registry, SourceKind::Path]
        }
        .findings(&package, &workspace, DependencyKind::Normal),
        vec!["foo".to_string()],
        "RestrictDependencySourcesTo must flag the same-named dependency's disallowed Git source"
    );
    assert_eq!(
        Rule::RestrictWorkspaceDependenciesTo {
            allowed: Vec::<String>::new()
        }
        .findings(&package, &workspace, DependencyKind::Normal),
        vec!["foo".to_string()],
        "RestrictWorkspaceDependenciesTo shares the identical dependencies() observation, so it \
         must flag it too, exactly as it would flag any other external dependency whose name \
         happens to match a workspace member's name (here, the target's own)"
    );
    assert_eq!(
        Rule::RestrictFeaturesOf {
            crate_: "foo".to_string(),
            allowed: vec![]
        }
        .findings(&package, &workspace, DependencyKind::Normal),
        vec!["foo/default".to_string(), "foo/x".to_string()],
        "RestrictFeaturesOf must observe the same-named externally-sourced dependency's features \
         (including the implicit default-features request, since uses_default_features is absent)"
    );
}

#[test]
pub(super) fn coverage_counts_a_module_only_covered_crate_as_covered() {
    let members = vec!["app".to_string(), "core".to_string(), "memory".to_string()];
    let constitution = Constitution::new("c")
        .boundary(
            CrateBoundary::crate_("core")
                .forbid_all_workspace_dependencies()
                .because("core is independent"),
        )
        .boundary(
            ModuleBoundary::in_crate("app")
                .module("crate::kernel")
                .must_not_import("crate::projection")
                .because("layering"),
        );
    let coverage = coverage_from(members, &constitution);
    assert_eq!(coverage.total, 3);
    // `app` is covered by the module boundary, `core` by the crate boundary;
    // only `memory` has no boundary at all.
    assert_eq!(coverage.uncovered, vec!["memory".to_string()]);
}

pub(super) fn mixed_constitution() -> Constitution {
    Constitution::new("my-project")
        .boundary(
            CrateBoundary::crate_("my-core")
                .deny_external_dependencies()
                .allow_external(["serde"])
                .because("my-core must stay dependency-light"),
        )
        .boundary(
            CrateBoundary::crate_("my-core")
                .forbid_dependency_on(["my-adapters"])
                .because("the core must not depend on adapters"),
        )
        .boundary(
            ModuleBoundary::in_crate("my-app")
                .module("crate::domain")
                .must_not_import("crate::http")
                .warn()
                .because("the domain must not import the HTTP layer"),
        )
}

#[test]
pub(super) fn constitution_text_projects_every_boundary_with_its_parameters() {
    let text = constitution_text(&mixed_constitution());
    assert!(
        text.contains("Constitution: my-project  (3 boundaries)"),
        "{text}"
    );
    assert!(text.contains("crate my-core"), "{text}");
    assert!(
        text.contains("deny external dependencies (allow: serde)"),
        "{text}"
    );
    assert!(text.contains("forbid dependency on: my-adapters"), "{text}");
    assert!(text.contains("module crate::domain in my-app"), "{text}");
    assert!(text.contains("must not import crate::http"), "{text}");
    // Severity and reason both surface.
    assert!(
        text.contains("[warn]") && text.contains("[enforce]"),
        "{text}"
    );
    assert!(
        text.contains("the domain must not import the HTTP layer"),
        "{text}"
    );
}

#[test]
pub(super) fn constitution_json_projects_boundaries_with_kinds_and_parameters() {
    let json = constitution_json(&mixed_constitution());
    let doc: serde_json::Value = serde_json::from_str(&json).expect("valid JSON");
    assert_eq!(doc["constitution"], "my-project");
    let boundaries = doc["boundaries"].as_array().expect("array");
    assert_eq!(boundaries.len(), 3);

    // Crate boundary with an allowlist.
    assert_eq!(boundaries[0]["kind"], "crate");
    assert_eq!(boundaries[0]["target"], "my-core");
    assert_eq!(boundaries[0]["rule"], "deny external dependencies");
    assert_eq!(boundaries[0]["severity"], "enforce");
    assert_eq!(boundaries[0]["allowed"][0], "serde");

    // Forbid-dependency-on carries its crate list.
    assert_eq!(boundaries[1]["rule"], "forbid dependency on");
    assert_eq!(boundaries[1]["crates"][0], "my-adapters");

    // Module boundary: target is the module path (report convention), plus crate
    // and forbidden import.
    assert_eq!(boundaries[2]["kind"], "module");
    assert_eq!(boundaries[2]["target"], "crate::domain");
    assert_eq!(boundaries[2]["crate"], "my-app");
    assert_eq!(boundaries[2]["forbidden"], "crate::http");
    assert_eq!(boundaries[2]["severity"], "warn");
}

#[test]
pub(super) fn an_empty_constitution_projects_cleanly() {
    let constitution = Constitution::new("fresh");
    let text = constitution_text(&constitution);
    assert!(
        text.contains("Constitution: fresh  (0 boundaries)"),
        "{text}"
    );
    let doc: serde_json::Value = serde_json::from_str(&constitution_json(&constitution)).unwrap();
    assert_eq!(doc["boundaries"].as_array().unwrap().len(), 0);
}

#[test]
pub(super) fn restrict_to_projects_its_allowlist() {
    let constitution = Constitution::new("p")
        .boundary(
            CrateBoundary::crate_("a")
                .restrict_dependencies_to(["serde", "types"])
                .because("a may depend on only serde and types"),
        )
        .boundary(
            CrateBoundary::crate_("b")
                .restrict_dependencies_to::<[&str; 0], &str>([])
                .because("b must depend on nothing"),
        );

    let text = constitution_text(&constitution);
    assert!(
        text.contains("restrict dependencies to: serde, types"),
        "{text}"
    );
    assert!(text.contains("restrict dependencies to nothing"), "{text}");

    let doc: serde_json::Value = serde_json::from_str(&constitution_json(&constitution)).unwrap();
    assert_eq!(doc["boundaries"][0]["rule"], "restrict dependencies to");
    // A distinct key (`only`, not deny-external's `allowed`) for the closed set.
    assert_eq!(doc["boundaries"][0]["only"][0], "serde");
    assert!(doc["boundaries"][0]["allowed"].is_null());
    // The empty allowlist is still emitted, as `[]`.
    assert_eq!(doc["boundaries"][1]["only"].as_array().unwrap().len(), 0);
}

// A synthesized `cargo metadata --no-deps` package mirroring the source-kind probe:
// a registry dep, a path dep, a plain git dep, an optional git dep, a renamed git dep
// (real name `serde`, local alias `mydep`), a `{ git, version }` dep, an inherited
// workspace git dep (cargo flattens the source into the member as `git+…`), and a git
// dev-dependency. Every `source` string is exactly what cargo emits (verified).
pub(super) fn source_package() -> Value {
    serde_json::json!({
        "dependencies": [
            {
                "name": "crates_io",
                "source": "registry+https://github.com/rust-lang/crates.io-index",
                "kind": null
            },
            { "name": "localdep", "source": null, "kind": null },
            { "name": "gitdep", "source": "git+https://example.invalid/a.git", "kind": null },
            {
                "name": "optgit",
                "source": "git+https://example.invalid/b.git",
                "kind": null,
                "optional": true
            },
            {
                "name": "serde",
                "rename": "mydep",
                "source": "git+https://example.invalid/c.git",
                "kind": null
            },
            { "name": "gitver", "source": "git+https://example.invalid/d.git", "kind": null },
            {
                "name": "inherited",
                "source": "git+https://example.invalid/e.git",
                "kind": null
            },
            { "name": "devgit", "source": "git+https://example.invalid/f.git", "kind": "dev" },
        ]
    })
}

#[test]
pub(super) fn source_rule_flags_every_git_source_outside_a_registry_or_path_allowlist() {
    let package = source_package();
    // Permit [Registry, Path]: every git-sourced normal dep is flagged — the plain
    // git dep, the OPTIONAL git dep (declared regardless of feature state), the
    // `{ git, version }` dep (a stated hygiene bound — it would publish, yet its
    // declared source is git), and the INHERITED workspace git dep (cargo flattens
    // the git source into the member). A RENAMED git dep is reported by its REAL
    // package name `serde`, not its alias `mydep`. The registry and path deps pass;
    // the git DEV-dep is not in the Normal-scoped surface.
    let rule = Rule::RestrictDependencySourcesTo {
        allowed: vec![SourceKind::Registry, SourceKind::Path],
    };
    assert_eq!(
        rule.findings(&package, &[], DependencyKind::Normal),
        vec![
            "gitdep".to_string(),
            "gitver".to_string(),
            "inherited".to_string(),
            "optgit".to_string(),
            "serde".to_string(),
        ],
        "every declared git source is flagged (optional/version/inherited included), \
             by real package name, while registry+path pass and the dev git dep is unscoped",
    );
}

/// `crate-dependency-boundary/an-optional-dependency-edge-is-observed-as-a-declared-one-a-stated-bound`
///
/// `UnderReacts`, owned by the engine. `RestrictDependenciesTo` reads the **declared** dependency set, and
/// cargo reports an `optional = true` edge in that set like any other. So a crate whose edge exists only
/// under a feature is governed as though the edge were unconditional, and a boundary cannot express
/// *depends on this only when that feature is on*.
///
/// Both directions on one package, differing only in the flag: the ordinary edge and the optional one are
/// reported identically, which is what makes the bound a bound rather than an oversight. The sibling
/// source-rule direction states the same fact for source kinds; this states it for the dependency set,
/// which is the rule an adopter reaches for first.
#[test]
pub(super) fn an_optional_dependency_edge_is_observed_as_a_declared_one() {
    let package = serde_json::json!({
        "dependencies": [
            { "name": "always", "source": null, "kind": null },
            { "name": "gated", "source": null, "kind": null, "optional": true },
        ]
    });
    let rule = Rule::RestrictDependenciesTo {
        allowed: vec!["always".to_string()],
    };
    assert_eq!(
        rule.findings(&package, &[], DependencyKind::Normal),
        vec!["gated".to_string()],
        "an optional edge is in the declared set, so it is governed exactly as an unconditional one — the \
         reader has no way to tell them apart, which is the declared bound"
    );

    let permissive = Rule::RestrictDependenciesTo {
        allowed: vec!["always".to_string(), "gated".to_string()],
    };
    assert!(
        permissive
            .findings(&package, &[], DependencyKind::Normal)
            .is_empty(),
        "and naming it in the allowlist clears it, whether or not the feature enabling it is ever on"
    );
}

#[test]
pub(super) fn source_rule_registry_only_flags_a_path_dependency() {
    // Permit only [Registry]: the path dep is now flagged too (alongside every git
    // dep), documenting that Path is a governed source, not a silent exemption.
    let package = source_package();
    let rule = Rule::RestrictDependencySourcesTo {
        allowed: vec![SourceKind::Registry],
    };
    let findings = rule.findings(&package, &[], DependencyKind::Normal);
    assert!(findings.contains(&"localdep".to_string()), "{findings:?}");
    assert!(!findings.contains(&"crates_io".to_string()), "{findings:?}");
}

#[test]
pub(super) fn source_rule_is_clean_when_every_governed_source_is_allowed() {
    // A package whose only normal deps are registry + path, under [Registry, Path].
    let package = serde_json::json!({
        "dependencies": [
            { "name": "crates_io", "source": "registry+https://x", "kind": null },
            { "name": "localdep", "source": null, "kind": null },
        ]
    });
    let rule = Rule::RestrictDependencySourcesTo {
        allowed: vec![SourceKind::Registry, SourceKind::Path],
    };
    assert!(
        rule.findings(&package, &[], DependencyKind::Normal)
            .is_empty(),
        "all-registry-or-path is clean under a [Registry, Path] allowlist",
    );
}

#[test]
pub(super) fn source_rule_does_not_observe_a_patch_redirect_declared_as_registry() {
    // The declared-vs-resolved bound: a registry dep that `[patch]` would redirect to
    // git still declares `source = registry+…` in `--no-deps` metadata, so it
    // classifies Registry and does NOT violate a [Registry] allowlist. Observing the
    // resolved git source is cargo-deny's `[sources]` lane, not a Tianheng capability.
    let package = serde_json::json!({
        "dependencies": [
            { "name": "patched", "source": "registry+https://x", "kind": null },
        ]
    });
    let rule = Rule::RestrictDependencySourcesTo {
        allowed: vec![SourceKind::Registry],
    };
    assert!(
        rule.findings(&package, &[], DependencyKind::Normal)
            .is_empty(),
        "the declared layer does not observe [patch]; correct — [patch] never blocks publish",
    );
}

#[test]
pub(super) fn source_rule_scopes_to_the_dependency_kind() {
    // Only the git dev-dep exists; a Normal-scoped boundary does not observe it, a
    // Dev-scoped one does.
    let package = serde_json::json!({
        "dependencies": [
            { "name": "devgit", "source": "git+https://x", "kind": "dev" },
        ]
    });
    let rule = Rule::RestrictDependencySourcesTo {
        allowed: vec![SourceKind::Registry],
    };
    assert!(
        rule.findings(&package, &[], DependencyKind::Normal)
            .is_empty(),
        "a dev git dep is outside a Normal-scoped surface",
    );
    assert_eq!(
        rule.findings(&package, &[], DependencyKind::Dev),
        vec!["devgit (dev)".to_string()],
        "a Dev-scoped boundary governs the dev table",
    );
}

#[test]
pub(super) fn source_rule_empty_allowlist_forbids_every_dependency_by_source() {
    let package = serde_json::json!({
        "dependencies": [
            { "name": "crates_io", "source": "registry+https://x", "kind": null },
            { "name": "localdep", "source": null, "kind": null },
            { "name": "gitdep", "source": "git+https://x", "kind": null },
        ]
    });
    let rule = Rule::RestrictDependencySourcesTo { allowed: vec![] };
    assert_eq!(
        rule.findings(&package, &[], DependencyKind::Normal),
        vec![
            "crates_io".to_string(),
            "gitdep".to_string(),
            "localdep".to_string(),
        ],
        "an empty source allowlist forbids every dependency regardless of source",
    );
}

#[test]
pub(super) fn source_boundary_absent_target_is_a_constitution_error() {
    // Parity with the other crate rules: a boundary on a crate not in the workspace is
    // a constitution error (→ exit 2), never a silent pass.
    let metadata = serde_json::json!({ "packages": [{ "name": "present" }] });
    let boundary = CrateBoundary::crate_("absent")
        .restrict_dependency_sources_to([SourceKind::Registry])
        .because("absent must publish to crates.io");
    let mut violations = Vec::new();
    let result = check_crate_boundary(&metadata, &[], &boundary, &mut violations);
    assert!(
        result.is_err(),
        "an absent target crate must be a constitution error, not exit 0/1",
    );
}

#[test]
pub(super) fn source_boundary_carries_its_severity_and_gates_against_the_baseline() {
    // A source violation folds into the shared report identity (target, rule,
    // finding) and honors severity + baseline exactly as the sibling rules do.
    let metadata = serde_json::json!({
        "packages": [{
            "name": "infra",
            "dependencies": [
                { "name": "gitdep", "source": "git+https://x", "kind": null },
            ],
        }]
    });
    // Warn severity: the violation is recorded but must not fail the reaction.
    let warn = CrateBoundary::crate_("infra")
        .restrict_dependency_sources_to([SourceKind::Registry, SourceKind::Path])
        .warn()
        .because("infra should publish; a git source is advisory here");
    let mut violations = Vec::new();
    check_crate_boundary(&metadata, &[], &warn, &mut violations).unwrap();
    assert_eq!(violations.len(), 1);
    assert_eq!(violations[0].severity, Severity::Warn);
    assert_eq!(violations[0].rule, "restrict dependency sources to");
    assert_eq!(violations[0].finding, "gitdep");
    let id = violations[0].id();
    assert_eq!(id.target(), "infra");
    assert_eq!(
        id.rule_key().rule_type(),
        "tianheng.rule/guibiao/restrict-dependency-sources-to"
    );
    let fact = id.fact();
    assert_eq!(fact.fact_type(), "tianheng.fact/guibiao/dependency-source");
    assert_eq!(fact.shape(), "declared-source");
    assert_eq!(
        fact.fields().collect::<Vec<_>>(),
        vec![("kind", "normal"), ("package", "gitdep"), ("source", "git")]
    );
    assert!(
        violations[0].file.is_none(),
        "a source violation is a manifest relation, not a source line",
    );

    // Enforce + baseline parity: the same violation, once baselined, does not fail.
    let enforce = CrateBoundary::crate_("infra")
        .restrict_dependency_sources_to([SourceKind::Registry, SourceKind::Path])
        .because("infra must publish to crates.io, so no git source");
    let mut v = Vec::new();
    check_crate_boundary(&metadata, &[], &enforce, &mut v).unwrap();
    let mut report = Report::new(v);
    let baseline = Baseline::of(&report);
    apply_baseline(&mut report, &baseline);
    assert_eq!(
        Outcome::Violations(report).exit_code(),
        0,
        "a fully baselined source violation does not fail the reaction",
    );
}

#[test]
pub(super) fn source_boundary_projects_its_allowed_sources() {
    let constitution = Constitution::new("p")
        .boundary(
            CrateBoundary::crate_("infra")
                .restrict_dependency_sources_to([SourceKind::Registry, SourceKind::Path])
                .because("infra must publish to crates.io, so its manifest declares no git"),
        )
        .boundary(
            CrateBoundary::crate_("locked")
                .restrict_dependency_sources_to([])
                .because("locked must declare no dependencies at all"),
        );

    let text = constitution_text(&constitution);
    assert!(
        text.contains("restrict dependency sources to: registry, path"),
        "{text}"
    );
    assert!(
        text.contains("forbid all dependencies (by source)"),
        "{text}"
    );

    let doc: serde_json::Value = serde_json::from_str(&constitution_json(&constitution)).unwrap();
    assert_eq!(
        doc["boundaries"][0]["rule"],
        "restrict dependency sources to"
    );
    assert_eq!(doc["boundaries"][0]["allowed_sources"][0], "registry");
    assert_eq!(doc["boundaries"][0]["allowed_sources"][1], "path");
    // The empty allowlist is still emitted, as `[]`.
    assert_eq!(
        doc["boundaries"][1]["allowed_sources"]
            .as_array()
            .unwrap()
            .len(),
        0
    );
}
