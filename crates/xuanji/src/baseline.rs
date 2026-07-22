//! Baseline identity, entry, and baseline snapshot operations.

use serde_json::Value;
use std::collections::BTreeSet;

use crate::{Report, RuleKey, StructuredFactIdentity, Violation, pretty_json};

const BASELINE_FORMAT: &str = "tianheng.baseline/structured-facts";

/// A violation's stable semantic identity.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
#[non_exhaustive]
pub struct ViolationId {
    pub(crate) target: String,
    pub(crate) rule_key: RuleKey,
    pub(crate) fact: StructuredFactIdentity,
}

impl ViolationId {
    /// Build identity from governed target, semantic rule, and observed fact roles.
    pub fn new(target: impl Into<String>, rule_key: RuleKey, fact: StructuredFactIdentity) -> Self {
        Self {
            target: target.into(),
            rule_key,
            fact,
        }
    }

    /// Governed target.
    pub fn target(&self) -> &str {
        &self.target
    }

    /// Semantic rule identity.
    pub fn rule_key(&self) -> &RuleKey {
        &self.rule_key
    }

    /// Structured observed-fact identity.
    pub fn fact(&self) -> &StructuredFactIdentity {
        &self.fact
    }
}

/// One recorded baseline entry.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BaselineEntry {
    /// Accepted violation identity.
    pub id: ViolationId,
    /// Human-readable violated rule presentation.
    pub rule: String,
    /// Human-readable finding presentation.
    pub finding: String,
    /// Accepted debt owner (optional metadata).
    pub owner: Option<String>,
    /// External tracking issue (optional metadata).
    pub tracker: Option<String>,
}

fn sort_dedup_by_id(entries: &mut Vec<BaselineEntry>) {
    let mut seen = BTreeSet::new();
    entries.retain(|entry| seen.insert(entry.id.clone()));
    entries.sort_by(|a, b| a.id.cmp(&b.id));
}

/// Recorded set of accepted violations.
#[derive(Debug, Default)]
pub struct Baseline {
    entries: Vec<BaselineEntry>,
}

impl Baseline {
    /// Build baseline snapshot from report violations.
    pub fn of(report: &Report) -> Self {
        Self::of_preserving(report, &Baseline::default())
    }

    /// Build next baseline snapshot while preserving metadata by identity.
    pub fn of_preserving(report: &Report, previous: &Baseline) -> Self {
        let mut entries: Vec<BaselineEntry> = report
            .violations
            .iter()
            .map(|violation| {
                let id = violation.id();
                let prior = previous
                    .entries
                    .iter()
                    .find(|entry| baseline_id_matches(&entry.id, &id));
                BaselineEntry {
                    rule: violation.rule.clone(),
                    finding: violation.finding.clone(),
                    owner: prior.and_then(|entry| entry.owner.clone()),
                    tracker: prior.and_then(|entry| entry.tracker.clone()),
                    id,
                }
            })
            .collect();
        sort_dedup_by_id(&mut entries);
        Baseline { entries }
    }

    /// Iterator over recorded baseline entries.
    pub fn entries(&self) -> impl Iterator<Item = &BaselineEntry> {
        self.entries.iter()
    }

    /// Check whether baseline records given violation.
    pub fn contains(&self, violation: &Violation) -> bool {
        let id = violation.id();
        self.entries
            .iter()
            .any(|entry| baseline_id_matches(&entry.id, &id))
    }

    /// Baseline entries matching no current violation.
    pub fn stale(&self, report: &Report) -> Vec<&BaselineEntry> {
        let current: Vec<ViolationId> = report.violations.iter().map(Violation::id).collect();
        self.entries
            .iter()
            .filter(|entry| !current.iter().any(|id| baseline_id_matches(&entry.id, id)))
            .collect()
    }

    /// Serialize baseline to JSON document string.
    pub fn to_json(&self) -> String {
        let violations: Vec<Value> = self
            .entries
            .iter()
            .map(|entry| {
                let mut object = serde_json::json!({
                    "target": entry.id.target(),
                    "rule": entry.rule,
                    "finding": entry.finding,
                    "rule_key": entry.id.rule_key().to_json(),
                    "fact": entry.id.fact().to_json(),
                });
                if let Some(owner) = &entry.owner {
                    object["owner"] = serde_json::json!(owner);
                }
                if let Some(tracker) = &entry.tracker {
                    object["tracker"] = serde_json::json!(tracker);
                }
                object
            })
            .collect();
        let doc = serde_json::json!({ "format": BASELINE_FORMAT, "violations": violations });
        pretty_json(&doc)
    }

    /// Parse baseline from JSON document string.
    pub fn from_json(text: &str) -> Result<Self, String> {
        let doc: Value = serde_json::from_str(text).map_err(|err| err.to_string())?;
        match doc.get("format") {
            Some(Value::String(format)) if format == BASELINE_FORMAT => {}
            Some(Value::String(format)) => {
                return Err(format!("unsupported baseline format `{format}`"));
            }
            Some(_) => return Err("baseline `format` must be a string".to_string()),
            None if doc.get("version").is_some() => {
                return Err("numeric baseline versions are unsupported".to_string());
            }
            None => return Err("baseline is missing string `format`".to_string()),
        }
        let array = doc["violations"]
            .as_array()
            .ok_or_else(|| "baseline `violations` must be an array".to_string())?;
        let mut entries = Vec::with_capacity(array.len());
        for (index, item) in array.iter().enumerate() {
            let field = |name: &str| -> Result<String, String> {
                item[name]
                    .as_str()
                    .map(str::to_string)
                    .ok_or_else(|| format!("baseline entry is missing string `{name}`"))
            };
            let optional = |name: &str| -> Result<Option<String>, String> {
                match item.get(name) {
                    None | Some(Value::Null) => Ok(None),
                    Some(Value::String(value)) => Ok(Some(value.clone())),
                    Some(_) => Err(format!(
                        "baseline entry {index} `{name}` must be a string or null"
                    )),
                }
            };
            let target = field("target")?;
            let rule = field("rule")?;
            let finding = field("finding")?;
            let id = ViolationId::new(
                target,
                RuleKey::from_json(&item["rule_key"])?,
                StructuredFactIdentity::from_semantic_json(&item["fact"])?,
            );
            entries.push(BaselineEntry {
                id,
                rule,
                finding,
                owner: optional("owner")?,
                tracker: optional("tracker")?,
            });
        }
        sort_dedup_by_id(&mut entries);
        Ok(Baseline { entries })
    }
}

fn baseline_id_matches(baseline: &ViolationId, current: &ViolationId) -> bool {
    baseline == current
}

/// Mark each violation recorded in baseline as baselined.
pub fn apply_baseline(report: &mut Report, baseline: &Baseline) {
    for violation in &mut report.violations {
        if baseline.contains(violation) {
            violation.baselined = true;
        }
    }
}
