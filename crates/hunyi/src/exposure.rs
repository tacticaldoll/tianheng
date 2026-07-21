//! Signature-coupling (`semantic-signature-coupling`): a module's public API must not **expose** a
//! forbidden type. The heaviest capability — [`module_findings`] resolves each exposed type path
//! against the in-scope `use`s, the crate-wide re-export/alias closure, and the extern-crate
//! oracle before matching the forbidden set.

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
use crate::module_resolve::resolve_module_items_with_files;
use crate::resolve::{
    BareFallback, apply_bare_alias_rename, apply_crate_root_rename, bare_local_alias,
    canonical_path_str, canonicalize_through_aliases, collect_uses, extern_verbatim_renamed,
    renames_shadowed, resolve_path,
};
use crate::rules::SIGNATURE_RULE;
use crate::scan::scan_crate;

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
            reason: &boundary.reason,
            severity: boundary.severity,
            anchor: boundary.anchor(),
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
        resolve_module_items_with_files(src_dir, root_file, module, crate_package)?;
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
    // review; see `PROJECT.md`'s Decisions. The branch index `resolve_module_items_with_files`
    // pairs each item with is the finer key that keeps them apart.
    let mut items_by_branch: HashMap<usize, Vec<syn::Item>> = HashMap::new();
    for (item, _file, branch) in &items_with_files {
        items_by_branch
            .entry(*branch)
            .or_default()
            .push(item.clone());
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
    // `externs_reexport` (the re-export head oracle subtracts THIS FILE's own child-module names —
    // only modules, since a `pub use` head must be a module/crate), and `renames_bare` (a
    // crate-root `extern crate X as Y;` binds `Y` crate-wide, but a governed submodule that
    // declares its OWN child `mod Y` shadows the alias there — so a bare head uses the rename map
    // with THIS FILE's own child-module names removed). The crate-relative (`crate::Y::…`) and
    // leading-`::` forms are NOT shadowable, so they keep the full `extern_renames` regardless.
    struct FileScope {
        uses: crate::resolve::UseMap,
        externs_type: HashSet<String>,
        externs_reexport: HashSet<String>,
        renames_bare: crate::resolve::ExternRenameMap,
    }
    let scopes: HashMap<usize, FileScope> = items_by_branch
        .iter()
        .map(|(branch, file_items)| {
            let child_mods = child_module_names(file_items);
            let externs_type = externs
                .difference(&local_type_namespace_names(file_items))
                .cloned()
                .collect();
            let externs_reexport = externs.difference(&child_mods).cloned().collect();
            let renames_bare = renames_shadowed(&extern_renames, &child_mods);
            (
                *branch,
                FileScope {
                    uses: collect_uses(file_items),
                    externs_type,
                    externs_reexport,
                    renames_bare,
                },
            )
        })
        .collect();
    let forbidden: Vec<String> = forbidden.iter().map(|f| canonical_path_str(f)).collect();

    let mut exposed = Vec::new();
    for (ordinal, (item, file, branch)) in items_with_files.iter().enumerate() {
        let uses = &scopes[branch].uses;
        let mut buf = Vec::new();
        collect_item_exposures(item, module, uses, ordinal, &mut buf);
        // Opt-in depth: also observe the module's trait `impl` blocks' impl-site-authored
        // positions (`semantic-trait-impl-exposure`). The same resolve → canonicalize → match →
        // `{type} exposed by {seam}` pipeline below applies unchanged; only the seam differs.
        if include_trait_impls {
            collect_trait_impl_exposures(item, module, uses, ordinal, &mut buf);
        }
        exposed.extend(
            buf.into_iter()
                .map(|exposure| (exposure, file.clone(), *branch)),
        );
    }

    let mut findings: Vec<(SemanticFact, PathBuf)> = exposed
        .iter()
        .filter_map(|(exposure, file, branch)| {
            let scope = &scopes[branch];
            let uses = &scope.uses;
            // `resolve_path` returns None for a bare head (not `crate`-relative, not in the
            // `use`-map); the extern oracle then fires for an external-crate head, resolving
            // the inline extern path to itself. Ordering guarantees a local `use … as <dep>`
            // alias (found in the `use`-map) still wins over a dependency of the same name. A
            // re-export head uses the child-module-excluded set (a same-named child `mod` shadows a
            // bare `pub use` head); a type-position head uses the full type-namespace-excluded set.
            let type_externs = if exposure.is_reexport {
                &scope.externs_reexport
            } else {
                &scope.externs_type
            };
            // A leading `::` is an unambiguous extern (edition 2018+): resolve against the RAW
            // extern set (with the crate-root `extern crate … as` rename applied, so a
            // `::<rename>::Type` head still resolves to its real crate), bypassing the `use`-map
            // and the local type-namespace shadow, as a HARD short-circuit — a non-dependency
            // `::head` stays unresolved (a bound), never mis-attributed through the `use`-map.
            // `extern_verbatim_renamed` ignores `leading_colon` (it iterates the segments), so
            // the raw set makes it react to `::serde` under a local `mod serde` (the shadow case)
            // while the rename hop is preserved; a local `mod` is never a rename, so no FP.
            // Otherwise: a bare single-segment local `type` alias resolves before the extern
            // oracle (a local alias shadows a same-named dependency), and the combined closure
            // follows alias→alias / alias→re-export hops.
            let resolved = if exposure.path.leading_colon.is_some() {
                extern_verbatim_renamed(&exposure.path, &externs, &extern_renames)
            } else {
                resolve_path(&exposure.path, uses, module, BareFallback::Ignore)
                    .or_else(|| bare_local_alias(&exposure.path, module, &aliases))
                    // The bare-head extern-rename rewrite uses `renames_bare`: a `Y::…` head shadowed
                    // by this module's own child `mod Y` is not rewritten to the crate (rustc resolves
                    // it to the local module), while an unshadowed `Y::…` still rewrites (no FN).
                    .or_else(|| {
                        extern_verbatim_renamed(&exposure.path, type_externs, &scope.renames_bare)
                    })
            };
            resolved
                .map(|canonical| canonicalize_through_aliases(&canonical, &aliases, &reexports))
                // Crate-relative spelling of a crate-root rename: `crate::Y::rest` → `X::rest`.
                // `crate::Y` unambiguously names the crate-root extern rename (a crate-root `mod Y`
                // cannot coexist with `extern crate … as Y`), so this is unconditional and uses the
                // full rename map; only the segment immediately after `crate` is treated as the alias.
                // Applied AFTER the alias/re-export closure so a `crate::Y::…` reached directly OR
                // through a `type` alias / `pub use` target (whose stored target keeps the verbatim
                // `crate::Y::…`) is rewritten alike — otherwise the aliased form is a residual FN.
                .map(|canonical| apply_crate_root_rename(canonical, &extern_renames))
                // Bare spelling of the same rename: a forbidden type imported by a private
                // `use Y::…;` resolves through the use-map to `Y::…` verbatim (unlike the direct
                // type-position form, which the extern oracle already rewrote), so rewrite a bare
                // alias head here too. Uses `renames_bare` — a head shadowed by a local `mod Y` is
                // left alone (rustc resolves bare `Y` to the local module there).
                .map(|canonical| apply_bare_alias_rename(canonical, &scope.renames_bare))
                .filter(|canonical| matches_forbidden(canonical, &forbidden))
                // Seam-qualify: two distinct seams exposing the same forbidden type stay distinct
                // findings, so baselining one never masks a new leak at another (the one forbidden
                // bug) — the shape/existential rules do the same below.
                .map(|canonical| SemanticFact::Exposed {
                    kind: ExposureKind::Signature,
                    subject: canonical,
                    seam: exposure.seam.clone(),
                })
                .map(|fact| (fact, file.clone()))
        })
        .collect();
    sort_faceted_facts(&mut findings);
    Ok(findings)
}
