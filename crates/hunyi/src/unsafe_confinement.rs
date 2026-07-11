//! Unsafe-confinement (`semantic-unsafe-confinement`): `unsafe` (blocks, `fn`, `impl`, `trait`,
//! `unsafe extern`) may appear only under a declared subtree. Walk the whole crate for `unsafe`
//! sites and react to those whose module lies outside the allowed set. Confinement-only: an empty
//! or crate-root allowed set is a constitution error (the crate-wide ban is `#![forbid]`'s job).

use std::collections::HashMap;
use std::path::Path;

use serde_json::Value;
use xuanji::{BoundaryKind, Outcome, Polarity, Violation};

use crate::containment::matches_allowed;
use crate::driver::run_boundaries;
use crate::dsl::UnsafeBoundary;
use crate::errors::{unsafe_crate_root_allowed_error, unsafe_empty_allowed_error};
use crate::file_scope::{per_finding_file, resolve_crate};
use crate::resolve::canonical_path_str;
use crate::rules::UNSAFE_CONFINEMENT_RULE;
use crate::scan::scan_unsafe_sites;

/// Run the unsafe-confinement boundaries against the Cargo workspace at `manifest_path`.
///
/// Mirrors [`crate::check`]: resolve each boundary's crate, walk it for `unsafe` sites, react to
/// those whose module is outside the allowed subtree(s), and return the outcome. An unresolvable
/// crate, an empty / crate-root allowed set, or an unreadable/unparseable source is a constitution
/// error (exit 2), never a silent pass.
pub fn check_unsafe_confinement(boundaries: &[UnsafeBoundary], manifest_path: &Path) -> Outcome {
    run_boundaries(boundaries, manifest_path, check_unsafe_boundary)
}

pub(crate) fn check_unsafe_boundary(
    metadata: &Value,
    boundary: &UnsafeBoundary,
    violations: &mut Vec<Violation>,
) -> Result<(), String> {
    let (_package, root_file, src_dir) = resolve_crate(metadata, &boundary.crate_package)?;
    let src_dir = src_dir.as_path();

    let allowed: Vec<String> = boundary
        .allowed_locations
        .iter()
        .map(|a| canonical_path_str(a))
        .collect();
    let findings = unsafe_findings(src_dir, &root_file, &allowed, &boundary.crate_package)?;

    // A fixed rule string: the allowed subtree(s) are policy configuration (surfaced in the `list`
    // projection and the reason), not part of the violation's identity — editing the allowed set
    // does not turn a still-misplaced site into a "new" violation against a baseline (mirroring
    // trait-impl-locality).
    let rule = UNSAFE_CONFINEMENT_RULE.to_string();
    let mut file_cache: HashMap<String, Option<String>> = HashMap::new();
    for (finding, module) in findings {
        let file = per_finding_file(
            &module,
            src_dir,
            &root_file,
            &boundary.crate_package,
            &mut file_cache,
        );
        violations.push(
            Violation::new(
                BoundaryKind::Semantic,
                boundary.crate_package.clone(),
                rule.clone(),
                finding,
                boundary.reason.clone(),
                boundary.severity,
            )
            .with_file(file)
            .with_anchor(boundary.anchor.clone())
            .with_polarity(Polarity::AllowlistGap),
        );
    }
    Ok(())
}

/// The pure heart, testable without spawning `cargo`: scan the whole crate for `unsafe` sites and
/// return the sorted, deduplicated findings — each site whose module is outside every allowed
/// subtree, as `("{label} in {module}", module)`. An anonymous `unsafe {}` block is module-granular
/// (`unsafe block in {module}`), so N blocks in one module dedup to one stable finding.
pub(crate) fn unsafe_findings(
    src_dir: &Path,
    root_file: &Path,
    allowed: &[String],
    crate_package: &str,
) -> Result<Vec<(String, String)>, String> {
    // Confinement-only, enforced loud (exit 2): the crate-wide "no unsafe" case is #![forbid]'s,
    // and an allowed set naming the crate root could never react. Guarded here (the pure heart) so
    // the rejection is testable without spawning `cargo`.
    if allowed.is_empty() {
        return Err(unsafe_empty_allowed_error(crate_package));
    }
    if allowed.iter().any(|a| a == "crate") {
        return Err(unsafe_crate_root_allowed_error(crate_package));
    }
    let sites = scan_unsafe_sites(src_dir, root_file, crate_package)?;
    let mut findings: Vec<(String, String)> = sites
        .into_iter()
        .filter(|site| !matches_allowed(&site.module, allowed))
        .map(|site| (format!("{} in {}", site.label, site.module), site.module))
        .collect();
    findings.sort();
    findings.dedup_by(|a, b| a.0 == b.0);
    Ok(findings)
}
