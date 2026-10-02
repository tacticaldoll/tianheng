//! Static-item boundary (`semantic-static-item-boundary`): a module's subtree declares no `static`
//! item, foreign `static` or `thread_local!`. Walk the whole compilation unit for declared statics
//! and react to those whose declaring module lies within the anchored subtree.

use std::path::{Path, PathBuf};

use serde_json::Value;
use xuanji::{Outcome, Polarity, Violation};

use crate::anchor::canonical_module_anchor;
use crate::containment::under_subtree;
use crate::driver::run_boundaries;
use crate::dsl::StaticBoundary;
use crate::emit::{MultiModuleViolationContext, push_multi_module_violations};
use crate::file_scope::{UnitAnchor, over_each_unit, resolve_crate_units};
use crate::finding::{SemanticFact, sort_attributed_facts};
use crate::rules::STATIC_ITEM_RULE;
use crate::scan::{refuse_thread_local_rename, scan_static_sites};

/// Run the static-item boundaries against the Cargo workspace at `manifest_path`.
///
/// Resolve each boundary's crate and module anchor, collect the statics declared in the anchored
/// subtree, and react. An unresolvable crate or module, an unreadable or unparseable source, a
/// renamed `thread_local`, or a `thread_local!` body in the anchored subtree that is not `static`
/// declarations is a constitution error (exit 2), never a silent pass. The same body outside the
/// subtree is ungoverned and does not stop the unit being read for a rename.
pub fn check_static_item(boundaries: &[StaticBoundary], manifest_path: &Path) -> Outcome {
    run_boundaries(boundaries, manifest_path, check_static_boundary)
}

pub(crate) fn check_static_boundary(
    metadata: &Value,
    boundary: &StaticBoundary,
    violations: &mut Vec<Violation>,
) -> Result<(), String> {
    let module = canonical_module_anchor(&boundary.module, &boundary.crate_package)?;
    let (_package, units) = resolve_crate_units(metadata, &boundary.crate_package)?;
    let anchor = UnitAnchor::Module {
        module: &module,
        crate_package: &boundary.crate_package,
    };
    over_each_unit(&units, anchor, |root_file, src_dir, unit| {
        let findings = static_item_findings(src_dir, root_file, &module, &boundary.crate_package)?;
        push_multi_module_violations(
            violations,
            MultiModuleViolationContext {
                target: &module,
                rule: STATIC_ITEM_RULE,
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
        Ok(())
    })
}

/// The pure heart: the sorted, deduplicated `(fact, declaring module, file)` triples for every
/// static declared at or beneath `module` in one compilation unit.
///
/// Precondition: `module` exists in this unit's module graph (verified by [`over_each_unit`]).
/// The whole unit is then scanned, because a rename of `thread_local` anywhere in it
/// is refused before any finding is reported.
pub(crate) fn static_item_findings(
    src_dir: &Path,
    root_file: &Path,
    module: &str,
    crate_package: &str,
) -> Result<Vec<(SemanticFact, String, PathBuf)>, crate::errors::ResolveError> {
    let scan = scan_static_sites(src_dir, root_file, crate_package)
        .map_err(crate::errors::ResolveError::Other)?;
    refuse_thread_local_rename(&scan, crate_package)?;
    if let Some(err) = scan
        .errors
        .into_iter()
        .find(|e| under_subtree(&e.module, module))
    {
        return Err(crate::errors::ResolveError::Other(err.error));
    }
    let mut findings: Vec<(SemanticFact, String, PathBuf)> = scan
        .sites
        .into_iter()
        .filter(|site| under_subtree(&site.module, module))
        .map(|site| {
            (
                SemanticFact::StaticItem {
                    module: site.module.clone(),
                    kind: site.kind,
                    name: site.name,
                    owner: site.owner,
                },
                site.module,
                site.file,
            )
        })
        .collect();
    sort_attributed_facts(&mut findings)?;
    Ok(findings)
}
