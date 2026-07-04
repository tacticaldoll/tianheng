//! 渾儀 (Húnyí) — the **semantic** observation dimension of Tianheng.
//!
//! Where the gnomon 圭表 observes *imports* (does `domain` import `infra`?), 渾儀
//! observes *meaning* via the AST (`syn`): does a module's **public API expose** a
//! forbidden type? That is the complement of import-governance — a type imported for
//! internal use is fine; a type named in a `pub` signature is a leak, and one named via a
//! fully-qualified path with no `use` is invisible to a token scanner but caught here.
//!
//! Declare a [`SemanticBoundary`] in Rust, [`check`] it against a Cargo workspace, and get
//! an [`Outcome`] with the same exit-code contract (0/1/2) and reaction model as the static
//! dimension — both express findings in the shared 璇璣 (`xuanji`) crate. The heavy `syn`
//! dependency is quarantined to this crate, never the core (`self_governance.rs`).
//!
//! **Observed surface and its honest bounds.** The exposed surface of a module anchor is
//! its `pub` free functions (params/returns), `pub` struct/enum/union field types, `pub`
//! type-alias targets, `pub` const/static types, `pub` trait method signatures and
//! associated-type bounds (and supertraits), the generic bounds / `where`-clauses of those
//! items, the `pub` methods of **inherent** `impl` blocks, and **named public re-exports**
//! (a `pub use crate::infra::X;` republishes the forbidden type on the module's surface — observed
//! by default; a glob re-export reacts when its root is in/under the forbidden set). A **trait**
//! `impl` block's impl-site-authored positions are out of scope by default but observable via the opt-in
//! `.including_trait_impls()` depth (`semantic-trait-impl-exposure`): the trait ref's generic
//! args, the `Self` type, associated-type bindings, the impl's own generics/`where`-clause, and
//! the method **return** type as written (its params/receiver stay trait-dictated). Out of scope
//! (stated bounds, not silent passes): a type reachable only through a **glob** import or a
//! **macro**; and a type knowable only through **inference** (a return-position `impl Trait` that
//! *hides* a concrete type, or a **complex-target or generic** type alias — `type X = Vec<Db>` /
//! `type X<T> = …`). A **resolvable single-path** `type X = a::b::C;` alias chain, by contrast, *is*
//! followed to its defining path and reacts (v0.1.4). Within the resolved scope there is no false
//! negative: a forbidden type that *is* resolvable always reacts.
//!
//! Govern by reaction, not instruction.

#![deny(missing_docs)]

use std::collections::{HashMap, HashSet};
use std::path::{Path, PathBuf};
use std::process::Command;

use serde_json::Value;
use syn::parse::Parser;
use syn::visit::Visit;

mod resolve;
use resolve::{
    AliasMap, BareFallback, DynCollector, ExternRenameMap, ImplTraitCollector, PathCollector,
    ReexportMap, ShapeExposure, UseMap, alias_nominal_target, bare_local_alias, canonical_path_str,
    canonical_self_owner, canonicalize_through_aliases, canonicalize_through_reexports,
    collect_reexports, collect_uses, extern_verbatim_renamed, path_to_string, resolve_path,
    stamp_seam, strip_raw, type_to_string,
};

// The reaction model is the shared 璇璣 crate, re-exported so a consumer can stay on
// hunyi's surface; these names are also used internally below.
pub use xuanji::{
    Baseline, BoundaryKind, Outcome, Report, Severity, Violation, ViolationId, apply_baseline,
};

// --- Canonical rule labels (single source per rule) --------------------------
//
// Each semantic rule's label is written **once**, here, and referenced by both the
// `check`-side `Violation::new(...)` (the reaction) and the `list` projections in the
// 天衡 shell (`tianheng` depends on `hunyi`, so importing these is the allowed direction).
// Editing a label in one place updates every projection — the `list`/`check` and
// text/JSON drift this closes. These are the rule *family* strings; a per-boundary operand
// detail (e.g. the dyn/impl-trait operand set) stays a parameter layered on at projection.

/// Signature-coupling: a module's public API must not expose a forbidden type.
pub const SIGNATURE_RULE: &str = "must not expose";
/// Dyn-trait: a module's public API must not expose trait-object (`dyn`) syntax.
pub const DYN_TRAIT_RULE: &str = "must not expose dyn";
/// Impl-trait: a module's public API must not return a written `impl Trait` (RPIT).
pub const IMPL_TRAIT_RULE: &str = "must not expose impl trait";
/// Async-exposure: a module's public API must not declare an `async fn`.
pub const ASYNC_EXPOSURE_RULE: &str = "must not expose async fn";
/// Trait-impl-locality: a trait may be implemented only in its declared location(s).
pub const TRAIT_IMPL_RULE: &str = "must only be implemented in the declared location(s)";
/// Visibility: a module must not declare bare-`pub` items.
pub const VISIBILITY_RULE: &str = "must not declare pub items";
/// Forbidden-marker: a subtree's types must not acquire a forbidden trait.
pub const FORBIDDEN_MARKER_RULE: &str = "must not acquire trait";

mod dsl;
pub use dsl::*;

// --- Constitution-error messages ---------------------------------------------

fn unreadable_workspace_error(manifest_path: &Path, err: &str) -> String {
    format!(
        "a boundary is observed against a real workspace, so an unreadable one cannot be judged \
         and its verdict would be a false pass: cannot read target workspace at {} ({err}); check \
         the manifest path and that `cargo metadata` succeeds",
        manifest_path.display()
    )
}

fn crate_not_found_error(crate_package: &str) -> String {
    // Duplicated verbatim with guibiao's twin (the price of the dimension split, guibiao:~47);
    // the two copies carry the SAME wording in-place rather than sharing a module (which would
    // need a forbidden guibiao↔hunyi edge).
    format!(
        "a boundary must govern a real crate or it silently never reacts: target crate \
         '{crate_package}' is not a member of the target workspace — check the name or --manifest-path"
    )
}

fn missing_src_error(crate_package: &str) -> String {
    format!(
        "a semantic boundary is observed from source, so with no src it could never react: cannot \
         locate the crate root source for '{crate_package}'"
    )
}

fn unknown_module_error(module: &str, crate_package: &str) -> String {
    format!(
        "a boundary must anchor to a real module or it silently never reacts: module '{module}' is \
         not found among the modules of crate '{crate_package}' (declared via `mod`) — check the path"
    )
}

fn unknown_trait_error(trait_path: &str, crate_package: &str) -> String {
    format!(
        "a trait-impl-locality boundary must anchor to a real local trait or it silently never \
         reacts: trait '{trait_path}' is not found as a `trait` item (directly or via a local \
         `pub use`) in crate '{crate_package}' — check the path"
    )
}

fn missing_module_file_error(module: &str, crate_package: &str) -> String {
    format!(
        "module '{module}' of crate '{crate_package}' is declared (`mod …;`) but its source file \
         could not be located (expected `<name>.rs` or `<name>/mod.rs`)"
    )
}

fn unreadable_source_error(file: &Path, err: &str) -> String {
    format!("cannot read source file '{}': {err}", file.display())
}

fn unparseable_source_error(file: &Path, err: &str) -> String {
    // A file we cannot parse is "cannot judge", not "nothing to judge": skipping it could
    // hide a real exposure. Fail loud as a scan error (exit 2), never a silent pass.
    format!("cannot parse source file '{}': {err}", file.display())
}

// --- The 渾儀 dimension's boundary set ----------------------------------------

/// The 渾儀 (semantic) dimension's boundaries, gathered so the shell takes the dimension as
/// one unit rather than one parameter per capability. Each field is one capability's
/// boundaries; [`check_all`] evaluates them all with a single `cargo metadata` read.
#[derive(Debug, Clone, Default)]
pub struct SemanticBoundaries {
    /// Exposure boundaries (`semantic-signature-coupling`).
    pub signature: Vec<SemanticBoundary>,
    /// Impl-locality boundaries (`semantic-trait-impl-locality`).
    pub trait_impl: Vec<TraitImplBoundary>,
    /// Visibility boundaries (`semantic-visibility-boundary`).
    pub visibility: Vec<VisibilityBoundary>,
    /// Forbidden-marker boundaries (`semantic-forbidden-marker`).
    pub forbidden_marker: Vec<ForbiddenMarkerBoundary>,
    /// Dyn-trait exposure boundaries (`semantic-dyn-trait-boundary`).
    pub dyn_trait: Vec<DynTraitBoundary>,
    /// Impl-trait (existential) exposure boundaries (`semantic-impl-trait-boundary`).
    pub impl_trait: Vec<ImplTraitBoundary>,
    /// Async-fn (implicit existential) exposure boundaries (`semantic-async-exposure-boundary`).
    pub async_exposure: Vec<AsyncExposureBoundary>,
}

impl SemanticBoundaries {
    /// Whether no semantic boundary of any kind is declared.
    pub fn is_empty(&self) -> bool {
        self.signature.is_empty()
            && self.trait_impl.is_empty()
            && self.visibility.is_empty()
            && self.forbidden_marker.is_empty()
            && self.dyn_trait.is_empty()
            && self.impl_trait.is_empty()
            && self.async_exposure.is_empty()
    }
}

/// Evaluate every declared semantic boundary against the workspace with a **single**
/// `cargo metadata` read, merging all findings into one outcome. A constitution error on any
/// boundary supersedes (exit 2). The per-capability `check`/`check_trait_impl_locality`/
/// `check_visibility` entries remain for direct use; the shell composes via this.
pub fn check_all(boundaries: &SemanticBoundaries, manifest_path: &Path) -> Outcome {
    let metadata = match cargo_metadata(manifest_path) {
        Ok(metadata) => metadata,
        Err(err) => {
            return Outcome::ConstitutionError(unreadable_workspace_error(manifest_path, &err));
        }
    };
    let mut violations = Vec::new();
    for boundary in &boundaries.signature {
        if let Err(error) = check_boundary(&metadata, boundary, &mut violations) {
            return Outcome::ConstitutionError(error);
        }
    }
    for boundary in &boundaries.trait_impl {
        if let Err(error) = check_trait_impl_boundary(&metadata, boundary, &mut violations) {
            return Outcome::ConstitutionError(error);
        }
    }
    for boundary in &boundaries.visibility {
        if let Err(error) = check_visibility_boundary(&metadata, boundary, &mut violations) {
            return Outcome::ConstitutionError(error);
        }
    }
    for boundary in &boundaries.forbidden_marker {
        if let Err(error) = check_forbidden_marker_boundary(&metadata, boundary, &mut violations) {
            return Outcome::ConstitutionError(error);
        }
    }
    for boundary in &boundaries.dyn_trait {
        if let Err(error) = check_dyn_trait_boundary(&metadata, boundary, &mut violations) {
            return Outcome::ConstitutionError(error);
        }
    }
    for boundary in &boundaries.impl_trait {
        if let Err(error) = check_impl_trait_boundary(&metadata, boundary, &mut violations) {
            return Outcome::ConstitutionError(error);
        }
    }
    for boundary in &boundaries.async_exposure {
        if let Err(error) = check_async_exposure_boundary(&metadata, boundary, &mut violations) {
            return Outcome::ConstitutionError(error);
        }
    }
    if violations.is_empty() {
        Outcome::Clean
    } else {
        Outcome::Violations(Report::new(violations))
    }
}

// --- The reaction ------------------------------------------------------------

/// Run the semantic boundaries against the Cargo workspace at `manifest_path`.
///
/// The spine mirrors the static dimension — resolve → observe → compare → react: resolve
/// each boundary's crate and module anchor, observe the module's public-API surface from
/// the AST, compare each exposed type against the forbidden set, and return the outcome. An
/// unresolvable crate or module (or an unreadable/unparseable source) is a constitution
/// error (exit 2), never a silent pass.
pub fn check(boundaries: &[SemanticBoundary], manifest_path: &Path) -> Outcome {
    let metadata = match cargo_metadata(manifest_path) {
        Ok(metadata) => metadata,
        Err(err) => {
            return Outcome::ConstitutionError(unreadable_workspace_error(manifest_path, &err));
        }
    };
    let mut violations = Vec::new();
    for boundary in boundaries {
        if let Err(error) = check_boundary(&metadata, boundary, &mut violations) {
            return Outcome::ConstitutionError(error);
        }
    }
    if violations.is_empty() {
        Outcome::Clean
    } else {
        Outcome::Violations(Report::new(violations))
    }
}

/// The governed module's source file rendered for a single-module semantic violation's `file`
/// (`display()`-rendered to match the static dimension). Resolved **only when there is
/// something to report**, so a clean module never pays the second traversal and no error path
/// opens on an empty result; `None` when there are no findings. Shared by the five
/// single-module semantic capabilities (exposure, dyn-trait, impl-trait, async-exposure,
/// visibility). The two whole-crate-scan capabilities (trait-impl-locality, forbidden-marker)
/// do NOT use this — their violations sit at per-site files across the crate, a stated `null`
/// bound narrowed to them.
fn seam_file(
    findings: &[String],
    src_dir: &Path,
    root_file: &Path,
    module: &str,
    crate_package: &str,
) -> Result<Option<String>, String> {
    if findings.is_empty() {
        return Ok(None);
    }
    Ok(Some(
        resolve_module_file(src_dir, root_file, module, crate_package)?
            .display()
            .to_string(),
    ))
}

/// The source file for a **whole-crate-scan** semantic violation (trait-impl-locality,
/// forbidden-marker), whose findings each name their own module — unlike [`seam_file`]'s single
/// per-boundary module. Memoized in `cache` so a boundary with many findings across few modules
/// parses each module path once. Degrades to `None` on a resolution failure (**`.ok()`, never
/// `?`**): the module comes from the whole-crate scan while the file comes from the single-path
/// resolver, so — though they agree for every module a finding can come from — a failure must
/// leave the violation firing with a `null` file, never turn it into an exit-2 error or drop it.
fn per_finding_file(
    module: &str,
    src_dir: &Path,
    root_file: &Path,
    crate_package: &str,
    cache: &mut HashMap<String, Option<String>>,
) -> Option<String> {
    if let Some(cached) = cache.get(module) {
        return cached.clone();
    }
    let file = resolve_module_file(src_dir, root_file, module, crate_package)
        .ok()
        .map(|path| path.display().to_string());
    cache.insert(module.to_string(), file.clone());
    file
}

/// Resolve a semantic boundary's target crate to `(package, crate-root file, source dir)` — the
/// shared preamble every single-crate `check_*_boundary` opens with. One home for the three
/// constitution errors (crate-not-found, and missing-src for a target with no crate-root file or a
/// root file with no parent dir) so the seven capabilities cannot drift apart on resolution. The
/// `src_dir` is returned owned (it would otherwise borrow the root file), so callers hold both.
fn resolve_crate<'m>(
    metadata: &'m Value,
    crate_package: &str,
) -> Result<(&'m Value, PathBuf, PathBuf), String> {
    let package = find_package(metadata, crate_package)
        .ok_or_else(|| crate_not_found_error(crate_package))?;
    let root_file = crate_root_file(package).ok_or_else(|| missing_src_error(crate_package))?;
    let src_dir = root_file
        .parent()
        .ok_or_else(|| missing_src_error(crate_package))?
        .to_path_buf();
    Ok((package, root_file, src_dir))
}

fn check_boundary(
    metadata: &Value,
    boundary: &SemanticBoundary,
    violations: &mut Vec<Violation>,
) -> Result<(), String> {
    let (package, root_file, src_dir) = resolve_crate(metadata, &boundary.crate_package)?;
    let src_dir = src_dir.as_path();

    let findings = module_findings(
        src_dir,
        &root_file,
        &boundary.module,
        &boundary.forbidden,
        &boundary.crate_package,
        boundary.including_trait_impls,
        &dependency_names(package),
    )?;

    // A single-module semantic violation's `file` is the governed module's source file — the
    // file the scan descended to observe this module's items, where the exposing seam is
    // written. `finding` is a canonical type path (resolved through re-export chains), so the
    // forbidden type may be *defined* in another file; the `file` reports the seam's location,
    // the actionable one. Resolved once (only when reporting), sharing `module_findings`'
    // traversal so it cannot disagree with the items reacted on.
    let module_file = seam_file(
        &findings,
        src_dir,
        &root_file,
        &boundary.module,
        &boundary.crate_package,
    )?;
    for finding in findings {
        violations.push(
            Violation::new(
                BoundaryKind::Semantic,
                boundary.module.clone(),
                SIGNATURE_RULE.to_string(),
                finding,
                boundary.reason.clone(),
                boundary.severity,
            )
            .with_file(module_file.clone()),
        );
    }
    Ok(())
}

/// The sysroot crates: valid extern path heads that never appear in a package's declared
/// `dependencies`, so the external-crate set includes them explicitly (forbidding e.g.
/// `std::process` at a facade seam is legitimate intent, not a lint).
const SYSROOT_CRATES: [&str; 5] = ["std", "core", "alloc", "proc_macro", "test"];

/// The names a crate's declared dependencies are written under **in source**: each
/// dependency's `rename` when present (a Cargo `pkg = { package = "…" }` rename), else its
/// package `name`, normalized `-`→`_` to the Rust path spelling (`async-trait` →
/// `async_trait`). Read from the `cargo metadata --no-deps` package — declared-manifest data,
/// no resolved graph, no network. This is a deliberate **superset**: dev/build/target/optional
/// dependency names are kept too (the false-negative-safe direction — a name that cannot appear
/// in a compiling public-surface path simply never matches). A dependency that renames its
/// **`[lib] name`** to something not derivable from the package name is a stated bound (the
/// foreign crate's target name is not in `--no-deps`).
fn dependency_names(package: &Value) -> Vec<String> {
    package["dependencies"]
        .as_array()
        .map(|deps| {
            deps.iter()
                .filter_map(|dep| {
                    dep["rename"]
                        .as_str()
                        .or_else(|| dep["name"].as_str())
                        .map(|name| name.replace('-', "_"))
                })
                .collect()
        })
        .unwrap_or_default()
}

/// The **raw** external-crate name set: declared dependency names (already `-`→`_` normalized,
/// rename-aware) ∪ the sysroot crates. This is the oracle for a bare **re-export** (`pub use`)
/// head, which is an external crate by edition-2018+ grammar regardless of any local module of
/// the same name. A bare **type-position** head is resolved against this set with the governed
/// module's own type-namespace items excluded (see [`local_type_namespace_names`]) — the shadow
/// that keeps a local `mod serde` / `struct serde` from being read as the dependency `serde`.
fn external_crate_set(dep_names: &[String]) -> HashSet<String> {
    dep_names
        .iter()
        .cloned()
        .chain(SYSROOT_CRATES.iter().map(|s| s.to_string()))
        .collect()
}

/// The governed module's own **type-namespace** item names — `mod`, `struct`, `enum`, `union`,
/// `trait`, and `type` alias (generic or not) declarations. In a **type-position** path, a bare
/// head naming one of these denotes that local item, not an external crate of the same name
/// (rustc: any local type-namespace item shadows the extern prelude — `pub struct serde` makes
/// bare `serde` the struct, and `serde::X` then cannot denote the dependency at all), so these
/// are excluded from the external-crate set for type positions (a per-module shadow, correct at
/// any edition). Scoped to the module being analyzed — a crate-root item never shadows a *child*
/// module's bare paths. Previously only `mod` was excluded, which false-positived a local type
/// named like a dependency.
fn local_type_namespace_names(items: &[syn::Item]) -> HashSet<String> {
    items
        .iter()
        .filter_map(|item| match item {
            syn::Item::Mod(m) => Some(strip_raw(&m.ident.to_string())),
            syn::Item::Struct(s) => Some(strip_raw(&s.ident.to_string())),
            syn::Item::Enum(e) => Some(strip_raw(&e.ident.to_string())),
            syn::Item::Union(u) => Some(strip_raw(&u.ident.to_string())),
            syn::Item::Trait(t) => Some(strip_raw(&t.ident.to_string())),
            syn::Item::Type(t) => Some(strip_raw(&t.ident.to_string())),
            _ => None,
        })
        .collect()
}

/// The pure heart, testable without spawning `cargo`: resolve the module's items, observe
/// the exposed type paths, resolve each against the in-scope `use`s, and return the sorted,
/// deduplicated canonical paths that fall within the forbidden set.
pub(crate) fn module_findings(
    src_dir: &Path,
    root_file: &Path,
    module: &str,
    forbidden: &[String],
    crate_package: &str,
    include_trait_impls: bool,
    dep_names: &[String],
) -> Result<Vec<String>, String> {
    let items = resolve_module_items(src_dir, root_file, module, crate_package)?;
    let uses = collect_uses(&items);
    // The external-crate name set: declared dependencies (`-`→`_` normalized, rename-aware) ∪
    // the sysroot crates. A bare head in it denotes an external crate, so an inline extern path
    // resolves to itself verbatim and reacts — closing the extern-path false negative. Applied
    // only in the bare-fallback branch (after `use`-map / `crate`-relative), and only here + the
    // re-export closure.
    let externs = external_crate_set(dep_names);
    // A bare **type-position** head may be a child module of the governed module (a local
    // `mod serde` denotes `crate::…::serde`, not the dependency `serde`), so type positions use
    // the set with the module's own child modules excluded. A bare **re-export** head is extern
    // by edition-2018+ grammar even with a same-named local module, so re-exports use the raw
    // set — resolving these differently is what keeps a subtree's real extern re-export reacting
    // (no false negative) while a same-named local type is not misread (no false positive).
    let externs_type: HashSet<String> = externs
        .difference(&local_type_namespace_names(&items))
        .cloned()
        .collect();
    // The re-export and alias closures are crate-wide: a forbidden type exposed through a
    // `pub use` facade or a `type X = <path>;` alias must canonicalize to its defining path
    // before matching. The re-export closure retains an extern-headed target (raw set — a bare
    // `pub use` head is extern by grammar), so a local facade chain terminating at an extern
    // type canonicalizes to it; the alias closure follows resolvable-nominal-path aliases.
    let scan = scan_crate(src_dir, root_file, crate_package, &externs)?;
    let reexports = scan.reexports;
    let aliases = scan.aliases;
    // Source-level crate-root `extern crate X as Y;` renames: a renamed head resolves to the real
    // crate before the extern check (the whole walk completes before we resolve, so the map is
    // fully populated — no ordering hazard).
    let extern_renames = scan.extern_renames;
    let forbidden: Vec<String> = forbidden.iter().map(|f| canonical_path_str(f)).collect();

    let mut exposed = Vec::new();
    for (ordinal, item) in items.iter().enumerate() {
        collect_item_exposures(item, module, &uses, ordinal, &mut exposed);
        // Opt-in depth: also observe the module's trait `impl` blocks' impl-site-authored
        // positions (`semantic-trait-impl-exposure`). The same resolve → canonicalize → match →
        // `{type} exposed by {seam}` pipeline below applies unchanged; only the seam differs.
        if include_trait_impls {
            collect_trait_impl_exposures(item, module, &uses, ordinal, &mut exposed);
        }
    }

    let mut findings: Vec<String> = exposed
        .iter()
        .filter_map(|exposure| {
            // `resolve_path` returns None for a bare head (not `crate`-relative, not in the
            // `use`-map); the extern oracle then fires for an external-crate head, resolving
            // the inline extern path to itself. Ordering guarantees a local `use … as <dep>`
            // alias (found in the `use`-map) still wins over a dependency of the same name. A
            // re-export head uses the raw set; a type-position head uses the shadow-excluded set.
            let type_externs = if exposure.is_reexport {
                &externs
            } else {
                &externs_type
            };
            // A leading `::` is an unambiguous extern (edition 2018+): resolve against the RAW
            // extern set (with the crate-root `extern crate … as` rename applied, so a
            // `::<rename>::Type` head still resolves to its real crate), bypassing the `use`-map
            // and the local type-namespace shadow, as a HARD short-circuit — a non-dependency
            // `::head` stays unresolved (a bound), never mis-attributed through the `use`-map.
            // `extern_verbatim_renamed` ignores `leading_colon` (it iterates the segments), so
            // the raw set makes it react to `::serde` under a local `mod serde` (the shadow case)
            // while the rename hop is preserved; a local `mod` is never a rename, so no FP.
            // Otherwise: a bare single-segment local `type` alias resolves before the extern
            // oracle (a local alias shadows a same-named dependency), and the combined closure
            // follows alias→alias / alias→re-export hops.
            let resolved = if exposure.path.leading_colon.is_some() {
                extern_verbatim_renamed(&exposure.path, &externs, &extern_renames)
            } else {
                resolve_path(&exposure.path, &uses, module, BareFallback::Ignore)
                    .or_else(|| bare_local_alias(&exposure.path, module, &aliases))
                    .or_else(|| {
                        extern_verbatim_renamed(&exposure.path, type_externs, &extern_renames)
                    })
            };
            resolved
                .map(|canonical| canonicalize_through_aliases(&canonical, &aliases, &reexports))
                .filter(|canonical| matches_forbidden(canonical, &forbidden))
                // Seam-qualify: two distinct seams exposing the same forbidden type stay distinct
                // findings, so baselining one never masks a new leak at another (the one forbidden
                // bug) — the shape/existential rules do the same below.
                .map(|canonical| {
                    SemanticFinding::Exposed {
                        subject: canonical,
                        seam: exposure.seam.clone(),
                    }
                    .to_string()
                })
        })
        .collect();
    findings.sort();
    findings.dedup();
    Ok(findings)
}

// --- Dyn-trait-boundary: the reaction ----------------------------------------

/// Run the dyn-trait boundaries against the Cargo workspace at `manifest_path`.
///
/// Mirrors [`check`]: resolve each boundary's crate and module anchor, observe the module's
/// public-API surface for trait-object (`dyn`) nodes at any depth, and react. An
/// unresolvable crate or module (or an unreadable/unparseable source) is a constitution
/// error (exit 2), never a silent pass. The per-capability entry remains for direct use; the
/// shell composes via [`check_all`].
pub fn check_dyn_trait(boundaries: &[DynTraitBoundary], manifest_path: &Path) -> Outcome {
    let metadata = match cargo_metadata(manifest_path) {
        Ok(metadata) => metadata,
        Err(err) => {
            return Outcome::ConstitutionError(unreadable_workspace_error(manifest_path, &err));
        }
    };
    let mut violations = Vec::new();
    for boundary in boundaries {
        if let Err(error) = check_dyn_trait_boundary(&metadata, boundary, &mut violations) {
            return Outcome::ConstitutionError(error);
        }
    }
    if violations.is_empty() {
        Outcome::Clean
    } else {
        Outcome::Violations(Report::new(violations))
    }
}

fn check_dyn_trait_boundary(
    metadata: &Value,
    boundary: &DynTraitBoundary,
    violations: &mut Vec<Violation>,
) -> Result<(), String> {
    let (package, root_file, src_dir) = resolve_crate(metadata, &boundary.crate_package)?;
    let src_dir = src_dir.as_path();

    // Empty operand set ⇒ shape-only (any dyn), using the resolution-free path unchanged; a
    // named set ⇒ operand-scoped, resolving each dyn's principal trait against the forbidden set.
    let findings = if boundary.forbidden_operands.is_empty() {
        dyn_module_findings(
            src_dir,
            &root_file,
            &boundary.module,
            &boundary.crate_package,
        )?
    } else {
        dyn_operand_module_findings(
            src_dir,
            &root_file,
            &boundary.module,
            &boundary.forbidden_operands,
            &boundary.crate_package,
            &dependency_names(package),
        )?
    };

    // The governed module's source file — where the exposing seam is written (see
    // `check_boundary`). Shared with the dyn shape/operand hearts' traversal.
    let module_file = seam_file(
        &findings,
        src_dir,
        &root_file,
        &boundary.module,
        &boundary.crate_package,
    )?;
    for finding in findings {
        violations.push(
            Violation::new(
                BoundaryKind::Semantic,
                boundary.module.clone(),
                DYN_TRAIT_RULE.to_string(),
                finding,
                boundary.reason.clone(),
                boundary.severity,
            )
            .with_file(module_file.clone()),
        );
    }
    Ok(())
}

/// The pure heart of dyn-trait-boundary, testable without spawning `cargo`: resolve the
/// module's items and return the sorted, deduplicated rendered `dyn` shapes exposed in its
/// public surface. The *reaction* is on the *presence* of a `dyn` node (shape-only), so it needs
/// no name resolution and no re-export closure — `pub use`-chain following is inert for a `dyn`
/// (a re-export carries a name, never a `dyn` node). The `use`-map it does collect serves only to
/// canonicalize an inherent impl's self-type **owner** in the seam (a finding-identity concern,
/// not detection); no re-export closure is needed for that.
pub(crate) fn dyn_module_findings(
    src_dir: &Path,
    root_file: &Path,
    module: &str,
    crate_package: &str,
) -> Result<Vec<String>, String> {
    let items = resolve_module_items(src_dir, root_file, module, crate_package)?;
    // `uses` is not needed to *detect* a `dyn` (shape-only), but it canonicalizes an inherent
    // impl's self-type owner in the seam — cheap (reads the already-parsed items' `use` decls),
    // and it needs no re-export closure (the owner identity does not resolve through facades).
    let uses = collect_uses(&items);
    let mut exposures = Vec::new();
    for (ordinal, item) in items.iter().enumerate() {
        collect_item_dyn_exposures(item, module, &uses, ordinal, &mut exposures);
    }
    let mut findings: Vec<String> = exposures.into_iter().map(shape_finding).collect();
    findings.sort();
    findings.dedup();
    Ok(findings)
}

/// The pure heart of the **operand-scoped** dyn-trait boundary: like [`dyn_module_findings`]
/// but keeps only the `dyn` nodes whose **principal trait** resolves into the forbidden operand
/// set. Unlike the shape-only path it **needs** the module's `use`-map and re-export closure —
/// the principal trait is resolved and canonicalized exactly as [`module_findings`] resolves an
/// exposed type (`resolve_path(BareFallback::Ignore)` → `canonicalize_through_reexports` →
/// `matches_forbidden`, exact-or-module-prefix), so a re-exported/aliased trait facade matches
/// its defining path. A principal that does not resolve (a bare name with no `use`, a
/// macro-generated or glob/cross-crate re-exported trait) is dropped — the stated
/// resolver-coverage bound, never a silent pass of a *resolvable* operand. The finding stays the
/// rendered `dyn …` shape (parity with the shape-only rule and its baseline identity).
pub(crate) fn dyn_operand_module_findings(
    src_dir: &Path,
    root_file: &Path,
    module: &str,
    forbidden: &[String],
    crate_package: &str,
    dep_names: &[String],
) -> Result<Vec<String>, String> {
    let items = resolve_module_items(src_dir, root_file, module, crate_package)?;
    let uses = collect_uses(&items);
    let ExternResolution {
        externs,
        externs_type,
        reexports,
        extern_renames,
    } = extern_resolution(src_dir, root_file, crate_package, dep_names, &items)?;
    let forbidden: Vec<String> = forbidden.iter().map(|f| canonical_path_str(f)).collect();

    let mut exposures = Vec::new();
    for (ordinal, item) in items.iter().enumerate() {
        collect_item_dyn_exposures(item, module, &uses, ordinal, &mut exposures);
    }

    let mut findings: Vec<String> = exposures
        .into_iter()
        .filter(|exposure| {
            // Empty forbidden set ⇒ any dyn (the shape-only semantic), never a silent no-op —
            // safe even if a future caller routes an empty set here (check routes it to the
            // cheaper resolution-free path, but the invariant must not depend on that).
            forbidden.is_empty()
                || exposure
                    .principal
                    .as_ref()
                    .and_then(|path| {
                        resolve_principal(
                            path,
                            &uses,
                            module,
                            &externs,
                            &externs_type,
                            &extern_renames,
                            &reexports,
                        )
                    })
                    .is_some_and(|canonical| matches_forbidden(&canonical, &forbidden))
        })
        .map(shape_finding)
        .collect();
    findings.sort();
    findings.dedup();
    Ok(findings)
}

/// The extern-resolution context an operand principal-trait match needs, mirroring the type
/// side's setup in [`module_findings`]: the raw external-crate set, its type-position
/// shadow-excluded variant, the (extern-aware) re-export closure, and the crate-root
/// `extern crate … as` rename map. Built once per operand-heart call.
struct ExternResolution {
    externs: HashSet<String>,
    externs_type: HashSet<String>,
    reexports: ReexportMap,
    extern_renames: ExternRenameMap,
}

fn extern_resolution(
    src_dir: &Path,
    root_file: &Path,
    crate_package: &str,
    dep_names: &[String],
    items: &[syn::Item],
) -> Result<ExternResolution, String> {
    let externs = external_crate_set(dep_names);
    let externs_type: HashSet<String> = externs
        .difference(&local_type_namespace_names(items))
        .cloned()
        .collect();
    let scan = scan_crate(src_dir, root_file, crate_package, &externs)?;
    Ok(ExternResolution {
        externs,
        externs_type,
        reexports: scan.reexports,
        extern_renames: scan.extern_renames,
    })
}

/// Resolve a shape's **principal-trait** path through the same extern-aware ladder
/// signature-coupling uses for an exposed type — **minus the type-alias closure**: a `dyn`/`impl`
/// of a type alias is not stable Rust (rustc E0404 "type aliases cannot be used as traits"), so a
/// trait operand has no alias hop to follow. A leading-`::` head is an unambiguous extern (raw set,
/// crate-root rename applied); otherwise the `use`-map / `crate`-relative resolution, then the
/// extern oracle over the type-shadow-excluded set, then the `pub use` re-export closure. `None`
/// for a genuinely unresolvable principal — a bare local/prelude name, a macro/glob/foreign
/// re-export — the stated resolver bound, never a silent pass of a *resolvable* operand.
fn resolve_principal(
    path: &syn::Path,
    uses: &UseMap,
    module: &str,
    externs: &HashSet<String>,
    externs_type: &HashSet<String>,
    extern_renames: &ExternRenameMap,
    reexports: &ReexportMap,
) -> Option<String> {
    let resolved = if path.leading_colon.is_some() {
        extern_verbatim_renamed(path, externs, extern_renames)
    } else {
        resolve_path(path, uses, module, BareFallback::Ignore)
            .or_else(|| extern_verbatim_renamed(path, externs_type, extern_renames))
    };
    resolved.map(|canonical| canonicalize_through_reexports(&canonical, reexports))
}

// --- Impl-trait-boundary (existential exposure): the reaction -----------------

/// Run the impl-trait boundaries against the Cargo workspace at `manifest_path`.
///
/// Mirrors [`check_dyn_trait`]: resolve each boundary's crate and module anchor, observe the
/// module's public-API **return** positions for written `impl Trait` (RPIT) nodes at any depth,
/// and react. An unresolvable crate or module (or an unreadable/unparseable source) is a
/// constitution error (exit 2), never a silent pass. The shell composes via [`check_all`].
pub fn check_impl_trait(boundaries: &[ImplTraitBoundary], manifest_path: &Path) -> Outcome {
    let metadata = match cargo_metadata(manifest_path) {
        Ok(metadata) => metadata,
        Err(err) => {
            return Outcome::ConstitutionError(unreadable_workspace_error(manifest_path, &err));
        }
    };
    let mut violations = Vec::new();
    for boundary in boundaries {
        if let Err(error) = check_impl_trait_boundary(&metadata, boundary, &mut violations) {
            return Outcome::ConstitutionError(error);
        }
    }
    if violations.is_empty() {
        Outcome::Clean
    } else {
        Outcome::Violations(Report::new(violations))
    }
}

fn check_impl_trait_boundary(
    metadata: &Value,
    boundary: &ImplTraitBoundary,
    violations: &mut Vec<Violation>,
) -> Result<(), String> {
    let (package, root_file, src_dir) = resolve_crate(metadata, &boundary.crate_package)?;
    let src_dir = src_dir.as_path();

    // Empty operand set ⇒ shape-only (any returned impl Trait), via the resolution-free path; a
    // named set ⇒ operand-scoped, resolving each returned impl Trait's principal trait.
    let findings = if boundary.forbidden_operands.is_empty() {
        impl_trait_module_findings(
            src_dir,
            &root_file,
            &boundary.module,
            &boundary.crate_package,
        )?
    } else {
        impl_trait_operand_module_findings(
            src_dir,
            &root_file,
            &boundary.module,
            &boundary.forbidden_operands,
            &boundary.crate_package,
            &dependency_names(package),
        )?
    };

    // The governed module's source file — where the exposing seam is written (see
    // `check_boundary`). Shared with the impl-trait shape/operand hearts' traversal.
    let module_file = seam_file(
        &findings,
        src_dir,
        &root_file,
        &boundary.module,
        &boundary.crate_package,
    )?;
    for finding in findings {
        violations.push(
            Violation::new(
                BoundaryKind::Semantic,
                boundary.module.clone(),
                IMPL_TRAIT_RULE.to_string(),
                finding,
                boundary.reason.clone(),
                boundary.severity,
            )
            .with_file(module_file.clone()),
        );
    }
    Ok(())
}

/// The pure heart of impl-trait-boundary, testable without spawning `cargo`: resolve the module's
/// items and return the sorted, deduplicated rendered `impl …` shapes appearing in a **return
/// position** of the module's public functions/methods. Shape-only, so no name resolution is
/// involved. Governs return positions only — argument-position `impl Trait` (APIT) is universal,
/// not existential, and is never visited; a trait-*impl* method's return is dictated by the trait
/// declaration (governed there), so it is excluded.
pub(crate) fn impl_trait_module_findings(
    src_dir: &Path,
    root_file: &Path,
    module: &str,
    crate_package: &str,
) -> Result<Vec<String>, String> {
    let items = resolve_module_items(src_dir, root_file, module, crate_package)?;
    // `uses` canonicalizes an inherent impl's self-type owner in the seam (see the dyn path).
    let uses = collect_uses(&items);
    let mut exposures = Vec::new();
    for (ordinal, item) in items.iter().enumerate() {
        collect_item_return_impl_traits(item, module, &uses, ordinal, &mut exposures);
    }
    let mut findings: Vec<String> = exposures.into_iter().map(shape_finding).collect();
    findings.sort();
    findings.dedup();
    Ok(findings)
}

/// The pure heart of the **operand-scoped** impl-trait boundary: like [`impl_trait_module_findings`]
/// but keeps only the returned `impl Trait` nodes whose **principal trait** resolves into the
/// forbidden operand set — the exact pipeline [`dyn_operand_module_findings`] uses
/// (`resolve_path(BareFallback::Ignore)` → `canonicalize_through_reexports` → `matches_forbidden`,
/// exact-or-module-prefix), so a re-exported/aliased trait facade matches its defining path. An
/// empty set ⇒ any returned `impl Trait` (never a silent no-op). An unresolvable principal (a bare
/// std trait, macro/glob re-export) is dropped — the stated resolver bound, never a silent pass of
/// a *resolvable* operand. The finding stays the rendered `impl …` shape (parity with shape-only).
pub(crate) fn impl_trait_operand_module_findings(
    src_dir: &Path,
    root_file: &Path,
    module: &str,
    forbidden: &[String],
    crate_package: &str,
    dep_names: &[String],
) -> Result<Vec<String>, String> {
    let items = resolve_module_items(src_dir, root_file, module, crate_package)?;
    let uses = collect_uses(&items);
    let ExternResolution {
        externs,
        externs_type,
        reexports,
        extern_renames,
    } = extern_resolution(src_dir, root_file, crate_package, dep_names, &items)?;
    let forbidden: Vec<String> = forbidden.iter().map(|f| canonical_path_str(f)).collect();

    let mut exposures = Vec::new();
    for (ordinal, item) in items.iter().enumerate() {
        collect_item_return_impl_traits(item, module, &uses, ordinal, &mut exposures);
    }

    let mut findings: Vec<String> = exposures
        .into_iter()
        .filter(|exposure| {
            forbidden.is_empty()
                || exposure
                    .principal
                    .as_ref()
                    .and_then(|path| {
                        resolve_principal(
                            path,
                            &uses,
                            module,
                            &externs,
                            &externs_type,
                            &extern_renames,
                            &reexports,
                        )
                    })
                    .is_some_and(|canonical| matches_forbidden(&canonical, &forbidden))
        })
        .map(shape_finding)
        .collect();
    findings.sort();
    findings.dedup();
    Ok(findings)
}

/// Collect the returned-`impl Trait` [`ShapeExposure`]s in the **return type** of a public item's
/// functions/methods only (the existential positions). Never visits argument positions (APIT is
/// universal, not a leak) nor trait-*impl* methods (their return shape is dictated by the trait).
fn collect_item_return_impl_traits(
    item: &syn::Item,
    module: &str,
    uses: &UseMap,
    ordinal: usize,
    out: &mut Vec<ShapeExposure>,
) {
    match item {
        syn::Item::Fn(item) if is_public(&item.vis) => {
            let seam = fn_seam(module, &item.sig.ident);
            out.extend(stamp_seam(impl_traits_in_return(&item.sig), &seam));
        }
        syn::Item::Trait(item) if is_public(&item.vis) => {
            // A trait method's return is part of the public trait API (trait items carry no
            // individual visibility); the trait DECLARES any RPIT here.
            let trait_name = strip_raw(&item.ident.to_string());
            for trait_item in &item.items {
                if let syn::TraitItem::Fn(method) = trait_item {
                    let seam = trait_method_seam(module, &trait_name, &method.sig.ident);
                    out.extend(stamp_seam(impl_traits_in_return(&method.sig), &seam));
                }
            }
        }
        syn::Item::Impl(item) if item.trait_.is_none() => {
            let owner = canonical_self_owner(&item.self_ty, uses, module, ordinal);
            for impl_item in &item.items {
                if let syn::ImplItem::Fn(method) = impl_item {
                    if is_public(&method.vis) {
                        let seam = inherent_method_seam(&owner, &method.sig.ident);
                        out.extend(stamp_seam(impl_traits_in_return(&method.sig), &seam));
                    }
                }
            }
        }
        _ => {}
    }
}

/// The returned-`impl Trait` [`ShapeExposure`]s in a signature's **return type** (at any depth).
/// Visits `sig.output` ONLY — never `sig.inputs`, so argument-position `impl Trait` (APIT) is
/// excluded.
fn impl_traits_in_return(sig: &syn::Signature) -> Vec<ShapeExposure> {
    let mut collector = ImplTraitCollector::default();
    if let syn::ReturnType::Type(_, ty) = &sig.output {
        collector.visit_type(ty);
    }
    collector.exposures
}

// --- Async-exposure-boundary (implicit existential): the reaction ------------

/// Run the async-exposure boundaries against the Cargo workspace at `manifest_path`.
///
/// Mirrors [`check_impl_trait`]: resolve each boundary's crate and module anchor, observe the
/// module's public-API `async fn` declarations, and react. An unresolvable crate or module (or an
/// unreadable/unparseable source) is a constitution error (exit 2). The shell composes via
/// [`check_all`].
pub fn check_async_exposure(boundaries: &[AsyncExposureBoundary], manifest_path: &Path) -> Outcome {
    let metadata = match cargo_metadata(manifest_path) {
        Ok(metadata) => metadata,
        Err(err) => {
            return Outcome::ConstitutionError(unreadable_workspace_error(manifest_path, &err));
        }
    };
    let mut violations = Vec::new();
    for boundary in boundaries {
        if let Err(error) = check_async_exposure_boundary(&metadata, boundary, &mut violations) {
            return Outcome::ConstitutionError(error);
        }
    }
    if violations.is_empty() {
        Outcome::Clean
    } else {
        Outcome::Violations(Report::new(violations))
    }
}

fn check_async_exposure_boundary(
    metadata: &Value,
    boundary: &AsyncExposureBoundary,
    violations: &mut Vec<Violation>,
) -> Result<(), String> {
    let (_package, root_file, src_dir) = resolve_crate(metadata, &boundary.crate_package)?;
    let src_dir = src_dir.as_path();

    let findings = async_exposure_module_findings(
        src_dir,
        &root_file,
        &boundary.module,
        &boundary.crate_package,
    )?;

    // The governed module's source file — where the exposing seam is written (see
    // `check_boundary`). Shared with `async_exposure_module_findings`' traversal.
    let module_file = seam_file(
        &findings,
        src_dir,
        &root_file,
        &boundary.module,
        &boundary.crate_package,
    )?;
    for finding in findings {
        violations.push(
            Violation::new(
                BoundaryKind::Semantic,
                boundary.module.clone(),
                ASYNC_EXPOSURE_RULE.to_string(),
                finding,
                boundary.reason.clone(),
                boundary.severity,
            )
            .with_file(module_file.clone()),
        );
    }
    Ok(())
}

/// The pure heart of async-exposure-boundary: resolve the module's items and return the sorted,
/// deduplicated **owner-qualified** identities of the public `async fn`s it declares — public free
/// fns, public inherent methods, and public trait method declarations (observed from
/// `sig.asyncness`). Trait-*impl* methods (asyncness dictated by the trait) and private items are
/// excluded. Shape-only: no name resolution, no return-type walk.
pub(crate) fn async_exposure_module_findings(
    src_dir: &Path,
    root_file: &Path,
    module: &str,
    crate_package: &str,
) -> Result<Vec<String>, String> {
    let items = resolve_module_items(src_dir, root_file, module, crate_package)?;
    // `uses` canonicalizes an inherent impl's self-type owner in the seam (see the dyn path).
    let uses = collect_uses(&items);
    let mut found = Vec::new();
    for (ordinal, item) in items.iter().enumerate() {
        collect_item_async_exposures(item, module, &uses, ordinal, &mut found);
    }
    found.sort();
    found.dedup();
    Ok(found)
}

fn collect_item_async_exposures(
    item: &syn::Item,
    module: &str,
    uses: &UseMap,
    ordinal: usize,
    out: &mut Vec<String>,
) {
    match item {
        syn::Item::Fn(item) if is_public(&item.vis) => {
            if item.sig.asyncness.is_some() {
                out.push(
                    SemanticFinding::AsyncFreeFn {
                        module: module.to_string(),
                        name: strip_raw(&item.sig.ident.to_string()),
                        tail: render_sig_tail(&item.sig),
                    }
                    .to_string(),
                );
            }
        }
        syn::Item::Trait(item) if is_public(&item.vis) => {
            let trait_name = strip_raw(&item.ident.to_string());
            for trait_item in &item.items {
                if let syn::TraitItem::Fn(method) = trait_item {
                    if method.sig.asyncness.is_some() {
                        out.push(
                            SemanticFinding::AsyncTraitMethod {
                                module: module.to_string(),
                                trait_name: trait_name.clone(),
                                name: strip_raw(&method.sig.ident.to_string()),
                                tail: render_sig_tail(&method.sig),
                            }
                            .to_string(),
                        );
                    }
                }
            }
        }
        syn::Item::Impl(item) if item.trait_.is_none() => {
            // Owner-qualify by the impl's canonical self type (via the shared `canonical_self_owner`,
            // as the other three collectors do) so `impl A`/`impl B` async methods of the same name
            // never collide under the (target, rule, finding) baseline (a false negative). Generics
            // stay distinct (`Foo<u8>` vs `Foo<u16>`); a self type with an unrenderable const-generic
            // expression is disambiguated by the impl's position, never collapsed.
            let owner = canonical_self_owner(&item.self_ty, uses, module, ordinal);
            for impl_item in &item.items {
                if let syn::ImplItem::Fn(method) = impl_item {
                    if is_public(&method.vis) && method.sig.asyncness.is_some() {
                        out.push(
                            SemanticFinding::AsyncInherentMethod {
                                owner: owner.clone(),
                                name: strip_raw(&method.sig.ident.to_string()),
                                tail: render_sig_tail(&method.sig),
                            }
                            .to_string(),
                        );
                    }
                }
            }
        }
        _ => {}
    }
}

/// Render a signature's `(params) -> ret` tail for an owner-qualified finding — for readability and
/// extra collision-margin, NOT to represent the implicit future. Params render each input's type
/// via [`type_to_string`] (a receiver as `self`/`&self`/`&mut self`); the return renders
/// `sig.output`'s written type (empty for `-> ()`); an unrenderable type contributes `_`.
fn render_sig_tail(sig: &syn::Signature) -> String {
    let params: Vec<String> = sig
        .inputs
        .iter()
        .map(|arg| match arg {
            syn::FnArg::Receiver(receiver) => {
                let reference = if receiver.reference.is_some() {
                    "&"
                } else {
                    ""
                };
                let mutability = if receiver.mutability.is_some() {
                    "mut "
                } else {
                    ""
                };
                format!("{reference}{mutability}self")
            }
            syn::FnArg::Typed(pat_type) => {
                type_to_string(&pat_type.ty).unwrap_or_else(|| "_".to_string())
            }
        })
        .collect();
    let ret = match &sig.output {
        syn::ReturnType::Type(_, ty) => {
            format!(
                " -> {}",
                type_to_string(ty).unwrap_or_else(|| "_".to_string())
            )
        }
        syn::ReturnType::Default => String::new(),
    };
    format!("({}){ret}", params.join(", "))
}

// --- Trait-impl-locality: the reaction ---------------------------------------

/// Run the trait-impl-locality boundaries against the Cargo workspace at `manifest_path`.
///
/// Mirrors [`check`]: resolve each boundary's crate and trait anchor, walk the crate for
/// `impl <Trait> for <Type>` sites, react to those of the anchored trait whose module
/// location is outside the allowed set, and return the outcome. An unresolvable crate or
/// trait anchor (or an unreadable/unparseable source) is a constitution error (exit 2),
/// never a silent pass.
pub fn check_trait_impl_locality(
    boundaries: &[TraitImplBoundary],
    manifest_path: &Path,
) -> Outcome {
    let metadata = match cargo_metadata(manifest_path) {
        Ok(metadata) => metadata,
        Err(err) => {
            return Outcome::ConstitutionError(unreadable_workspace_error(manifest_path, &err));
        }
    };
    let mut violations = Vec::new();
    for boundary in boundaries {
        if let Err(error) = check_trait_impl_boundary(&metadata, boundary, &mut violations) {
            return Outcome::ConstitutionError(error);
        }
    }
    if violations.is_empty() {
        Outcome::Clean
    } else {
        Outcome::Violations(Report::new(violations))
    }
}

fn check_trait_impl_boundary(
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
    // baseline (mirroring how `xuanji` excludes reason/severity from the violation id).
    let rule = TRAIT_IMPL_RULE.to_string();
    // Each finding carries the module its offending impl sits in; report that module's source
    // file (memoized per module). See `per_finding_file` for the `.ok()`-degrades-to-null rule.
    let mut file_cache: HashMap<String, Option<String>> = HashMap::new();
    for (finding, module) in findings {
        let file = per_finding_file(
            &module,
            src_dir,
            &root_file,
            &boundary.crate_package,
            &mut file_cache,
        );
        violations.push(
            Violation::new(
                BoundaryKind::Semantic,
                canonical_path_str(&boundary.trait_path),
                rule.clone(),
                finding,
                boundary.reason.clone(),
                boundary.severity,
            )
            .with_file(file),
        );
    }
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
) -> Result<Vec<(String, String)>, String> {
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
        // The finding identifies the offending impl by its module location and its implemented-for
        // type, canonicalized like the inherent-impl seam owner, so two misplaced impls in one
        // module stay distinct — even when the self type carries an unrenderable const-generic
        // expression (then disambiguated by the impl's position, never collapsed to a shared
        // location-only finding that would mask one). Stated label bound: a trait impl's self type
        // MAY be foreign (`impl LocalTrait for Box<Foo>`), which the module-relative
        // canonicalization over-qualifies (`crate::m::Box<…>`) — this is a stable identity label,
        // not a resolved-path claim; the actionable part (the module location) is exact.
        let owner = canonical_self_owner(&site.self_ty, &site.uses, &site.module, ordinal);
        // Pair the finding with the module the offending impl sits in, so the reaction layer can
        // report its source file. Dedup BY FINDING (below) keeps the count identical to before —
        // `file` is metadata, never a second identity key.
        findings.push((
            SemanticFinding::MisplacedImpl {
                module: site.module.clone(),
                owner,
            }
            .to_string(),
            site.module.clone(),
        ));
    }
    findings.sort();
    findings.dedup_by(|a, b| a.0 == b.0);
    Ok(findings)
}

/// One impl site observed in the crate: its enclosing module path, the written trait
/// path, the implemented-for type, and that module's `use`-map (for resolution).
struct ImplSite {
    module: String,
    trait_path: syn::Path,
    self_ty: syn::Type,
    uses: UseMap,
}

/// One type definition observed in the crate: its canonical path (`module::Name`), the module
/// it is defined in (for a forbidden-`derive` finding's source file), and the paths in its
/// `#[derive(...)]`/`#[cfg_attr(_, derive(...))]`.
struct TypeDef {
    canonical: String,
    module: String,
    derives: Vec<syn::Path>,
}

/// One crate-wide scan: the `pub use` re-export closure, the set of locally-defined trait
/// paths (for anchor verification), every trait-impl site, and every type definition.
struct CrateScan {
    reexports: ReexportMap,
    aliases: AliasMap,
    extern_renames: ExternRenameMap,
    trait_defs: HashSet<String>,
    impls: Vec<ImplSite>,
    type_defs: Vec<TypeDef>,
}

/// Collect crate-root `extern crate X as Y;` renames (`Y → X`) into `out`. Crate-root only: such a
/// rename binds `Y` crate-wide via the extern prelude, whereas a module-scoped `extern crate … as`
/// binds only locally (collecting it crate-wide would false-positive on a same-named head elsewhere
/// — a stated bound). `as _` / `X == Y` / `extern crate self as …` are no-ops.
fn collect_crate_root_extern_renames(items: &[syn::Item], out: &mut ExternRenameMap) {
    for item in items {
        if let syn::Item::ExternCrate(ec) = item {
            if let Some((_, rename)) = &ec.rename {
                let alias = strip_raw(&rename.to_string());
                let real = strip_raw(&ec.ident.to_string());
                if alias != "_" && alias != real && real != "self" {
                    out.insert(alias, real);
                }
            }
        }
    }
}

/// A bare single-segment alias target (`type X = Inner`) whose ident names a non-generic type
/// alias in the *current* module resolves to that alias's canonical key `{module}::{ident}`, so the
/// query fixpoint can follow a bare alias-of-an-alias chain (order-independent). `None` for a
/// leading-`::` / multi-segment / generic-argument-bearing path, or a name that is not a local
/// alias — leaving a bare non-alias target (a local struct, a std prelude type like `String`)
/// unresolved, matching the exposure query's `Ignore` policy for a bare non-alias head (no
/// mis-record, so no false positive even under a boundary forbidding the module's own path).
fn bare_local_alias_target(
    target: &syn::Path,
    module: &str,
    local_alias_names: &HashSet<String>,
) -> Option<String> {
    if target.leading_colon.is_some() || target.segments.len() != 1 {
        return None;
    }
    let seg = &target.segments[0];
    if !matches!(seg.arguments, syn::PathArguments::None) {
        return None;
    }
    let name = strip_raw(&seg.ident.to_string());
    local_alias_names
        .contains(&name)
        .then(|| format!("{module}::{name}"))
}

/// Walk the whole crate from its root, descending every file-based and inline module,
/// collecting re-exports, trait definitions, and trait-impl sites. This is a fresh
/// whole-crate traversal (the single-path [`descend`] does not fit a "nowhere except
/// here" property); it reuses only the leaf primitives and the shared resolver.
fn scan_crate(
    src_dir: &Path,
    root_file: &Path,
    crate_package: &str,
    externs: &HashSet<String>,
) -> Result<CrateScan, String> {
    let root = read_parse(root_file)?;
    let mut scan = CrateScan {
        reexports: ReexportMap::new(),
        aliases: AliasMap::new(),
        extern_renames: ExternRenameMap::new(),
        trait_defs: HashSet::new(),
        impls: Vec::new(),
        type_defs: Vec::new(),
    };
    // Pre-collect crate-root `extern crate X as Y;` renames BEFORE the walk, so the rename map is
    // complete before any alias-target or re-export-closure resolution — every source-order
    // (forward-reference) hazard is eliminated (an alias or re-export preceding the `extern crate`
    // in root source order still resolves). Renames are crate-root-only (they bind crate-wide via
    // the extern prelude; a module-scoped one is a stated bound), so one root scan suffices.
    collect_crate_root_extern_renames(&root.items, &mut scan.extern_renames);
    walk_module(
        root.items,
        "crate".to_string(),
        src_dir.to_path_buf(),
        crate_package,
        externs,
        &mut scan,
    )?;
    Ok(scan)
}

fn walk_module(
    items: Vec<syn::Item>,
    module: String,
    child_dir: PathBuf,
    crate_package: &str,
    externs: &HashSet<String>,
    scan: &mut CrateScan,
) -> Result<(), String> {
    let uses = collect_uses(&items);
    collect_reexports(
        &items,
        &module,
        externs,
        &scan.extern_renames,
        &mut scan.reexports,
    );
    // Alias targets resolve in the same per-module shadow as type positions: a bare head naming
    // a local child module (`mod serde` + `type X = serde::Foo`) is local, not the dependency.
    let externs_type: HashSet<String> = externs
        .difference(&local_type_namespace_names(&items))
        .cloned()
        .collect();
    // This module's own non-generic type-alias names — the only bare single-segment targets the
    // alias-collection ladder resolves against the current module (a bare intermediate in an
    // alias-of-an-alias chain, always same-module). Gating to these names keeps a bare non-alias
    // target (a local struct, or a std prelude type like `String`) from being mis-recorded as
    // `{module}::{name}` — which would false-positive under a boundary forbidding the module's own
    // path. Computed once here so the check is order-independent within the module.
    let local_alias_names: HashSet<String> = items
        .iter()
        .filter_map(|it| match it {
            syn::Item::Type(t) if t.generics.params.is_empty() => {
                Some(strip_raw(&t.ident.to_string()))
            }
            _ => None,
        })
        .collect();

    for item in &items {
        match item {
            syn::Item::Trait(trait_item) => {
                scan.trait_defs.insert(format!(
                    "{module}::{}",
                    strip_raw(&trait_item.ident.to_string())
                ));
            }
            // Trait impls only (`impl Trait for Type`); inherent impls carry no `trait_`.
            syn::Item::Impl(impl_item) if impl_item.trait_.is_some() => {
                let (_, trait_path, _) = impl_item.trait_.as_ref().expect("trait_ is Some");
                scan.impls.push(ImplSite {
                    module: module.clone(),
                    trait_path: trait_path.clone(),
                    self_ty: (*impl_item.self_ty).clone(),
                    uses: uses.clone(),
                });
            }
            syn::Item::Struct(i) => {
                push_type_def(&i.attrs, &i.ident, &module, scan)?;
            }
            syn::Item::Enum(i) => {
                push_type_def(&i.attrs, &i.ident, &module, scan)?;
            }
            syn::Item::Union(i) => {
                push_type_def(&i.attrs, &i.ident, &module, scan)?;
            }
            // A non-generic `type X = <nominal path>;` alias: record `{module}::X → target`
            // so the exposure pipeline can follow it to the defining path. The target-resolution
            // ladder is byte-identical to the query site's, so no resolvable target is dropped and
            // no local shadow is misread:
            //   0. a leading-`::` target — an unambiguous extern (raw set, with the crate-root
            //      rename applied), a HARD short-circuit, so `type X = ::serde::Value;` records the
            //      extern even under a local `mod serde`, and `type X = ::<rename>::Foo;` too;
            //   1. `resolve_path(Ignore)` — use-map / `crate`·`self`·`super`;
            //   2. `bare_local_alias_target` — a bare single-segment target naming one of THIS
            //      module's own type aliases recorded as `{module}::{name}` (its canonical alias-map
            //      key), tried BEFORE the extern oracle so a local alias shadows a same-named
            //      dependency (rustc's own resolution); the query-time `canonicalize_through_aliases`
            //      fixpoint then closes a *bare* alias-of-an-alias chain regardless of source order.
            //      Gated to local alias names, so a bare non-alias target (a local struct, a std
            //      prelude type like `String`) is never mis-recorded — no false positive;
            //   3. `extern_verbatim_renamed` — an extern head, incl. a crate-root `extern crate as`
            //      rename (the rename map is pre-collected, so this is order-independent).
            // A generic alias (`type X<T> = …`) or a complex target (`Vec<T>`, `&T`, a
            // tuple/`dyn`/`impl`) is skipped — a stated coverage bound, never a silent claim.
            syn::Item::Type(type_item) if type_item.generics.params.is_empty() => {
                if let Some(target) = alias_nominal_target(&type_item.ty) {
                    let alias = format!("{module}::{}", strip_raw(&type_item.ident.to_string()));
                    let resolved = if target.leading_colon.is_some() {
                        extern_verbatim_renamed(target, externs, &scan.extern_renames)
                    } else {
                        resolve_path(target, &uses, &module, BareFallback::Ignore)
                            .or_else(|| {
                                bare_local_alias_target(target, &module, &local_alias_names)
                            })
                            .or_else(|| {
                                extern_verbatim_renamed(target, &externs_type, &scan.extern_renames)
                            })
                    };
                    if let Some(resolved) = resolved {
                        if resolved != alias {
                            scan.aliases.insert(alias, resolved);
                        }
                    }
                }
            }
            _ => {}
        }
    }

    for item in items {
        if let syn::Item::Mod(module_item) = item {
            // A `#[path]`-remapped module is located off the conventional path; not
            // observed (a stated coverage bound), never a silent claim of cleanliness.
            if has_path_attr(&module_item.attrs) {
                continue;
            }
            let name = strip_raw(&module_item.ident.to_string());
            let child_module = format!("{module}::{name}");
            match module_item.content {
                // Inline `mod x { … }`: descend its lexical items; file-children under `x/`.
                Some((_, inner)) => {
                    walk_module(
                        inner,
                        child_module,
                        child_dir.join(&name),
                        crate_package,
                        externs,
                        scan,
                    )?;
                }
                // File `mod x;`: `<dir>/x.rs` or `<dir>/x/mod.rs`; children under `x/`.
                None => match locate_module_file(&child_dir, &name) {
                    Some(file) => {
                        let parsed = read_parse(&file)?;
                        walk_module(
                            parsed.items,
                            child_module,
                            child_dir.join(&name),
                            crate_package,
                            externs,
                            scan,
                        )?;
                    }
                    // A `#[cfg]`-gated module may legitimately have no source file when the
                    // feature is off (a standard optional-feature pattern) — a stated
                    // coverage bound, not a scan error. A non-cfg missing file is a real
                    // scan error: fail loud (exit 2), never a silent pass.
                    None => {
                        if !has_cfg_attr(&module_item.attrs) {
                            return Err(missing_module_file_error(&child_module, crate_package));
                        }
                    }
                },
            }
        }
    }
    Ok(())
}

fn has_path_attr(attrs: &[syn::Attribute]) -> bool {
    attrs.iter().any(|attr| attr.path().is_ident("path"))
}

/// Record a type definition with its derive paths into the scan.
fn push_type_def(
    attrs: &[syn::Attribute],
    ident: &syn::Ident,
    module: &str,
    scan: &mut CrateScan,
) -> Result<(), String> {
    let name = strip_raw(&ident.to_string());
    let derives = extract_derives(attrs)?;
    scan.type_defs.push(TypeDef {
        canonical: format!("{module}::{name}"),
        module: module.to_string(),
        derives,
    });
    Ok(())
}

/// Extract the derive paths from a type's `#[derive(...)]` and `#[cfg_attr(_, derive(...))]`
/// attributes (the latter read cfg-agnostically). A `derive` whose arguments fail to parse is
/// a scan error (exit 2) — "cannot judge" is never a silent skip.
fn extract_derives(attrs: &[syn::Attribute]) -> Result<Vec<syn::Path>, String> {
    let mut out = Vec::new();
    for attr in attrs {
        if attr.path().is_ident("derive") {
            out.extend(parse_derive_paths(&attr.meta)?);
        } else if attr.path().is_ident("cfg_attr") {
            let metas = attr
                .parse_args_with(meta_list_parser())
                .map_err(|e| format!("cannot parse #[cfg_attr(...)]: {e}"))?;
            extract_derives_from_cfg_metas(&metas, &mut out)?;
        }
    }
    Ok(out)
}

fn meta_list_parser() -> impl Parser<Output = syn::punctuated::Punctuated<syn::Meta, syn::Token![,]>>
{
    syn::punctuated::Punctuated::<syn::Meta, syn::Token![,]>::parse_terminated
}

/// Parse the comma-separated paths of a `derive(...)` meta-list (empty `#[derive]`/non-list
/// yields none).
fn parse_derive_paths(meta: &syn::Meta) -> Result<Vec<syn::Path>, String> {
    let parser = syn::punctuated::Punctuated::<syn::Path, syn::Token![,]>::parse_terminated;
    match meta {
        syn::Meta::List(list) => Ok(list
            .parse_args_with(parser)
            .map_err(|e| format!("cannot parse derive(...): {e}"))?
            .into_iter()
            .collect()),
        _ => Ok(Vec::new()),
    }
}

/// Extract derives from a `cfg_attr`'s metas: the first is the cfg predicate (skipped); the
/// rest are conditionally-applied attributes — a `derive(...)`, or a **nested** `cfg_attr(...)`
/// recursed into (so `#[cfg_attr(a, cfg_attr(b, derive(X)))]` still yields `X`).
fn extract_derives_from_cfg_metas(
    metas: &syn::punctuated::Punctuated<syn::Meta, syn::Token![,]>,
    out: &mut Vec<syn::Path>,
) -> Result<(), String> {
    for meta in metas.iter().skip(1) {
        if let syn::Meta::List(list) = meta {
            if list.path.is_ident("derive") {
                out.extend(parse_derive_paths(meta)?);
            } else if list.path.is_ident("cfg_attr") {
                let inner = list
                    .parse_args_with(meta_list_parser())
                    .map_err(|e| format!("cannot parse nested #[cfg_attr(...)]: {e}"))?;
                extract_derives_from_cfg_metas(&inner, out)?;
            }
        }
    }
    Ok(())
}

fn has_cfg_attr(attrs: &[syn::Attribute]) -> bool {
    attrs.iter().any(|attr| attr.path().is_ident("cfg"))
}

// --- Visibility boundary: the reaction ---------------------------------------

/// Run the visibility boundaries against the Cargo workspace at `manifest_path`.
///
/// Mirrors [`check`]: resolve each boundary's crate and module anchor, scan the module's
/// direct items for bare-`pub` declarations, and return the outcome. An unresolvable crate
/// or module (or an unreadable/unparseable source) is a constitution error (exit 2), never
/// a silent pass.
pub fn check_visibility(boundaries: &[VisibilityBoundary], manifest_path: &Path) -> Outcome {
    let metadata = match cargo_metadata(manifest_path) {
        Ok(metadata) => metadata,
        Err(err) => {
            return Outcome::ConstitutionError(unreadable_workspace_error(manifest_path, &err));
        }
    };
    let mut violations = Vec::new();
    for boundary in boundaries {
        if let Err(error) = check_visibility_boundary(&metadata, boundary, &mut violations) {
            return Outcome::ConstitutionError(error);
        }
    }
    if violations.is_empty() {
        Outcome::Clean
    } else {
        Outcome::Violations(Report::new(violations))
    }
}

fn check_visibility_boundary(
    metadata: &Value,
    boundary: &VisibilityBoundary,
    violations: &mut Vec<Violation>,
) -> Result<(), String> {
    let (_package, root_file, src_dir) = resolve_crate(metadata, &boundary.crate_package)?;
    let src_dir = src_dir.as_path();

    let findings = visibility_findings(
        src_dir,
        &root_file,
        &boundary.module,
        &boundary.crate_package,
    )?;

    // The governed module's source file — where the offending `pub` item is declared (see
    // `check_boundary`). Shared with `visibility_findings`' traversal.
    let module_file = seam_file(
        &findings,
        src_dir,
        &root_file,
        &boundary.module,
        &boundary.crate_package,
    )?;
    for finding in findings {
        violations.push(
            Violation::new(
                BoundaryKind::Semantic,
                boundary.module.clone(),
                VISIBILITY_RULE.to_string(),
                finding,
                boundary.reason.clone(),
                boundary.severity,
            )
            .with_file(module_file.clone()),
        );
    }
    Ok(())
}

/// The pure heart, testable without spawning `cargo`: resolve the module's direct items and
/// return the sorted, deduplicated descriptions of those declared bare-`pub`.
pub(crate) fn visibility_findings(
    src_dir: &Path,
    root_file: &Path,
    module: &str,
    crate_package: &str,
) -> Result<Vec<String>, String> {
    let items = resolve_module_items(src_dir, root_file, module, crate_package)?;
    let mut findings: Vec<String> = items.iter().filter_map(pub_item_description).collect();
    findings.sort();
    findings.dedup();
    Ok(findings)
}

/// Describe a direct item declared bare-`pub` (`Visibility::Public`) by kind and name, or
/// `None` for a non-`pub` item or one with no governed visibility. `pub use` (including a
/// glob) is observed as a raw `Item::Use`; attribute-derived public surface
/// (`#[macro_export]`, `#[no_mangle]`) carries no `pub` keyword and is out of scope.
fn pub_item_description(item: &syn::Item) -> Option<String> {
    match item {
        syn::Item::Fn(i) if is_public(&i.vis) => Some(format!("pub fn {}", i.sig.ident)),
        syn::Item::Struct(i) if is_public(&i.vis) => Some(format!("pub struct {}", i.ident)),
        syn::Item::Enum(i) if is_public(&i.vis) => Some(format!("pub enum {}", i.ident)),
        syn::Item::Union(i) if is_public(&i.vis) => Some(format!("pub union {}", i.ident)),
        syn::Item::Type(i) if is_public(&i.vis) => Some(format!("pub type {}", i.ident)),
        syn::Item::Const(i) if is_public(&i.vis) => Some(format!("pub const {}", i.ident)),
        syn::Item::Static(i) if is_public(&i.vis) => Some(format!("pub static {}", i.ident)),
        syn::Item::Trait(i) if is_public(&i.vis) => Some(format!("pub trait {}", i.ident)),
        syn::Item::TraitAlias(i) if is_public(&i.vis) => {
            Some(format!("pub trait {} (alias)", i.ident))
        }
        syn::Item::Mod(i) if is_public(&i.vis) => Some(format!("pub mod {}", i.ident)),
        syn::Item::ExternCrate(i) if is_public(&i.vis) => {
            Some(format!("pub extern crate {}", i.ident))
        }
        syn::Item::Use(i) if is_public(&i.vis) => Some(format!(
            "pub use {}{}",
            if i.leading_colon.is_some() { "::" } else { "" },
            use_tree_desc(&i.tree)
        )),
        // A `pub macro` (declarative macros 2.0) parses as `Item::Verbatim` with no readable
        // visibility field, and a `#[macro_export] macro_rules!` / `#[no_mangle]` symbol
        // carries no `pub` keyword — all out of this capability's syntactic scope (stated
        // bounds; the deferred attribute capability's domain).
        _ => None,
    }
}

/// Render a `use` tree to a stable description for a finding (`crate::db::Handle`,
/// `crate::db::*`, `a as b`, `{x, y}`), reusing path-segment joining — no `quote`.
fn use_tree_desc(tree: &syn::UseTree) -> String {
    match tree {
        syn::UseTree::Path(p) => {
            format!(
                "{}::{}",
                strip_raw(&p.ident.to_string()),
                use_tree_desc(&p.tree)
            )
        }
        syn::UseTree::Name(n) => strip_raw(&n.ident.to_string()),
        syn::UseTree::Rename(r) => format!(
            "{} as {}",
            strip_raw(&r.ident.to_string()),
            strip_raw(&r.rename.to_string())
        ),
        syn::UseTree::Glob(_) => "*".to_string(),
        syn::UseTree::Group(g) => {
            let inner: Vec<String> = g.items.iter().map(use_tree_desc).collect();
            format!("{{{}}}", inner.join(", "))
        }
    }
}

// --- Forbidden-marker: the reaction ------------------------------------------

/// Run the forbidden-marker boundaries against the Cargo workspace at `manifest_path`.
pub fn check_forbidden_marker(
    boundaries: &[ForbiddenMarkerBoundary],
    manifest_path: &Path,
) -> Outcome {
    let metadata = match cargo_metadata(manifest_path) {
        Ok(metadata) => metadata,
        Err(err) => {
            return Outcome::ConstitutionError(unreadable_workspace_error(manifest_path, &err));
        }
    };
    let mut violations = Vec::new();
    for boundary in boundaries {
        if let Err(error) = check_forbidden_marker_boundary(&metadata, boundary, &mut violations) {
            return Outcome::ConstitutionError(error);
        }
    }
    if violations.is_empty() {
        Outcome::Clean
    } else {
        Outcome::Violations(Report::new(violations))
    }
}

fn check_forbidden_marker_boundary(
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
    // an `impl`, the defining type's module for a `#[derive]`; report that module's source file
    // (memoized per module). See `per_finding_file` for the `.ok()`-degrades-to-null rule.
    let mut file_cache: HashMap<String, Option<String>> = HashMap::new();
    for (finding, module) in findings {
        let file = per_finding_file(
            &module,
            src_dir,
            &root_file,
            &boundary.crate_package,
            &mut file_cache,
        );
        violations.push(
            Violation::new(
                BoundaryKind::Semantic,
                boundary.module.clone(),
                FORBIDDEN_MARKER_RULE.to_string(),
                finding,
                boundary.reason.clone(),
                boundary.severity,
            )
            .with_file(file),
        );
    }
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
) -> Result<Vec<(String, String)>, String> {
    let scan = scan_crate(src_dir, root_file, crate_package, &HashSet::new())?;
    let subtree = canonical_path_str(subtree);

    let mut findings = Vec::new();
    for entry in forbidden {
        let entry_leaf = leaf_of(entry);

        // Derive form: a derive on a type defined under the subtree.
        for td in &scan.type_defs {
            if !under_subtree(&td.canonical, &subtree) {
                continue;
            }
            for derived in &td.derives {
                if path_leaf(derived) == entry_leaf {
                    // A derive sits in the defining type's module — its source file, not any
                    // impl site's.
                    findings.push((
                        SemanticFinding::ForbiddenDerive {
                            marker: entry.clone(),
                            canonical: td.canonical.clone(),
                        }
                        .to_string(),
                        td.module.clone(),
                    ));
                }
            }
        }

        // Impl form: `impl T for X` (anywhere) whose self-type is defined under the subtree.
        for site in &scan.impls {
            if path_leaf(&site.trait_path) != entry_leaf {
                continue;
            }
            let Some(self_canonical) = resolve_self_type(&site.self_ty, &site.uses, &site.module)
            else {
                continue; // self-type not placeable (glob/external/complex) — a stated bound
            };
            if under_subtree(&self_canonical, &subtree) {
                // A forbidden `impl` sits in the impl site's module.
                findings.push((
                    format!("impl {entry} for {self_canonical}"),
                    site.module.clone(),
                ));
            }
        }
    }
    findings.sort();
    // Dedup BY FINDING (keep the first module), so the count is identical to before — `file` is
    // metadata attached to a finding, never a second identity key.
    findings.dedup_by(|a, b| a.0 == b.0);
    Ok(findings)
}

/// Sibling-safe `::`-path containment: `path` equals `prefix` or sits strictly beneath it
/// (`crate::a` contains `crate::a::b`, never the sibling `crate::ab`). The single home of the
/// containment rule every capability's subtree/forbidden/allowed test shares, so no copy can
/// drift to a bare `starts_with` that would admit a sibling — a false positive on the allowed
/// side, a false negative on the forbidden side.
fn path_within(path: &str, prefix: &str) -> bool {
    path == prefix || path.starts_with(&format!("{prefix}::"))
}

/// A canonical path is under `subtree` — [`path_within`] read with subtree-containment naming at
/// the call site (`crate::a` contains `crate::a::b`, never the sibling `crate::ab`).
fn under_subtree(canonical: &str, subtree: &str) -> bool {
    path_within(canonical, subtree)
}

/// The leaf identifier of a `::`-delimited path string.
fn leaf_of(path: &str) -> &str {
    path.rsplit("::").next().unwrap_or(path)
}

/// The leaf identifier of a `syn::Path` (raw-canonicalized).
fn path_leaf(path: &syn::Path) -> String {
    path.segments
        .last()
        .map(|s| strip_raw(&s.ident.to_string()))
        .unwrap_or_default()
}

/// Resolve an `impl`'s self-type to the canonical path of its definition, or `None` when it
/// is not a placeable nominal path (a reference/tuple/complex shape — a stated bound). For a
/// `Type::Path` (incl. a generic `Wrapper<T>`, governed by the outer `Wrapper`), the leading
/// path resolves via the impl module's `use`s / current-module / re-exports.
fn resolve_self_type(self_ty: &syn::Type, uses: &UseMap, module: &str) -> Option<String> {
    match self_ty {
        syn::Type::Path(tp) => resolve_path(&tp.path, uses, module, BareFallback::CurrentModule),
        _ => None,
    }
}

// --- Module resolution -------------------------------------------------------

/// The crate's root source file (the `lib` target's `src_path`, else a `bin` target's),
/// observed from `cargo metadata`.
fn crate_root_file(package: &Value) -> Option<PathBuf> {
    let targets = package["targets"].as_array()?;
    let has_kind = |target: &Value, wanted: &str| {
        target["kind"]
            .as_array()
            .map(|kinds| kinds.iter().any(|k| k.as_str() == Some(wanted)))
            .unwrap_or(false)
    };
    let pick = targets
        .iter()
        .find(|t| has_kind(t, "lib"))
        .or_else(|| targets.iter().find(|t| has_kind(t, "bin")))?;
    pick["src_path"].as_str().map(PathBuf::from)
}

/// The path segments of a module relative to the crate root: `crate::domain::sub` →
/// `["domain", "sub"]`; `crate` → `[]`. A leading `crate` is stripped; canonicalized so a
/// raw-identifier segment (`r#type`) compares as its plain form.
fn module_segments(module: &str) -> Vec<String> {
    module
        .split("::")
        .map(strip_raw)
        .enumerate()
        .filter(|(i, seg)| !(*i == 0 && seg == "crate"))
        .map(|(_, seg)| seg)
        .filter(|seg| !seg.is_empty())
        .collect()
}

/// Resolve a module path to the items it owns, descending `mod` declarations from the crate
/// root (inline `mod x { … }` and file-based `mod x;` both). An unknown segment is a
/// constitution error; a declared-but-fileless module is a scan error — never a silent pass.
fn resolve_module_items(
    src_dir: &Path,
    root_file: &Path,
    module: &str,
    crate_package: &str,
) -> Result<Vec<syn::Item>, String> {
    resolve_module(src_dir, root_file, module, crate_package).map(|(items, _file)| items)
}

/// Resolve a module path to the **source file** its items live in — the crate root for `crate`
/// or an inline module, or the located `<name>.rs` / `<name>/mod.rs` for a file module. This is
/// the file a single-module semantic violation reports (`Violation::with_file`): the file the
/// reaction already descends to in order to observe the module's items, where the offending
/// seam is written (the finding names the canonicalized forbidden type, which may be *defined*
/// elsewhere). It shares [`resolve_module`]'s one traversal with [`resolve_module_items`], so
/// the reported file can never disagree with the items reacted on.
fn resolve_module_file(
    src_dir: &Path,
    root_file: &Path,
    module: &str,
    crate_package: &str,
) -> Result<PathBuf, String> {
    resolve_module(src_dir, root_file, module, crate_package).map(|(_items, file)| file)
}

/// The shared module resolution: the items a module owns **and** the file they live in, from
/// one descent. [`resolve_module_items`] and [`resolve_module_file`] each keep one half, so the
/// two views come from the same traversal and never drift (a `mod`-resolution divergence is the
/// false-negative class the project forbids).
fn resolve_module(
    src_dir: &Path,
    root_file: &Path,
    module: &str,
    crate_package: &str,
) -> Result<(Vec<syn::Item>, PathBuf), String> {
    let root = read_parse(root_file)?;
    let segments = module_segments(module);
    descend(
        root.items,
        src_dir.to_path_buf(),
        root_file.to_path_buf(),
        &segments,
        module,
        crate_package,
    )
}

fn descend(
    items: Vec<syn::Item>,
    child_dir: PathBuf,
    current_file: PathBuf,
    segments: &[String],
    module: &str,
    crate_package: &str,
) -> Result<(Vec<syn::Item>, PathBuf), String> {
    let Some(seg) = segments.first() else {
        return Ok((items, current_file));
    };
    for item in &items {
        if let syn::Item::Mod(module_item) = item {
            // A `#[path]`-remapped module is located off the conventional path; the
            // single-module resolver does not observe it (matching `walk_module`'s
            // crate-wide skip), so it falls through to a loud `unknown_module_error`
            // (exit 2) rather than governing a same-named stale conventional file — never
            // a silent claim of cleanliness over a file rustc does not compile.
            if has_path_attr(&module_item.attrs) {
                continue;
            }
            if strip_raw(&module_item.ident.to_string()) != *seg {
                continue;
            }
            match &module_item.content {
                // Inline `mod x { … }`: descend into the lexical items; the current file is
                // unchanged (an inline module's items live in the enclosing file). Its
                // file-children (if any) live under `<child_dir>/x/`.
                Some((_, inner)) => {
                    return descend(
                        inner.clone(),
                        child_dir.join(seg),
                        current_file,
                        &segments[1..],
                        module,
                        crate_package,
                    );
                }
                // File `mod x;`: `<child_dir>/x.rs` or `<child_dir>/x/mod.rs` becomes the current
                // file; x's children live under `<child_dir>/x/`.
                None => {
                    let file = locate_module_file(&child_dir, seg)
                        .ok_or_else(|| missing_module_file_error(module, crate_package))?;
                    let parsed = read_parse(&file)?;
                    return descend(
                        parsed.items,
                        child_dir.join(seg),
                        file,
                        &segments[1..],
                        module,
                        crate_package,
                    );
                }
            }
        }
    }
    Err(unknown_module_error(module, crate_package))
}

fn locate_module_file(child_dir: &Path, seg: &str) -> Option<PathBuf> {
    let flat = child_dir.join(format!("{seg}.rs"));
    if flat.is_file() {
        return Some(flat);
    }
    let nested = child_dir.join(seg).join("mod.rs");
    if nested.is_file() {
        return Some(nested);
    }
    None
}

fn read_parse(file: &Path) -> Result<syn::File, String> {
    let text = std::fs::read_to_string(file)
        .map_err(|err| unreadable_source_error(file, &err.to_string()))?;
    syn::parse_file(&text).map_err(|err| unparseable_source_error(file, &err.to_string()))
}

// --- Containment matching ----------------------------------------------------
//
// Name resolution (`collect_uses` / `resolve_path` / `canonical_path_str` / `strip_raw`)
// lives in the shared `resolve` module — see `resolve.rs`.

/// `::`-delimited containment: a canonical path is forbidden when it equals a forbidden
/// entry or sits beneath it (so `crate::infra` matches `crate::infra::db::Pool` but never
/// the sibling `crate::infrastructure`).
fn matches_forbidden(canonical: &str, forbidden: &[String]) -> bool {
    forbidden.iter().any(|entry| path_within(canonical, entry))
}

/// `::`-delimited containment at allowed-vs-location polarity: a module location is
/// allowed when it equals an allowed entry or sits beneath it (so `crate::commands`
/// allows `crate::commands::greet` but never the sibling `crate::commandeer`).
fn matches_allowed(location: &str, allowed: &[String]) -> bool {
    allowed.iter().any(|entry| path_within(location, entry))
}

// --- Exposure collection -----------------------------------------------------

fn is_public(vis: &syn::Visibility) -> bool {
    matches!(vis, syn::Visibility::Public(_))
}

fn paths_in_signature(sig: &syn::Signature) -> Vec<syn::Path> {
    let mut c = PathCollector::default();
    c.visit_signature(sig);
    c.paths
}

fn paths_in_type(ty: &syn::Type) -> Vec<syn::Path> {
    let mut c = PathCollector::default();
    c.visit_type(ty);
    c.paths
}

fn paths_in_generics(generics: &syn::Generics) -> Vec<syn::Path> {
    let mut c = PathCollector::default();
    c.visit_generics(generics);
    c.paths
}

fn dyns_in_signature(sig: &syn::Signature) -> Vec<ShapeExposure> {
    let mut c = DynCollector::default();
    c.visit_signature(sig);
    c.exposures
}

fn dyns_in_type(ty: &syn::Type) -> Vec<ShapeExposure> {
    let mut c = DynCollector::default();
    c.visit_type(ty);
    c.exposures
}

fn dyns_in_generics(generics: &syn::Generics) -> Vec<ShapeExposure> {
    let mut c = DynCollector::default();
    c.visit_generics(generics);
    c.exposures
}

/// One exposed type path (signature-coupling), tagged with the public **seam** it was exposed at
/// — the `syn::Path` counterpart of [`ShapeExposure`]'s `seam`. The seam becomes part of the
/// finding so two distinct seams exposing the *same* forbidden type never collapse to one
/// `(target, rule, finding)` baseline entry and mask a new leak (the one forbidden bug).
struct PathExposure {
    seam: String,
    path: syn::Path,
    /// A named public re-export (`pub use`) position vs. a signature/field/type position.
    /// A bare `pub use` head is an external crate by edition-2018+ grammar, so it is resolved
    /// against the raw external-crate set; a bare **type-position** head, by contrast, may be
    /// a local child module of the governed module, so it is resolved against the set with the
    /// module's own child modules excluded (the shadow) — the two need different oracle inputs.
    is_reexport: bool,
}

/// The finding vocabulary of the semantic dimension, rendered in one place.
///
/// A semantic violation's `finding` is the third component of its `(target, rule, finding)`
/// baseline identity, so every format literal that can become a `finding` lives here and only
/// here: a reviewer sees the whole vocabulary at once, and a new finding shape must add a variant
/// rather than sprout an inline `format!`. Behavior-preserving — each variant's `Display` renders
/// byte-identically to the inline format it replaced, and the `*_findings` functions still return
/// `Vec<String>` / `Vec<(String, String)>`, so baseline identity and the injectivity tests are
/// unchanged. Visibility findings are deliberately *not* here: they are a heterogeneous
/// `pub {kind} {name}` item descriptor, already cohesive in `pub_item_description`, not one
/// canonical relation line.
enum SemanticFinding {
    /// `{subject} exposed by {seam}` — signature-coupling and its re-export / trait-impl depths,
    /// plus the dyn-/impl-trait shapes (`subject` is a canonical type path or a `dyn …`/`impl …`
    /// shape; both render identically). The one exposure literal, formerly written twice
    /// (path pipeline + shape pipeline).
    Exposed { subject: String, seam: String },
    /// `{module} (impl for {owner})` — trait-impl-locality: a trait impl outside its allowed site.
    MisplacedImpl { module: String, owner: String },
    /// `derive {marker} on {canonical}` — forbidden-marker: a forbidden `#[derive]` on a type.
    ForbiddenDerive { marker: String, canonical: String },
    /// `async fn {module}::{name}{tail}` — a public free `async fn` (implicit-existential exposure).
    AsyncFreeFn {
        module: String,
        name: String,
        tail: String,
    },
    /// `async fn trait {module}::{trait_name}::{name}{tail}` — a public trait's `async fn` method.
    AsyncTraitMethod {
        module: String,
        trait_name: String,
        name: String,
        tail: String,
    },
    /// `async fn <{owner}>::{name}{tail}` — a public inherent `async fn` method, owner-qualified.
    AsyncInherentMethod {
        owner: String,
        name: String,
        tail: String,
    },
}

impl std::fmt::Display for SemanticFinding {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Exposed { subject, seam } => write!(f, "{subject} exposed by {seam}"),
            Self::MisplacedImpl { module, owner } => write!(f, "{module} (impl for {owner})"),
            Self::ForbiddenDerive { marker, canonical } => {
                write!(f, "derive {marker} on {canonical}")
            }
            Self::AsyncFreeFn { module, name, tail } => {
                write!(f, "async fn {module}::{name}{tail}")
            }
            Self::AsyncTraitMethod {
                module,
                trait_name,
                name,
                tail,
            } => write!(f, "async fn trait {module}::{trait_name}::{name}{tail}"),
            Self::AsyncInherentMethod { owner, name, tail } => {
                write!(f, "async fn <{owner}>::{name}{tail}")
            }
        }
    }
}

/// Render a shape exposure (`dyn …` / `impl …`) as its seam-qualified finding string — the
/// shape/existential analogue of signature-coupling's `{type} exposed by {seam}`. Two distinct
/// seams exposing the same shape stay distinct findings (the one forbidden bug), so a baselined
/// exposure never masks a new one at another seam.
fn shape_finding(exposure: ShapeExposure) -> String {
    SemanticFinding::Exposed {
        subject: exposure.shape,
        seam: exposure.seam,
    }
    .to_string()
}

/// Attach `seam` to every path a position-walker produced (the signature-coupling analogue of
/// [`resolve::stamp_seam`]).
fn tag_paths(paths: Vec<syn::Path>, seam: &str) -> Vec<PathExposure> {
    paths
        .into_iter()
        .map(|path| PathExposure {
            seam: seam.to_string(),
            path,
            is_reexport: false,
        })
        .collect()
}

// Seam labels — the public element an exposure lives at, in one vocabulary shared by all three
// 渾儀 exposure collectors (signature-coupling, dyn, impl-trait) and disjoint-by-prefix with
// async-exposure's `async fn …` identities, so no two element kinds ever render the same seam.
// A free fn is `fn {module}::name`; an inherent method `fn <{SelfTy}>::name` (owner-qualified
// like async, so `impl A`/`impl B` methods stay distinct); a trait method `fn trait
// {module}::Trait::name`. A named item (struct/enum/union/trait/type/const/static) is `{kind}
// {module}::name`; a field/variant is `{field|variant} {module}::Owner::name`; a trait associated
// item `{type|const} trait {module}::Trait::name`.

fn fn_seam(module: &str, name: &syn::Ident) -> String {
    format!("fn {module}::{}", strip_raw(&name.to_string()))
}

fn inherent_method_seam(owner: &str, name: &syn::Ident) -> String {
    format!("fn <{owner}>::{}", strip_raw(&name.to_string()))
}

fn trait_method_seam(module: &str, trait_name: &str, name: &syn::Ident) -> String {
    format!(
        "fn trait {module}::{trait_name}::{}",
        strip_raw(&name.to_string())
    )
}

fn item_seam(kind: &str, module: &str, name: &syn::Ident) -> String {
    format!("{kind} {module}::{}", strip_raw(&name.to_string()))
}

fn field_seam(kind: &str, module: &str, owner: &str, member: &str) -> String {
    format!("{kind} {module}::{owner}::{member}")
}

fn trait_assoc_seam(kind: &str, module: &str, trait_name: &str, name: &syn::Ident) -> String {
    format!(
        "{kind} trait {module}::{trait_name}::{}",
        strip_raw(&name.to_string())
    )
}

/// Render a field's member name — a named field's ident, or a tuple field's positional index.
fn member_label(index: usize, field: &syn::Field) -> String {
    match &field.ident {
        Some(ident) => strip_raw(&ident.to_string()),
        None => index.to_string(),
    }
}

/// Collect the type paths exposed by one item's public surface. Only `pub` items
/// contribute; `pub(crate)`/`pub(in …)`/private are internal, not exposed. Trait `impl`
/// blocks are skipped (out of scope — their shape is the trait's, not the impl site's).
fn collect_item_exposures(
    item: &syn::Item,
    module: &str,
    uses: &UseMap,
    ordinal: usize,
    out: &mut Vec<PathExposure>,
) {
    match item {
        syn::Item::Fn(item) if is_public(&item.vis) => {
            let seam = fn_seam(module, &item.sig.ident);
            out.extend(tag_paths(paths_in_signature(&item.sig), &seam));
        }
        syn::Item::Struct(item) if is_public(&item.vis) => {
            let name = strip_raw(&item.ident.to_string());
            out.extend(tag_paths(
                paths_in_generics(&item.generics),
                &item_seam("struct", module, &item.ident),
            ));
            for (index, field) in item.fields.iter().enumerate() {
                if is_public(&field.vis) {
                    let seam = field_seam("field", module, &name, &member_label(index, field));
                    out.extend(tag_paths(paths_in_type(&field.ty), &seam));
                }
            }
        }
        syn::Item::Enum(item) if is_public(&item.vis) => {
            let name = strip_raw(&item.ident.to_string());
            out.extend(tag_paths(
                paths_in_generics(&item.generics),
                &item_seam("enum", module, &item.ident),
            ));
            // Enum variants and their fields are as public as the enum itself. Each field
            // carries a per-member seam (`variant {Enum}::{Variant}::{index|name}`), mirroring
            // struct/union fields, so two forbidden fields of one variant stay distinct findings
            // — never collapsing to one `(target, rule, finding)` and masking a new leak.
            for variant in &item.variants {
                let owner = format!("{name}::{}", strip_raw(&variant.ident.to_string()));
                for (index, field) in variant.fields.iter().enumerate() {
                    let seam = field_seam("variant", module, &owner, &member_label(index, field));
                    out.extend(tag_paths(paths_in_type(&field.ty), &seam));
                }
            }
        }
        syn::Item::Union(item) if is_public(&item.vis) => {
            let name = strip_raw(&item.ident.to_string());
            out.extend(tag_paths(
                paths_in_generics(&item.generics),
                &item_seam("union", module, &item.ident),
            ));
            for (index, field) in item.fields.named.iter().enumerate() {
                if is_public(&field.vis) {
                    let seam = field_seam("field", module, &name, &member_label(index, field));
                    out.extend(tag_paths(paths_in_type(&field.ty), &seam));
                }
            }
        }
        syn::Item::Type(item) if is_public(&item.vis) => {
            let seam = item_seam("type", module, &item.ident);
            out.extend(tag_paths(paths_in_generics(&item.generics), &seam));
            out.extend(tag_paths(paths_in_type(&item.ty), &seam));
        }
        syn::Item::Const(item) if is_public(&item.vis) => {
            out.extend(tag_paths(
                paths_in_type(&item.ty),
                &item_seam("const", module, &item.ident),
            ));
        }
        syn::Item::Static(item) if is_public(&item.vis) => {
            out.extend(tag_paths(
                paths_in_type(&item.ty),
                &item_seam("static", module, &item.ident),
            ));
        }
        syn::Item::Trait(item) if is_public(&item.vis) => {
            let trait_name = strip_raw(&item.ident.to_string());
            let trait_seam = item_seam("trait", module, &item.ident);
            out.extend(tag_paths(paths_in_generics(&item.generics), &trait_seam));
            // Supertraits are part of the trait's public contract.
            for bound in &item.supertraits {
                if let syn::TypeParamBound::Trait(trait_bound) = bound {
                    out.push(PathExposure {
                        seam: trait_seam.clone(),
                        path: trait_bound.path.clone(),
                        is_reexport: false,
                    });
                }
            }
            for trait_item in &item.items {
                match trait_item {
                    syn::TraitItem::Fn(method) => {
                        let seam = trait_method_seam(module, &trait_name, &method.sig.ident);
                        out.extend(tag_paths(paths_in_signature(&method.sig), &seam));
                    }
                    syn::TraitItem::Type(assoc) => {
                        let seam = trait_assoc_seam("type", module, &trait_name, &assoc.ident);
                        for bound in &assoc.bounds {
                            if let syn::TypeParamBound::Trait(trait_bound) = bound {
                                out.push(PathExposure {
                                    seam: seam.clone(),
                                    path: trait_bound.path.clone(),
                                    is_reexport: false,
                                });
                            }
                        }
                    }
                    syn::TraitItem::Const(assoc) => {
                        let seam = trait_assoc_seam("const", module, &trait_name, &assoc.ident);
                        out.extend(tag_paths(paths_in_type(&assoc.ty), &seam));
                    }
                    _ => {}
                }
            }
        }
        // Inherent `impl Type { … }` (no trait): its `pub` methods are public API the module
        // authored. Trait impls (`impl Trait for Type`) carry `trait_` and are out of scope.
        syn::Item::Impl(item) if item.trait_.is_none() => {
            let owner = canonical_self_owner(&item.self_ty, uses, module, ordinal);
            for impl_item in &item.items {
                if let syn::ImplItem::Fn(method) = impl_item {
                    if is_public(&method.vis) {
                        let seam = inherent_method_seam(&owner, &method.sig.ident);
                        out.extend(tag_paths(paths_in_signature(&method.sig), &seam));
                    }
                }
            }
        }
        // A bare `pub use` republishes what it names on the module's public surface — the most
        // direct exposure (`semantic-reexport-exposure`). Restricted-visibility re-exports are
        // internal, like a private field. The walked path flows through the same resolve →
        // canonicalize → match pipeline as any exposed type.
        syn::Item::Use(item) if is_public(&item.vis) => {
            walk_reexport_tree(&item.tree, Vec::new(), module, out);
        }
        // A `pub extern crate X [as Y];` republishes the external crate root `X` on the module's
        // public surface — like `pub use ::X;`. The exposure names the **real** crate `X` (not the
        // `as`-rename), a bare extern head (raw external set, `is_reexport`). `extern crate self`
        // renames the current crate, not an external exposure.
        syn::Item::ExternCrate(item) if is_public(&item.vis) && item.ident != "self" => {
            let name = strip_raw(&item.ident.to_string());
            out.push(PathExposure {
                seam: format!("pub extern crate {name}"),
                path: syn::Path::from(item.ident.clone()),
                is_reexport: true,
            });
        }
        _ => {}
    }
}

/// Walk a `pub use` tree, pushing one [`PathExposure`] per re-exported leaf (and the root of a
/// glob), seam-qualified by the **exported** path so two aliases of the same forbidden type stay
/// distinct findings. Handles: named/renamed leaves; grouped re-exports (per leaf); a whole-module
/// re-export (`pub use crate::infra as fs` — the leaf path is a module, matched like any path); a
/// `self` group member (`{self, X}` — re-exports the prefix module, keyed by the prefix's final
/// segment, never the literal `self`); a glob (the root prefix, which reacts iff it resolves
/// in/under the forbidden set). `as _` binds no nameable path — a stated non-observed bound.
/// `self` group member and a renamed `self` both mean "the prefix module itself" — collapse to
/// the prefix, keyed by the prefix's final segment (or the alias). Recognised on the raw `Ident`
/// (`self` is a keyword and never a raw identifier, so a string compare is exact).
fn is_self_segment(ident: &syn::Ident) -> bool {
    ident == "self"
}
fn walk_reexport_tree(
    tree: &syn::UseTree,
    prefix: Vec<syn::Ident>,
    module: &str,
    out: &mut Vec<PathExposure>,
) {
    match tree {
        syn::UseTree::Path(path) => {
            let mut segs = prefix;
            segs.push(path.ident.clone());
            walk_reexport_tree(&path.tree, segs, module, out);
        }
        syn::UseTree::Name(name) => {
            if is_self_segment(&name.ident) {
                // `pub use crate::infra::{self, …}` re-exports the prefix module, bound under the
                // prefix's final segment (never the literal `self`).
                let exported = prefix.last().map(seg_name);
                push_reexport(&prefix, exported.as_deref(), module, out);
            } else {
                let exported = seg_name(&name.ident);
                let mut segs = prefix;
                segs.push(name.ident.clone());
                push_reexport(&segs, Some(&exported), module, out);
            }
        }
        syn::UseTree::Rename(rename) => {
            let alias = seg_name(&rename.rename);
            if alias == "_" {
                return; // `as _` binds no nameable path — a stated non-observed bound
            }
            if is_self_segment(&rename.ident) {
                // `pub use crate::infra::{self as fs}` — the prefix module, renamed.
                push_reexport(&prefix, Some(&alias), module, out);
            } else {
                let mut segs = prefix;
                segs.push(rename.ident.clone());
                push_reexport(&segs, Some(&alias), module, out);
            }
        }
        syn::UseTree::Glob(_) => {
            // The glob root: reacts iff it resolves in/under the forbidden set (the pipeline
            // decides). A sibling/ancestor root simply does not match — a stated glob bound.
            push_reexport(&prefix, Some("*"), module, out);
        }
        syn::UseTree::Group(group) => {
            for item in &group.items {
                walk_reexport_tree(item, prefix.clone(), module, out);
            }
        }
    }
}

/// A path segment's display name, raw-identifier prefix stripped (`r#type` → `type`), for the
/// human-facing exported name in the seam.
fn seg_name(ident: &syn::Ident) -> String {
    strip_raw(&ident.to_string())
}

/// Push a re-export exposure. The `syn::Path` is built **directly from the segment idents** (never
/// re-parsed from a string), so a raw-identifier segment (`pub use crate::r#type::X;`) is preserved
/// and matches correctly — `resolve_path`/`matches_forbidden` normalize raw idents downstream. The
/// seam is `pub use {module}::{exported}`. An empty segment list is skipped (a `self` under no
/// prefix cannot arise from a legal re-export).
fn push_reexport(
    segs: &[syn::Ident],
    exported: Option<&str>,
    module: &str,
    out: &mut Vec<PathExposure>,
) {
    let (Some(exported), false) = (exported, segs.is_empty()) else {
        return;
    };
    let segments = segs
        .iter()
        .map(|ident| syn::PathSegment {
            ident: ident.clone(),
            arguments: syn::PathArguments::None,
        })
        .collect();
    out.push(PathExposure {
        path: syn::Path {
            leading_colon: None,
            segments,
        },
        seam: format!("pub use {module}::{exported}"),
        is_reexport: true,
    });
}

/// The type paths in a signature's **return type** only (`sig.output`) — never `sig.inputs`.
/// A trait impl method's parameters/receiver are invariant with the trait declaration (not
/// refinable at the impl site), but its return MAY be refined (return-position `impl Trait` in
/// traits / async fn in traits), so a concretely-written return can expose an impl-site-authored
/// type. Observed without classifying refined-vs-dictated (that would need the possibly-foreign
/// trait definition — an essential gap), so a concrete return is observed either way.
fn paths_in_return(sig: &syn::Signature) -> Vec<syn::Path> {
    let mut c = PathCollector::default();
    if let syn::ReturnType::Type(_, ty) = &sig.output {
        c.visit_type(ty);
    }
    c.paths
}

/// The paths named across a set of trait-bounds — each bound's trait path *and* any type nested
/// in its generic arguments (`T: From<crate::infra::Secret>` yields both `From` and
/// `crate::infra::Secret`). Used for the impl-site `where` position.
fn paths_in_bounds(
    bounds: &syn::punctuated::Punctuated<syn::TypeParamBound, syn::token::Plus>,
) -> Vec<syn::Path> {
    let mut c = PathCollector::default();
    for bound in bounds {
        c.visit_type_param_bound(bound);
    }
    c.paths
}

/// Collect the type paths exposed by one **trait `impl` block**'s impl-site-authored positions
/// (`semantic-trait-impl-exposure`, opt-in). Only fires for `impl Trait for Type` (inherent impls
/// are `collect_item_exposures`'s job). The observed positions — each seam-qualified so two of them
/// exposing the same forbidden type stay distinct findings (the one forbidden bug) — are:
/// `trait-arg` (the trait ref's generic arguments, NOT the trait path itself: implementing a
/// forbidden *trait* is `must_not_acquire`/locality's concern), `self` (the Self type, bare and
/// nested), `assoc {name}` (associated type/value bindings), `where {bounded-type}` (the impl's own
/// generics + `where`-clause, keyed by the bounded type so two bounds never collapse), and
/// `method {name} return` (the written return type only — params/receiver are trait-dictated). The
/// pushed [`PathExposure`]s flow through the same resolve → canonicalize → match → `{type} exposed
/// by {seam}` pipeline as signature-coupling, with `BareFallback::Ignore` parity.
fn collect_trait_impl_exposures(
    item: &syn::Item,
    module: &str,
    uses: &UseMap,
    ordinal: usize,
    out: &mut Vec<PathExposure>,
) {
    let syn::Item::Impl(item) = item else { return };
    let Some((_, trait_path, _)) = &item.trait_ else {
        return; // inherent impl — governed by `collect_item_exposures`
    };
    // Seam prefix `impl {Trait} for {SelfTy}`. The Self label is canonicalized (parity with the
    // inherent-impl / locality seam owner); the trait label is the written path (a rendering-
    // granularity choice — its generic args distinguish `From<Vec<X>>` from `From<Box<X>>`).
    let trait_label = path_to_string(trait_path).unwrap_or_else(|| format!("trait_#{ordinal}"));
    let self_label = canonical_self_owner(&item.self_ty, uses, module, ordinal);
    let prefix = format!("impl {trait_label} for {self_label}");

    // 1. trait-arg — the trait ref's generic arguments (not the trait base path).
    if let Some(syn::PathArguments::AngleBracketed(args)) =
        trait_path.segments.last().map(|s| &s.arguments)
    {
        let seam = format!("{prefix} (trait-arg)");
        for arg in &args.args {
            match arg {
                syn::GenericArgument::Type(ty) => out.extend(tag_paths(paths_in_type(ty), &seam)),
                syn::GenericArgument::AssocType(at) => {
                    out.extend(tag_paths(paths_in_type(&at.ty), &seam))
                }
                _ => {}
            }
        }
    }

    // 2. self — the Self type, bare (`impl T for infra::Forbidden`) and nested
    //    (`impl T for Vec<infra::Forbidden>`). A bare `Self`/`Self::X` in a return (position 5)
    //    does not resolve and cannot double-fire here.
    out.extend(tag_paths(
        paths_in_type(&item.self_ty),
        &format!("{prefix} (self)"),
    ));

    // 4. where — impl generic-param bounds and the `where`-clause, keyed by the bounded type so
    //    two distinct bounds exposing the same type never collapse under the baseline.
    for param in &item.generics.params {
        match param {
            syn::GenericParam::Type(tp) => {
                let key = strip_raw(&tp.ident.to_string());
                let seam = format!("{prefix} (where {key})");
                out.extend(tag_paths(paths_in_bounds(&tp.bounds), &seam));
            }
            // A const-param's *type* annotation (`impl<const N: crate::infra::X>`) is impl-site-
            // authored — v1's `paths_in_generics` observes it, so the hand-rolled walk must too.
            syn::GenericParam::Const(cp) => {
                let key = strip_raw(&cp.ident.to_string());
                let seam = format!("{prefix} (where {key})");
                out.extend(tag_paths(paths_in_type(&cp.ty), &seam));
            }
            syn::GenericParam::Lifetime(_) => {}
        }
    }
    if let Some(where_clause) = &item.generics.where_clause {
        for predicate in &where_clause.predicates {
            if let syn::WherePredicate::Type(pt) = predicate {
                let key = type_to_string(&pt.bounded_ty).unwrap_or_else(|| "_".to_string());
                let seam = format!("{prefix} (where {key})");
                // Both sides are impl-site-authored: a forbidden type in the bounded (LHS) type
                // (`where crate::infra::X: Clone`) leaks as surely as one in the bound (RHS). v1's
                // `paths_in_generics` observes both; the hand-rolled walk must not lose that.
                out.extend(tag_paths(paths_in_type(&pt.bounded_ty), &seam));
                out.extend(tag_paths(paths_in_bounds(&pt.bounds), &seam));
            }
        }
    }

    for impl_item in &item.items {
        match impl_item {
            // 3. assoc {name} — associated type/value bindings authored in the impl. Both an
            //    associated `type X = …` and an associated `const X: … ` carry an impl-site type
            //    (parity with v1's trait-def walk, which observes both).
            syn::ImplItem::Type(assoc) => {
                let seam = format!("{prefix} (assoc {})", strip_raw(&assoc.ident.to_string()));
                out.extend(tag_paths(paths_in_type(&assoc.ty), &seam));
            }
            syn::ImplItem::Const(assoc) => {
                let seam = format!("{prefix} (assoc {})", strip_raw(&assoc.ident.to_string()));
                out.extend(tag_paths(paths_in_type(&assoc.ty), &seam));
            }
            // 5. method {name} return — the written return type only (never params/receiver).
            syn::ImplItem::Fn(method) => {
                let seam = format!(
                    "{prefix} (method {} return)",
                    strip_raw(&method.sig.ident.to_string())
                );
                out.extend(tag_paths(paths_in_return(&method.sig), &seam));
            }
            _ => {}
        }
    }
}

/// Collect the `dyn` trait-object shapes exposed by one item's public surface — the
/// dyn-shape complement of [`collect_item_exposures`], over the same governed positions.
/// Kept **deliberately parallel, not merged**: signature-coupling pushes bare supertrait /
/// associated-bound *paths* (whose collected paths a shared visitor would change), and this
/// walk additionally observes associated-type **defaults** (`type T = Box<dyn …>;`), a
/// position exposure-governance does not cover. A `dyn` cannot appear in a supertrait or a
/// `: Bound` (those are trait, not type, positions), so they are skipped here.
fn collect_item_dyn_exposures(
    item: &syn::Item,
    module: &str,
    uses: &UseMap,
    ordinal: usize,
    out: &mut Vec<ShapeExposure>,
) {
    match item {
        syn::Item::Fn(item) if is_public(&item.vis) => {
            let seam = fn_seam(module, &item.sig.ident);
            out.extend(stamp_seam(dyns_in_signature(&item.sig), &seam));
        }
        syn::Item::Struct(item) if is_public(&item.vis) => {
            let name = strip_raw(&item.ident.to_string());
            out.extend(stamp_seam(
                dyns_in_generics(&item.generics),
                &item_seam("struct", module, &item.ident),
            ));
            for (index, field) in item.fields.iter().enumerate() {
                if is_public(&field.vis) {
                    let seam = field_seam("field", module, &name, &member_label(index, field));
                    out.extend(stamp_seam(dyns_in_type(&field.ty), &seam));
                }
            }
        }
        syn::Item::Enum(item) if is_public(&item.vis) => {
            let name = strip_raw(&item.ident.to_string());
            out.extend(stamp_seam(
                dyns_in_generics(&item.generics),
                &item_seam("enum", module, &item.ident),
            ));
            // Enum variants and their fields are as public as the enum itself; per-member seam
            // for the same injectivity guarantee as the type-exposure collector above.
            for variant in &item.variants {
                let owner = format!("{name}::{}", strip_raw(&variant.ident.to_string()));
                for (index, field) in variant.fields.iter().enumerate() {
                    let seam = field_seam("variant", module, &owner, &member_label(index, field));
                    out.extend(stamp_seam(dyns_in_type(&field.ty), &seam));
                }
            }
        }
        syn::Item::Union(item) if is_public(&item.vis) => {
            let name = strip_raw(&item.ident.to_string());
            out.extend(stamp_seam(
                dyns_in_generics(&item.generics),
                &item_seam("union", module, &item.ident),
            ));
            for (index, field) in item.fields.named.iter().enumerate() {
                if is_public(&field.vis) {
                    let seam = field_seam("field", module, &name, &member_label(index, field));
                    out.extend(stamp_seam(dyns_in_type(&field.ty), &seam));
                }
            }
        }
        syn::Item::Type(item) if is_public(&item.vis) => {
            let seam = item_seam("type", module, &item.ident);
            out.extend(stamp_seam(dyns_in_generics(&item.generics), &seam));
            // A public type-alias target writing `dyn` is exposed at the alias item itself; a
            // public item that merely *names* this alias is not expanded (the resolver does
            // not expand `type` aliases — a stated bound).
            out.extend(stamp_seam(dyns_in_type(&item.ty), &seam));
        }
        syn::Item::Const(item) if is_public(&item.vis) => {
            out.extend(stamp_seam(
                dyns_in_type(&item.ty),
                &item_seam("const", module, &item.ident),
            ));
        }
        syn::Item::Static(item) if is_public(&item.vis) => {
            out.extend(stamp_seam(
                dyns_in_type(&item.ty),
                &item_seam("static", module, &item.ident),
            ));
        }
        syn::Item::Trait(item) if is_public(&item.vis) => {
            let trait_name = strip_raw(&item.ident.to_string());
            out.extend(stamp_seam(
                dyns_in_generics(&item.generics),
                &item_seam("trait", module, &item.ident),
            ));
            for trait_item in &item.items {
                match trait_item {
                    syn::TraitItem::Fn(method) => {
                        let seam = trait_method_seam(module, &trait_name, &method.sig.ident);
                        out.extend(stamp_seam(dyns_in_signature(&method.sig), &seam));
                    }
                    // The associated-type **default** (`type T = Box<dyn …>;`) is an exposed
                    // type position; the `: Bound`s are trait positions and cannot be `dyn`.
                    syn::TraitItem::Type(assoc) => {
                        if let Some((_, default)) = &assoc.default {
                            let seam = trait_assoc_seam("type", module, &trait_name, &assoc.ident);
                            out.extend(stamp_seam(dyns_in_type(default), &seam));
                        }
                    }
                    syn::TraitItem::Const(assoc) => {
                        let seam = trait_assoc_seam("const", module, &trait_name, &assoc.ident);
                        out.extend(stamp_seam(dyns_in_type(&assoc.ty), &seam));
                    }
                    _ => {}
                }
            }
        }
        syn::Item::Impl(item) if item.trait_.is_none() => {
            let owner = canonical_self_owner(&item.self_ty, uses, module, ordinal);
            for impl_item in &item.items {
                if let syn::ImplItem::Fn(method) = impl_item {
                    if is_public(&method.vis) {
                        let seam = inherent_method_seam(&owner, &method.sig.ident);
                        out.extend(stamp_seam(dyns_in_signature(&method.sig), &seam));
                    }
                }
            }
        }
        _ => {}
    }
}

// --- cargo metadata IO -------------------------------------------------------

fn cargo_metadata(manifest_path: &Path) -> Result<Value, String> {
    let output = Command::new("cargo")
        .args([
            "metadata",
            "--no-deps",
            "--format-version",
            "1",
            "--manifest-path",
        ])
        .arg(manifest_path)
        .output()
        .map_err(|err| err.to_string())?;
    if !output.status.success() {
        return Err(String::from_utf8_lossy(&output.stderr).trim().to_string());
    }
    serde_json::from_slice(&output.stdout).map_err(|err| err.to_string())
}

fn find_package<'a>(metadata: &'a Value, package: &str) -> Option<&'a Value> {
    metadata["packages"]
        .as_array()?
        .iter()
        .find(|candidate| candidate["name"].as_str() == Some(package))
}

#[cfg(test)]
mod tests;
