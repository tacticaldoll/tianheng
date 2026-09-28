//! Trait-impl-locality (`semantic-trait-impl-locality`): a trait may be implemented only in its
//! declared location(s). Scan the whole crate for `impl <Trait> for <Type>` sites, resolve the
//! anchor (re-export-aware) to a real local trait, and react to the anchored trait's impls whose
//! module location lies outside the allowed set.

use std::collections::HashSet;
use std::path::{Path, PathBuf};

use serde_json::Value;
use xuanji::{Outcome, Polarity, Violation};

use crate::anchor::{canonical_module_locations, require_locations_exist};
use crate::containment::matches_allowed;
use crate::driver::run_boundaries;
use crate::dsl::TraitImplBoundary;
use crate::emit::{MultiModuleViolationContext, push_multi_module_violations};
use crate::file_scope::{UnitAnchor, over_each_unit, resolve_crate_units};
use crate::finding::{SemanticFact, sort_attributed_facts};
use crate::resolve::{
    AliasMap, BareFallback, canonical_path_str, canonical_self_owner, expand_canonical_paths,
    render_last_segment_args, resolve_path_all,
};
use crate::rules::TRAIT_IMPL_RULE;
use crate::scan::scan_crate;

/// Run the trait-impl-locality boundaries against the Cargo workspace at `manifest_path`.
///
/// Mirrors [`crate::check`]: resolve each boundary's crate and trait anchor, walk the crate for
/// `impl <Trait> for <Type>` sites, react to those of the anchored trait whose module
/// location is outside the allowed set, and return the outcome. An unresolvable crate or
/// trait anchor (or an unreadable/unparseable source) is a constitution error (exit 2),
/// never a silent pass.
pub fn check_trait_impl_locality(
    boundaries: &[TraitImplBoundary],
    manifest_path: &Path,
) -> Outcome {
    run_boundaries(boundaries, manifest_path, check_trait_impl_boundary)
}

/// Check a single trait-impl boundary against the resolved crate compilation units.
///
/// Evaluates each unit separately so unit-varying traits and misplaced impls (e.g. in binaries)
/// are observed with unit identity. Identity components are keyed on the resolved defining anchor.
pub(crate) fn check_trait_impl_boundary(
    metadata: &Value,
    boundary: &TraitImplBoundary,
    violations: &mut Vec<Violation>,
) -> Result<(), String> {
    let allowed = canonical_module_locations(&boundary.allowed_locations, &boundary.crate_package)?;
    let (_package, units) = resolve_crate_units(metadata, &boundary.crate_package)?;
    require_locations_exist(&units, &allowed, &boundary.crate_package)?;
    let anchor = UnitAnchor::Trait {
        trait_path: &boundary.trait_path,
        crate_package: &boundary.crate_package,
    };
    over_each_unit(&units, anchor, |root_file, src_dir, unit| {
        let TraitImplReaction { anchor, findings } = trait_impl_findings(
            src_dir,
            root_file,
            &boundary.trait_path,
            &allowed,
            &boundary.crate_package,
        )?;

        let target = anchor;
        push_multi_module_violations(
            violations,
            MultiModuleViolationContext {
                target: &target,
                rule: TRAIT_IMPL_RULE,
                rule_key: boundary.rule_key_for_anchor(&target),
                reason: &boundary.reason,
                severity: boundary.severity,
                anchor: boundary.anchor(),
                polarity: Polarity::AllowlistGap,
                crate_package: &boundary.crate_package,
                unit,
            },
            findings,
        );
        Ok(())
    })
}

/// One trait-impl-locality evaluation's result: the anchor the declaration resolved to, and the
/// misplaced impls found under it.
///
/// The anchor is returned rather than recomputed by the caller because it is an identity role — the
/// violation's `target` and its rule key's `trait` field — and recomputing it would mean a second
/// resolution site free to disagree with the one that decided the matches.
pub(crate) struct TraitImplReaction {
    pub(crate) anchor: String,
    pub(crate) findings: Vec<(SemanticFact, String, PathBuf)>,
}

/// The pure heart, testable without spawning `cargo`: scan the whole crate for trait
/// impls and re-exports, resolve the anchor (re-export-aware) to a real local trait —
/// else a constitution error — then return that anchor with the sorted, deduplicated findings: the
/// impls of the anchored trait whose module location lies outside the allowed set.
///
/// Allowed locations must each be a canonical module anchor (`crate::…`).
/// Finding identities preserve written generic arguments and canonicalized self types.
pub(crate) fn trait_impl_findings(
    src_dir: &Path,
    root_file: &Path,
    trait_path: &str,
    allowed: &[String],
    crate_package: &str,
) -> Result<TraitImplReaction, crate::errors::ResolveError> {
    let allowed = canonical_module_locations(allowed, crate_package)
        .map_err(crate::errors::ResolveError::Other)?;
    let scan = scan_crate(src_dir, root_file, crate_package, &HashSet::new())
        .map_err(crate::errors::ResolveError::Other)?;
    let given = canonical_path_str(trait_path);
    let true_anchors = expand_canonical_paths(&given, &AliasMap::new(), &scan.reexports);
    let mut defining_anchors: Vec<String> = true_anchors
        .iter()
        .filter(|anchor| scan.trait_defs.contains(*anchor))
        .cloned()
        .collect();
    defining_anchors.sort();
    defining_anchors.dedup();
    let anchor = match defining_anchors.len() {
        0 => {
            return Err(crate::errors::ResolveError::UnknownTrait(
                trait_path.to_string(),
                crate_package.to_string(),
            ));
        }
        1 => defining_anchors.remove(0),
        _ => {
            return Err(crate::errors::ResolveError::AmbiguousTraitAnchor(
                trait_path.to_string(),
                crate_package.to_string(),
                defining_anchors,
            ));
        }
    };

    let mut findings = Vec::new();
    for (ordinal, site) in scan.impls.iter().enumerate() {
        let resolved_candidates = resolve_path_all(
            &site.trait_path,
            &site.uses,
            &site.module,
            BareFallback::CurrentModule,
        );
        if resolved_candidates.is_empty() {
            continue;
        }
        let canonical_candidates: Vec<String> = resolved_candidates
            .iter()
            .flat_map(|resolved| {
                expand_canonical_paths(resolved, &AliasMap::new(), &scan.reexports)
            })
            .collect();
        let Some(canonical) = canonical_candidates
            .iter()
            .find(|candidate| true_anchors.contains(candidate))
        else {
            continue;
        };
        if matches_allowed(&site.module, &allowed) {
            continue;
        }
        let owner = canonical_self_owner(
            &site.self_ty,
            &site.uses,
            &site.module,
            ordinal,
            &site.type_params,
        );
        let trait_ref = format!(
            "{canonical}{}",
            render_last_segment_args(&site.trait_path).unwrap_or_else(|| format!("<_#{ordinal}>"))
        );
        findings.push((
            SemanticFact::MisplacedImpl {
                module: site.module.clone(),
                trait_ref,
                owner,
            },
            site.module.clone(),
            site.file.clone(),
        ));
    }
    sort_attributed_facts(&mut findings)?;
    Ok(TraitImplReaction { anchor, findings })
}
