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
/// A module-level generic, so two instantiations can be compared without a fn-local path.
struct Wrapper<T>(#[allow(dead_code)] T);

/// Two layers, so the generic-argument bound can be pinned with real types rather than a rendered
/// string: a generic defined in one module, instantiated with a type defined in another.
mod blessed_layer {
    pub struct Wrapper<T>(#[allow(dead_code)] pub T);
}
mod rogue_layer {
    pub struct Payload;
}

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

/// The closure, asserted in the direction that matters: the only entry constructible for a type is
/// the honest one. Its predecessor in this file reproduced the false negative it replaces — a rogue
/// type registered under an allowlisted origin crossed a seam with no reaction — and stopped
/// **compiling** the moment the constructor stopped accepting an origin. That is the transition; this
/// test is what holds the ground it took.
///
/// The first assertion is the whole change's load-bearing claim, machine-checked rather than argued:
/// for a type registered inside its own module the derived origin equals `module_path!()` there, so the
/// documented idiom's `only_origins(...)` entries are byte-identical to what they were. If a toolchain
/// ever renders a type's path differently, this fails here rather than in an adopter's allowlist.
#[test]
fn a_registration_can_only_name_its_own_types_defining_module() {
    let entry = crate::OriginEntry::__from_register_origin::<Domain>();
    assert_eq!(
        entry.origin,
        module_path!(),
        "a type registered in its own module derives exactly the module path that module reports — \
         the byte-identical claim the migration rests on"
    );
    assert_eq!(entry.type_id, TypeId::of::<Domain>());

    // And the consequence end to end: nothing can present this type under another origin, so a seam
    // allowing only that other origin reacts instead of passing it.
    let reg = registry(
        &[("domain-entry", &["app::blessed"], Severity::Enforce)],
        &[(entry.type_id, entry.origin, entry.type_name)],
    );
    let reaction = check_crossing("domain-entry", TypeId::of::<Domain>(), &reg)
        .expect("the seam is declared, so this is not a constitution error")
        .expect("a type whose derived origin is not on the allowlist must react");
    assert!(
        reaction.0.finding.contains(module_path!()),
        "the finding names the origin actually observed, so an adopter reads which value to allow: \
         {}",
        reaction.0.finding
    );
}

/// The derivation's shape bounds, pinned as *relationships* rather than as rustc's exact rendering —
/// except the one that must be exact, which the test above owns. `runtime-origin-assertion` states
/// each of these; this is where they stop being prose.
#[test]
fn the_derived_origin_honors_its_stated_shape_bounds() {
    use crate::dsl::defining_module;

    // A generic type's arguments are not part of its origin, and an argument containing path
    // separators must not be mistaken for the type's own path — the reason the argument list is cut
    // before the final separator is sought, not after.
    assert_eq!(defining_module("app::infra::Repo<u8>"), "app::infra");
    assert_eq!(
        defining_module("app::infra::Repo<std::string::String>"),
        "app::infra",
        "the final `::` inside the ARGUMENTS must never be taken for the type's own"
    );
    assert_eq!(
        defining_module("app::infra::Repo<a::B<c::D>>"),
        "app::infra",
        "nesting inside the argument list changes nothing: the first `<` is the top-level one"
    );

    // Two instantiations of one generic type share one origin (same defining module).
    assert_eq!(
        defining_module(std::any::type_name::<Wrapper<u8>>()),
        defining_module(std::any::type_name::<Wrapper<String>>()),
    );

    // The generic bound in the direction that matters, with real types: a generic DEFINED in one
    // module carries that module's origin whatever it wraps, so an argument from another module does
    // not taint it. Stated in `runtime-origin-assertion` as the bound of observing an origin as a
    // module — governing which instantiations may cross is a different capability. Pinned here so it
    // cannot change state silently in either direction.
    let outer = defining_module(std::any::type_name::<
        blessed_layer::Wrapper<rogue_layer::Payload>,
    >());
    assert_eq!(
        outer,
        defining_module(std::any::type_name::<blessed_layer::Wrapper<u8>>()),
        "the argument does not change the origin"
    );
    assert!(
        outer.ends_with("blessed_layer") && !outer.contains("rogue_layer"),
        "the origin is the outermost type's defining module: {outer}"
    );

    // A shape with no path at all yields its own rendering — stated, not an error, because it matches
    // no allowlist entry and therefore reacts fail-closed.
    assert_eq!(defining_module("&u8"), "&u8");
    assert_eq!(defining_module("(u8, u8)"), "(u8, u8)");

    // A foreign type's origin is its OWN defining path, not the registering layer's. Asserted
    // structurally: it is a real path, and it is not this module — pinning std's internal rendering
    // would break on a toolchain that reorganizes it, which is not what this bound is about.
    let foreign = defining_module(std::any::type_name::<std::collections::HashMap<u8, u8>>());
    assert!(
        foreign.contains("::") && foreign != module_path!(),
        "a foreign type does not inherit the registering layer's origin: {foreign}"
    );

    // A function-local type's path is qualified by its enclosing function, so it is not a module path.
    fn enclosing() -> &'static str {
        struct Local;
        defining_module(std::any::type_name::<Local>())
    }
    assert_ne!(
        enclosing(),
        module_path!(),
        "a fn-local type's derived origin is fn-qualified — a stated bound, reacting fail-closed"
    );
}

/// The tests above pin the *behaviour*; this one pins the **claim**, because that is what drifted
/// hardest. It has now been through both states, and its shape follows the ground truth rather than
/// the other way round.
///
/// While the gap was open it required the process-trust-boundary prose to be **present** on every
/// surface describing the guarantee, and the absolute form ("origin is observed, NOT self-asserted")
/// to be absent — that absolute form having outlived its own correction in four places at once. The
/// gap is now closed, so the requirement inverts: an origin is derived from the type and cannot be
/// asserted at all, so a surface still promising a *bound* promises a limit the crate no longer has.
/// Understating a guarantee is a smaller sin than overstating one, but it is the same sin: the summary
/// a reader meets first is the capability contract.
///
/// Both directions are asserted, whitespace-flattened so a line wrap cannot hide either, and the
/// needles are assembled from fragments so this test's own source never contains them.
#[test]
fn the_origin_guarantee_is_stated_as_derived_on_every_surface() {
    fn flattened(path: &std::path::Path) -> String {
        let text = std::fs::read_to_string(path)
            .unwrap_or_else(|err| panic!("cannot read {}: {err}", path.display()));
        text.split_whitespace().collect::<Vec<_>>().join(" ")
    }

    fn assert_retired_absent(label: &str, text: &str, retired: &[String]) {
        for needle in retired {
            assert!(
                !text.contains(needle.as_str()),
                "{label} still states a retired form of the origin guarantee ({needle:?}) — an \
                 origin is derived from the type, so neither an absolute claim nor a cooperative \
                 bound describes it"
            );
        }
    }

    let crate_dir = std::path::Path::new(env!("CARGO_MANIFEST_DIR"));
    // Every retired wording in one list: the absolute claim the 0.4.0 window removed, and the two
    // forms of the cooperative bound this change removes.
    let retired = [
        ["self", "-asserted"].concat(),
        ["assert", "able"].concat(),
        ["trust boundary is the ", "process"].concat(),
    ];
    let stated = "derived from the type";

    // Crate-local sources: present in the published tarball too, so always checked. The tree is walked
    // rather than enumerated, so a file added later cannot reintroduce a retired wording unobserved
    // (this test's own needles are assembled from fragments, so it does not self-trip).
    fn forbid_under(dir: &std::path::Path, retired: &[String]) {
        for entry in std::fs::read_dir(dir).expect("louke's own source directory is readable") {
            let path = entry.expect("a readable source dir entry").path();
            if path.is_dir() {
                forbid_under(&path, retired);
            } else if path.extension().and_then(|ext| ext.to_str()) == Some("rs")
                // This file names the retired wordings, above, to say what they were; a test's own doc
                // is not a surface an adopter reads the guarantee from.
                && path.file_name().and_then(|name| name.to_str()) != Some("tests.rs")
            {
                let text = std::fs::read_to_string(&path)
                    .unwrap_or_else(|err| panic!("cannot read {}: {err}", path.display()));
                let flat = text.split_whitespace().collect::<Vec<_>>().join(" ");
                assert_retired_absent(&path.display().to_string(), &flat, retired);
            }
        }
    }
    forbid_under(&crate_dir.join("src"), &retired);

    // The surfaces an adopter meets the guarantee on must state what it now is. A deletion that
    // removed the statement would otherwise pass a forbid-only guard.
    for relative in ["README.md", "src/dsl.rs", "src/lib.rs"] {
        let text = flattened(&crate_dir.join(relative));
        assert_retired_absent(relative, &text, &retired);
        assert!(
            text.contains(stated),
            "{relative} does not state that an origin is derived from the type — the summary a \
             reader meets first is the capability contract"
        );
    }

    // Workspace surfaces: absent from the packaged tarball (where this test still runs), so skipped
    // there — but never silently in CI, where the workspace must be present.
    //
    // `PROJECT.md` is in the set because its Core Contract states the same claim one level up, which
    // is how an independent review reached this gap from that document rather than from the
    // specification. The root `README.md` and `COOKBOOK.md` are in it because they carry the
    // `register_origin!` samples and **nothing compiles them**: `ReadmeDoctests` includes
    // `crates/tianheng/README.md`, which does not mention the macro, and the root README cannot join
    // that net — a `#[cfg(doctest)]` include of a file outside the crate would break `cargo test` from
    // the published tarball, where that file does not exist. So this guard is the only reaction those
    // samples can have, which is a weaker thing than compiling them and is stated as such rather than
    // left to look like coverage.
    for relative in [
        "../../openspec/specs/runtime-origin-assertion/spec.md",
        "../../PROJECT.md",
        "../../README.md",
        "../../COOKBOOK.md",
    ] {
        let path = crate_dir.join(relative);
        if !path.exists() {
            assert!(
                std::env::var_os("TIANHENG_WORKSPACE_TESTS").is_none(),
                "{relative} is absent while TIANHENG_WORKSPACE_TESTS is set — the claim guard must \
                 not silently skip an authoritative surface in CI"
            );
            continue;
        }
        let text = flattened(&path);
        assert_retired_absent(relative, &text, &retired);
        assert!(
            text.contains(stated),
            "{relative} does not state that an origin is derived from the type"
        );
    }
}
