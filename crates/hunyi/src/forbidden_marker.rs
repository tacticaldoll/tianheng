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
use crate::file_scope::resolve_crate;
use crate::finding::{SemanticFact, sort_attributed_facts};
use crate::resolve::{
    BareFallback, canonical_path_str, canonical_self_owner, path_to_string, resolve_path,
};
use crate::rules::FORBIDDEN_MARKER_RULE;
use crate::scan::scan_crate;

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
    let (_package, root_file, src_dir) = resolve_crate(metadata, &boundary.crate_package)?;
    let src_dir = src_dir.as_path();

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
            reason: &boundary.reason,
            severity: boundary.severity,
            anchor: boundary.anchor(),
            polarity: Polarity::DenyBreach,
        },
        findings,
    );
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

        // Derive form: a derive on a type defined under the subtree.
        for td in &scan.type_defs {
            if !under_subtree(&td.canonical, &subtree) {
                continue;
            }
            for (ordinal, derived) in td.derives.iter().enumerate() {
                // Resolve the written derive path through the defining module's `use`-map before
                // leaf-matching, so a locally renamed derive macro (`use serde::Serialize as Ser;
                // #[derive(Ser)]`) reacts by its true leaf; an unresolved bare/prelude/extern path
                // falls back to its written leaf, so leaf-matching stays cross-crate-blind (a
                // `serde_derive::Serialize` path still matches the leaf `Serialize`).
                let derived_leaf =
                    resolve_path(derived, &td.uses, &td.module, BareFallback::Ignore)
                        .map(|p| leaf_of(&p).to_string())
                        .unwrap_or_else(|| path_leaf(derived));
                if derived_leaf == entry_leaf {
                    // A derive sits in the defining type's module — its source file, not any
                    // impl site's. Render the marker from the WRITTEN derive path so two distinct
                    // forbidden derives sharing a leaf on one type (`#[derive(a::Marker, b::Marker)]`)
                    // stay distinct findings; an unrenderable path falls back to the config entry
                    // plus the derive's position, never collapsing (finding-identity injectivity).
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

        // Impl form: `impl T for X` (anywhere) whose self-type is a crate-defined type under the
        // subtree.
        for (ordinal, site) in scan.impls.iter().enumerate() {
            // Resolve the written trait path through the impl site's `use`-map before leaf-matching,
            // so a locally renamed trait (`use serde::Serialize as Ser; impl Ser for …`) reacts by
            // its true leaf; an unresolved bare/prelude/extern path falls back to its written leaf,
            // keeping leaf-matching cross-crate-blind (a `serde_derive::Serialize` still matches).
            let trait_leaf = resolve_path(
                &site.trait_path,
                &site.uses,
                &site.module,
                BareFallback::Ignore,
            )
            .map(|p| leaf_of(&p).to_string())
            .unwrap_or_else(|| path_leaf(&site.trait_path));
            if trait_leaf != entry_leaf {
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
            let Some(landing) = resolve_self_type(
                &site.self_ty,
                &site.uses,
                &site.module,
                &scan.alias_targets,
                &scan.reexports,
                &site.type_params,
            ) else {
                continue; // self-type not placeable (glob/external/complex) — a stated bound
            };
            if !(under_subtree(&landing, &subtree) && defined.contains(landing.as_str())) {
                continue;
            }
            // Injective identity: the written trait path WITH generic args, the self type WITH
            // generic args (owner-qualified like the seam owner), and the impl-site module. Two
            // distinct acquisitions — `impl Marker<u8>`/`impl Marker<u16>`, or the same leaf from
            // different modules — thus stay distinct findings, so a baseline cannot mask a new one.
            // An unrenderable trait arg falls back to the config entry PLUS the impl's position
            // (never the bare entry alone), keeping distinct unrenderable-arg impls distinct.
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
    }
    // Dedup BY FINDING (keep the first module), so the count is identical to before — `file` is
    // metadata attached to a finding, never a second identity key.
    sort_attributed_facts(&mut findings);
    Ok(findings)
}
