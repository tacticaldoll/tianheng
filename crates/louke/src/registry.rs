use std::any::TypeId;
use std::collections::HashMap;
use std::sync::OnceLock;
use std::sync::atomic::{AtomicU64, Ordering};

use xuanji::{BoundaryKind, Polarity, Severity, Violation, ViolationId};

use crate::dsl::{OriginEntry, Posture, RuntimeBoundary, runtime_rule_key};
use crate::finding;
use crate::runtime_seam_rule_line;
use crate::tracked::TidMap;

/// The repair hint shared between the prod runtime panic-path error below and the CI-audit's
/// `UndeclaredProbe` violation reason (`audit.rs`) — a probe naming a seam that was never
/// declared. Written once so the two "aligned" wordings cannot drift the way an unenforced,
/// hand-copied claim of alignment otherwise risks.
pub(crate) const UNDECLARED_SEAM_REPAIR_HINT: &str =
    "declare the RuntimeBoundary or fix the probe's seam name";

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
        format!(
            "an undeclared seam is never enforced — {UNDECLARED_SEAM_REPAIR_HINT}: probe \
                 references undeclared runtime seam '{seam}'"
        )
    })?;

    let info = registry.origins.get(&type_id);
    let allowed = info.is_some_and(|i| s.allowed.contains(&i.origin));
    if allowed {
        return Ok(None);
    }

    let finding = match info {
        Some(i) => finding::RuntimeFact::RegisteredCrossing {
            origin: i.origin.to_string(),
            type_name: i.type_name.to_string(),
        },
        None => finding::RuntimeFact::UnregisteredCrossing {
            type_id: format!("{type_id:?}"),
        },
    };
    let rule = runtime_seam_rule_line(&s.allowed);
    let finding = finding.into_finding();
    Ok(Some((
        Violation::new(
            BoundaryKind::Runtime,
            ViolationId::new(seam, runtime_rule_key(&s.allowed), finding.fact().clone()),
            rule,
            finding.text(),
            s.reason.clone(),
            s.severity,
        )
        .with_anchor(s.anchor.clone())
        .with_polarity(Polarity::AllowlistGap),
        s.posture,
    )))
}

#[doc(hidden)]
pub fn __react(seam: &'static str, type_id: TypeId) {
    let registry = REGISTRY.get().unwrap_or_else(|| {
        panic!("louke: assert_boundary!(\"{seam}\", …) ran before louke::install")
    });
    match check_crossing(seam, type_id, registry) {
        Ok(None) => {}
        Ok(Some((violation, posture))) => {
            emit(&violation);
            if posture == Posture::Panic && violation.severity == Severity::Enforce {
                panic!(
                    "louke: runtime boundary '{seam}' violated by {}",
                    violation.finding
                );
            }
        }
        Err(message) => panic!("louke constitution error: {message}"),
    }
}

#[allow(clippy::type_complexity)]
static SINK: OnceLock<Box<dyn Fn(&Violation) + Send + Sync>> = OnceLock::new();

/// Count of `Violation` events the shipped default sink has dropped because its write to
/// stderr failed, with no custom sink installed to receive them instead. See
/// [`dropped_sink_events`].
static DROPPED_SINK_EVENTS: AtomicU64 = AtomicU64::new(0);

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

/// How many `Violation` events the shipped default sink has dropped, for the whole process
/// lifetime, because its write to stderr failed (a closed pipe, a closed fd) while `set_sink`
/// was never called to receive them instead. Always `0` unless that has actually happened. A
/// single process-wide count across every seam — it tells an adopter *that* a drop happened,
/// never *which* boundary's violation was lost. See `runtime-origin-assertion`'s "Default-safe
/// reaction — a Violation event, panic opt-in" requirement for the full rationale: why the
/// default sink never panics on a broken pipe instead, why the increment is a single infallible
/// atomic add, and why a custom sink's own success/failure is never counted here.
pub fn dropped_sink_events() -> u64 {
    DROPPED_SINK_EVENTS.load(Ordering::Relaxed)
}

pub(crate) fn emit(violation: &Violation) {
    match SINK.get() {
        Some(sink) => sink(violation),
        None => emit_default(std::io::stderr(), violation),
    }
}

/// The default sink's write, isolated behind a generic writer so the dropped-event counting can
/// be pinned with a writer that always fails, without touching the real process stderr.
pub(crate) fn emit_default(mut w: impl std::io::Write, violation: &Violation) {
    if writeln!(
        w,
        "louke: runtime boundary violated\n{}",
        xuanji::pretty_json(&violation.to_json())
    )
    .is_err()
    {
        DROPPED_SINK_EVENTS.fetch_add(1, Ordering::Relaxed);
    }
}
