use serde_json::Value;
use std::path::Path;

use crate::{
    Baseline, BaselineEntry, BoundDecl, BoundId, BoundaryKind, Demonstrates, Extent,
    FactGranularity, Finding, Owner, Polarity, Reached, Report, RuleKey, ScanDepth, Severity,
    StructuredFactIdentity, Violation, ViolationId,
};

fn test_finding(text: &str) -> Finding {
    Finding::new(
        text,
        StructuredFactIdentity::new("test", "fact", [("value", text)]).unwrap(),
    )
}

fn test_id(target: &str, rule: &str, finding: &str) -> ViolationId {
    ViolationId::new(
        target,
        RuleKey::of("tianheng.rule/test/policy", [("policy", rule)]),
        test_finding(finding).fact().clone(),
    )
}

#[test]
fn finding_fact_accessor_keeps_the_key_compatibility_alias() {
    let finding = test_finding("observed");
    assert_eq!(finding.fact(), finding.key());
}

#[test]
fn boundary_kind_labels_cover_every_dimension() {
    assert_eq!(BoundaryKind::Crate.as_str(), "crate");
    assert_eq!(BoundaryKind::Module.as_str(), "module");
    assert_eq!(BoundaryKind::Semantic.as_str(), "semantic");
    assert_eq!(BoundaryKind::Runtime.as_str(), "runtime");
}

#[test]
fn scan_depth_labels_and_defaults_are_correct() {
    assert_eq!(ScanDepth::default(), ScanDepth::Shallow);
    assert!(ScanDepth::Shallow.is_shallow());
    assert!(!ScanDepth::Subtree.is_shallow());
    assert_eq!(ScanDepth::Shallow.as_str(), "shallow");
    assert_eq!(ScanDepth::Subtree.as_str(), "subtree");
}

#[test]
fn structured_fact_identity_validates_and_canonicalizes_its_envelope() {
    let key = StructuredFactIdentity::new(
        "module",
        "forbidden_import",
        [("module", "crate::z"), ("importer", "crate::a")],
    )
    .unwrap();
    assert_eq!(
        key.fields().collect::<Vec<_>>(),
        vec![("importer", "crate::a"), ("module", "crate::z")]
    );
    assert!(StructuredFactIdentity::new("", "fact", [("value", "x")]).is_err());
    assert!(StructuredFactIdentity::new("module", "", [("value", "x")]).is_err());
    assert!(StructuredFactIdentity::new("module", "fact", [("", "x")]).is_err());
    assert!(
        StructuredFactIdentity::new("module", "fact", [("value", "x"), ("value", "y")]).is_err()
    );
}

#[test]
fn semantic_identity_primitives_validate_and_canonicalize_scalar_fields() {
    let rule = RuleKey::new(
        "tianheng.rule/test/deny-dependency",
        [("target", "serde"), ("kind", "normal")],
    )
    .unwrap();
    assert_eq!(
        rule.fields().collect::<Vec<_>>(),
        vec![("kind", "normal"), ("target", "serde")]
    );
    assert_eq!(rule.rule_type(), "tianheng.rule/test/deny-dependency");

    let fact = StructuredFactIdentity::new(
        "tianheng.fact/test/dependency",
        "dependency-edge",
        [("package", "serde"), ("kind", "normal")],
    )
    .unwrap();
    assert_eq!(fact.fact_type(), "tianheng.fact/test/dependency");
    assert_eq!(fact.shape(), "dependency-edge");
    assert_eq!(
        fact.fields().collect::<Vec<_>>(),
        vec![("kind", "normal"), ("package", "serde")]
    );

    assert!(RuleKey::new("", [("value", "x")]).is_err());
    assert!(RuleKey::new("rule", [("", "x")]).is_err());
    assert!(RuleKey::new("rule", [("value", "x"), ("value", "y")]).is_err());
    assert!(StructuredFactIdentity::new("", "shape", [("value", "x")]).is_err());
    assert!(StructuredFactIdentity::new("fact", "", [("value", "x")]).is_err());
    assert!(StructuredFactIdentity::new("fact", "shape", [("", "x")]).is_err());
    assert!(
        StructuredFactIdentity::new("fact", "shape", [("value", "x"), ("value", "y")]).is_err()
    );
}

#[test]
fn structured_path_uses_target_rule_key_and_fact_only() {
    let rule = RuleKey::of(
        "tianheng.rule/test/deny-dependency",
        [("dependency", "serde")],
    );
    let fact = StructuredFactIdentity::of(
        "tianheng.fact/test/dependency",
        "dependency-edge",
        [("package", "serde")],
    );
    let old = ViolationId::new("core", rule.clone(), fact.clone());
    let new = ViolationId::new("core", rule, fact);

    assert_eq!(old, new, "presentation stays outside the typed algebra");
    assert_eq!(
        old.rule_key().rule_type(),
        "tianheng.rule/test/deny-dependency"
    );
    assert_eq!(
        old.to_json().to_string(),
        r#"{"fact":{"fields":{"package":"serde"},"shape":"dependency-edge","type":"tianheng.fact/test/dependency"},"rule_key":{"fields":{"dependency":"serde"},"type":"tianheng.rule/test/deny-dependency"},"target":"core"}"#,
        "the canonical identity serialization is a machine-contract input"
    );
}

#[test]
fn presentation_and_diagnostics_cannot_rekey_a_violation() {
    let id = ViolationId::new(
        "core",
        RuleKey::of("tianheng.rule/test/deny", [("policy", "external")]),
        StructuredFactIdentity::of(
            "tianheng.fact/test/dependency",
            "dependency-edge",
            [("package", "serde")],
        ),
    );
    let original = Violation::new(
        BoundaryKind::Crate,
        id.clone(),
        "old rule wording",
        "old finding wording",
        "old reason".to_string(),
        Severity::Warn,
    );
    let mut changed = Violation::new(
        BoundaryKind::Runtime,
        id,
        "new rule wording",
        "new finding wording and diagnostic signature",
        "new reason".to_string(),
        Severity::Enforce,
    )
    .with_file(Some("src/new.rs".to_string()))
    .with_anchor(Some("new-anchor".to_string()))
    .with_polarity(Polarity::AllowlistGap);
    changed.baselined = true;

    assert_ne!(original, changed, "diagnostic records really did change");
    assert_eq!(
        original.id(),
        changed.id(),
        "kind, wording, diagnostics, reason, severity, file, anchor, polarity, and baseline state stay outside identity"
    );

    let changed_fact = ViolationId::new(
        "core",
        original.rule_key().clone(),
        StructuredFactIdentity::of(
            "tianheng.fact/test/dependency",
            "dependency-edge",
            [("package", "tokio")],
        ),
    );
    assert_ne!(
        original.id(),
        changed_fact,
        "an identity scalar must re-key"
    );
}

#[test]
fn production_sources_have_no_presentation_derived_identity_bridge() {
    fn visit(path: &Path, offenders: &mut Vec<String>) {
        for entry in std::fs::read_dir(path).unwrap() {
            let path = entry.unwrap().path();
            if path.is_dir() {
                visit(&path, offenders);
            } else if path.extension().and_then(|value| value.to_str()) == Some("rs")
                && path.file_name().and_then(|value| value.to_str()) != Some("tests.rs")
                && !path.components().any(|part| part.as_os_str() == "tests")
            {
                let source = std::fs::read_to_string(&path).unwrap();
                let old_constructor = ["ViolationId::", "structured("].concat();
                let old_alias = ["Finding", "Key"].concat();
                if source.contains(&old_constructor) || source.contains(&old_alias) {
                    offenders.push(path.display().to_string());
                }
            }
        }
    }

    let workspace = Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
    let crates = workspace.join("crates");
    if !crates.exists() {
        assert!(
            std::env::var_os("TIANHENG_WORKSPACE_TESTS").is_none(),
            "workspace crates expected but absent while TIANHENG_WORKSPACE_TESTS is set — \
             the production-source reaction must not silently skip in CI"
        );
        return;
    }
    let mut offenders = Vec::new();
    visit(&crates, &mut offenders);
    assert!(
        offenders.is_empty(),
        "legacy identity construction remains in production sources: {offenders:?}"
    );
}

#[test]
fn structured_path_round_trips_through_baseline() {
    let fact = StructuredFactIdentity::of(
        "tianheng.fact/test/dependency",
        "dependency-edge",
        [("package", "serde")],
    );
    let violation = Violation::new(
        BoundaryKind::Crate,
        ViolationId::new(
            "core",
            RuleKey::of(
                "tianheng.rule/test/deny-dependency",
                [("dependency", "serde")],
            ),
            fact,
        ),
        "deny dependency on serde",
        "serde",
        "core stays independent".to_string(),
        Severity::Enforce,
    );
    let report = Report::new(vec![violation]);
    let baseline = Baseline::of(&report);
    let document: Value = serde_json::from_str(&baseline.to_json()).unwrap();
    assert_eq!(
        document["violations"][0]["rule_key"]["type"],
        "tianheng.rule/test/deny-dependency"
    );
    let reparsed = Baseline::from_json(&baseline.to_json()).unwrap();
    assert!(reparsed.contains(&report.violations[0]));
}

fn sample_violation() -> Violation {
    Violation::new(
        BoundaryKind::Module,
        test_id("crate::kernel", "must not import", "crate::projection"),
        "must not import",
        "crate::projection",
        "the kernel must not depend on a projection".to_string(),
        Severity::Enforce,
    )
}

fn wording_violation(text: &str) -> Violation {
    let key = StructuredFactIdentity::new("test", "dependency", [("package", "serde")]).unwrap();
    Violation::new(
        BoundaryKind::Crate,
        ViolationId::new(
            "core",
            RuleKey::of(
                "tianheng.rule/test/deny-dependency",
                std::iter::empty::<(&str, &str)>(),
            ),
            key,
        ),
        "deny",
        text,
        "reason".to_string(),
        Severity::Enforce,
    )
}

/// Assert that setting a violation's metadata field via `with_field` projects `expected` into
/// `to_json()[field]`, and that the field plays no role in the violation's baseline identity
/// (`id()` is unchanged by it) — the shared shape behind every metadata-only field (`file`,
/// `anchor`, `polarity`). Call once per variant for a multi-variant field like `polarity`, since
/// the identity-exclusion must hold for every value the field can take, not just one.
fn assert_metadata_field(field: &str, with_field: impl Fn(Violation) -> Violation, expected: &str) {
    let without = sample_violation();
    assert_eq!(without.to_json()[field], Value::Null);
    let with = with_field(sample_violation());
    assert_eq!(with.to_json()[field], Value::String(expected.to_string()));
    assert_eq!(without.id(), with.id());
}

#[test]
fn to_json_emits_the_file_key_in_both_states() {
    assert_metadata_field(
        "file",
        |v| v.with_file(Some("src/kernel.rs".to_string())),
        "src/kernel.rs",
    );
}

#[test]
fn file_is_not_part_of_the_baseline_identity() {
    assert_metadata_field(
        "file",
        |v| v.with_file(Some("src/kernel.rs".to_string())),
        "src/kernel.rs",
    );
}

#[test]
fn to_json_emits_the_anchor_key_in_both_states() {
    assert_metadata_field(
        "anchor",
        |v| v.with_anchor(Some("ADR-014".to_string())),
        "ADR-014",
    );
}

#[test]
fn anchor_is_not_part_of_the_baseline_identity() {
    assert_metadata_field(
        "anchor",
        |v| v.with_anchor(Some("ADR-014".to_string())),
        "ADR-014",
    );
}

#[test]
fn to_json_emits_the_polarity_key_in_both_states() {
    assert_metadata_field(
        "polarity",
        |v| v.with_polarity(Polarity::DenyBreach),
        "deny_breach",
    );
    assert_metadata_field(
        "polarity",
        |v| v.with_polarity(Polarity::AllowlistGap),
        "allowlist_gap",
    );
}

#[test]
fn polarity_is_not_part_of_the_baseline_identity() {
    assert_metadata_field(
        "polarity",
        |v| v.with_polarity(Polarity::AllowlistGap),
        "allowlist_gap",
    );
}

#[test]
fn baseline_round_trips_through_json() {
    let report = Report::new(vec![
        sample_violation(),
        Violation::new(
            BoundaryKind::Crate,
            test_id("core", "deny external dependencies", "serde"),
            "deny external dependencies",
            "serde",
            "core stays dependency-light".to_string(),
            Severity::Enforce,
        ),
    ]);
    let original = Baseline::of(&report);
    let document: Value = serde_json::from_str(&original.to_json()).unwrap();
    assert_eq!(document["format"], "tianheng.baseline/structured-facts");
    assert!(document.get("version").is_none());
    assert!(document["violations"][0]["rule_key"].is_object());
    assert!(document["violations"][0]["fact"].is_object());
    assert!(document["violations"][0]["fact"]["type"].is_string());
    assert!(document["violations"][0]["fact"]["shape"].is_string());
    let reparsed = Baseline::from_json(&original.to_json()).expect("round-trips");
    assert!(reparsed.contains(&sample_violation()));
    assert!(
        reparsed.stale(&report).is_empty(),
        "no entry is stale against its own report"
    );
    assert_eq!(reparsed.to_json(), original.to_json());
}

#[test]
fn semantic_baseline_matches_and_preserves_metadata_across_wording_changes() {
    let previous = Baseline::from_json(
        r#"{"format":"tianheng.baseline/structured-facts","violations":[{
            "target":"core","rule":"old rule wording","finding":"old finding wording",
            "rule_key":{"type":"tianheng.rule/test/deny-dependency","fields":{}},
            "fact":{"type":"test","shape":"dependency","fields":{"package":"serde"}},
            "owner":"team-core","tracker":"ISSUE-9"
        }]}"#,
    )
    .unwrap();
    let report = Report::new(vec![wording_violation("new wording")]);
    assert!(previous.contains(&report.violations[0]));
    let rewritten = Baseline::of_preserving(&report, &previous);
    let entry = rewritten.entries().next().unwrap();
    assert_eq!(entry.finding, "new wording");
    assert_eq!(entry.owner.as_deref(), Some("team-core"));
    assert_eq!(entry.tracker.as_deref(), Some("ISSUE-9"));
}

#[test]
fn equal_presentation_cannot_substitute_for_a_different_fact_identity() {
    let accepted = Baseline::from_json(
        r#"{"format":"tianheng.baseline/structured-facts","violations":[{
            "target":"core","rule":"deny","finding":"same wording",
            "rule_key":{"type":"tianheng.rule/test/deny-dependency","fields":{}},
            "fact":{"type":"test","shape":"dependency","fields":{"package":"serde"}}
        }]}"#,
    )
    .unwrap();
    let current = Violation::new(
        BoundaryKind::Crate,
        ViolationId::new(
            "core",
            RuleKey::of(
                "tianheng.rule/test/deny-dependency",
                std::iter::empty::<(&str, &str)>(),
            ),
            StructuredFactIdentity::of("test", "dependency", [("package", "tokio")]),
        ),
        "deny",
        "same wording",
        "r".to_string(),
        Severity::Enforce,
    );
    assert!(!accepted.contains(&current));
}

#[test]
fn semantic_baseline_deduplicates_by_identity_and_keeps_the_first_entry() {
    let baseline = Baseline::from_json(
        r#"{"format":"tianheng.baseline/structured-facts","violations":[
            {"target":"core","rule":"deny","finding":"first","owner":"first",
             "rule_key":{"type":"tianheng.rule/test/deny-dependency","fields":{}},
             "fact":{"type":"test","shape":"dependency","fields":{"package":"serde"}}},
            {"target":"core","rule":"changed wording","finding":"second","owner":"second",
             "rule_key":{"type":"tianheng.rule/test/deny-dependency","fields":{}},
             "fact":{"type":"test","shape":"dependency","fields":{"package":"serde"}}}
        ]}"#,
    )
    .unwrap();
    let entries: Vec<_> = baseline.entries().collect();
    assert_eq!(entries.len(), 1);
    assert_eq!(entries[0].finding, "first");
    assert_eq!(entries[0].owner.as_deref(), Some("first"));
}

#[test]
fn owner_and_tracker_round_trip_and_are_emitted_only_when_set() {
    let json = r#"{"format":"tianheng.baseline/structured-facts","violations":[
        {"target":"core","rule":"r","finding":"serde",
         "rule_key":{"type":"tianheng.rule/test/policy","fields":{"policy":"r"}},
         "fact":{"type":"test","shape":"fact","fields":{"value":"serde"}},
         "owner":"team-core","tracker":"ISSUE-7"},
        {"target":"zeta","rule":"r","finding":"tokio",
         "rule_key":{"type":"tianheng.rule/test/policy","fields":{"policy":"r"}},
         "fact":{"type":"test","shape":"fact","fields":{"value":"tokio"}}}
    ]}"#;
    let baseline = Baseline::from_json(json).expect("semantic annotations parse");
    let entries: Vec<&BaselineEntry> = baseline.entries().collect();
    assert_eq!(entries[0].id.target(), "core");
    assert_eq!(entries[0].owner.as_deref(), Some("team-core"));
    assert_eq!(entries[0].tracker.as_deref(), Some("ISSUE-7"));
    assert_eq!(entries[1].owner, None);
    assert_eq!(entries[1].tracker, None);
    let out = baseline.to_json();
    assert_eq!(Baseline::from_json(&out).unwrap().to_json(), out);
    let doc: Value = serde_json::from_str(&out).unwrap();
    let zeta = &doc["violations"][1];
    assert_eq!(zeta["target"], "zeta");
    assert!(zeta.get("owner").is_none() && zeta.get("tracker").is_none());
}

#[test]
fn optional_baseline_metadata_accepts_only_absent_null_or_string() {
    let entry = serde_json::json!({
        "target": "core",
        "rule": "deny",
        "finding": "serde",
        "rule_key": {"type": "tianheng.rule/test/deny", "fields": {}},
        "fact": {"type": "tianheng.fact/test/dependency", "shape": "edge", "fields": {"package": "serde"}},
    });

    for field in ["owner", "tracker"] {
        let absent = serde_json::json!({
            "format": "tianheng.baseline/structured-facts",
            "violations": [entry.clone()]
        });
        let parsed = Baseline::from_json(&absent.to_string()).expect("omission is absence");
        assert_eq!(parsed.entries().next().unwrap().owner.as_deref(), None);
        assert_eq!(parsed.entries().next().unwrap().tracker.as_deref(), None);

        let mut null_entry = entry.clone();
        null_entry[field] = Value::Null;
        let parsed = Baseline::from_json(
            &serde_json::json!({
                "format": "tianheng.baseline/structured-facts",
                "violations": [null_entry]
            })
            .to_string(),
        )
        .expect("explicit null is absence");
        let serialized: Value = serde_json::from_str(&parsed.to_json()).unwrap();
        assert!(
            serialized["violations"][0].get(field).is_none(),
            "explicit-null {field} stays canonical omission"
        );

        let mut string_entry = entry.clone();
        string_entry[field] = serde_json::json!("recorded");
        let parsed = Baseline::from_json(
            &serde_json::json!({
                "format": "tianheng.baseline/structured-facts",
                "violations": [string_entry]
            })
            .to_string(),
        )
        .expect("string metadata parses");
        let parsed_entry = parsed.entries().next().unwrap();
        let actual = match field {
            "owner" => parsed_entry.owner.as_deref(),
            "tracker" => parsed_entry.tracker.as_deref(),
            _ => unreachable!(),
        };
        assert_eq!(actual, Some("recorded"));

        for wrong in [
            serde_json::json!(7),
            serde_json::json!(true),
            serde_json::json!(["team-core"]),
            serde_json::json!({"name": "team-core"}),
        ] {
            let mut wrong_entry = entry.clone();
            wrong_entry[field] = wrong;
            let error = Baseline::from_json(
                &serde_json::json!({
                    "format": "tianheng.baseline/structured-facts",
                    "violations": [wrong_entry]
                })
                .to_string(),
            )
            .expect_err("wrong-typed metadata must invalidate the baseline");
            assert!(
                error.contains(field),
                "error must identify {field}: {error}"
            );
        }
    }
}

#[test]
fn of_preserving_carries_surviving_metadata_drops_stale_and_none_for_new() {
    let previous = Baseline::from_json(
        r#"{"format":"tianheng.baseline/structured-facts","violations":[
            {"target":"core","rule":"old wording","finding":"old finding","owner":"team-core","tracker":"ISSUE-7",
             "rule_key":{"type":"tianheng.rule/test/policy","fields":{"policy":"r"}},
             "fact":{"type":"test","shape":"fact","fields":{"value":"serde"}}},
            {"target":"gone","rule":"r","finding":"old","owner":"team-x",
             "rule_key":{"type":"tianheng.rule/test/policy","fields":{"policy":"r"}},
             "fact":{"type":"test","shape":"fact","fields":{"value":"old"}}}
        ]}"#,
    )
    .unwrap();
    let mk = |t: &str, f: &str| {
        Violation::new(
            BoundaryKind::Crate,
            test_id(t, "r", f),
            "r",
            f,
            "x".to_string(),
            Severity::Enforce,
        )
    };
    let report = Report::new(vec![mk("core", "serde"), mk("new", "reqwest")]);
    let next = Baseline::of_preserving(&report, &previous);
    let entries: Vec<&BaselineEntry> = next.entries().collect();
    assert_eq!(entries.len(), 2);
    let core = entries.iter().find(|e| e.id.target() == "core").unwrap();
    assert_eq!(core.owner.as_deref(), Some("team-core"));
    assert_eq!(core.tracker.as_deref(), Some("ISSUE-7"));
    let new = entries.iter().find(|e| e.id.target() == "new").unwrap();
    assert_eq!(new.owner, None);
    assert!(
        entries.iter().all(|e| e.id.target() != "gone"),
        "a resolved violation's entry (and metadata) drops"
    );
}

#[test]
fn unsupported_or_malformed_baseline_formats_fail_loud() {
    assert!(
        Baseline::from_json("{ not json").is_err(),
        "malformed JSON is an error"
    );
    assert!(
        Baseline::from_json(r#"{"version": 1, "violations": []}"#).is_err(),
        "numeric v1 is unsupported"
    );
    assert!(
        Baseline::from_json(r#"{"version": 2, "violations": []}"#).is_err(),
        "numeric v2 is unsupported"
    );
    assert!(
        Baseline::from_json(r#"{"violations": []}"#).is_err(),
        "an unmarked document is unsupported"
    );
    assert!(
        Baseline::from_json(r#"{"format":"tianheng.baseline/other","violations":[]}"#).is_err(),
        "an unknown semantic format is unsupported"
    );
    assert!(
        Baseline::from_json(
            r#"{"format":"tianheng.baseline/structured-facts","violations":"none"}"#
        )
        .is_err(),
        "wrong-typed violations are malformed"
    );

    for malformed in [
        serde_json::json!({
            "target": "core", "rule": "deny", "finding": "serde",
            "fact": {"type": "test", "shape": "edge", "fields": {}}
        }),
        serde_json::json!({
            "target": "core", "rule": "deny", "finding": "serde",
            "rule_key": {"type": "tianheng.rule/test/deny", "fields": {}},
            "fact": {"type": "test", "fields": {}}
        }),
        serde_json::json!({
            "target": "core", "rule": "deny", "finding": "serde",
            "rule_key": {"type": "tianheng.rule/test/deny", "fields": {"mode": 7}},
            "fact": {"type": "test", "shape": "edge", "fields": {}}
        }),
        serde_json::json!({
            "target": "core", "rule": "deny", "finding": "serde",
            "rule_key": {"type": "tianheng.rule/test/deny", "fields": {}},
            "fact": {"type": "test", "shape": "edge", "fields": {"package": ["serde"]}}
        }),
    ] {
        let document = serde_json::json!({
            "format": "tianheng.baseline/structured-facts",
            "violations": [malformed]
        });
        assert!(
            Baseline::from_json(&document.to_string()).is_err(),
            "malformed structured entry must fail: {document}"
        );
    }
}

#[test]
fn a_fixed_violation_leaves_a_stale_baseline_entry() {
    let baseline = Baseline::of(&Report::new(vec![sample_violation()]));
    let stale = baseline.stale(&Report::empty());
    assert_eq!(
        stale.len(),
        1,
        "the fixed violation's entry is reported stale"
    );
    assert_eq!(stale[0].id, sample_violation().id());
    assert_eq!(stale[0].rule, "must not import");
    assert_eq!(stale[0].finding, "crate::projection");
}

// --- the declared-observation-bound model ---
//
// What is deliberately NOT tested here, recorded rather than silently skipped: "an out-of-reach bound cannot
// claim an owner" and "granularity is carried only by the as-intended extent" have no tests, because the code
// expressing either would not compile. A test that has to name a field the type does not offer is the wrong
// proof; the nesting is the proof.

fn out_of_reach_bound() -> BoundDecl {
    BoundDecl::pinned(
        BoundId::new(
            "external-crate-confinement/a-confined-crate-use-inside-a-string-is-a-stated-bound",
        ),
        "a `use` inside a string literal or macro body",
        Extent::OutOfReach {
            because: "comments, string literals and macro bodies are stripped before scanning"
                .into(),
        },
        "a_confined_use_inside_a_string_or_macro_body_is_not_observed",
    )
}

#[test]
fn every_extent_derives_what_its_pinning_test_must_demonstrate() {
    // Derived, never declared: a direction beside the extent would be a second copy of one fact, and two
    // copies can disagree. Each pairing below is the one the specs' own defences take.
    let cases = [
        (
            Extent::OutOfReach {
                because: "stripped before scanning".into(),
            },
            Demonstrates::DoesNotReact,
        ),
        (
            Extent::Reached(Reached::RefusesToJudge {
                because: "the source file cannot be located".into(),
            }),
            Demonstrates::RefusesToJudge,
        ),
        (
            Extent::Reached(Reached::DeclinesToRefuse {
                because: "a cfg-gated module's file is absent".into(),
            }),
            Demonstrates::DoesNotRefuse,
        ),
        (
            Extent::Reached(Reached::OverReacts {
                because: "the rule governs the declared source kind".into(),
            }),
            Demonstrates::ReactsOnHarmlessShape,
        ),
        (
            Extent::Reached(Reached::UnderReacts {
                because: "the use-map reads `use` only".into(),
                owner: Owner::Adopter,
            }),
            Demonstrates::DoesNotReact,
        ),
        (
            Extent::Reached(Reached::NotAViolation {
                because: "`as _` binds no nameable path".into(),
            }),
            Demonstrates::DoesNotReact,
        ),
        (
            Extent::Reached(Reached::AsIntended {
                bounded: FactGranularity::Identity,
                because: "the sub-node cannot be rendered without macro expansion".into(),
            }),
            Demonstrates::CollapsesGranularity,
        ),
    ];

    for (extent, expected) in cases {
        assert_eq!(
            extent.demonstrates(),
            expected,
            "{} must be defended by a test that {}",
            extent.as_str(),
            expected.as_str()
        );
    }
}

#[test]
fn only_an_under_reaction_is_a_declared_false_negative() {
    // The projection leads with this count, so the predicate is what decides whether a reader sees a bound as
    // the family's audit backlog or as governed conservatism. An over-reaction is NOT one: it costs a false
    // positive, which is the safe direction.
    assert!(
        Extent::Reached(Reached::UnderReacts {
            because: "only crate-root renames are collected".into(),
            owner: Owner::Engine,
        })
        .is_declared_false_negative()
    );

    for safe in [
        Extent::OutOfReach {
            because: "foreign AST is not scanned".into(),
        },
        Extent::Reached(Reached::RefusesToJudge {
            because: "no verdict is possible".into(),
        }),
        Extent::Reached(Reached::DeclinesToRefuse {
            because: "skipping beats erroring".into(),
        }),
        Extent::Reached(Reached::OverReacts {
            because: "fail-closed on a composite shape".into(),
        }),
        Extent::Reached(Reached::AsIntended {
            bounded: FactGranularity::Presentation,
            because: "a lifetime carries no architectural intent".into(),
        }),
    ] {
        assert!(
            !safe.is_declared_false_negative(),
            "{} is not a declared false negative",
            safe.as_str()
        );
    }
}

#[test]
fn a_declaration_reports_the_id_shape_extent_and_pin_it_was_given() {
    let bound = out_of_reach_bound();
    assert_eq!(
        bound.id().as_str(),
        "external-crate-confinement/a-confined-crate-use-inside-a-string-is-a-stated-bound"
    );
    assert_eq!(
        bound.shape(),
        "a `use` inside a string literal or macro body"
    );
    assert_eq!(bound.extent().as_str(), "out of reach");
    assert_eq!(bound.extent().demonstrates(), Demonstrates::DoesNotReact);
    assert_eq!(
        bound
            .defence()
            .pinning_tests()
            .expect("the declaration is pinned")
            .collect::<Vec<_>>(),
        ["a_confined_use_inside_a_string_or_macro_body_is_not_observed"]
    );
    // The id renders as itself, so a diagnostic can name a bound without a lookup table.
    assert_eq!(bound.id().to_string(), bound.id().as_str());
}

#[test]
fn every_projection_label_is_distinct_within_its_enum() {
    // Two values sharing a label would collapse in the projection while remaining distinct in code, so a
    // reader would see one group where two exist. Asserted as a set, so adding a value with a duplicate label
    // fails rather than merging silently.
    let extents = [
        Extent::OutOfReach { because: "".into() }.as_str(),
        Reached::RefusesToJudge { because: "".into() }.as_str(),
        Reached::DeclinesToRefuse { because: "".into() }.as_str(),
        Reached::OverReacts { because: "".into() }.as_str(),
        Reached::UnderReacts {
            because: "".into(),
            owner: Owner::Engine,
        }
        .as_str(),
        Reached::AsIntended {
            bounded: FactGranularity::Identity,
            because: "".into(),
        }
        .as_str(),
    ];
    let unique: std::collections::BTreeSet<_> = extents.iter().collect();
    assert_eq!(
        unique.len(),
        extents.len(),
        "extent labels must be distinct"
    );

    let owners = [
        Owner::Engine.as_str(),
        Owner::Inherited { from: "".into() }.as_str(),
        Owner::Adopter.as_str(),
    ];
    assert_eq!(
        owners
            .iter()
            .collect::<std::collections::BTreeSet<_>>()
            .len(),
        owners.len(),
        "owner labels must be distinct"
    );

    let demos = [
        Demonstrates::DoesNotReact.as_str(),
        Demonstrates::ReactsOnHarmlessShape.as_str(),
        Demonstrates::RefusesToJudge.as_str(),
        Demonstrates::DoesNotRefuse.as_str(),
        Demonstrates::CollapsesGranularity.as_str(),
    ];
    assert_eq!(
        demos
            .iter()
            .collect::<std::collections::BTreeSet<_>>()
            .len(),
        demos.len(),
        "demonstration labels must be distinct"
    );
}

#[test]
fn a_bound_may_be_declared_from_computed_strings() {
    // What the `&'static str` form forbade outright. `Observer::bounds` carries no default body, so declaring
    // bounds is a condition of implementing the protocol — and an implementor whose bounds are *discovered*
    // rather than written was mandated to declare limits it had no way to name. This test is the whole reason the
    // strings are owned-or-borrowed; without the change it does not compile, which is the honest negative run for
    // a type-level guard.
    let discovered = "plugin-42";
    let declaration = BoundDecl::pinned(
        BoundId::new(format!(
            "third-party-observer/{discovered}-is-not-scanned-a-stated-bound"
        )),
        format!("a shape only {discovered} exhibits"),
        Extent::OutOfReach {
            because: format!(
                "{discovered} was discovered at run time, so no literal could name it"
            )
            .into(),
        },
        format!("a_{}_bound_is_pinned", discovered.replace('-', "_")),
    );

    assert_eq!(
        declaration.id().as_str(),
        "third-party-observer/plugin-42-is-not-scanned-a-stated-bound"
    );
    assert_eq!(
        declaration
            .defence()
            .pinning_tests()
            .expect("the declaration is pinned")
            .collect::<Vec<_>>(),
        ["a_plugin_42_bound_is_pinned"]
    );
    // And it behaves exactly as a literal declaration does: the extent still decides the predicted evidence, and
    // an out-of-reach bound is still not a declared false negative.
    assert_eq!(
        declaration.extent().demonstrates(),
        Demonstrates::DoesNotReact
    );
    assert!(!declaration.extent().is_declared_false_negative());
}

#[test]
fn a_literal_declaration_borrows_every_string_it_carries() {
    // The string-borrowing claim is asserted rather than intended: every one of this family's declarations is a
    // literal, and `observation_bounds()` runs on every pass of the reaction that holds them against the specs.
    // Pointer identity observes the shape directly — a borrowed value still points into the literal, while an
    // owned one cannot — and `borrows_every_string()` reaches the other string positions.
    //
    // How many there are is deliberately not written here. It was, as "fifty-three", while the register counted
    // fifty-four — a census in prose with no observation source, which `crates/jiaochou/tests/bound_register.rs` prints on every
    // clean run precisely so a number like that is read rather than remembered.
    const SHAPE: &str = "a shape whose pointer identity is what this test reads";
    let declaration = BoundDecl::pinned(
        BoundId::new("probe-capability/a-literal-declaration-a-stated-bound"),
        SHAPE,
        Extent::OutOfReach {
            because: "a literal rationale".into(),
        },
        "a_literal_declaration_borrows_every_string_it_carries",
    );
    assert!(
        std::ptr::eq(declaration.shape().as_ptr(), SHAPE.as_ptr()),
        "a declaration written from a literal must borrow it, not allocate a copy"
    );
    assert!(
        declaration.borrows_every_string(),
        "and it says so of itself, which is what the reaction over the family's declarations reads"
    );
}

#[test]
fn an_unpinned_declaration_carries_a_tracker_and_no_test() {
    let declaration = BoundDecl::unpinned(
        BoundId::new("probe-capability/an-unpinned-declaration-a-stated-bound"),
        "a shape whose missing defence is explicit",
        Extent::OutOfReach {
            because: "the source does not reach the fixture".into(),
        },
        "BACKLOG.md READY-PATCH missing-defence",
    );

    assert!(declaration.defence().pinning_tests().is_none());
    assert_eq!(
        declaration.defence().tracker(),
        Some("BACKLOG.md READY-PATCH missing-defence")
    );
    assert!(declaration.borrows_every_string());
}

#[test]
fn a_pinned_declaration_carries_every_test_and_cannot_be_empty() {
    let declaration = BoundDecl::pinned_by_many(
        BoundId::new("probe-capability/a-multiply-pinned-declaration-a-stated-bound"),
        "two fixture shapes sharing one declared bound",
        Extent::OutOfReach {
            because: "both shapes stop at the same observation edge".into(),
        },
        "the_first_shape_is_pinned",
        ["the_second_shape_is_pinned"],
    );

    assert_eq!(
        declaration
            .defence()
            .pinning_tests()
            .expect("the declaration is pinned")
            .collect::<Vec<_>>(),
        ["the_first_shape_is_pinned", "the_second_shape_is_pinned"]
    );
    assert_eq!(declaration.defence().tracker(), None);
}

/// The discriminant answers **`false`** for a computed string in **each position, independently**.
///
/// Without this the reaction over the family's declarations would be untestable in the direction that matters. A
/// discriminant that returned a constant `true` would satisfy every family declaration; and one written as a
/// single short-circuiting `&&` chain — which it is — can pass while examining only its first operand. So each
/// position is perturbed on its own, with every other string left literal, and the answer must be `false` for all
/// of them.
///
/// The positions include both defence variants: the test that pins a bound and the tracker owning an unpinned
/// bound, as well as its id, shape, extent rationale, and inherited ownership layer.
#[test]
fn a_computed_string_in_any_position_is_not_a_borrowing_declaration() {
    let literal_id = || BoundId::new("probe-capability/a-computed-declaration-a-stated-bound");
    let computed = || format!("built at {}", 1 + 1);

    let owned_id = BoundDecl::pinned(
        BoundId::new(computed()),
        "a literal shape",
        Extent::OutOfReach {
            because: "a literal rationale".into(),
        },
        "a_pin",
    );
    assert!(!owned_id.borrows_every_string(), "the id is computed");

    let owned_shape = BoundDecl::pinned(
        literal_id(),
        computed(),
        Extent::OutOfReach {
            because: "a literal rationale".into(),
        },
        "a_pin",
    );
    assert!(!owned_shape.borrows_every_string(), "the shape is computed");

    let owned_pin = BoundDecl::pinned(
        literal_id(),
        "a literal shape",
        Extent::OutOfReach {
            because: "a literal rationale".into(),
        },
        computed(),
    );
    assert!(!owned_pin.borrows_every_string(), "the pin is computed");

    let owned_additional_pin = BoundDecl::pinned_by_many(
        literal_id(),
        "a literal shape",
        Extent::OutOfReach {
            because: "a literal rationale".into(),
        },
        "a_literal_pin",
        [computed()],
    );
    assert!(
        !owned_additional_pin.borrows_every_string(),
        "an additional pin is computed"
    );

    let owned_tracker = BoundDecl::unpinned(
        literal_id(),
        "a literal shape",
        Extent::OutOfReach {
            because: "a literal rationale".into(),
        },
        computed(),
    );
    assert!(
        !owned_tracker.borrows_every_string(),
        "the unpinned tracker is computed"
    );

    let owned_rationale = BoundDecl::pinned(
        literal_id(),
        "a literal shape",
        Extent::OutOfReach {
            because: computed().into(),
        },
        "a_pin",
    );
    assert!(
        !owned_rationale.borrows_every_string(),
        "the extent's rationale is computed"
    );

    // The deepest string a declaration carries: nested two levels, inside the one extent that names an owner.
    let owned_layer = BoundDecl::pinned(
        literal_id(),
        "a literal shape",
        Extent::Reached(Reached::UnderReacts {
            because: "a literal rationale".into(),
            owner: Owner::Inherited {
                from: computed().into(),
            },
        }),
        "a_pin",
    );
    assert!(
        !owned_layer.borrows_every_string(),
        "the inherited layer name is computed"
    );

    // And the control: the same nested shape, all literal, must still answer `true` — otherwise the assertions
    // above would hold for the wrong reason, an extent this deep simply always reading as owned.
    let all_literal = BoundDecl::pinned(
        literal_id(),
        "a literal shape",
        Extent::Reached(Reached::UnderReacts {
            because: "a literal rationale".into(),
            owner: Owner::Inherited {
                from: "a literal layer".into(),
            },
        }),
        "a_pin",
    );
    assert!(
        all_literal.borrows_every_string(),
        "a fully literal declaration borrows every string, however deeply nested"
    );
}
