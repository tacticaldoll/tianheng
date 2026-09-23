use std::path::{Path, PathBuf};

use serde_json::Value;
use xuanji::ScanDepth;

use crate::cargo_metadata::{
    compilation_unit_label, crate_root_file, crate_root_files, find_package,
};
use crate::errors::{
    confine_external_crate_on_crate_error, crate_not_found_error, inline_empty_prefix_error,
    inline_empty_verbs_error, inline_module_target_error, inline_narrow_and_strict_error,
    missing_src_error, must_not_be_imported_by_on_crate_error,
    must_only_be_imported_by_on_crate_error, out_of_package_root_error,
    restrict_imports_to_on_crate_error, unknown_module_error, unreadable_governed_file_error,
};
use crate::finding::ModuleFact;
use crate::model::module_rule::Perimeter;
use crate::module_scan::{
    ImportedPath, InlineFinding, canonical_module_path, declaration_text,
    external_imports_with_importers, governed_files, imports_with_importers,
    inline_symbol_findings, package_name_to_import_ident, path_within, reachable_modules,
    rust_files, value_namespace_item_names,
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

/// Compare module identities. An import path must first resolve to its containing module:
/// its item leaf would make a shallow import from the anchor look like a descendant.
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
/// **This function is namespace-blind, and its caller compensates.** Rust resolves `mod foo` and
/// `fn foo` in different namespaces, so both may be declared in one module, and a single `use m::foo;`
/// then binds **both** (verified against rustc, not reasoned: with `mod foo` and `pub fn foo` in `m`, an
/// importer writing that one `use` can call `foo()` *and* reach `foo::INSIDE`). Seeing only the path,
/// this returns the longest reachable module — the module reading. The two readings differ only under
/// `Shallow` on the module's own parent: there, `use m::foo;` meaning the `fn` reaches `m` and must
/// react, while the module reading resolves to the descendant `m::foo` and would not. Under `Subtree`
/// both readings are within `m`, so nothing turns on it.
///
/// That gap used to be a **stated bound** — a recorded false negative — on the grounds that closing it
/// "needs a value-namespace item observation this crate does not have". That premise was wrong: it does
/// have one. [`also_binds_a_value_of_the_governed_module`] consults [`value_namespace_item_names`] and
/// reacts only when the governed module really declares a `fn`/`const`/`static` of that name, which is
/// why it does not become the broad false positive that reacting on both readings would have been — an
/// ordinary `use m::child;` naming only a module still does not react, as `rule-model-surface` requires
/// and `shallow_inbound_rules_protect_only_the_exact_module` pins.
///
/// What remains bounded is the observation, not the resolution: a value declared inside a macro body or
/// arriving through a re-export is not seen, matching every other reader in `module_scan` (macro bodies
/// are stripped before declarations are read). Either directs the reaction toward the module reading
/// alone, which is the pre-existing behaviour rather than a new gap.
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

/// Whether `import_path` ALSO binds a value declared directly in `governed_module`, which
/// [`resolve_import_module`] cannot see because it reads only the path.
///
/// Rust resolves `mod foo` and `fn foo` in different namespaces, so both may be declared in one module
/// and a single `use m::foo;` binds **both** — verified against rustc. The path alone resolves to the
/// longest reachable module, `m::foo`, which under `Shallow` anchored at `m` is only a descendant and
/// does not react; yet the value reading reaches `m` itself and must. That was a recorded false
/// negative, left because closing it "needs a value-namespace item observation guibiao does not have".
/// It does have one: [`value_namespace_item_names`] reads exactly the `fn`/`const`/`static` names a
/// module declares at its own top level, with the true-inline-module and top-level-only disciplines
/// already established for the local-precedence ladder.
///
/// Four conditions must hold together, and each is what keeps this from becoming the broad false
/// positive that reacting on both readings would have been:
///
/// 1. The import **form can bind a value at all** ([`ImportedPath::can_bind_a_value`]). Two forms
///    cannot, and both normalize to a path indistinguishable from a plain import of the same module —
///    which is exactly how each was missed: a **glob** (`use m::foo::*;`, stored at its base module with
///    `::*` stripped) and a **`{self}` leaf** (`use m::foo::{self};`, stored as its prefix module). Each
///    arrives byte-identical to a bare `use m::foo;` — same path, same single-segment leaf, same declared
///    `fn foo`. Neither can bind a value of the *parent*, by the language rather than by likelihood, and
///    the rule lives on `ImportedPath` rather than as conditions here so the next form is one question,
///    not a third ad-hoc test.
/// 2. The whole import path resolved to itself as a module (`import_module == import_path`). With a
///    further segment — `use m::foo::deep::Thing;` — only the *module* `foo` can be meant, since a `fn`
///    has no children, so there is no ambiguity to resolve.
/// 3. The path is `{governed_module}::{leaf}` with `leaf` a single segment. A deeper descendant is
///    reached through the module, never through a value of the governed module.
/// 4. `governed_module` really declares a value named `leaf`. When it declares only `mod leaf`, an
///    ordinary `use m::child;` stays silent — the behaviour
///    `shallow_inbound_rules_protect_only_the_exact_module` pins.
///
/// The outbound family needs no equivalent: it tests `path_within(&import.path, forbidden)` with no
/// depth narrowing on the target side, so both readings lie within the forbidden module and the
/// ambiguity cannot arise.
///
/// Depth is not tested: under `Subtree` both readings already lie within the governed module, so
/// [`within_scan_depth`] has returned true and this is never consulted. Naming `Shallow` here would add
/// a condition the caller's short-circuit already enforces.
fn also_binds_a_value_of_the_governed_module(
    import_path: &str,
    import_module: &str,
    governed_module: &str,
    can_bind_a_value: bool,
    cache: &mut Option<std::collections::HashSet<String>>,
    ctx: &ScanContext<'_>,
) -> Result<bool, String> {
    if !can_bind_a_value {
        return Ok(false);
    }
    if import_module != import_path {
        return Ok(false);
    }
    let Some(leaf) = import_path.strip_prefix(&format!("{governed_module}::")) else {
        return Ok(false);
    };
    if leaf.contains("::") {
        return Ok(false);
    }
    if cache.is_none() {
        *cache = Some(ctx.governed_module_value_items(governed_module)?);
    }
    Ok(cache
        .as_ref()
        .is_some_and(|items| items.contains(&format!("{governed_module}::{leaf}"))))
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
    /// The value-namespace item names `governed_module` declares at its own top level, read from the
    /// files that back that module alone (`Shallow`), not the whole crate: the question is only ever
    /// about the governed module itself. A module can be backed by more than one reachable file (a
    /// `#[path]` remap beside a conventional file, a `cfg_attr` union), so every backing file
    /// contributes, and inline descendants are excluded by the collector's own true-module keying.
    /// Source is cleaned through `declaration_text` before scanning item names.
    fn governed_module_value_items(
        &self,
        governed_module: &str,
    ) -> Result<std::collections::HashSet<String>, String> {
        let mut items = std::collections::HashSet::new();
        for (file, module) in governed_files(
            self.src_dir,
            self.files,
            governed_module,
            self.reachable,
            self.inline_only,
            self.remapped,
            self.remap_shadowed,
            self.root_relative,
            ScanDepth::Shallow,
        ) {
            let raw = std::fs::read_to_string(&file).map_err(|err| {
                crate::errors::unreadable_governed_file_error(&file, &err.to_string())
            })?;
            items.extend(value_namespace_item_names(&module, &declaration_text(&raw)));
        }
        Ok(items)
    }

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
///
/// Governing the crate root inbound is refused as an error (exit 2) because all modules are
/// within its subtree. The importer is the module that lexically declares the `use` (an inline
/// submodule is its own importer). Files within the protected module's subtree host only self-imports
/// and are skipped. Offending modules are deduplicated, keeping the first file deterministically.
fn check_inbound_rule(
    ctx: &ScanContext,
    boundary: &ModuleBoundary,
    governed_module: &str,
    rule: &str,
    violations: &mut Vec<Violation>,
) -> Result<(), String> {
    if governed_module == "crate" {
        return Err(match &boundary.rule {
            ModuleRule::MustNotBeImportedBy { .. } => {
                must_not_be_imported_by_on_crate_error(&boundary.crate_package)
            }
            _ => must_only_be_imported_by_on_crate_error(&boundary.crate_package),
        });
    }
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
    let mut offenders: Vec<(String, String)> = Vec::new();
    let mut governed_value_items: Option<std::collections::HashSet<String>> = None;
    for (file, file_module) in ctx.all_files() {
        if is_inside_protected_module(&file_module, governed_module) {
            continue;
        }
        if let Some(forbidden) = &forbidden_importer {
            if !(path_within(&file_module, forbidden) || path_within(forbidden, &file_module)) {
                continue;
            }
        }
        let text = std::fs::read_to_string(&file)
            .map_err(|err| unreadable_governed_file_error(&file, &err.to_string()))?;
        for (importer, import) in imports_with_importers(&text, &file_module, ctx.root_modules)? {
            if is_inside_protected_module(&importer, governed_module) {
                continue;
            }
            if let Some(forbidden) = &forbidden_importer {
                if !path_within(&importer, forbidden) {
                    continue;
                }
            }
            let import_module = resolve_import_module(&import.path, ctx.reachable);
            let imports_protected =
                within_scan_depth(import_module, governed_module, boundary.depth)
                    || (import.is_glob && path_within(governed_module, &import.path))
                    || also_binds_a_value_of_the_governed_module(
                        &import.path,
                        import_module,
                        governed_module,
                        import.can_bind_a_value(),
                        &mut governed_value_items,
                        ctx,
                    )?;
            if !imports_protected {
                continue;
            }
            if forbidden_importer.is_none() {
                let within_allowed = allowed_importers
                    .iter()
                    .any(|entry| path_within(&importer, entry));
                if within_allowed {
                    continue;
                }
            }
            offenders.push((importer, file.display().to_string()));
        }
    }
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
///
/// Confining on the crate root is refused as an error (exit 2). Confined crate names fold hyphens
/// to underscores to match import identifiers. Files whose module is within the permitted subtree
/// host only permitted imports and are skipped. Offending importer modules are deduplicated.
fn check_external_confinement(
    ctx: &ScanContext,
    boundary: &ModuleBoundary,
    governed_module: &str,
    rule: &str,
    crate_name: &str,
    violations: &mut Vec<Violation>,
) -> Result<(), String> {
    if governed_module == "crate" {
        return Err(confine_external_crate_on_crate_error(
            &boundary.crate_package,
        ));
    }
    let confined = package_name_to_import_ident(&canonical_module_path(crate_name));
    let mut offenders: Vec<(String, String)> = Vec::new();
    for (file, file_module) in ctx.all_files() {
        if hosts_only_permitted_importers(&file_module, governed_module, boundary.depth) {
            continue;
        }
        let text = std::fs::read_to_string(&file)
            .map_err(|err| unreadable_governed_file_error(&file, &err.to_string()))?;
        for (importer, external) in
            external_imports_with_importers(&text, &file_module, ctx.root_modules)?
        {
            if external != confined {
                continue;
            }
            if within_scan_depth(&importer, governed_module, boundary.depth) {
                continue;
            }
            offenders.push((importer, file.display().to_string()));
        }
    }
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
///
/// Empty prefix, conflicting narrow-and-strict, or empty verbs misdeclarations fail loud (exit 2).
/// Crate-wide files feed type alias and pub use resolution; dependency names are read on demand
/// when external confinement is active.
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
    if prefix.trim().is_empty() {
        return Err(inline_empty_prefix_error(&boundary.crate_package));
    }
    if ending_with.is_some() && strict {
        return Err(inline_narrow_and_strict_error(&boundary.crate_package));
    }
    if ending_with.is_some_and(|verbs| verbs.is_empty()) {
        return Err(inline_empty_verbs_error(&boundary.crate_package));
    }
    let all_files = ctx.all_files();
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
///
/// Restricting imports on crate root is refused as an error (exit 2). Allowlist entries are
/// canonicalized. An unreadable file fails as a scan error (exit 2). Violations are deduplicated
/// per distinct (importing module, import path).
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
            if governed_module == "crate" {
                return Err(restrict_imports_to_on_crate_error(&boundary.crate_package));
            }
            let allowed: Vec<String> = allowed
                .iter()
                .map(|entry| canonical_module_path(entry))
                .collect();
            let governed_self = governed_module.to_string();
            Box::new(move |import: &ImportedPath| {
                let within_own = path_within(&import.path, &governed_self);
                let within_allowed = allowed.iter().any(|entry| path_within(&import.path, entry));
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
    let mut findings: Vec<(String, String, String)> = Vec::new();
    for (file, current_module) in governed {
        let text = std::fs::read_to_string(&file)
            .map_err(|err| unreadable_governed_file_error(&file, &err.to_string()))?;
        for (importer, import) in imports_with_importers(&text, &current_module, ctx.root_modules)?
        {
            if is_violation(&import) {
                findings.push((importer, import.path, file.display().to_string()));
            }
        }
    }
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
///
/// If metadata reports no target at all, the conventional source directory fallback is used.
/// Only missing-module errors are deferred; an inline target, unreadable files and other scan failures
/// propagate immediately.
pub(crate) fn check_module_boundary(
    metadata: &Value,
    boundary: &ModuleBoundary,
    violations: &mut Vec<Violation>,
) -> Result<(), String> {
    let package = find_package(metadata, &boundary.crate_package)
        .ok_or_else(|| crate_not_found_error(&boundary.crate_package))?;
    let roots = crate_root_files(package);
    if roots.is_empty() {
        let mut found = Vec::new();
        return match check_one_root(package, None, None, boundary, &mut found)? {
            RootOutcome::Governed => {
                violations.append(&mut found);
                Ok(())
            }
            RootOutcome::ModuleAbsent(reason) => Err(reason),
        };
    }
    let mut deferred: Option<String> = None;
    let mut governed_somewhere = false;
    let mut found = Vec::new();
    for root in &roots {
        match check_one_root(
            package,
            Some(root.as_path()),
            Some(&roots),
            boundary,
            &mut found,
        )? {
            RootOutcome::Governed => governed_somewhere = true,
            RootOutcome::ModuleAbsent(reason) => {
                if deferred.is_none() {
                    deferred = Some(reason);
                }
            }
        }
    }
    match deferred {
        Some(reason) if !governed_somewhere => Err(reason),
        _ => {
            violations.append(&mut found);
            Ok(())
        }
    }
}

/// Whether one root hosted the governed module. Distinguishing absence from failure is what keeps a real
/// scan error from being deferred away by a sibling root — see the loop above.
enum RootOutcome {
    Governed,
    ModuleAbsent(String),
}

/// Check one compilation unit root. Custom target roots relative to `src_dir` map to `crate`.
/// Roots outside the package manifest directory error as configuration errors.
/// Sibling compilation unit roots are excluded from module discovery to prevent duplicate violations.
/// Inline modules own no source file and cannot be governed targets (exit 2). An inline target is
/// present in its root rather than absent from it, so that refusal is returned at once: deferring it,
/// a sibling root backing the same path with a file would absorb it and leave the inline body
/// unobserved behind a clean report.
///
/// A root without the governed module reports [`RootOutcome::ModuleAbsent`]. Whether it is judged
/// first is the rule's [`Perimeter`]: under `GovernedModule` it holds nothing the rule governs and is
/// not; under `WholeRoot` it is judged through the same dispatch as a governed root, with an empty
/// permitted region. The caller keeps those findings only when some root is governed, so a package where
/// no root has the module is still a constitution error.
fn check_one_root(
    package: &Value,
    root_file: Option<&Path>,
    sibling_roots: Option<&[PathBuf]>,
    boundary: &ModuleBoundary,
    violations: &mut Vec<Violation>,
) -> Result<RootOutcome, String> {
    let src_dir = match root_file.and_then(Path::parent) {
        Some(dir) => dir.to_path_buf(),
        None => {
            package_src_dir(package).ok_or_else(|| missing_src_error(&boundary.crate_package))?
        }
    };

    let root_relative = root_file
        .and_then(|rf| rf.strip_prefix(&src_dir).ok())
        .map(|p| p.to_path_buf());
    let unit_owned = match root_file {
        Some(rf) => Some(
            compilation_unit_label(package, rf)
                .ok_or_else(|| out_of_package_root_error(&boundary.crate_package, rf))?,
        ),
        None => None,
    };
    let unit: &str = unit_owned.as_deref().unwrap_or("src");
    let mut files = rust_files(&src_dir)?;
    if let Some(siblings) = sibling_roots {
        files.retain(|f| root_file.is_some_and(|r| r == f.as_path()) || !siblings.contains(f));
    }
    let (reachable, inline_only, remapped, remap_shadowed) =
        reachable_modules(&src_dir, &files, root_relative.as_deref())?;
    let root_modules: Vec<String> = reachable
        .iter()
        .filter_map(|module| {
            module
                .strip_prefix("crate::")
                .filter(|rest| !rest.contains("::"))
                .map(str::to_string)
        })
        .collect();
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

    if governed.is_empty() && inline_only.contains(&governed_module) {
        let leaf = governed_module
            .rsplit_once("::")
            .map_or(governed_module.as_str(), |(_, leaf)| leaf);
        return Err(inline_module_target_error(
            &boundary.module,
            &boundary.crate_package,
            leaf,
        ));
    }
    let outcome = if governed.is_empty() {
        RootOutcome::ModuleAbsent(unknown_module_error(
            &boundary.module,
            &boundary.crate_package,
        ))
    } else {
        RootOutcome::Governed
    };
    if governed.is_empty() && boundary.rule.perimeter() == Perimeter::GovernedModule {
        return Ok(outcome);
    }

    let inbound = matches!(
        &boundary.rule,
        ModuleRule::MustNotBeImportedBy { .. } | ModuleRule::MustOnlyBeImportedBy { .. }
    );
    if inbound {
        check_inbound_rule(&ctx, boundary, &governed_module, rule, violations)?;
        return Ok(outcome);
    }
    if let ModuleRule::ConfineExternalCrate { crate_name } = &boundary.rule {
        check_external_confinement(
            &ctx,
            boundary,
            &governed_module,
            rule,
            crate_name,
            violations,
        )?;
        return Ok(outcome);
    }
    if let Some((prefix, ending_with, strict, external)) = boundary.rule.inline_payload() {
        check_inline_confinement(
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
        )?;
        return Ok(outcome);
    }
    check_outbound_rule(&ctx, boundary, &governed_module, governed, rule, violations)?;
    Ok(outcome)
}
