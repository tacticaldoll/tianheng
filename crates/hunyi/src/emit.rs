use std::path::Path;

use xuanji::{BoundaryKind, Polarity, Severity, Violation};

use crate::file_scope::seam_file;

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
