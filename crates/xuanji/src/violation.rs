//! Violation and evaluation report models.

use serde_json::Value;

use crate::{BoundaryKind, FindingKey, Polarity, RuleKey, Severity, ViolationId};

/// One violated boundary.
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub struct Violation {
    /// Which kind of boundary produced this violation.
    pub kind: BoundaryKind,
    /// The governed target.
    pub target: String,
    /// The violated rule label.
    pub rule: String,
    /// The offending finding text.
    pub finding: String,
    /// Structured identity key.
    pub(crate) finding_key: FindingKey,
    /// Semantic rule identity during the 0.3 instrument migration.
    pub(crate) rule_key: Option<RuleKey>,
    /// The boundary's reason / repair hint.
    pub reason: String,
    /// Severity level.
    pub severity: Severity,
    /// Whether violation is recorded in active baseline.
    pub baselined: bool,
    /// Offending source file path, if observed.
    pub file: Option<String>,
    /// Governance anchor pointer, if declared.
    pub anchor: Option<String>,
    /// Repair direction polarity, if applicable.
    pub polarity: Option<Polarity>,
}

impl Violation {
    /// Build a violation observed during evaluation.
    pub fn new(kind: BoundaryKind, id: ViolationId, reason: String, severity: Severity) -> Self {
        let ViolationId {
            target,
            rule,
            finding,
            rule_key,
            finding_key,
        } = id;
        Violation {
            kind,
            target,
            rule,
            finding,
            rule_key,
            finding_key: finding_key.expect(
                "a live violation requires a structured finding; a version-1 baseline id is legacy data, not an observed fact",
            ),
            reason,
            severity,
            baselined: false,
            file: None,
            anchor: None,
            polarity: None,
        }
    }

    /// The stable key for this observed finding.
    pub fn finding_key(&self) -> &FindingKey {
        &self.finding_key
    }

    /// Semantic rule identity, when the producing instrument has migrated.
    pub fn rule_key(&self) -> Option<&RuleKey> {
        self.rule_key.as_ref()
    }

    /// Attach source file metadata.
    pub fn with_file(mut self, file: Option<String>) -> Self {
        self.file = file;
        self
    }

    /// Attach governance anchor metadata.
    pub fn with_anchor(mut self, anchor: Option<String>) -> Self {
        self.anchor = anchor;
        self
    }

    /// Attach repair-direction polarity metadata.
    pub fn with_polarity(mut self, polarity: Polarity) -> Self {
        self.polarity = Some(polarity);
        self
    }

    /// Structured identity matching key for baselining.
    pub fn id(&self) -> ViolationId {
        ViolationId {
            target: self.target.clone(),
            rule: self.rule.clone(),
            finding: self.finding.clone(),
            rule_key: self.rule_key.clone(),
            finding_key: Some(self.finding_key.clone()),
        }
    }

    /// Canonical JSON rendering of one violation.
    pub fn to_json(&self) -> Value {
        serde_json::json!({
            "kind": self.kind.as_str(),
            "target": self.target,
            "rule": self.rule,
            "finding": self.finding,
            "finding_key": self.finding_key.to_json(),
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
    /// Every violation found in evaluation.
    pub violations: Vec<Violation>,
}

impl Report {
    /// An empty report.
    pub fn empty() -> Self {
        Report {
            violations: Vec::new(),
        }
    }

    /// Construct a report containing given violations.
    pub fn new(violations: Vec<Violation>) -> Self {
        Report { violations }
    }
}
