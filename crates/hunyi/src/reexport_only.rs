//! Re-export-only module observation.

use serde_json::Value;
use std::path::{Path, PathBuf};
use xuanji::{Outcome, Polarity, ScanDepth, Violation};

use crate::driver::run_boundaries;
use crate::dsl::ReexportOnlyBoundary;
use crate::emit::{MultiModuleViolationContext, push_multi_module_violations};
use crate::errors::unknown_module_error;
use crate::file_scope::{over_each_unit, resolve_crate_units};
use crate::finding::{SemanticFact, sort_attributed_facts};
use crate::module_resolve::resolve_module_direct_items_with_files;
use crate::resolve::{path_to_string, type_to_string};
use crate::rules::REEXPORT_ONLY_RULE;
use crate::scan::walk_subtree_direct_modules;

/// Evaluate re-export-only boundaries against a Cargo workspace.
pub fn check_reexport_only(boundaries: &[ReexportOnlyBoundary], manifest_path: &Path) -> Outcome {
    run_boundaries(boundaries, manifest_path, check_reexport_only_boundary)
}

pub(crate) fn check_reexport_only_boundary(
    metadata: &Value,
    boundary: &ReexportOnlyBoundary,
    violations: &mut Vec<Violation>,
) -> Result<(), String> {
    let (_package, units) = resolve_crate_units(metadata, &boundary.crate_package)?;
    over_each_unit(
        &units,
        &unknown_module_error(&boundary.module, &boundary.crate_package),
        |root_file, src_dir, unit| {
            let findings = reexport_only_findings(
                src_dir,
                root_file,
                &boundary.module,
                &boundary.crate_package,
                boundary.depth,
            )?;
            push_multi_module_violations(
                violations,
                MultiModuleViolationContext {
                    target: &boundary.module,
                    rule: REEXPORT_ONLY_RULE,
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
        },
    )
}

pub(crate) fn reexport_only_findings(
    src_dir: &Path,
    root_file: &Path,
    module: &str,
    crate_package: &str,
    depth: ScanDepth,
) -> Result<Vec<(SemanticFact, String, PathBuf)>, String> {
    let modules = match depth {
        ScanDepth::Shallow => {
            resolve_module_direct_items_with_files(src_dir, root_file, module, crate_package)?
                .into_iter()
                .map(|(item, file)| (module.to_string(), vec![item], file))
                .collect()
        }
        ScanDepth::Subtree => {
            walk_subtree_direct_modules(src_dir, root_file, module, crate_package)?
        }
        _ => return Err("unsupported re-export-only scan depth".into()),
    };
    let mut findings = Vec::new();
    for (mod_path, items, file) in modules {
        for item in items {
            if matches!(item, syn::Item::Use(_))
                || (depth == ScanDepth::Subtree && matches!(item, syn::Item::Mod(_)))
            {
                continue;
            }
            let (item_kind, item_name) = describe_item(&item);
            findings.push((
                SemanticFact::DeclaredItemKind {
                    module: mod_path.clone(),
                    item_kind,
                    item_name,
                },
                mod_path.clone(),
                file.clone(),
            ));
        }
    }
    sort_attributed_facts(&mut findings)?;
    Ok(findings)
}

pub(crate) fn describe_item(item: &syn::Item) -> (String, String) {
    match item {
        syn::Item::Fn(i) => ("fn".into(), i.sig.ident.to_string()),
        syn::Item::Struct(i) => ("struct".into(), i.ident.to_string()),
        syn::Item::Enum(i) => ("enum".into(), i.ident.to_string()),
        syn::Item::Union(i) => ("union".into(), i.ident.to_string()),
        syn::Item::Type(i) => ("type".into(), i.ident.to_string()),
        syn::Item::Const(i) => ("const".into(), i.ident.to_string()),
        syn::Item::Static(i) => ("static".into(), i.ident.to_string()),
        syn::Item::Trait(i) => ("trait".into(), i.ident.to_string()),
        syn::Item::TraitAlias(i) => ("trait alias".into(), i.ident.to_string()),
        syn::Item::Mod(i) => ("mod".into(), i.ident.to_string()),
        syn::Item::ExternCrate(i) => ("extern crate".into(), i.ident.to_string()),
        syn::Item::ForeignMod(_) => ("extern block".into(), "".into()),
        syn::Item::Impl(i) => {
            let owner = type_to_string(&i.self_ty).unwrap_or_else(|| "<unrenderable>".into());
            let name = match &i.trait_ {
                Some((_, path, _)) => format!(
                    "{} for {owner}",
                    path_to_string(path).unwrap_or_else(|| "<unrenderable>".into())
                ),
                None => owner,
            };
            ("impl".into(), name)
        }
        syn::Item::Macro(i) => {
            let path = path_to_string(&i.mac.path).unwrap_or_else(|| "<unrenderable>".into());
            if let Some(name) = &i.ident {
                ("macro_rules".into(), name.to_string())
            } else {
                ("macro".into(), format!("{path}!"))
            }
        }
        syn::Item::Verbatim(_) => ("verbatim".into(), "<unrenderable>".into()),
        syn::Item::Use(_) => unreachable!("use is permitted"),
        _ => ("unknown".into(), "<unrenderable>".into()),
    }
}
