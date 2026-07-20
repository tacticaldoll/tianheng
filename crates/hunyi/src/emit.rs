use std::collections::HashMap;
use std::path::Path;

use xuanji::{BoundaryKind, Polarity, Severity, Violation, ViolationId};

use crate::file_scope::{per_finding_file, seam_file};
use crate::finding::SemanticFact;

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
/// Identity stays `(target, rule, finding_key)`; text, file, anchor, and polarity remain metadata.
pub(crate) fn push_single_module_violations(
    violations: &mut Vec<Violation>,
    context: SingleModuleViolationContext<'_>,
    findings: Vec<SemanticFact>,
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
                ViolationId::new(context.module, context.rule, finding.into_finding()),
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
    /// `(target, rule, finding_key)` does not shift as the governed subtree grows.
    pub(crate) target: &'a str,
    pub(crate) crate_package: &'a str,
    pub(crate) rule: &'a str,
    pub(crate) reason: &'a str,
    pub(crate) severity: Severity,
    pub(crate) anchor: Option<&'a str>,
    /// The finding's polarity metadata (deny-breach vs allowlist-gap). Not part of the violation
    /// identity, so each capability passes its own without shifting structured identity.
    pub(crate) polarity: Polarity,
}

/// Add violations for a boundary whose findings sit across many modules — the shared emitter for
/// every whole-crate-scan capability (forbidden-marker, trait-impl, unsafe-confinement, and the
/// async-exposure subtree branch), of either polarity: each caller supplies its own `polarity` via
/// the context. Each finding carries its enclosing module, used only to resolve the per-module
/// source file (a metadata nicety, cached across findings); the violation `target` stays the
/// boundary's anchor, so a finding's structured identity is stable — the enclosing
/// module is metadata, never part of the identity.
pub(crate) fn push_multi_module_violations(
    violations: &mut Vec<Violation>,
    context: MultiModuleViolationContext<'_>,
    findings: Vec<(SemanticFact, String)>,
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
                ViolationId::new(context.target, context.rule, finding.into_finding()),
                context.reason.to_string(),
                context.severity,
            )
            .with_file(file)
            .with_anchor(anchor.clone())
            .with_polarity(context.polarity),
        );
    }
}
