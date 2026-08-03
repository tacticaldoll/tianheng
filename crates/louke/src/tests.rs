use std::any::TypeId;
use std::collections::HashMap;

use xuanji::{BoundaryKind, Severity};

use crate::dsl::Posture;
use crate::registry::{
    OriginInfo, Registry, Seam, check_crossing, dropped_sink_events, emit_default,
};
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
    assert_eq!(v.target(), "seam");
    assert_eq!(v.rule, runtime_seam_rule_line(&["app::domain"]));
    let id = v.id();
    let key = id.fact();
    let rule = id.rule_key();
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
fn a_failed_default_sink_write_is_counted_not_silently_lost() {
    // A writer that always fails, so the counting path can be pinned without touching the real
    // process stderr (a `#[cfg(unix)]` integration test in `tests/` covers the real OS-level
    // broken-pipe path end to end).
    struct AlwaysFails;
    impl std::io::Write for AlwaysFails {
        fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
            let _ = buf;
            Err(std::io::Error::from(std::io::ErrorKind::BrokenPipe))
        }
        fn flush(&mut self) -> std::io::Result<()> {
            Ok(())
        }
    }

    let reg = registry(&[("seam", &["app::domain"], Severity::Enforce)], &[]);
    let (violation, _posture) = check_crossing("seam", TypeId::of::<Infra>(), &reg)
        .unwrap()
        .unwrap();

    // Delta, not an absolute value: this static is process-global across every test in this
    // binary, so a prior test's drop (there is none today) must not make this one flaky.
    let before = dropped_sink_events();
    emit_default(AlwaysFails, &violation);
    assert_eq!(
        dropped_sink_events(),
        before + 1,
        "a failed default-sink write must be counted exactly once, never silently discarded"
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

/// KNOWN, DEFERRED trust bound, pinned rather than only listed: an origin is **observed** for code
/// that goes through `register_origin!`, and merely **asserted** by code that does not.
///
/// A `macro_rules!` expands at its call site, so the constructor it names must stay reachable from
/// there — `pub(crate)` would break every legitimate `register_origin!` in an adopter's crate. It is
/// `#[doc(hidden)]` and named `__from_register_origin` so a hand-written call reads as the bypass it
/// is, but nothing in std can stop that call. 漏刻's trust boundary is therefore the process: it
/// catches architectural drift, not an in-process adversary.
///
/// This test exists so the gap cannot quietly change state in either direction. If it starts failing,
/// the bound has been closed — and `runtime-origin-assertion`'s requirement, the constructor's own
/// doc, and `BACKLOG.md`'s decision entry must be updated together with it.
///
/// The second assertion records the evidence a future fix could key on: the type's OWN path disagrees
/// with the asserted origin, and unlike a call-site string it is not the caller's to choose. Deriving
/// the origin from it would close this, at the cost of redefining an origin from "where registered" to
/// "where defined" and resting identity on `type_name`'s deliberately unspecified format — a design
/// decision, recorded with the proc-macro alternative in the backlog, not taken here.
#[test]
fn a_hand_built_origin_entry_is_accepted_a_known_trust_bound() {
    struct Rogue;
    let forged =
        crate::OriginEntry::__from_register_origin(TypeId::of::<Rogue>(), "app::blessed", "Rogue");
    assert_eq!(
        forged.origin, "app::blessed",
        "a hand-built entry's asserted origin is taken as given — the bound this test pins"
    );
    let real_path = std::any::type_name::<Rogue>();
    assert!(
        !real_path.starts_with("app::blessed"),
        "and the type's own path contradicts it ({real_path}), which is the evidence a future fix \
         could use to refuse the assertion"
    );
}
