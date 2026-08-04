//! Forbidden-marker (`semantic-forbidden-marker`): a subtree's types must not acquire a forbidden
//! trait. For each forbidden trait, emit findings two ways — a `#[derive]` on a subtree type, and
//! an `impl T for X` (anywhere) whose self-type resolves to a subtree definition.

use std::collections::HashSet;
use std::path::{Path, PathBuf};

use serde_json::Value;
use xuanji::{Outcome, Polarity, Violation};

use crate::containment::{leaf_of, path_leaf, resolve_self_type, under_subtree};
use crate::driver::run_boundaries;
use crate::dsl::ForbiddenMarkerBoundary;
use crate::emit::{MultiModuleViolationContext, push_multi_module_violations};
use crate::file_scope::resolve_crate_units;
use crate::finding::{SemanticFact, sort_attributed_facts};
use crate::resolve::{
    BareFallback, UseMap, canonical_path_str, canonical_self_owner, path_to_string,
    resolve_path_all, validate_path_operands,
};
use crate::rules::FORBIDDEN_MARKER_RULE;
use crate::scan::{CrateScan, scan_crate};

/// Resolve `path` through `uses`/`module`'s use-map (cfg-blind: every candidate checked) and
/// return each candidate's leaf identifier, falling back to the written path's own leaf when no
/// use-map candidate resolves. Shared by the derive form and the impl form below, which both need
/// the identical leaf-matching step: a locally renamed derive macro/trait (`use serde::Serialize
/// as Ser;`) reacts by its true leaf, while an unresolved bare/prelude/extern path still matches
/// by its written leaf, so leaf-matching stays cross-crate-blind either way.
fn resolved_leaves(path: &syn::Path, uses: &UseMap, module: &str) -> Vec<String> {
    let use_candidates = resolve_path_all(path, uses, module, BareFallback::Ignore);
    if use_candidates.is_empty() {
        vec![path_leaf(path)]
    } else {
        use_candidates
            .iter()
            .map(|p| leaf_of(p).to_string())
            .collect()
    }
}

/// Run the forbidden-marker boundaries against the Cargo workspace at `manifest_path`.
pub fn check_forbidden_marker(
    boundaries: &[ForbiddenMarkerBoundary],
    manifest_path: &Path,
) -> Outcome {
    run_boundaries(boundaries, manifest_path, check_forbidden_marker_boundary)
}

pub(crate) fn check_forbidden_marker_boundary(
    metadata: &Value,
    boundary: &ForbiddenMarkerBoundary,
    violations: &mut Vec<Violation>,
) -> Result<(), String> {
    let (_package, units) = resolve_crate_units(metadata, &boundary.crate_package)?;
    // Each of a package's crate roots is its own compilation unit: same module path `crate`,
    // separate module graph. Evaluated once per unit so an exposure in a `bin` beside a library
    // is observed, with the unit carried into each finding's identity.
    for (root_file, src_dir, unit) in &units {
        let src_dir = src_dir.as_path();
        let unit = unit.as_str();

        let findings = forbidden_marker_findings(
            src_dir,
            &root_file,
            &boundary.module,
            &boundary.forbidden,
            &boundary.crate_package,
        )?;

        // Each finding carries the module its offending element sits in — the impl site's module for
        // an `impl`, the defining type's module for a `#[derive]`; the shared emit helper resolves that
        // module's source file (memoized per module) and stamps the deny-breach polarity.
        push_multi_module_violations(
            violations,
            MultiModuleViolationContext {
                target: &boundary.module,
                rule: FORBIDDEN_MARKER_RULE,
                rule_key: boundary.rule_key(),
                reason: &boundary.reason,
                severity: boundary.severity,
                anchor: boundary.anchor(),
                polarity: Polarity::DenyBreach,
                crate_package: &boundary.crate_package,
                unit,
            },
            findings,
        );
    }
    Ok(())
}

/// The pure heart: scan the crate, then for each forbidden trait emit findings two ways — a
/// `#[derive]` on a subtree type, and an `impl T for X` (anywhere) whose self-type resolves to
/// a subtree definition. Matching is leaf-identifier (so the derive-macro re-export path and
/// the trait path both match; never a silent miss). Sorted, deduplicated.
pub(crate) fn forbidden_marker_findings(
    src_dir: &Path,
    root_file: &Path,
    subtree: &str,
    forbidden: &[String],
    crate_package: &str,
) -> Result<Vec<(SemanticFact, String, PathBuf)>, String> {
    // A forbidden entry with an empty `::`-segment could never match a real leaf identifier — a
    // trailing/doubled `::` (or the empty string) makes `leaf_of` compute an empty leaf, and no
    // real identifier is ever empty; a leading `::` is harmless for leaf matching alone but
    // rejected anyway, for consistency with every other forbidden/allowed-operand-shaped DSL
    // method in this family (see `resolve::has_empty_path_segment`'s own doc). Checked before any
    // scanning, mirroring `exposure::module_findings`'s guard for its own forbidden set.
    validate_path_operands(forbidden)?;
    let scan = scan_crate(src_dir, root_file, crate_package, &HashSet::new())?;
    let subtree = canonical_path_str(subtree);
    // The canonical paths of every type the crate actually DEFINES — the only types that can
    // "acquire" a marker. A trait impl's self type is cross-checked against this so a foreign or
    // prelude self type (`impl Marker for Vec<u8>` / `Box<…>`), whose bare head the
    // `CurrentModule` fallback would otherwise fabricate into a phantom `crate::<mod>::Vec`, is not
    // mistaken for a governed-subtree type (a false positive). The derive form already scans only
    // these definitions, so the impl form now shares the same authoritative set.
    let defined: HashSet<&str> = scan
        .type_defs
        .iter()
        .map(|td| td.canonical.as_str())
        .collect();

    let mut findings = Vec::new();
    for entry in forbidden {
        let entry_leaf = leaf_of(entry);
        findings.extend(derive_marker_findings(&scan, &subtree, entry, entry_leaf));
        findings.extend(impl_marker_findings(
            &scan, &subtree, &defined, entry, entry_leaf,
        ));
    }
    // Dedup BY FINDING (keep the first module), so the count is identical to before — `file` is
    // metadata attached to a finding, never a second identity key.
    sort_attributed_facts(&mut findings)?;
    Ok(findings)
}

/// Derive form: a `#[derive(...)]` on a type defined under `subtree` whose leaf matches
/// `entry_leaf`. Resolves the written derive path through the defining module's `use`-map before
/// leaf-matching, so a locally renamed derive macro (`use serde::Serialize as Ser;
/// #[derive(Ser)]`) reacts by its true leaf; an unresolved bare/prelude/extern path falls back to
/// its written leaf, so leaf-matching stays cross-crate-blind (a `serde_derive::Serialize` path
/// still matches the leaf `Serialize`). Checks EVERY use-map candidate (cfg-blind): a
/// mutually-exclusive `#[cfg]`-gated alias for the derive's name must not have its other
/// candidate's leaf silently dropped (found on adversarial review of
/// `hunyi-cfg-branch-use-reexport-merging`).
fn derive_marker_findings(
    scan: &CrateScan,
    subtree: &str,
    entry: &str,
    entry_leaf: &str,
) -> Vec<(SemanticFact, String, PathBuf)> {
    let mut findings = Vec::new();
    for td in &scan.type_defs {
        if !under_subtree(&td.canonical, subtree) {
            continue;
        }
        for (ordinal, derived) in td.derives.iter().enumerate() {
            let derived_leaves = resolved_leaves(derived, &td.uses, &td.module);
            if derived_leaves.iter().any(|leaf| leaf == entry_leaf) {
                // A derive sits in the defining type's module — its source file, not any
                // impl site's. Render the marker from the WRITTEN derive path so two distinct
                // forbidden derives sharing a leaf on one type (`#[derive(a::Marker, b::Marker)]`)
                // stay distinct findings. An unrenderable path carries an internal positional
                // sentinel that the shared sorting reaction rejects before emission.
                let marker =
                    path_to_string(derived).unwrap_or_else(|| format!("{entry}<_#{ordinal}>"));
                findings.push((
                    SemanticFact::ForbiddenDerive {
                        marker,
                        canonical: td.canonical.clone(),
                    },
                    td.module.clone(),
                    td.file.clone(),
                ));
            }
        }
    }
    findings
}

/// Impl form: `impl T for X` (anywhere) whose self-type is a crate-defined type under `subtree`,
/// and whose trait leaf matches `entry_leaf`. Resolves the written trait path through the impl
/// site's `use`-map before leaf-matching, so a locally renamed trait (`use serde::Serialize as
/// Ser; impl Ser for …`) reacts by its true leaf; an unresolved bare/prelude/extern path falls
/// back to its written leaf, keeping leaf-matching cross-crate-blind (a
/// `serde_derive::Serialize` still matches). Checks EVERY use-map candidate (cfg-blind), the
/// identical treatment the derive form gets (found on adversarial review of
/// `hunyi-cfg-branch-use-reexport-merging`).
fn impl_marker_findings(
    scan: &CrateScan,
    subtree: &str,
    defined: &HashSet<&str>,
    entry: &str,
    entry_leaf: &str,
) -> Vec<(SemanticFact, String, PathBuf)> {
    let mut findings = Vec::new();
    for (ordinal, site) in scan.impls.iter().enumerate() {
        let trait_leaves = resolved_leaves(&site.trait_path, &site.uses, &site.module);
        if !trait_leaves.iter().any(|leaf| leaf == entry_leaf) {
            continue;
        }
        // The concrete type the marker LANDS on: `resolve_self_type` follows the re-export and
        // type-alias closures to the definition, so `impl Marker for crate::facade::Order` (a
        // `pub use` facade) and `impl Marker for Bar` where `type Bar = Real` both land on the
        // real subtree def, while a foreign/prelude self (`impl Marker for Vec<u8>`, fabricated by
        // the CurrentModule fallback into a phantom `crate::<mod>::Vec`) or an alias to a foreign
        // type (`type Baz = Vec<u8>`) lands off the governed subtree — each rejected by the
        // `defined` + `under_subtree` gate below (a false positive). Only a crate-DEFINED type
        // under the subtree can acquire a marker.
        // Every landing candidate is checked (cfg-blind): a self type reached through a
        // mutually-exclusive `#[cfg]`-gated alias must not have its other candidate's landing
        // silently dropped (found on adversarial review of
        // `hunyi-cfg-branch-use-reexport-merging`).
        let landings = resolve_self_type(
            &site.self_ty,
            &site.uses,
            &site.module,
            &scan.alias_targets,
            &scan.reexports,
            &site.type_params,
        );
        if landings.is_empty() {
            continue; // self-type not placeable (glob/external/complex) — a stated bound
        }
        if !landings
            .iter()
            .any(|landing| under_subtree(landing, subtree) && defined.contains(landing.as_str()))
        {
            continue;
        }
        // Injective identity: the written trait path WITH generic args, the self type WITH
        // generic args (owner-qualified like the seam owner), and the impl-site module. Two
        // distinct acquisitions — `impl Marker<u8>`/`impl Marker<u16>`, or the same leaf from
        // different modules — thus stay distinct findings, so a baseline cannot mask a new one.
        // An unrenderable trait arg carries the config entry plus an internal positional
        // sentinel; the shared sorting reaction rejects it before public identity emission.
        let marker =
            path_to_string(&site.trait_path).unwrap_or_else(|| format!("{entry}<_#{ordinal}>"));
        let owner = canonical_self_owner(
            &site.self_ty,
            &site.uses,
            &site.module,
            ordinal,
            &site.type_params,
        );
        findings.push((
            SemanticFact::ForbiddenImpl {
                marker,
                owner,
                module: site.module.clone(),
            },
            site.module.clone(),
            site.file.clone(),
        ));
    }
    findings
}
