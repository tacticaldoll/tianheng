//! Impl-trait (existential) exposure (`semantic-impl-trait-boundary`): a module's public API must
//! not return a written `impl Trait` (RPIT). Shape-only when no operands are named; operand-scoped
//! when a forbidden set is given (resolve each returned `impl Trait`'s principal traits).

use std::path::{Path, PathBuf};

use serde_json::Value;
use xuanji::{Outcome, Polarity, Violation};

use crate::collect::collect_item_return_impl_traits;
use crate::crate_scope::{dependency_names, extern_resolution, file_extern_scope};
use crate::driver::run_boundaries;
use crate::dsl::ImplTraitBoundary;
use crate::emit::{
    MultiModuleViolationContext, SingleModuleViolationContext, push_multi_module_violations,
    push_single_module_violations,
};
use crate::file_scope::resolve_crate;
use crate::finding::{ExposureKind, SemanticFact, shape_finding, sort_attributed_facts};
use crate::resolve::{canonical_path_str, collect_uses};
use crate::rules::IMPL_TRAIT_RULE;
use crate::scan::walk_subtree_modules;
use crate::shape_scan::{
    matches_forbidden_principal, operand_module_findings, shape_module_findings,
};

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
    let rule_key = boundary.rule_key();

    // Subtree opt-in: descend the anchored module's whole subtree, emitting per-module findings.
    // The default path governs only the anchored module's own seam (byte-identical to before).
    if boundary.including_submodules() {
        let findings = if boundary.forbidden_operands.is_empty() {
            impl_trait_subtree_findings(
                src_dir,
                &root_file,
                &boundary.module,
                &boundary.crate_package,
            )?
        } else {
            impl_trait_operand_subtree_findings(
                src_dir,
                &root_file,
                &boundary.module,
                &boundary.forbidden_operands,
                &boundary.crate_package,
                &dependency_names(package),
            )?
        };
        push_multi_module_violations(
            violations,
            MultiModuleViolationContext {
                target: &boundary.module,
                rule: IMPL_TRAIT_RULE,
                rule_key,
                reason: &boundary.reason,
                severity: boundary.severity,
                anchor: boundary.anchor(),
                polarity: Polarity::DenyBreach,
            },
            findings,
        );
        return Ok(());
    }

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
            module: &boundary.module,
            rule: IMPL_TRAIT_RULE,
            rule_key,
            reason: &boundary.reason,
            severity: boundary.severity,
            anchor: boundary.anchor(),
        },
        findings,
    );
    Ok(())
}

/// The pure heart of the **subtree** impl-trait reaction: walk the anchored module's whole subtree
/// and return the sorted, deduplicated `(finding, enclosing module, file)` triples — every returned
/// `impl Trait` at or below the anchor, each attributed to the module that declares it AND the real
/// file that module's own branch was resolved from (never re-resolved afterward from the module
/// string alone, which misattributes a finding once two `#[cfg]`-split branches share one module
/// path). The subtree analogue of [`impl_trait_module_findings`]: same per-item collector
/// ([`collect_item_return_impl_traits`], so a seam finding is byte-identical to the single-module
/// path), applied at every module the subtree walk yields.
///
/// The `ordinal` passed to the collector is ONE counter incrementing continuously across every
/// item the subtree walk yields — never reset per module or per branch — because (unlike async's
/// collector, which ignores it) impl-trait's owner resolution can fall back to an ordinal-keyed
/// positional sentinel for a genuinely unrenderable self type. That sentinel is never published as
/// identity regardless (`reject_positional_identity`, invoked by [`sort_attributed_facts`] below,
/// fails the whole reaction loud the moment any sentinel-bearing fact appears), but a non-unique
/// ordinal would still let two genuinely distinct unrenderable sites collide into one internal
/// value before that gate ever runs — thread it correctly rather than relying on the gate alone.
pub(crate) fn impl_trait_subtree_findings(
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
            collect_item_return_impl_traits(item, mod_path, &uses, ordinal, &mut collected);
            ordinal += 1;
            findings.extend(collected.into_iter().map(|exposure| {
                (
                    shape_finding(exposure, ExposureKind::ImplTrait),
                    mod_path.clone(),
                    file.clone(),
                )
            }));
        }
    }
    sort_attributed_facts(&mut findings)?;
    Ok(findings)
}

/// Operand-scoped subtree analogue of [`impl_trait_operand_module_findings`]. Module traversal
/// remains owned by [`walk_subtree_modules`]; each returned branch builds its own `use` and
/// extern-shadow scope while sharing one crate-wide extern/re-export resolution. This keeps
/// mutually-exclusive cfg branches isolated and applies the same principal matcher as the
/// single-module operand reaction.
pub(crate) fn impl_trait_operand_subtree_findings(
    src_dir: &Path,
    root_file: &Path,
    module: &str,
    forbidden: &[String],
    crate_package: &str,
    dep_names: &[String],
) -> Result<Vec<(SemanticFact, String, PathBuf)>, String> {
    let modules = walk_subtree_modules(src_dir, root_file, module, crate_package)?;
    let resolution = extern_resolution(src_dir, root_file, crate_package, dep_names)?;
    let forbidden: Vec<String> = forbidden
        .iter()
        .map(|path| canonical_path_str(path))
        .collect();
    let mut findings = Vec::new();
    let mut ordinal = 0usize;

    for (mod_path, items, file) in &modules {
        let uses = collect_uses(items);
        let file_scope = file_extern_scope(&resolution, items);
        for item in items {
            let mut collected = Vec::new();
            collect_item_return_impl_traits(item, mod_path, &uses, ordinal, &mut collected);
            ordinal += 1;
            findings.extend(
                collected
                    .into_iter()
                    .filter(|exposure| {
                        matches_forbidden_principal(
                            exposure,
                            &uses,
                            mod_path,
                            &resolution,
                            &file_scope,
                            &forbidden,
                        )
                    })
                    .map(|exposure| {
                        (
                            shape_finding(exposure, ExposureKind::ImplTrait),
                            mod_path.clone(),
                            file.clone(),
                        )
                    }),
            );
        }
    }

    sort_attributed_facts(&mut findings)?;
    Ok(findings)
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
) -> Result<Vec<(SemanticFact, PathBuf)>, String> {
    shape_module_findings(
        src_dir,
        root_file,
        module,
        crate_package,
        |item, module, uses, ordinal, out| {
            collect_item_return_impl_traits(item, module, uses, ordinal, out);
            Ok(())
        },
        |exposure| shape_finding(exposure, ExposureKind::ImplTrait),
    )
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
) -> Result<Vec<(SemanticFact, PathBuf)>, String> {
    operand_module_findings(
        src_dir,
        root_file,
        module,
        forbidden,
        crate_package,
        dep_names,
        (ExposureKind::ImplTrait, collect_item_return_impl_traits),
    )
}
