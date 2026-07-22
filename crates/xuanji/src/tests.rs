use serde_json::Value;

use crate::{
    Baseline, BaselineEntry, BoundaryKind, Finding, FindingKey, Polarity, Report, RuleKey,
    Severity, StructuredFactIdentity, Violation, ViolationId,
};

fn test_finding(text: &str) -> Finding {
    Finding::new(
        text,
        FindingKey::new("test", "fact", [("value", text)]).unwrap(),
    )
}

fn test_id(target: &str, rule: &str, finding: &str) -> ViolationId {
    ViolationId::structured(
        target,
        rule,
        RuleKey::of("tianheng.rule/test/policy", [("policy", rule)]),
        test_finding(finding),
    )
}

#[test]
fn boundary_kind_labels_cover_every_dimension() {
    assert_eq!(BoundaryKind::Crate.as_str(), "crate");
    assert_eq!(BoundaryKind::Module.as_str(), "module");
    assert_eq!(BoundaryKind::Semantic.as_str(), "semantic");
    assert_eq!(BoundaryKind::Runtime.as_str(), "runtime");
}

#[test]
fn finding_key_validates_and_canonicalizes_its_envelope() {
    let key = FindingKey::new(
        "module",
        "forbidden_import",
        [("module", "crate::z"), ("importer", "crate::a")],
    )
    .unwrap();
    assert_eq!(
        key.fields().collect::<Vec<_>>(),
        vec![("importer", "crate::a"), ("module", "crate::z")]
    );
    assert!(FindingKey::new("", "fact", [("value", "x")]).is_err());
    assert!(FindingKey::new("module", "", [("value", "x")]).is_err());
    assert!(FindingKey::new("module", "fact", [("", "x")]).is_err());
    assert!(FindingKey::new("module", "fact", [("value", "x"), ("value", "y")]).is_err());
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
    let old = ViolationId::structured(
        "core",
        "old rule wording",
        rule.clone(),
        Finding::new("old finding wording", fact.clone()),
    );
    let new = ViolationId::structured(
        "core",
        "new rule wording",
        rule,
        Finding::new("new finding wording", fact),
    );
    let transitional = ViolationId::new(
        "core",
        "old rule wording",
        test_finding("old finding wording"),
    );

    assert_eq!(old, new, "presentation stays outside the typed algebra");
    assert_ne!(
        old, transitional,
        "migration provenances never compare equal"
    );
    assert_eq!(
        old.rule_key().unwrap().rule_type(),
        "tianheng.rule/test/deny-dependency"
    );
}

#[test]
fn structured_path_round_trips_through_the_temporary_baseline_bridge() {
    let violation = Violation::new(
        BoundaryKind::Crate,
        ViolationId::structured(
            "core",
            "deny dependency on serde",
            RuleKey::of(
                "tianheng.rule/test/deny-dependency",
                [("dependency", "serde")],
            ),
            Finding::new(
                "serde",
                StructuredFactIdentity::of(
                    "tianheng.fact/test/dependency",
                    "dependency-edge",
                    [("package", "serde")],
                ),
            ),
        ),
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
        "the kernel must not depend on a projection".to_string(),
        Severity::Enforce,
    )
}

fn wording_violation(text: &str) -> Violation {
    let key = FindingKey::new("test", "dependency", [("package", "serde")]).unwrap();
    Violation::new(
        BoundaryKind::Crate,
        ViolationId::structured(
            "core",
            "deny",
            RuleKey::of(
                "tianheng.rule/test/deny-dependency",
                std::iter::empty::<(&str, &str)>(),
            ),
            Finding::new(text, key),
        ),
        "reason".to_string(),
        Severity::Enforce,
    )
}

#[test]
fn to_json_emits_the_file_key_in_both_states() {
    let without = sample_violation();
    assert_eq!(without.to_json()["file"], Value::Null);
    let with = sample_violation().with_file(Some("src/kernel.rs".to_string()));
    assert_eq!(
        with.to_json()["file"],
        Value::String("src/kernel.rs".to_string())
    );
}

#[test]
fn file_is_not_part_of_the_baseline_identity() {
    let without = sample_violation();
    let with = sample_violation().with_file(Some("src/kernel.rs".to_string()));
    assert_eq!(without.id(), with.id());
}

#[test]
fn to_json_emits_the_anchor_key_in_both_states() {
    let without = sample_violation();
    assert_eq!(without.to_json()["anchor"], Value::Null);
    let with = sample_violation().with_anchor(Some("ADR-014".to_string()));
    assert_eq!(
        with.to_json()["anchor"],
        Value::String("ADR-014".to_string())
    );
}

#[test]
fn anchor_is_not_part_of_the_baseline_identity() {
    let without = sample_violation();
    let with = sample_violation().with_anchor(Some("ADR-014".to_string()));
    assert_eq!(without.id(), with.id());
}

#[test]
fn to_json_emits_the_polarity_key_in_both_states() {
    let without = sample_violation();
    assert_eq!(without.to_json()["polarity"], Value::Null);
    let deny = sample_violation().with_polarity(Polarity::DenyBreach);
    assert_eq!(
        deny.to_json()["polarity"],
        Value::String("deny_breach".to_string())
    );
    let allow = sample_violation().with_polarity(Polarity::AllowlistGap);
    assert_eq!(
        allow.to_json()["polarity"],
        Value::String("allowlist_gap".to_string())
    );
}

#[test]
fn polarity_is_not_part_of_the_baseline_identity() {
    let without = sample_violation();
    let with = sample_violation().with_polarity(Polarity::AllowlistGap);
    assert_eq!(without.id(), with.id());
}

#[test]
fn baseline_round_trips_through_json() {
    let report = Report::new(vec![
        sample_violation(),
        Violation::new(
            BoundaryKind::Crate,
            test_id("core", "deny external dependencies", "serde"),
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
    assert_eq!(entry.id.finding, "new wording");
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
        ViolationId::structured(
            "core",
            "deny",
            RuleKey::of(
                "tianheng.rule/test/deny-dependency",
                std::iter::empty::<(&str, &str)>(),
            ),
            Finding::new(
                "same wording",
                StructuredFactIdentity::of("test", "dependency", [("package", "tokio")]),
            ),
        ),
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
    assert_eq!(entries[0].id.finding, "first");
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
    assert_eq!(entries[0].id.target, "core");
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
            "x".to_string(),
            Severity::Enforce,
        )
    };
    let report = Report::new(vec![mk("core", "serde"), mk("new", "reqwest")]);
    let next = Baseline::of_preserving(&report, &previous);
    let entries: Vec<&BaselineEntry> = next.entries().collect();
    assert_eq!(entries.len(), 2);
    let core = entries.iter().find(|e| e.id.target == "core").unwrap();
    assert_eq!(core.owner.as_deref(), Some("team-core"));
    assert_eq!(core.tracker.as_deref(), Some("ISSUE-7"));
    let new = entries.iter().find(|e| e.id.target == "new").unwrap();
    assert_eq!(new.owner, None);
    assert!(
        entries.iter().all(|e| e.id.target != "gone"),
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
    assert_eq!(stale[0], &sample_violation().id());
}
