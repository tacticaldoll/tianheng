//! Impl-trait (existential) exposure (`semantic-impl-trait-boundary`): a module's public API must
//! not return a written `impl Trait` (RPIT). Shape-only when no operands are named; operand-scoped
//! when a forbidden set is given (resolve each returned `impl Trait`'s principal traits).

use std::path::Path;

use serde_json::Value;
use xuanji::{Outcome, Violation};

use crate::collect::collect_item_return_impl_traits;
use crate::containment::matches_forbidden;
use crate::crate_scope::{dependency_names, extern_resolution, resolve_principal};
use crate::driver::run_boundaries;
use crate::dsl::ImplTraitBoundary;
use crate::emit::{SingleModuleViolationContext, push_single_module_violations};
use crate::file_scope::resolve_crate;
use crate::finding::shape_finding;
use crate::module_resolve::resolve_module_items;
use crate::resolve::{canonical_path_str, collect_uses};
use crate::rules::IMPL_TRAIT_RULE;

/// Run the impl-trait boundaries against the Cargo workspace at `manifest_path`.
///
/// Mirrors [`crate::check_dyn_trait`]: resolve each boundary's crate and module anchor, observe the
/// module's public-API **return** positions for written `impl Trait` (RPIT) nodes at any depth,
/// and react. An unresolvable crate or module (or an unreadable/unparseable source) is a
/// constitution error (exit 2), never a silent pass. The shell composes via [`crate::check_all`].
pub fn check_impl_trait(boundaries: &[ImplTraitBoundary], manifest_path: &Path) -> Outcome {
    run_boundaries(boundaries, manifest_path, check_impl_trait_boundary)
}

pub(crate) fn check_impl_trait_boundary(
    metadata: &Value,
    boundary: &ImplTraitBoundary,
    violations: &mut Vec<Violation>,
) -> Result<(), String> {
    let (package, root_file, src_dir) = resolve_crate(metadata, &boundary.crate_package)?;
    let src_dir = src_dir.as_path();

    // Empty operand set ⇒ shape-only (any returned impl Trait), via the resolution-free path; a
    // named set ⇒ operand-scoped, resolving each returned impl Trait's principal trait.
    let findings = if boundary.forbidden_operands.is_empty() {
        impl_trait_module_findings(
            src_dir,
            &root_file,
            &boundary.module,
            &boundary.crate_package,
        )?
    } else {
        impl_trait_operand_module_findings(
            src_dir,
            &root_file,
            &boundary.module,
            &boundary.forbidden_operands,
            &boundary.crate_package,
            &dependency_names(package),
        )?
    };

    push_single_module_violations(
        violations,
        SingleModuleViolationContext {
            src_dir,
            root_file: &root_file,
            module: &boundary.module,
            crate_package: &boundary.crate_package,
            rule: IMPL_TRAIT_RULE,
            reason: &boundary.reason,
            severity: boundary.severity,
            anchor: boundary.anchor(),
        },
        findings,
    )
}

/// The pure heart of impl-trait-boundary, testable without spawning `cargo`: resolve the module's
/// items and return the sorted, deduplicated rendered `impl …` shapes appearing in a **return
/// position** of the module's public functions/methods. Shape-only, so no name resolution is
/// involved. Governs return positions only — argument-position `impl Trait` (APIT) is universal,
/// not existential, and is never visited; a trait-*impl* method's return is dictated by the trait
/// declaration (governed there), so it is excluded.
pub(crate) fn impl_trait_module_findings(
    src_dir: &Path,
    root_file: &Path,
    module: &str,
    crate_package: &str,
) -> Result<Vec<String>, String> {
    let items = resolve_module_items(src_dir, root_file, module, crate_package)?;
    // `uses` canonicalizes an inherent impl's self-type owner in the seam (see the dyn path).
    let uses = collect_uses(&items);
    let mut exposures = Vec::new();
    for (ordinal, item) in items.iter().enumerate() {
        collect_item_return_impl_traits(item, module, &uses, ordinal, &mut exposures);
    }
    let mut findings: Vec<String> = exposures.into_iter().map(shape_finding).collect();
    findings.sort();
    findings.dedup();
    Ok(findings)
}

/// The pure heart of the **operand-scoped** impl-trait boundary: like [`impl_trait_module_findings`]
/// but keeps only the returned `impl Trait` nodes **any of whose non-auto traits** resolves into
/// the forbidden operand set — a returned `impl Trait` may name several (`impl Foo + Bar`), and
/// forbidding any one flags it. The exact pipeline [`dyn_operand_module_findings`] uses
/// (`resolve_path(BareFallback::Ignore)` → `canonicalize_through_reexports` → `matches_forbidden`,
/// exact-or-module-prefix), so a re-exported/aliased trait facade matches its defining path. An
/// empty set ⇒ any returned `impl Trait` (never a silent no-op). An unresolvable trait (a bare
/// std trait, macro/glob re-export) is dropped — the stated resolver bound, never a silent pass of
/// a *resolvable* operand. The finding stays the rendered `impl …` shape (parity with shape-only).
pub(crate) fn impl_trait_operand_module_findings(
    src_dir: &Path,
    root_file: &Path,
    module: &str,
    forbidden: &[String],
    crate_package: &str,
    dep_names: &[String],
) -> Result<Vec<String>, String> {
    let items = resolve_module_items(src_dir, root_file, module, crate_package)?;
    let uses = collect_uses(&items);
    let resolution = extern_resolution(src_dir, root_file, crate_package, dep_names, &items)?;
    let forbidden: Vec<String> = forbidden.iter().map(|f| canonical_path_str(f)).collect();

    let mut exposures = Vec::new();
    for (ordinal, item) in items.iter().enumerate() {
        collect_item_return_impl_traits(item, module, &uses, ordinal, &mut exposures);
    }

    let mut findings: Vec<String> = exposures
        .into_iter()
        .filter(|exposure| {
            forbidden.is_empty()
                || exposure.principals.iter().any(|path| {
                    resolve_principal(path, &uses, module, &resolution)
                        .is_some_and(|canonical| matches_forbidden(&canonical, &forbidden))
                })
        })
        .map(shape_finding)
        .collect();
    findings.sort();
    findings.dedup();
    Ok(findings)
}
