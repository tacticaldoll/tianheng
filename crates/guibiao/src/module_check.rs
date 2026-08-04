use std::path::{Path, PathBuf};

use serde_json::Value;
use xuanji::ScanDepth;

use crate::cargo_metadata::{crate_root_file, crate_root_files, find_package};
use crate::errors::{
    confine_external_crate_on_crate_error, crate_not_found_error, inline_empty_prefix_error,
    inline_empty_verbs_error, inline_module_target_error, inline_narrow_and_strict_error,
    missing_src_error, must_not_be_imported_by_on_crate_error,
    must_only_be_imported_by_on_crate_error, restrict_imports_to_on_crate_error,
    unknown_module_error, unreadable_governed_file_error,
};
use crate::finding::ModuleFact;
use crate::module_scan::{
    ImportedPath, InlineFinding, canonical_module_path, external_imports_with_importers,
    governed_files, imported_module_paths, imports_with_importers, inline_symbol_findings,
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

#[allow(clippy::too_many_arguments)]
fn push_module_violation(
    violations: &mut Vec<Violation>,
    target: &str,
    rule: &str,
    fact: ModuleFact,
    file: String,
    boundary: &ModuleBoundary,
    unit: &str,
) {
    let finding = fact.into_finding(&boundary.crate_package, unit);
    violations.push(
        Violation::new(
            BoundaryKind::Module,
            ViolationId::new(target, boundary.rule_key(), finding.fact().clone()),
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

fn within_scan_depth(candidate: &str, anchor: &str, depth: ScanDepth) -> bool {
    if depth == ScanDepth::Shallow {
        candidate == anchor
    } else {
        path_within(candidate, anchor)
    }
}

/// The inbound rules' self-import exemption, in ONE place: a module within the protected module's
/// own subtree is never an inbound importer of it. Deliberately **depth-free** — `ScanDepth` narrows
/// what counts as *reaching* the protected module, never who counts as *inside* it
/// (`rule-model-surface`, and the same distinction outbound's `RestrictImportsTo` already draws).
///
/// Called both as the file-level fast path (every importer a file can host is within its own
/// module's subtree, so a file inside the protected subtree cannot host an inbound edge — skip the
/// read) and as the per-import exemption. One predicate, two call sites, so the pre-filter and the
/// real rule cannot drift: a depth-gated fast path over a depth-free exemption left the same
/// violations but read files the exemption would have excused, and an unreadable or
/// nest-cap-exceeding one then turned a `Shallow` inbound rule into exit 2 where `Subtree` exits 0.
fn is_inside_protected_module(module: &str, governed_module: &str) -> bool {
    path_within(module, governed_module)
}

/// External confinement's file-level fast path — a different contract from the inbound exemption
/// above, which is why it is a separate function rather than one shared helper. Skip the read only
/// when EVERY importer the file can host is permitted. The importers a file can host are its own
/// module and its inline descendants, all within `file_module`'s subtree, so that holds exactly when
/// the whole subtree is permitted: under `Subtree` iff the file is within the permitted module;
/// under `Shallow` never, because the permitted set is the anchored module alone and an inline
/// `mod inner { … }` inside the permitted file is itself outside it — skipping there would silently
/// drop a real confinement violation.
fn hosts_only_permitted_importers(
    file_module: &str,
    governed_module: &str,
    depth: ScanDepth,
) -> bool {
    depth == ScanDepth::Subtree && path_within(file_module, governed_module)
}

/// Resolves an import path to the module it actually denotes: itself when the whole path is a
/// reachable module (a bare module import), otherwise its longest reachable prefix — an item-form
/// import, whose tail names an item living in that module. `crate::internal::Secret` (an item in
/// `crate::internal`) resolves to `crate::internal`; `crate::internal::deep::Thing` (an item in
/// the descendant module `deep`) resolves to `crate::internal::deep` — the two stay
/// distinguishable, which a lexical-only (string) comparison against `governed_module` cannot do.
/// Falls back to the full path when no prefix is reachable (a path through a module
/// `reachable_modules` cannot model, e.g. produced by a non-`cfg_if!` macro) rather than guessing.
///
/// **Stated bound — this resolution is namespace-blind.** Rust resolves `mod foo` and `fn foo` in
/// different namespaces, so both may be declared in one module, and a single `use m::foo;` then binds
/// **both** (verified against rustc, not reasoned: with `mod foo` and `pub fn foo` in `m`, an
/// importer writing that one `use` can call `foo()` *and* reach `foo::INSIDE`). This function sees
/// only the path, so it returns the longest reachable module — the module reading — and the value
/// reading is unobserved. The two readings differ only under `Shallow` on the module's own parent:
/// there, `use m::foo;` meaning the `fn` reaches `m` and should react, while the module reading
/// resolves to the descendant `m::foo` and does not. Under `Subtree` both readings are within `m`,
/// so nothing is lost.
///
/// Deliberately not "fixed" by reacting on both readings: that would make every ordinary bare import
/// of a child module (`use m::child;`) react under `Shallow`, contradicting
/// `rule-model-surface`'s own scenario that an importer of only a descendant module does not violate
/// an exact-seam boundary — trading a narrow, exotic false negative for a broad false positive.
/// Distinguishing the two needs a value-namespace item observation this crate does not have (and one
/// that would carry its own bounds: a macro-generated or re-exported `fn`). Recorded in `BACKLOG.md`
/// with that promotion trigger, stated in `rule-model-surface`, and pinned by
/// `shallow_inbound_target_match_is_namespace_blind_a_stated_bound` so the bound cannot change state
/// unnoticed — the same way this crate states its other observation bounds rather than implying them.
fn resolve_import_module<'a>(
    import_path: &'a str,
    reachable: &std::collections::BTreeSet<String>,
) -> &'a str {
    let mut candidate = import_path;
    loop {
        if reachable.contains(candidate) {
            return candidate;
        }
        match candidate.rsplit_once("::") {
            Some((prefix, _)) => candidate = prefix,
            None => return import_path,
        }
    }
}

/// The crate-wide scan state every rule family below reads from — resolved once in
/// [`check_module_boundary`], then shared read-only across whichever family actually evaluates.
struct ScanContext<'a> {
    /// The compilation unit these observations came from — see `ModuleFact::into_finding`.
    unit: &'a str,
    src_dir: &'a Path,
    files: &'a [PathBuf],
    root_relative: Option<&'a Path>,
    reachable: &'a std::collections::BTreeSet<String>,
    inline_only: &'a std::collections::BTreeSet<String>,
    remapped: &'a [(PathBuf, String)],
    remap_shadowed: &'a std::collections::BTreeSet<String>,
    root_modules: &'a [String],
}

impl ScanContext<'_> {
    /// `governed_files(.., "crate", ScanDepth::Subtree)` reused by every rule family that scans
    /// the whole crate rather than just the governed subtree (inbound, external confinement,
    /// inline confinement) — the identical crate-wide selector, no new scanner.
    fn all_files(&self) -> Vec<(PathBuf, String)> {
        governed_files(
            self.src_dir,
            self.files,
            "crate",
            self.reachable,
            self.inline_only,
            self.remapped,
            self.remap_shadowed,
            self.root_relative,
            ScanDepth::Subtree,
        )
    }
}

/// The inbound rules invert the scope: they scan every reachable file and test each importing
/// *module* (not an import path) against the rule, so they have their own evaluation rather than
/// the shared import-path predicate the outbound rules use. `must_not_be_imported_by` reacts to
/// an importer beneath a forbidden importer; the closed dual `must_only_be_imported_by` reacts to
/// any importer NOT within the allowlist.
fn check_inbound_rule(
    ctx: &ScanContext,
    boundary: &ModuleBoundary,
    governed_module: &str,
    rule: &str,
    violations: &mut Vec<Violation>,
) -> Result<(), String> {
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
    // Collect `(importer module, offending file)` pairs *before* de-duplication: the file
    // is in hand here (the scan reads it to observe the import) but is gone once the list
    // collapses to module identities. The violation count stays per-importer-module; the
    // file is attached to the representative after collapsing, never a de-dup key. The
    // importer is the module that *lexically declares* the `use` — an inline `mod inner { … }`
    // is its own importer, not the file's module — so an inbound edge from an inline submodule
    // is attributed (and pre-filtered / allow-listed) at its true identity, not the file's.
    let mut offenders: Vec<(String, String)> = Vec::new();
    for (file, file_module) in ctx.all_files() {
        // Fast path: a file whose module is within the protected subtree hosts only
        // self-imports (its inline descendants are within it, hence within the protected
        // module too), never an inbound edge — skip the read. Same predicate as the
        // per-import exemption below, at every depth.
        if is_inside_protected_module(&file_module, governed_module) {
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
        for (importer, import) in imports_with_importers(&text, &file_module, ctx.root_modules)? {
            // A module importing from within the protected subtree is not an inbound edge
            // (an inline submodule of the protected module resolves to within it here) — the
            // same depth-free predicate the file-level fast path above uses, so the two cannot
            // disagree about who is inside the protected module.
            if is_inside_protected_module(&importer, governed_module) {
                continue;
            }
            // Forbid-one: only the forbidden importer (or beneath, `::`-delimited) can violate.
            if let Some(forbidden) = &forbidden_importer {
                if !path_within(&importer, forbidden) {
                    continue;
                }
            }
            // This importer must actually import the protected module (either directly,
            // via descendant path, or via an ancestor glob wildcard import). The import is
            // resolved to the module it denotes before the depth-gated comparison: an
            // item-form import's own path string includes the item leaf, which `within_scan_depth`
            // must never compare directly (an item in the anchored module and an item in a
            // descendant module would otherwise be lexically indistinguishable under `Shallow`).
            let import_module = resolve_import_module(&import.path, ctx.reachable);
            let imports_protected =
                within_scan_depth(import_module, governed_module, boundary.depth)
                    || (import.is_glob && path_within(governed_module, &import.path));
            if !imports_protected {
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
    // One violation per offending importer module (the spec's dedup guarantee). A module can be
    // backed by more than one REACHABLE file, so the same importer can appear twice: a `#[path]`
    // remap and a conventional file of the same name are additive and cfg-blind, and a
    // `cfg_attr(path)` union descends several candidate bases for one inline body. NOT because a
    // lib+bin package's two roots share module `crate` — they do not: only the ONE resolved crate
    // root and the modules reachable from it are governed, and a `main.rs` beside a `lib.rs` is not
    // observed at all (a stated bound — see `module-boundary`'s single-governed-root requirement).
    // Sort then collapse by the module (the identity), keeping the first file (deterministic after
    // the sort) as the reported `file`. The count is unchanged.
    offenders.sort();
    offenders.dedup_by(|a, b| a.0 == b.0);
    for (importer_module, file) in offenders {
        push_module_violation(
            violations,
            governed_module,
            rule,
            ModuleFact::ImporterModule(importer_module),
            file,
            boundary,
            ctx.unit,
        );
    }
    Ok(())
}

/// External-crate confinement is the one rule that observes *external* imports. It scans every
/// reachable file (like the inbound rules), but a `use <crate>::…` from a module outside the
/// permitted subtree (the governed module's own subtree) offends. The confined crate is the
/// violation *target* — so two confinements of different crates on one module stay injective —
/// and the offending importer module is the finding.
fn check_external_confinement(
    ctx: &ScanContext,
    boundary: &ModuleBoundary,
    governed_module: &str,
    rule: &str,
    crate_name: &str,
    violations: &mut Vec<Violation>,
) -> Result<(), String> {
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
    // `(offending importer module, file)` collected before de-dup, for the same reason as
    // the inbound rule: the file is in hand during the scan but lost once the list
    // collapses to importer identities.
    let mut offenders: Vec<(String, String)> = Vec::new();
    for (file, file_module) in ctx.all_files() {
        // A file whose module is within the permitted subtree hosts only permitted imports
        // (its inline descendants are within it too) — skip the read. Depth-gated here, unlike
        // the inbound exemption: see `hosts_only_permitted_importers`.
        if hosts_only_permitted_importers(&file_module, governed_module, boundary.depth) {
            continue;
        }
        let text = std::fs::read_to_string(&file)
            .map_err(|err| unreadable_governed_file_error(&file, &err.to_string()))?;
        for (importer, external) in
            external_imports_with_importers(&text, &file_module, ctx.root_modules)?
        {
            // Only the confined crate, imported from outside the permitted subtree.
            if external != confined {
                continue;
            }
            if within_scan_depth(&importer, governed_module, boundary.depth) {
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
            ctx.unit,
        );
    }
    Ok(())
}

/// Inline-symbol-path confinement (layer b): the one rule that observes *calls* (and, under
/// strict, any path mention) inside the governed subtree's bodies — macro bodies included —
/// rather than `use` imports. The confined prefix is the violation *target* (so nested-prefix
/// confinements on one subtree stay injective); the finding is the per-call resolved path (or a
/// hazardous glob) plus its module.
/// Both inline forms (default and strict-external) route through this ONE shared path via the
/// `inline_payload` accessor — never through the exhaustive `is_violation` match in
/// [`check_outbound_rule`] (whose inline arm is `unreachable!()`), which would skip the inline
/// scan and silently observe nothing (a false negative). Identity (`target`/`rule`/`finding`) is
/// byte-identical across the two forms; the only strict-external-conditional behavior is inside
/// `inline_symbol_findings` / `resolve_head`. `external` reflects the single rule's
/// `strict_external` modifier.
#[allow(clippy::too_many_arguments)]
fn check_inline_confinement(
    ctx: &ScanContext,
    boundary: &ModuleBoundary,
    package: &Value,
    governed: &[(PathBuf, String)],
    rule: &str,
    prefix: &str,
    ending_with: Option<&[String]>,
    strict: bool,
    external: bool,
    violations: &mut Vec<Violation>,
) -> Result<(), String> {
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
    let all_files = ctx.all_files();
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
        governed,
        ctx.root_modules,
        prefix,
        ending_with,
        strict,
        external,
        &dependency_names,
    )?;
    for InlineFinding { fact, file } in findings {
        push_module_violation(
            violations,
            &confined_prefix,
            rule,
            fact,
            file,
            boundary,
            ctx.unit,
        );
    }
    Ok(())
}

/// Each outbound rule reduces to one predicate over the governed module's observed internal
/// imports — all `crate::…` (the scanner already filters externals). The file/import loop and
/// the `Violation` it produces are shared; only the predicate (and, for `RestrictImportsTo`, a
/// crate-root pre-check) differ. Containment is `::`-delimited throughout (exact match OR an
/// `x::` prefix), so a sibling like `crate::types_extra` is never mistaken for being beneath
/// `crate::types`.
fn check_outbound_rule(
    ctx: &ScanContext,
    boundary: &ModuleBoundary,
    governed_module: &str,
    governed: Vec<(PathBuf, String)>,
    rule: &str,
    violations: &mut Vec<Violation>,
) -> Result<(), String> {
    let is_violation: Box<dyn Fn(&ImportedPath) -> bool> = match &boundary.rule {
        ModuleRule::MustNotImport { module } => {
            let forbidden = canonical_module_path(module);
            Box::new(move |import: &ImportedPath| {
                path_within(&import.path, &forbidden)
                    || (import.is_glob && path_within(&forbidden, &import.path))
            })
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
            let governed_self = governed_module.to_string();
            Box::new(move |import: &ImportedPath| {
                let within_own = path_within(&import.path, &governed_self);
                let within_allowed = allowed.iter().any(|entry| path_within(&import.path, entry));
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
    // `(importing module, import path, offending file)`. The importer is the module that
    // **lexically declares** the `use` — an inline `mod inner { … }` is its own importer, not the
    // file's — read through the same accessor the inbound rules use so the two families agree on who
    // imported something. The file is collected before de-duplication for the same reason as the
    // inbound rule: it is in hand during the scan but gone once the list collapses.
    let mut findings: Vec<(String, String, String)> = Vec::new();
    for (file, current_module) in governed {
        // A governed file we cannot read is "cannot judge", not "nothing to judge":
        // silently skipping it could hide a real violation. Fail as a scan error
        // (exit 2), never a silent pass.
        let text = std::fs::read_to_string(&file)
            .map_err(|err| unreadable_governed_file_error(&file, &err.to_string()))?;
        for (importer, import) in imports_with_importers(&text, &current_module, ctx.root_modules)?
        {
            if is_violation(&import) {
                findings.push((importer, import.path, file.display().to_string()));
            }
        }
    }
    // One violation per distinct (importing module, import path). A single module can be backed by
    // more than one reachable source, so the same import can be found twice for one importer; that
    // pair collapses, and the first file after the sort is the reported one.
    //
    // The importing module is part of the identity, not only the report: without it, two DIFFERENT
    // modules of the governed subtree importing the same forbidden path collapse to one finding, so
    // baselining the first silently masks the second — a real drift event the tool exists to catch.
    // The dedup's own stated reason was only ever "one module backed by two files", which is narrower
    // than collapsing distinct modules, and the inbound rules have always qualified by importer. This
    // makes the two families symmetric.
    findings.sort();
    findings.dedup_by(|a, b| a.0 == b.0 && a.1 == b.1);
    for (importer, path, file) in findings {
        push_module_violation(
            violations,
            governed_module,
            rule,
            ModuleFact::ImportedPath { path, importer },
            file,
            boundary,
            ctx.unit,
        );
    }
    Ok(())
}

/// Evaluate a module boundary against **every** compiled root of its package.
///
/// A package's roots are separate compilation units: each denotes the module path `crate`, and neither's
/// declarations, inline shadowing, nor `#[path]` remaps belong in the other's graph — so each is resolved
/// as its own corpus and the results are merged. Governing only the first root left a violation written
/// in a `bin` beside a library unobserved, which is the false negative this composition closes.
///
/// The unknown-module error is deliberately deferred to the end: a module legitimately exists in one
/// root's graph and not another's (a library's internals are not the binary's), so erroring per root
/// would make a boundary on a library-only module exit 2 for the package's `bin` root — refusing to judge
/// source that compiles. It fires only when NO root has the module, and then reports the first root's own
/// reason so the message still names a real expected location.
pub(crate) fn check_module_boundary(
    metadata: &Value,
    boundary: &ModuleBoundary,
    violations: &mut Vec<Violation>,
) -> Result<(), String> {
    let package = find_package(metadata, &boundary.crate_package)
        .ok_or_else(|| crate_not_found_error(&boundary.crate_package))?;
    let roots = crate_root_files(package);
    if roots.is_empty() {
        // Metadata reporting no target at all is the shape synthetic metadata in a caller's own tests
        // carries; the conventional-source-directory fallback below is load-bearing for it, and its
        // reachability walk already treats every conventional top-level root as a root.
        return check_one_root(package, None, None, boundary, violations);
    }
    let mut deferred: Option<String> = None;
    let mut governed_somewhere = false;
    for root in &roots {
        let mut per_root = Vec::new();
        match check_one_root(
            package,
            Some(root.as_path()),
            Some(&roots),
            boundary,
            &mut per_root,
        ) {
            Ok(()) => {
                governed_somewhere = true;
                violations.append(&mut per_root);
            }
            Err(reason) => {
                // A root that cannot host this module is not yet an error — another root may.
                if deferred.is_none() {
                    deferred = Some(reason);
                }
            }
        }
    }
    match deferred {
        Some(reason) if !governed_somewhere => Err(reason),
        _ => Ok(()),
    }
}

fn check_one_root(
    package: &Value,
    root_file: Option<&Path>,
    sibling_roots: Option<&[PathBuf]>,
    boundary: &ModuleBoundary,
    violations: &mut Vec<Violation>,
) -> Result<(), String> {
    let src_dir = match root_file.and_then(Path::parent) {
        Some(dir) => dir.to_path_buf(),
        None => {
            package_src_dir(package).ok_or_else(|| missing_src_error(&boundary.crate_package))?
        }
    };

    // The root file relative to `src_dir` — usually `lib.rs`/`main.rs`, but Cargo permits a custom
    // target root (`[lib] path = "src/core.rs"`), which must still map to `crate`. `None` keeps the
    // conventional-root behaviour the target-less fallback depends on.
    let root_relative = root_file
        .and_then(|rf| rf.strip_prefix(&src_dir).ok())
        .map(|p| p.to_path_buf());
    // The compilation unit's identity role: the root's path relative to the package's manifest
    // directory. Both sides come from `cargo metadata`'s own JSON strings, so neither can be non-UTF-8
    // and no lossy conversion is involved; a root outside the manifest directory keeps its path as
    // given. With no target at all (synthetic metadata), the conventional source directory the fallback
    // assumes IS the unit.
    let manifest_dir = package["manifest_path"]
        .as_str()
        .map(Path::new)
        .and_then(Path::parent);
    let unit: &str = match (root_file, manifest_dir) {
        (Some(rf), Some(dir)) => rf
            .strip_prefix(dir)
            .ok()
            .and_then(Path::to_str)
            .or_else(|| rf.to_str())
            .unwrap_or("src"),
        (Some(rf), None) => rf.to_str().unwrap_or("src"),
        (None, _) => "src",
    };
    let mut files = rust_files(&src_dir)?;
    // A SIBLING root is not a module of this unit — it is another compilation unit. Without this the
    // conventional-root rule (a top-level `lib.rs`/`main.rs` is segment-less, hence `crate`) makes every
    // sibling root map to `crate` in *this* root's walk too, so one violation is reported once per root:
    // a duplicate, and a worse defect than the false negative the per-root corpus closes. A root
    // declared as a module by another root (`mod main;`) is a stated bound of that exclusion — it is a
    // dual-role file rustc compiles twice, and this unit's corpus keeps the unit's own root only.
    if let Some(siblings) = sibling_roots {
        files.retain(|f| root_file.is_some_and(|r| r == f.as_path()) || !siblings.contains(f));
    }
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
    let ctx = ScanContext {
        unit,
        src_dir: &src_dir,
        files: &files,
        root_relative: root_relative.as_deref(),
        reachable: &reachable,
        inline_only: &inline_only,
        remapped: &remapped,
        remap_shadowed: &remap_shadowed,
        root_modules: &root_modules,
    };

    let inbound = matches!(
        &boundary.rule,
        ModuleRule::MustNotBeImportedBy { .. } | ModuleRule::MustOnlyBeImportedBy { .. }
    );
    if inbound {
        return check_inbound_rule(&ctx, boundary, &governed_module, rule, violations);
    }
    if let ModuleRule::ConfineExternalCrate { crate_name } = &boundary.rule {
        return check_external_confinement(
            &ctx,
            boundary,
            &governed_module,
            rule,
            crate_name,
            violations,
        );
    }
    if let Some((prefix, ending_with, strict, external)) = boundary.rule.inline_payload() {
        return check_inline_confinement(
            &ctx,
            boundary,
            package,
            &governed,
            rule,
            prefix,
            ending_with,
            strict,
            external,
            violations,
        );
    }
    check_outbound_rule(&ctx, boundary, &governed_module, governed, rule, violations)
}
