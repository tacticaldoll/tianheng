//! 璇璣 (Xuánjī) — the shared **reaction model** of Tianheng, the 底 the whole stack
//! turns on.
//!
//! The jade pivot of the armillary sphere, the instrument of celestial measure: the
//! dimension-agnostic vocabulary [`Severity`], [`BoundaryKind`], [`Violation`],
//! [`Report`], [`Baseline`], and [`Outcome`] (each a finding's shape; [`ViolationId`] is
//! `Violation`'s baseline identity). Every observation
//! dimension — the static 圭表 (`guibiao`), semantic 渾儀 (`hunyi`), and runtime 漏刻
//! (`louke`) — expresses its findings in these types, so a dimension may reuse the reaction
//! vocabulary without depending on another dimension's engine.
//!
//! This crate carries the JSON (de)serialization that is **intrinsic** to its types: a
//! [`Baseline`] *is* a generated JSON snapshot, and a [`Violation`] has a canonical JSON
//! shape. It does **not** carry the report-document *assembly* (which folds in
//! dimension-specific data such as the static `Coverage`) — that stays in the consuming
//! crate. `serde_json` is its only dependency; it renders **no verdict** — it holds the
//! *measure*, never the react itself (comparing a declared boundary against observed reality
//! lives in the dimensions and the shell, never here).
//!
//! Govern by reaction, not instruction.

#![deny(missing_docs)]

use serde_json::Value;

/// Serialize an owned [`Value`] to pretty JSON. Infallible by construction, not by
/// hope: a `Value`'s `Serialize` impl never errors, its map keys are always strings,
/// and it cannot hold a non-finite float (`json!(f64::NAN)` yields `Null`), so the
/// only two documented `to_string_pretty` failure modes are both unreachable; the
/// sink is an in-memory `String`, so there is no I/O error path either. The `expect`
/// is therefore a proof annotation, not unhandled error. We deliberately keep it
/// over `-> Result<String, _>` plumbing into the callers: that would defend an
/// impossible state, which the minimalism bound rules out (fail-loud is for
/// observable misconfiguration, not for facts that cannot occur). This is the single
/// place that decision lives — change it here, with reasoning, not site by site.
pub fn pretty_json(document: &Value) -> String {
    serde_json::to_string_pretty(document).expect("a serde_json::Value is always serializable")
}

/// How strongly a boundary reacts. `Enforce` fails the reaction (exit 1); `Warn`
/// reports the violation as advisory without failing — the first rung of adoption,
/// so a dirty project can observe a boundary before enforcing it.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
#[non_exhaustive]
pub enum Severity {
    /// Violations fail the reaction (exit 1). The default.
    #[default]
    Enforce,
    /// Violations are reported as advisory but do not fail — the first rung of
    /// adoption, observed before a boundary is enforced.
    Warn,
}

impl Severity {
    /// The projection label (`"enforce"` / `"warn"`), the single source for both the
    /// report and constitution renderings.
    pub fn as_str(&self) -> &'static str {
        match self {
            Severity::Enforce => "enforce",
            Severity::Warn => "warn",
        }
    }
}

/// Which kind of boundary produced a violation — surfaced in the JSON report so a
/// consumer need not reverse-engineer the rule string. Not part of the baseline
/// identity ([`ViolationId`]), so adding it does not invalidate existing baselines.
/// `#[non_exhaustive]`: a further dimension (e.g. runtime) adds its own kind
/// without breaking consumers.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
pub enum BoundaryKind {
    /// The violation came from a crate boundary.
    Crate,
    /// The violation came from a module boundary.
    Module,
    /// The violation came from a semantic (AST) boundary — the 渾儀 dimension.
    Semantic,
    /// The violation came from a runtime boundary — the 漏刻 dimension.
    Runtime,
}

impl BoundaryKind {
    /// The projection label (`"crate"` / `"module"` / `"semantic"` / `"runtime"`).
    pub fn as_str(&self) -> &'static str {
        match self {
            BoundaryKind::Crate => "crate",
            BoundaryKind::Module => "module",
            BoundaryKind::Semantic => "semantic",
            BoundaryKind::Runtime => "runtime",
        }
    }
}

/// The **repair direction** a boundary-drift violation points to — a different axis from
/// [`BoundaryKind`], which names *which dimension* saw it. Derived from the producing rule's
/// type (known at the reaction site), never observed from code and never declared by the adopter.
/// `#[non_exhaustive]`: the axis is deliberately two-valued, but a future rung stays additive.
///
/// A violation not on this axis carries no polarity (`Violation::polarity` is `None`) — the runtime
/// CI-audit coverage/consistency violations are on a declaration/probe axis, not this one, and their
/// repair direction is read from the `reason`/`finding`. `None` means "off this axis", not "unknown".
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
pub enum Polarity {
    /// The rule forbids a specific target or shape; the repair is to **remove** the offending code
    /// (`forbid_*` / `must_not_*`).
    DenyBreach,
    /// The rule permits a set and reacts to a member outside it; the repair is to remove the code
    /// **or** declare the intent by widening the set (`restrict_*_to` / `only_*` /
    /// `deny_external_dependencies`, whose `allow_external` exceptions are an in-boundary
    /// declaration path).
    AllowlistGap,
}

impl Polarity {
    /// The projection label (`"deny_breach"` / `"allowlist_gap"`), the single source for the
    /// report and SARIF renderings.
    pub fn as_str(&self) -> &'static str {
        match self {
            Polarity::DenyBreach => "deny_breach",
            Polarity::AllowlistGap => "allowlist_gap",
        }
    }
}

/// One violated boundary. `severity` is the producing boundary's severity, so the
/// exit-code decision and the report can treat enforce and warn findings apart.
/// `baselined` is set when baseline gating records the violation in a baseline; a
/// baselined violation does not fail the reaction.
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub struct Violation {
    /// Which kind of boundary produced this violation.
    pub kind: BoundaryKind,
    /// The governed target (crate name, or module path for a module boundary).
    pub target: String,
    /// The rule label that was violated.
    pub rule: String,
    /// The offending finding (e.g. the dependency name, or the imported module path).
    pub finding: String,
    /// The boundary's reason — the repair hint.
    pub reason: String,
    /// The producing boundary's severity.
    pub severity: Severity,
    /// Whether this violation is recorded in the active baseline (so it does not fail).
    pub baselined: bool,
    /// The offending source file, when the producing dimension genuinely observes one — a
    /// faithful byproduct of the scan (e.g. the file a forbidden import sits in). `None` when
    /// the violation has no single source file (a dependency edge, a seam name) or the
    /// dimension does not yet observe a per-element file. Set via [`Violation::with_file`], not
    /// the constructor, so adding it leaves [`Violation::new`] non-breaking; it is **not** part
    /// of the baseline identity ([`Violation::id`]), so it never affects baseline matching.
    pub file: Option<String>,
    /// The producing boundary's durable governance anchor — a stable pointer (e.g. `"ADR-014"`)
    /// into the project's governance, distinct from the free-text `reason` sentence, which accretes
    /// ephemeral refs (PR numbers, handles, "recently") that rot faster than the invariant they
    /// justify. `None` when the boundary declared none. Set via [`Violation::with_anchor`], not the
    /// constructor, so adding it leaves [`Violation::new`] non-breaking; like `file`, it is metadata,
    /// **not** part of the baseline identity ([`Violation::id`]), so it never affects baseline
    /// matching, and it is never a reaction input — a pure durable pointer.
    pub anchor: Option<String>,
    /// The **repair direction** of a boundary-drift violation ([`Polarity`]) — `DenyBreach` (remove
    /// the offending code) or `AllowlistGap` (remove, or declare the intent). Derived from the
    /// producing rule's type at the reaction site, set via [`Violation::with_polarity`], not the
    /// constructor. `None` for a violation off the boundary-drift axis (the runtime CI-audit
    /// coverage violations), whose repair is read from the `reason`/`finding`. Like `file`/`anchor`,
    /// it is metadata: **not** part of the baseline identity ([`Violation::id`]) — being a pure
    /// function of the rule it is constant for a given identity anyway — and never a reaction input.
    pub polarity: Option<Polarity>,
}

impl Violation {
    /// Build a violation an engine has just observed: `baselined` starts `false` and is
    /// set later by [`apply_baseline`]. The constructor a dimension crate needs because
    /// `Violation` is `#[non_exhaustive]` and cannot be struct-literal-built across the
    /// crate boundary.
    pub fn new(
        kind: BoundaryKind,
        target: String,
        rule: String,
        finding: String,
        reason: String,
        severity: Severity,
    ) -> Self {
        Violation {
            kind,
            target,
            rule,
            finding,
            reason,
            severity,
            baselined: false,
            file: None,
            anchor: None,
            polarity: None,
        }
    }

    /// Attach the offending source file, consuming and returning `self` so a dimension can
    /// fold it into construction: `Violation::new(…).with_file(Some(path))`. Kept off
    /// [`Violation::new`] on purpose — the constructor's signature stays stable (non-breaking)
    /// and dimensions that observe no file simply never call this. The file is metadata, never
    /// part of the baseline identity ([`Violation::id`]).
    pub fn with_file(mut self, file: Option<String>) -> Self {
        self.file = file;
        self
    }

    /// Attach the producing boundary's durable governance anchor, consuming and returning `self`
    /// so a dimension can fold it into construction: `Violation::new(…).with_anchor(boundary…)`.
    /// Kept off [`Violation::new`] on purpose — the constructor's signature stays stable
    /// (non-breaking), and a boundary that declared no anchor simply never calls this (or passes
    /// `None`). The anchor is metadata, never part of the baseline identity ([`Violation::id`]).
    pub fn with_anchor(mut self, anchor: Option<String>) -> Self {
        self.anchor = anchor;
        self
    }

    /// Stamp the violation's repair-direction [`Polarity`], consuming and returning `self` so a
    /// dimension can fold it into construction: `Violation::new(…).with_polarity(rule.polarity())`.
    /// Takes a concrete `Polarity` (not an `Option`) because a reaction site that stamps one always
    /// knows it; a violation off the boundary-drift axis simply never calls this, leaving `None`.
    /// The polarity is metadata, never part of the baseline identity ([`Violation::id`]).
    pub fn with_polarity(mut self, polarity: Polarity) -> Self {
        self.polarity = Some(polarity);
        self
    }

    /// The `(target, rule, finding)` identity used to match against a baseline.
    pub fn id(&self) -> ViolationId {
        ViolationId {
            target: self.target.clone(),
            rule: self.rule.clone(),
            finding: self.finding.clone(),
        }
    }

    /// The canonical JSON rendering of one violation — the per-type projection the
    /// report-document assembly composes. The model owns this shape so every dimension
    /// renders its violations identically.
    pub fn to_json(&self) -> Value {
        serde_json::json!({
            "kind": self.kind.as_str(),
            "target": self.target,
            "rule": self.rule,
            "finding": self.finding,
            "reason": self.reason,
            "severity": self.severity.as_str(),
            "baselined": self.baselined,
            "file": self.file,
            "anchor": self.anchor,
            "polarity": self.polarity.map(|p| p.as_str()),
        })
    }
}

/// All violations from one evaluation.
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub struct Report {
    /// Every violation found in one evaluation.
    pub violations: Vec<Violation>,
}

impl Report {
    /// An empty report — no violations. A constructor a consumer needs because
    /// `Report` is `#[non_exhaustive]` and cannot be struct-literal-built from
    /// another crate.
    pub fn empty() -> Self {
        Report {
            violations: Vec::new(),
        }
    }

    /// A report of the given violations. The constructor an engine needs to assemble a
    /// report across the crate boundary (`Report` is `#[non_exhaustive]`).
    pub fn new(violations: Vec<Violation>) -> Self {
        Report { violations }
    }
}

/// A violation's identity for baseline matching: `(target, rule, finding)`. Reason
/// and severity are excluded so editing them does not turn a known violation new.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct ViolationId {
    /// The governed target (crate name or module path).
    pub target: String,
    /// The violated rule's label.
    pub rule: String,
    /// The offending finding.
    pub finding: String,
}

/// One recorded baseline entry: an accepted violation's identity plus optional
/// governance-tracking metadata. `owner` (who owns this accepted debt) and `tracker` (the external
/// issue tracking its fix) describe *how the accepted violation is tracked after acceptance*, not
/// the basis of the law — so they are **metadata only**, never part of the [`ViolationId`] match
/// key. There is deliberately no `anchor` here: the governance anchor already rides the live
/// boundary→violation ([`Violation::anchor`]), so a baseline copy would duplicate and drift from it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BaselineEntry {
    /// The accepted violation's identity — the match key.
    pub id: ViolationId,
    /// Who owns this accepted debt (optional; set by hand-annotating the baseline).
    pub owner: Option<String>,
    /// The external issue tracking this debt's fix (optional).
    pub tracker: Option<String>,
}

/// Sort and de-duplicate baseline entries **by identity** (a stable sort keeps input order among
/// equal ids, so `dedup_by` on the id keeps the first — the recorded tie-break for a hand-edited
/// duplicate). The `owner`/`tracker` metadata is never part of the sort or de-dup key, so the file
/// stays stable and diffable exactly as when entries were bare identities.
fn sort_dedup_by_id(entries: &mut Vec<BaselineEntry>) {
    entries.sort_by(|a, b| a.id.cmp(&b.id));
    entries.dedup_by(|a, b| a.id == b.id);
}

/// A recorded set of accepted violations — a generated observation snapshot, not
/// policy. The gate fails only on violations absent from it.
#[derive(Debug, Default)]
pub struct Baseline {
    entries: Vec<BaselineEntry>,
}

impl Baseline {
    /// Build a baseline from the current report's violations, with no metadata. Entries are
    /// sorted and de-duplicated by identity, so the file stays stable and diffable.
    pub fn of(report: &Report) -> Self {
        let mut entries: Vec<BaselineEntry> = report
            .violations
            .iter()
            .map(|violation| BaselineEntry {
                id: violation.id(),
                owner: None,
                tracker: None,
            })
            .collect();
        sort_dedup_by_id(&mut entries);
        Baseline { entries }
    }

    /// Build the next baseline snapshot from the current report, **preserving** each surviving
    /// entry's `owner`/`tracker` by identity: a violation still present carries its previous
    /// metadata forward, a newly-appearing one gets none, and a violation no longer present drops
    /// (its metadata with it). This keeps `--write-baseline` from silently discarding hand-added
    /// governance records while staying a snapshot of the currently-present accepted violations.
    pub fn of_preserving(report: &Report, previous: &Baseline) -> Self {
        let mut entries: Vec<BaselineEntry> = report
            .violations
            .iter()
            .map(|violation| {
                let id = violation.id();
                let prior = previous.entries.iter().find(|entry| entry.id == id);
                BaselineEntry {
                    owner: prior.and_then(|entry| entry.owner.clone()),
                    tracker: prior.and_then(|entry| entry.tracker.clone()),
                    id,
                }
            })
            .collect();
        sort_dedup_by_id(&mut entries);
        Baseline { entries }
    }

    /// The recorded entries, for reading their identity and metadata.
    pub fn entries(&self) -> impl Iterator<Item = &BaselineEntry> {
        self.entries.iter()
    }

    /// Whether this baseline records the given violation's identity.
    pub fn contains(&self, violation: &Violation) -> bool {
        let id = violation.id();
        self.entries.iter().any(|entry| entry.id == id)
    }

    /// Baseline entries that match no current violation — stale, safe to remove.
    pub fn stale(&self, report: &Report) -> Vec<&ViolationId> {
        let current: Vec<ViolationId> = report.violations.iter().map(Violation::id).collect();
        self.entries
            .iter()
            .filter(|entry| !current.iter().any(|id| id == &entry.id))
            .map(|entry| &entry.id)
            .collect()
    }

    /// Serialize to the on-disk JSON form: a `version` and sorted `violations`.
    pub fn to_json(&self) -> String {
        let violations: Vec<Value> = self
            .entries
            .iter()
            .map(|entry| {
                let mut object = serde_json::json!({
                    "target": entry.id.target,
                    "rule": entry.id.rule,
                    "finding": entry.id.finding,
                });
                // owner/tracker emitted only when set, so an un-annotated entry is byte-identical
                // to the pre-metadata form (the same Some-only discipline as `file`/`anchor`).
                if let Some(owner) = &entry.owner {
                    object["owner"] = serde_json::json!(owner);
                }
                if let Some(tracker) = &entry.tracker {
                    object["tracker"] = serde_json::json!(tracker);
                }
                object
            })
            .collect();
        let doc = serde_json::json!({ "version": 1, "violations": violations });
        pretty_json(&doc)
    }

    /// Parse from the on-disk JSON form. A malformed document or unknown version is
    /// an error, never a silently empty baseline.
    pub fn from_json(text: &str) -> Result<Self, String> {
        let doc: Value = serde_json::from_str(text).map_err(|err| err.to_string())?;
        match doc["version"].as_i64() {
            Some(1) => {}
            Some(other) => return Err(format!("unsupported baseline version {other}")),
            None => return Err("baseline is missing a numeric `version`".to_string()),
        }
        let array = doc["violations"]
            .as_array()
            .ok_or_else(|| "baseline `violations` must be an array".to_string())?;
        let mut entries = Vec::with_capacity(array.len());
        for item in array {
            let field = |name: &str| -> Result<String, String> {
                item[name]
                    .as_str()
                    .map(str::to_string)
                    .ok_or_else(|| format!("baseline entry is missing string `{name}`"))
            };
            // owner/tracker are optional metadata — absent (or, tolerant of the lenient parse
            // style, non-string) reads as None; an older baseline without them parses unchanged.
            let optional = |name: &str| item[name].as_str().map(str::to_string);
            entries.push(BaselineEntry {
                id: ViolationId {
                    target: field("target")?,
                    rule: field("rule")?,
                    finding: field("finding")?,
                },
                owner: optional("owner"),
                tracker: optional("tracker"),
            });
        }
        sort_dedup_by_id(&mut entries);
        Ok(Baseline { entries })
    }
}

/// Mark each violation the baseline records as `baselined`, so it no longer fails
/// the reaction; violations absent from the baseline are left as new.
pub fn apply_baseline(report: &mut Report, baseline: &Baseline) {
    for violation in &mut report.violations {
        if baseline.contains(violation) {
            violation.baselined = true;
        }
    }
}

/// The reaction's outcome. Exit codes separate architectural drift (1) from a
/// misconfiguration (2), so a mistyped target is not reported as drift.
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub enum Outcome {
    /// No enforce-severity boundary was violated (exit 0).
    Clean,
    /// One or more boundaries were violated; carries the full report (exit 1 if any
    /// non-baselined enforce violation exists, else exit 0).
    Violations(Report),
    /// The constitution could not be evaluated — a misconfiguration or scan error
    /// (exit 2). Carries a human-readable message.
    ConstitutionError(String),
}

impl Outcome {
    /// `0` clean, warn-only, or fully baselined; `1` when a non-baselined
    /// enforce-severity violation exists; `2` for a constitution/scan error.
    pub fn exit_code(&self) -> u8 {
        match self {
            Outcome::Clean => 0,
            Outcome::Violations(report) => {
                if report.violations.iter().any(|violation| {
                    violation.severity == Severity::Enforce && !violation.baselined
                }) {
                    1
                } else {
                    0
                }
            }
            Outcome::ConstitutionError(_) => 2,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn boundary_kind_labels_cover_every_dimension() {
        assert_eq!(BoundaryKind::Crate.as_str(), "crate");
        assert_eq!(BoundaryKind::Module.as_str(), "module");
        assert_eq!(BoundaryKind::Semantic.as_str(), "semantic");
        assert_eq!(BoundaryKind::Runtime.as_str(), "runtime");
    }

    fn sample_violation() -> Violation {
        Violation::new(
            BoundaryKind::Module,
            "crate::kernel".to_string(),
            "must not import".to_string(),
            "crate::projection".to_string(),
            "the kernel must not depend on a projection".to_string(),
            Severity::Enforce,
        )
    }

    #[test]
    fn to_json_emits_the_file_key_in_both_states() {
        // Absent → explicit null (a faithful absence, distinguishable from an unknown schema).
        let without = sample_violation();
        assert_eq!(without.to_json()["file"], Value::Null);
        // Present → the string.
        let with = sample_violation().with_file(Some("src/kernel.rs".to_string()));
        assert_eq!(
            with.to_json()["file"],
            Value::String("src/kernel.rs".to_string())
        );
    }

    #[test]
    fn file_is_not_part_of_the_baseline_identity() {
        // Attaching a file must not change the (target, rule, finding) identity, so a
        // file-bearing violation still matches a baseline entry recorded without one.
        let without = sample_violation();
        let with = sample_violation().with_file(Some("src/kernel.rs".to_string()));
        assert_eq!(without.id(), with.id());
    }

    #[test]
    fn to_json_emits_the_anchor_key_in_both_states() {
        // Absent → explicit null (a faithful absence, distinguishable from an unknown schema).
        let without = sample_violation();
        assert_eq!(without.to_json()["anchor"], Value::Null);
        // Present → the string.
        let with = sample_violation().with_anchor(Some("ADR-014".to_string()));
        assert_eq!(
            with.to_json()["anchor"],
            Value::String("ADR-014".to_string())
        );
    }

    #[test]
    fn anchor_is_not_part_of_the_baseline_identity() {
        // Like `file`, the anchor is metadata: attaching it must not change the
        // (target, rule, finding) identity, so an anchored violation still matches a baseline
        // entry recorded without one and never churns an existing baseline.
        let without = sample_violation();
        let with = sample_violation().with_anchor(Some("ADR-014".to_string()));
        assert_eq!(without.id(), with.id());
    }

    #[test]
    fn to_json_emits_the_polarity_key_in_both_states() {
        // Off-axis (no polarity stamped) → explicit null, a faithful absence.
        let without = sample_violation();
        assert_eq!(without.to_json()["polarity"], Value::Null);
        // On-axis → the snake-case label.
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
        // Like `file`/`anchor`, the polarity is metadata: stamping it must not change the
        // (target, rule, finding) identity, so it never re-baselines or churns a count.
        let without = sample_violation();
        let with = sample_violation().with_polarity(Polarity::AllowlistGap);
        assert_eq!(without.id(), with.id());
    }

    #[test]
    fn baseline_round_trips_through_json() {
        // The `violation-baseline` spec's round-trip scenario: a baseline written to JSON and read
        // back holds the same `(target, rule, finding)` entries. (Previously code-correct but with
        // no dedicated test.)
        let report = Report::new(vec![
            sample_violation(),
            Violation::new(
                BoundaryKind::Crate,
                "core".to_string(),
                "deny external dependencies".to_string(),
                "serde".to_string(),
                "core stays dependency-light".to_string(),
                Severity::Enforce,
            ),
        ]);
        let original = Baseline::of(&report);
        let reparsed = Baseline::from_json(&original.to_json()).expect("round-trips");
        // Every original violation is still contained; a stale check against the same report is empty.
        assert!(reparsed.contains(&sample_violation()));
        assert!(
            reparsed.stale(&report).is_empty(),
            "no entry is stale against its own report"
        );
        // Serializing the reparsed baseline yields byte-identical JSON (stable + diffable).
        assert_eq!(reparsed.to_json(), original.to_json());
    }

    #[test]
    fn owner_and_tracker_round_trip_and_are_some_only() {
        // A hand-annotated baseline: metadata on one entry, none on the other (a pre-metadata form).
        let json = r#"{"version":1,"violations":[
            {"target":"core","rule":"r","finding":"serde","owner":"team-core","tracker":"ISSUE-7"},
            {"target":"zeta","rule":"r","finding":"tokio"}
        ]}"#;
        let baseline = Baseline::from_json(json).expect("parses (old + annotated entries)");
        let entries: Vec<&BaselineEntry> = baseline.entries().collect();
        // Sorted by identity: "core" precedes "zeta".
        assert_eq!(entries[0].id.target, "core");
        assert_eq!(entries[0].owner.as_deref(), Some("team-core"));
        assert_eq!(entries[0].tracker.as_deref(), Some("ISSUE-7"));
        assert_eq!(entries[1].owner, None);
        assert_eq!(entries[1].tracker, None);
        // Round-trip preserves metadata and is byte-stable.
        let out = baseline.to_json();
        assert_eq!(Baseline::from_json(&out).unwrap().to_json(), out);
        // Some-only: the un-annotated entry carries only the three identity keys.
        let doc: Value = serde_json::from_str(&out).unwrap();
        let zeta = &doc["violations"][1];
        assert_eq!(zeta["target"], "zeta");
        assert!(zeta.get("owner").is_none() && zeta.get("tracker").is_none());
    }

    #[test]
    fn of_preserving_carries_surviving_metadata_drops_stale_and_none_for_new() {
        let previous = Baseline::from_json(
            r#"{"version":1,"violations":[
                {"target":"core","rule":"r","finding":"serde","owner":"team-core","tracker":"ISSUE-7"},
                {"target":"gone","rule":"r","finding":"old","owner":"team-x"}
            ]}"#,
        )
        .unwrap();
        // "core" survives, "gone" is resolved, "new" appears.
        let mk = |t: &str, f: &str| {
            Violation::new(
                BoundaryKind::Crate,
                t.to_string(),
                "r".to_string(),
                f.to_string(),
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
    fn a_duplicate_identity_keeps_the_first_entry() {
        // A hand-edited baseline with the same identity twice, different owners — de-dup by id,
        // keeping the first (the recorded tie-break), never two entries for one identity.
        let baseline = Baseline::from_json(
            r#"{"version":1,"violations":[
                {"target":"core","rule":"r","finding":"serde","owner":"first"},
                {"target":"core","rule":"r","finding":"serde","owner":"second"}
            ]}"#,
        )
        .unwrap();
        let entries: Vec<&BaselineEntry> = baseline.entries().collect();
        assert_eq!(entries.len(), 1, "de-duplicated by identity");
        assert_eq!(
            entries[0].owner.as_deref(),
            Some("first"),
            "keep-first tie-break"
        );
    }

    #[test]
    fn a_malformed_or_unknown_version_baseline_is_an_error_not_empty() {
        // The spec's "never silently treat a bad baseline as empty" scenario.
        assert!(
            Baseline::from_json("{ not json").is_err(),
            "malformed JSON is an error"
        );
        assert!(
            Baseline::from_json(r#"{"version": 2, "violations": []}"#).is_err(),
            "an unknown version is an error, not a silently-empty baseline"
        );
        assert!(
            Baseline::from_json(r#"{"violations": []}"#).is_err(),
            "a missing version is an error"
        );
        // A well-formed version-1 empty baseline parses to an empty baseline (the valid empty case).
        assert!(
            Baseline::from_json(r#"{"version": 1, "violations": []}"#)
                .expect("valid empty baseline")
                .stale(&Report::empty())
                .is_empty()
        );
    }

    #[test]
    fn a_fixed_violation_leaves_a_stale_baseline_entry() {
        // The spec's stale-reporting scenario: an entry matching no current violation is stale.
        let baseline = Baseline::of(&Report::new(vec![sample_violation()]));
        let stale = baseline.stale(&Report::empty());
        assert_eq!(
            stale.len(),
            1,
            "the fixed violation's entry is reported stale"
        );
        assert_eq!(stale[0], &sample_violation().id());
    }
}
