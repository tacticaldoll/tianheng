use std::any::TypeId;
use std::collections::HashMap;
use std::sync::OnceLock;

use xuanji::{BoundaryKind, Polarity, Severity, Violation, ViolationId};

use crate::dsl::{OriginEntry, Posture, RuntimeBoundary};
use crate::finding;
use crate::runtime_seam_rule_line;
use crate::tracked::TidMap;

// --- Private internals -------------------------------------------------------

pub(crate) struct Seam {
    pub(crate) allowed: Vec<&'static str>,
    pub(crate) reason: String,
    pub(crate) severity: Severity,
    pub(crate) posture: Posture,
    pub(crate) anchor: Option<String>,
}

pub(crate) struct OriginInfo {
    pub(crate) origin: &'static str,
    pub(crate) type_name: &'static str,
}

pub(crate) struct Registry {
    pub(crate) origins: TidMap<OriginInfo>,
    pub(crate) seams: HashMap<&'static str, Seam>,
}

static REGISTRY: OnceLock<Registry> = OnceLock::new();

// --- Public API --------------------------------------------------------------

/// Install the runtime constitution **once at startup**: the declared boundaries and the
/// origin registrations ([`crate::register_origin!`]). The registry is **write-once** so the probe
/// hot path reads it without a lock; calling `install` a second time is a constitution error
/// (it fails loud rather than silently replacing the law).
pub fn install<B, O>(boundaries: B, origins: O)
where
    B: IntoIterator<Item = RuntimeBoundary>,
    O: IntoIterator<Item = OriginEntry>,
{
    let mut seams = HashMap::new();
    for b in boundaries {
        // A seam declared twice is an observable misconfiguration: a silent overwrite would let
        // the last declaration shadow the earlier law (a declared boundary that never enforces —
        // the one bug this tool forbids). Fail loud, like a constitution error.
        if seams.contains_key(b.seam) {
            panic!(
                "louke: runtime seam '{}' declared more than once — each seam is declared exactly \
                 once (a duplicate would silently shadow the earlier boundary)",
                b.seam
            );
        }
        seams.insert(
            b.seam,
            Seam {
                allowed: b.allowed,
                reason: b.reason,
                severity: b.severity,
                posture: b.posture,
                anchor: b.anchor,
            },
        );
    }
    let mut origin_map: TidMap<OriginInfo> = TidMap::default();
    for e in origins {
        // Same type registered twice (e.g. two `register_origin!` sites) would silently keep the
        // last origin — fail loud rather than let an observed origin be silently replaced.
        if origin_map.contains_key(&e.type_id) {
            panic!(
                "louke: an origin for type '{}' was registered more than once — each type \
                 registers its origin exactly once",
                e.type_name
            );
        }
        origin_map.insert(
            e.type_id,
            OriginInfo {
                origin: e.origin,
                type_name: e.type_name,
            },
        );
    }
    if REGISTRY
        .set(Registry {
            origins: origin_map,
            seams,
        })
        .is_err()
    {
        panic!("louke: install called twice — the runtime constitution is write-once");
    }
}

// --- The pure core ----------------------------------------------------------

/// The pure reaction, testable without process-global state: resolve the seam (an undeclared
/// seam is a constitution error), resolve the crossing type's origin (an unregistered type
/// has none), and match the allowlist **fail-closed** — an origin not in the allowlist, or
/// an unknown origin, reacts. Returns the `Violation` to react with **and the seam's posture**
/// (already resolved here, so the caller need not look the seam up a second time), or `None` when
/// clean.
pub(crate) fn check_crossing(
    seam: &str,
    type_id: TypeId,
    registry: &Registry,
) -> Result<Option<(Violation, Posture)>, String> {
    let s = registry.seams.get(seam).ok_or_else(|| {
        // Reason-led, aligned with the CI-audit twin: name the intent (an undeclared seam is
        // never enforced) before the mechanics. Keeps the `undeclared runtime seam '{seam}'`
        // substring the prod contract and tests depend on.
        format!(
            "an undeclared seam is never enforced — declare the RuntimeBoundary or fix the \
                 probe's seam name: probe references undeclared runtime seam '{seam}'"
        )
    })?;

    let info = registry.origins.get(&type_id);
    // Fail-closed: a type that never registered an origin is not in the allowlist.
    let allowed = info.is_some_and(|i| s.allowed.contains(&i.origin));
    if allowed {
        return Ok(None);
    }

    let finding = match info {
        Some(i) => finding::RuntimeFact::RegisteredCrossing {
            origin: i.origin.to_string(),
            type_name: i.type_name.to_string(),
        },
        // An unregistered type recorded neither an origin nor a name, so the only per-type datum a
        // crossing carries is its `TypeId` (from `Any`; a concrete type NAME is unrecoverable from a
        // `&dyn Any` on stable Rust). Append it so two DISTINCT unregistered types crossing the same
        // seam produce distinct structured identities — otherwise baselining one
        // silently masks the other (a false negative). The `<unregistered origin>` prefix is kept so
        // the substring the prod contract/tests depend on still holds. The Debug bytes become the
        // published version-2 `type_id` key field: they are identity wire and stable only within a
        // build (a hash of the type's identity), not a promised cross-toolchain type name.
        None => finding::RuntimeFact::UnregisteredCrossing {
            type_id: format!("{type_id:?}"),
        },
    };
    // The rule family is the canonical `RUNTIME_SEAM_RULE` label; the allowed-origin set is the
    // per-boundary detail appended here, so the prod reaction and the `list` projection share one
    // rule label (the shell's `runtime` projection references the same const).
    let rule = runtime_seam_rule_line(&s.allowed);
    Ok(Some((
        Violation::new(
            BoundaryKind::Runtime,
            ViolationId::new(seam, rule, finding.into_finding()),
            s.reason.clone(),
            s.severity,
        )
        .with_anchor(s.anchor.clone())
        .with_polarity(Polarity::AllowlistGap),
        s.posture,
    )))
}

// --- The probe reaction (hot path) ------------------------------------------

#[doc(hidden)]
pub fn __react(seam: &'static str, type_id: TypeId) {
    let registry = REGISTRY.get().unwrap_or_else(|| {
        panic!("louke: assert_boundary!(\"{seam}\", …) ran before louke::install")
    });
    match check_crossing(seam, type_id, registry) {
        Ok(None) => {}
        Ok(Some((violation, posture))) => {
            emit(&violation);
            // `warn` is always event-only; panic only on an opted-in enforce violation.
            if posture == Posture::Panic && violation.severity == Severity::Enforce {
                panic!(
                    "louke: runtime boundary '{seam}' violated by {}",
                    violation.finding
                );
            }
        }
        // An undeclared seam (or probe-before-install) is a constitution error: fail loud,
        // never silently pass — the runtime analogue of exit 2.
        Err(message) => panic!("louke constitution error: {message}"),
    }
}

// --- The reaction sink ------------------------------------------------------

#[allow(clippy::type_complexity)]
static SINK: OnceLock<Box<dyn Fn(&Violation) + Send + Sync>> = OnceLock::new();

/// Install the sink that receives runtime `Violation` events (a logger, an audit pipeline).
/// Set once at startup; the default sink (used if none is installed) prints the violation as
/// JSON to stderr.
pub fn set_sink<F>(sink: F)
where
    F: Fn(&Violation) + Send + Sync + 'static,
{
    if SINK.set(Box::new(sink)).is_err() {
        panic!("louke: set_sink called twice — the sink is set once at startup");
    }
}

pub(crate) fn emit(violation: &Violation) {
    match SINK.get() {
        Some(sink) => sink(violation),
        // The default sink runs on every violation under the default `Event` posture, *before*
        // the opt-in panic gate — so it must never itself panic. `eprintln!` panics if the stderr
        // write fails (a closed/broken pipe, `… 2>&1 | consumer` after the consumer exits), which
        // would crash the production process on a reaction — the exact failure the crate's
        // no-panic-on-false-positive invariant forbids. Write directly and ignore a write error.
        None => {
            use std::io::Write;
            let _ = writeln!(
                std::io::stderr(),
                "louke: runtime boundary violated\n{}",
                xuanji::pretty_json(&violation.to_json())
            );
        }
    }
}
