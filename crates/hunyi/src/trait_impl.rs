//! Trait-impl-locality (`semantic-trait-impl-locality`): a trait may be implemented only in its
//! declared location(s). Scan the whole crate for `impl <Trait> for <Type>` sites, resolve the
//! anchor (re-export-aware) to a real local trait, and react to the anchored trait's impls whose
//! module location lies outside the allowed set.

use std::collections::HashSet;
use std::path::{Path, PathBuf};

use serde_json::Value;
use xuanji::{Outcome, Polarity, Violation};

use crate::containment::matches_allowed;
use crate::driver::run_boundaries;
use crate::dsl::TraitImplBoundary;
use crate::emit::{MultiModuleViolationContext, push_multi_module_violations};
use crate::errors::unknown_trait_error;
use crate::file_scope::resolve_crate;
use crate::finding::{SemanticFact, sort_attributed_facts};
use crate::resolve::{
    BareFallback, canonical_path_str, canonical_self_owner, canonicalize_through_reexports,
    render_last_segment_args, resolve_path,
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

pub(crate) fn check_trait_impl_boundary(
    metadata: &Value,
    boundary: &TraitImplBoundary,
    violations: &mut Vec<Violation>,
) -> Result<(), String> {
    let (_package, root_file, src_dir) = resolve_crate(metadata, &boundary.crate_package)?;
    let src_dir = src_dir.as_path();

    let findings = trait_impl_findings(
        src_dir,
        &root_file,
        &boundary.trait_path,
        &boundary.allowed_locations,
        &boundary.crate_package,
    )?;

    // A fixed rule string: the allowed locations are policy configuration (surfaced in the
    // `list` projection and the reason), not part of the violation's identity — so editing
    // the allowed set does not turn a still-misplaced impl into a "new" violation against a
    // baseline (mirroring how `xuanji` excludes reason/severity from the violation id). The
    // violation `target` is the canonical trait anchor (spelling-stable across use/rename forms).
    let target = canonical_path_str(&boundary.trait_path);
    // Each finding carries the module its offending impl sits in; the shared emit helper resolves
    // that module's source file (memoized per module) and stamps the allowlist-gap polarity.
    push_multi_module_violations(
        violations,
        MultiModuleViolationContext {
            target: &target,
            rule: TRAIT_IMPL_RULE,
            reason: &boundary.reason,
            severity: boundary.severity,
            anchor: boundary.anchor(),
            polarity: Polarity::AllowlistGap,
        },
        findings,
    );
    Ok(())
}

/// The pure heart, testable without spawning `cargo`: scan the whole crate for trait
/// impls and re-exports, resolve the anchor (re-export-aware) to a real local trait —
/// else a constitution error — then return the sorted, deduplicated findings: the impls
/// of the anchored trait whose module location lies outside the allowed set.
pub(crate) fn trait_impl_findings(
    src_dir: &Path,
    root_file: &Path,
    trait_path: &str,
    allowed: &[String],
    crate_package: &str,
) -> Result<Vec<(SemanticFact, String, PathBuf)>, String> {
    let scan = scan_crate(src_dir, root_file, crate_package, &HashSet::new())?;
    let given = canonical_path_str(trait_path);
    let true_anchor = canonicalize_through_reexports(&given, &scan.reexports);
    if !scan.trait_defs.contains(&true_anchor) {
        return Err(unknown_trait_error(trait_path, crate_package));
    }
    let allowed: Vec<String> = allowed.iter().map(|a| canonical_path_str(a)).collect();

    let mut findings = Vec::new();
    for (ordinal, site) in scan.impls.iter().enumerate() {
        let Some(resolved) = resolve_path(
            &site.trait_path,
            &site.uses,
            &site.module,
            BareFallback::CurrentModule,
        ) else {
            // The trait path did not resolve (a glob/macro bound) — not silently matched.
            continue;
        };
        let canonical = canonicalize_through_reexports(&resolved, &scan.reexports);
        if canonical != true_anchor {
            continue;
        }
        if matches_allowed(&site.module, &allowed) {
            continue;
        }
        // The finding identifies the offending impl by its module location, the **written trait
        // path with its generic arguments**, and its implemented-for type (canonicalized like the
        // inherent-impl seam owner). Including the trait's generic args keeps two distinct
        // instantiations for the same self type — `impl Convert<u8> for Foo` and
        // `impl Convert<u16> for Foo`, both legal and coherent — as distinct findings, so a baseline
        // accepting one cannot mask the other (finding-identity injectivity). The self type is
        // likewise disambiguated even under an unrenderable const-generic expression (then keyed by
        // the impl's position). Stated label bound: a trait impl's self type MAY be foreign
        // (`impl LocalTrait for Box<Foo>`), which the module-relative canonicalization over-qualifies
        // (`crate::m::Box<…>`) — a stable identity label, not a resolved-path claim; the actionable
        // part (the module location) is exact.
        let owner = canonical_self_owner(&site.self_ty, &site.uses, &site.module, ordinal);
        // The canonical anchor (spelling-stable across `use`/rename/relative forms) plus the
        // written generic arguments; an unrenderable arg (a complex const-generic expression) falls
        // back to the impl's position, so two such impls still stay distinct rather than collapsing.
        let trait_ref = format!(
            "{canonical}{}",
            render_last_segment_args(&site.trait_path).unwrap_or_else(|| format!("<_#{ordinal}>"))
        );
        // Pair the finding with the module the offending impl sits in, so the reaction layer can
        // report its source file. Dedup BY FINDING (below) keeps the count identical to before —
        // `file` is metadata, never a second identity key.
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
    sort_attributed_facts(&mut findings);
    Ok(findings)
}
