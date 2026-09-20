//! Signature-coupling (`semantic-signature-coupling`): a module's public API must not **expose** a
//! forbidden type. The heaviest capability — [`module_findings`] resolves each exposed type path
//! against the in-scope `use`s, the crate-wide re-export/alias closure, and the extern-crate
//! oracle before matching the forbidden set.

use std::borrow::Cow;
use std::collections::{HashMap, HashSet};
use std::path::{Path, PathBuf};

use serde_json::Value;
use xuanji::{Outcome, Violation};

use crate::collect::{collect_item_exposures, collect_trait_impl_exposures};
use crate::containment::matches_forbidden;
use crate::crate_scope::{
    child_module_names, dependency_names, external_crate_set, local_type_namespace_names,
};
use crate::driver::run_boundaries;
use crate::dsl::SignatureBoundary;
use crate::emit::{SingleModuleViolationContext, push_single_module_violations};
use crate::errors::unknown_module_error;
use crate::file_scope::{over_each_unit, resolve_crate_units};
use crate::finding::{ExposureKind, PathExposure, SemanticFact, sort_faceted_facts};
use crate::module_resolve::resolve_module_items_with_cfg_tags;
use crate::resolve::{
    AliasMap, BareFallback, ExternRenameMap, ReexportMap, UseMap, apply_bare_alias_rename,
    apply_crate_root_rename, bare_local_alias, canonical_path_str, collect_uses,
    expand_canonical_paths, extern_verbatim_renamed, renames_shadowed, resolve_path_all,
    validate_path_operands,
};
use crate::rules::SIGNATURE_RULE;
use crate::scan::scan_crate;
use crate::syn_util::{FlatItem, child_module_decls, reexport_externs_for, reexport_renames_for};

/// Run the semantic boundaries against the Cargo workspace at `manifest_path`.
///
/// The spine mirrors the static dimension — resolve → observe → compare → react: resolve
/// each boundary's crate and module anchor, observe the module's public-API surface from
/// the AST, compare each exposed type against the forbidden set, and return the outcome. An
/// unresolvable crate or module (or an unreadable/unparseable source) is a constitution
/// error (exit 2), never a silent pass.
pub fn check(boundaries: &[SignatureBoundary], manifest_path: &Path) -> Outcome {
    run_boundaries(boundaries, manifest_path, check_boundary)
}

/// Check a single signature boundary against the resolved crate compilation units.
///
/// Evaluates each unit separately so unit-varying modules and exposed types (e.g. in binaries)
/// are observed with unit identity.
pub(crate) fn check_boundary(
    metadata: &Value,
    boundary: &SignatureBoundary,
    violations: &mut Vec<Violation>,
) -> Result<(), String> {
    let (package, units) = resolve_crate_units(metadata, &boundary.crate_package)?;
    over_each_unit(
        &units,
        &unknown_module_error(&boundary.module, &boundary.crate_package),
        |root_file, src_dir, unit| {
            let findings = module_findings(
                src_dir,
                root_file,
                &boundary.module,
                &boundary.forbidden,
                &boundary.crate_package,
                boundary.including_trait_impls,
                &dependency_names(package),
            )?;

            push_single_module_violations(
                violations,
                SingleModuleViolationContext {
                    module: &boundary.module,
                    rule: SIGNATURE_RULE,
                    rule_key: boundary.rule_key(),
                    reason: &boundary.reason,
                    severity: boundary.severity,
                    anchor: boundary.anchor(),
                    crate_package: &boundary.crate_package,
                    unit,
                },
                findings,
            );
            Ok(())
        },
    )
}

/// Per-branch (mutually-exclusive `#[cfg]`-group) resolution context: `uses` (a bare local `use …
/// as <dep>` alias), `externs_type` (a bare **type-position** head may be a child module of THIS
/// branch's own module — a local `mod serde` denotes `crate::…::serde`, not the dependency
/// `serde` — so type positions use the set with THIS FILE's own child modules excluded; a bare
/// **re-export** head is extern by edition-2018+ grammar even with a same-named local module, so
/// re-exports use the raw set), `mod_decls` (this file's own child-**module** declarations,
/// item-level — see `reexport_externs_for`'s cfg-aware use in [`resolve_exposure_to_findings`],
/// the re-export-head analogue of `externs_type`'s whole-branch subtraction), and `renames_bare`
/// (a crate-root `extern crate X as Y;` binds `Y` crate-wide, but a governed submodule that
/// declares its OWN child `mod Y` shadows the alias there — so a bare head uses the rename map
/// with THIS FILE's own child-module names removed). `renames_bare` stays the whole-branch,
/// cfg-blind computation and backs only a **type-position** head, exactly mirroring
/// `externs_type`'s own scope: a re-export exposure's bare head instead computes its own cfg-aware
/// rename map per item, via `reexport_renames_for` (the rename-alias analogue of
/// `reexport_externs_for`) — an unfixed cfg-blind rename shadow there would not merely
/// under-shadow the re-export, it would drop the resolution outright, since the shadowed alias
/// spelling is never itself a member of the externs-set fallback (found by an independent
/// adversarial review of the `mod_decls` fix). The crate-relative (`crate::Y::…`) and
/// leading-`::` forms are NOT shadowable, so they keep the full `extern_renames` regardless.
struct FileScope {
    uses: UseMap,
    externs_type: HashSet<String>,
    mod_decls: Vec<(String, FlatItem)>,
    renames_bare: ExternRenameMap,
}

/// Build each branch's own [`FileScope`]. Grouped by BRANCH INDEX, not by file and not one shared
/// computation over the flattened cross-branch union: two mutually-exclusive `#[cfg]` branches are
/// never compiled together, so deriving a shadow set (a `use`-map, a child-module-name set, a
/// rename map) from their UNION lets one branch's own declarations silently apply to the OTHER,
/// mutually-exclusive branch's resolution — a confirmed false negative. **Every** derivation is
/// per-branch, not only the `use`-map: `externs_type`/`externs_reexport`/`renames_bare` are all
/// derived from each file's own child-module names and carry the identical conflation — e.g. a
/// branch with no
/// local `mod net` had its genuine `pub use net::Something;` (the real extern crate) silently
/// suppressed merely because a MUTUALLY-EXCLUSIVE sibling branch happened to declare its own local
/// `mod net`. Grouping by FILE ALONE is itself insufficient: two mutually-exclusive **inline**
/// `#[cfg]` siblings share one identical enclosing file, so a file-keyed group re-merges them —
/// the identical conflation one hop past item observation. The branch index
/// `resolve_module_items_with_cfg_tags` pairs each
/// item with is the finer key that keeps them apart.
fn build_file_scopes(
    items_by_branch: &HashMap<usize, Vec<FlatItem>>,
    externs: &HashSet<String>,
    extern_renames: &ExternRenameMap,
) -> HashMap<usize, FileScope> {
    items_by_branch
        .iter()
        .map(|(branch, flat_items)| {
            let items: Vec<syn::Item> = flat_items.iter().map(|f| f.item.clone()).collect();
            let child_mods = child_module_names(&items);
            let externs_type = externs
                .difference(&local_type_namespace_names(&items))
                .cloned()
                .collect();
            let renames_bare = renames_shadowed(extern_renames, &child_mods);
            (
                *branch,
                FileScope {
                    uses: collect_uses(&items),
                    externs_type,
                    mod_decls: child_module_decls(flat_items),
                    renames_bare,
                },
            )
        })
        .collect()
}

/// Observe each item's exposed type paths (and, when opted in, trait-impl-site positions). Each
/// exposure keeps its OWN generating item's [`FlatItem`] tag alongside it (not just the
/// file/branch), so a re-export exposure's later child-module-shadow check
/// (`reexport_externs_for`) can compare its own cfg-gating against a sibling `mod` declaration's,
/// rather than only the branch it happens to share with that sibling. Branch grouping alone is
/// still not fine enough for this: two mutually-exclusive `#[cfg]`/`cfg_if!` SIBLING ITEMS (a
/// `#[cfg(unix)] mod x;` beside a `#[cfg(not(unix))] pub use x::Y;`, or the two arms of one
/// `cfg_if!`) share the identical branch — there is no module-path split to lean on, since the
/// governed module itself resolves to exactly one branch here.
fn collect_all_exposures(
    items_with_files: &[(FlatItem, PathBuf, usize)],
    scopes: &HashMap<usize, FileScope>,
    module: &str,
    include_trait_impls: bool,
) -> Vec<(PathExposure, PathBuf, usize, FlatItem)> {
    let mut exposed = Vec::new();
    for (ordinal, (flat, file, branch)) in items_with_files.iter().enumerate() {
        let uses = &scopes[branch].uses;
        let mut buf = Vec::new();
        collect_item_exposures(&flat.item, module, uses, ordinal, &mut buf);
        if include_trait_impls {
            collect_trait_impl_exposures(&flat.item, module, uses, ordinal, &mut buf);
        }
        exposed.extend(
            buf.into_iter()
                .map(|exposure| (exposure, file.clone(), *branch, flat.clone())),
        );
    }
    exposed
}

/// Resolve one exposure's path against the in-scope `use`s, the crate-wide re-export/alias
/// closure, and the extern-crate oracle, then match the resolved canonical paths against the
/// forbidden set.
///
/// Leading `::` paths bypass the use-map and resolve directly against the raw extern set.
/// Bare heads check use-map candidates, then local type aliases, and finally external crates
/// accounting for child-module shadowing and extern-crate renames.
#[allow(clippy::too_many_arguments)]
fn resolve_exposure_to_findings(
    exposure: &PathExposure,
    file: &Path,
    branch: usize,
    origin: &FlatItem,
    scopes: &HashMap<usize, FileScope>,
    module: &str,
    aliases: &AliasMap,
    reexports: &ReexportMap,
    extern_renames: &ExternRenameMap,
    externs: &HashSet<String>,
    forbidden: &[String],
) -> Vec<(SemanticFact, PathBuf)> {
    let scope = &scopes[&branch];
    let uses = &scope.uses;
    let type_externs: Cow<HashSet<String>> = if exposure.is_reexport {
        Cow::Owned(reexport_externs_for(externs, &scope.mod_decls, origin))
    } else {
        Cow::Borrowed(&scope.externs_type)
    };
    let renames_for_item: Cow<HashMap<String, String>> = if exposure.is_reexport {
        Cow::Owned(reexport_renames_for(
            extern_renames,
            &scope.mod_decls,
            origin,
        ))
    } else {
        Cow::Borrowed(&scope.renames_bare)
    };
    let resolved: Vec<String> = if exposure.path.leading_colon.is_some() {
        extern_verbatim_renamed(&exposure.path, externs, extern_renames)
            .into_iter()
            .collect()
    } else {
        let use_map_candidates =
            resolve_path_all(&exposure.path, uses, module, BareFallback::Ignore);
        if !use_map_candidates.is_empty() {
            use_map_candidates
        } else {
            bare_local_alias(&exposure.path, module, aliases)
                .or_else(|| {
                    extern_verbatim_renamed(&exposure.path, &type_externs, &renames_for_item)
                })
                .into_iter()
                .collect()
        }
    };
    let canonicals: Vec<String> = resolved
        .iter()
        .flat_map(|canonical| expand_canonical_paths(canonical, aliases, reexports))
        .collect();
    canonicals
        .into_iter()
        .map(|canonical| apply_crate_root_rename(canonical, extern_renames))
        .map(|canonical| apply_bare_alias_rename(canonical, &renames_for_item))
        .filter(|canonical| matches_forbidden(canonical, forbidden))
        .map(|canonical| {
            (
                SemanticFact::Exposed {
                    kind: ExposureKind::Signature,
                    subject: canonical,
                    seam: exposure.seam.clone(),
                },
                file.to_path_buf(),
            )
        })
        .collect()
}

/// The pure heart, testable without spawning `cargo`: resolve the module's items, observe
/// the exposed type paths, resolve each against the in-scope `use`s, and return the sorted,
/// deduplicated canonical paths that fall within the forbidden set. Each finding pairs with the
/// real file its own item's branch was resolved from — never a single first-branch file for the
/// whole module, which would misattribute a finding produced by a non-first `#[cfg]`-split branch.
///
/// Forbidden operands are validated to ensure they contain valid `::`-delimited path segments.
pub(crate) fn module_findings(
    src_dir: &Path,
    root_file: &Path,
    module: &str,
    forbidden: &[String],
    crate_package: &str,
    include_trait_impls: bool,
    dep_names: &[String],
) -> Result<Vec<(SemanticFact, PathBuf)>, String> {
    validate_path_operands(forbidden)?;
    let items_with_files =
        resolve_module_items_with_cfg_tags(src_dir, root_file, module, crate_package)?;
    let mut items_by_branch: HashMap<usize, Vec<FlatItem>> = HashMap::new();
    for (flat, _file, branch) in &items_with_files {
        items_by_branch
            .entry(*branch)
            .or_default()
            .push(flat.clone());
    }
    let externs = external_crate_set(dep_names);
    let scan = scan_crate(src_dir, root_file, crate_package, &externs)?;
    let reexports = scan.reexports;
    let aliases = scan.aliases;
    let extern_renames = scan.extern_renames;
    let scopes = build_file_scopes(&items_by_branch, &externs, &extern_renames);
    let forbidden: Vec<String> = forbidden.iter().map(|f| canonical_path_str(f)).collect();

    let exposed = collect_all_exposures(&items_with_files, &scopes, module, include_trait_impls);

    let mut findings: Vec<(SemanticFact, PathBuf)> = exposed
        .iter()
        .flat_map(|(exposure, file, branch, origin)| {
            resolve_exposure_to_findings(
                exposure,
                file,
                *branch,
                origin,
                &scopes,
                module,
                &aliases,
                &reexports,
                &extern_renames,
                &externs,
                &forbidden,
            )
        })
        .collect();
    sort_faceted_facts(&mut findings)?;
    Ok(findings)
}
