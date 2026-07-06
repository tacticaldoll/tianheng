//! 圭表 (Guībiǎo) — the gnomon: Tianheng's static observation core. It reads the cast
//! shadow — a crate's imports and dependencies.
//!
//! The dependency-light **functional core**, derived from `modou`: declare a
//! [`Constitution`] in Rust, observe the real shape from `cargo metadata` and source
//! `use` scans, and [`check`] for drift, returning an [`Outcome`]. This crate is pure
//! observation + comparison — it carries **no** command-line, filesystem, or
//! stdout/stderr shell. The imperative shell lives in the sibling `tianheng` crate,
//! which must depend on this core and never the reverse — a crate-level invariant
//! Tianheng enforces on itself (`tianheng` workspace `tests/self_governance.rs`).
//!
//! Two reaction kinds, each with its own observation source: [`CrateBoundary`] over
//! `cargo metadata`, and [`ModuleBoundary`] over the crate's own source `use`
//! declarations. Each carries a [`Severity`]; violations gate against a [`Baseline`].
//!
//! Govern by reaction, not instruction.

#![deny(missing_docs)]

use std::path::{Path, PathBuf};

use serde_json::Value;

mod module_scan;
mod projection;
pub use projection::{constitution_json, constitution_text, report_json};
mod cargo_metadata;
pub(crate) use cargo_metadata::*;
mod crate_check;
use crate_check::check_crate_boundary;
mod coverage;
pub use coverage::Coverage;
use coverage::coverage_from;
mod errors;
use errors::unreadable_workspace_error;
#[cfg(test)]
use errors::{
    inline_module_target_error, must_not_be_imported_by_on_crate_error,
    must_only_be_imported_by_on_crate_error, restrict_imports_to_on_crate_error,
    unknown_module_error,
};
mod module_check;
use module_check::check_module_boundary;
mod model;
pub use model::*;

// The shared reaction DSL lives in the dimension-agnostic `xuanji` (璇璣) crate,
// re-exported here so `guibiao`'s public surface is unchanged after the extraction
// (PROJECT.md). Only the per-type vocabulary moved; the report/constitution *assembly*
// (projection.rs), which folds in the static `Coverage`, stays in this crate.
pub use xuanji::{
    Baseline, BaselineEntry, BoundaryKind, Outcome, Polarity, Report, Severity, Violation,
    ViolationId, apply_baseline,
};

/// Run the constitution's boundaries against the Cargo workspace at `manifest_path`.
///
/// The spine is **resolve -> observe -> compare -> react**: resolve each target to
/// a workspace package, observe (its dependencies, or its source imports), compare
/// against the rule, and return the outcome. An unresolvable target (or an
/// unreadable workspace) is a constitution error, never a silent pass.
pub fn check(constitution: &Constitution, manifest_path: &Path) -> Outcome {
    match cargo_metadata(manifest_path) {
        Ok(metadata) => evaluate(constitution, &metadata),
        Err(err) => Outcome::ConstitutionError(unreadable_workspace_error(manifest_path, &err)),
    }
}

/// Evaluate the constitution's boundaries against already-observed `cargo metadata` —
/// the compare -> react half of the spine, with the metadata read (the one IO step)
/// left to the caller so a single read can feed both evaluation and coverage (see
/// [`check_and_cover`]). An unresolvable target or a scan error is a constitution
/// error, never a silent pass.
fn evaluate(constitution: &Constitution, metadata: &Value) -> Outcome {
    let workspace = workspace_member_names(metadata);
    let mut violations = Vec::new();
    for boundary in constitution.boundaries() {
        match boundary {
            Boundary::Crate(crate_boundary) => {
                if let Err(error) =
                    check_crate_boundary(metadata, &workspace, crate_boundary, &mut violations)
                {
                    return Outcome::ConstitutionError(error);
                }
            }
            Boundary::Module(module_boundary) => {
                if let Err(error) =
                    check_module_boundary(metadata, module_boundary, &mut violations)
                {
                    return Outcome::ConstitutionError(error);
                }
            }
        }
    }

    // Two identical crate boundaries (same target/rule/kind) declared on one constitution would
    // each flag the same dependency, emitting duplicate violations with equal identity. The
    // baseline already dedups by identity, so gating is unaffected, but the report and its count
    // should not double-count a single architectural fact — dedup by `(target, rule, finding)`.
    // When duplicates differ in severity (the same rule declared once `warn` and once `enforce`
    // on one crate — a plausible mid-promotion state), keep the **more severe** reaction: `id()`
    // excludes severity, so the two collapse, and keeping first-seen would let a `warn` duplicate
    // mask an `enforce` one — silently dropping an exit-1 to exit-0, the forbidden false negative.
    // `Enforce` dominates `Warn`; `Severity` is `#[non_exhaustive]` with no `Ord`, so the
    // domination is spelled out explicitly (module rules already dedup per finding upstream).
    let mut deduped: Vec<Violation> = Vec::new();
    for violation in std::mem::take(&mut violations) {
        match deduped.iter_mut().find(|kept| kept.id() == violation.id()) {
            Some(kept) => {
                if kept.severity == Severity::Warn && violation.severity == Severity::Enforce {
                    *kept = violation;
                }
            }
            None => deduped.push(violation),
        }
    }
    violations = deduped;

    if violations.is_empty() {
        Outcome::Clean
    } else {
        Outcome::Violations(Report::new(violations))
    }
}

/// Read the target workspace once and return both the reaction outcome and workspace
/// coverage. Coverage is `Some` whenever the metadata was observed — including when the
/// outcome is a constitution error from a later boundary; the caller decides whether to
/// surface it. It is `None` only when the metadata itself could not be read. One
/// `cargo metadata` spawn feeds both, where `check` plus a separate coverage pass would
/// have spawned twice.
pub fn check_and_cover(
    constitution: &Constitution,
    manifest_path: &Path,
) -> (Outcome, Option<Coverage>) {
    let metadata = match cargo_metadata(manifest_path) {
        Ok(metadata) => metadata,
        Err(err) => {
            return (
                Outcome::ConstitutionError(unreadable_workspace_error(manifest_path, &err)),
                None,
            );
        }
    };
    let coverage = coverage_from(workspace_member_names(&metadata), constitution);
    (evaluate(constitution, &metadata), Some(coverage))
}

/// Resolve every workspace member's source-root directory from the target workspace at
/// `manifest_path`, so a caller (the 天衡 shell, composing the 漏刻 runtime CI audit) can
/// hand resolved `&Path`s to a dimension that must stay std-only and never read `cargo
/// metadata` itself. Each root is the parent of the member's `lib` (else `bin`) target
/// `src_path` — the same resolution the semantic dimension uses, not the `manifest_dir/src`
/// shortcut (which would silently miss a custom layout). An unreadable workspace is a
/// constitution error, never a silent empty set.
pub fn workspace_member_src_dirs(manifest_path: &Path) -> Result<Vec<PathBuf>, String> {
    match cargo_metadata(manifest_path) {
        Ok(metadata) => Ok(member_src_dirs(&metadata)),
        Err(err) => Err(unreadable_workspace_error(manifest_path, &err)),
    }
}

#[cfg(test)]
mod tests;
