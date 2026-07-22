//! Unsafe-confinement (`semantic-unsafe-confinement`): `unsafe` (blocks, `fn`, `impl`, `trait`,
//! `unsafe extern`) may appear only under a declared subtree. Walk the whole crate for `unsafe`
//! sites and react to those whose module lies outside the allowed set. Confinement-only: an empty
//! or crate-root allowed set is a constitution error (the crate-wide ban is `#![forbid]`'s job).

use std::path::{Path, PathBuf};

use serde_json::Value;
use xuanji::{Outcome, Polarity, RuleKey, Violation};

use crate::containment::matches_allowed;
use crate::driver::run_boundaries;
use crate::dsl::UnsafeBoundary;
use crate::emit::{MultiModuleViolationContext, push_multi_module_violations};
use crate::errors::{unsafe_crate_root_allowed_error, unsafe_empty_allowed_error};
use crate::file_scope::resolve_crate;
use crate::finding::{SemanticFact, sort_attributed_facts};
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
    let mut key_allowed = allowed.clone();
    key_allowed.sort_unstable();
    key_allowed.dedup();
    let allowed_key = serde_json::to_string(&key_allowed)
        .expect("a list of canonical module paths always serializes");

    // Human rule text stays fixed, while the semantic rule key carries the canonical allowed set:
    // changing where unsafe is permitted changes the law and therefore re-keys the reaction. The
    // violation target remains the crate package (the confinement scope).
    // The shared emit helper resolves each finding's module source file and stamps the
    // allowlist-gap polarity.
    push_multi_module_violations(
        violations,
        MultiModuleViolationContext {
            target: &boundary.crate_package,
            rule: UNSAFE_CONFINEMENT_RULE,
            rule_key: Some(RuleKey::of(
                "tianheng.rule/hunyi/unsafe-confinement",
                [("allowed", allowed_key)],
            )),
            reason: &boundary.reason,
            severity: boundary.severity,
            anchor: boundary.anchor(),
            polarity: Polarity::AllowlistGap,
        },
        findings,
    );
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
) -> Result<Vec<(SemanticFact, String, PathBuf)>, String> {
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
    let mut findings: Vec<(SemanticFact, String, PathBuf)> = sites
        .into_iter()
        .filter(|site| !matches_allowed(&site.module, allowed))
        .map(|site| {
            let module = site.module;
            (
                SemanticFact::UnsafeSite {
                    module: module.clone(),
                    site: site.site,
                },
                module,
                site.file,
            )
        })
        .collect();
    sort_attributed_facts(&mut findings);
    Ok(findings)
}
