//! Async-fn (implicit existential) exposure (`semantic-async-exposure-boundary`): a module's
//! public API must not declare an `async fn`. Shape-only — observed from `sig.asyncness`, no name
//! resolution.

use std::path::{Path, PathBuf};

use serde_json::Value;
use xuanji::{Outcome, Polarity, Violation};

use crate::collect::collect_item_async_exposures;
use crate::driver::run_boundaries;
use crate::dsl::AsyncExposureBoundary;
use crate::emit::{
    MultiModuleViolationContext, SingleModuleViolationContext, push_multi_module_violations,
    push_single_module_violations,
};
use crate::file_scope::resolve_crate;
use crate::finding::{SemanticFact, sort_attributed_facts};
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
                target: &boundary.module,
                rule: ASYNC_EXPOSURE_RULE,
                rule_key: None,
                reason: &boundary.reason,
                severity: boundary.severity,
                anchor: boundary.anchor(),
                polarity: Polarity::DenyBreach,
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
            module: &boundary.module,
            rule: ASYNC_EXPOSURE_RULE,
            rule_key: None,
            reason: &boundary.reason,
            severity: boundary.severity,
            anchor: boundary.anchor(),
        },
        findings,
    );
    Ok(())
}

/// The pure heart of the **subtree** async-exposure reaction: walk the anchored module's whole
/// subtree and return the sorted, deduplicated `(finding, enclosing module, file)` triples — every
/// public `async fn` at or below the anchor, each attributed to the module that declares it AND
/// the real file that module's own branch was resolved from (never re-resolved afterward from the
/// module string alone, which misattributes a finding once two `#[cfg]`-split branches share one
/// module path). The subtree analogue of [`async_exposure_module_findings`]: same per-module
/// collector ([`collect_item_async_exposures`], so a seam finding is byte-identical to the
/// single-module path), applied at every module the subtree walk yields.
///
/// `ordinal` is ONE counter incrementing continuously across every `(module, items, file)` tuple
/// `walk_subtree_modules` returns — never reset per tuple. `canonical_self_owner` falls back to a
/// positional `_#{ordinal}` label when an impl block's self-type is genuinely unrenderable (a
/// complex const-generic argument), and that label is `SemanticFact::AsyncInherentMethod`'s ONLY
/// disambiguator (unlike its `AsyncFreeFn`/`AsyncTraitMethod` siblings, which embed `module`
/// directly) — so two DIFFERENT tuples (two `#[cfg]`-split branches of the same anchor, or two
/// distinct descendant modules) each producing the SAME same-named-method-at-the-same-position
/// unrenderable self type must never be assigned the SAME ordinal, or their facts become
/// byte-identical and the shared fact-only dedup (`sort_attributed_facts`) silently collapses two
/// genuinely distinct violations into one — a real false negative (found on a round-11 adversarial
/// review; see `PROJECT.md`'s Decisions). Resetting per tuple (the prior behavior) also disagreed
/// with the seam path's own continuous `enumerate()` over the flattened branch union
/// (`shape_module_findings`), assigning the anchor module's own items a DIFFERENT ordinal — and
/// thus a different owner-fallback string — depending on which path observed them, contradicting
/// this function's own "byte-identical to the single-module path" doc promise for exactly the
/// unrenderable-self-type case that promise exists to cover.
pub(crate) fn async_exposure_subtree_findings(
    src_dir: &Path,
    root_file: &Path,
    module: &str,
    crate_package: &str,
) -> Result<Vec<(SemanticFact, String, PathBuf)>, String> {
    let modules = walk_subtree_modules(src_dir, root_file, module, crate_package)?;
    let mut findings: Vec<(SemanticFact, String, PathBuf)> = Vec::new();
    let mut ordinal = 0usize;
    for (mod_path, items, file) in &modules {
        let uses = collect_uses(items);
        for item in items {
            let mut collected = Vec::new();
            collect_item_async_exposures(item, mod_path, &uses, ordinal, &mut collected);
            ordinal += 1;
            findings.extend(
                collected
                    .into_iter()
                    .map(|finding| (finding, mod_path.clone(), file.clone())),
            );
        }
    }
    sort_attributed_facts(&mut findings);
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
) -> Result<Vec<(SemanticFact, PathBuf)>, String> {
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
