use std::path::{Path, PathBuf};

use serde_json::Value;
use xuanji::ScanDepth;

use crate::cargo_metadata::{crate_root_file, find_package};
use crate::errors::{
    confine_external_crate_on_crate_error, crate_not_found_error, inline_empty_prefix_error,
    inline_empty_verbs_error, inline_module_target_error, inline_narrow_and_strict_error,
    missing_src_error, must_not_be_imported_by_on_crate_error,
    must_only_be_imported_by_on_crate_error, restrict_imports_to_on_crate_error,
    unknown_module_error, unreadable_governed_file_error,
};
use crate::finding::ModuleFact;
use crate::module_scan::{
    InlineFinding, canonical_module_path, external_imports_with_importers, governed_files,
    imported_module_paths, imports_with_importers, inline_symbol_findings,
    package_name_to_import_ident, path_within, reachable_modules, rust_files,
};
use crate::{BoundaryKind, ModuleBoundary, ModuleRule, Violation, ViolationId};

/// The source-root directory for a package's lib/proc-macro/bin target (resolved by 星表's
/// `crate_root_file`). Prefer Cargo's observed `targets[].src_path` so custom `[lib] path =
/// "lib.rs"`, proc-macro, and bin-only crates are scanned at the real compiled root; fall back to
/// `manifest_dir/src` only for synthetic unit-test metadata that omits targets.
fn package_src_dir(package: &Value) -> Option<PathBuf> {
    crate_root_file(package)
        .and_then(|root| root.parent().map(Path::to_path_buf))
        .or_else(|| {
            package["manifest_path"]
                .as_str()
                .and_then(|manifest| Path::new(manifest).parent())
                .map(|crate_dir| crate_dir.join("src"))
        })
}

fn push_module_violation(
    violations: &mut Vec<Violation>,
    target: &str,
    rule: &str,
    fact: ModuleFact,
    file: String,
    boundary: &ModuleBoundary,
) {
    let finding = fact.into_finding();
    violations.push(
        Violation::new(
            BoundaryKind::Module,
            ViolationId::new(target, boundary.rule_key(), finding.key().clone()),
            rule,
            finding.text(),
            boundary.reason.clone(),
            boundary.severity,
        )
        .with_file(Some(file))
        .with_anchor(boundary.anchor.clone())
        .with_polarity(boundary.rule.polarity()),
    );
}

pub(crate) fn check_module_boundary(
    metadata: &Value,
    boundary: &ModuleBoundary,
    violations: &mut Vec<Violation>,
) -> Result<(), String> {
    let package = find_package(metadata, &boundary.crate_package)
        .ok_or_else(|| crate_not_found_error(&boundary.crate_package))?;
    let src_dir =
        package_src_dir(package).ok_or_else(|| missing_src_error(&boundary.crate_package))?;

    // The crate's real root file relative to `src_dir` — usually `lib.rs`/`main.rs`, but Cargo
    // permits a custom target root (`[lib] path = "src/core.rs"`), which must still map to `crate`.
    let root_relative = crate_root_file(package)
        .and_then(|rf| rf.strip_prefix(&src_dir).ok().map(|p| p.to_path_buf()));
    let files = rust_files(&src_dir)?;
    let (reachable, inline_only, remapped, remap_shadowed) =
        reachable_modules(&src_dir, &files, root_relative.as_deref())?;
    // The crate-root module names (direct children of `crate`) feed bare-`use` resolution
    // (a root-relative `use foo::…` is the local module only if `foo` is one of them).
    let root_modules: Vec<String> = reachable
        .iter()
        .filter_map(|module| {
            module
                .strip_prefix("crate::")
                .filter(|rest| !rest.contains("::"))
                .map(str::to_string)
        })
        .collect();
    // Canonicalize the declared module and forbidden paths (raw-identifier `r#name` ->
    // `name`) so they compare in the same vocabulary as the observed paths, which are
    // canonicalized at the file, `mod`, and `use` derivations. A boundary may be written
    // with either the raw or plain form and still match.
    let governed_module = canonical_module_path(&boundary.module);
    let governed = governed_files(
        &src_dir,
        &files,
        &governed_module,
        &reachable,
        &inline_only,
        &remapped,
        &remap_shadowed,
        root_relative.as_deref(),
        boundary.depth,
    );
    if governed.is_empty() {
        // Two distinct misconfigurations, kept apart so the error is self-describing
        // (PROJECT.md): an inline `mod name { … }` is reachable but owns no source file,
        // so it cannot be a governed target — module boundaries govern file-based modules.
        // A path that is not reachable at all is a genuinely unknown module (e.g. a typo) —
        // which now also covers a plain/`#[path]`-declared module whose sole declaration was
        // `#[cfg]`-tolerated away (reachable, but neither inline nor governed): anchoring
        // directly at a module absent on this build is "cannot judge," matching 渾儀's own
        // `descend` precedent for the identical shape (its empty-branches case also falls to
        // `unknown_module_error`, never a vacuous clean pass). Checked via `inline_only`
        // specifically, not `reachable` (`inline_only` ⊆ `reachable`), so this distinction holds.
        // Both exit 2, never a silent pass; only the message differs.
        if inline_only.contains(&governed_module) {
            let leaf = governed_module
                .rsplit("::")
                .next()
                .unwrap_or(&governed_module);
            return Err(inline_module_target_error(
                &boundary.module,
                &boundary.crate_package,
                leaf,
            ));
        }
        return Err(unknown_module_error(
            &boundary.module,
            &boundary.crate_package,
        ));
    }

    let rule = boundary.rule.label();

    // The inbound rules invert the scope: they scan every reachable file and test each
    // importing *module* (not an import path) against the rule, so they have their own
    // evaluation rather than the shared import-path predicate used by the outbound rules below.
    // `must_not_be_imported_by` reacts to an importer beneath a forbidden importer; the closed
    // dual `must_only_be_imported_by` reacts to any importer NOT within the allowlist.
    let inbound = matches!(
        &boundary.rule,
        ModuleRule::MustNotBeImportedBy { .. } | ModuleRule::MustOnlyBeImportedBy { .. }
    );
    if inbound {
        // The crate root degenerates: every module is within the protected subtree, so no module
        // is an inbound importer and the rule could never react. Fail loud (exit 2) rather than
        // silently pass (PROJECT.md).
        if governed_module == "crate" {
            return Err(match &boundary.rule {
                ModuleRule::MustNotBeImportedBy { .. } => {
                    must_not_be_imported_by_on_crate_error(&boundary.crate_package)
                }
                _ => must_only_be_imported_by_on_crate_error(&boundary.crate_package),
            });
        }
        // `must_not_be_imported_by`: only a module beneath this forbidden importer can offend, so
        // pre-filter before reading the file. `must_only_be_imported_by`: no single pre-filter —
        // every importer of the protected module that is not within the allowlist offends.
        let forbidden_importer = match &boundary.rule {
            ModuleRule::MustNotBeImportedBy { importer } => Some(canonical_module_path(importer)),
            _ => None,
        };
        let allowed_importers: Vec<String> = match &boundary.rule {
            ModuleRule::MustOnlyBeImportedBy { allowed } => {
                allowed.iter().map(|e| canonical_module_path(e)).collect()
            }
            _ => Vec::new(),
        };
        // `governed_files(.., "crate", ..)` yields every reachable (file, file-module) pair —
        // the crate-wide scan, reusing the existing selector with no new scanner.
        let all_files = governed_files(
            &src_dir,
            &files,
            "crate",
            &reachable,
            &inline_only,
            &remapped,
            &remap_shadowed,
            root_relative.as_deref(),
            ScanDepth::Subtree,
        );
        // Collect `(importer module, offending file)` pairs *before* de-duplication: the file
        // is in hand here (the scan reads it to observe the import) but is gone once the list
        // collapses to module identities. The violation count stays per-importer-module; the
        // file is attached to the representative after collapsing, never a de-dup key. The
        // importer is the module that *lexically declares* the `use` — an inline `mod inner { … }`
        // is its own importer, not the file's module — so an inbound edge from an inline submodule
        // is attributed (and pre-filtered / allow-listed) at its true identity, not the file's.
        let mut offenders: Vec<(String, String)> = Vec::new();
        for (file, file_module) in all_files {
            // Fast path: a file whose module is within the protected subtree hosts only
            // self-imports (its inline descendants are within it, hence within the protected
            // module too), never an inbound edge — skip the read.
            if path_within(&file_module, &governed_module) {
                continue;
            }
            // Forbid-one perf pre-filter: the importers a file can carry are its own module and its
            // inline descendants — all within `file_module`'s subtree. So it can host the forbidden
            // importer (or a module beneath it) only when the two subtrees overlap: `file_module`
            // within `forbidden` (the file itself is beneath it), or `forbidden` within `file_module`
            // (an inline descendant could be it). No overlap ⇒ no possible offender; skip the read.
            // The closed-allowlist rule has no single forbidden subtree, so it reads every file.
            if let Some(forbidden) = &forbidden_importer {
                if !(path_within(&file_module, forbidden) || path_within(forbidden, &file_module)) {
                    continue;
                }
            }
            let text = std::fs::read_to_string(&file)
                .map_err(|err| unreadable_governed_file_error(&file, &err.to_string()))?;
            for (importer, import) in imports_with_importers(&text, &file_module, &root_modules) {
                // A module importing from within the protected subtree is not an inbound edge
                // (an inline submodule of the protected module resolves to within it here).
                if path_within(&importer, &governed_module) {
                    continue;
                }
                // Forbid-one: only the forbidden importer (or beneath, `::`-delimited) can violate.
                if let Some(forbidden) = &forbidden_importer {
                    if !path_within(&importer, forbidden) {
                        continue;
                    }
                }
                // This importer must actually import the protected module.
                if !path_within(&import, &governed_module) {
                    continue;
                }
                // Closed allowlist: an importer within any allowed entry (or beneath it) is
                // authorized; every other importer of the protected module offends.
                if forbidden_importer.is_none() {
                    let within_allowed = allowed_importers
                        .iter()
                        .any(|entry| path_within(&importer, entry));
                    if within_allowed {
                        continue;
                    }
                }
                // finding = the importing module path; file = where the offending import sits.
                offenders.push((importer, file.display().to_string()));
            }
        }
        // One violation per offending importer module (the spec's dedup guarantee). A
        // module can be backed by more than one file — a lib+bin package has both
        // `lib.rs` and `main.rs` at module `crate` — so the same importer can appear
        // twice; sort then collapse by the module (the identity), keeping the first file
        // (deterministic after the sort) as the reported `file`. The count is unchanged.
        offenders.sort();
        offenders.dedup_by(|a, b| a.0 == b.0);
        for (importer_module, file) in offenders {
            push_module_violation(
                violations,
                &governed_module,
                rule,
                ModuleFact::ImporterModule(importer_module),
                file,
                boundary,
            );
        }
        return Ok(());
    }

    // External-crate confinement is the one rule that observes *external* imports. It scans
    // every reachable file (like the inbound rules), but a `use <crate>::…` from a module
    // outside the permitted subtree (the governed module's own subtree) offends. The confined
    // crate is the violation *target* — so two confinements of different crates on one module
    // stay injective — and the offending importer module is the finding.
    if let ModuleRule::ConfineExternalCrate { crate_name } = &boundary.rule {
        // Confining to the crate root permits the crate everywhere (its subtree is the whole
        // crate), so the rule could never react. Fail loud (exit 2), never a silent pass.
        if governed_module == "crate" {
            return Err(confine_external_crate_on_crate_error(
                &boundary.crate_package,
            ));
        }
        // Canonicalize the confined crate name into the same vocabulary as the observed external
        // heads: strip a raw-identifier `r#`, and fold a package-name `-` to `_` — Cargo maps a
        // hyphenated package (`windows-sys`) to an underscore import identifier (`windows_sys`),
        // and the scanner only ever sees the identifier (a `use` path cannot contain `-`). Without
        // the fold, confining the hyphenated FFI/platform crates this rule targets would silently
        // never react. A boundary may thus be written with either the package or identifier form.
        let confined = package_name_to_import_ident(&canonical_module_path(crate_name));
        let all_files = governed_files(
            &src_dir,
            &files,
            "crate",
            &reachable,
            &inline_only,
            &remapped,
            &remap_shadowed,
            root_relative.as_deref(),
            ScanDepth::Subtree,
        );
        // `(offending importer module, file)` collected before de-dup, for the same reason as
        // the inbound rule: the file is in hand during the scan but lost once the list
        // collapses to importer identities.
        let mut offenders: Vec<(String, String)> = Vec::new();
        for (file, file_module) in all_files {
            // A file whose module is within the permitted subtree hosts only permitted imports
            // (its inline descendants are within it too) — skip the read.
            if path_within(&file_module, &governed_module) {
                continue;
            }
            let text = std::fs::read_to_string(&file)
                .map_err(|err| unreadable_governed_file_error(&file, &err.to_string()))?;
            for (importer, external) in
                external_imports_with_importers(&text, &file_module, &root_modules)
            {
                // Only the confined crate, imported from outside the permitted subtree.
                if external != confined {
                    continue;
                }
                if path_within(&importer, &governed_module) {
                    continue;
                }
                offenders.push((importer, file.display().to_string()));
            }
        }
        // One violation per offending importer module (the dedup guarantee). The target is the
        // confined crate (`confined`), constant for this boundary; the finding is the importer.
        offenders.sort();
        offenders.dedup_by(|a, b| a.0 == b.0);
        for (importer_module, file) in offenders {
            push_module_violation(
                violations,
                &confined,
                rule,
                ModuleFact::ExternalImporter(importer_module),
                file,
                boundary,
            );
        }
        return Ok(());
    }

    // Inline-symbol-path confinement (layer b): the one rule that observes *calls* (and, under
    // strict, any path mention) inside the governed subtree's bodies — macro bodies included —
    // rather than `use` imports. The confined prefix is the violation *target* (so nested-prefix
    // confinements on one subtree stay injective); the finding is the per-call resolved path (or a
    // hazardous glob) plus its module.
    // Both inline forms (default and strict-external) route through this ONE shared path via the
    // `inline_payload` accessor — never through the exhaustive `is_violation` match below (whose
    // inline arm is `unreachable!()`), which would skip the inline scan and silently observe
    // nothing (a false negative). Identity (`target`/`rule`/`finding`) is byte-identical across the
    // two forms; the only strict-external-conditional behavior is inside `inline_symbol_findings` /
    // `resolve_head`. `external` reflects the single rule's `strict_external` modifier.
    if let Some((prefix, ending_with, strict, external)) = boundary.rule.inline_payload() {
        // Misdeclarations are loud (exit 2), never a silent no-op — for both forms.
        if prefix.trim().is_empty() {
            return Err(inline_empty_prefix_error(&boundary.crate_package));
        }
        if ending_with.is_some() && strict {
            return Err(inline_narrow_and_strict_error(&boundary.crate_package));
        }
        if ending_with.is_some_and(|verbs| verbs.is_empty()) {
            return Err(inline_empty_verbs_error(&boundary.crate_package));
        }
        // Crate-wide files feed the `type`-alias / `pub use` resolution closure; the governed
        // subtree's files are where calls are forbidden.
        let all_files = governed_files(
            &src_dir,
            &files,
            "crate",
            &reachable,
            &inline_only,
            &remapped,
            &remap_shadowed,
            root_relative.as_deref(),
            ScanDepth::Subtree,
        );
        // The rename-aware declared-dependency import identifiers back the strict-external head
        // ladder — read ONLY when the external variant is in play, so the default path reads
        // nothing new (and no `guibiao → hunyi` edge: 圭表's own reader, from the same `package`).
        let dependency_names = if external {
            crate::cargo_metadata::dependency_import_names(package)
        } else {
            Vec::new()
        };
        let confined_prefix = canonical_module_path(prefix);
        let findings = inline_symbol_findings(
            &all_files,
            &governed,
            &root_modules,
            prefix,
            ending_with,
            strict,
            external,
            &dependency_names,
        )?;
        for InlineFinding { fact, file } in findings {
            push_module_violation(violations, &confined_prefix, rule, fact, file, boundary);
        }
        return Ok(());
    }

    // Each outbound rule reduces to one predicate over the governed module's observed
    // internal imports — all `crate::…` (the scanner already filters externals). The
    // file/import loop and the Violation it produces are shared; only the predicate (and,
    // for `RestrictImportsTo`, a crate-root pre-check) differ. Containment is
    // `::`-delimited throughout (exact match OR an `x::` prefix), so a sibling like
    // `crate::types_extra` is never mistaken for being beneath `crate::types`.
    let is_violation: Box<dyn Fn(&str) -> bool> = match &boundary.rule {
        ModuleRule::MustNotImport { module } => {
            let forbidden = canonical_module_path(module);
            Box::new(move |import: &str| path_within(import, &forbidden))
        }
        ModuleRule::RestrictImportsTo { allowed } => {
            // The crate root has no outward internal edge — every import is within its
            // own subtree, so the rule could never react. Fail loud (exit 2) rather than
            // silently pass (PROJECT.md: the one thing the core contract forbids).
            if governed_module == "crate" {
                return Err(restrict_imports_to_on_crate_error(&boundary.crate_package));
            }
            // Canonicalize allowlist entries (raw-id `r#name` -> `name`) like the governed
            // path, so a boundary may be written with either form and still match.
            let allowed: Vec<String> = allowed
                .iter()
                .map(|entry| canonical_module_path(entry))
                .collect();
            let governed_self = governed_module.clone();
            Box::new(move |import: &str| {
                let within_own = path_within(import, &governed_self);
                let within_allowed = allowed.iter().any(|entry| path_within(import, entry));
                // A violation is any outward edge: neither within the module's own subtree
                // nor within an allowlist entry.
                !(within_own || within_allowed)
            })
        }
        ModuleRule::MustNotBeImportedBy { .. }
        | ModuleRule::MustOnlyBeImportedBy { .. }
        | ModuleRule::ConfineExternalCrate { .. }
        | ModuleRule::ConfineInlineSymbolPath { .. } => {
            unreachable!("the inbound / confinement rules are evaluated above and return early")
        }
    };
    // `(finding, offending file)` pairs collected before de-duplication, for the same
    // reason as the inbound rule: the file is in hand during the scan but lost once the
    // list collapses to findings. The count stays per-finding; the file is metadata.
    let mut findings: Vec<(String, String)> = Vec::new();
    for (file, current_module) in governed {
        // A governed file we cannot read is "cannot judge", not "nothing to judge":
        // silently skipping it could hide a real violation. Fail as a scan error
        // (exit 2), never a silent pass.
        let text = std::fs::read_to_string(&file)
            .map_err(|err| unreadable_governed_file_error(&file, &err.to_string()))?;
        for import in imported_module_paths(&text, &current_module, &root_modules) {
            if is_violation(&import) {
                findings.push((import, file.display().to_string()));
            }
        }
    }
    // One violation per distinct finding. The governed module's subtree can span more
    // than one file (a parent and child file, or `lib.rs` + `main.rs` both at `crate`),
    // so the same forbidden import can be found twice; sort then collapse by the finding
    // (the identity), keeping the first file as the reported `file` — the same identity
    // guarantee the inbound rule makes, now for the outbound rules.
    findings.sort();
    findings.dedup_by(|a, b| a.0 == b.0);
    for (finding, file) in findings {
        push_module_violation(
            violations,
            &governed_module,
            rule,
            ModuleFact::ImportedPath(finding),
            file,
            boundary,
        );
    }
    Ok(())
}
