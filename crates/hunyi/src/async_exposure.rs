//! Async-fn (implicit existential) exposure (`semantic-async-exposure-boundary`): a module's
//! public API must not declare an `async fn`. Shape-only — observed from `sig.asyncness`, no name
//! resolution.

use std::path::Path;

use serde_json::Value;
use xuanji::{Outcome, Violation};

use crate::collect::collect_item_async_exposures;
use crate::driver::run_boundaries;
use crate::dsl::AsyncExposureBoundary;
use crate::emit::{
    MultiModuleViolationContext, SingleModuleViolationContext, push_multi_module_violations,
    push_single_module_violations,
};
use crate::file_scope::resolve_crate;
use crate::resolve::collect_uses;
use crate::rules::ASYNC_EXPOSURE_RULE;
use crate::scan::walk_subtree_modules;
use crate::shape_scan::shape_module_findings;

/// Run the async-exposure boundaries against the Cargo workspace at `manifest_path`.
///
/// Mirrors [`crate::check_impl_trait`]: resolve each boundary's crate and module anchor, observe the
/// module's public-API `async fn` declarations, and react. An unresolvable crate or module (or an
/// unreadable/unparseable source) is a constitution error (exit 2). The shell composes via
/// [`crate::check_all`].
pub fn check_async_exposure(boundaries: &[AsyncExposureBoundary], manifest_path: &Path) -> Outcome {
    run_boundaries(boundaries, manifest_path, check_async_exposure_boundary)
}

pub(crate) fn check_async_exposure_boundary(
    metadata: &Value,
    boundary: &AsyncExposureBoundary,
    violations: &mut Vec<Violation>,
) -> Result<(), String> {
    let (_package, root_file, src_dir) = resolve_crate(metadata, &boundary.crate_package)?;
    let src_dir = src_dir.as_path();

    // Subtree opt-in: descend the anchored module's whole subtree, emitting per-module findings.
    // The default path governs only the anchored module's own seam (byte-identical to before).
    if boundary.including_submodules() {
        let findings = async_exposure_subtree_findings(
            src_dir,
            &root_file,
            &boundary.module,
            &boundary.crate_package,
        )?;
        push_multi_module_violations(
            violations,
            MultiModuleViolationContext {
                src_dir,
                root_file: &root_file,
                target: &boundary.module,
                crate_package: &boundary.crate_package,
                rule: ASYNC_EXPOSURE_RULE,
                reason: &boundary.reason,
                severity: boundary.severity,
                anchor: boundary.anchor(),
            },
            findings,
        );
        return Ok(());
    }

    let findings = async_exposure_module_findings(
        src_dir,
        &root_file,
        &boundary.module,
        &boundary.crate_package,
    )?;

    push_single_module_violations(
        violations,
        SingleModuleViolationContext {
            src_dir,
            root_file: &root_file,
            module: &boundary.module,
            crate_package: &boundary.crate_package,
            rule: ASYNC_EXPOSURE_RULE,
            reason: &boundary.reason,
            severity: boundary.severity,
            anchor: boundary.anchor(),
        },
        findings,
    )
}

/// The pure heart of the **subtree** async-exposure reaction: walk the anchored module's whole
/// subtree and return the sorted, deduplicated `(finding, enclosing module)` pairs — every public
/// `async fn` at or below the anchor, each attributed to the module that declares it. The subtree
/// analogue of [`async_exposure_module_findings`]: same per-module collector
/// ([`collect_item_async_exposures`], so a seam finding is byte-identical to the single-module
/// path), applied at every module the subtree walk yields.
pub(crate) fn async_exposure_subtree_findings(
    src_dir: &Path,
    root_file: &Path,
    module: &str,
    crate_package: &str,
) -> Result<Vec<(String, String)>, String> {
    let modules = walk_subtree_modules(src_dir, root_file, module, crate_package)?;
    let mut findings: Vec<(String, String)> = Vec::new();
    for (mod_path, items) in &modules {
        let uses = collect_uses(items);
        for (ordinal, item) in items.iter().enumerate() {
            let mut collected = Vec::new();
            collect_item_async_exposures(item, mod_path, &uses, ordinal, &mut collected);
            findings.extend(
                collected
                    .into_iter()
                    .map(|finding| (finding, mod_path.clone())),
            );
        }
    }
    findings.sort();
    findings.dedup();
    Ok(findings)
}

/// The pure heart of async-exposure-boundary: resolve the module's items and return the sorted,
/// deduplicated **owner-qualified** identities of the public `async fn`s it declares — public free
/// fns, public inherent methods, and public trait method declarations (observed from
/// `sig.asyncness`). Trait-*impl* methods (asyncness dictated by the trait) and private items are
/// excluded. Shape-only: no name resolution, no return-type walk.
pub(crate) fn async_exposure_module_findings(
    src_dir: &Path,
    root_file: &Path,
    module: &str,
    crate_package: &str,
) -> Result<Vec<String>, String> {
    // async collectors emit owner-qualified `String` identities directly, so the shared shape heart
    // renders with the identity function (no `shape_finding` map, unlike the dyn / impl-trait path).
    shape_module_findings(
        src_dir,
        root_file,
        module,
        crate_package,
        collect_item_async_exposures,
        |identity| identity,
    )
}
