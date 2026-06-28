//! The projections: the text and JSON renderings of an [`Outcome`] and a
//! [`Constitution`]. A projection is a faithful, self-describing view of the model
//! for humans (text) and machines (JSON) — it adds no policy and makes no decision
//! (PROJECT.md). The *per-type* serialization (a [`Violation`] → JSON, the
//! infallible `pretty_json`) lives in the dimension-agnostic `xuanji` crate; this
//! module assembles the *document* — folding in the static [`Coverage`] and stale
//! baseline entries — which is why it stays here in the engine, not in the model.

use super::*;
use serde_json::Value;
use xuanji::pretty_json;

/// Render the outcome as a JSON document for machine consumption: a faithful
/// projection of [`Outcome`] with each violation's `kind`, the boundary `reason` as
/// the repair hint, and `exit_code` mirroring the process exit. `stale` lists
/// baseline entries matching no current violation (empty outside gate mode).
pub fn report_json(
    outcome: &Outcome,
    stale: &[ViolationId],
    coverage: Option<&Coverage>,
) -> String {
    let (label, violations, error) = match outcome {
        Outcome::Clean => ("clean", Vec::new(), Value::Null),
        Outcome::Violations(report) => (
            "violations",
            report.violations.iter().map(Violation::to_json).collect(),
            Value::Null,
        ),
        Outcome::ConstitutionError(message) => (
            "constitution_error",
            Vec::new(),
            Value::String(message.clone()),
        ),
        // `Outcome` is `#[non_exhaustive]` (it lives in 璇璣, shared across dimensions).
        // A future variant this engine does not yet produce renders as a neutral label;
        // the projection decides nothing — `exit_code()` (authoritative, in 璇璣) governs
        // the reaction — so this fallback is a faithful "don't know", never a false pass.
        _ => ("unknown", Vec::new(), Value::Null),
    };
    let stale_baseline: Vec<Value> = stale
        .iter()
        .map(
            |id| serde_json::json!({ "target": id.target, "rule": id.rule, "finding": id.finding }),
        )
        .collect();
    let mut document = serde_json::json!({
        "outcome": label,
        "exit_code": outcome.exit_code(),
        "violations": violations,
        "stale_baseline": stale_baseline,
        "error": error,
    });
    if let Some(coverage) = coverage {
        document["coverage"] = serde_json::json!({
            "workspace_crates": coverage.total,
            "uncovered": coverage.uncovered,
        });
    }
    pretty_json(&document)
}

/// Render the declared constitution as a human-readable projection — the law as
/// code declares it, for a steward reviewing an amendment or an operator reading a
/// CI log. A projection of the Rust source of truth, never a second source and never
/// a reaction. An empty constitution renders its name and `(0 boundaries)`.
pub fn constitution_text(constitution: &Constitution) -> String {
    let boundaries = constitution.boundaries();
    let noun = if boundaries.len() == 1 {
        "boundary"
    } else {
        "boundaries"
    };
    let mut out = format!(
        "Constitution: {}  ({} {noun})\n",
        constitution.name(),
        boundaries.len()
    );
    for boundary in boundaries {
        let (severity, target, rule, reason) = match boundary {
            Boundary::Crate(b) => {
                let rule = match dependency_kind_label(b.dependency_kind()) {
                    Some(kind) => format!("{} ({kind} dependencies)", b.rule().text()),
                    None => b.rule().text(),
                };
                (
                    b.severity(),
                    format!("crate {}", b.target().package),
                    rule,
                    b.reason(),
                )
            }
            Boundary::Module(b) => (
                b.severity,
                format!("module {} in {}", b.module, b.crate_package),
                b.rule.text(),
                b.reason.as_str(),
            ),
        };
        out.push_str(&format!(
            "\n[{}] {target}\n  rule:   {rule}\n  reason: {reason}\n",
            severity.as_str()
        ));
    }
    out
}

/// Render the declared constitution as a JSON projection: a `constitution` name and
/// a `boundaries` array. Each entry carries `kind`, `target` (the crate name, or the
/// module path for a module boundary — the same convention as a violation's
/// `target`), `severity`, `reason`, and the rule with its parameters. No field is
/// invented for data the constitution does not hold.
pub fn constitution_json(constitution: &Constitution) -> String {
    let boundaries: Vec<Value> = constitution
        .boundaries()
        .iter()
        .map(boundary_json)
        .collect();
    let document = serde_json::json!({
        "constitution": constitution.name(),
        "boundaries": boundaries,
    });
    pretty_json(&document)
}

fn boundary_json(boundary: &Boundary) -> Value {
    match boundary {
        Boundary::Crate(b) => {
            let mut object = serde_json::json!({
                "kind": "crate",
                "target": b.target().package,
                "rule": b.rule().label(),
                "severity": b.severity().as_str(),
                "reason": b.reason(),
            });
            for (key, value) in b.rule().json_params() {
                object[key] = value;
            }
            if let Some(kind) = dependency_kind_label(b.dependency_kind()) {
                object["dependency_kind"] = serde_json::json!(kind);
            }
            object
        }
        Boundary::Module(b) => {
            let mut object = serde_json::json!({
                "kind": "module",
                "target": b.module,
                "crate": b.crate_package,
                "rule": b.rule.label(),
                "severity": b.severity.as_str(),
                "reason": b.reason,
            });
            for (key, value) in b.rule.json_params() {
                object[key] = value;
            }
            object
        }
    }
}

/// The projection label for a non-default dependency kind, or `None` for `Normal` so
/// the common projection is unchanged.
fn dependency_kind_label(kind: DependencyKind) -> Option<&'static str> {
    match kind {
        DependencyKind::Normal => None,
        DependencyKind::Dev => Some("dev"),
        DependencyKind::Build => Some("build"),
    }
}
