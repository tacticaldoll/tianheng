//! Shared scan hearts for the existential-exposure boundaries (dyn-trait, impl-trait,
//! async-exposure). Each boundary's per-module finding computation is the same skeleton — resolve
//! the module's items, walk each with a per-item collector, then render, sort, and dedup — differing
//! only in the collector (and, for the operand-scoped path, an added principal-resolution filter).
//! These two helpers hold that skeleton once so the three boundary modules pass only their
//! collector, rather than each re-implementing the identical pipeline.

use std::path::Path;

use crate::containment::matches_forbidden;
use crate::crate_scope::{extern_resolution, resolve_principal};
use crate::finding::{ExposureKind, SemanticFact, shape_finding, sort_facts};
use crate::module_resolve::resolve_module_items;
use crate::resolve::{ShapeExposure, UseMap, canonical_path_str, collect_uses};

/// Resolve `module`'s items, collect each item's exposures with `collect`, render each to a finding
/// with `render`, then sort + dedup. The shape-only heart shared by the dyn / impl-trait / async
/// boundaries: `collect` is the only per-boundary difference. The dyn and impl-trait boundaries
/// collect [`ShapeExposure`] and pass [`shape_finding`] as `render`; async collects owner-qualified
/// `String` identities directly and passes the identity `render`. `uses` is collected because the
/// collectors canonicalize an inherent impl's self-type owner in the seam (a finding-identity
/// concern), even where the boundary itself needs no name resolution.
pub(crate) fn shape_module_findings<E>(
    src_dir: &Path,
    root_file: &Path,
    module: &str,
    crate_package: &str,
    collect: impl Fn(&syn::Item, &str, &UseMap, usize, &mut Vec<E>),
    render: impl Fn(E) -> SemanticFact,
) -> Result<Vec<SemanticFact>, String> {
    let items = resolve_module_items(src_dir, root_file, module, crate_package)?;
    let uses = collect_uses(&items);
    let mut collected = Vec::new();
    for (ordinal, item) in items.iter().enumerate() {
        collect(item, module, &uses, ordinal, &mut collected);
    }
    let mut findings: Vec<SemanticFact> = collected.into_iter().map(render).collect();
    sort_facts(&mut findings);
    Ok(findings)
}

/// The operand-scoped heart shared by the dyn / impl-trait boundaries: like
/// [`shape_module_findings`] over [`ShapeExposure`], but additionally resolves each exposure's
/// principal traits and keeps only those any of whose principal resolves into `forbidden` (via
/// `resolve_principal` → `matches_forbidden`, exact-or-module-prefix, so a re-exported/aliased trait
/// facade matches its defining path). An **empty** forbidden set keeps every exposure — the
/// shape-only semantic, never a silent no-op, safe even if a caller routes an empty set here. An
/// unresolvable principal (a bare std trait, macro/glob re-export) is dropped: the stated
/// resolver-coverage bound, never a silent pass of a *resolvable* operand. `collect` is the only
/// per-boundary difference; the finding stays the rendered shape (parity with the shape-only rule).
pub(crate) fn operand_module_findings(
    src_dir: &Path,
    root_file: &Path,
    module: &str,
    forbidden: &[String],
    crate_package: &str,
    dep_names: &[String],
    (fact_kind, collect): (
        ExposureKind,
        impl Fn(&syn::Item, &str, &UseMap, usize, &mut Vec<ShapeExposure>),
    ),
) -> Result<Vec<SemanticFact>, String> {
    let items = resolve_module_items(src_dir, root_file, module, crate_package)?;
    let uses = collect_uses(&items);
    let resolution = extern_resolution(src_dir, root_file, crate_package, dep_names, &items)?;
    let forbidden: Vec<String> = forbidden.iter().map(|f| canonical_path_str(f)).collect();

    let mut exposures = Vec::new();
    for (ordinal, item) in items.iter().enumerate() {
        collect(item, module, &uses, ordinal, &mut exposures);
    }

    let mut findings: Vec<SemanticFact> = exposures
        .into_iter()
        .filter(|exposure| {
            forbidden.is_empty()
                || exposure.principals.iter().any(|path| {
                    resolve_principal(path, &uses, module, &resolution)
                        .is_some_and(|canonical| matches_forbidden(&canonical, &forbidden))
                })
        })
        .map(|exposure| shape_finding(exposure, fact_kind))
        .collect();
    sort_facts(&mut findings);
    Ok(findings)
}
