use super::render::{coverage_report, report_sarif, violations_text, violations_text_styled};
use super::term_color::Style;
use super::{
    Coverage, boundary_params, check_constitution, constitution_markdown, dispatch, dyn_trait_text,
    impl_trait_text, list_document, list_markdown, merge_outcomes, nearest_manifest_from,
    projection_gate, report_json, runtime_text, semantic_text, trait_impl_text, visibility_text,
};
use crate::prelude::*;
use serde_json::Value;
use std::path::{Path, PathBuf};

/// A unique temp path, cleaned up (as a directory tree or a lone file, whichever this test
/// builds) before use and on drop — replaces the hand-rolled `remove_dir_all`/`remove_file`
/// pre/post pairs this file's fixture tests otherwise each repeat. Doesn't create anything
/// itself; each test still builds its own directory tree or file content under the path.
struct TempPath(PathBuf);

impl TempPath {
    fn new(path: PathBuf) -> Self {
        let guard = Self(path);
        guard.clean();
        guard
    }

    fn path(&self) -> &Path {
        &self.0
    }

    fn clean(&self) {
        if self.0.is_dir() {
            let _ = std::fs::remove_dir_all(&self.0);
        } else {
            let _ = std::fs::remove_file(&self.0);
        }
    }
}

impl Drop for TempPath {
    fn drop(&mut self) {
        self.clean();
    }
}

fn test_id(target: &str, rule: &str, finding: &str) -> ViolationId {
    ViolationId::new(
        target,
        RuleKey::of("tianheng.rule/test/policy", [("policy", rule)]),
        StructuredFactIdentity::new("tianheng-test", "fact", [("value", finding)]).unwrap(),
    )
}

fn violation(target: &str, rule: &str, finding: &str, file: Option<&str>) -> Violation {
    Violation::new(
        BoundaryKind::Crate,
        test_id(target, rule, finding),
        rule,
        finding,
        format!("reason-for-{target}"),
        Severity::Enforce,
    )
    .with_file(file.map(str::to_string))
}

fn enforce_violation(kind: BoundaryKind, finding: &str) -> Violation {
    Violation::new(
        kind,
        test_id("target", "rule", finding),
        "rule",
        finding,
        "reason".to_string(),
        Severity::Enforce,
    )
}

#[test]
fn merge_combines_violations_from_both_dimensions() {
    let static_outcome = Outcome::Violations(Report::new(vec![enforce_violation(
        BoundaryKind::Crate,
        "serde",
    )]));
    let semantic_outcome = Outcome::Violations(Report::new(vec![enforce_violation(
        BoundaryKind::Semantic,
        "crate::infra::DbPool",
    )]));
    let merged = merge_outcomes(static_outcome, semantic_outcome);
    match merged {
        Outcome::Violations(report) => assert_eq!(report.violations.len(), 2),
        other => panic!("expected merged violations, got {other:?}"),
    }
}

#[test]
fn merge_is_clean_only_when_both_are_clean() {
    assert_eq!(
        merge_outcomes(Outcome::Clean, Outcome::Clean),
        Outcome::Clean
    );
}

#[test]
fn a_semantic_constitution_error_supersedes_static_violations() {
    let static_outcome = Outcome::Violations(Report::new(vec![enforce_violation(
        BoundaryKind::Crate,
        "serde",
    )]));
    let semantic_outcome = Outcome::ConstitutionError("module 'crate::ghost' not found".into());
    let merged = merge_outcomes(static_outcome, semantic_outcome);
    assert!(matches!(merged, Outcome::ConstitutionError(_)));
    assert_eq!(
        merged.exit_code(),
        2,
        "a constitution error supersedes (exit 2)"
    );
}

#[test]
fn a_static_constitution_error_wins_when_both_error() {
    let merged = merge_outcomes(
        Outcome::ConstitutionError("bad static crate".into()),
        Outcome::ConstitutionError("bad semantic module".into()),
    );
    assert!(
        matches!(merged, Outcome::ConstitutionError(message) if message == "bad static crate"),
        "the static error is checked first and wins deterministically",
    );
}

#[test]
fn composed_check_preserves_static_error_precedence() {
    let Some(manifest) = workspace_manifest() else {
        return;
    };
    let constitution = Constitution::new("error-precedence")
        .boundary(
            CrateBoundary::crate_("no-such-static-package")
                .forbid_dependency_on(["serde"])
                .because("the static target must resolve first"),
        )
        .signature_boundary(
            SemanticBoundary::in_crate("xuanji")
                .module("crate::no_such_semantic_module")
                .must_not_expose("crate::Hidden")
                .because("the later semantic target is also invalid"),
        );
    let outcome = check_constitution(&constitution, &manifest);
    assert!(
        matches!(outcome, Outcome::ConstitutionError(ref message) if message.contains("no-such-static-package")),
        "the first static constitution error wins before semantic evaluation: {outcome:?}",
    );
}

#[test]
fn semantic_text_lists_each_boundary() {
    let boundary = SemanticBoundary::in_crate("app")
        .module("crate::domain")
        .must_not_expose("crate::infra")
        .because("the domain API must not leak infrastructure types");
    let text = semantic_text(&[boundary]);
    assert!(text.contains("module crate::domain in app"), "{text}");
    assert!(text.contains("must not expose: crate::infra"), "{text}");
}

#[test]
fn including_trait_impls_projects_into_text_json_and_markdown() {
    let boundary = SemanticBoundary::in_crate("app")
        .module("crate::domain")
        .must_not_expose("crate::infra")
        .including_trait_impls()
        .because("no infra leak even via impl-site contracts");
    let text = semantic_text(std::slice::from_ref(&boundary));
    assert!(
        text.contains("must not expose: crate::infra (including trait impls)"),
        "{text}"
    );
    let c = Constitution::new("app").signature_boundary(boundary);
    let json = serde_json::to_string(&list_document(&c)).expect("json");
    assert!(json.contains("\"including_trait_impls\":true"), "{json}");
    let md = constitution_markdown(&c);
    // Pin the exact param group so a future ordering/join change (e.g. preserve_order) is
    // caught: `forbidden` precedes `including_trait_impls` (lexicographic), joined by `; `.
    assert!(
        md.contains("(forbidden: crate::infra; including_trait_impls: true)"),
        "{md}"
    );
}

#[test]
fn a_bare_boundary_omits_the_opt_in_from_every_projection() {
    let boundary = SemanticBoundary::in_crate("app")
        .module("crate::domain")
        .must_not_expose("crate::infra")
        .because("the domain API must not leak infrastructure types");
    let text = semantic_text(std::slice::from_ref(&boundary));
    assert!(!text.contains("including trait impls"), "{text}");
    let c = Constitution::new("app").signature_boundary(boundary);
    let json = serde_json::to_string(&list_document(&c)).expect("json");
    assert!(!json.contains("including_trait_impls"), "{json}");
    let md = constitution_markdown(&c);
    assert!(!md.contains("including_trait_impls"), "{md}");
}

#[test]
fn an_anchored_semantic_boundary_projects_its_anchor_only_when_set() {
    let anchored = SemanticBoundary::in_crate("app")
        .module("crate::domain")
        .must_not_expose("crate::infra")
        .because("the domain API must not leak infrastructure types")
        .with_anchor("ADR-014");
    let c = Constitution::new("app").signature_boundary(anchored);
    let json = serde_json::to_string(&list_document(&c)).expect("json");
    assert!(json.contains("\"anchor\":\"ADR-014\""), "{json}");
    // Markdown derives from the JSON params generically, so the anchor surfaces there too.
    assert!(constitution_markdown(&c).contains("ADR-014"));
    // Parity: the text projection surfaces the anchor too (it appeared in json/markdown but not
    // text before — the `list` three-format parity gap).
    assert!(
        semantic_text(std::slice::from_ref(
            c.semantic_boundaries().signature.first().unwrap()
        ))
        .contains("anchor: ADR-014"),
        "text projection must surface the anchor line, like json and markdown"
    );

    let bare = SemanticBoundary::in_crate("app")
        .module("crate::domain")
        .must_not_expose("crate::infra")
        .because("the domain API must not leak infrastructure types");
    let bare_c = Constitution::new("app").signature_boundary(bare);
    let bare_json = serde_json::to_string(&list_document(&bare_c)).expect("json");
    assert!(!bare_json.contains("anchor"), "{bare_json}");
    // And a bare boundary's text stays byte-identical (no stray `anchor:` line).
    assert!(
        !semantic_text(std::slice::from_ref(
            bare_c.semantic_boundaries().signature.first().unwrap()
        ))
        .contains("anchor"),
        "a boundary without an anchor emits no anchor line in text"
    );
}

#[test]
fn markdown_params_exclude_exactly_the_structural_base_keys() {
    // Guard the hand-maintained `STRUCTURAL` list (markdown.rs) against drift from the keys
    // `boundary_json_base` emits: a boundary JSON with every base field set (crate + anchor) plus a
    // rule param must render the non-structural keys as params and NONE of the structural six. A
    // future always-present base key added without updating `STRUCTURAL` would leak into params here.
    let boundary = serde_json::json!({
        "kind": "semantic", "target": "app", "crate": "app",
        "rule": "expose", "severity": "enforce", "reason": "why",
        "forbidden": ["Foo"], "anchor": "ADR-014",
    });
    let params = boundary_params(&boundary);
    for structural in ["kind", "target", "crate", "rule", "severity", "reason"] {
        assert!(
            !params.contains(&format!("{structural}:")),
            "structural base key `{structural}` leaked into markdown params: {params}"
        );
    }
    assert!(
        params.contains("forbidden:"),
        "a rule param must surface: {params}"
    );
    assert!(
        params.contains("anchor:"),
        "anchor is a non-structural param: {params}"
    );
}

#[test]
fn report_sarif_carries_the_anchor_in_a_result_property_bag() {
    let anchored =
        violation("core", "rule", "serde", None).with_anchor(Some("ADR-014".to_string()));
    let outcome = Outcome::Violations(Report::new(vec![anchored]));
    let sarif: Value = serde_json::from_str(&report_sarif(&outcome)).expect("valid SARIF");
    assert_eq!(
        sarif["runs"][0]["results"][0]["properties"]["anchor"],
        "ADR-014"
    );

    // A violation with no anchor emits no `properties` key — byte-unchanged SARIF.
    let bare = Outcome::Violations(Report::new(vec![violation("core", "rule", "serde", None)]));
    let bare_sarif: Value = serde_json::from_str(&report_sarif(&bare)).expect("valid SARIF");
    assert!(
        bare_sarif["runs"][0]["results"][0]
            .get("properties")
            .is_none()
    );
}

#[test]
fn report_sarif_merges_anchor_and_polarity_into_one_property_bag() {
    // Both present → one bag carrying both, never overwriting (the reconciliation the polarity
    // change made to the anchor-era "no properties" rule).
    let both = violation("core", "rule", "serde", None)
        .with_anchor(Some("ADR-014".to_string()))
        .with_polarity(Polarity::AllowlistGap);
    let sarif: Value =
        serde_json::from_str(&report_sarif(&Outcome::Violations(Report::new(vec![both]))))
            .expect("valid SARIF");
    let props = &sarif["runs"][0]["results"][0]["properties"];
    assert_eq!(props["anchor"], "ADR-014");
    assert_eq!(props["polarity"], "allowlist_gap");

    // Polarity only (no anchor) → properties carries polarity, no anchor key.
    let pol_only = violation("core", "rule", "serde", None).with_polarity(Polarity::DenyBreach);
    let sarif: Value =
        serde_json::from_str(&report_sarif(&Outcome::Violations(Report::new(vec![
            pol_only,
        ]))))
        .expect("valid SARIF");
    let props = &sarif["runs"][0]["results"][0]["properties"];
    assert_eq!(props["polarity"], "deny_breach");
    assert!(props.get("anchor").is_none());

    // Neither (an off-axis / audit-style violation with no polarity) → no properties key at all.
    let neither = Outcome::Violations(Report::new(vec![violation("core", "rule", "serde", None)]));
    let sarif: Value = serde_json::from_str(&report_sarif(&neither)).expect("valid SARIF");
    assert!(sarif["runs"][0]["results"][0].get("properties").is_none());
}

#[test]
fn sarif_fingerprints_file_less_violations_by_their_full_identity() {
    // SARIF presentation carries no target, so target-differing file-less violations need the
    // canonical structured identity fingerprint to remain distinct alerts.
    let same_rule = "deny external dependencies";
    let same_finding = "serde";
    let same_reason = "keep the graph lean";
    let mk = |target: &str| {
        Violation::new(
            BoundaryKind::Crate,
            test_id(target, same_rule, same_finding),
            same_rule,
            same_finding,
            same_reason.to_string(),
            Severity::Enforce,
        )
    };
    let outcome = Outcome::Violations(Report::new(vec![mk("web"), mk("cli")]));
    let sarif: Value = serde_json::from_str(&report_sarif(&outcome)).expect("valid SARIF");
    let results = sarif["runs"][0]["results"]
        .as_array()
        .expect("results array");
    assert_eq!(
        results.len(),
        2,
        "two violations differing only in target are two results: {results:?}"
    );
    let fp = |r: &Value| {
        r["partialFingerprints"]["tianheng/structured-fact-identity"]
            .as_str()
            .expect("a fingerprint string")
            .to_string()
    };
    let (fp0, fp1) = (fp(&results[0]), fp(&results[1]));
    assert_ne!(
        fp0, fp1,
        "target-differing violations must get distinct fingerprints: {fp0} vs {fp1}"
    );
    assert_eq!(fp0.len(), 32);
    assert_eq!(fp1.len(), 32);
    assert!(
        results.iter().all(|result| result["partialFingerprints"]
            .get("tianhengViolationId/v1")
            .is_none()),
        "the presentation-derived fingerprint property is removed"
    );
}

#[test]
fn sarif_fingerprint_ignores_presentation_diagnostics_and_result_order() {
    let identity = ViolationId::new(
        "core",
        RuleKey::of("tianheng.rule/test/deny", [("policy", "external")]),
        StructuredFactIdentity::of(
            "tianheng.fact/test/dependency",
            "dependency-edge",
            [("package", "serde")],
        ),
    );
    let old = Violation::new(
        BoundaryKind::Crate,
        identity.clone(),
        "old rule",
        "old finding",
        "old reason".to_string(),
        Severity::Warn,
    );
    let changed = Violation::new(
        BoundaryKind::Runtime,
        identity,
        "new rule",
        "new finding with signature diagnostics",
        "new reason".to_string(),
        Severity::Enforce,
    )
    .with_file(Some("src/new.rs".to_string()))
    .with_anchor(Some("law".to_string()))
    .with_polarity(Polarity::AllowlistGap);
    let unrelated = violation("other", "other rule", "other fact", None);

    let fingerprint_of = |report: Report, index: usize| {
        let sarif: Value =
            serde_json::from_str(&report_sarif(&Outcome::Violations(report))).unwrap();
        sarif["runs"][0]["results"][index]["partialFingerprints"]
            ["tianheng/structured-fact-identity"]
            .as_str()
            .unwrap()
            .to_string()
    };
    let first = fingerprint_of(Report::new(vec![old, unrelated.clone()]), 0);
    let reordered = fingerprint_of(Report::new(vec![unrelated, changed]), 1);
    assert_eq!(first, reordered);

    let changed_identity = Violation::new(
        BoundaryKind::Crate,
        test_id("core", "old rule", "tokio"),
        "old rule",
        "old finding",
        "old reason".to_string(),
        Severity::Warn,
    );
    assert_ne!(
        first,
        fingerprint_of(Report::new(vec![changed_identity]), 0),
        "an identity-bearing fact change must change the fingerprint"
    );
}

#[test]
fn dyn_trait_text_lists_each_boundary() {
    let boundary = DynTraitBoundary::in_crate("app")
        .module("crate::core")
        .must_not_expose_dyn()
        .because("the core seam is statically dispatched");
    let text = dyn_trait_text(&[boundary]);
    assert!(text.contains("module crate::core in app"), "{text}");
    assert!(text.contains("must not expose dyn"), "{text}");
    assert!(
        text.contains("the core seam is statically dispatched"),
        "{text}"
    );
}

#[test]
fn dyn_trait_boundary_projects_into_list_document_and_markdown() {
    let c = Constitution::new("app").dyn_trait_boundary(
        DynTraitBoundary::in_crate("app")
            .module("crate::core")
            .must_not_expose_dyn()
            .because("the core seam is statically dispatched"),
    );
    let doc = list_document(&c);
    let arr = doc
        .get("dyn_trait_boundaries")
        .and_then(Value::as_array)
        .expect("dyn_trait_boundaries projected");
    assert_eq!(arr.len(), 1);
    assert_eq!(arr[0]["rule"], "must not expose dyn");
    assert!(
        arr[0].get("forbidden").is_none(),
        "shape-only: no forbidden set"
    );
    let md = list_markdown(&doc);
    assert!(md.contains("## Dyn-trait boundaries"), "{md}");
    assert!(md.contains("must not expose dyn"), "{md}");
    assert!(
        md.contains("the core seam is statically dispatched"),
        "{md}"
    );
}

#[test]
fn async_exposure_boundary_projects_into_list_document_and_markdown() {
    let c = Constitution::new("app").async_exposure_boundary(
        AsyncExposureBoundary::in_crate("app")
            .module("crate::core")
            .must_not_expose_async_fn()
            .because("the core seam is synchronous; async lives at the edges"),
    );
    let doc = list_document(&c);
    let arr = doc["async_exposure_boundaries"]
        .as_array()
        .expect("projected");
    assert_eq!(arr[0]["rule"], "must not expose async fn");
    assert_eq!(arr[0]["target"], "crate::core");
    let md = list_markdown(&doc);
    assert!(md.contains("## Async-exposure boundaries"), "{md}");
    assert!(md.contains("must not expose async fn"), "{md}");
}

#[test]
fn impl_trait_boundary_projects_into_list_document_and_markdown() {
    let c = Constitution::new("app").impl_trait_boundary(
        ImplTraitBoundary::in_crate("app")
            .module("crate::core")
            .must_not_expose_impl_trait()
            .because("the core seam must return named types, not an existential"),
    );
    let doc = list_document(&c);
    let arr = doc["impl_trait_boundaries"].as_array().expect("projected");
    assert_eq!(arr.len(), 1);
    assert_eq!(arr[0]["rule"], "must not expose impl trait");
    assert_eq!(arr[0]["target"], "crate::core");
    let md = list_markdown(&doc);
    assert!(md.contains("## Impl-trait boundaries"), "{md}");
    assert!(md.contains("must not expose impl trait"), "{md}");
}

#[test]
fn operand_scoped_impl_trait_boundary_projects_its_forbidden_operands() {
    let c = Constitution::new("app").impl_trait_boundary(
        ImplTraitBoundary::in_crate("app")
            .module("crate::core")
            .must_not_expose_impl_trait_of(["crate::ports::Port"])
            .because("the core seam must not return an existential Port"),
    );
    let doc = list_document(&c);
    let arr = doc["impl_trait_boundaries"].as_array().expect("projected");
    assert_eq!(arr[0]["rule"], "must not expose impl trait");
    assert_eq!(arr[0]["forbidden"][0], "crate::ports::Port");
    let md = list_markdown(&doc);
    assert!(
        md.contains("forbidden: crate::ports::Port"),
        "the operand set surfaces as a param:\n{md}"
    );
    // Text parity: the operand set must surface in `list --format text` too, not only JSON /
    // markdown — else an operand-scoped and a shape-only boundary read identically in text.
    let text = impl_trait_text(&[ImplTraitBoundary::in_crate("app")
        .module("crate::core")
        .must_not_expose_impl_trait_of(["crate::ports::Port"])
        .because("the core seam must not return an existential Port")]);
    assert!(
        text.contains("must not expose impl trait of: crate::ports::Port"),
        "operand set must surface in text:\n{text}"
    );
}

#[test]
fn operand_scoped_dyn_boundary_projects_its_forbidden_operands() {
    let c = Constitution::new("app").dyn_trait_boundary(
        DynTraitBoundary::in_crate("app")
            .module("crate::core")
            .must_not_expose_dyn_of(["crate::ports::Port"])
            .because("the core seam must not leak a dyn Port"),
    );
    let doc = list_document(&c);
    let arr = doc["dyn_trait_boundaries"].as_array().expect("projected");
    assert_eq!(arr[0]["rule"], "must not expose dyn");
    assert_eq!(
        arr[0]["forbidden"][0], "crate::ports::Port",
        "an operand-scoped boundary projects its forbidden operand set"
    );
    let md = list_markdown(&doc);
    assert!(
        md.contains("forbidden: crate::ports::Port"),
        "the operand set surfaces as a generic param:\n{md}"
    );
    // Text parity: the operand set must surface in `list --format text` too (see the impl-trait
    // sibling test) — the defect that let text drift from JSON / markdown for operand scoping.
    let text = dyn_trait_text(&[DynTraitBoundary::in_crate("app")
        .module("crate::core")
        .must_not_expose_dyn_of(["crate::ports::Port"])
        .because("the core seam must not leak a dyn Port")]);
    assert!(
        text.contains("must not expose dyn of: crate::ports::Port"),
        "operand set must surface in text:\n{text}"
    );
}

#[test]
fn trait_impl_text_lists_each_boundary() {
    let boundary = TraitImplBoundary::in_crate("app")
        .trait_("crate::command::Command")
        .only_implemented_in("crate::commands")
        .and_in("crate::builtins")
        .because("Command impls live with the registry");
    let text = trait_impl_text(&[boundary]);
    assert!(
        text.contains("trait crate::command::Command in app"),
        "{text}"
    );
    // Unified with the JSON/check phrasing (the three-way drift closed): the text now says
    // "must only be implemented in the declared location(s)", with the locations as a detail.
    assert!(
        text.contains("must only be implemented in the declared location(s)"),
        "{text}"
    );
    assert!(text.contains("crate::commands, crate::builtins"), "{text}");
}

#[test]
fn visibility_text_lists_each_boundary_and_is_empty_when_none() {
    // The empty-guard protects existing `list` output: a project not using the
    // dimension gets byte-identical projection (no section emitted).
    assert_eq!(visibility_text(&[]), "");
    let boundary = VisibilityBoundary::in_crate("app")
        .module("crate::internal")
        .must_not_declare_pub()
        .because("internal is an impl detail");
    let text = visibility_text(&[boundary]);
    assert!(text.contains("module crate::internal in app"), "{text}");
    assert!(text.contains("must not declare pub items"), "{text}");

    // A non-Crate ceiling must render its own rule label in text too (one source with check/json/
    // markdown) — regression guard for the text projection desync.
    let sealed = VisibilityBoundary::in_crate("app")
        .module("crate::deep")
        .max_visibility(VisibilityCeiling::Super)
        .because("sealed to its parent");
    let sealed_text = visibility_text(&[sealed]);
    assert!(
        sealed_text.contains("must not declare items more visible than pub(super)"),
        "{sealed_text}"
    );
}

#[test]
fn merge_folds_a_trait_impl_violation_into_the_report() {
    // The three-dimension composition reuses the same binary merge: a trait-impl
    // finding lands in the one aggregated report alongside static and semantic ones.
    let static_outcome = Outcome::Violations(Report::new(vec![enforce_violation(
        BoundaryKind::Crate,
        "serde",
    )]));
    let trait_impl_outcome = Outcome::Violations(Report::new(vec![enforce_violation(
        BoundaryKind::Semantic,
        "crate::domain (impl for Foo)",
    )]));
    let merged = merge_outcomes(static_outcome, trait_impl_outcome);
    match merged {
        Outcome::Violations(report) => assert_eq!(report.violations.len(), 2),
        other => panic!("expected merged violations, got {other:?}"),
    }
}

fn fixture(name: &str) -> String {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures")
        .join(name)
        .join("Cargo.toml")
        .to_string_lossy()
        .into_owned()
}

/// The Tianheng workspace manifest, two levels up. `None` when it is absent — e.g. inside a
/// published `.crate` tarball, which has no workspace root — so the workspace-dependent
/// dispatch tests below SKIP rather than fail when the crate is tested standalone. In the
/// repo the path exists, so they run as a real end-to-end gate.
fn workspace_manifest() -> Option<PathBuf> {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../Cargo.toml");
    if path.exists() {
        return Some(path);
    }
    // Absent. In the repo/CI the workspace root always exists, so CI sets
    // TIANHENG_WORKSPACE_TESTS=1 to turn a missing manifest (a checkout/layout regression)
    // into a LOUD failure rather than a silent skip of the gate. Without the env (e.g. a
    // packaged .crate tested standalone) the absence is legitimate, so skip.
    assert!(
        std::env::var_os("TIANHENG_WORKSPACE_TESTS").is_none(),
        "workspace manifest expected but absent while TIANHENG_WORKSPACE_TESTS is set — \
             the dispatch gate must not silently skip in CI"
    );
    None
}

fn example_constitution() -> Constitution {
    Constitution::new("example").boundary(
        CrateBoundary::crate_("example-core")
            .deny_external_dependencies()
            .because("example-core must stay dependency-light"),
    )
}

fn run_args(args: &[&str]) -> u8 {
    dispatch(&example_constitution(), args.iter().map(|s| s.to_string()))
}

// Most runner unit tests below need no fixture: each asserts an exit code decided
// during argument parsing, before any workspace is observed. The reaction paths that
// require a real workspace are exercised against one directly: `tests/self_governance.rs`
// drives the static `check` end-to-end against Tianheng's own workspace, and the
// dispatch tests below (e.g. `the_trait_impl_dimension_is_wired_through_dispatch`) drive
// each dimension through `dispatch` + real `cargo metadata`. The per-dimension finding
// logic is unit-tested in its own crate's pure heart (`hunyi`).

#[test]
fn the_trait_impl_dimension_is_wired_through_dispatch() {
    // End-to-end proof the new dimension is composed into `dispatch` (not only
    // unit-tested in isolation): an unresolvable trait anchor must flow through dispatch
    // and real `cargo metadata` to a constitution error (exit 2). The static
    // constitution is empty (clean), so the exit-2 can only come from the trait-impl
    // dimension — proving it is actually evaluated.
    let Some(manifest) = workspace_manifest() else {
        return; // no workspace root (e.g. a packaged crate) — skip this end-to-end test
    };
    let boundary = TraitImplBoundary::in_crate("xuanji")
        .trait_("crate::NoSuchTrait")
        .only_implemented_in("crate::nowhere")
        .because("wiring check");
    let code = dispatch(
        &Constitution::new("wiring").trait_impl_boundary(boundary),
        [
            "tianheng".to_string(),
            "check".to_string(),
            "--manifest-path".to_string(),
            manifest.to_string_lossy().into_owned(),
        ],
    );
    assert_eq!(
        code, 2,
        "an unresolvable trait anchor reaches exit 2 through dispatch"
    );
}

#[test]
fn the_visibility_dimension_is_wired_through_dispatch() {
    // End-to-end proof the visibility dimension is composed into `dispatch`: an
    // unresolvable module anchor flows through dispatch + real `cargo metadata` to a
    // constitution error (exit 2). Empty static constitution, so exit-2 can only come
    // from the visibility dimension.
    let Some(manifest) = workspace_manifest() else {
        return; // no workspace root (e.g. a packaged crate) — skip this end-to-end test
    };
    let boundary = VisibilityBoundary::in_crate("xuanji")
        .module("crate::no_such_module")
        .must_not_declare_pub()
        .because("wiring check");
    let code = dispatch(
        &Constitution::new("wiring").visibility_boundary(boundary),
        [
            "tianheng".to_string(),
            "check".to_string(),
            "--manifest-path".to_string(),
            manifest.to_string_lossy().into_owned(),
        ],
    );
    assert_eq!(
        code, 2,
        "an unresolvable visibility module anchor reaches exit 2 through dispatch"
    );
}

#[test]
fn the_runtime_dimension_is_wired_through_dispatch() {
    // End-to-end proof the 漏刻 CI face is composed into `dispatch`: a declared runtime seam
    // with no probe anywhere in the workspace flows through dispatch + real `cargo metadata`
    // (member-src-dir resolution) + the probe-coverage audit to an enforce violation (exit 1).
    // The static and semantic dimensions are empty, so the exit-1 can only come from the
    // runtime audit — proving it is actually evaluated against the workspace.
    let Some(manifest) = workspace_manifest() else {
        return; // no workspace root (e.g. a packaged crate) — skip this end-to-end test
    };
    let args = || {
        [
            "tianheng".to_string(),
            "check".to_string(),
            "--manifest-path".to_string(),
            manifest.to_string_lossy().into_owned(),
        ]
    };
    let boundary = RuntimeBoundary::at("a-seam-no-probe-covers")
        .only_origins(["app::domain"])
        .because("wiring check");
    let code = dispatch(&Constitution::new("wiring").runtime(boundary), args());
    assert_eq!(
        code, 1,
        "a declared-but-unprobed runtime seam reaches exit 1 through dispatch"
    );
    // Causation: with NO runtime boundary the audit still runs (it is no longer guarded off an
    // empty boundary set) but finds no probe in this workspace's in-scope source, so the same
    // workspace exits 0 — proving the exit-1 above is caused by the declared-unprobed seam, not
    // pre-existing drift. (The orphan-probe direction — a probe with no declared seam — is
    // exercised by `an_orphan_probe_reacts_with_no_declared_boundary` below.)
    assert_eq!(
        dispatch(&Constitution::new("wiring"), args()),
        0,
        "an empty constitution over a probe-free workspace is clean (the audit runs, finds nothing)"
    );
}

#[test]
fn an_orphan_probe_reacts_with_no_declared_boundary() {
    // Fixture-driven: the `orphan_probe`/`clean`/`violating` fixtures are not shipped in the
    // packaged `.crate`, so skip when absent — the same repo-vs-packaged sentinel the other
    // dispatch tests use (`TIANHENG_WORKSPACE_TESTS` turns a missing repo layout into a loud
    // failure, never a silent skip in CI).
    if workspace_manifest().is_none() {
        return;
    }
    // The change's purpose: a member's source carries an `assert_boundary!("ghost", …)` probe
    // but the constitution declares NO runtime boundary (a boundary deleted, its probe left
    // behind). The audit now runs even against an empty boundary set, so the orphan probe
    // reacts as an undeclared seam (exit 1) — previously the audit was skipped and this passed
    // green, then panicked in production. The `orphan_probe` fixture is its own workspace.
    let orphan_manifest = fixture("orphan_probe");
    let args = [
        "tianheng".to_string(),
        "check".to_string(),
        "--manifest-path".to_string(),
        orphan_manifest.clone(),
    ];
    assert_eq!(
        dispatch(&Constitution::new("empty"), args),
        1,
        "an orphan `assert_boundary!` probe with no declared boundary reacts at CI"
    );
    assert_eq!(
        check_constitution(&Constitution::new("empty"), &PathBuf::from(orphan_manifest),)
            .exit_code(),
        1,
        "the library check shares the always-run orphan-probe audit",
    );
    // Contrast: the `clean` fixture has no probe, so an empty constitution scans clean — the
    // always-run audit does not disturb a probe-free workspace.
    let clean_manifest = fixture("clean");
    let clean_args = [
        "tianheng".to_string(),
        "check".to_string(),
        "--manifest-path".to_string(),
        clean_manifest.clone(),
    ];
    assert_eq!(
        dispatch(&Constitution::new("empty"), clean_args),
        0,
        "a probe-free workspace under an empty constitution stays clean"
    );
    assert_eq!(
        check_constitution(&Constitution::new("empty"), &PathBuf::from(clean_manifest),),
        Outcome::Clean,
        "the library check keeps an empty constitution clean on a probe-free workspace",
    );
}

#[test]
fn the_crate_source_rule_is_wired_through_dispatch() {
    // End-to-end proof the crate-source rule composes into `dispatch` (not only unit-tested in
    // guibiao's pure heart): restricting `guibiao`'s declared dependency sources to Registry
    // flags its internal `xuanji` path dependency (source-kind Path), flowing through dispatch
    // + real `cargo metadata` to an enforce violation (exit 1). Causation: the same workspace
    // under an empty constitution is clean (exit 0), so the exit-1 is caused by this rule.
    let Some(manifest) = workspace_manifest() else {
        return; // no workspace root (e.g. a packaged crate) — skip this end-to-end test
    };
    let args = || {
        [
            "tianheng".to_string(),
            "check".to_string(),
            "--manifest-path".to_string(),
            manifest.to_string_lossy().into_owned(),
        ]
    };
    let c = Constitution::new("wiring").boundary(
        CrateBoundary::crate_("guibiao")
            .restrict_dependency_sources_to([SourceKind::Registry])
            .because("wiring check"),
    );
    assert_eq!(
        dispatch(&c, args()),
        1,
        "a path dependency under a Registry-only source rule reaches exit 1 through dispatch"
    );
    assert_eq!(
        dispatch(&Constitution::new("wiring"), args()),
        0,
        "an empty constitution over the same workspace is clean"
    );
}

#[test]
fn the_runtime_audit_reports_the_declared_unprobed_seam() {
    // Specificity (robust to noise): resolve the workspace's member src roots and run the
    // audit directly, asserting the *named* declared-unprobed seam surfaces — so this cannot
    // pass for the wrong reason (Direction-B / un-auditable noise elsewhere).
    let Some(manifest) = workspace_manifest() else {
        return; // no workspace root (e.g. a packaged crate) — skip this end-to-end test
    };
    let src_dirs = crate::workspace_member_src_dirs(&manifest).expect("resolve src dirs");
    let boundary = RuntimeBoundary::at("a-seam-no-probe-covers")
        .only_origins(["app::domain"])
        .because("wiring check");
    let outcome = crate::audit_probe_coverage(&[boundary], &src_dirs);
    match outcome {
        Outcome::Violations(report) => assert!(
            report
                .violations
                .iter()
                .any(|v| v.target == "a-seam-no-probe-covers"
                    && v.finding.contains("no assert_boundary! probe")),
            "the declared-unprobed seam must be the reported finding: {:?}",
            report.violations
        ),
        other => panic!("expected a violation naming the unprobed seam, got {other:?}"),
    }
}

#[test]
fn composed_runtime_audit_uses_custom_roots_and_rejects_orphan_only_coverage() {
    let base = TempPath::new(
        std::env::temp_dir().join(format!("tianheng-runtime-root-{}", std::process::id())),
    );
    let base = base.path();
    std::fs::create_dir_all(base).unwrap();
    std::fs::write(
        base.join("Cargo.toml"),
        "[package]\nname='runtime-root-fixture'\nversion='0.0.0'\nedition='2021'\n\
         [lib]\npath='custom_root.rs'\n[workspace]\n",
    )
    .unwrap();
    std::fs::write(base.join("custom_root.rs"), "mod live;").unwrap();
    std::fs::write(
        base.join("live.rs"),
        "fn f() { assert_boundary!(\"reachable\", o); }",
    )
    .unwrap();
    std::fs::write(
        base.join("orphan.rs"),
        "fn f() { assert_boundary!(\"orphan\", o); }",
    )
    .unwrap();

    let constitution = Constitution::new("root-aware")
        .runtime(
            RuntimeBoundary::at("reachable")
                .only_origins(["o"])
                .because("reachable probe remains covered"),
        )
        .runtime(
            RuntimeBoundary::at("orphan")
                .only_origins(["o"])
                .because("an orphan file never enforces a runtime seam"),
        );
    let outcome = check_constitution(&constitution, &base.join("Cargo.toml"));
    let violations = match outcome {
        Outcome::Violations(report) => report.violations,
        other => panic!("orphan-only coverage must react: {other:?}"),
    };
    assert_eq!(
        violations.len(),
        1,
        "reachable custom-root module stays covered"
    );
    assert_eq!(violations[0].target, "orphan");

    let _ = std::fs::remove_dir_all(base);
}

#[test]
fn list_document_covers_every_populated_dimension() {
    // The previous json-list test ran only an empty SemanticBoundaries, so the projection's
    // per-dimension key insertion was never exercised (a blind spot). Build one boundary of
    // every dimension and assert each lands in the document — and that an empty dimension
    // adds no key (the static-only projection stays byte-identical).
    let empty = Constitution::new("empty");
    let doc = list_document(&empty);
    assert_eq!(doc["format"], "tianheng.constitution/declared-boundaries");
    assert!(
        doc.get("semantic_boundaries").is_none(),
        "empty adds no key: {doc}"
    );
    assert!(
        doc.get("runtime_boundaries").is_none(),
        "empty adds no key: {doc}"
    );

    let full = Constitution::new("full")
        .boundary(
            CrateBoundary::crate_("core")
                .deny_external_dependencies()
                .because("core stays light"),
        )
        .signature_boundary(
            SemanticBoundary::in_crate("app")
                .module("crate::domain")
                .must_not_expose("crate::infra")
                .because("no infra leak"),
        )
        .trait_impl_boundary(
            TraitImplBoundary::in_crate("app")
                .trait_("crate::Command")
                .only_implemented_in("crate::commands")
                .because("impls live with the registry"),
        )
        .visibility_boundary(
            VisibilityBoundary::in_crate("app")
                .module("crate::internal")
                .must_not_declare_pub()
                .because("internal is private"),
        )
        .forbidden_marker_boundary(
            ForbiddenMarkerBoundary::in_crate("app")
                .module("crate::domain")
                .must_not_acquire("serde::Serialize")
                .because("domain is not wire"),
        )
        .runtime(
            RuntimeBoundary::at("domain-entry")
                .only_origins(["app::domain"])
                .because("only domain crosses"),
        );
    let doc = list_document(&full);
    // Each populated dimension is a non-empty array whose first entry carries the kind and
    // target the projection contract promises (deep-checked, not merely present).
    for (key, kind, target) in [
        ("semantic_boundaries", "semantic", "crate::domain"),
        ("trait_impl_boundaries", "semantic", "crate::Command"),
        ("visibility_boundaries", "semantic", "crate::internal"),
        ("forbidden_marker_boundaries", "semantic", "crate::domain"),
        ("runtime_boundaries", "runtime", "domain-entry"),
    ] {
        let arr = doc[key]
            .as_array()
            .unwrap_or_else(|| panic!("{key} must be an array: {doc}"));
        assert!(!arr.is_empty(), "{key} must be non-empty: {doc}");
        assert_eq!(arr[0]["kind"], kind, "{key}[0] kind: {}", arr[0]);
        assert_eq!(arr[0]["target"], target, "{key}[0] target: {}", arr[0]);
    }

    // And the text projection of the runtime section is non-empty and names the seam.
    let text = runtime_text(full.runtime_boundaries());
    assert!(text.contains("seam domain-entry"), "{text}");
}

/// The Markdown projection must carry a section for **every** dimension the JSON document emits
/// (`constitution-projection`'s "no less than the JSON" guarantee). This reaction replaces a
/// hand-maintained capability enumeration — so a capability added to `list_document` without a `list_markdown` section fails
/// CI here rather than silently under-projecting. The dimension set is enumerated in the test (a
/// reaction), not in prose; the final parity count catches a *new* dimension nobody wired in.
#[test]
fn markdown_projection_covers_every_dimension_the_json_document_emits() {
    let full = Constitution::new("app")
        .boundary(
            CrateBoundary::crate_("app")
                .deny_external_dependencies()
                .because("the core stays dependency-light"),
        )
        .signature_boundary(
            SemanticBoundary::in_crate("app")
                .module("crate::domain")
                .must_not_expose("crate::infra")
                .because("the domain API must not leak infra"),
        )
        .trait_impl_boundary(
            TraitImplBoundary::in_crate("app")
                .trait_("crate::command::Command")
                .only_implemented_in("crate::commands")
                .because("Command impls live with the registry"),
        )
        .visibility_boundary(
            VisibilityBoundary::in_crate("app")
                .module("crate::internal")
                .must_not_declare_pub()
                .because("internal is private"),
        )
        .forbidden_marker_boundary(
            ForbiddenMarkerBoundary::in_crate("app")
                .module("crate::domain")
                .must_not_acquire("serde::Serialize")
                .because("domain is not wire"),
        )
        .dyn_trait_boundary(
            DynTraitBoundary::in_crate("app")
                .module("crate::core")
                .must_not_expose_dyn()
                .because("the core seam is statically dispatched"),
        )
        .impl_trait_boundary(
            ImplTraitBoundary::in_crate("app")
                .module("crate::core")
                .must_not_expose_impl_trait()
                .because("the core seam returns named types"),
        )
        .async_exposure_boundary(
            AsyncExposureBoundary::in_crate("app")
                .module("crate::core")
                .must_not_expose_async_fn()
                .because("the core seam is synchronous"),
        )
        .runtime(
            RuntimeBoundary::at("domain-entry")
                .only_origins(["app::domain"])
                .because("only domain crosses"),
        );

    let doc = list_document(&full);
    let md = list_markdown(&doc);

    // Each known dimension: the fixture must populate it (a non-empty JSON array), and the
    // Markdown must carry its section — so the Markdown never carries less than the JSON.
    for (key, heading) in [
        ("boundaries", "## Static boundaries"),
        ("semantic_boundaries", "## Semantic boundaries"),
        ("trait_impl_boundaries", "## Trait-impl-locality boundaries"),
        ("visibility_boundaries", "## Visibility boundaries"),
        (
            "forbidden_marker_boundaries",
            "## Forbidden-marker boundaries",
        ),
        ("dyn_trait_boundaries", "## Dyn-trait boundaries"),
        ("impl_trait_boundaries", "## Impl-trait boundaries"),
        ("async_exposure_boundaries", "## Async-exposure boundaries"),
        ("runtime_boundaries", "## Runtime boundaries"),
    ] {
        assert!(
            doc.get(key)
                .and_then(Value::as_array)
                .is_some_and(|a| !a.is_empty()),
            "fixture must populate {key} so this guard is not vacuous: {doc}"
        );
        assert!(
            md.contains(heading),
            "Markdown must carry a `{heading}` section for `{key}` — it under-projects:\n{md}"
        );
    }

    // Parity: no dimension the JSON emits is left unenumerated above. A NEW dimension added to
    // `list_document` (and this fixture) that is not wired into `list_markdown` and listed here
    // trips this count — the drift is CI-caught, never a silent under-projection.
    let json_dimensions = doc
        .as_object()
        .expect("list_document is a JSON object")
        .keys()
        .filter(|key| key.ends_with("boundaries"))
        .count();
    assert_eq!(
        json_dimensions, 9,
        "list_document emits {json_dimensions} dimensions but this coverage guard enumerates 9; \
             a new dimension must be wired into list_markdown's section table and added here"
    );
}

/// A multi-dimension constitution to exercise the Markdown projection across every
/// dimension at once (mirrors the JSON test's `full`).
fn full_constitution() -> Constitution {
    Constitution::new("full")
        .boundary(
            CrateBoundary::crate_("core")
                .deny_external_dependencies()
                .because("core stays light"),
        )
        .signature_boundary(
            SemanticBoundary::in_crate("app")
                .module("crate::domain")
                .must_not_expose("crate::infra")
                .because("no infra leak"),
        )
        .trait_impl_boundary(
            TraitImplBoundary::in_crate("app")
                .trait_("crate::Command")
                .only_implemented_in("crate::commands")
                .because("impls live with the registry"),
        )
        .visibility_boundary(
            VisibilityBoundary::in_crate("app")
                .module("crate::internal")
                .must_not_declare_pub()
                .because("internal is private"),
        )
        .forbidden_marker_boundary(
            ForbiddenMarkerBoundary::in_crate("app")
                .module("crate::domain")
                .must_not_acquire("serde::Serialize")
                .because("domain is not wire"),
        )
        .runtime(
            RuntimeBoundary::at("domain-entry")
                .only_origins(["app::domain"])
                .because("only domain crosses"),
        )
}

#[test]
fn list_markdown_covers_every_dimension_with_target_rule_and_reason() {
    // The Markdown is rendered from `list_document`, so this also proves it carries no less
    // than the JSON: every dimension's target, rule parameter, and declared reason appear.
    let md = list_markdown(&list_document(&full_constitution()));
    assert!(md.contains("# Constitution: full"), "{md}");
    // A section heading per non-empty dimension.
    for heading in [
        "## Static boundaries",
        "## Semantic boundaries",
        "## Trait-impl-locality boundaries",
        "## Visibility boundaries",
        "## Forbidden-marker boundaries",
        "## Runtime boundaries",
    ] {
        assert!(md.contains(heading), "missing {heading} in:\n{md}");
    }
    // Each dimension's target, a rule parameter, and its reason (the agent-actionable triple).
    for needle in [
        "core",                // static target
        "core stays light",    // static reason
        "crate::domain",       // semantic target
        "crate::infra",        // semantic forbidden param
        "no infra leak",       // semantic reason
        "crate::Command",      // trait-impl target
        "crate::commands",     // trait-impl allowed_locations param
        "crate::internal",     // visibility target
        "serde::Serialize",    // forbidden-marker param
        "domain-entry",        // runtime seam target
        "app::domain",         // runtime allowed_origins param
        "only domain crosses", // runtime reason
    ] {
        assert!(md.contains(needle), "missing '{needle}' in:\n{md}");
    }
}

#[test]
fn constitution_markdown_equals_the_cli_projection_byte_for_byte() {
    // The public helper MUST add nothing of its own — no preamble, no trailing newline — so
    // it equals what the `list --format markdown` branch prints (`list_markdown(&list_document)`,
    // via `print!`). This guards Contract A's "same renderer, no parallel projection path":
    // a stray newline or wrapper here would silently drift the agent artifact from the CLI.
    let c = full_constitution();
    assert_eq!(constitution_markdown(&c), list_markdown(&list_document(&c)));
}

#[test]
fn markdown_foregrounds_the_reason_before_rule_and_classification() {
    // Contract B / 潛移: the reason leads the block. This asserts the ORDERING INVARIANT ONLY
    // (reason before rule before kind/severity). It deliberately does NOT assert the blockquote
    // rendering — the spec frees "the blockquote choice, wording, spacing" — so the layout stays
    // free to evolve; never a byte-for-byte snapshot.
    let c = Constitution::new("t").boundary(
        CrateBoundary::crate_("core")
            .deny_external_dependencies()
            .because("the gravity-bearing principle text"),
    );
    let md = constitution_markdown(&c);
    let r = md
        .find("the gravity-bearing principle text")
        .expect("reason");
    let rule = md.find("**rule**").expect("rule");
    let kind = md.find("**kind**").expect("kind");
    assert!(
        r < rule && rule < kind,
        "reason must lead, then rule, then classification:\n{md}"
    );
}

#[test]
fn markdown_projects_a_dependency_source_boundary_with_its_allowed_sources() {
    // The source rule projects through the generic static-boundary path (no per-rule
    // markdown code): its label and the `allowed_sources` param surface as params.
    let c = Constitution::new("t").boundary(
        CrateBoundary::crate_("infra")
            .restrict_dependency_sources_to([SourceKind::Registry, SourceKind::Path])
            .because("infra must publish to crates.io"),
    );
    let md = constitution_markdown(&c);
    assert!(
        md.contains("restrict dependency sources to"),
        "the source rule label surfaces:\n{md}"
    );
    assert!(
        md.contains("allowed_sources: registry, path"),
        "the allowed source kinds surface as a generic param:\n{md}"
    );
}

#[test]
fn markdown_reasonless_boundary_has_no_blockquote_or_orphan_blank_line() {
    // No reason → no blockquote, and the heading is immediately followed by the rule bullet
    // (no orphan blank line where the blockquote would have been).
    let c = Constitution::new("t").boundary(
        CrateBoundary::crate_("core")
            .deny_external_dependencies()
            .because(""),
    );
    let md = constitution_markdown(&c);
    assert!(!md.contains("\n> "), "no blockquote when no reason:\n{md}");
    assert!(
        md.contains("### `core`\n- **rule**"),
        "heading immediately followed by the rule bullet:\n{md}"
    );
}

#[test]
fn report_text_leads_with_reason_and_shows_the_offending_file() {
    let report = Report::new(vec![violation(
        "crate::core",
        "must not import crate::adapter",
        "crate::adapter::Db",
        Some("src/core/mod.rs"),
    )]);
    let text = violations_text(&report);
    let reason = text.find("Reason:").expect("reason");
    let boundary = text.find("Boundary:").expect("boundary");
    let rule = text.find("Rule:").expect("rule");
    let found = text.find("Found:").expect("found");
    let file = text.find("File:").expect("file");
    let reaction = text.find("Reaction:").expect("reaction");
    assert!(
        reason < boundary && boundary < rule && rule < found && found < file && file < reaction,
        "order must be reason → boundary → rule → found → file → reaction:\n{text}"
    );
    assert!(
        text.contains("File:\n  src/core/mod.rs"),
        "the offending file is shown as the repair location:\n{text}"
    );
}

#[test]
fn plain_render_carries_no_ansi_escapes() {
    // The un-styled report — what a pipe, a CI log, and every unit test see — must stay a clean
    // byte stream: presentation colour never leaks into the machine-facing default output.
    let report = Report::new(vec![violation("crate::core", "rule", "finding", None)]);
    let text = violations_text(&report);
    assert!(
        !text.contains('\u{1b}'),
        "plain render must contain no ANSI escape:\n{text:?}"
    );
}

#[test]
fn constitution_error_machine_output_carries_no_ansi() {
    // The machine projections must stay a clean byte stream regardless of the coloured exit-2
    // constitution-error voice on the human text path — presentation never leaks into the
    // JSON/SARIF a tool parses, the same presentation⊥verdict contract the violation paths hold.
    let outcome = Outcome::ConstitutionError("module 'crate::ghost' not found".into());
    let json = report_json(&outcome, &[], None);
    let sarif = report_sarif(&outcome);
    assert!(
        !json.contains('\u{1b}'),
        "the JSON projection of a constitution error must carry no ANSI escape:\n{json}"
    );
    assert!(
        !sarif.contains('\u{1b}'),
        "the SARIF projection of a constitution error must carry no ANSI escape:\n{sarif}"
    );
}

#[test]
fn style_error_is_plain_identity_and_active_wraps() {
    // The exit-2 error voice follows the same rule as the severity headers: PLAIN is byte-identical
    // (what a pipe/CI/redirect sees), ACTIVE wraps the identical text in escapes. So the newly
    // coloured error path cannot change what a non-terminal consumer reads.
    let msg = "Tianheng constitution error: module 'crate::ghost' not found";
    assert_eq!(Style::PLAIN.error(msg), msg, "PLAIN error is the identity");
    let active = Style::ACTIVE.error(msg);
    assert!(
        active.contains('\u{1b}'),
        "ACTIVE error carries ANSI escapes"
    );
    assert!(
        active.contains(msg),
        "ACTIVE error wraps the identical text"
    );
}

#[test]
fn active_render_colours_around_the_text_without_changing_it() {
    // With colour active the escape codes appear, but the reason text and the field order are
    // unchanged — colour wraps the fields, it never reorders or removes them.
    let report = Report::new(vec![violation("crate::core", "rule", "finding", None)]);
    let styled = violations_text_styled(&report, Style::ACTIVE);
    assert!(
        styled.contains('\u{1b}'),
        "active render must carry ANSI escapes"
    );
    assert!(
        styled.contains("reason-for-crate::core"),
        "the reason text survives styling"
    );
    // Same field order as the plain render — colour is layered around, not through, the structure.
    let plain = violations_text(&report);
    let strip = |s: &str| {
        s.replace('\u{1b}', "")
            .replace("[0m", "")
            .replace("[1m", "")
            .replace("[1;31m", "")
            .replace("[1;33m", "")
    };
    assert_eq!(
        strip(&styled),
        strip(&plain),
        "stripping the escapes yields the plain report byte-for-byte"
    );
}

#[test]
fn report_text_omits_the_file_element_when_absent() {
    let report = Report::new(vec![violation("crate::x", "rule", "finding", None)]);
    let text = violations_text(&report);
    assert!(
        !text.contains("File:"),
        "no file element when the violation carries none:\n{text}"
    );
}

#[test]
fn report_text_shows_anchor_and_repair_polarity_after_the_located_facts() {
    // The text projection surfaces the durable anchor and the repair-direction polarity after the
    // File line and before the Reaction (the cli-check-runner spec's text-report scenarios). Pins
    // the order so a future reshuffle can't silently move them into the reason-led opening.
    let report = Report::new(vec![
        violation(
            "crate::core",
            "must not import crate::adapter",
            "crate::adapter::Db",
            Some("src/core/mod.rs"),
        )
        .with_anchor(Some("ADR-014".to_string()))
        .with_polarity(Polarity::DenyBreach),
    ]);
    let text = violations_text(&report);
    let file = text.find("File:").expect("file");
    let anchor = text.find("Anchor:").expect("anchor");
    let repair = text.find("Repair:").expect("repair");
    let reaction = text.find("Reaction:").expect("reaction");
    assert!(
        file < anchor && anchor < repair && repair < reaction,
        "order must be … file → anchor → repair → reaction:\n{text}"
    );
    assert!(text.contains("Anchor:\n  ADR-014"), "anchor shown:\n{text}");
    assert!(
        text.contains("Repair:\n  deny_breach"),
        "repair polarity shown as the boundary's repair direction:\n{text}"
    );
}

#[test]
fn report_text_omits_anchor_and_repair_when_absent() {
    // An anchor-less, polarity-less violation (e.g. an audit-coverage violation) shows neither line
    // rather than an empty or "none" one — the same faithful-absence shape as the File element.
    let report = Report::new(vec![violation("crate::x", "rule", "finding", None)]);
    let text = violations_text(&report);
    assert!(
        !text.contains("Anchor:"),
        "no anchor line when the violation carries none:\n{text}"
    );
    assert!(
        !text.contains("Repair:"),
        "no repair line when the violation carries no polarity:\n{text}"
    );
}

#[test]
fn report_text_groups_violations_by_boundary() {
    // Input order is intentionally unsorted; the text groups by (target, rule).
    let report = Report::new(vec![
        violation("z-crate", "r1", "f", None),
        violation("a-crate", "r1", "f", None),
        violation("a-crate", "r0", "f", None),
    ]);
    let text = violations_text(&report);
    assert!(
        text.find("Boundary:\n  a-crate").unwrap() < text.find("Boundary:\n  z-crate").unwrap(),
        "the a-crate group precedes z-crate:\n{text}"
    );
    assert!(
        text.find("\n  r0").unwrap() < text.find("\n  r1").unwrap(),
        "within a-crate, r0 precedes r1:\n{text}"
    );
}

#[test]
fn json_projection_is_unchanged_by_the_text_grouping() {
    // The text sort is presentation-only: the JSON keeps the input (detection) order.
    let outcome = Outcome::Violations(Report::new(vec![
        violation("z-crate", "r", "f", None),
        violation("a-crate", "r", "f", None),
    ]));
    let json = report_json(&outcome, &[], None);
    assert!(
        json.find("z-crate").unwrap() < json.find("a-crate").unwrap(),
        "JSON keeps input order (z before a), unaffected by the text grouping:\n{json}"
    );
}

#[test]
fn sarif_projects_violations_with_file_level_locations_and_no_region() {
    let outcome = Outcome::Violations(Report::new(vec![
        violation(
            "crate::core",
            "must not import crate::adapter",
            "crate::adapter::Db",
            Some("src/core/mod.rs"),
        ),
        violation("dep-crate", "deny external", "serde", None),
    ]));
    let doc: serde_json::Value =
        serde_json::from_str(&report_sarif(&outcome)).expect("valid SARIF JSON");
    assert_eq!(doc["version"], "2.1.0");
    assert_eq!(doc["runs"][0]["tool"]["driver"]["name"], "tianheng");
    let results = doc["runs"][0]["results"].as_array().expect("results array");
    assert_eq!(results.len(), 2, "one result per non-baselined violation");
    // With a file: error level, ruleId in place, file-level location with NO region.
    assert_eq!(results[0]["level"], "error");
    assert_eq!(results[0]["ruleId"], "must not import crate::adapter");
    assert!(
        results[0]["message"]["text"]
            .as_str()
            .unwrap()
            .contains("reason-for-crate::core")
    );
    assert_eq!(
        results[0]["locations"][0]["physicalLocation"]["artifactLocation"]["uri"],
        "src/core/mod.rs"
    );
    assert!(
        results[0]["locations"][0]["physicalLocation"]["region"].is_null(),
        "no region — the line is not observed, never fabricated"
    );
    // File-less violation: no locations at all.
    assert!(
        results[1]["locations"].is_null(),
        "a file-less violation projects no location"
    );
}

#[test]
fn semantic_violation_projects_its_file_in_json_and_sarif() {
    // Every semantic violation now carries a file — a single-module one by its governed
    // module, a whole-crate-scan one (trait-impl-locality / forbidden-marker) by the
    // offending element's module. A crate-dependency violation is the genuinely file-less
    // case. All project faithfully.
    let single_module = Violation::new(
        BoundaryKind::Semantic,
        test_id(
            "crate::domain",
            "must not expose",
            "crate::infra::Db exposed by fn crate::domain::leak",
        ),
        "must not expose",
        "crate::infra::Db exposed by fn crate::domain::leak",
        "domain must not expose infra".to_string(),
        Severity::Enforce,
    )
    .with_file(Some("src/domain.rs".to_string()));
    let whole_crate_scan = Violation::new(
        BoundaryKind::Semantic,
        test_id(
            "crate::Command",
            "must be implemented only in the allowed locations",
            "crate::plugins (impl for crate::plugins::P)",
        ),
        "must be implemented only in the allowed locations",
        "crate::plugins (impl for crate::plugins::P)",
        "Command impls live in crate::allowed".to_string(),
        Severity::Enforce,
    )
    .with_file(Some("src/plugins.rs".to_string()));
    let file_less = Violation::new(
        BoundaryKind::Crate,
        test_id("dep-crate", "deny external", "serde"),
        "deny external",
        "serde",
        "core must stay dependency-light".to_string(),
        Severity::Enforce,
    );
    let outcome = Outcome::Violations(Report::new(vec![
        single_module,
        whole_crate_scan,
        file_less,
    ]));

    // JSON: both semantic violations name their file; the crate-dependency one is null.
    let json: serde_json::Value =
        serde_json::from_str(&report_json(&outcome, &[], None)).expect("valid JSON");
    assert_eq!(json["violations"][0]["file"], "src/domain.rs");
    assert_eq!(json["violations"][1]["file"], "src/plugins.rs");
    assert!(
        json["violations"][2]["file"].is_null(),
        "a crate-dependency violation has no single source file"
    );

    // SARIF: the file-bearing ones get file-level locations (no region); the null one none.
    let sarif: serde_json::Value =
        serde_json::from_str(&report_sarif(&outcome)).expect("valid SARIF");
    let results = sarif["runs"][0]["results"]
        .as_array()
        .expect("results array");
    assert_eq!(
        results[0]["locations"][0]["physicalLocation"]["artifactLocation"]["uri"],
        "src/domain.rs"
    );
    assert_eq!(
        results[1]["locations"][0]["physicalLocation"]["artifactLocation"]["uri"],
        "src/plugins.rs"
    );
    assert!(
        results[0]["locations"][0]["physicalLocation"]["region"].is_null(),
        "no region — the line is not observed for a semantic violation either"
    );
    assert!(
        results[2]["locations"].is_null(),
        "a file-less violation projects no SARIF location"
    );
}

#[test]
fn sarif_clean_is_empty_and_constitution_error_marks_execution_unsuccessful() {
    let clean: serde_json::Value = serde_json::from_str(&report_sarif(&Outcome::Clean)).unwrap();
    assert!(
        clean["runs"][0]["results"].as_array().unwrap().is_empty(),
        "clean → empty results"
    );
    let err: serde_json::Value =
        serde_json::from_str(&report_sarif(&Outcome::ConstitutionError("bad law".into()))).unwrap();
    assert_eq!(
        err["runs"][0]["invocations"][0]["executionSuccessful"],
        serde_json::Value::Bool(false),
        "a constitution error marks the invocation unsuccessful (required by SARIF)"
    );
    assert!(
        err["runs"][0]["invocations"][0]["toolExecutionNotifications"][0]["message"]["text"]
            .as_str()
            .unwrap()
            .contains("bad law")
    );
}

#[test]
fn sarif_exits_like_json() {
    // Fixture-driven — skip in a packaged `.crate` where fixtures are absent (see
    // `an_orphan_probe_reacts_with_no_declared_boundary` / `workspace_manifest`).
    if workspace_manifest().is_none() {
        return;
    }
    // Presentation only: the same outcome exits identically under each machine format.
    for format in ["json", "sarif"] {
        assert_eq!(
            run_args(&[
                "tianheng",
                "check",
                "--manifest-path",
                &fixture("violating"),
                "--format",
                format,
            ]),
            1,
            "violating fixture exits 1 under --format {format}"
        );
        assert_eq!(
            run_args(&[
                "tianheng",
                "check",
                "--manifest-path",
                &fixture("clean"),
                "--format",
                format,
            ]),
            0,
            "clean fixture exits 0 under --format {format}"
        );
    }
}

#[test]
fn list_rejects_the_check_only_sarif_format() {
    // SARIF projects the reaction, not the law — check-only, like markdown is list-only.
    assert_eq!(run_args(&["tianheng", "list", "--format", "sarif"]), 2);
}

#[test]
fn list_markdown_empty_constitution_has_a_title_but_no_sections() {
    // An empty dimension adds no section, mirroring the text and JSON projections.
    let md = list_markdown(&list_document(&Constitution::new("empty")));
    assert!(md.contains("# Constitution: empty"), "{md}");
    assert!(
        !md.contains("\n## "),
        "no dimension sections expected:\n{md}"
    );
}

#[test]
fn list_accepts_markdown_format() {
    // `list --format markdown` is a pure projection: it observes no workspace and exits 0.
    assert_eq!(run_args(&["tianheng", "list", "--format", "markdown"]), 0);
    assert_eq!(run_args(&["tianheng", "list", "--format=markdown"]), 0);
}

#[test]
fn check_rejects_the_list_only_markdown_format() {
    // markdown is a list-only projection of the declared law; check's machine output is the
    // JSON report, so check --format markdown is a usage error (exit 2), not a silent fallback.
    assert_eq!(
        run_args(&[
            "tianheng",
            "check",
            "--manifest-path",
            &fixture("clean"),
            "--format",
            "markdown",
        ]),
        2
    );
}

#[test]
fn the_runtime_projection_distinguishes_posture() {
    // A `.panic_on_violation()` boundary must NOT project identically to a default event-only
    // one — posture is part of the declared law, and the projection is faithful.
    let event = Constitution::new("c").runtime(
        RuntimeBoundary::at("s")
            .only_origins(["app::a"])
            .because("default event"),
    );
    let panicking = Constitution::new("c").runtime(
        RuntimeBoundary::at("s")
            .only_origins(["app::a"])
            .panic_on_violation()
            .because("opt-in panic"),
    );
    let ej = list_document(&event)["runtime_boundaries"][0].clone();
    let pj = list_document(&panicking)["runtime_boundaries"][0].clone();
    assert_eq!(ej["posture"], "event", "default posture is event: {ej}");
    assert_eq!(pj["posture"], "panic", "opt-in posture is panic: {pj}");
    assert_ne!(ej, pj, "posture must make the two projections differ");
    assert!(
        runtime_text(panicking.runtime_boundaries()).contains("posture: panic"),
        "the text projection names the posture too"
    );
}

#[test]
fn both_baseline_flags_exit_2() {
    // The contradictory pair is a pure usage error (exit 2), and its check runs BEFORE manifest
    // resolution — so with an also-absent `--manifest-path` the flag conflict is still what gets
    // reported, not a masking "no Cargo.toml found" diagnostic.
    assert_eq!(
        run_args(&[
            "tianheng",
            "check",
            "--manifest-path",
            &fixture("clean"),
            "--baseline",
            "a.json",
            "--write-baseline",
            "b.json",
        ]),
        2
    );
}

#[test]
fn unknown_format_exits_2() {
    assert_eq!(
        run_args(&[
            "tianheng",
            "check",
            "--manifest-path",
            &fixture("clean"),
            "--format",
            "yaml",
        ]),
        2
    );
}

#[test]
fn flag_missing_its_value_is_a_usage_error() {
    // The foot-gun: a value-taking flag with no following token must fail loud
    // (exit 2), not silently downgrade (--format -> text and exit 0, --baseline
    // / --write-baseline -> a plain check). The trailing flag errors during
    // parsing, before any workspace is observed, so no fixture is needed.
    for flag in [
        "--manifest-path",
        "--baseline",
        "--write-baseline",
        "--format",
    ] {
        assert_eq!(
            run_args(&[
                "tianheng",
                "check",
                "--manifest-path",
                &fixture("clean"),
                flag
            ]),
            2,
            "{flag} without a value must exit 2",
        );
    }
}

#[test]
fn list_needs_no_manifest_path_and_exits_0() {
    assert_eq!(run_args(&["tianheng", "list"]), 0);
}

#[test]
fn list_json_exits_0() {
    assert_eq!(run_args(&["tianheng", "list", "--format", "json"]), 0);
}

#[test]
fn list_unknown_format_is_a_usage_error() {
    assert_eq!(run_args(&["tianheng", "list", "--format", "yaml"]), 2);
}

#[test]
fn misspelled_flag_fails_loud_instead_of_being_ignored() {
    // The foot-gun: a typo'd --write-baseline must not silently run a plain
    // check (and write no baseline).
    assert_eq!(
        run_args(&[
            "tianheng",
            "check",
            "--manifest-path",
            &fixture("violating"),
            "--write-baselin",
            "out.json",
        ]),
        2
    );
}

#[test]
fn unknown_flag_exits_2() {
    assert_eq!(
        run_args(&[
            "tianheng",
            "check",
            "--manifest-path",
            &fixture("clean"),
            "--frobnicate",
        ]),
        2
    );
}

#[test]
fn stray_positional_exits_2() {
    assert_eq!(
        run_args(&[
            "tianheng",
            "check",
            "stray",
            "--manifest-path",
            &fixture("clean")
        ]),
        2
    );
}

#[test]
fn list_unknown_flag_exits_2() {
    assert_eq!(run_args(&["tianheng", "list", "--bogus"]), 2);
}

#[test]
fn list_rejects_check_only_flags() {
    // `list` observes no workspace, so a check-only flag is a usage error (exit 2),
    // never a silent no-op. Each is rejected during parsing/dispatch, no fixture.
    for args in [
        &["tianheng", "list", "--manifest-path", "Cargo.toml"][..],
        &["tianheng", "list", "--baseline", "b.json"][..],
        &["tianheng", "list", "--write-baseline", "b.json"][..],
        &["tianheng", "list", "--warn-uncovered"][..],
    ] {
        assert_eq!(
            run_args(args),
            2,
            "a check-only flag supplied to list must exit 2: {args:?}",
        );
    }
}

#[test]
fn the_coverage_advisory_names_each_uncovered_crate_only_under_the_flag() {
    // The advisory content itself (the other half of the flag's contract): without the flag
    // only the one-line summary prints; with it, every uncovered crate is named and each
    // block states the reaction is a warning that never fails CI. Asserting the text guards
    // against the flag silently going inert — a green exit code alone would not catch that.
    let coverage = Coverage {
        total: 3,
        uncovered: vec!["alpha".to_string(), "beta".to_string()],
    };
    let quiet = coverage_report(&coverage, false);
    assert!(
        quiet.contains("2 of 3 workspace crate(s) have no boundary"),
        "the summary line always prints: {quiet}"
    );
    assert!(
        !quiet.contains("Tianheng advisory"),
        "no per-crate advisory without the flag: {quiet}"
    );
    let loud = coverage_report(&coverage, true);
    assert!(loud.contains("Uncovered crate:\n  alpha"), "{loud}");
    assert!(loud.contains("Uncovered crate:\n  beta"), "{loud}");
    assert_eq!(
        loud.matches("warning only — CI not failed.").count(),
        2,
        "one warning-only advisory per uncovered crate: {loud}"
    );
    // A fully-covered workspace reports the all-clear, never an advisory — regardless of flag.
    let covered = Coverage {
        total: 3,
        uncovered: vec![],
    };
    let all_clear = coverage_report(&covered, true);
    assert!(
        all_clear.contains("all 3 workspace crate(s) have a boundary"),
        "{all_clear}"
    );
    assert!(
        !all_clear.contains("Tianheng advisory"),
        "a covered workspace emits no advisory even under the flag: {all_clear}"
    );
}

#[test]
fn warn_uncovered_never_changes_the_exit_code() {
    // Fixture-driven — skip in a packaged `.crate` where fixtures are absent (see
    // `an_orphan_probe_reacts_with_no_declared_boundary` / `workspace_manifest`).
    if workspace_manifest().is_none() {
        return;
    }
    // Coverage is an observation, not a reaction. An empty constitution leaves the `clean`
    // fixture's one member (`example-core`) uncovered, yet the run is clean — so with OR
    // without `--warn-uncovered` the exit stays 0. The flag prints a per-crate advisory to
    // stderr; it never turns an uncovered crate into a CI failure (that would be a silent
    // policy the DSL never declared). A non-zero here would mean coverage had leaked into
    // the exit code.
    let clean = fixture("clean");
    let with = [
        "tianheng",
        "check",
        "--manifest-path",
        clean.as_str(),
        "--warn-uncovered",
    ];
    let without = ["tianheng", "check", "--manifest-path", clean.as_str()];
    assert_eq!(
        dispatch(&Constitution::new("empty"), with),
        0,
        "an uncovered-but-clean workspace stays exit 0 under --warn-uncovered (advisory only)",
    );
    assert_eq!(
        dispatch(&Constitution::new("empty"), without),
        0,
        "…and without the flag too: coverage never decides the exit code",
    );
}

#[test]
fn nearest_manifest_walks_up_to_the_nearest_cargo_toml() {
    // `check` defaults its target to the nearest `Cargo.toml`, cargo-style. Drive the pure
    // ascent over a real temp tree so the walk is proven without touching the process cwd.
    let root = TempPath::new(
        std::env::temp_dir().join(format!("tianheng-nearest-{}", std::process::id())),
    );
    let root = root.path();
    let outer = root.join("outer");
    let inner = outer.join("inner");
    let leaf = inner.join("a").join("b");
    std::fs::create_dir_all(&leaf).expect("mkdir leaf");
    std::fs::write(outer.join("Cargo.toml"), "[workspace]\n").expect("write outer manifest");

    // From a deep leaf with a single manifest above it, the walk finds that manifest.
    assert_eq!(
        nearest_manifest_from(leaf.clone()),
        Some(outer.join("Cargo.toml")),
        "the ascent finds the one Cargo.toml above the leaf",
    );

    // With a second, nearer manifest, the walk stops at the *nearest* — it does not climb
    // past the first hit to the outer one.
    std::fs::write(inner.join("Cargo.toml"), "[workspace]\n").expect("write inner manifest");
    assert_eq!(
        nearest_manifest_from(leaf.clone()),
        Some(inner.join("Cargo.toml")),
        "the nearest manifest wins over a farther ancestor",
    );

    // A directory that already holds a Cargo.toml resolves to itself, not an ancestor.
    assert_eq!(
        nearest_manifest_from(inner.clone()),
        Some(inner.join("Cargo.toml")),
        "the start dir counts as its own nearest manifest",
    );
}

#[test]
fn write_baseline_preserves_hand_added_metadata_across_regeneration() {
    // The metadata-preserving merge, driven through the real write path + a temp file: write a
    // baseline, hand-annotate an entry, re-write from the same report — the annotation survives.
    let path = TempPath::new(std::env::temp_dir().join(format!(
        "tianheng-baseline-merge-{}.json",
        std::process::id()
    )));
    let path = path.path();
    let path_str = path.to_str().expect("utf-8 temp path");

    let outcome = Outcome::Violations(Report::new(vec![violation("core", "rule", "serde", None)]));

    // First write: no metadata yet.
    assert_eq!(super::write_baseline(&outcome, path_str), 0);
    let first = std::fs::read_to_string(path).expect("baseline written");
    let first_doc: Value = serde_json::from_str(&first).unwrap();
    assert_eq!(first_doc["format"], "tianheng.baseline/structured-facts");
    assert!(first_doc.get("version").is_none());
    assert!(first_doc["violations"][0]["fact"].is_object());
    assert!(
        !first.contains("owner"),
        "fresh baseline has no metadata: {first}"
    );

    // Hand-annotate the entry (as a maintainer would), then re-write from the same report.
    let annotated = first.replace(
        "\"finding\": \"serde\"",
        "\"finding\": \"serde\",\n      \"owner\": \"team-core\",\n      \"tracker\": \"ISSUE-7\"",
    );
    std::fs::write(path, &annotated).expect("annotate");
    assert_eq!(super::write_baseline(&outcome, path_str), 0);

    // The re-written baseline still carries the hand-added owner/tracker (merged by identity).
    let rewritten = std::fs::read_to_string(path).expect("baseline rewritten");
    let doc: Value = serde_json::from_str(&rewritten).expect("valid baseline json");
    assert_eq!(doc["violations"][0]["owner"], "team-core");
    assert_eq!(doc["violations"][0]["tracker"], "ISSUE-7");
}

#[test]
fn write_baseline_refuses_every_unsupported_existing_document_without_modifying_it() {
    let path = TempPath::new(std::env::temp_dir().join(format!(
        "tianheng-baseline-v1-upgrade-{}.json",
        std::process::id()
    )));
    let path = path.path();
    let path_str = path.to_str().expect("utf-8 temp path");
    let outcome = Outcome::Violations(Report::new(vec![violation("core", "rule", "serde", None)]));
    for unsupported in [
        r#"{"version":1,"violations":[]}"#,
        r#"{"version":2,"violations":[]}"#,
        r#"{"violations":[]}"#,
        r#"{"format":"tianheng.baseline/unknown","violations":[]}"#,
        "{ malformed",
    ] {
        std::fs::write(path, unsupported).expect("write unsupported baseline");
        assert_eq!(super::write_baseline(&outcome, path_str), 2);
        assert_eq!(
            std::fs::read_to_string(path).unwrap(),
            unsupported,
            "refusal must preserve the existing file byte-for-byte"
        );
    }
}

#[test]
fn projection_gate_reacts_to_missing_stale_and_regenerates_on_bless() {
    // Pass `bless` as a bool (the helper reads no environment), so this test mutates no
    // process-global state and cannot race the parallel self-law gate.
    let dir =
        TempPath::new(std::env::temp_dir().join(format!("tianheng-gate-{}", std::process::id())));
    // A not-yet-existing subdir, so bless must `create_dir_all` the parent.
    let path = dir.path().join("sub").join("law.md");
    let hint = "BLESS=1 cargo test";

    // Missing file, no bless → Err naming both the path and the regenerate hint.
    let err = projection_gate("live", &path, hint, false).unwrap_err();
    assert!(
        err.contains("law.md") && err.contains(hint),
        "missing must name path + hint: {err}"
    );

    // Bless → creates the parent dir, writes, Ok.
    projection_gate("live", &path, hint, true).expect("bless writes");
    assert_eq!(std::fs::read_to_string(&path).unwrap(), "live");

    // Fresh (equal, no bless) → Ok.
    projection_gate("live", &path, hint, false).expect("fresh passes");

    // Stale (differs, no bless) → Err naming path + hint.
    let err = projection_gate("different", &path, hint, false).unwrap_err();
    assert!(
        err.contains("law.md") && err.contains(hint) && err.contains("stale"),
        "stale must name path + hint: {err}"
    );
}
