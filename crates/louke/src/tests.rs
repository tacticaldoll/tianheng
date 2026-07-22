use std::any::TypeId;
use std::collections::HashMap;

use xuanji::{BoundaryKind, Severity};

use crate::dsl::Posture;
use crate::registry::{OriginInfo, Registry, Seam, check_crossing};
use crate::tracked::TidMap;
use crate::{RuntimeBoundary, runtime_seam_rule_line};

// Build a registry directly (the pure core needs no globals — so tests never touch the
// process-global write-once REGISTRY/SINK and can run in parallel).
fn registry(
    seams: &[(&'static str, &[&'static str], Severity)],
    origins: &[(TypeId, &'static str, &'static str)],
) -> Registry {
    let mut s = HashMap::new();
    for (seam, allowed, severity) in seams {
        s.insert(
            *seam,
            Seam {
                allowed: allowed.to_vec(),
                reason: "r".to_string(),
                severity: *severity,
                posture: Posture::Event,
                anchor: None,
            },
        );
    }
    let mut o: TidMap<OriginInfo> = TidMap::default();
    for (tid, origin, name) in origins {
        o.insert(
            *tid,
            OriginInfo {
                origin,
                type_name: name,
            },
        );
    }
    Registry {
        origins: o,
        seams: s,
    }
}

struct Domain;
struct Infra;
struct Unrelated;

#[test]
fn an_allowed_origin_passes() {
    let reg = registry(
        &[("seam", &["app::domain"], Severity::Enforce)],
        &[(TypeId::of::<Domain>(), "app::domain", "Domain")],
    );
    assert!(
        check_crossing("seam", TypeId::of::<Domain>(), &reg)
            .unwrap()
            .is_none()
    );
}

#[test]
fn a_disallowed_origin_reacts() {
    let reg = registry(
        &[("seam", &["app::domain"], Severity::Enforce)],
        &[(TypeId::of::<Infra>(), "app::infra", "Infra")],
    );
    let (v, _posture) = check_crossing("seam", TypeId::of::<Infra>(), &reg)
        .unwrap()
        .unwrap();
    assert_eq!(v.kind, BoundaryKind::Runtime);
    assert_eq!(v.target, "seam");
    assert_eq!(v.rule, runtime_seam_rule_line(&["app::domain"]));
    let id = v.id();
    let key = id
        .finding_key()
        .expect("a production violation has structured identity");
    let rule = id
        .rule_key()
        .expect("a production violation has a structured rule");
    assert_eq!(rule.rule_type(), "tianheng.rule/louke/runtime-seam");
    assert_eq!(
        rule.fields().collect::<Vec<_>>(),
        vec![("allowed_origin_0", "app::domain")]
    );
    assert_eq!(key.fact_type(), "tianheng.fact/louke/runtime-crossing");
    assert_eq!(key.shape(), "registered-origin");
    assert_eq!(
        key.fields().collect::<Vec<_>>(),
        vec![("origin", "app::infra"), ("type_name", "Infra")]
    );
    assert!(v.finding.contains("app::infra"));
    // This is the prod default-sink violation (emitted via `to_json`). An origin-assertion
    // violation names an origin, not a source file, so its `file` is `None` and the
    // emitted JSON carries `file: null` — the additive, non-breaking effect of the shared
    // `to_json` gaining a `file` key, asserted here on the default-sink path.
    assert!(
        v.file.is_none(),
        "an origin-assertion violation has no source file"
    );
    assert!(
        v.to_json()["file"].is_null(),
        "the prod default-sink JSON carries file: null"
    );
}

#[test]
fn an_unknown_origin_reacts_fail_closed() {
    let reg = registry(&[("seam", &["app::domain"], Severity::Enforce)], &[]);
    let (v, _posture) = check_crossing("seam", TypeId::of::<Infra>(), &reg)
        .unwrap()
        .unwrap();
    assert!(v.finding.contains("<unregistered origin>"), "{}", v.finding);
}

#[test]
fn the_runtime_rule_line_is_shared_by_reaction_and_projection() {
    // The folded `… (only origins: …)` wording lives once in `runtime_seam_rule_line`; the prod
    // reaction (`check_crossing`) and the shell's text projection both call it, so the two
    // human-readable renderings cannot drift (the twin-drift bug class).
    assert_eq!(
        runtime_seam_rule_line(&["app::domain", "app::api"]),
        "only declared origins may cross the seam (only origins: app::domain, app::api)",
    );
    // The reaction's violation `rule` is exactly that formatter's output.
    let reg = registry(&[("seam", &["app::domain"], Severity::Enforce)], &[]);
    let (v, _) = check_crossing("seam", TypeId::of::<Infra>(), &reg)
        .unwrap()
        .unwrap();
    assert_eq!(v.rule, runtime_seam_rule_line(&["app::domain"]));
}

#[test]
fn distinct_unregistered_types_stay_distinct_findings() {
    // Two DIFFERENT unregistered types crossing the
    // same seam must not share one Violation identity — otherwise baselining one silently masks
    // the other's later crossing (a false negative). The TypeId discriminant keeps them distinct.
    let reg = registry(&[("seam", &["app::domain"], Severity::Enforce)], &[]);
    let a = check_crossing("seam", TypeId::of::<Infra>(), &reg)
        .unwrap()
        .unwrap()
        .0;
    let b = check_crossing("seam", TypeId::of::<Domain>(), &reg)
        .unwrap()
        .unwrap()
        .0;
    assert!(a.finding.contains("<unregistered origin>"));
    assert!(b.finding.contains("<unregistered origin>"));
    assert_ne!(
        a.id(),
        b.id(),
        "distinct unregistered types must have distinct Violation ids: {} vs {}",
        a.finding,
        b.finding
    );
}

#[test]
fn registered_crossing_identity_survives_registry_reorder_and_unrelated_insertion() {
    let first = registry(
        &[("seam", &["app::domain"], Severity::Enforce)],
        &[
            (TypeId::of::<Infra>(), "app::infra", "Infra"),
            (TypeId::of::<Domain>(), "app::domain", "Domain"),
        ],
    );
    let reordered = registry(
        &[("seam", &["app::domain"], Severity::Enforce)],
        &[
            (TypeId::of::<Unrelated>(), "app::other", "Unrelated"),
            (TypeId::of::<Domain>(), "app::domain", "Domain"),
            (TypeId::of::<Infra>(), "app::infra", "Infra"),
        ],
    );
    let first_id = check_crossing("seam", TypeId::of::<Infra>(), &first)
        .unwrap()
        .unwrap()
        .0
        .id()
        .clone();
    let reordered_id = check_crossing("seam", TypeId::of::<Infra>(), &reordered)
        .unwrap()
        .unwrap()
        .0
        .id()
        .clone();
    assert_eq!(first_id, reordered_id);
}

#[test]
fn an_undeclared_seam_is_a_constitution_error() {
    let reg = registry(&[], &[]);
    let err = check_crossing("ghost", TypeId::of::<Domain>(), &reg).unwrap_err();
    assert!(err.contains("undeclared runtime seam 'ghost'"), "{err}");
}

#[test]
fn the_builder_carries_posture_and_severity() {
    let b = RuntimeBoundary::at("s")
        .only_origins(["app::domain"])
        .panic_on_violation()
        .warn()
        .because("r");
    assert_eq!(b.seam(), "s");
    assert_eq!(b.allowed_origins(), &["app::domain"]);
}

#[test]
fn runtime_rule_identity_is_set_order_stable_and_policy_sensitive() {
    let left = RuntimeBoundary::at("seam-a")
        .only_origins(["app::domain", "app::api"])
        .because("first wording");
    let reordered = RuntimeBoundary::at("seam-b")
        .only_origins(["app::api", "app::domain", "app::domain"])
        .panic_on_violation()
        .warn()
        .because("different wording")
        .with_anchor("GOV-1");
    let expanded = RuntimeBoundary::at("seam-a")
        .only_origins(["app::domain", "app::api", "app::infra"])
        .because("first wording");

    assert_eq!(left.rule_key(), reordered.rule_key());
    assert_ne!(left.rule_key(), expanded.rule_key());
    assert_eq!(
        left.rule_key().fields().collect::<Vec<_>>(),
        vec![
            ("allowed_origin_0", "app::api"),
            ("allowed_origin_1", "app::domain"),
        ]
    );
}

#[test]
fn the_fold_hasher_distinguishes_types() {
    let mut m: TidMap<u8> = TidMap::default();
    m.insert(TypeId::of::<Domain>(), 1);
    m.insert(TypeId::of::<Infra>(), 2);
    assert_eq!(m.get(&TypeId::of::<Domain>()), Some(&1));
    assert_eq!(m.get(&TypeId::of::<Infra>()), Some(&2));
    assert_eq!(m.len(), 2);
}
