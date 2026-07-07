//! Async-fn (implicit existential) exposure (`semantic-async-exposure-boundary`): a module's
//! public API must not declare an `async fn`. Shape-only — observed from `sig.asyncness`, no name
//! resolution.

use std::path::Path;

use serde_json::Value;
use xuanji::{Outcome, Violation};

use crate::collect::collect_item_async_exposures;
use crate::driver::run_boundaries;
use crate::dsl::AsyncExposureBoundary;
use crate::emit::{SingleModuleViolationContext, push_single_module_violations};
use crate::file_scope::resolve_crate;
use crate::module_resolve::resolve_module_items;
use crate::resolve::collect_uses;
use crate::rules::ASYNC_EXPOSURE_RULE;

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
    let items = resolve_module_items(src_dir, root_file, module, crate_package)?;
    // `uses` canonicalizes an inherent impl's self-type owner in the seam (see the dyn path).
    let uses = collect_uses(&items);
    let mut found = Vec::new();
    for (ordinal, item) in items.iter().enumerate() {
        collect_item_async_exposures(item, module, &uses, ordinal, &mut found);
    }
    found.sort();
    found.dedup();
    Ok(found)
}
