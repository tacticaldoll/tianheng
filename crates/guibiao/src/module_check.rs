use std::path::{Path, PathBuf};

use serde_json::Value;
use xingbiao::CrateRoots;
use xuanji::ScanDepth;

use crate::cargo_metadata::{compilation_unit_label, crate_roots, find_package};
use crate::errors::{
    confine_external_crate_on_crate_error, confine_inline_call_on_crate_error,
    confine_inline_call_shallow_error, crate_not_found_error, inline_empty_prefix_error,
    inline_empty_verbs_error, inline_module_target_error, inline_narrow_and_strict_error,
    must_not_be_imported_by_on_crate_error, must_only_be_imported_by_on_crate_error,
    no_compiled_root_error, non_canonical_inline_prefix_error, non_canonical_module_path_error,
    restrict_imports_to_on_crate_error, unknown_allowed_module_error,
    unknown_forbidden_module_error, unknown_inline_prefix_error, unknown_inline_prefix_head_error,
    unknown_module_error,
};
use crate::finding::ModuleFact;
use crate::model::module_rule::Perimeter;
use crate::module_scan::{
    EvaluationScans, ImportedPath, InlineFinding, PrefixRoot, RootScan, SymbolPrefix,
    canonical_module_path, canonical_module_spelling, canonical_symbol_path_spelling,
    governed_files, package_name_to_import_ident, path_within, sysroot_crate,
};
use crate::{BoundaryKind, ModuleBoundary, ModuleRule, Violation, ViolationId};

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
/// have one. [`also_binds_a_value_of_the_governed_module`] consults
/// [`crate::module_scan::UnitScan::value_items`] and
/// reacts only when the governed module really declares a `fn`/`const`/`static` of that name, which is
/// why it does not become the broad false positive that reacting on both readings would have been — an
/// ordinary `use m::child;` naming only a module still does not react, as `rule-model-surface` requires
/// and `shallow_inbound_rules_protect_only_the_exact_module` pins.
///
/// What remains bounded is the observation, not the resolution: a value declared inside a macro body or
/// arriving through a re-export is not seen, matching every other reader in `module_scan` (no
/// declaration inside a macro's group is recorded). Either directs the reaction toward the module reading
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
/// It does have one: [`crate::module_scan::UnitScan::value_items`] reads exactly the
/// `fn`/`const`/`static` names a
/// module declares at its own top level, with the true-inline-module and top-level-only disciplines
/// already established for the prefix existence check's item set.
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
    root: &RootScan,
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
        *cache = Some(root.governed_module_value_items(governed_module)?);
    }
    Ok(cache
        .as_ref()
        .is_some_and(|items| items.contains(&format!("{governed_module}::{leaf}"))))
}

/// `written` in its canonical module spelling, or the constitution error naming it and the spelling
/// it most plausibly meant.
fn canonical_spelling_or_error(written: &str, crate_package: &str) -> Result<String, String> {
    canonical_module_spelling(written).map_err(|suggestion| {
        non_canonical_module_path_error(written, crate_package, suggestion.as_deref())
    })
}

/// The modules a rule names besides the governed module, each in its canonical spelling, with the
/// builder that names them and whether they are an allowlist. Every module path a boundary carries
/// passes through here or through the governed module's own check in [`check_module_boundary`], so
/// none is matched against the module graph in a spelling the graph never produces.
struct NamedModules {
    rule_method: &'static str,
    allowlist: bool,
    modules: Vec<String>,
}

impl NamedModules {
    fn of(boundary: &ModuleBoundary) -> Result<Self, String> {
        let (rule_method, allowlist, written): (&'static str, bool, Vec<&String>) =
            match &boundary.rule {
                ModuleRule::MustNotImport { module } => ("must_not_import", false, vec![module]),
                ModuleRule::MustNotBeImportedBy { importer } => {
                    ("must_not_be_imported_by", false, vec![importer])
                }
                ModuleRule::RestrictImportsTo { allowed } => {
                    ("restrict_imports_to", true, allowed.iter().collect())
                }
                ModuleRule::MustOnlyBeImportedBy { allowed } => {
                    ("must_only_be_imported_by", true, allowed.iter().collect())
                }
                ModuleRule::ConfineExternalCrate { .. }
                | ModuleRule::ConfineInlineSymbolPath { .. }
                | ModuleRule::ConfineInlineCall { .. } => ("", false, Vec::new()),
            };
        let modules = written
            .into_iter()
            .map(|module| canonical_spelling_or_error(module, &boundary.crate_package))
            .collect::<Result<_, _>>()?;
        Ok(Self {
            rule_method,
            allowlist,
            modules,
        })
    }

    /// Refuse the first named module that no compiled root declares. A package's roots are separate
    /// module graphs, so a module present in one root and absent from another is a real module; it
    /// is refused only when it is present in none — the policy the governed module is held to.
    fn require_exist(
        &self,
        declared: &std::collections::BTreeSet<String>,
        crate_package: &str,
    ) -> Result<(), String> {
        match self
            .modules
            .iter()
            .find(|module| !declared.contains(*module))
        {
            None => Ok(()),
            Some(module) if self.allowlist => Err(unknown_allowed_module_error(
                module,
                crate_package,
                self.rule_method,
            )),
            Some(module) => Err(unknown_forbidden_module_error(
                module,
                crate_package,
                self.rule_method,
            )),
        }
    }
}

/// An inline confinement's prefix, in its canonical spelling, with the builder that declared it.
///
/// Both builders — `must_not_call_inline` and `confine_inline_call` — pass through here, so the
/// prefix a call is compared with is one spelling in either. The spelling is judged before the
/// package is read; what the prefix names is judged by [`DeclaredPrefix::require_names_something`]
/// once every root has been, since a module or item present in one compilation unit is real.
struct DeclaredPrefix {
    rule_method: &'static str,
    written: String,
    prefix: SymbolPrefix,
}

/// What a boundary's rule declares as an inline confinement: nothing, because the rule is no inline confinement; or
/// the confinement, whose prefix is blank or canonical. Only the second reaches the inline judgement, so a rule with
/// no inline payload cannot be judged as one.
enum InlinePrefix<'a> {
    NotInline,
    Inline(Inline<'a>),
}

/// An inline confinement's declaration: the builder method it was declared through, its prefix and the modifiers
/// the judgement reads.
struct Inline<'a> {
    declared: DeclaredPrefix,
    ending_with: Option<&'a [String]>,
    strict: bool,
    external: bool,
}

impl<'a> InlinePrefix<'a> {
    /// Read an inline confinement's declaration, refusing every misdeclaration the boundary alone decides — a
    /// blank or non-canonical prefix, and `confine_inline_call` over `crate` or at `ScanDepth::Shallow` — before
    /// any root is walked, so a scan refusal in some file cannot stand in front of the line the operator must
    /// change.
    fn of(boundary: &'a ModuleBoundary) -> Result<Self, String> {
        let Some((prefix, ending_with, strict, external)) = boundary.rule.inline_payload() else {
            return Ok(InlinePrefix::NotInline);
        };
        let permitting = matches!(boundary.rule, ModuleRule::ConfineInlineCall { .. });
        let rule_method = if permitting {
            "confine_inline_call"
        } else {
            "must_not_call_inline"
        };
        let blank = prefix.trim().is_empty();
        let canonical = if blank {
            None
        } else {
            Some(
                canonical_symbol_path_spelling(prefix).map_err(|suggestion| {
                    non_canonical_inline_prefix_error(
                        prefix,
                        &boundary.crate_package,
                        rule_method,
                        suggestion.as_deref(),
                    )
                })?,
            )
        };
        if permitting && canonical_module_path(&boundary.module) == "crate" {
            return Err(confine_inline_call_on_crate_error(&boundary.crate_package));
        }
        if permitting && boundary.depth == ScanDepth::Shallow {
            return Err(confine_inline_call_shallow_error(&boundary.crate_package));
        }
        let Some(canonical) = canonical else {
            return Err(inline_empty_prefix_error(
                &boundary.crate_package,
                rule_method,
            ));
        };
        if ending_with.is_some() && strict {
            return Err(inline_narrow_and_strict_error(
                &boundary.crate_package,
                rule_method,
            ));
        }
        if ending_with.is_some_and(|verbs| verbs.is_empty()) {
            return Err(inline_empty_verbs_error(
                &boundary.crate_package,
                rule_method,
            ));
        }
        let declared = DeclaredPrefix {
            rule_method,
            written: prefix.to_string(),
            prefix: canonical,
        };
        Ok(InlinePrefix::Inline(Inline {
            declared,
            ending_with,
            strict,
            external,
        }))
    }
}

impl DeclaredPrefix {
    /// Refuse a prefix that names nothing a call can reach, where the refusal can be decided.
    ///
    /// A `crate::` prefix must name a module some compiled root declares, or an item defined at the
    /// top level of one; segments past that item are not read, since the scanner collects no
    /// associated items. Any other first segment names a crate, and what that crate holds is its own
    /// source, which the scanner does not read. A sysroot crate, a dependency under its local name, or
    /// the package's own library is accepted as such. A first segment none of those confirms is still
    /// accepted — a dependency's crate name can differ from its package name, and `--no-deps` metadata
    /// does not say what it is — unless the same path rooted at `crate` names something, which is the
    /// one reading the text determines: that prefix is refused, suggesting the rooted spelling.
    fn require_names_something(
        &self,
        modules: &std::collections::BTreeSet<String>,
        items: &std::collections::BTreeSet<String>,
        external_crates: &[String],
        crate_package: &str,
    ) -> Result<(), String> {
        let path = self.prefix.path.as_str();
        let head = path.split_once("::").map_or(path, |(head, _)| head);
        match head {
            _ if self.prefix.root == PrefixRoot::Global => Ok(()),
            "crate" if names_a_local_path(path, modules, items) => Ok(()),
            "crate" => Err(unknown_inline_prefix_error(
                &self.written,
                crate_package,
                self.rule_method,
            )),
            _ if sysroot_crate(head).is_some() => Ok(()),
            _ if external_crates.iter().any(|name| name == head) => Ok(()),
            _ => {
                let rooted = format!("crate::{path}");
                if names_a_local_path(&rooted, modules, items) {
                    Err(unknown_inline_prefix_head_error(
                        &self.written,
                        crate_package,
                        self.rule_method,
                        &rooted,
                    ))
                } else {
                    Ok(())
                }
            }
        }
    }
}

/// Whether a `crate`-rooted path names a declared module, or descends through declared modules to an
/// item one of them defines. Whatever follows that item is not read.
fn names_a_local_path(
    path: &str,
    modules: &std::collections::BTreeSet<String>,
    items: &std::collections::BTreeSet<String>,
) -> bool {
    let segments: Vec<&str> = path.split("::").collect();
    for end in 2..=segments.len() {
        let named = segments[..end].join("::");
        if !modules.contains(&named) {
            return items.contains(&named);
        }
    }
    true
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
    root: &RootScan,
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
    for (file, file_module) in root.all_files() {
        if is_inside_protected_module(file_module, governed_module) {
            continue;
        }
        if let Some(forbidden) = &forbidden_importer {
            if !(path_within(file_module, forbidden) || path_within(forbidden, file_module)) {
                continue;
            }
        }
        for (importer, import) in root.unit_scan().imports(file, file_module)? {
            if is_inside_protected_module(importer, governed_module) {
                continue;
            }
            if let Some(forbidden) = &forbidden_importer {
                if !path_within(importer, forbidden) {
                    continue;
                }
            }
            let import_module = resolve_import_module(&import.path, &root.reachable);
            let imports_protected =
                within_scan_depth(import_module, governed_module, boundary.depth)
                    || (import.is_glob && path_within(governed_module, &import.path))
                    || also_binds_a_value_of_the_governed_module(
                        &import.path,
                        import_module,
                        governed_module,
                        import.can_bind_a_value(),
                        &mut governed_value_items,
                        root,
                    )?;
            if !imports_protected {
                continue;
            }
            if forbidden_importer.is_none() {
                let within_allowed = allowed_importers
                    .iter()
                    .any(|entry| path_within(importer, entry));
                if within_allowed {
                    continue;
                }
            }
            offenders.push((importer.clone(), file.display().to_string()));
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
            root.unit_label(),
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
    root: &RootScan,
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
    for (file, file_module) in root.all_files() {
        if hosts_only_permitted_importers(file_module, governed_module, boundary.depth) {
            continue;
        }
        for (importer, external) in root.unit_scan().external_imports(file, file_module)? {
            if external != &confined {
                continue;
            }
            if within_scan_depth(importer, governed_module, boundary.depth) {
                continue;
            }
            offenders.push((importer.clone(), file.display().to_string()));
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
            root.unit_label(),
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
/// `UnitScan::findings` / `resolve_written`. `external` reflects the single rule's
/// `strict_external` modifier.
///
/// Its misdeclarations are refused by `InlinePrefix::of` before any root is walked.
/// Crate-wide files feed type alias and pub use resolution; dependency names are read on demand
/// when external confinement is active.
#[allow(clippy::too_many_arguments)]
fn check_inline_confinement(
    root: &RootScan,
    boundary: &ModuleBoundary,
    package: &Value,
    governed: &[(PathBuf, String)],
    rule: &str,
    prefix: &SymbolPrefix,
    ending_with: Option<&[String]>,
    strict: bool,
    external: bool,
    violations: &mut Vec<Violation>,
) -> Result<(), String> {
    let dependency_names = if external {
        crate::cargo_metadata::dependency_import_names(package)
    } else {
        Vec::new()
    };
    let findings = root.unit_scan().findings(
        governed,
        prefix,
        ending_with,
        strict,
        external,
        &dependency_names,
    )?;
    for InlineFinding { fact, file } in findings {
        push_module_violation(
            violations,
            &prefix.path,
            rule,
            fact,
            file,
            boundary,
            root.unit_label(),
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
    root: &RootScan,
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
        | ModuleRule::ConfineInlineSymbolPath { .. }
        | ModuleRule::ConfineInlineCall { .. } => {
            unreachable!("the inbound / confinement rules are evaluated above and return early")
        }
    };
    let mut findings: Vec<(String, String, String)> = Vec::new();
    for (file, current_module) in governed {
        for (importer, import) in root.unit_scan().imports(&file, &current_module)? {
            if is_violation(import) {
                findings.push((
                    importer.clone(),
                    import.path.clone(),
                    file.display().to_string(),
                ));
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
            root.unit_label(),
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
/// propagate immediately. A package whose every target is an example, a test, a bench or a build script is
/// refused before any root is read: no compiled root reads its `src/`, so a boundary over it could never react.
///
/// Every module path the boundary carries — the governed module and each module its rule names — is held
/// to its canonical spelling before the package is read. The named modules are held to existence after the
/// governed module is: each must be declared in some root's graph, by the same deferral, or the boundary is
/// refused rather than judged against a module no import can reach.
pub(crate) fn check_module_boundary(
    metadata: &Value,
    scans: &EvaluationScans,
    boundary: &ModuleBoundary,
    violations: &mut Vec<Violation>,
) -> Result<(), String> {
    canonical_spelling_or_error(&boundary.module, &boundary.crate_package)?;
    let named = NamedModules::of(boundary)?;
    let inline_prefix = InlinePrefix::of(boundary)?;
    let package = find_package(metadata, &boundary.crate_package)
        .ok_or_else(|| crate_not_found_error(&boundary.crate_package))?;
    let require_named = |declared: &std::collections::BTreeSet<String>,
                         items: &std::collections::BTreeSet<String>| {
        named.require_exist(declared, &boundary.crate_package)?;
        match &inline_prefix {
            InlinePrefix::Inline(Inline {
                declared: prefix, ..
            }) => prefix.require_names_something(
                declared,
                items,
                &crate::cargo_metadata::dependency_import_names(package)
                    .into_iter()
                    .chain(crate::cargo_metadata::library_import_names(package))
                    .collect::<Vec<_>>(),
                &boundary.crate_package,
            ),
            InlinePrefix::NotInline => Ok(()),
        }
    };
    let compiled = match crate_roots(package) {
        CrateRoots::Compiled(roots) => roots,
        CrateRoots::NoneCompiled => return Err(no_compiled_root_error(&boundary.crate_package)),
        CrateRoots::Unreported => {
            let root = scans.root_scan(package, &boundary.crate_package, None, None)?;
            let mut judged = check_one_root(&root, package, boundary, &inline_prefix)?;
            return match judged.outcome {
                RootOutcome::Governed => {
                    require_named(&judged.declared, &judged.items)?;
                    violations.append(&mut judged.violations);
                    Ok(())
                }
                RootOutcome::ModuleAbsent(reason) => Err(reason),
            };
        }
    };
    let roots = compiled.as_slice();
    let mut deferred: Option<String> = None;
    let mut governed_somewhere = false;
    let mut declared = std::collections::BTreeSet::new();
    let mut items = std::collections::BTreeSet::new();
    let mut found = Vec::new();
    for root in roots {
        let scan = scans.root_scan(
            package,
            &boundary.crate_package,
            Some(root.as_path()),
            Some(roots),
        )?;
        let mut judged = check_one_root(&scan, package, boundary, &inline_prefix)?;
        declared.append(&mut judged.declared);
        items.append(&mut judged.items);
        found.append(&mut judged.violations);
        match judged.outcome {
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
            require_named(&declared, &items)?;
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

/// What one root contributes to its package's judgement: whether it hosted the governed module, the modules
/// and items it declares — which the named modules and an inline prefix are held to across every root — and
/// the violations found in it. The caller merges every root's and keeps the violations only when some root is
/// governed.
struct RootJudgement {
    outcome: RootOutcome,
    declared: std::collections::BTreeSet<String>,
    items: std::collections::BTreeSet<String>,
    violations: Vec<Violation>,
}

/// Derives the package-relative label for an inline module extraction suggestion.
///
/// Uses the same [`compilation_unit_label`] mechanism that derives the root's own unit label,
/// so single-root and multi-root packages format suggested paths identically.
fn suggested_module_path(package: &Value, candidate: &Path) -> String {
    compilation_unit_label(package, candidate).unwrap_or_else(|| xingbiao::path_label(candidate))
}

/// Judge one compiled root by deriving its membership, deciding its outcome, then dispatching its rule family.
fn check_one_root(
    root: &RootScan,
    package: &Value,
    boundary: &ModuleBoundary,
    inline_prefix: &InlinePrefix,
) -> Result<RootJudgement, String> {
    let facts = collect_root_facts(root, boundary, inline_prefix);
    match decide_root_outcome(root, package, boundary, &facts)? {
        RootDecision::Complete(outcome) => Ok(RootJudgement {
            outcome,
            declared: facts.declared,
            items: facts.items,
            violations: Vec::new(),
        }),
        RootDecision::Judge(outcome) => {
            dispatch_root_rule_family(root, package, boundary, inline_prefix, facts, outcome)
        }
    }
}

/// The root-level sets derived from one shared scan and one boundary.
struct RootFacts {
    declared: std::collections::BTreeSet<String>,
    items: std::collections::BTreeSet<String>,
    governed_module: String,
    governed: Vec<(PathBuf, String)>,
}

/// Derive the governed set and root-level declared and item sets from the shared scan. [`RootScan`]
/// owns the walk and unit scan — custom target roots relative to `src_dir` map to `crate`, roots
/// outside the package manifest directory error as configuration errors, and sibling compilation
/// unit roots are excluded from module discovery to prevent duplicate violations. The caller obtains
/// that scan before this step, so a walk refusal and a file the unit scan cannot read both surface
/// ahead of whether the governed module exists.
fn collect_root_facts(
    root: &RootScan,
    boundary: &ModuleBoundary,
    inline_prefix: &InlinePrefix,
) -> RootFacts {
    let declared = root.reachable.iter().cloned().collect();
    let governed_module = canonical_module_path(&boundary.module);
    let governed = governed_files(
        &root.src_dir,
        &root.files,
        &governed_module,
        &root.reachable,
        &root.inline_only,
        &root.remapped,
        &root.remap_shadowed,
        root.root_relative.as_deref(),
        boundary.depth,
    );
    let inline = match inline_prefix {
        InlinePrefix::NotInline => None,
        InlinePrefix::Inline(inline) => Some(inline),
    };
    let items = if inline.is_some() {
        root.item_definitions().iter().cloned().collect()
    } else {
        std::collections::BTreeSet::new()
    };

    RootFacts {
        declared,
        items,
        governed_module,
        governed,
    }
}

/// Decide whether this root contains the governed module and whether its perimeter still judges it.
/// Inline modules own no source file and cannot be governed targets (exit 2). An inline target is
/// present in its root rather than absent from it, so its refusal returns at once: deferring it lets
/// a sibling root backing the same path with a file absorb it and leave the inline body unobserved
/// behind a clean report.
///
/// A root without the governed module reports [`RootOutcome::ModuleAbsent`]. Under
/// [`Perimeter::GovernedModule`] it holds nothing the rule governs and is not judged; under
/// [`Perimeter::WholeRoot`] it is judged through the same dispatch as a governed root, with an
/// empty permitted region. The caller keeps those findings only when some root is governed, so a
/// package where no root has the module is still a constitution error.
fn decide_root_outcome(
    root: &RootScan,
    package: &Value,
    boundary: &ModuleBoundary,
    facts: &RootFacts,
) -> Result<RootDecision, String> {
    let outcome = match (
        facts.governed.is_empty(),
        root.inline_only.get(&facts.governed_module),
        boundary.rule.perimeter(),
    ) {
        (true, Some(candidate), _) => {
            let leaf = facts
                .governed_module
                .rsplit_once("::")
                .map_or(facts.governed_module.as_str(), |(_, leaf)| leaf);
            let suggested_path = suggested_module_path(package, candidate);
            return Err(inline_module_target_error(
                &boundary.module,
                &boundary.crate_package,
                leaf,
                root.unit.as_deref(),
                &suggested_path,
            ));
        }
        (true, None, Perimeter::GovernedModule) => {
            return Ok(RootDecision::Complete(RootOutcome::ModuleAbsent(
                unknown_module_error(&boundary.module, &boundary.crate_package),
            )));
        }
        (true, None, Perimeter::WholeRoot) => RootOutcome::ModuleAbsent(unknown_module_error(
            &boundary.module,
            &boundary.crate_package,
        )),
        (false, _, _) => RootOutcome::Governed,
    };

    Ok(RootDecision::Judge(outcome))
}

/// Dispatch this root to its rule family without changing the family's inputs or precedence.
/// `confine_inline_call` judges the complement of `must_not_call_inline`'s set: every file of the
/// root whose module is outside the permitted subtree. An inline finding carries its file's module,
/// and every inline child of a file lies within that file's module subtree, so excluding a file by
/// its module is exact under subtree depth. It is not exact under `ScanDepth::Shallow`, where the
/// permitted region is the anchored module alone and its inline children fall outside it, so a
/// shallow declaration is refused rather than judged.
fn dispatch_root_rule_family(
    root: &RootScan,
    package: &Value,
    boundary: &ModuleBoundary,
    inline_prefix: &InlinePrefix,
    facts: RootFacts,
    outcome: RootOutcome,
) -> Result<RootJudgement, String> {
    let RootFacts {
        declared,
        items,
        governed_module,
        governed,
    } = facts;
    let rule = boundary.rule.label();
    let inline = match inline_prefix {
        InlinePrefix::NotInline => None,
        InlinePrefix::Inline(inline) => Some(inline),
    };
    let inbound = matches!(
        &boundary.rule,
        ModuleRule::MustNotBeImportedBy { .. } | ModuleRule::MustOnlyBeImportedBy { .. }
    );
    let mut violations = Vec::new();
    if inbound {
        check_inbound_rule(root, boundary, &governed_module, rule, &mut violations)?;
    } else if let ModuleRule::ConfineExternalCrate { crate_name } = &boundary.rule {
        check_external_confinement(
            root,
            boundary,
            &governed_module,
            rule,
            crate_name,
            &mut violations,
        )?;
    } else if let Some(inline) = inline {
        let permitting = matches!(boundary.rule, ModuleRule::ConfineInlineCall { .. });
        let judged: Vec<(PathBuf, String)> = if permitting {
            root.all_files()
                .iter()
                .filter(|(_, module)| !within_scan_depth(module, &governed_module, boundary.depth))
                .cloned()
                .collect()
        } else {
            governed
        };
        let prefix = &inline.declared.prefix;
        check_inline_confinement(
            root,
            boundary,
            package,
            &judged,
            rule,
            prefix,
            inline.ending_with,
            inline.strict,
            inline.external,
            &mut violations,
        )?;
    } else {
        check_outbound_rule(
            root,
            boundary,
            &governed_module,
            governed,
            rule,
            &mut violations,
        )?;
    }
    Ok(RootJudgement {
        outcome,
        declared,
        items,
        violations,
    })
}

/// A root either contributes a completed absence or continues through its rule family.
enum RootDecision {
    Complete(RootOutcome),
    Judge(RootOutcome),
}
