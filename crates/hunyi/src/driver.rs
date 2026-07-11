//! The reaction spine shared by every semantic capability: read the workspace once, evaluate a
//! capability's boundaries into one accumulator, and fold the result into an [`Outcome`] with the
//! 0/1/2 exit-code contract. Foundation below the capability modules — it names no capability, so
//! each capability depends *down* on it (no cycle).

use std::path::Path;

use serde_json::Value;
use xuanji::{Outcome, Report, Severity, Violation};

use crate::errors::unreadable_workspace_error;
use xingbiao::cargo_metadata;

/// Fold accumulated violations into an outcome: `Clean` when none, else `Violations`.
///
/// Two boundaries of the same capability on the same module can emit an identical `ViolationId`
/// (`target, rule, finding`) — a plausible mid-promotion state (one `.warn()`, one enforce), or two
/// overlapping forbidden sets. Collapse them by id, keeping the **more severe** reaction (Enforce
/// dominates Warn), so one architectural fact is reported once and the baseline-suppressed count is
/// honest. Keeping the more severe is what stops a `warn` duplicate from masking an `enforce` one.
/// This mirrors the 圭表 static dimension's dedup; each dimension owns its copy (三儀 ⊥ 三儀).
pub(crate) fn outcome_from(violations: Vec<Violation>) -> Outcome {
    let mut deduped: Vec<Violation> = Vec::new();
    for violation in violations {
        match deduped.iter_mut().find(|kept| kept.id() == violation.id()) {
            Some(kept) => {
                if kept.severity == Severity::Warn && violation.severity == Severity::Enforce {
                    *kept = violation;
                }
            }
            None => deduped.push(violation),
        }
    }
    if deduped.is_empty() {
        Outcome::Clean
    } else {
        Outcome::Violations(Report::new(deduped))
    }
}

/// Read `cargo metadata` for the workspace, mapping an unreadable workspace to the shared
/// constitution error (exit 2) — the single-read gate every semantic reaction opens with.
pub(crate) fn read_metadata(manifest_path: &Path) -> Result<Value, Outcome> {
    cargo_metadata(manifest_path)
        .map_err(|err| Outcome::ConstitutionError(unreadable_workspace_error(manifest_path, &err)))
}

/// Evaluate one capability's boundaries against an already-read `metadata`, folding findings into
/// the shared `violations` accumulator; the first constitution error short-circuits (exit 2
/// supersedes any accumulated drift). Shared by the single-capability `check_*` drivers and
/// `check_all` — the latter reads `metadata` **once** and evaluates all eight capabilities into
/// one accumulator, so the single-read and error-supersedes semantics are identical across both.
pub(crate) fn eval_into<B>(
    metadata: &Value,
    boundaries: &[B],
    per_boundary: impl Fn(&Value, &B, &mut Vec<Violation>) -> Result<(), String>,
    violations: &mut Vec<Violation>,
) -> Result<(), String> {
    for boundary in boundaries {
        per_boundary(metadata, boundary, violations)?;
    }
    Ok(())
}

/// A single-capability reaction: one `cargo metadata` read, evaluate every boundary, react. The
/// spine every per-capability `check_*` entry shares — a constitution error supersedes (exit 2),
/// otherwise `Clean`/`Violations` (exit 0/1).
pub(crate) fn run_boundaries<B>(
    boundaries: &[B],
    manifest_path: &Path,
    per_boundary: impl Fn(&Value, &B, &mut Vec<Violation>) -> Result<(), String>,
) -> Outcome {
    let metadata = match read_metadata(manifest_path) {
        Ok(metadata) => metadata,
        Err(outcome) => return outcome,
    };
    let mut violations = Vec::new();
    match eval_into(&metadata, boundaries, per_boundary, &mut violations) {
        Ok(()) => outcome_from(violations),
        Err(error) => Outcome::ConstitutionError(error),
    }
}
