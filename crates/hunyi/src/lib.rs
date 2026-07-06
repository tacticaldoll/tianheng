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
use std::path::Path;

use serde_json::Value;

mod resolve;
use resolve::{
    BareFallback, apply_bare_alias_rename, apply_crate_root_rename, bare_local_alias,
    canonical_path_str, canonical_self_owner, canonicalize_through_aliases,
    canonicalize_through_reexports, collect_uses, extern_verbatim_renamed, renames_shadowed,
    resolve_path,
};

// The reaction model is the shared 璇璣 crate, re-exported so a consumer can stay on
// hunyi's surface; these names are also used internally below.
pub use xuanji::{
    Baseline, BoundaryKind, Outcome, Polarity, Report, Severity, Violation, ViolationId,
    apply_baseline,
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

mod metadata;
use metadata::cargo_metadata;

mod collect;
mod finding;
mod module_resolve;
mod scan;
use collect::{
    collect_item_async_exposures, collect_item_dyn_exposures, collect_item_exposures,
    collect_item_return_impl_traits, collect_trait_impl_exposures,
};
use finding::{SemanticFinding, shape_finding};
use module_resolve::resolve_module_items;
use scan::scan_crate;

mod containment;
mod crate_scope;
mod emit;
mod errors;
mod file_scope;
mod syn_util;
use containment::{
    leaf_of, matches_allowed, matches_forbidden, path_leaf, resolve_self_type, under_subtree,
};
use crate_scope::{
    child_module_names, dependency_names, extern_resolution, external_crate_set,
    local_type_namespace_names, resolve_principal,
};
use emit::{SingleModuleViolationContext, push_single_module_violations};
use errors::{unknown_trait_error, unreadable_workspace_error};
use file_scope::{per_finding_file, resolve_crate};
use syn_util::pub_item_description;

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

/// Fold accumulated violations into an outcome: `Clean` when none, else `Violations`.
///
/// Two boundaries of the same capability on the same module can emit an identical `ViolationId`
/// (`target, rule, finding`) — a plausible mid-promotion state (one `.warn()`, one enforce), or two
/// overlapping forbidden sets. Collapse them by id, keeping the **more severe** reaction (Enforce
/// dominates Warn), so one architectural fact is reported once and the baseline-suppressed count is
/// honest. Keeping the more severe is what stops a `warn` duplicate from masking an `enforce` one.
/// This mirrors the 圭表 static dimension's dedup; each dimension owns its copy (三儀 ⊥ 三儀).
fn outcome_from(violations: Vec<Violation>) -> Outcome {
    let mut deduped: Vec<Violation> = Vec::new();
    for violation in violations {
        match deduped.iter_mut().find(|kept| kept.id() == violation.id()) {
            Some(kept) => {
                if kept.severity == Severity::Warn && violation.severity == Severity::Enforce {
                    *kept = violation;
                }
            }
            None => deduped.push(violation),
        }
    }
    if deduped.is_empty() {
        Outcome::Clean
    } else {
        Outcome::Violations(Report::new(deduped))
    }
}

/// Read `cargo metadata` for the workspace, mapping an unreadable workspace to the shared
/// constitution error (exit 2) — the single-read gate every semantic reaction opens with.
fn read_metadata(manifest_path: &Path) -> Result<Value, Outcome> {
    cargo_metadata(manifest_path)
        .map_err(|err| Outcome::ConstitutionError(unreadable_workspace_error(manifest_path, &err)))
}

/// Evaluate one capability's boundaries against an already-read `metadata`, folding findings into
/// the shared `violations` accumulator; the first constitution error short-circuits (exit 2
/// supersedes any accumulated drift). Shared by the single-capability `check_*` drivers and
/// [`check_all`] — the latter reads `metadata` **once** and evaluates all seven capabilities into
/// one accumulator, so the single-read and error-supersedes semantics are identical across both.
fn eval_into<B>(
    metadata: &Value,
    boundaries: &[B],
    per_boundary: impl Fn(&Value, &B, &mut Vec<Violation>) -> Result<(), String>,
    violations: &mut Vec<Violation>,
) -> Result<(), String> {
    for boundary in boundaries {
        per_boundary(metadata, boundary, violations)?;
    }
    Ok(())
}

/// A single-capability reaction: one `cargo metadata` read, evaluate every boundary, react. The
/// spine every per-capability `check_*` entry shares — a constitution error supersedes (exit 2),
/// otherwise `Clean`/`Violations` (exit 0/1).
fn run_boundaries<B>(
    boundaries: &[B],
    manifest_path: &Path,
    per_boundary: impl Fn(&Value, &B, &mut Vec<Violation>) -> Result<(), String>,
) -> Outcome {
    let metadata = match read_metadata(manifest_path) {
        Ok(metadata) => metadata,
        Err(outcome) => return outcome,
    };
    let mut violations = Vec::new();
    match eval_into(&metadata, boundaries, per_boundary, &mut violations) {
        Ok(()) => outcome_from(violations),
        Err(error) => Outcome::ConstitutionError(error),
    }
}

/// Evaluate every declared semantic capability against `metadata` into the one accumulator, in a
/// fixed order; the first constitution error short-circuits. Split out so [`check_all`] keeps the
/// single-read + exit-2-supersedes contract with plain `?`, not seven repeated error blocks.
fn eval_all(
    metadata: &Value,
    boundaries: &SemanticBoundaries,
    violations: &mut Vec<Violation>,
) -> Result<(), String> {
    eval_into(metadata, &boundaries.signature, check_boundary, violations)?;
    eval_into(
        metadata,
        &boundaries.trait_impl,
        check_trait_impl_boundary,
        violations,
    )?;
    eval_into(
        metadata,
        &boundaries.visibility,
        check_visibility_boundary,
        violations,
    )?;
    eval_into(
        metadata,
        &boundaries.forbidden_marker,
        check_forbidden_marker_boundary,
        violations,
    )?;
    eval_into(
        metadata,
        &boundaries.dyn_trait,
        check_dyn_trait_boundary,
        violations,
    )?;
    eval_into(
        metadata,
        &boundaries.impl_trait,
        check_impl_trait_boundary,
        violations,
    )?;
    eval_into(
        metadata,
        &boundaries.async_exposure,
        check_async_exposure_boundary,
        violations,
    )?;
    Ok(())
}

/// Evaluate every declared semantic boundary against the workspace with a **single**
/// `cargo metadata` read, merging all findings into one outcome. A constitution error on any
/// boundary supersedes (exit 2). The per-capability `check`/`check_trait_impl_locality`/
/// `check_visibility` entries remain for direct use; the shell composes via this.
pub fn check_all(boundaries: &SemanticBoundaries, manifest_path: &Path) -> Outcome {
    let metadata = match read_metadata(manifest_path) {
        Ok(metadata) => metadata,
        Err(outcome) => return outcome,
    };
    let mut violations = Vec::new();
    match eval_all(&metadata, boundaries, &mut violations) {
        Ok(()) => outcome_from(violations),
        Err(error) => Outcome::ConstitutionError(error),
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
    run_boundaries(boundaries, manifest_path, check_boundary)
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

    push_single_module_violations(
        violations,
        SingleModuleViolationContext {
            src_dir,
            root_file: &root_file,
            module: &boundary.module,
            crate_package: &boundary.crate_package,
            rule: SIGNATURE_RULE,
            reason: &boundary.reason,
            severity: boundary.severity,
            anchor: boundary.anchor(),
        },
        findings,
    )
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
    // A bare `pub use HEAD::X;` head is shadowed by a same-named child `mod HEAD` of the
    // re-exporting module (rustc resolves it to the local module, not the dependency), so the
    // re-export head oracle subtracts the module's own child-module names — only modules, since a
    // `pub use` head must be a module/crate (see `child_module_names`). A crate-root module is not
    // in this module's `items`, so it does not suppress a child's re-export (no false negative).
    let child_mods = child_module_names(&items);
    let externs_reexport: HashSet<String> = externs.difference(&child_mods).cloned().collect();
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
    // A crate-root `extern crate X as Y;` binds `Y` crate-wide, but a governed submodule that
    // declares its own child `mod Y` shadows the alias there (rustc: bare `Y::…` is the local
    // module). So a **bare** head uses the rename map with this module's own child-module names
    // removed — suppressing the rewrite only where a local `mod Y` shadows it, while every
    // unshadowed module keeps the crate-wide rewrite (removing it there would be a false negative).
    // The crate-relative (`crate::Y::…`) and leading-`::` forms are NOT shadowable, so they keep
    // the full `extern_renames`.
    let renames_bare = renames_shadowed(&extern_renames, &child_mods);
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
            // re-export head uses the child-module-excluded set (a same-named child `mod` shadows a
            // bare `pub use` head); a type-position head uses the full type-namespace-excluded set.
            let type_externs = if exposure.is_reexport {
                &externs_reexport
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
                    // The bare-head extern-rename rewrite uses `renames_bare`: a `Y::…` head shadowed
                    // by this module's own child `mod Y` is not rewritten to the crate (rustc resolves
                    // it to the local module), while an unshadowed `Y::…` still rewrites (no FN).
                    .or_else(|| {
                        extern_verbatim_renamed(&exposure.path, type_externs, &renames_bare)
                    })
            };
            resolved
                .map(|canonical| canonicalize_through_aliases(&canonical, &aliases, &reexports))
                // Crate-relative spelling of a crate-root rename: `crate::Y::rest` → `X::rest`.
                // `crate::Y` unambiguously names the crate-root extern rename (a crate-root `mod Y`
                // cannot coexist with `extern crate … as Y`), so this is unconditional and uses the
                // full rename map; only the segment immediately after `crate` is treated as the alias.
                // Applied AFTER the alias/re-export closure so a `crate::Y::…` reached directly OR
                // through a `type` alias / `pub use` target (whose stored target keeps the verbatim
                // `crate::Y::…`) is rewritten alike — otherwise the aliased form is a residual FN.
                .map(|canonical| apply_crate_root_rename(canonical, &extern_renames))
                // Bare spelling of the same rename: a forbidden type imported by a private
                // `use Y::…;` resolves through the use-map to `Y::…` verbatim (unlike the direct
                // type-position form, which the extern oracle already rewrote), so rewrite a bare
                // alias head here too. Uses `renames_bare` — a head shadowed by a local `mod Y` is
                // left alone (rustc resolves bare `Y` to the local module there).
                .map(|canonical| apply_bare_alias_rename(canonical, &renames_bare))
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
    run_boundaries(boundaries, manifest_path, check_dyn_trait_boundary)
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

    push_single_module_violations(
        violations,
        SingleModuleViolationContext {
            src_dir,
            root_file: &root_file,
            module: &boundary.module,
            crate_package: &boundary.crate_package,
            rule: DYN_TRAIT_RULE,
            reason: &boundary.reason,
            severity: boundary.severity,
            anchor: boundary.anchor(),
        },
        findings,
    )
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
    let resolution = extern_resolution(src_dir, root_file, crate_package, dep_names, &items)?;
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
                    .and_then(|path| resolve_principal(path, &uses, module, &resolution))
                    .is_some_and(|canonical| matches_forbidden(&canonical, &forbidden))
        })
        .map(shape_finding)
        .collect();
    findings.sort();
    findings.dedup();
    Ok(findings)
}

// --- Impl-trait-boundary (existential exposure): the reaction -----------------

/// Run the impl-trait boundaries against the Cargo workspace at `manifest_path`.
///
/// Mirrors [`check_dyn_trait`]: resolve each boundary's crate and module anchor, observe the
/// module's public-API **return** positions for written `impl Trait` (RPIT) nodes at any depth,
/// and react. An unresolvable crate or module (or an unreadable/unparseable source) is a
/// constitution error (exit 2), never a silent pass. The shell composes via [`check_all`].
pub fn check_impl_trait(boundaries: &[ImplTraitBoundary], manifest_path: &Path) -> Outcome {
    run_boundaries(boundaries, manifest_path, check_impl_trait_boundary)
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

    push_single_module_violations(
        violations,
        SingleModuleViolationContext {
            src_dir,
            root_file: &root_file,
            module: &boundary.module,
            crate_package: &boundary.crate_package,
            rule: IMPL_TRAIT_RULE,
            reason: &boundary.reason,
            severity: boundary.severity,
            anchor: boundary.anchor(),
        },
        findings,
    )
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
    let resolution = extern_resolution(src_dir, root_file, crate_package, dep_names, &items)?;
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
                    .and_then(|path| resolve_principal(path, &uses, module, &resolution))
                    .is_some_and(|canonical| matches_forbidden(&canonical, &forbidden))
        })
        .map(shape_finding)
        .collect();
    findings.sort();
    findings.dedup();
    Ok(findings)
}

// --- Async-exposure-boundary (implicit existential): the reaction ------------

/// Run the async-exposure boundaries against the Cargo workspace at `manifest_path`.
///
/// Mirrors [`check_impl_trait`]: resolve each boundary's crate and module anchor, observe the
/// module's public-API `async fn` declarations, and react. An unresolvable crate or module (or an
/// unreadable/unparseable source) is a constitution error (exit 2). The shell composes via
/// [`check_all`].
pub fn check_async_exposure(boundaries: &[AsyncExposureBoundary], manifest_path: &Path) -> Outcome {
    run_boundaries(boundaries, manifest_path, check_async_exposure_boundary)
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

    push_single_module_violations(
        violations,
        SingleModuleViolationContext {
            src_dir,
            root_file: &root_file,
            module: &boundary.module,
            crate_package: &boundary.crate_package,
            rule: ASYNC_EXPOSURE_RULE,
            reason: &boundary.reason,
            severity: boundary.severity,
            anchor: boundary.anchor(),
        },
        findings,
    )
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
    run_boundaries(boundaries, manifest_path, check_trait_impl_boundary)
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
            .with_file(file)
            .with_anchor(boundary.anchor.clone())
            .with_polarity(Polarity::AllowlistGap),
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

// --- Visibility boundary: the reaction ---------------------------------------

/// Run the visibility boundaries against the Cargo workspace at `manifest_path`.
///
/// Mirrors [`check`]: resolve each boundary's crate and module anchor, scan the module's
/// direct items for bare-`pub` declarations, and return the outcome. An unresolvable crate
/// or module (or an unreadable/unparseable source) is a constitution error (exit 2), never
/// a silent pass.
pub fn check_visibility(boundaries: &[VisibilityBoundary], manifest_path: &Path) -> Outcome {
    run_boundaries(boundaries, manifest_path, check_visibility_boundary)
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

    push_single_module_violations(
        violations,
        SingleModuleViolationContext {
            src_dir,
            root_file: &root_file,
            module: &boundary.module,
            crate_package: &boundary.crate_package,
            rule: VISIBILITY_RULE,
            reason: &boundary.reason,
            severity: boundary.severity,
            anchor: boundary.anchor(),
        },
        findings,
    )
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

// --- Forbidden-marker: the reaction ------------------------------------------

/// Run the forbidden-marker boundaries against the Cargo workspace at `manifest_path`.
pub fn check_forbidden_marker(
    boundaries: &[ForbiddenMarkerBoundary],
    manifest_path: &Path,
) -> Outcome {
    run_boundaries(boundaries, manifest_path, check_forbidden_marker_boundary)
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
            .with_file(file)
            .with_anchor(boundary.anchor.clone())
            .with_polarity(Polarity::DenyBreach),
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

#[cfg(test)]
mod tests;
