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
use crate::errors::{ambiguous_trait_anchor_error, unknown_trait_error};
use crate::file_scope::resolve_crate;
use crate::finding::{SemanticFact, sort_attributed_facts};
use crate::resolve::{
    AliasMap, BareFallback, canonical_path_str, canonical_self_owner, expand_canonical_paths,
    render_last_segment_args, resolve_path_all, validate_path_operands,
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

    let TraitImplReaction { anchor, findings } = trait_impl_findings(
        src_dir,
        &root_file,
        &boundary.trait_path,
        &boundary.allowed_locations,
        &boundary.crate_package,
    )?;

    // Both identity components are keyed on the RESOLVED anchor, not the declared spelling: the same
    // trait declared through a facade `pub use` and through its defining path is one governed thing,
    // so it must produce one `ViolationId`. Matching already resolved both sides; only identity kept
    // the raw declaration, so renaming a constitution declaration between two equivalent spellings —
    // a pure refactor with no code change — silently invalidated every affected baseline entry.
    //
    // `allowed_locations` remains inside the rule key. An earlier version of this comment claimed the
    // opposite ("not part of the violation's identity — so editing the allowed set does not turn a
    // still-misplaced impl into a new violation"), which `ViolationId`'s own equality contradicts: it
    // compares `rule_key` in full. Keeping the allowed set in the key is what stops two boundaries
    // governing the same trait with different allowed sets from collapsing onto one identity for one
    // misplaced impl, which would let a baseline accepting the first suppress the second's
    // never-accepted violation. The cost is real and now stated rather than denied: editing the
    // allowed set re-fires still-misplaced impls as new and reports the old entries stale — loud
    // churn, never masking.
    let target = anchor;
    // Each finding carries the module its offending impl sits in; the shared emit helper resolves
    // that module's source file (memoized per module) and stamps the allowlist-gap polarity.
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
        },
        findings,
    );
    Ok(())
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
pub(crate) fn trait_impl_findings(
    src_dir: &Path,
    root_file: &Path,
    trait_path: &str,
    allowed: &[String],
    crate_package: &str,
) -> Result<TraitImplReaction, String> {
    // An allowed-location entry with an empty `::`-segment could never contain a real module
    // location — checked before any scanning, the identical guard `must_not_expose`/
    // `must_not_acquire`'s forbidden-operand family applies to their own operand lists
    // (`resolve::validate_path_operands`). Left unvalidated, such an entry silently matched no
    // real site in `matches_allowed`, misreporting every legitimately-placed impl as a spurious
    // violation instead of naming the actual typo.
    validate_path_operands(allowed)?;
    let scan = scan_crate(src_dir, root_file, crate_package, &HashSet::new())?;
    let given = canonical_path_str(trait_path);
    // Every re-export candidate the declared anchor's own facade could denote (cfg-blind): a
    // `pub use` closure reached through a mutually-exclusive `#[cfg]` collision must not have its
    // other candidate silently dropped (found on adversarial review of
    // `hunyi-cfg-branch-use-reexport-merging`).
    let true_anchors = expand_canonical_paths(&given, &AliasMap::new(), &scan.reexports);
    // The one anchor the declaration actually denotes — a trait DEFINITION among its re-export
    // candidates. This, not the declared spelling, becomes the violation's `target` and rule key
    // below, so declaring the same trait through a facade `pub use` and through its defining path
    // yields one identity instead of two for the same real-world fact (a pure declaration refactor
    // used to silently invalidate every affected baseline entry). For a declaration that already
    // names the defining path — the ordinary case — this is the declared string itself, so nothing
    // moves.
    let mut defining_anchors: Vec<String> = true_anchors
        .iter()
        .filter(|anchor| scan.trait_defs.contains(*anchor))
        .cloned()
        .collect();
    defining_anchors.sort();
    defining_anchors.dedup();
    let anchor = match defining_anchors.len() {
        0 => return Err(unknown_trait_error(trait_path, crate_package)),
        1 => defining_anchors.remove(0),
        // The facade denotes two different traits (cfg-collided `pub use`). Choosing one would make
        // identity arbitrary, and it is the declaration that is ambiguous — so the adopter hears it.
        _ => {
            return Err(ambiguous_trait_anchor_error(
                trait_path,
                crate_package,
                &defining_anchors,
            ));
        }
    };
    let allowed: Vec<String> = allowed.iter().map(|a| canonical_path_str(a)).collect();

    let mut findings = Vec::new();
    for (ordinal, site) in scan.impls.iter().enumerate() {
        let resolved_candidates = resolve_path_all(
            &site.trait_path,
            &site.uses,
            &site.module,
            BareFallback::CurrentModule,
        );
        if resolved_candidates.is_empty() {
            // The trait path did not resolve (a glob/macro bound) — not silently matched.
            continue;
        }
        // Every candidate, through every re-export candidate, is checked against every possible
        // anchor (cfg-blind on both sides of the match). The rendered identity below uses
        // whichever candidate actually matched, so the label reflects the reason for the reaction.
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
        // The finding identifies the offending impl by its module location, the **written trait
        // path with its generic arguments**, and its implemented-for type (canonicalized like the
        // inherent-impl seam owner). Including the trait's generic args keeps two distinct
        // instantiations for the same self type — `impl Convert<u8> for Foo` and
        // `impl Convert<u16> for Foo`, both legal and coherent — as distinct findings, so a baseline
        // accepting one cannot mask the other (finding-identity injectivity). The self type is
        // likewise retained when renderable; an unrenderable expression carries an internal
        // positional sentinel that the shared sorting reaction rejects. Stated label bound: a
        // trait impl's self type MAY be foreign (`impl LocalTrait for Box<Foo>`), which the
        // module-relative canonicalization over-qualifies (`crate::m::Box<…>`) — a stable identity
        // label, not a resolved-path claim; the actionable part (the module location) is exact.
        let owner = canonical_self_owner(
            &site.self_ty,
            &site.uses,
            &site.module,
            ordinal,
            &site.type_params,
        );
        // The canonical anchor (spelling-stable across `use`/rename/relative forms) plus the
        // written generic arguments. An unrenderable arg carries the same rejected internal
        // sentinel, so it fails loud rather than becoming public positional identity.
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
    sort_attributed_facts(&mut findings)?;
    Ok(TraitImplReaction { anchor, findings })
}
