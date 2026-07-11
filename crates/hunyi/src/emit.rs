use std::collections::HashMap;
use std::path::Path;

use xuanji::{BoundaryKind, Polarity, Severity, Violation};

use crate::file_scope::{per_finding_file, seam_file};

pub(crate) struct SingleModuleViolationContext<'a> {
    pub(crate) src_dir: &'a Path,
    pub(crate) root_file: &'a Path,
    pub(crate) module: &'a str,
    pub(crate) crate_package: &'a str,
    pub(crate) rule: &'a str,
    pub(crate) reason: &'a str,
    pub(crate) severity: Severity,
    pub(crate) anchor: Option<&'a str>,
}

/// Add deny-style violations for a boundary whose findings all sit on one governed module seam.
/// The file metadata is the governed module's source file: where the exposing seam is written.
/// Finding identity stays `(target, rule, finding)`; file, anchor, and polarity remain metadata.
pub(crate) fn push_single_module_violations(
    violations: &mut Vec<Violation>,
    context: SingleModuleViolationContext<'_>,
    findings: Vec<String>,
) -> Result<(), String> {
    let module_file = seam_file(
        &findings,
        context.src_dir,
        context.root_file,
        context.module,
        context.crate_package,
    )?;
    let anchor = context.anchor.map(str::to_string);
    for finding in findings {
        violations.push(
            Violation::new(
                BoundaryKind::Semantic,
                context.module.to_string(),
                context.rule.to_string(),
                finding,
                context.reason.to_string(),
                context.severity,
            )
            .with_file(module_file.clone())
            .with_anchor(anchor.clone())
            .with_polarity(Polarity::DenyBreach),
        );
    }
    Ok(())
}

pub(crate) struct MultiModuleViolationContext<'a> {
    pub(crate) src_dir: &'a Path,
    pub(crate) root_file: &'a Path,
    /// The violation `target` — the boundary's anchored module, kept stable so identity
    /// `(target, rule, finding)` does not shift as the governed subtree grows.
    pub(crate) target: &'a str,
    pub(crate) crate_package: &'a str,
    pub(crate) rule: &'a str,
    pub(crate) reason: &'a str,
    pub(crate) severity: Severity,
    pub(crate) anchor: Option<&'a str>,
}

/// Add deny-style violations for a **subtree** boundary whose findings sit across many modules.
/// Each finding carries its enclosing module, used only to resolve the per-module source file (a
/// metadata nicety, cached across findings); the violation `target` stays the boundary's anchor, so
/// a finding's identity `(target, rule, finding)` is stable whether or not the subtree opt-in is
/// set — enabling the opt-in adds only new, deeper findings, never re-identifies the seam ones.
pub(crate) fn push_multi_module_violations(
    violations: &mut Vec<Violation>,
    context: MultiModuleViolationContext<'_>,
    findings: Vec<(String, String)>,
) {
    let anchor = context.anchor.map(str::to_string);
    let mut cache: HashMap<String, Option<String>> = HashMap::new();
    for (finding, module) in findings {
        let file = per_finding_file(
            &module,
            context.src_dir,
            context.root_file,
            context.crate_package,
            &mut cache,
        );
        violations.push(
            Violation::new(
                BoundaryKind::Semantic,
                context.target.to_string(),
                context.rule.to_string(),
                finding,
                context.reason.to_string(),
                context.severity,
            )
            .with_file(file)
            .with_anchor(anchor.clone())
            .with_polarity(Polarity::DenyBreach),
        );
    }
}
