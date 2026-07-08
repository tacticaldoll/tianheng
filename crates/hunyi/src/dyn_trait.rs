//! Dyn-trait exposure (`semantic-dyn-trait-boundary`): a module's public API must not expose
//! trait-object (`dyn`) syntax. Shape-only when no operands are named (react on the *presence* of a
//! `dyn` node); operand-scoped when a forbidden set is given (resolve each `dyn`'s principal trait).

use std::path::Path;

use serde_json::Value;
use xuanji::{Outcome, Violation};

use crate::collect::collect_item_dyn_exposures;
use crate::crate_scope::dependency_names;
use crate::driver::run_boundaries;
use crate::dsl::DynTraitBoundary;
use crate::emit::{SingleModuleViolationContext, push_single_module_violations};
use crate::file_scope::resolve_crate;
use crate::finding::shape_finding;
use crate::rules::DYN_TRAIT_RULE;
use crate::shape_scan::{operand_module_findings, shape_module_findings};

/// Run the dyn-trait boundaries against the Cargo workspace at `manifest_path`.
///
/// Mirrors [`crate::check`]: resolve each boundary's crate and module anchor, observe the module's
/// public-API surface for trait-object (`dyn`) nodes at any depth, and react. An
/// unresolvable crate or module (or an unreadable/unparseable source) is a constitution
/// error (exit 2), never a silent pass. The per-capability entry remains for direct use; the
/// shell composes via [`crate::check_all`].
pub fn check_dyn_trait(boundaries: &[DynTraitBoundary], manifest_path: &Path) -> Outcome {
    run_boundaries(boundaries, manifest_path, check_dyn_trait_boundary)
}

pub(crate) fn check_dyn_trait_boundary(
    metadata: &Value,
    boundary: &DynTraitBoundary,
    violations: &mut Vec<Violation>,
) -> Result<(), String> {
    let (package, root_file, src_dir) = resolve_crate(metadata, &boundary.crate_package)?;
    let src_dir = src_dir.as_path();

    // Empty operand set ⇒ shape-only (any dyn), using the resolution-free path unchanged; a
    // named set ⇒ operand-scoped, resolving each dyn's principal trait against the forbidden set.
    let findings = if boundary.forbidden_operands.is_empty() {
        dyn_module_findings(
            src_dir,
            &root_file,
            &boundary.module,
            &boundary.crate_package,
        )?
    } else {
        dyn_operand_module_findings(
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
            rule: DYN_TRAIT_RULE,
            reason: &boundary.reason,
            severity: boundary.severity,
            anchor: boundary.anchor(),
        },
        findings,
    )
}

/// The pure heart of dyn-trait-boundary, testable without spawning `cargo`: resolve the
/// module's items and return the sorted, deduplicated rendered `dyn` shapes exposed in its
/// public surface. The *reaction* is on the *presence* of a `dyn` node (shape-only), so it needs
/// no name resolution and no re-export closure — `pub use`-chain following is inert for a `dyn`
/// (a re-export carries a name, never a `dyn` node). The `use`-map it does collect serves only to
/// canonicalize an inherent impl's self-type **owner** in the seam (a finding-identity concern,
/// not detection); no re-export closure is needed for that.
pub(crate) fn dyn_module_findings(
    src_dir: &Path,
    root_file: &Path,
    module: &str,
    crate_package: &str,
) -> Result<Vec<String>, String> {
    shape_module_findings(
        src_dir,
        root_file,
        module,
        crate_package,
        collect_item_dyn_exposures,
        shape_finding,
    )
}

/// The pure heart of the **operand-scoped** dyn-trait boundary: like [`dyn_module_findings`]
/// but keeps only the `dyn` nodes whose **principal (non-auto) trait** resolves into the forbidden
/// operand set (a `dyn` has exactly one non-auto trait, found regardless of whether an auto trait
/// like `Send` is written before it). Unlike the shape-only path it **needs** the module's
/// `use`-map and re-export closure — the principal trait is resolved and canonicalized exactly as [`crate::module_findings`] resolves an
/// exposed type (`resolve_path(BareFallback::Ignore)` → `canonicalize_through_reexports` →
/// `matches_forbidden`, exact-or-module-prefix), so a re-exported/aliased trait facade matches
/// its defining path. A principal that does not resolve (a bare name with no `use`, a
/// macro-generated or glob/cross-crate re-exported trait) is dropped — the stated
/// resolver-coverage bound, never a silent pass of a *resolvable* operand. The finding stays the
/// rendered `dyn …` shape (parity with the shape-only rule and its baseline identity).
pub(crate) fn dyn_operand_module_findings(
    src_dir: &Path,
    root_file: &Path,
    module: &str,
    forbidden: &[String],
    crate_package: &str,
    dep_names: &[String],
) -> Result<Vec<String>, String> {
    operand_module_findings(
        src_dir,
        root_file,
        module,
        forbidden,
        crate_package,
        dep_names,
        collect_item_dyn_exposures,
    )
}
