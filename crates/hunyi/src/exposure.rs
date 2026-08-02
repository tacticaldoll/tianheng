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
use crate::dsl::SemanticBoundary;
use crate::emit::{SingleModuleViolationContext, push_single_module_violations};
use crate::file_scope::resolve_crate;
use crate::finding::{ExposureKind, SemanticFact, sort_faceted_facts};
use crate::module_resolve::resolve_module_items_with_cfg_tags;
use crate::resolve::{
    BareFallback, apply_bare_alias_rename, apply_crate_root_rename, bare_local_alias,
    canonical_path_str, collect_uses, expand_canonical_paths, extern_verbatim_renamed,
    renames_shadowed, resolve_path_all,
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
pub fn check(boundaries: &[SemanticBoundary], manifest_path: &Path) -> Outcome {
    run_boundaries(boundaries, manifest_path, check_boundary)
}

pub(crate) fn check_boundary(
    metadata: &Value,
    boundary: &SemanticBoundary,
    violations: &mut Vec<Violation>,
) -> Result<(), String> {
    let (package, root_file, src_dir) = resolve_crate(metadata, &boundary.crate_package)?;
    let src_dir = src_dir.as_path();

    let findings = module_findings(
        src_dir,
        &root_file,
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
        },
        findings,
    );
    Ok(())
}

/// The pure heart, testable without spawning `cargo`: resolve the module's items, observe
/// the exposed type paths, resolve each against the in-scope `use`s, and return the sorted,
/// deduplicated canonical paths that fall within the forbidden set. Each finding pairs with the
/// real file its own item's branch was resolved from — never a single first-branch file for the
/// whole module, which would misattribute a finding produced by a non-first `#[cfg]`-split branch.
pub(crate) fn module_findings(
    src_dir: &Path,
    root_file: &Path,
    module: &str,
    forbidden: &[String],
    crate_package: &str,
    include_trait_impls: bool,
    dep_names: &[String],
) -> Result<Vec<(SemanticFact, PathBuf)>, String> {
    let items_with_files =
        resolve_module_items_with_cfg_tags(src_dir, root_file, module, crate_package)?;
    // Grouped by BRANCH INDEX, not by file and not one shared computation over the flattened
    // cross-branch union: two mutually-exclusive `#[cfg]` branches are never compiled together, so
    // deriving a shadow set (a `use`-map, a child-module-name set, a rename map) from their UNION
    // lets one branch's own declarations silently apply to the OTHER, mutually-exclusive branch's
    // resolution — a confirmed false negative, found on round-6/7 adversarial reviews; see
    // `PROJECT.md`'s Decisions. `uses_by_file` was fixed in round 6; `externs_type`/
    // `externs_reexport`/`renames_bare` (all derived from each file's own child-module names) had
    // the identical conflation left unfixed, found in round 7 — e.g. a branch with no local `mod
    // net` had its genuine `pub use net::Something;` (the real extern crate) silently suppressed
    // merely because a MUTUALLY-EXCLUSIVE sibling branch happened to declare its own local `mod
    // net`. Grouping by FILE ALONE is itself insufficient: two mutually-exclusive **inline**
    // `#[cfg]` siblings share one identical enclosing file, so a file-keyed group re-merges them —
    // the identical conflation one hop past item observation, found on a round-8 adversarial
    // review; see `PROJECT.md`'s Decisions. The branch index `resolve_module_items_with_cfg_tags`
    // pairs each item with is the finer key that keeps them apart.
    //
    // Branch grouping alone is still not fine enough for `externs_reexport`: two mutually-exclusive
    // `#[cfg]`/`cfg_if!` SIBLING ITEMS (a `#[cfg(unix)] mod x;` beside a `#[cfg(not(unix))] pub use
    // x::Y;`, or the two arms of one `cfg_if!`) share the identical branch — there is no
    // module-path split to lean on, since the governed module itself resolves to exactly one
    // branch here. `externs_reexport` therefore cannot stay a single per-branch set (round-9
    // finding, the one item PR #149 explicitly left out of scope: `exposure.rs:157`'s
    // child-module shadow was still cfg-blind at the item level even after branch grouping closed
    // the file/branch level). Each branch's own `mod` declarations are kept below as `mod_decls` —
    // paired with their own [`FlatItem`] tag — so a specific `pub use` exposure's child-module
    // shadow is computed against ITS OWN cfg-compatible siblings only (via `reexport_externs_for`),
    // never the branch's whole, cfg-blind child-module-name set.
    let mut items_by_branch: HashMap<usize, Vec<FlatItem>> = HashMap::new();
    for (flat, _file, branch) in &items_with_files {
        items_by_branch
            .entry(*branch)
            .or_default()
            .push(flat.clone());
    }
    // The external-crate name set: declared dependencies (`-`→`_` normalized, rename-aware) ∪
    // the sysroot crates. A bare head in it denotes an external crate, so an inline extern path
    // resolves to itself verbatim and reacts — closing the extern-path false negative. Applied
    // only in the bare-fallback branch (after `use`-map / `crate`-relative), and only here + the
    // re-export closure. Crate-wide, not per-file: dependencies are declared once for the crate.
    let externs = external_crate_set(dep_names);
    // The re-export and alias closures are crate-wide: a forbidden type exposed through a
    // `pub use` facade or a `type X = <path>;` alias must canonicalize to its defining path
    // before matching. The re-export closure retains an extern-headed target (raw set — a bare
    // `pub use` head is extern by grammar), so a local facade chain terminating at an extern
    // type canonicalizes to it; the alias closure follows resolvable-nominal-path aliases.
    let scan = scan_crate(src_dir, root_file, crate_package, &externs)?;
    let reexports = scan.reexports;
    let aliases = scan.aliases;
    // Source-level crate-root `extern crate X as Y;` renames: a renamed head resolves to the real
    // crate before the extern check (the whole walk completes before we resolve, so the map is
    // fully populated — no ordering hazard). Crate-wide: such a rename binds via the extern
    // prelude for the whole crate, not per-branch.
    let extern_renames = scan.extern_renames;
    // Per-file: `uses` (a bare local `use … as <dep>` alias), `externs_type` (a bare
    // **type-position** head may be a child module of THIS branch's own module — a local
    // `mod serde` denotes `crate::…::serde`, not the dependency `serde` — so type positions use
    // the set with THIS FILE's own child modules excluded; a bare **re-export** head is extern by
    // edition-2018+ grammar even with a same-named local module, so re-exports use the raw set),
    // `mod_decls` (this file's own child-**module** declarations, item-level — see
    // `reexport_externs_for`'s cfg-aware use below, the re-export-head analogue of `externs_type`'s
    // whole-branch subtraction), and `renames_bare` (a crate-root `extern crate X as Y;` binds `Y`
    // crate-wide, but a governed submodule that declares its OWN child `mod Y` shadows the alias
    // there — so a bare head uses the rename map with THIS FILE's own child-module names removed).
    // `renames_bare` here stays the whole-branch, cfg-blind computation and backs only a
    // **type-position** head, exactly mirroring `externs_type`'s own scope: a re-export exposure's
    // bare head instead computes its own cfg-aware rename map per item, via `reexport_renames_for`
    // below (the rename-alias analogue of `reexport_externs_for`) — an unfixed cfg-blind rename
    // shadow there would not merely under-shadow the re-export, it would drop the resolution
    // outright, since the shadowed alias spelling is never itself a member of the externs-set
    // fallback (found by an independent adversarial review of the `mod_decls` fix). The
    // crate-relative (`crate::Y::…`) and leading-`::` forms are NOT shadowable, so they keep the
    // full `extern_renames` regardless.
    struct FileScope {
        uses: crate::resolve::UseMap,
        externs_type: HashSet<String>,
        mod_decls: Vec<(String, FlatItem)>,
        renames_bare: crate::resolve::ExternRenameMap,
    }
    let scopes: HashMap<usize, FileScope> = items_by_branch
        .iter()
        .map(|(branch, flat_items)| {
            let items: Vec<syn::Item> = flat_items.iter().map(|f| f.item.clone()).collect();
            let child_mods = child_module_names(&items);
            let externs_type = externs
                .difference(&local_type_namespace_names(&items))
                .cloned()
                .collect();
            let renames_bare = renames_shadowed(&extern_renames, &child_mods);
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
        .collect();
    let forbidden: Vec<String> = forbidden.iter().map(|f| canonical_path_str(f)).collect();

    let mut exposed = Vec::new();
    for (ordinal, (flat, file, branch)) in items_with_files.iter().enumerate() {
        let uses = &scopes[branch].uses;
        let mut buf = Vec::new();
        collect_item_exposures(&flat.item, module, uses, ordinal, &mut buf);
        // Opt-in depth: also observe the module's trait `impl` blocks' impl-site-authored
        // positions (`semantic-trait-impl-exposure`). The same resolve → canonicalize → match →
        // `{type} exposed by {seam}` pipeline below applies unchanged; only the seam differs.
        if include_trait_impls {
            collect_trait_impl_exposures(&flat.item, module, uses, ordinal, &mut buf);
        }
        // Each exposure keeps its OWN generating item's `FlatItem` tag alongside it (not just the
        // file/branch), so a re-export exposure's later child-module-shadow check
        // (`reexport_externs_for`) can compare its own cfg-gating against a sibling `mod`
        // declaration's, rather than only the branch it happens to share with that sibling.
        exposed.extend(
            buf.into_iter()
                .map(|exposure| (exposure, file.clone(), *branch, flat.clone())),
        );
    }

    let mut findings: Vec<(SemanticFact, PathBuf)> = exposed
        .iter()
        .flat_map(|(exposure, file, branch, origin)| {
            let scope = &scopes[branch];
            let uses = &scope.uses;
            // `resolve_path_all` returns no candidates for a bare head (not `crate`-relative, not
            // in the `use`-map); the extern oracle then fires for an external-crate head, resolving
            // the inline extern path to itself. Ordering guarantees a local `use … as <dep>`
            // alias (found in the `use`-map) still wins over a dependency of the same name. A
            // re-export head uses the child-module-excluded set (a same-named child `mod` shadows a
            // bare `pub use` head) — computed against THIS exposure's own `origin` tag, so a
            // same-named `mod` that is provably mutually exclusive with this exposure's own
            // cfg-gating (a different `#[cfg]`/`cfg_if!` arm, never compiled alongside it) does not
            // suppress it; a type-position head uses the full type-namespace-excluded set, unchanged.
            let type_externs: Cow<HashSet<String>> = if exposure.is_reexport {
                Cow::Owned(reexport_externs_for(&externs, &scope.mod_decls, origin))
            } else {
                Cow::Borrowed(&scope.externs_type)
            };
            // The crate-root extern-crate-rename shadow gets the identical cfg-aware treatment for
            // a re-export exposure's own bare head (found by an independent adversarial review of
            // the fix above): `extern_verbatim_renamed` checks the rename map BEFORE the externs
            // set, and a rename alias (`wc` from `extern crate serde as wc;`) is never itself a
            // member of the externs set (only the real crate name `serde` is) — so leaving
            // `renames_bare` cfg-blind here does not merely under-shadow a re-export through a
            // rename alias, it drops the resolution outright once the shadowed alias falls through
            // to an externs-set fallback with no candidate for it at all. A type-position head
            // keeps the branch-wide, cfg-blind `renames_bare` unchanged (an explicit non-goal,
            // matching `externs_type`'s own scope).
            let renames_for_item: Cow<HashMap<String, String>> = if exposure.is_reexport {
                Cow::Owned(reexport_renames_for(
                    &extern_renames,
                    &scope.mod_decls,
                    origin,
                ))
            } else {
                Cow::Borrowed(&scope.renames_bare)
            };
            // A leading `::` is an unambiguous extern (edition 2018+): resolve against the RAW
            // extern set (with the crate-root `extern crate … as` rename applied, so a
            // `::<rename>::Type` head still resolves to its real crate), bypassing the `use`-map
            // and the local type-namespace shadow, as a HARD short-circuit — a non-dependency
            // `::head` stays unresolved (a bound), never mis-attributed through the `use`-map.
            // `extern_verbatim_renamed` ignores `leading_colon` (it iterates the segments), so
            // the raw set makes it react to `::serde` under a local `mod serde` (the shadow case)
            // while the rename hop is preserved; a local `mod` is never a rename, so no FP. This
            // path is a deterministic syntactic short-circuit (no `use`-map involved), so it stays
            // single-valued — cfg-blind multi-candidate resolution has nothing to disambiguate here.
            // Otherwise: `resolve_path_all` returns every candidate the head resolves to through
            // the `use`-map (cfg-blind: two mutually-exclusive `#[cfg]` branches' conflicting `use
            // ... as Name;` are never compiled together, so neither candidate may be silently
            // dropped in favor of the other purely because of source order). When the `use`-map has
            // no candidate at all, the single-valued fallbacks apply exactly as before: a bare
            // single-segment local `type` alias resolves before the extern oracle (a local alias
            // shadows a same-named dependency), and the combined closure follows alias→alias /
            // alias→re-export hops.
            let resolved: Vec<String> = if exposure.path.leading_colon.is_some() {
                extern_verbatim_renamed(&exposure.path, &externs, &extern_renames)
                    .into_iter()
                    .collect()
            } else {
                let use_map_candidates =
                    resolve_path_all(&exposure.path, uses, module, BareFallback::Ignore);
                if !use_map_candidates.is_empty() {
                    use_map_candidates
                } else {
                    bare_local_alias(&exposure.path, module, &aliases)
                        // The bare-head extern-rename rewrite uses `renames_for_item`: a `Y::…`
                        // head shadowed by this module's own child `mod Y` is not rewritten to the
                        // crate (rustc resolves it to the local module), while an unshadowed `Y::…`
                        // still rewrites (no FN) — cfg-aware for a re-export exposure, as above.
                        .or_else(|| {
                            extern_verbatim_renamed(
                                &exposure.path,
                                &type_externs,
                                &renames_for_item,
                            )
                        })
                        .into_iter()
                        .collect()
                }
            };
            let canonicals: Vec<String> = resolved
                .iter()
                .flat_map(|canonical| expand_canonical_paths(canonical, &aliases, &reexports))
                .collect();
            let file_ref = file.clone();
            let seam_ref = exposure.seam.clone();
            canonicals
                .into_iter()
                .map(|canonical| apply_crate_root_rename(canonical, &extern_renames))
                .map(move |canonical| apply_bare_alias_rename(canonical, &renames_for_item))
                .filter(|canonical| matches_forbidden(canonical, &forbidden))
                .map(move |canonical| {
                    (
                        SemanticFact::Exposed {
                            kind: ExposureKind::Signature,
                            subject: canonical,
                            seam: seam_ref.clone(),
                        },
                        file_ref.clone(),
                    )
                })
        })
        .collect();
    sort_faceted_facts(&mut findings)?;
    Ok(findings)
}
