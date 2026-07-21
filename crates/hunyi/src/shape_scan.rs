//! Shared scan hearts for the existential-exposure boundaries (dyn-trait, impl-trait,
//! async-exposure). Each boundary's per-module finding computation is the same skeleton — resolve
//! the module's items, walk each with a per-item collector, then render, sort, and dedup — differing
//! only in the collector (and, for the operand-scoped path, an added principal-resolution filter).
//! These two helpers hold that skeleton once so the three boundary modules pass only their
//! collector, rather than each re-implementing the identical pipeline.

use std::collections::HashMap;
use std::path::{Path, PathBuf};

use crate::containment::matches_forbidden;
use crate::crate_scope::{extern_resolution, file_extern_scope, resolve_principal};
use crate::finding::{ExposureKind, SemanticFact, shape_finding, sort_faceted_facts};
use crate::module_resolve::resolve_module_items_with_files;
use crate::resolve::{ShapeExposure, UseMap, canonical_path_str, collect_uses};

/// A `use`-map per BRANCH, not one shared map over the flattened cross-branch union: two
/// mutually-exclusive `#[cfg]` branches are never compiled together, so merging their `use`
/// declarations into one map lets the branch unioned last silently overwrite an earlier branch's
/// alias for the same local name (a realistic per-platform shim), misresolving a bare reference in
/// the FIRST branch through the SECOND branch's `use` — a confirmed false negative, found on a
/// round-6 adversarial review; see `PROJECT.md`'s Decisions. Keyed on the branch index, not the
/// file alone: two mutually-exclusive **inline** `#[cfg]` siblings share one identical enclosing
/// file, so a file-keyed map would re-merge them — the identical conflation one hop past item
/// observation, found on a round-8 adversarial review; see `PROJECT.md`'s Decisions.
fn uses_by_branch(items_with_files: &[(syn::Item, PathBuf, usize)]) -> HashMap<usize, UseMap> {
    let mut items_by_branch: HashMap<usize, Vec<syn::Item>> = HashMap::new();
    for (item, _file, branch) in items_with_files {
        items_by_branch
            .entry(*branch)
            .or_default()
            .push(item.clone());
    }
    items_by_branch
        .iter()
        .map(|(branch, branch_items)| (*branch, collect_uses(branch_items)))
        .collect()
}

/// Resolve `module`'s items, collect each item's exposures with `collect`, render each to a finding
/// with `render`, then sort + dedup. The shape-only heart shared by the dyn / impl-trait / async
/// boundaries: `collect` is the only per-boundary difference. The dyn and impl-trait boundaries
/// collect [`ShapeExposure`] and pass [`shape_finding`] as `render`; async collects owner-qualified
/// `String` identities directly and passes the identity `render`. `uses` is collected because the
/// collectors canonicalize an inherent impl's self-type owner in the seam (a finding-identity
/// concern), even where the boundary itself needs no name resolution. Each finding pairs with the
/// real file its own item's branch was resolved from — never a single first-branch file for the
/// whole module, which would misattribute a finding produced by a non-first `#[cfg]`-split branch.
pub(crate) fn shape_module_findings<E>(
    src_dir: &Path,
    root_file: &Path,
    module: &str,
    crate_package: &str,
    collect: impl Fn(&syn::Item, &str, &UseMap, usize, &mut Vec<E>),
    render: impl Fn(E) -> SemanticFact,
) -> Result<Vec<(SemanticFact, PathBuf)>, String> {
    let items_with_files =
        resolve_module_items_with_files(src_dir, root_file, module, crate_package)?;
    let uses_by_branch = uses_by_branch(&items_with_files);
    let mut collected: Vec<(E, PathBuf)> = Vec::new();
    for (ordinal, (item, file, branch)) in items_with_files.iter().enumerate() {
        let uses = &uses_by_branch[branch];
        let mut buf = Vec::new();
        collect(item, module, uses, ordinal, &mut buf);
        collected.extend(buf.into_iter().map(|exposure| (exposure, file.clone())));
    }
    let mut findings: Vec<(SemanticFact, PathBuf)> = collected
        .into_iter()
        .map(|(exposure, file)| (render(exposure), file))
        .collect();
    sort_faceted_facts(&mut findings);
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
/// Each finding pairs with the real file its own item's branch was resolved from, like
/// [`shape_module_findings`].
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
) -> Result<Vec<(SemanticFact, PathBuf)>, String> {
    let items_with_files =
        resolve_module_items_with_files(src_dir, root_file, module, crate_package)?;
    let uses_by_branch = uses_by_branch(&items_with_files);
    // Per-branch, not crate-wide and not per-file: `externs_type`/`renames_bare` derive from a
    // specific branch's own child-module names, so a #[cfg]-split module's several branches must
    // never share one (see `file_extern_scope`'s doc — the identical conflation class round 6 fixed
    // for the use-map). Keyed on the branch index rather than file alone: two mutually-exclusive
    // inline `#[cfg]` siblings share one identical enclosing file, so a file-keyed map would
    // re-merge them (round-8 adversarial review; see `PROJECT.md`'s Decisions).
    let mut items_by_branch: HashMap<usize, Vec<syn::Item>> = HashMap::new();
    for (item, _file, branch) in &items_with_files {
        items_by_branch
            .entry(*branch)
            .or_default()
            .push(item.clone());
    }
    let resolution = extern_resolution(src_dir, root_file, crate_package, dep_names)?;
    let file_scopes: HashMap<usize, crate::crate_scope::FileExternScope> = items_by_branch
        .iter()
        .map(|(branch, branch_items)| (*branch, file_extern_scope(&resolution, branch_items)))
        .collect();
    let forbidden: Vec<String> = forbidden.iter().map(|f| canonical_path_str(f)).collect();

    let mut exposures: Vec<(ShapeExposure, PathBuf, usize)> = Vec::new();
    for (ordinal, (item, file, branch)) in items_with_files.iter().enumerate() {
        let uses = &uses_by_branch[branch];
        let mut buf = Vec::new();
        collect(item, module, uses, ordinal, &mut buf);
        exposures.extend(
            buf.into_iter()
                .map(|exposure| (exposure, file.clone(), *branch)),
        );
    }

    let mut findings: Vec<(SemanticFact, PathBuf)> = exposures
        .into_iter()
        .filter(|(exposure, _file, branch)| {
            let uses = &uses_by_branch[branch];
            let file_scope = &file_scopes[branch];
            forbidden.is_empty()
                || exposure.principals.iter().any(|path| {
                    resolve_principal(path, uses, module, &resolution, file_scope)
                        .is_some_and(|canonical| matches_forbidden(&canonical, &forbidden))
                })
        })
        .map(|(exposure, file, _branch)| (shape_finding(exposure, fact_kind), file))
        .collect();
    sort_faceted_facts(&mut findings);
    Ok(findings)
}
