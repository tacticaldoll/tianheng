//! 璇璣 (Xuánjī) — the shared **reaction model** of Tianheng, the 底 the whole stack
//! turns on.
//!
//! The jade pivot of the armillary sphere, the instrument of celestial measure: the
//! dimension-agnostic vocabulary [`Severity`], [`BoundaryKind`], [`Violation`],
//! [`Report`], [`ViolationId`], [`Baseline`], and [`Outcome`]. Every observation
//! dimension — the static 圭表 (`guibiao`), semantic 渾儀 (`hunyi`), and runtime 漏刻
//! (`louke`) — expresses its findings in these types, so a dimension may reuse the reaction
//! vocabulary without depending on another dimension's engine.
//!
//! This crate carries the JSON (de)serialization that is **intrinsic** to its types: a
//! [`Baseline`] *is* a generated JSON snapshot, and a [`Violation`] has a canonical JSON
//! shape. It does **not** carry the report-document *assembly* (which folds in
//! dimension-specific data such as the static `Coverage`) — that stays in the consuming
//! crate. `serde_json` is its only dependency; it holds no observation engine.
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
    /// The projection label (`"crate"` / `"module"` / `"semantic"`).
    pub fn as_str(&self) -> &'static str {
        match self {
            BoundaryKind::Crate => "crate",
            BoundaryKind::Module => "module",
            BoundaryKind::Semantic => "semantic",
            BoundaryKind::Runtime => "runtime",
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

/// A recorded set of accepted violations — a generated observation snapshot, not
/// policy. The gate fails only on violations absent from it.
#[derive(Debug, Default)]
pub struct Baseline {
    entries: Vec<ViolationId>,
}

impl Baseline {
    /// Build a baseline from the current report's violations.
    pub fn of(report: &Report) -> Self {
        let mut entries: Vec<ViolationId> = report.violations.iter().map(Violation::id).collect();
        entries.sort();
        entries.dedup();
        Baseline { entries }
    }

    /// Whether this baseline records the given violation's identity.
    pub fn contains(&self, violation: &Violation) -> bool {
        let id = violation.id();
        self.entries.iter().any(|entry| entry == &id)
    }

    /// Baseline entries that match no current violation — stale, safe to remove.
    pub fn stale(&self, report: &Report) -> Vec<&ViolationId> {
        let current: Vec<ViolationId> = report.violations.iter().map(Violation::id).collect();
        self.entries
            .iter()
            .filter(|entry| !current.iter().any(|id| id == *entry))
            .collect()
    }

    /// Serialize to the on-disk JSON form: a `version` and sorted `violations`.
    pub fn to_json(&self) -> String {
        let violations: Vec<Value> = self
            .entries
            .iter()
            .map(|entry| {
                serde_json::json!({
                    "target": entry.target,
                    "rule": entry.rule,
                    "finding": entry.finding,
                })
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
            entries.push(ViolationId {
                target: field("target")?,
                rule: field("rule")?,
                finding: field("finding")?,
            });
        }
        entries.sort();
        entries.dedup();
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
}
