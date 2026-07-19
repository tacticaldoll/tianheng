//! 漏刻 (Lòukè) — the **runtime** observation dimension of Tianheng.
//!
//! Where 圭表 reads imports and 渾儀 reads the AST — both at CI time, against source —
//! 漏刻 reacts at **runtime, in your binary**, against **live objects**: it sees the
//! concrete type behind a `dyn Trait` crossing an architectural seam, which static and
//! semantic analysis structurally cannot.
//!
//! Two faces:
//! - **Prod face.** Declare `RuntimeBoundary::at("seam").only_origins([…])` and
//!   [`install`] it once at startup; opt a type into an **observed** origin with
//!   [`register_origin!`] (it captures `module_path!()` at the registration site — the
//!   origin is *where the type is registered*, not a free label); place
//!   [`assert_boundary!`]`("seam", obj)` at the seam. The probe reads the live object's
//!   concrete origin and reacts **fail-closed** (an unknown origin is not in the allowlist,
//!   so it reacts). The default reaction emits a [`xuanji::Violation`] event; `panic` is
//!   opt-in — a governance tool must never crash production on a false positive.
//! - **CI face** (the non-default `audit` feature). `audit_probe_coverage` takes the **declared
//!   `RuntimeBoundary` objects** as
//!   the authoritative seam set and scans the workspace's source for `assert_boundary!` probes,
//!   so every declared seam has a probe (and every probe a declared seam) — closing the
//!   "declared but never enforced" gap at CI time. A non-literal probe seam is reacted to, not
//!   silently skipped. Declarations come from the objects, never a source scan, so an
//!   unconventionally spelled declaration cannot hide a seam. It lives in a dedicated `audit`
//!   module, compiled only under the feature.
//!
//! The hot path is std-only and lock-free (a write-once registry); the `TypeId`→origin lookup
//! uses a fold-hasher rather than the default SipHash, while the tiny fixed seam-name map is an
//! ordinary `std` map (its cost is negligible against the `TypeId` lookup). `serde_json` (via
//! 璇璣) is used only on the cold path (emitting an event). 漏刻 depends on 璇璣 (`xuanji`) alone.
//!
//! Govern by reaction, not instruction.

#![forbid(unsafe_code)]
#![deny(missing_docs)]

use std::any::{Any, TypeId};
use std::collections::HashMap;
use std::hash::{BuildHasherDefault, Hasher};
use std::sync::OnceLock;

pub use xuanji::{
    BoundaryKind, Finding, FindingKey, Outcome, Polarity, Report, Severity, Violation, ViolationId,
};

mod finding;

// CI face (the non-default `audit` feature): the probe-coverage audit + source scanner, in its
// own module so a prod dependency on louke compiles none of it. The prod face (the declaration
// DSL, the write-once registry, and the fail-closed probe reaction) stays in this root module.
#[cfg(feature = "audit")]
mod audit;
#[cfg(feature = "audit")]
pub use audit::audit_probe_coverage;

/// The canonical runtime seam-origin rule label — written **once** here and referenced by
/// both the prod reaction (the crate's internal `check_crossing`) and the 天衡 shell's `list`
/// projection (`tianheng` depends on `louke`, so importing this is the allowed direction). Editing it
/// in one place updates every projection. The specific allowed-origin set is a per-boundary
/// detail layered on at each site, not part of this rule-family label.
pub const RUNTIME_SEAM_RULE: &str = "only declared origins may cross the seam";

/// The full runtime seam **rule line** — the canonical [`RUNTIME_SEAM_RULE`] label with the
/// per-boundary allowed-origin set folded in (`… (only origins: A, B)`). Written **once** here and
/// shared by the prod reaction (`check_crossing`'s violation rule) and the 天衡 shell's
/// `list --format text` projection, so the human-readable line the two render never drifts. The JSON
/// projection deliberately keeps the label bare and carries the origins as a separate field, so it
/// does not use this.
pub fn runtime_seam_rule_line(allowed_origins: &[&str]) -> String {
    format!(
        "{RUNTIME_SEAM_RULE} (only origins: {})",
        allowed_origins.join(", ")
    )
}

// --- Tracked: the trait-level instrumentation -------------------------------

/// The supertrait a governed trait carries so a probe can recover the concrete type behind
/// a `dyn Trait` without trait upcasting (rust 1.85). Write `trait DomainPort: louke::Tracked`;
/// the blanket impl supplies `as_any` for every `'static` type, so no per-type boilerplate is
/// needed. A `&dyn Trait` over **borrowed** (non-`'static`) data cannot be probed — `Any`
/// requires `'static` (a stated bound).
pub trait Tracked: Any {
    /// Recover `&dyn Any` (and thus the concrete `TypeId`) from a trait object.
    fn as_any(&self) -> &dyn Any;
}

impl<T: Any> Tracked for T {
    fn as_any(&self) -> &dyn Any {
        self
    }
}

// --- The fold-hasher: TypeId is already a hash; never SipHash ----------------

/// A std-only hasher that folds the written bytes. A `TypeId` is already a good hash, so
/// folding avoids SipHash's per-lookup cost (the overhead-spike's only trap). `write` is
/// implemented for the general byte path (not only `write_u64`/`write_u128`), since a
/// `TypeId`'s hash may route through either across toolchains.
#[derive(Default)]
struct FoldHasher(u64);

impl Hasher for FoldHasher {
    fn finish(&self) -> u64 {
        self.0
    }
    fn write(&mut self, bytes: &[u8]) {
        for &b in bytes {
            self.0 = self.0.rotate_left(8) ^ b as u64;
        }
    }
    fn write_u64(&mut self, i: u64) {
        self.0 ^= i;
    }
    fn write_u128(&mut self, i: u128) {
        self.0 ^= i as u64 ^ (i >> 64) as u64;
    }
}

type TidMap<V> = HashMap<TypeId, V, BuildHasherDefault<FoldHasher>>;

// --- Declaration DSL --------------------------------------------------------

/// How a violated boundary reacts in production. `Event` (the default) emits a structured
/// `Violation`; `Panic` additionally aborts — opt-in only, never the default.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Posture {
    /// Emit a `Violation` event to the sink and continue. The default.
    Event,
    /// Emit the event, then panic — opt-in only (`enforce` severity).
    Panic,
}

impl Posture {
    /// A stable lower-case label for projection (`list`/`--format json`).
    pub fn as_str(&self) -> &'static str {
        match self {
            Posture::Event => "event",
            Posture::Panic => "panic",
        }
    }
}

/// A runtime boundary: only the declared **origins** may cross the named **seam**. Declared
/// in Rust (the single source of truth) and installed once at startup; a probe references the
/// seam by name, so the policy lives in this declaration, not at the call site.
#[derive(Debug, Clone)]
pub struct RuntimeBoundary {
    seam: &'static str,
    allowed: Vec<&'static str>,
    reason: String,
    severity: Severity,
    posture: Posture,
    anchor: Option<String>,
}

impl RuntimeBoundary {
    /// Begin a boundary at the named runtime seam.
    pub fn at(seam: &'static str) -> RuntimeSeamDraft {
        RuntimeSeamDraft { seam }
    }

    /// The governed seam name.
    pub fn seam(&self) -> &str {
        self.seam
    }

    /// The origins allowed to cross the seam.
    pub fn allowed_origins(&self) -> &[&'static str] {
        &self.allowed
    }

    /// The human-readable reason (the repair hint).
    pub fn reason(&self) -> &str {
        &self.reason
    }

    /// Attach a durable governance anchor (e.g. `"ADR-014"`) — a stable pointer into the
    /// project's governance, distinct from the free-text `reason`. Optional; a boundary with
    /// none projects and reacts exactly as before.
    pub fn with_anchor(mut self, anchor: &str) -> Self {
        self.anchor = Some(anchor.to_string());
        self
    }

    /// The durable governance anchor recorded with the boundary, if any.
    pub fn anchor(&self) -> Option<&str> {
        self.anchor.as_deref()
    }

    /// The declared severity. The CI face reacts to a declared-but-unprobed seam at this
    /// severity (a `warn` boundary yields an advisory, not a CI failure).
    pub fn severity(&self) -> Severity {
        self.severity
    }

    /// The declared production reaction posture (`Event` default, `Panic` opt-in). Exposed so the
    /// `list` projection is faithful — a `panic_on_violation` boundary must not project identically
    /// to a default event-only one.
    pub fn posture(&self) -> Posture {
        self.posture
    }
}

/// A boundary awaiting its allowed-origin set.
#[doc(hidden)]
pub struct RuntimeSeamDraft {
    seam: &'static str,
}

impl RuntimeSeamDraft {
    /// Allow only the given origins (origin labels — typically a `module_path!()` captured by
    /// [`register_origin!`]) to cross this seam.
    pub fn only_origins<I>(self, origins: I) -> RuntimeBoundaryDraft
    where
        I: IntoIterator<Item = &'static str>,
    {
        RuntimeBoundaryDraft {
            seam: self.seam,
            allowed: origins.into_iter().collect(),
            severity: Severity::Enforce,
            posture: Posture::Event,
        }
    }
}

/// A boundary awaiting severity/posture (optional) and a reason.
#[doc(hidden)]
pub struct RuntimeBoundaryDraft {
    seam: &'static str,
    allowed: Vec<&'static str>,
    severity: Severity,
    posture: Posture,
}

impl RuntimeBoundaryDraft {
    /// Make this advisory (`warn`): violations are reported but never panic, regardless of
    /// posture — the first rung of adoption.
    pub fn warn(mut self) -> Self {
        self.severity = Severity::Warn;
        self
    }

    /// Opt into panicking on an `enforce`-severity violation (default is event-only).
    pub fn panic_on_violation(mut self) -> Self {
        self.posture = Posture::Panic;
        self
    }

    /// Finish the boundary with its human-readable reason (the repair hint).
    pub fn because(self, reason: &str) -> RuntimeBoundary {
        RuntimeBoundary {
            seam: self.seam,
            allowed: self.allowed,
            reason: reason.to_string(),
            severity: self.severity,
            posture: self.posture,
            anchor: None,
        }
    }
}

/// An origin registration produced by [`register_origin!`] — a `TypeId`, the **observed**
/// origin (`module_path!()` at the registration site), and the type's name (for findings).
/// Pass these to [`install`].
#[derive(Debug, Clone)]
pub struct OriginEntry {
    type_id: TypeId,
    origin: &'static str,
    type_name: &'static str,
}

impl OriginEntry {
    /// Construct an origin entry. Prefer [`register_origin!`], which captures the call-site
    /// `module_path!()` so the origin is observed, not hand-asserted.
    pub fn new(type_id: TypeId, origin: &'static str, type_name: &'static str) -> Self {
        OriginEntry {
            type_id,
            origin,
            type_name,
        }
    }
}

// --- The write-once registry ------------------------------------------------

struct Seam {
    allowed: Vec<&'static str>,
    reason: String,
    severity: Severity,
    posture: Posture,
    anchor: Option<String>,
}

struct OriginInfo {
    origin: &'static str,
    type_name: &'static str,
}

struct Registry {
    origins: TidMap<OriginInfo>,
    seams: HashMap<&'static str, Seam>,
}

static REGISTRY: OnceLock<Registry> = OnceLock::new();

/// Install the runtime constitution **once at startup**: the declared boundaries and the
/// origin registrations ([`register_origin!`]). The registry is **write-once** so the probe
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
fn check_crossing(
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
    let origin = info.map(|i| i.origin);
    let allowed = match origin {
        Some(o) => s.allowed.contains(&o),
        // Fail-closed: a type that never registered an origin is not in the allowlist.
        None => false,
    };
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
        // the substring the prod contract/tests depend on still holds; the TypeId is stable within a
        // build (a hash of the type's identity).
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

fn emit(violation: &Violation) {
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

// --- Macros -----------------------------------------------------------------

/// Register a type's **observed** origin: `register_origin!(PostgresRepo)` captures
/// `module_path!()` at the call site (so the origin is *where the type is registered*, not a
/// self-asserted label) and yields an [`OriginEntry`] to pass to [`install`]. Declarative —
/// no proc-macro, no `syn`.
#[macro_export]
macro_rules! register_origin {
    ($ty:ty) => {
        $crate::OriginEntry::new(
            ::std::any::TypeId::of::<$ty>(),
            ::std::module_path!(),
            ::std::any::type_name::<$ty>(),
        )
    };
}

/// Probe a runtime seam: `assert_boundary!("domain-entry", obj)` reads `obj`'s concrete
/// origin (via the [`Tracked`] supertrait on its trait) and reacts fail-closed against the
/// seam's allowlist. `obj` must be a reference to a `dyn Trait` whose trait carries
/// `: louke::Tracked`.
#[macro_export]
macro_rules! assert_boundary {
    ($seam:expr, $obj:expr) => {
        $crate::__react($seam, $crate::Tracked::as_any($obj).type_id())
    };
}

#[cfg(test)]
mod tests {
    use super::*;

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
    fn the_fold_hasher_distinguishes_types() {
        let mut m: TidMap<u8> = TidMap::default();
        m.insert(TypeId::of::<Domain>(), 1);
        m.insert(TypeId::of::<Infra>(), 2);
        assert_eq!(m.get(&TypeId::of::<Domain>()), Some(&1));
        assert_eq!(m.get(&TypeId::of::<Infra>()), Some(&2));
        assert_eq!(m.len(), 2);
    }
}
