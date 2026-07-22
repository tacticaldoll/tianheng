//! Baseline identity, entry, and baseline snapshot operations.

use serde_json::Value;
use std::cmp::Ordering;

use crate::{Finding, FindingKey, Report, RuleKey, Violation, pretty_json};

/// A violation's baseline identity and human finding text.
#[derive(Debug, Clone)]
#[non_exhaustive]
pub struct ViolationId {
    /// Governed target.
    pub target: String,
    /// Violated rule label.
    pub rule: String,
    /// Offending finding text.
    pub finding: String,
    pub(crate) rule_key: Option<RuleKey>,
    pub(crate) finding_key: Option<FindingKey>,
}

impl ViolationId {
    /// Build identity from a dimension-owned typed finding.
    pub fn new(target: impl Into<String>, rule: impl Into<String>, finding: Finding) -> Self {
        Self {
            target: target.into(),
            rule: rule.into(),
            finding: finding.text,
            rule_key: None,
            finding_key: Some(finding.key),
        }
    }

    /// Build identity from semantic rule and fact roles during the instrument migration.
    ///
    /// This coexists with [`Self::new`] only until every production emitter migrates.
    #[doc(hidden)]
    pub fn structured(
        target: impl Into<String>,
        rule: impl Into<String>,
        rule_key: RuleKey,
        finding: Finding,
    ) -> Self {
        Self {
            target: target.into(),
            rule: rule.into(),
            finding: finding.text,
            rule_key: Some(rule_key),
            finding_key: Some(finding.key),
        }
    }

    pub(crate) fn legacy(target: String, rule: String, finding: String) -> Self {
        Self {
            target,
            rule,
            finding,
            rule_key: None,
            finding_key: None,
        }
    }

    /// Semantic rule identity, when this emitter has migrated to the 0.3 model.
    pub fn rule_key(&self) -> Option<&RuleKey> {
        self.rule_key.as_ref()
    }

    /// Stable finding key, or `None` for legacy V1 baselines.
    pub fn finding_key(&self) -> Option<&FindingKey> {
        self.finding_key.as_ref()
    }

    fn identity_cmp(&self, other: &Self) -> Ordering {
        let provenance = |id: &Self| match (&id.rule_key, &id.finding_key) {
            (None, None) => 0,
            (None, Some(_)) => 1,
            (Some(_), Some(_)) => 2,
            (Some(_), None) => {
                unreachable!("a semantic rule key always accompanies a structured fact")
            }
        };
        match provenance(self).cmp(&provenance(other)) {
            Ordering::Less => Ordering::Less,
            Ordering::Greater => Ordering::Greater,
            Ordering::Equal => match (
                &self.rule_key,
                &self.finding_key,
                &other.rule_key,
                &other.finding_key,
            ) {
                (Some(left_rule), Some(left_fact), Some(right_rule), Some(right_fact)) => (
                    &self.target,
                    left_rule,
                    left_fact,
                )
                    .cmp(&(&other.target, right_rule, right_fact)),
                (None, Some(left), None, Some(right)) => {
                    (&self.target, &self.rule, left).cmp(&(&other.target, &other.rule, right))
                }
                (None, None, None, None) => (&self.target, &self.rule, &self.finding).cmp(&(
                    &other.target,
                    &other.rule,
                    &other.finding,
                )),
                _ => unreachable!("equal provenance has the same identity shape"),
            },
        }
    }
}

impl PartialEq for ViolationId {
    fn eq(&self, other: &Self) -> bool {
        self.identity_cmp(other) == Ordering::Equal
    }
}

impl Eq for ViolationId {}

impl PartialOrd for ViolationId {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for ViolationId {
    fn cmp(&self, other: &Self) -> Ordering {
        self.identity_cmp(other)
    }
}

/// One recorded baseline entry.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BaselineEntry {
    /// Accepted violation identity.
    pub id: ViolationId,
    /// Accepted debt owner (optional metadata).
    pub owner: Option<String>,
    /// External tracking issue (optional metadata).
    pub tracker: Option<String>,
}

fn sort_dedup_by_id(entries: &mut Vec<BaselineEntry>) {
    entries.sort_by(|a, b| a.id.cmp(&b.id));
    entries.dedup_by(|a, b| a.id == b.id);
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
enum BaselineFormat {
    V1,
    #[default]
    V2,
}

/// Recorded set of accepted violations.
#[derive(Debug, Default)]
pub struct Baseline {
    entries: Vec<BaselineEntry>,
    format: BaselineFormat,
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
                    owner: prior.and_then(|entry| entry.owner.clone()),
                    tracker: prior.and_then(|entry| entry.tracker.clone()),
                    id,
                }
            })
            .collect();
        sort_dedup_by_id(&mut entries);
        Baseline {
            entries,
            format: BaselineFormat::V2,
        }
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
    pub fn stale(&self, report: &Report) -> Vec<&ViolationId> {
        let current: Vec<ViolationId> = report.violations.iter().map(Violation::id).collect();
        self.entries
            .iter()
            .filter(|entry| !current.iter().any(|id| baseline_id_matches(&entry.id, id)))
            .map(|entry| &entry.id)
            .collect()
    }

    /// Serialize baseline to JSON document string.
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
                if self.format == BaselineFormat::V2 {
                    object["finding_key"] =
                        entry.id.finding_key().map(FindingKey::to_json).expect(
                            "a version-2 baseline entry must carry a structured finding key",
                        );
                    if let Some(rule_key) = entry.id.rule_key() {
                        object["rule_key"] = rule_key.to_json();
                    }
                }
                if let Some(owner) = &entry.owner {
                    object["owner"] = serde_json::json!(owner);
                }
                if let Some(tracker) = &entry.tracker {
                    object["tracker"] = serde_json::json!(tracker);
                }
                object
            })
            .collect();
        let version = match self.format {
            BaselineFormat::V1 => 1,
            BaselineFormat::V2 => 2,
        };
        let doc = serde_json::json!({ "version": version, "violations": violations });
        pretty_json(&doc)
    }

    /// Parse baseline from JSON document string.
    pub fn from_json(text: &str) -> Result<Self, String> {
        let doc: Value = serde_json::from_str(text).map_err(|err| err.to_string())?;
        let format = match doc["version"].as_i64() {
            Some(1) => BaselineFormat::V1,
            Some(2) => BaselineFormat::V2,
            Some(other) => return Err(format!("unsupported baseline version {other}")),
            None => return Err("baseline is missing a numeric `version`".to_string()),
        };
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
            let id = match format {
                BaselineFormat::V1 => ViolationId::legacy(target, rule, finding),
                BaselineFormat::V2 => ViolationId {
                    target,
                    rule,
                    finding,
                    rule_key: item.get("rule_key").map(RuleKey::from_json).transpose()?,
                    finding_key: Some(FindingKey::from_json(&item["finding_key"])?),
                },
            };
            entries.push(BaselineEntry {
                id,
                owner: optional("owner")?,
                tracker: optional("tracker")?,
            });
        }
        sort_dedup_by_id(&mut entries);
        Ok(Baseline { entries, format })
    }
}

fn baseline_id_matches(baseline: &ViolationId, current: &ViolationId) -> bool {
    match baseline.finding_key() {
        Some(_) => baseline == current,
        None => {
            baseline.target == current.target
                && baseline.rule == current.rule
                && baseline.finding == current.finding
        }
    }
}

/// Mark each violation recorded in baseline as baselined.
pub fn apply_baseline(report: &mut Report, baseline: &Baseline) {
    for violation in &mut report.violations {
        if baseline.contains(violation) {
            violation.baselined = true;
        }
    }
}
