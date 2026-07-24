//! Violation and evaluation report models.

use serde_json::Value;

use crate::{BoundaryKind, Polarity, RuleKey, Severity, StructuredFactIdentity, ViolationId};

/// One violated boundary.
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub struct Violation {
    /// Which kind of boundary produced this violation.
    pub kind: BoundaryKind,
    /// The governed target.
    pub(crate) target: String,
    /// The violated rule label.
    pub rule: String,
    /// The offending finding text.
    pub finding: String,
    /// Structured observed-fact identity.
    pub(crate) fact: StructuredFactIdentity,
    /// Semantic rule identity.
    pub(crate) rule_key: RuleKey,
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
    /// The governed target.
    pub fn target(&self) -> &str {
        &self.target
    }

    /// Build a violation observed during evaluation.
    pub fn new(
        kind: BoundaryKind,
        id: ViolationId,
        rule: impl Into<String>,
        finding: impl Into<String>,
        reason: String,
        severity: Severity,
    ) -> Self {
        let ViolationId {
            target,
            rule_key,
            fact,
        } = id;
        Violation {
            kind,
            target,
            rule: rule.into(),
            finding: finding.into(),
            rule_key,
            fact,
            reason,
            severity,
            baselined: false,
            file: None,
            anchor: None,
            polarity: None,
        }
    }

    /// The stable key for this observed finding.
    pub fn fact(&self) -> &StructuredFactIdentity {
        &self.fact
    }

    /// Semantic rule identity.
    pub fn rule_key(&self) -> &RuleKey {
        &self.rule_key
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
            rule_key: self.rule_key.clone(),
            fact: self.fact.clone(),
        }
    }

    /// Canonical JSON rendering of one violation.
    pub fn to_json(&self) -> Value {
        serde_json::json!({
            "kind": self.kind.as_str(),
            "target": self.target,
            "rule": self.rule,
            "finding": self.finding,
            "rule_key": self.rule_key.to_json(),
            "fact": self.fact.to_json(),
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
