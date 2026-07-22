use serde_json::Value;

use crate::cargo_metadata::find_package;
use crate::errors::crate_not_found_error;
use crate::{BoundaryKind, CrateBoundary, Violation, ViolationId};

pub(crate) fn check_crate_boundary(
    metadata: &Value,
    workspace: &[String],
    boundary: &CrateBoundary,
    violations: &mut Vec<Violation>,
) -> Result<(), String> {
    let package = find_package(metadata, &boundary.target.package)
        .ok_or_else(|| crate_not_found_error(&boundary.target.package))?;

    for fact in boundary.rule.facts(package, workspace, boundary.kind) {
        let finding = fact.into_finding();
        // No `with_file`: a crate-dependency violation is an edge in the dependency graph
        // (a `Cargo.toml` manifest relation), not a single source line, so its `file` is a
        // faithful `None` — the location already lives in `(target, finding)`.
        violations.push(
            Violation::new(
                BoundaryKind::Crate,
                ViolationId::new(
                    boundary.target.package.clone(),
                    boundary.rule.key(),
                    finding.key().clone(),
                ),
                boundary.rule.label(),
                finding.text(),
                boundary.reason.clone(),
                boundary.severity,
            )
            .with_anchor(boundary.anchor.clone())
            .with_polarity(boundary.rule.polarity()),
        );
    }
    Ok(())
}
