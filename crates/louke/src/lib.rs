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
//!   unconventionally spelled declaration cannot hide a seam.
//!
//! The hot path is std-only and lock-free (a write-once registry, a fold-hasher — never the
//! default SipHash); `serde_json` (via 璇璣) is used only on the cold path (emitting an
//! event). 漏刻 depends on 璇璣 (`xuanji`) alone.
//!
//! Govern by reaction, not instruction.

#![deny(missing_docs)]

use std::any::{Any, TypeId};
use std::collections::HashMap;
// HashSet / std::path are CI-scanner-only; gated so the default build has no unused imports.
#[cfg(feature = "audit")]
use std::collections::HashSet;
use std::hash::{BuildHasherDefault, Hasher};
#[cfg(feature = "audit")]
use std::path::{Path, PathBuf};
use std::sync::OnceLock;

pub use xuanji::{BoundaryKind, Outcome, Report, Severity, Violation};

/// The canonical runtime seam-origin rule label — written **once** here and referenced by
/// both the prod reaction (the crate's internal `check_crossing`) and the 天衡 shell's `list`
/// projection (`tianheng` depends on `louke`, so importing this is the allowed direction). Editing it
/// in one place updates every projection. The specific allowed-origin set is a per-boundary
/// detail layered on at each site, not part of this rule-family label.
pub const RUNTIME_SEAM_RULE: &str = "only declared origins may cross the seam";

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
/// an unknown origin, reacts. Returns the `Violation` to react with, or `None` when clean.
fn check_crossing(
    seam: &str,
    type_id: TypeId,
    registry: &Registry,
) -> Result<Option<Violation>, String> {
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
        Some(i) => format!("{} ({})", i.origin, i.type_name),
        None => "<unregistered origin>".to_string(),
    };
    // The rule family is the canonical `RUNTIME_SEAM_RULE` label; the allowed-origin set is the
    // per-boundary detail appended here, so the prod reaction and the `list` projection share one
    // rule label (the shell's `runtime` projection references the same const).
    let rule = format!(
        "{RUNTIME_SEAM_RULE} (only origins: {})",
        s.allowed.join(", ")
    );
    Ok(Some(Violation::new(
        BoundaryKind::Runtime,
        seam.to_string(),
        rule,
        finding,
        s.reason.clone(),
        s.severity,
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
        Ok(Some(violation)) => {
            emit(&violation);
            let posture = registry
                .seams
                .get(seam)
                .map(|s| s.posture)
                .unwrap_or(Posture::Event);
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
        None => eprintln!(
            "louke: runtime boundary violated\n{}",
            xuanji::pretty_json(&violation.to_json())
        ),
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

// --- CI face: probe-coverage audit ------------------------------------------
//
// Everything below (the `Probe` enum, `audit_probe_coverage`, and the source scanner) is the
// CI face, gated behind the non-default `audit` feature so a prod dependency on louke compiles
// none of it; the `tianheng` shell enables it. Why a feature, not a 5th crate: PROJECT.md.

/// What the source scan found for a probe occurrence (`assert_boundary!`).
#[cfg(feature = "audit")]
#[derive(Debug)]
enum Probe {
    /// A probe whose seam is a string literal (auditable, plain or raw): the seam value.
    Literal(String),
    /// A probe whose seam argument is NOT a string literal (a const or expression): the CI
    /// face cannot trace it to a declared seam, so it reacts rather than skipping. Carries the
    /// source file so the reaction is actionable (and the baseline identity stable).
    Unauditable { file: String },
}

/// **CI face.** Audit probe coverage against the **declared `RuntimeBoundary` objects** (the
/// authoritative seam set — the constitution, not a source scan for declarations) by scanning
/// the workspace's `src_dirs` for `assert_boundary!` probes. Reacts, with the static
/// dimensions' exit-code contract, in both directions plus an un-auditable case:
///
/// - **declared-but-unprobed** — a declared seam with no literal probe → a `Violation` at the
///   declaring boundary's severity (a `warn` boundary yields an advisory). Closes the
///   otherwise-essential "declared but never enforced" gap.
/// - **probed-but-undeclared** — a literal probe whose seam is not in the declared set → an
///   enforce `Violation` (a typo against the declared seams).
/// - **un-auditable probe** — an `assert_boundary!` whose seam argument is not a string literal
///   (e.g. a `const`) cannot be traced to a declared seam → an enforce `Violation` naming the
///   site, never a silent skip (a silent skip would be a false negative).
///
/// Declarations come from the passed objects, so an unconventionally spelled `RuntimeBoundary::at`
/// can no longer hide a seam. The probe scan is build/CI-time only (std-only, comment- and
/// string-literal-aware including raw/byte strings); source outside a member's lib/bin target
/// subtree is out of scope (the same bound as the semantic dimension). It does NOT observe the
/// live install registry — install-vs-constitution consistency is the prod face's runtime
/// fail-closed concern; this verifies coverage against the declared seams and the source.
///
/// **Stated bound (lexical, not semantic):** the scan is textual and does not evaluate `cfg`.
/// A probe behind a non-production `#[cfg(...)]` (e.g. `#[cfg(test)]`) is still counted as
/// covering its seam, so a seam whose *only* probe is compiled out of the production binary
/// would be reported covered. Keep a seam's production probe out of non-production `cfg`s.
///
/// Compiled only with the non-default `audit` feature (the CI face); see the module note above.
#[cfg(feature = "audit")]
pub fn audit_probe_coverage(declared: &[RuntimeBoundary], src_dirs: &[PathBuf]) -> Outcome {
    let mut probes = Vec::new();
    for dir in src_dirs {
        if let Err(message) = collect_probes(dir, &mut probes) {
            return Outcome::ConstitutionError(message);
        }
    }
    let probed_set: HashSet<&str> = probes
        .iter()
        .filter_map(|p| match p {
            Probe::Literal(seam) => Some(seam.as_str()),
            Probe::Unauditable { .. } => None,
        })
        .collect();
    let declared_set: HashSet<&str> = declared.iter().map(RuntimeBoundary::seam).collect();
    let mut violations = Vec::new();

    // Duplicate declared seam: the prod `install` fails loud on it (a duplicate would silently
    // shadow the earlier boundary); catch it at CI too — one enforce violation per duplicated
    // seam — so the misconfiguration surfaces before it reaches a running binary.
    let mut seen_decl = HashSet::new();
    let mut dup_reported = HashSet::new();
    for boundary in declared {
        let seam = boundary.seam();
        if !seen_decl.insert(seam) && dup_reported.insert(seam) {
            violations.push(Violation::new(
                BoundaryKind::Runtime,
                seam.to_string(),
                "each runtime seam must be declared exactly once".to_string(),
                format!("seam '{seam}' is declared more than once"),
                "a duplicate declaration would silently shadow the earlier boundary at install"
                    .to_string(),
                Severity::Enforce,
            ));
        }
    }

    // Declared but never probed: the boundary is never enforced at runtime. Reacts at the
    // declaring boundary's severity (a warn boundary is advisory, not a CI failure).
    let mut seen = HashSet::new();
    for boundary in declared {
        let seam = boundary.seam();
        if !probed_set.contains(seam) && seen.insert(seam) {
            violations.push(Violation::new(
                BoundaryKind::Runtime,
                seam.to_string(),
                "every declared runtime seam must be probed".to_string(),
                format!("declared seam '{seam}' has no assert_boundary! probe"),
                "a RuntimeBoundary with no probe is never enforced at runtime".to_string(),
                boundary.severity,
            ));
        }
    }
    // Probed but never declared: the probe references an undeclared seam, which panics at
    // runtime — catch the typo at CI instead of crashing production.
    let mut seen_probe = HashSet::new();
    for probe in &probes {
        if let Probe::Literal(seam) = probe {
            if !declared_set.contains(seam.as_str()) && seen_probe.insert(seam.as_str()) {
                violations.push(Violation::new(
                    BoundaryKind::Runtime,
                    seam.clone(),
                    "every probe must reference a declared seam".to_string(),
                    format!("probe references undeclared seam '{seam}'"),
                    "an undeclared seam panics at runtime — declare the RuntimeBoundary or fix the probe's seam name".to_string(),
                    Severity::Enforce,
                ));
            }
        }
    }
    // Un-auditable probes: a non-literal seam argument cannot be traced to a declared seam.
    // React rather than silently skip (a silent skip is a false negative). One reaction per
    // file (deduped, sorted) so the finding names where to look and the baseline id is stable.
    let mut unauditable_files: Vec<&str> = probes
        .iter()
        .filter_map(|p| match p {
            Probe::Unauditable { file } => Some(file.as_str()),
            Probe::Literal(_) => None,
        })
        .collect();
    unauditable_files.sort_unstable();
    unauditable_files.dedup();
    for file in unauditable_files {
        // The offending source file is in hand here (the probe scan captured it). Project it
        // into the `file` field as well as the finding text: it is a genuine observation, so
        // reporting `null` would be a dishonest null. This is the one runtime violation with a
        // source location — the seam-level ones below/above name a seam, not a file.
        violations.push(
            Violation::new(
                BoundaryKind::Runtime,
                "<un-auditable probe>".to_string(),
                "an assert_boundary! seam must be a string literal to be auditable".to_string(),
                format!(
                    "{file} has an assert_boundary! probe with a non-literal seam (const or \
                     expression), which the CI face cannot trace to a declared seam"
                ),
                "spell the seam as a string literal so probe coverage can be verified".to_string(),
                Severity::Enforce,
            )
            .with_file(Some(file.to_string())),
        );
    }
    if violations.is_empty() {
        Outcome::Clean
    } else {
        Outcome::Violations(Report::new(violations))
    }
}

#[cfg(feature = "audit")]
fn collect_probes(dir: &Path, probes: &mut Vec<Probe>) -> Result<(), String> {
    let read = std::fs::read_dir(dir).map_err(|e| format!("cannot read {}: {e}", dir.display()))?;
    // Sort entries so the scan order — and thus the violation order in the report — is
    // deterministic across runs (read_dir order is OS/filesystem-dependent and unsorted).
    let mut paths = Vec::new();
    for entry in read {
        let entry =
            entry.map_err(|e| format!("cannot read a dir entry under {}: {e}", dir.display()))?;
        // file_type() does NOT follow symlinks, so a symlinked directory does not recurse —
        // avoiding an infinite loop on a cyclic symlink (fail safe, not stack-overflow loud).
        let file_type = entry
            .file_type()
            .map_err(|e| format!("cannot stat {}: {e}", entry.path().display()))?;
        paths.push((file_type.is_dir(), entry.path()));
    }
    paths.sort();
    for (is_dir, path) in paths {
        if is_dir {
            collect_probes(&path, probes)?;
        } else if path.extension().and_then(|e| e.to_str()) == Some("rs") {
            let source = std::fs::read_to_string(&path)
                .map_err(|e| format!("cannot read source {}: {e}", path.display()))?;
            scan_source(&source, &path.display().to_string(), probes);
        }
    }
    Ok(())
}

/// Walk source skipping comments / string & char literals, and when the `assert_boundary!`
/// probe marker appears in code, record whether its seam argument is a string literal
/// (auditable) or not (un-auditable). The declaration marker is no longer scanned —
/// declarations come from the passed `RuntimeBoundary` objects. `file` labels an un-auditable
/// Skip a (possibly nested) block comment whose opening `/*` is at `i`, returning the index just
/// past its outermost `*/`. Rust block comments nest, so depth is tracked; an unterminated comment
/// runs to EOF. Shared by [`scan_source`] and [`skip_trivia`] so the two cannot drift — the
/// original non-nested bug existed in *both* precisely because they were independent copies.
#[cfg(feature = "audit")]
fn skip_block_comment(b: &[u8], mut i: usize) -> usize {
    let mut depth = 1usize;
    i += 2; // past the opening `/*`
    while i + 1 < b.len() && depth > 0 {
        if b[i] == b'/' && b[i + 1] == b'*' {
            depth += 1;
            i += 2;
        } else if b[i] == b'*' && b[i + 1] == b'/' {
            depth -= 1;
            i += 2;
        } else {
            i += 1;
        }
    }
    if depth > 0 { b.len() } else { i }
}

/// probe so the reaction is actionable.
#[cfg(feature = "audit")]
fn scan_source(source: &str, file: &str, probes: &mut Vec<Probe>) {
    let b = source.as_bytes();
    let mut i = 0;
    while i < b.len() {
        // line comment
        if b[i] == b'/' && i + 1 < b.len() && b[i + 1] == b'/' {
            while i < b.len() && b[i] != b'\n' {
                i += 1;
            }
            continue;
        }
        // block comment (nesting + drift rationale in `skip_block_comment`). The string/raw
        // checks below run only when this branch does not, so a `/*` inside a literal never
        // opens one — keeping the depth count uncorrupted.
        if b[i] == b'/' && i + 1 < b.len() && b[i + 1] == b'*' {
            i = skip_block_comment(b, i);
            continue;
        }
        // raw / byte string literal (r"…", r#"…"#, b"…", br#"…"#) — must be handled
        // before the plain-string case, or an inner `"` desyncs the scanner.
        if let Some(end) = raw_or_byte_string_end(b, i) {
            i = end;
            continue;
        }
        // plain string literal
        if b[i] == b'"' {
            i += 1;
            while i < b.len() && b[i] != b'"' {
                if b[i] == b'\\' {
                    i += 1;
                }
                i += 1;
            }
            i += 1;
            continue;
        }
        // char literal vs lifetime: only skip when it is clearly a char ('x' or '\n'),
        // leaving a lifetime ('a) to be walked as code.
        if b[i] == b'\'' {
            let is_char =
                (i + 1 < b.len() && b[i + 1] == b'\\') || (i + 2 < b.len() && b[i + 2] == b'\'');
            if is_char {
                i += 1;
                while i < b.len() && b[i] != b'\'' {
                    if b[i] == b'\\' {
                        i += 1;
                    }
                    i += 1;
                }
                i += 1;
                continue;
            }
        }
        // A left word boundary: `my_assert_boundary!` / `xassert_boundary!` are unrelated user
        // macros, not our probe. Require the preceding byte to be a non-identifier char so a
        // marker embedded in a longer identifier is not mis-counted as a probe.
        let left_boundary = i == 0 || !is_ident_byte(b[i - 1]);
        if left_boundary {
            if let Some(rest) = match_marker(b, i, b"assert_boundary!") {
                let (probe, next) = capture_probe(b, rest, file);
                if let Some(probe) = probe {
                    probes.push(probe);
                }
                i = next;
                continue;
            }
        }
        i += 1;
    }
}

/// Detect a raw or byte string literal starting at `i` (`r"…"`, `r#"…"#`, `b"…"`,
/// `br"…"`, `br#"…"#`) and return the index past its end, or `None` if `i` is not such a
/// literal. Rust syntax guarantees `r`/`b` immediately before `"`/`#` is a literal prefix
/// (no identifier can precede a string), so no token-boundary check is needed.
#[cfg(feature = "audit")]
fn raw_or_byte_string_end(b: &[u8], i: usize) -> Option<usize> {
    let mut j = i;
    let byte = j < b.len() && b[j] == b'b';
    if byte {
        j += 1;
    }
    let raw = j < b.len() && b[j] == b'r';
    if raw {
        j += 1;
        let mut hashes = 0;
        while j < b.len() && b[j] == b'#' {
            hashes += 1;
            j += 1;
        }
        if j >= b.len() || b[j] != b'"' {
            return None;
        }
        j += 1;
        // scan to the closing `"` followed by `hashes` `#`s
        while j < b.len() {
            if b[j] == b'"' {
                let mut k = j + 1;
                let mut h = 0;
                while k < b.len() && h < hashes && b[k] == b'#' {
                    k += 1;
                    h += 1;
                }
                if h == hashes {
                    return Some(k);
                }
            }
            j += 1;
        }
        return Some(b.len());
    }
    // a `b"…"` byte string (escaped like a normal string) — only when a `b` prefix was
    // consumed and a quote immediately follows.
    if byte && j < b.len() && b[j] == b'"' {
        j += 1;
        while j < b.len() && b[j] != b'"' {
            if b[j] == b'\\' {
                j += 1;
            }
            j += 1;
        }
        return Some((j + 1).min(b.len()));
    }
    None
}

#[cfg(feature = "audit")]
fn match_marker(b: &[u8], i: usize, marker: &[u8]) -> Option<usize> {
    if i + marker.len() <= b.len() && &b[i..i + marker.len()] == marker {
        Some(i + marker.len())
    } else {
        None
    }
}

/// An identifier byte (`[A-Za-z0-9_]`) — used for the marker's left word boundary.
#[cfg(feature = "audit")]
fn is_ident_byte(byte: u8) -> bool {
    byte.is_ascii_alphanumeric() || byte == b'_'
}

/// Skip ASCII whitespace and `//` / `/* */` comments, returning the next code index. Mirrors
/// the comment handling in [`scan_source`] so a comment between the `!` and `(`, or before the
/// seam argument, does not desync probe capture (which would silently drop a real probe).
#[cfg(feature = "audit")]
fn skip_trivia(b: &[u8], mut i: usize) -> usize {
    loop {
        while i < b.len() && b[i].is_ascii_whitespace() {
            i += 1;
        }
        if b.get(i) == Some(&b'/') && b.get(i + 1) == Some(&b'/') {
            while i < b.len() && b[i] != b'\n' {
                i += 1;
            }
            continue;
        }
        if b.get(i) == Some(&b'/') && b.get(i + 1) == Some(&b'*') {
            i = skip_block_comment(b, i);
            continue;
        }
        return i;
    }
}

/// After the `assert_boundary!` marker, classify the probe by its first argument and return
/// `(probe, next_index)`. Skip trivia, expect a macro opening delimiter (`(`, `{`, or `[`),
/// skip trivia; a plain or raw string first argument is an auditable [`Probe::Literal`] (its
/// value); any other first token (a `const`, an expression, a byte string) is
/// [`Probe::Unauditable`] — never a silent skip. `None` (with `next` past the marker) only when
/// the marker is not actually a probe call (no opening delimiter follows).
#[cfg(feature = "audit")]
fn capture_probe(b: &[u8], i: usize, file: &str) -> (Option<Probe>, usize) {
    let i = skip_trivia(b, i);
    // Rust macros accept `( )`, `{ }`, or `[ ]` interchangeably; a probe written
    // `assert_boundary!{"s", o}` or `["s", o]` is a real probe. Accept any of the three
    // opening delimiters so a non-`()` probe is not silently dropped — a silent drop would let
    // a typo'd seam escape the undeclared-seam check, a false negative.
    if !matches!(b.get(i), Some(&b'(') | Some(&b'{') | Some(&b'[')) {
        return (None, i);
    }
    let i = skip_trivia(b, i + 1);
    if i >= b.len() {
        return (None, i);
    }
    // A raw string `r"…"` / `r#"…"#` is a traceable literal — parse its value rather than
    // rejecting it as un-auditable (which would mis-flag a legitimate probe and double-report).
    if b[i] == b'r' && matches!(b.get(i + 1), Some(b'"') | Some(b'#')) {
        if let Some((seam, next)) = raw_string_value(b, i) {
            return (Some(Probe::Literal(seam)), next);
        }
        return (
            Some(Probe::Unauditable {
                file: file.to_string(),
            }),
            i,
        );
    }
    // A plain string literal.
    if b[i] == b'"' {
        let mut j = i + 1;
        let start = j;
        while j < b.len() && b[j] != b'"' {
            if b[j] == b'\\' {
                j += 1;
            }
            j += 1;
        }
        if j >= b.len() {
            return (None, j);
        }
        let seam = String::from_utf8_lossy(&b[start..j]).into_owned();
        return (Some(Probe::Literal(seam)), j + 1);
    }
    // Anything else (a const, an expression, a byte string) cannot be traced to a declared seam.
    (
        Some(Probe::Unauditable {
            file: file.to_string(),
        }),
        i,
    )
}

/// Parse a raw string literal `r"…"` / `r#…"…"#…` starting at `i`, returning `(value, next)`.
/// `None` if it is not a well-formed raw string.
#[cfg(feature = "audit")]
fn raw_string_value(b: &[u8], i: usize) -> Option<(String, usize)> {
    let mut j = i + 1; // past `r`
    let mut hashes = 0;
    while b.get(j) == Some(&b'#') {
        hashes += 1;
        j += 1;
    }
    if b.get(j) != Some(&b'"') {
        return None;
    }
    j += 1;
    let start = j;
    while j < b.len() {
        if b[j] == b'"' {
            let mut k = j + 1;
            let mut h = 0;
            while h < hashes && b.get(k) == Some(&b'#') {
                k += 1;
                h += 1;
            }
            if h == hashes {
                return Some((String::from_utf8_lossy(&b[start..j]).into_owned(), k));
            }
        }
        j += 1;
    }
    None
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
        let v = check_crossing("seam", TypeId::of::<Infra>(), &reg)
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
        let v = check_crossing("seam", TypeId::of::<Infra>(), &reg)
            .unwrap()
            .unwrap();
        assert!(v.finding.contains("<unregistered origin>"), "{}", v.finding);
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

    // A declared boundary for a seam, severity-parameterized (declarations are now objects,
    // not source-scanned — so the audit tests construct them directly).
    #[cfg(feature = "audit")]
    fn boundary(seam: &'static str, severity: Severity) -> RuntimeBoundary {
        let draft = RuntimeBoundary::at(seam).only_origins(["o"]);
        let draft = if severity == Severity::Warn {
            draft.warn()
        } else {
            draft
        };
        draft.because("r")
    }

    #[test]
    #[cfg(feature = "audit")]
    fn scan_collects_only_literal_probes_skipping_comments_and_strings() {
        let src = r#"
            fn setup() { louke::install([RuntimeBoundary::at("domain-entry").only_origins(["app::domain"]).because("x")], []); }
            fn used() { assert_boundary!("domain-entry", obj); }
            // a comment mentioning assert_boundary!("ignored-comment") must not count
            let s = "assert_boundary!(\"ignored-string\", x)";
        "#;
        let mut probes = Vec::new();
        scan_source(src, "test.rs", &mut probes);
        let literals: Vec<&str> = probes
            .iter()
            .filter_map(|p| match p {
                Probe::Literal(s) => Some(s.as_str()),
                Probe::Unauditable { .. } => None,
            })
            .collect();
        // The `RuntimeBoundary::at` declaration is no longer scanned (declarations are objects).
        assert_eq!(
            literals,
            vec!["domain-entry"],
            "{probes:?} should hold only the real probe"
        );
        assert!(
            !literals.contains(&"ignored-comment") && !literals.contains(&"ignored-string"),
            "markers in comments/strings must not count: {literals:?}"
        );
        assert!(
            !probes
                .iter()
                .any(|p| matches!(p, Probe::Unauditable { .. })),
            "no un-auditable probe in this fixture"
        );
    }

    #[test]
    #[cfg(feature = "audit")]
    fn scan_flags_a_non_literal_seam_probe_as_unauditable() {
        let src = r#"
            const SEAM: &str = "domain-entry";
            fn used() { assert_boundary!(SEAM, obj); }
            fn ok() { assert_boundary!("explicit", obj); }
        "#;
        let mut probes = Vec::new();
        scan_source(src, "test.rs", &mut probes);
        assert!(
            probes
                .iter()
                .any(|p| matches!(p, Probe::Unauditable { .. })),
            "a const-seam probe must be flagged un-auditable: {probes:?}"
        );
        assert!(
            probes
                .iter()
                .any(|p| matches!(p, Probe::Literal(s) if s == "explicit")),
            "the literal probe is still captured: {probes:?}"
        );
    }

    #[test]
    #[cfg(feature = "audit")]
    fn a_comment_between_bang_and_paren_does_not_drop_the_probe() {
        // The dangerous false negative: a probe must still be seen with a comment between `!`
        // and `(`, else an undeclared/typo seam there would escape Direction B and panic in prod.
        for src in [
            "fn f() { assert_boundary! /* x */ (\"c-seam\", o); }",
            "fn f() { assert_boundary! // c\n (\"c-seam\", o); }",
        ] {
            let mut probes = Vec::new();
            scan_source(src, "test.rs", &mut probes);
            assert!(
                probes
                    .iter()
                    .any(|p| matches!(p, Probe::Literal(s) if s == "c-seam")),
                "a comment between ! and ( must not drop the probe: {probes:?}"
            );
        }
    }

    #[test]
    #[cfg(feature = "audit")]
    fn an_identifier_ending_in_the_marker_is_not_a_probe() {
        // `my_assert_boundary!` / `xassert_boundary!` are unrelated user macros — a left word
        // boundary keeps them from being mis-counted (a false probe that could mask coverage).
        let src = "fn f() { my_assert_boundary!(\"prefixed\", o); xassert_boundary!(\"fp\", o); }";
        let mut probes = Vec::new();
        scan_source(src, "test.rs", &mut probes);
        assert!(
            probes.is_empty(),
            "an embedded marker must not count as a probe: {probes:?}"
        );
    }

    #[test]
    #[cfg(feature = "audit")]
    fn a_raw_string_seam_is_an_auditable_literal() {
        // A raw-string seam is a traceable literal — parse its value, do not mis-flag it.
        let src =
            "fn f() { assert_boundary!(r#\"raw-seam\"#, o); assert_boundary!(r\"plain-raw\", o); }";
        let mut probes = Vec::new();
        scan_source(src, "test.rs", &mut probes);
        assert!(
            probes
                .iter()
                .any(|p| matches!(p, Probe::Literal(s) if s == "raw-seam")),
            "r#\"…\"# seam value must be captured: {probes:?}"
        );
        assert!(
            probes
                .iter()
                .any(|p| matches!(p, Probe::Literal(s) if s == "plain-raw")),
            "r\"…\" seam value must be captured: {probes:?}"
        );
        assert!(
            !probes
                .iter()
                .any(|p| matches!(p, Probe::Unauditable { .. })),
            "a raw-string seam is auditable, not un-auditable: {probes:?}"
        );
    }

    #[test]
    #[cfg(feature = "audit")]
    fn a_raw_or_byte_string_does_not_desync_the_scanner() {
        // A raw string with an inner `"` must not swallow a later real probe, and a probe
        // marker inside a byte string must not be counted.
        let src = r####"
            let x = r#"he said "hi""#;
            fn f() { assert_boundary!("real-seam", o); }
            let y = b"assert_boundary!(\"bytestr\", z)";
        "####;
        let mut probes = Vec::new();
        scan_source(src, "test.rs", &mut probes);
        let literals: Vec<&str> = probes
            .iter()
            .filter_map(|p| match p {
                Probe::Literal(s) => Some(s.as_str()),
                Probe::Unauditable { .. } => None,
            })
            .collect();
        assert!(
            literals.contains(&"real-seam"),
            "a raw string must not desync and swallow a later probe: {literals:?}"
        );
        assert!(
            !literals.contains(&"bytestr"),
            "a marker inside a byte string must not count: {literals:?}"
        );
    }

    // Write a one-file crate dir under a unique base and return it.
    #[cfg(feature = "audit")]
    fn literal_seams(probes: &[Probe]) -> Vec<String> {
        probes
            .iter()
            .filter_map(|p| match p {
                Probe::Literal(s) => Some(s.clone()),
                Probe::Unauditable { .. } => None,
            })
            .collect()
    }

    #[cfg(feature = "audit")]
    #[test]
    fn a_probe_inside_a_nested_block_comment_is_not_counted() {
        // Rust block comments nest, so this entire span is ONE comment and the probe is
        // commented out. A non-depth-aware scan would leave comment mode at the inner `*/`
        // and wrongly count "s" as probed — the forbidden false negative (the seam would be
        // reported covered while never enforced).
        let mut probes = Vec::new();
        scan_source(
            r#"/* outer /* inner */ assert_boundary!("s", o); */"#,
            "t.rs",
            &mut probes,
        );
        assert!(
            probes.is_empty(),
            "a probe inside a nested block comment must not count: {probes:?}"
        );
    }

    #[cfg(feature = "audit")]
    #[test]
    fn a_real_probe_after_a_nested_block_comment_is_still_counted() {
        // The depth fix must not over-eat: `/* a /* b */ c */` is a complete (nested) comment,
        // and the probe that follows is real code and MUST count.
        let mut probes = Vec::new();
        scan_source(
            r#"/* a /* b */ c */ assert_boundary!("real", o);"#,
            "t.rs",
            &mut probes,
        );
        assert_eq!(
            literal_seams(&probes),
            ["real"],
            "a real probe after a closed nested comment must count: {probes:?}"
        );
    }

    #[cfg(feature = "audit")]
    #[test]
    fn a_brace_or_bracket_delimited_probe_is_captured() {
        // Rust macros accept `{ }` and `[ ]` identically to `( )`; such a probe is real and
        // must be audited, not silently dropped (a drop would let a typo'd seam escape the
        // undeclared-seam check — a false negative).
        let mut probes = Vec::new();
        scan_source(
            "fn f() { assert_boundary!{\"brace\", o}; assert_boundary![\"bracket\", o]; }",
            "t.rs",
            &mut probes,
        );
        let mut seams = literal_seams(&probes);
        seams.sort_unstable();
        assert_eq!(
            seams,
            ["brace", "bracket"],
            "brace/bracket-delimited probes must be captured: {probes:?}"
        );
    }

    #[cfg(feature = "audit")]
    #[test]
    fn audit_reacts_to_a_duplicate_declared_seam() {
        // A seam declared twice is a constitution error: prod `install` fails loud on it, and the
        // CI face must react too (enforce) so it surfaces before a running binary. Probe the seam
        // so the ONLY finding is the duplicate, not a declared-unprobed gap.
        let base = std::env::temp_dir().join(format!("louke-audit-dup-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&base);
        let dir = write_dir(&base, "m", "fn f() { assert_boundary!(\"twice\", o); }");
        let outcome = audit_probe_coverage(
            &[
                boundary("twice", Severity::Enforce),
                boundary("twice", Severity::Enforce),
            ],
            &[dir],
        );
        let _ = std::fs::remove_dir_all(&base);
        match outcome {
            Outcome::Violations(report) => assert!(
                report
                    .violations
                    .iter()
                    .any(|v| v.target == "twice" && v.finding.contains("declared more than once")),
                "a duplicate declared seam must react: {:?}",
                report.violations
            ),
            other => panic!("expected a duplicate-seam violation, got {other:?}"),
        }
    }

    #[cfg(feature = "audit")]
    #[test]
    fn a_nested_comment_between_bang_and_paren_does_not_drop_the_probe() {
        // skip_trivia shares the depth-aware skip with scan_source, so a NESTED comment between
        // `!` and `(` must be skipped whole; otherwise it desyncs and misses the real probe.
        let mut probes = Vec::new();
        scan_source(
            r#"fn f() { assert_boundary! /* a /* b */ c */ ("nested-trivia", o); }"#,
            "t.rs",
            &mut probes,
        );
        assert_eq!(
            literal_seams(&probes),
            ["nested-trivia"],
            "a probe after a nested comment between ! and ( must be captured: {probes:?}"
        );
    }

    #[cfg(feature = "audit")]
    fn write_dir(base: &Path, name: &str, body: &str) -> PathBuf {
        let dir = base.join(name);
        std::fs::create_dir_all(&dir).unwrap();
        std::fs::write(dir.join("a.rs"), body).unwrap();
        dir
    }

    #[test]
    #[cfg(feature = "audit")]
    fn audit_probe_coverage_reacts_both_directions() {
        let base = std::env::temp_dir().join(format!("louke-audit-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&base);

        // declared + probed match → clean (exit 0)
        let clean = write_dir(&base, "clean", "fn f() { assert_boundary!(\"s\", o); }");
        assert_eq!(
            audit_probe_coverage(&[boundary("s", Severity::Enforce)], &[clean]).exit_code(),
            0
        );

        // declared but unprobed (enforce) → react (exit 1)
        let unprobed = write_dir(&base, "unprobed", "fn f() {}");
        assert_eq!(
            audit_probe_coverage(&[boundary("orphan", Severity::Enforce)], &[unprobed]).exit_code(),
            1
        );

        // probed but undeclared (a typo) → react at CI, not a prod panic (exit 1)
        let typo = write_dir(&base, "typo", "fn f() { assert_boundary!(\"ghost\", o); }");
        assert_eq!(audit_probe_coverage(&[], &[typo]).exit_code(), 1);

        let _ = std::fs::remove_dir_all(&base);
    }

    #[test]
    #[cfg(feature = "audit")]
    fn a_warn_severity_unprobed_seam_is_advisory_not_a_failure() {
        let base = std::env::temp_dir().join(format!("louke-audit-warn-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&base);
        let dir = write_dir(&base, "warn", "fn f() {}");
        // A warn boundary with no probe reacts (a Violation) but does not by itself fail CI.
        let outcome = audit_probe_coverage(&[boundary("soft", Severity::Warn)], &[dir]);
        assert_eq!(outcome.exit_code(), 0, "warn-only is advisory: {outcome:?}");
        assert!(
            matches!(outcome, Outcome::Violations(_)),
            "it still reports the advisory: {outcome:?}"
        );
        let _ = std::fs::remove_dir_all(&base);
    }

    #[test]
    #[cfg(feature = "audit")]
    fn coverage_spans_the_workspace_corpus() {
        let base = std::env::temp_dir().join(format!("louke-audit-corpus-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&base);
        // Declared once; its only probe lives in a *different* member dir.
        let decl_only = write_dir(&base, "crate_a", "fn f() {}");
        let probe_only = write_dir(
            &base,
            "crate_b",
            "fn g() { assert_boundary!(\"shared\", o); }",
        );
        let outcome = audit_probe_coverage(
            &[boundary("shared", Severity::Enforce)],
            &[decl_only, probe_only],
        );
        assert_eq!(
            outcome.exit_code(),
            0,
            "the corpus is the union of all dirs: {outcome:?}"
        );
        let _ = std::fs::remove_dir_all(&base);
    }

    #[test]
    #[cfg(feature = "audit")]
    fn an_unauditable_probe_reacts() {
        let base = std::env::temp_dir().join(format!("louke-audit-unaud-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&base);
        let dir = write_dir(
            &base,
            "unaud",
            "const SEAM: &str = \"s\"; fn f() { assert_boundary!(SEAM, o); }",
        );
        // Even though a boundary "s" is declared, the probe is non-literal → un-auditable → react.
        let outcome = audit_probe_coverage(&[boundary("s", Severity::Enforce)], &[dir]);
        assert_eq!(
            outcome.exit_code(),
            1,
            "an un-auditable probe must react: {outcome:?}"
        );
        // The un-auditable violation carries the offending source file (the probe scan
        // captured it): a genuine observation, not a dishonest null.
        let violations = match &outcome {
            Outcome::Violations(report) => &report.violations,
            other => panic!("expected violations, got {other:?}"),
        };
        let file = violations
            .iter()
            .find_map(|v| v.file.as_deref())
            .expect("the un-auditable-probe violation carries its source file");
        assert!(
            file.ends_with("a.rs"),
            "file names the probe's source: {file}"
        );
        let _ = std::fs::remove_dir_all(&base);
    }

    #[test]
    #[cfg(feature = "audit")]
    fn a_seam_level_runtime_violation_has_no_file() {
        // A declared-but-never-probed seam names a seam, not a source location, so its `file`
        // is a faithful `None` — distinct from the un-auditable case, which does have a file.
        let base =
            std::env::temp_dir().join(format!("louke-audit-seamnull-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&base);
        let dir = write_dir(&base, "unprobed", "fn f() {}");
        let outcome = audit_probe_coverage(&[boundary("orphan", Severity::Enforce)], &[dir]);
        let violations = match &outcome {
            Outcome::Violations(report) => &report.violations,
            other => panic!("expected violations, got {other:?}"),
        };
        assert!(
            violations.iter().all(|v| v.file.is_none()),
            "a seam-level runtime violation has no source file: {violations:?}"
        );
        let _ = std::fs::remove_dir_all(&base);
    }
}
