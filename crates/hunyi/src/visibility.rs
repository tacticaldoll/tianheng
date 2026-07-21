//! Visibility (`semantic-visibility-boundary`): a module must not declare bare-`pub` items. Scan
//! the module's direct items and react to those declared bare-`pub`.

use std::path::{Path, PathBuf};

use serde_json::Value;
use xuanji::{Outcome, Violation};

use crate::driver::run_boundaries;
use crate::dsl::VisibilityBoundary;
use crate::emit::{SingleModuleViolationContext, push_single_module_violations};
use crate::file_scope::resolve_crate;
use crate::finding::{SemanticFact, sort_faceted_facts};
use crate::module_resolve::resolve_module_items_with_files;
use crate::syn_util::item_observation;

/// Run the visibility boundaries against the Cargo workspace at `manifest_path`.
///
/// Mirrors [`crate::check`]: resolve each boundary's crate and module anchor, scan the module's
/// direct items for bare-`pub` declarations, and return the outcome. An unresolvable crate
/// or module (or an unreadable/unparseable source) is a constitution error (exit 2), never
/// a silent pass.
pub fn check_visibility(boundaries: &[VisibilityBoundary], manifest_path: &Path) -> Outcome {
    run_boundaries(boundaries, manifest_path, check_visibility_boundary)
}

pub(crate) fn check_visibility_boundary(
    metadata: &Value,
    boundary: &VisibilityBoundary,
    violations: &mut Vec<Violation>,
) -> Result<(), String> {
    let (_package, root_file, src_dir) = resolve_crate(metadata, &boundary.crate_package)?;
    let src_dir = src_dir.as_path();

    let findings = visibility_findings(
        src_dir,
        &root_file,
        &boundary.module,
        &boundary.crate_package,
        boundary.ceiling().rank(),
    )?;

    push_single_module_violations(
        violations,
        SingleModuleViolationContext {
            module: &boundary.module,
            rule: boundary.ceiling().rule(),
            reason: &boundary.reason,
            severity: boundary.severity,
            anchor: boundary.anchor(),
        },
        findings,
    );
    Ok(())
}

/// The pure heart, testable without spawning `cargo`: resolve the module's direct items and
/// return the sorted, deduplicated descriptions of those whose declared-visibility rank exceeds
/// `ceiling_rank` (the boundary's ceiling — `Crate`=2, `Super`=1, `Module`=0). Each finding pairs
/// with the real file its own item's branch was resolved from — never a single first-branch file
/// for the whole module, which would misattribute a finding produced by a non-first `#[cfg]`-split
/// branch.
pub(crate) fn visibility_findings(
    src_dir: &Path,
    root_file: &Path,
    module: &str,
    crate_package: &str,
    ceiling_rank: u8,
) -> Result<Vec<(SemanticFact, PathBuf)>, String> {
    let items_with_files =
        resolve_module_items_with_files(src_dir, root_file, module, crate_package)?;
    let mut findings: Vec<(SemanticFact, PathBuf)> = items_with_files
        .iter()
        .filter_map(|(item, file, _branch)| {
            item_observation(item, ceiling_rank).map(|obs| (obs, file))
        })
        .map(|((visibility, item_kind, item_name), file)| {
            (
                SemanticFact::Visibility {
                    visibility,
                    item_kind,
                    item_name,
                },
                file.clone(),
            )
        })
        .collect();
    sort_faceted_facts(&mut findings);
    Ok(findings)
}
