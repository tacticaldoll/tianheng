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
//! items, and the `pub` methods of **inherent** `impl` blocks. Out of scope (stated bounds,
//! not silent passes): **trait** `impl` blocks (their shape is dictated by the trait, not
//! the impl site); a type reachable only through a **glob** import or a **macro**; and a
//! type knowable only through **inference** (a return-position `impl Trait` that *hides* a
//! concrete type, or an alias chain). Within the resolved scope there is no false negative:
//! a forbidden type that *is* resolvable always reacts.
//!
//! Govern by reaction, not instruction.

#![deny(missing_docs)]

use std::collections::HashSet;
use std::path::{Path, PathBuf};
use std::process::Command;

use serde_json::Value;
use syn::parse::Parser;
use syn::visit::Visit;

mod resolve;
use resolve::{
    BareFallback, PathCollector, ReexportMap, UseMap, canonical_path_str,
    canonicalize_through_reexports, collect_reexports, collect_uses, resolve_path, strip_raw,
    type_to_string,
};

// The reaction model is the shared 璇璣 crate, re-exported so a consumer can stay on
// hunyi's surface; these names are also used internally below.
pub use xuanji::{
    Baseline, BoundaryKind, Outcome, Report, Severity, Violation, ViolationId, apply_baseline,
};

// --- Declaration DSL ---------------------------------------------------------

/// A semantic boundary: the public API of a module must not **expose** any forbidden
/// type. Declared in Rust (the single source of truth), alongside — and composed with —
/// the static constitution at the gate. Each dimension owns its own declaration DSL and
/// expresses findings in the shared 璇璣 model; the shell merges them into one reaction.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SemanticBoundary {
    pub(crate) crate_package: String,
    pub(crate) module: String,
    pub(crate) forbidden: Vec<String>,
    pub(crate) reason: String,
    pub(crate) severity: Severity,
}

impl SemanticBoundary {
    /// Begin a semantic boundary in the crate named `package`.
    pub fn in_crate(package: &str) -> SemanticCrateDraft {
        SemanticCrateDraft {
            crate_package: package.to_string(),
        }
    }

    /// The crate this boundary governs.
    pub fn crate_package(&self) -> &str {
        &self.crate_package
    }

    /// The governed module path (e.g. `crate::domain`).
    pub fn module(&self) -> &str {
        &self.module
    }

    /// The forbidden type paths / module prefixes whose exposure is a violation.
    pub fn forbidden(&self) -> &[String] {
        &self.forbidden
    }

    /// The human-readable reason recorded with the boundary (the repair hint).
    pub fn reason(&self) -> &str {
        &self.reason
    }

    /// The boundary's severity (`enforce` or `warn`).
    pub fn severity(&self) -> Severity {
        self.severity
    }
}

/// A semantic boundary awaiting its module anchor.
pub struct SemanticCrateDraft {
    crate_package: String,
}

impl SemanticCrateDraft {
    /// Anchor the boundary to a module path within the crate (e.g. `crate::domain`).
    pub fn module(self, module: &str) -> SemanticModuleDraft {
        SemanticModuleDraft {
            crate_package: self.crate_package,
            module: module.to_string(),
        }
    }
}

/// A module-anchored boundary awaiting the forbidden set.
pub struct SemanticModuleDraft {
    crate_package: String,
    module: String,
}

impl SemanticModuleDraft {
    /// Forbid the module's public API from exposing the given type path or module prefix
    /// (`::`-delimited containment, so `crate::infra` also forbids `crate::infra::db::Pool`).
    pub fn must_not_expose(self, path: &str) -> SemanticBoundaryDraft {
        SemanticBoundaryDraft {
            crate_package: self.crate_package,
            module: self.module,
            forbidden: vec![path.to_string()],
            severity: Severity::Enforce,
        }
    }
}

/// A boundary awaiting severity (optional) and its reason.
pub struct SemanticBoundaryDraft {
    crate_package: String,
    module: String,
    forbidden: Vec<String>,
    severity: Severity,
}

impl SemanticBoundaryDraft {
    /// Also forbid exposing another type path / module prefix (a boundary MAY forbid more
    /// than one).
    pub fn and_not_expose(mut self, path: &str) -> Self {
        self.forbidden.push(path.to_string());
        self
    }

    /// Make this an advisory (`warn`) boundary: violations are reported but do not fail the
    /// reaction — the first rung of adoption.
    pub fn warn(mut self) -> Self {
        self.severity = Severity::Warn;
        self
    }

    /// Finish the boundary with its human-readable reason (the repair hint).
    pub fn because(self, reason: &str) -> SemanticBoundary {
        SemanticBoundary {
            crate_package: self.crate_package,
            module: self.module,
            forbidden: self.forbidden,
            reason: reason.to_string(),
            severity: self.severity,
        }
    }
}

// --- Trait-impl-locality declaration DSL -------------------------------------

/// A trait-impl-locality boundary: within a target crate, the named trait may be
/// implemented **only** inside the declared allowed module location(s). An
/// `impl <Trait> for <Type>` block outside them is a violation. Declared in Rust (the
/// single source of truth) and composed with the other dimensions at the gate. This
/// governs *impl locality* — the complement of exposure ([`SemanticBoundary`]) and of the
/// static import boundary. It governs only the target crate's own impl sites; it makes no
/// claim about downstream crates (that would be external trait sealing, an essential gap).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TraitImplBoundary {
    pub(crate) crate_package: String,
    pub(crate) trait_path: String,
    pub(crate) allowed_locations: Vec<String>,
    pub(crate) reason: String,
    pub(crate) severity: Severity,
}

impl TraitImplBoundary {
    /// Begin a trait-impl-locality boundary in the crate named `package`.
    pub fn in_crate(package: &str) -> TraitImplCrateDraft {
        TraitImplCrateDraft {
            crate_package: package.to_string(),
        }
    }

    /// The crate this boundary governs.
    pub fn crate_package(&self) -> &str {
        &self.crate_package
    }

    /// The governed trait's path (e.g. `crate::command::Command`).
    pub fn trait_(&self) -> &str {
        &self.trait_path
    }

    /// The allowed module-location prefixes where the trait MAY be implemented.
    pub fn allowed_locations(&self) -> &[String] {
        &self.allowed_locations
    }

    /// The human-readable reason recorded with the boundary (the repair hint).
    pub fn reason(&self) -> &str {
        &self.reason
    }

    /// The boundary's severity (`enforce` or `warn`).
    pub fn severity(&self) -> Severity {
        self.severity
    }
}

/// A trait-impl-locality boundary awaiting its trait anchor.
pub struct TraitImplCrateDraft {
    crate_package: String,
}

impl TraitImplCrateDraft {
    /// Anchor the boundary to a trait path within the crate (e.g. `crate::command::Command`).
    /// The anchor must resolve to a `trait` item defined in the crate (directly or via a
    /// local `pub use`); an unresolvable anchor is a constitution error (exit 2).
    pub fn trait_(self, trait_path: &str) -> TraitImplTraitDraft {
        TraitImplTraitDraft {
            crate_package: self.crate_package,
            trait_path: trait_path.to_string(),
        }
    }
}

/// A trait-anchored boundary awaiting its first allowed location.
pub struct TraitImplTraitDraft {
    crate_package: String,
    trait_path: String,
}

impl TraitImplTraitDraft {
    /// Allow the trait to be implemented under the given module path or prefix
    /// (`::`-delimited containment, so `crate::commands` also allows
    /// `crate::commands::greet`). Implementations outside the allowed location(s) react.
    pub fn only_implemented_in(self, location: &str) -> TraitImplBoundaryDraft {
        TraitImplBoundaryDraft {
            crate_package: self.crate_package,
            trait_path: self.trait_path,
            allowed_locations: vec![location.to_string()],
            severity: Severity::Enforce,
        }
    }
}

/// A boundary awaiting more allowed locations (optional), severity (optional), and reason.
pub struct TraitImplBoundaryDraft {
    crate_package: String,
    trait_path: String,
    allowed_locations: Vec<String>,
    severity: Severity,
}

impl TraitImplBoundaryDraft {
    /// Also allow the trait to be implemented under another module path / prefix (a
    /// boundary MAY allow more than one location).
    pub fn and_in(mut self, location: &str) -> Self {
        self.allowed_locations.push(location.to_string());
        self
    }

    /// Make this an advisory (`warn`) boundary: violations are reported but do not fail the
    /// reaction — the first rung of adoption.
    pub fn warn(mut self) -> Self {
        self.severity = Severity::Warn;
        self
    }

    /// Finish the boundary with its human-readable reason (the repair hint).
    pub fn because(self, reason: &str) -> TraitImplBoundary {
        TraitImplBoundary {
            crate_package: self.crate_package,
            trait_path: self.trait_path,
            allowed_locations: self.allowed_locations,
            reason: reason.to_string(),
            severity: self.severity,
        }
    }
}

// --- Visibility-boundary declaration DSL -------------------------------------

/// A visibility boundary: a governed module must not declare any bare-`pub` items —
/// a declared-visibility hygiene rule for an internal / impl-detail layer. The rule is
/// **syntactic** (the `pub` keyword on the module's own direct items), not crate-
/// reachability: `pub(crate)`/`pub(super)`/`pub(in …)`/private are allowed, and attribute-
/// derived public surface (`#[macro_export]`, `#[no_mangle]`) is out of scope (the deferred
/// attribute capability's domain). Declared in Rust and composed with the other dimensions
/// at the gate.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct VisibilityBoundary {
    pub(crate) crate_package: String,
    pub(crate) module: String,
    pub(crate) reason: String,
    pub(crate) severity: Severity,
}

impl VisibilityBoundary {
    /// Begin a visibility boundary in the crate named `package`.
    pub fn in_crate(package: &str) -> VisibilityCrateDraft {
        VisibilityCrateDraft {
            crate_package: package.to_string(),
        }
    }

    /// The crate this boundary governs.
    pub fn crate_package(&self) -> &str {
        &self.crate_package
    }

    /// The governed module path (e.g. `crate::internal`).
    pub fn module(&self) -> &str {
        &self.module
    }

    /// The human-readable reason recorded with the boundary (the repair hint).
    pub fn reason(&self) -> &str {
        &self.reason
    }

    /// The boundary's severity (`enforce` or `warn`).
    pub fn severity(&self) -> Severity {
        self.severity
    }
}

/// A visibility boundary awaiting its module anchor.
pub struct VisibilityCrateDraft {
    crate_package: String,
}

impl VisibilityCrateDraft {
    /// Anchor the boundary to a module path within the crate (e.g. `crate::internal`).
    pub fn module(self, module: &str) -> VisibilityModuleDraft {
        VisibilityModuleDraft {
            crate_package: self.crate_package,
            module: module.to_string(),
        }
    }
}

/// A module-anchored boundary awaiting the rule.
pub struct VisibilityModuleDraft {
    crate_package: String,
    module: String,
}

impl VisibilityModuleDraft {
    /// Forbid the module from declaring any bare-`pub` direct item.
    pub fn must_not_declare_pub(self) -> VisibilityBoundaryDraft {
        VisibilityBoundaryDraft {
            crate_package: self.crate_package,
            module: self.module,
            severity: Severity::Enforce,
        }
    }
}

/// A boundary awaiting severity (optional) and its reason.
pub struct VisibilityBoundaryDraft {
    crate_package: String,
    module: String,
    severity: Severity,
}

impl VisibilityBoundaryDraft {
    /// Make this an advisory (`warn`) boundary: violations are reported but do not fail the
    /// reaction — the first rung of adoption.
    pub fn warn(mut self) -> Self {
        self.severity = Severity::Warn;
        self
    }

    /// Finish the boundary with its human-readable reason (the repair hint).
    pub fn because(self, reason: &str) -> VisibilityBoundary {
        VisibilityBoundary {
            crate_package: self.crate_package,
            module: self.module,
            reason: reason.to_string(),
            severity: self.severity,
        }
    }
}

// --- Forbidden-marker declaration DSL ----------------------------------------

/// A forbidden-marker boundary: types **defined in a module subtree** must not acquire a
/// forbidden trait — by `#[derive(T)]` or a hand-written `impl T for <a subtree type>`.
/// Declared in Rust and composed at the gate. The complement to exposure, impl-locality, and
/// visibility; it delivers the "this layer is not `T`-able" intent.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ForbiddenMarkerBoundary {
    pub(crate) crate_package: String,
    pub(crate) module: String,
    pub(crate) forbidden: Vec<String>,
    pub(crate) reason: String,
    pub(crate) severity: Severity,
}

impl ForbiddenMarkerBoundary {
    /// Begin a forbidden-marker boundary in the crate named `package`.
    pub fn in_crate(package: &str) -> ForbiddenMarkerCrateDraft {
        ForbiddenMarkerCrateDraft {
            crate_package: package.to_string(),
        }
    }

    /// The crate this boundary governs.
    pub fn crate_package(&self) -> &str {
        &self.crate_package
    }

    /// The governed module-subtree prefix (e.g. `crate::domain`).
    pub fn module(&self) -> &str {
        &self.module
    }

    /// The forbidden trait paths/names.
    pub fn forbidden(&self) -> &[String] {
        &self.forbidden
    }

    /// The human-readable reason (the repair hint).
    pub fn reason(&self) -> &str {
        &self.reason
    }

    /// The boundary's severity.
    pub fn severity(&self) -> Severity {
        self.severity
    }
}

/// A forbidden-marker boundary awaiting its module-subtree anchor.
pub struct ForbiddenMarkerCrateDraft {
    crate_package: String,
}

impl ForbiddenMarkerCrateDraft {
    /// Anchor the boundary to a module-subtree prefix (e.g. `crate::domain`).
    pub fn module(self, module: &str) -> ForbiddenMarkerModuleDraft {
        ForbiddenMarkerModuleDraft {
            crate_package: self.crate_package,
            module: module.to_string(),
        }
    }
}

/// A subtree-anchored boundary awaiting its first forbidden trait.
pub struct ForbiddenMarkerModuleDraft {
    crate_package: String,
    module: String,
}

impl ForbiddenMarkerModuleDraft {
    /// Forbid the subtree's types from acquiring this trait (by name or path). Matching is
    /// by leaf identifier, so `Serialize`, `serde::Serialize`, and `serde_derive::Serialize`
    /// all match.
    pub fn must_not_acquire(self, trait_path: &str) -> ForbiddenMarkerBoundaryDraft {
        ForbiddenMarkerBoundaryDraft {
            crate_package: self.crate_package,
            module: self.module,
            forbidden: vec![trait_path.to_string()],
            severity: Severity::Enforce,
        }
    }
}

/// A boundary awaiting more forbidden traits (optional), severity (optional), and a reason.
pub struct ForbiddenMarkerBoundaryDraft {
    crate_package: String,
    module: String,
    forbidden: Vec<String>,
    severity: Severity,
}

impl ForbiddenMarkerBoundaryDraft {
    /// Also forbid acquiring another trait (a boundary MAY forbid more than one).
    pub fn and_not_acquire(mut self, trait_path: &str) -> Self {
        self.forbidden.push(trait_path.to_string());
        self
    }

    /// Make this an advisory (`warn`) boundary.
    pub fn warn(mut self) -> Self {
        self.severity = Severity::Warn;
        self
    }

    /// Finish the boundary with its human-readable reason (the repair hint).
    pub fn because(self, reason: &str) -> ForbiddenMarkerBoundary {
        ForbiddenMarkerBoundary {
            crate_package: self.crate_package,
            module: self.module,
            forbidden: self.forbidden,
            reason: reason.to_string(),
            severity: self.severity,
        }
    }
}

// --- Constitution-error messages ---------------------------------------------

fn unreadable_workspace_error(manifest_path: &Path, err: &str) -> String {
    format!(
        "cannot read target workspace at {}: {err}",
        manifest_path.display()
    )
}

fn crate_not_found_error(crate_package: &str) -> String {
    format!("target crate '{crate_package}' not found in the workspace")
}

fn missing_src_error(crate_package: &str) -> String {
    format!("cannot locate the crate root source for '{crate_package}'")
}

fn unknown_module_error(module: &str, crate_package: &str) -> String {
    format!(
        "module '{module}' not found among the modules of crate '{crate_package}' (declared via `mod`)"
    )
}

fn unknown_trait_error(trait_path: &str, crate_package: &str) -> String {
    format!(
        "trait '{trait_path}' not found as a `trait` item (directly or via a local `pub use`) in crate '{crate_package}'"
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
}

impl SemanticBoundaries {
    /// Whether no semantic boundary of any kind is declared.
    pub fn is_empty(&self) -> bool {
        self.signature.is_empty()
            && self.trait_impl.is_empty()
            && self.visibility.is_empty()
            && self.forbidden_marker.is_empty()
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

fn check_boundary(
    metadata: &Value,
    boundary: &SemanticBoundary,
    violations: &mut Vec<Violation>,
) -> Result<(), String> {
    let package = find_package(metadata, &boundary.crate_package)
        .ok_or_else(|| crate_not_found_error(&boundary.crate_package))?;
    let root_file =
        crate_root_file(package).ok_or_else(|| missing_src_error(&boundary.crate_package))?;
    let src_dir = root_file
        .parent()
        .ok_or_else(|| missing_src_error(&boundary.crate_package))?;

    let findings = module_findings(
        src_dir,
        &root_file,
        &boundary.module,
        &boundary.forbidden,
        &boundary.crate_package,
    )?;

    for finding in findings {
        // No `with_file`: the 渾儀 dimension does not yet observe a per-element source file.
        // `finding` is a canonical type path (resolved through re-export chains), so the
        // offending element may live in a different file than the governed module's, and the
        // construction scope holds only the crate root, not that file. Surfacing it would need
        // new per-finding tracking — a stated bound (born when built), the same for every
        // semantic capability (exposure, trait-impl-locality, visibility, forbidden-marker).
        // So a semantic violation's `file` is a faithful `None`.
        violations.push(Violation::new(
            BoundaryKind::Semantic,
            boundary.module.clone(),
            "must not expose".to_string(),
            finding,
            boundary.reason.clone(),
            boundary.severity,
        ));
    }
    Ok(())
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
) -> Result<Vec<String>, String> {
    let items = resolve_module_items(src_dir, root_file, module, crate_package)?;
    let uses = collect_uses(&items);
    // The re-export closure is crate-wide: a forbidden type exposed through a `pub use`
    // facade must canonicalize to its defining path before matching (closes the
    // re-export false negative). Exposure-governance keeps `BareFallback::Ignore` — a bare
    // local name is not the cross-module forbidden type, and resolving it could match a
    // boundary that forbids the module's own path.
    let reexports = scan_crate(src_dir, root_file, crate_package)?.reexports;
    let forbidden: Vec<String> = forbidden.iter().map(|f| canonical_path_str(f)).collect();

    let mut exposed = Vec::new();
    for item in &items {
        collect_item_exposures(item, &mut exposed);
    }

    let mut findings: Vec<String> = exposed
        .iter()
        .filter_map(|path| resolve_path(path, &uses, module, BareFallback::Ignore))
        .map(|canonical| canonicalize_through_reexports(&canonical, &reexports))
        .filter(|canonical| matches_forbidden(canonical, &forbidden))
        .collect();
    findings.sort();
    findings.dedup();
    Ok(findings)
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
    let package = find_package(metadata, &boundary.crate_package)
        .ok_or_else(|| crate_not_found_error(&boundary.crate_package))?;
    let root_file =
        crate_root_file(package).ok_or_else(|| missing_src_error(&boundary.crate_package))?;
    let src_dir = root_file
        .parent()
        .ok_or_else(|| missing_src_error(&boundary.crate_package))?;

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
    let rule = "must only be implemented in the declared location(s)".to_string();
    for finding in findings {
        violations.push(Violation::new(
            BoundaryKind::Semantic,
            canonical_path_str(&boundary.trait_path),
            rule.clone(),
            finding,
            boundary.reason.clone(),
            boundary.severity,
        ));
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
) -> Result<Vec<String>, String> {
    let scan = scan_crate(src_dir, root_file, crate_package)?;
    let given = canonical_path_str(trait_path);
    let true_anchor = canonicalize_through_reexports(&given, &scan.reexports);
    if !scan.trait_defs.contains(&true_anchor) {
        return Err(unknown_trait_error(trait_path, crate_package));
    }
    let allowed: Vec<String> = allowed.iter().map(|a| canonical_path_str(a)).collect();

    let mut findings = Vec::new();
    for site in &scan.impls {
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
        // The finding identifies the offending impl by its module location and the
        // implemented-for type, so two misplaced impls in one module stay distinct. A
        // self-type the hand-rolled renderer cannot express falls back to location-only.
        let finding = match type_to_string(&site.self_ty) {
            Some(self_ty) => format!("{} (impl for {self_ty})", site.module),
            None => site.module.clone(),
        };
        findings.push(finding);
    }
    findings.sort();
    findings.dedup();
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

/// One type definition observed in the crate: its canonical path (`module::Name`), leaf
/// name (for findings), the paths in its `#[derive(...)]`/`#[cfg_attr(_, derive(...))]`, and
/// its module's `use`-map (for resolving path-qualified derive entries).
struct TypeDef {
    canonical: String,
    derives: Vec<syn::Path>,
}

/// One crate-wide scan: the `pub use` re-export closure, the set of locally-defined trait
/// paths (for anchor verification), every trait-impl site, and every type definition.
struct CrateScan {
    reexports: ReexportMap,
    trait_defs: HashSet<String>,
    impls: Vec<ImplSite>,
    type_defs: Vec<TypeDef>,
}

/// Walk the whole crate from its root, descending every file-based and inline module,
/// collecting re-exports, trait definitions, and trait-impl sites. This is a fresh
/// whole-crate traversal (the single-path [`descend`] does not fit a "nowhere except
/// here" property); it reuses only the leaf primitives and the shared resolver.
fn scan_crate(src_dir: &Path, root_file: &Path, crate_package: &str) -> Result<CrateScan, String> {
    let root = read_parse(root_file)?;
    let mut scan = CrateScan {
        reexports: ReexportMap::new(),
        trait_defs: HashSet::new(),
        impls: Vec::new(),
        type_defs: Vec::new(),
    };
    walk_module(
        root.items,
        "crate".to_string(),
        src_dir.to_path_buf(),
        crate_package,
        &mut scan,
    )?;
    Ok(scan)
}

fn walk_module(
    items: Vec<syn::Item>,
    module: String,
    child_dir: PathBuf,
    crate_package: &str,
    scan: &mut CrateScan,
) -> Result<(), String> {
    let uses = collect_uses(&items);
    collect_reexports(&items, &module, &mut scan.reexports);

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
    let package = find_package(metadata, &boundary.crate_package)
        .ok_or_else(|| crate_not_found_error(&boundary.crate_package))?;
    let root_file =
        crate_root_file(package).ok_or_else(|| missing_src_error(&boundary.crate_package))?;
    let src_dir = root_file
        .parent()
        .ok_or_else(|| missing_src_error(&boundary.crate_package))?;

    let findings = visibility_findings(
        src_dir,
        &root_file,
        &boundary.module,
        &boundary.crate_package,
    )?;

    for finding in findings {
        violations.push(Violation::new(
            BoundaryKind::Semantic,
            boundary.module.clone(),
            "must not declare pub items".to_string(),
            finding,
            boundary.reason.clone(),
            boundary.severity,
        ));
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
    let package = find_package(metadata, &boundary.crate_package)
        .ok_or_else(|| crate_not_found_error(&boundary.crate_package))?;
    let root_file =
        crate_root_file(package).ok_or_else(|| missing_src_error(&boundary.crate_package))?;
    let src_dir = root_file
        .parent()
        .ok_or_else(|| missing_src_error(&boundary.crate_package))?;

    let findings = forbidden_marker_findings(
        src_dir,
        &root_file,
        &boundary.module,
        &boundary.forbidden,
        &boundary.crate_package,
    )?;

    for finding in findings {
        violations.push(Violation::new(
            BoundaryKind::Semantic,
            boundary.module.clone(),
            "must not acquire trait".to_string(),
            finding,
            boundary.reason.clone(),
            boundary.severity,
        ));
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
) -> Result<Vec<String>, String> {
    let scan = scan_crate(src_dir, root_file, crate_package)?;
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
                    findings.push(format!("derive {entry} on {}", td.canonical));
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
                findings.push(format!("impl {entry} for {self_canonical}"));
            }
        }
    }
    findings.sort();
    findings.dedup();
    Ok(findings)
}

/// `::`-delimited subtree containment: a type's canonical path is under the subtree when it
/// equals the prefix or sits beneath it (sibling-safe).
fn under_subtree(canonical: &str, subtree: &str) -> bool {
    canonical == subtree || canonical.starts_with(&format!("{subtree}::"))
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
    let root = read_parse(root_file)?;
    let segments = module_segments(module);
    descend(
        root.items,
        src_dir.to_path_buf(),
        &segments,
        module,
        crate_package,
    )
}

fn descend(
    items: Vec<syn::Item>,
    child_dir: PathBuf,
    segments: &[String],
    module: &str,
    crate_package: &str,
) -> Result<Vec<syn::Item>, String> {
    let Some(seg) = segments.first() else {
        return Ok(items);
    };
    for item in &items {
        if let syn::Item::Mod(module_item) = item {
            if strip_raw(&module_item.ident.to_string()) != *seg {
                continue;
            }
            match &module_item.content {
                // Inline `mod x { … }`: descend into the lexical items; its file-children
                // (if any) live under `<child_dir>/x/`.
                Some((_, inner)) => {
                    return descend(
                        inner.clone(),
                        child_dir.join(seg),
                        &segments[1..],
                        module,
                        crate_package,
                    );
                }
                // File `mod x;`: `<child_dir>/x.rs` or `<child_dir>/x/mod.rs`; x's children
                // live under `<child_dir>/x/`.
                None => {
                    let file = locate_module_file(&child_dir, seg)
                        .ok_or_else(|| missing_module_file_error(module, crate_package))?;
                    let parsed = read_parse(&file)?;
                    return descend(
                        parsed.items,
                        child_dir.join(seg),
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
    forbidden
        .iter()
        .any(|entry| canonical == entry || canonical.starts_with(&format!("{entry}::")))
}

/// `::`-delimited containment at allowed-vs-location polarity: a module location is
/// allowed when it equals an allowed entry or sits beneath it (so `crate::commands`
/// allows `crate::commands::greet` but never the sibling `crate::commandeer`).
fn matches_allowed(location: &str, allowed: &[String]) -> bool {
    allowed
        .iter()
        .any(|entry| location == entry || location.starts_with(&format!("{entry}::")))
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

/// Collect the type paths exposed by one item's public surface. Only `pub` items
/// contribute; `pub(crate)`/`pub(in …)`/private are internal, not exposed. Trait `impl`
/// blocks are skipped (out of scope — their shape is the trait's, not the impl site's).
fn collect_item_exposures(item: &syn::Item, out: &mut Vec<syn::Path>) {
    match item {
        syn::Item::Fn(item) if is_public(&item.vis) => {
            out.extend(paths_in_signature(&item.sig));
        }
        syn::Item::Struct(item) if is_public(&item.vis) => {
            out.extend(paths_in_generics(&item.generics));
            for field in &item.fields {
                if is_public(&field.vis) {
                    out.extend(paths_in_type(&field.ty));
                }
            }
        }
        syn::Item::Enum(item) if is_public(&item.vis) => {
            out.extend(paths_in_generics(&item.generics));
            // Enum variants and their fields are as public as the enum itself.
            for variant in &item.variants {
                for field in &variant.fields {
                    out.extend(paths_in_type(&field.ty));
                }
            }
        }
        syn::Item::Union(item) if is_public(&item.vis) => {
            out.extend(paths_in_generics(&item.generics));
            for field in &item.fields.named {
                if is_public(&field.vis) {
                    out.extend(paths_in_type(&field.ty));
                }
            }
        }
        syn::Item::Type(item) if is_public(&item.vis) => {
            out.extend(paths_in_generics(&item.generics));
            out.extend(paths_in_type(&item.ty));
        }
        syn::Item::Const(item) if is_public(&item.vis) => {
            out.extend(paths_in_type(&item.ty));
        }
        syn::Item::Static(item) if is_public(&item.vis) => {
            out.extend(paths_in_type(&item.ty));
        }
        syn::Item::Trait(item) if is_public(&item.vis) => {
            out.extend(paths_in_generics(&item.generics));
            // Supertraits are part of the trait's public contract.
            for bound in &item.supertraits {
                if let syn::TypeParamBound::Trait(trait_bound) = bound {
                    out.push(trait_bound.path.clone());
                }
            }
            for trait_item in &item.items {
                match trait_item {
                    syn::TraitItem::Fn(method) => out.extend(paths_in_signature(&method.sig)),
                    syn::TraitItem::Type(assoc) => {
                        for bound in &assoc.bounds {
                            if let syn::TypeParamBound::Trait(trait_bound) = bound {
                                out.push(trait_bound.path.clone());
                            }
                        }
                    }
                    syn::TraitItem::Const(assoc) => out.extend(paths_in_type(&assoc.ty)),
                    _ => {}
                }
            }
        }
        // Inherent `impl Type { … }` (no trait): its `pub` methods are public API the module
        // authored. Trait impls (`impl Trait for Type`) carry `trait_` and are out of scope.
        syn::Item::Impl(item) if item.trait_.is_none() => {
            for impl_item in &item.items {
                if let syn::ImplItem::Fn(method) = impl_item {
                    if is_public(&method.vis) {
                        out.extend(paths_in_signature(&method.sig));
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
mod tests {
    use super::*;

    /// Write `files` (each `(relative path, contents)`) under a unique temp `src` dir, then
    /// return the findings for `module` against `forbidden`. Exercises the whole evaluator
    /// (module resolution → exposure → use-resolution → match) without spawning `cargo`.
    fn findings(
        name: &str,
        files: &[(&str, &str)],
        module: &str,
        forbidden: &[&str],
    ) -> Result<Vec<String>, String> {
        let dir = std::env::temp_dir().join(format!("hunyi-{name}-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        let src = dir.join("src");
        for (rel, contents) in files {
            let path = src.join(rel);
            std::fs::create_dir_all(path.parent().expect("file has a parent")).expect("mkdir");
            std::fs::write(&path, contents).expect("write source");
        }
        let forbidden: Vec<String> = forbidden.iter().map(|s| s.to_string()).collect();
        let root = src.join("lib.rs");
        let result = module_findings(&src, &root, module, &forbidden, "x");
        let _ = std::fs::remove_dir_all(&dir);
        result
    }

    #[test]
    fn forbidden_type_in_a_public_return_is_a_finding() {
        let out = findings(
            "return",
            &[
                ("lib.rs", "pub mod domain;\n"),
                (
                    "domain.rs",
                    "pub fn pool() -> crate::infra::DbPool { todo!() }\n",
                ),
            ],
            "crate::domain",
            &["crate::infra"],
        )
        .unwrap();
        assert_eq!(out, ["crate::infra::DbPool"]);
    }

    #[test]
    fn a_type_used_only_internally_is_not_a_finding() {
        // Imported and used in a private fn body / private item — never in a public
        // signature. This is the exposure-vs-import distinction: a static import boundary
        // would flag the import; semantic correctly says clean.
        let out = findings(
            "internal-only",
            &[
                ("lib.rs", "pub mod domain;\n"),
                (
                    "domain.rs",
                    "use crate::infra::DbPool;\nfn helper() -> DbPool { todo!() }\nstruct Private { p: DbPool }\n",
                ),
            ],
            "crate::domain",
            &["crate::infra"],
        )
        .unwrap();
        assert!(out.is_empty(), "internal use is not exposure: {out:?}");
    }

    #[test]
    fn forbidden_type_in_a_public_field_is_a_finding() {
        let out = findings(
            "field",
            &[
                ("lib.rs", "pub mod domain;\n"),
                (
                    "domain.rs",
                    "pub struct Service { pub pool: crate::infra::DbPool, secret: u8 }\n",
                ),
            ],
            "crate::domain",
            &["crate::infra"],
        )
        .unwrap();
        assert_eq!(out, ["crate::infra::DbPool"]);
    }

    #[test]
    fn a_private_field_does_not_expose() {
        let out = findings(
            "private-field",
            &[
                ("lib.rs", "pub mod domain;\n"),
                (
                    "domain.rs",
                    "pub struct Service { pool: crate::infra::DbPool }\n",
                ),
            ],
            "crate::domain",
            &["crate::infra"],
        )
        .unwrap();
        assert!(out.is_empty(), "a private field is not public API: {out:?}");
    }

    #[test]
    fn inherent_impl_public_method_exposes() {
        let out = findings(
            "inherent-impl",
            &[
                ("lib.rs", "pub mod domain;\n"),
                (
                    "domain.rs",
                    "pub struct S;\nimpl S { pub fn pool(&self) -> crate::infra::DbPool { todo!() } }\n",
                ),
            ],
            "crate::domain",
            &["crate::infra"],
        )
        .unwrap();
        assert_eq!(out, ["crate::infra::DbPool"]);
    }

    #[test]
    fn trait_impl_is_out_of_scope() {
        let out = findings(
            "trait-impl",
            &[
                ("lib.rs", "pub mod domain;\n"),
                (
                    "domain.rs",
                    "pub struct S;\nimpl From<crate::infra::DbPool> for S { fn from(_: crate::infra::DbPool) -> S { S } }\n",
                ),
            ],
            "crate::domain",
            &["crate::infra"],
        )
        .unwrap();
        assert!(
            out.is_empty(),
            "trait impls are a documented bound: {out:?}"
        );
    }

    #[test]
    fn a_renamed_import_resolves_and_reacts() {
        let out = findings(
            "renamed",
            &[
                ("lib.rs", "pub mod domain;\n"),
                (
                    "domain.rs",
                    "use crate::infra::DbPool as Pool;\npub fn pool() -> Pool { todo!() }\n",
                ),
            ],
            "crate::domain",
            &["crate::infra"],
        )
        .unwrap();
        assert_eq!(out, ["crate::infra::DbPool"]);
    }

    #[test]
    fn a_use_imported_type_resolves_via_its_head() {
        let out = findings(
            "use-head",
            &[
                ("lib.rs", "pub mod domain;\n"),
                (
                    "domain.rs",
                    "use crate::infra;\npub fn pool() -> infra::DbPool { todo!() }\n",
                ),
            ],
            "crate::domain",
            &["crate::infra"],
        )
        .unwrap();
        assert_eq!(out, ["crate::infra::DbPool"]);
    }

    #[test]
    fn a_glob_import_is_a_documented_bound() {
        let out = findings(
            "glob",
            &[
                ("lib.rs", "pub mod domain;\n"),
                (
                    "domain.rs",
                    "use crate::infra::*;\npub fn pool() -> DbPool { todo!() }\n",
                ),
            ],
            "crate::domain",
            &["crate::infra"],
        )
        .unwrap();
        assert!(
            out.is_empty(),
            "glob is out of scope, not silently matched: {out:?}"
        );
    }

    #[test]
    fn a_forbidden_trait_in_a_generic_bound_is_a_finding() {
        let out = findings(
            "bound",
            &[
                ("lib.rs", "pub mod domain;\n"),
                (
                    "domain.rs",
                    "pub fn run<T: crate::infra::Pooled>(_: T) {}\n",
                ),
            ],
            "crate::domain",
            &["crate::infra"],
        )
        .unwrap();
        assert_eq!(out, ["crate::infra::Pooled"]);
    }

    #[test]
    fn a_module_prefix_matches_beneath_but_not_a_sibling() {
        let out = findings(
            "prefix",
            &[
                ("lib.rs", "pub mod domain;\n"),
                (
                    "domain.rs",
                    "pub fn a() -> crate::infra::db::Pool { todo!() }\npub fn b() -> crate::infrastructure::Helper { todo!() }\n",
                ),
            ],
            "crate::domain",
            &["crate::infra"],
        )
        .unwrap();
        assert_eq!(
            out,
            ["crate::infra::db::Pool"],
            "sibling must not match: {out:?}"
        );
    }

    #[test]
    fn a_nested_generic_argument_is_observed() {
        let out = findings(
            "nested",
            &[
                ("lib.rs", "pub mod domain;\n"),
                (
                    "domain.rs",
                    "pub fn pools() -> Vec<crate::infra::DbPool> { todo!() }\n",
                ),
            ],
            "crate::domain",
            &["crate::infra"],
        )
        .unwrap();
        assert_eq!(out, ["crate::infra::DbPool"]);
    }

    #[test]
    fn an_unknown_module_is_a_constitution_error() {
        let err = findings(
            "unknown",
            &[
                ("lib.rs", "pub mod domain;\n"),
                ("domain.rs", "// nothing\n"),
            ],
            "crate::ghost",
            &["crate::infra"],
        )
        .unwrap_err();
        assert_eq!(err, unknown_module_error("crate::ghost", "x"));
    }

    #[test]
    fn a_mod_rs_backed_module_resolves() {
        let out = findings(
            "modrs",
            &[
                ("lib.rs", "pub mod domain;\n"),
                (
                    "domain/mod.rs",
                    "pub fn pool() -> crate::infra::DbPool { todo!() }\n",
                ),
            ],
            "crate::domain",
            &["crate::infra"],
        )
        .unwrap();
        assert_eq!(out, ["crate::infra::DbPool"]);
    }

    #[test]
    fn an_inline_module_resolves() {
        let out = findings(
            "inline",
            &[(
                "lib.rs",
                "pub mod domain { pub fn pool() -> crate::infra::DbPool { todo!() } }\n",
            )],
            "crate::domain",
            &["crate::infra"],
        )
        .unwrap();
        assert_eq!(out, ["crate::infra::DbPool"]);
    }

    // --- signature-coupling re-export back-fill (S1) -------------------------

    #[test]
    fn a_forbidden_type_via_a_pub_use_facade_resolves_and_reacts() {
        // The closed false negative: domain imports the type via a facade that re-exports
        // it; resolution must follow the `pub use` chain to the forbidden defining path.
        let out = findings(
            "reexport-exposure",
            &[
                ("lib.rs", "pub mod domain;\npub mod facade;\n"),
                ("facade.rs", "pub use crate::infra::DbPool;\n"),
                (
                    "domain.rs",
                    "use crate::facade::DbPool;\npub fn pool() -> DbPool { todo!() }\n",
                ),
            ],
            "crate::domain",
            &["crate::infra"],
        )
        .unwrap();
        assert_eq!(
            out,
            ["crate::infra::DbPool"],
            "a forbidden type reached through a pub use facade must react"
        );
    }

    #[test]
    fn a_forbidden_type_via_a_super_relative_use_resolves_and_reacts() {
        // The same relative-use canonicalization fix applies to exposure-governance: a
        // forbidden type imported via `use super::infra::DbPool` must resolve to its
        // canonical path, not be silently passed.
        let out = findings(
            "super-exposure",
            &[
                ("lib.rs", "pub mod domain;\npub mod infra;\n"),
                ("infra.rs", "pub struct DbPool;\n"),
                (
                    "domain.rs",
                    "use super::infra::DbPool;\npub fn pool() -> DbPool { todo!() }\n",
                ),
            ],
            "crate::domain",
            &["crate::infra"],
        )
        .unwrap();
        assert_eq!(out, ["crate::infra::DbPool"]);
    }

    // --- trait-impl-locality ------------------------------------------------

    fn locality_findings(
        name: &str,
        files: &[(&str, &str)],
        trait_path: &str,
        allowed: &[&str],
    ) -> Result<Vec<String>, String> {
        let dir = std::env::temp_dir().join(format!("hunyi-loc-{name}-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        let src = dir.join("src");
        for (rel, contents) in files {
            let path = src.join(rel);
            std::fs::create_dir_all(path.parent().expect("file has a parent")).expect("mkdir");
            std::fs::write(&path, contents).expect("write source");
        }
        let allowed: Vec<String> = allowed.iter().map(|s| s.to_string()).collect();
        let root = src.join("lib.rs");
        let result = trait_impl_findings(&src, &root, trait_path, &allowed, "x");
        let _ = std::fs::remove_dir_all(&dir);
        result
    }

    #[test]
    fn an_impl_outside_the_allowed_location_is_a_finding() {
        let out = locality_findings(
            "outside",
            &[
                ("lib.rs", "pub mod command;\npub mod domain;\n"),
                ("command.rs", "pub trait Command {}\n"),
                (
                    "domain.rs",
                    "use crate::command::Command;\npub struct Foo;\nimpl Command for Foo {}\n",
                ),
            ],
            "crate::command::Command",
            &["crate::commands"],
        )
        .unwrap();
        assert_eq!(out, ["crate::domain (impl for Foo)"]);
    }

    #[test]
    fn an_impl_inside_the_allowed_location_is_clean() {
        let out = locality_findings(
            "inside",
            &[
                ("lib.rs", "pub mod command;\npub mod commands;\n"),
                ("command.rs", "pub trait Command {}\n"),
                (
                    "commands.rs",
                    "use crate::command::Command;\npub struct Foo;\nimpl Command for Foo {}\n",
                ),
            ],
            "crate::command::Command",
            &["crate::commands"],
        )
        .unwrap();
        assert!(
            out.is_empty(),
            "an impl in the allowed location is clean: {out:?}"
        );
    }

    #[test]
    fn a_nested_module_beneath_the_allowed_prefix_is_clean() {
        let out = locality_findings(
            "nested-allowed",
            &[
                ("lib.rs", "pub mod command;\npub mod commands;\n"),
                ("command.rs", "pub trait Command {}\n"),
                ("commands.rs", "pub mod greet;\n"),
                (
                    "commands/greet.rs",
                    "use crate::command::Command;\npub struct Foo;\nimpl Command for Foo {}\n",
                ),
            ],
            "crate::command::Command",
            &["crate::commands"],
        )
        .unwrap();
        assert!(
            out.is_empty(),
            "beneath an allowed prefix is clean: {out:?}"
        );
    }

    #[test]
    fn a_prefix_colliding_sibling_location_is_not_allowed() {
        let out = locality_findings(
            "sibling",
            &[
                ("lib.rs", "pub mod command;\npub mod commandeer;\n"),
                ("command.rs", "pub trait Command {}\n"),
                (
                    "commandeer.rs",
                    "use crate::command::Command;\npub struct Foo;\nimpl Command for Foo {}\n",
                ),
            ],
            "crate::command::Command",
            &["crate::commands"],
        )
        .unwrap();
        assert_eq!(
            out,
            ["crate::commandeer (impl for Foo)"],
            "a sibling of the allowed prefix is not allowed"
        );
    }

    #[test]
    fn an_impl_in_any_of_several_allowed_locations_is_clean() {
        let out = locality_findings(
            "multi-allowed",
            &[
                ("lib.rs", "pub mod command;\npub mod builtins;\n"),
                ("command.rs", "pub trait Command {}\n"),
                (
                    "builtins.rs",
                    "use crate::command::Command;\npub struct Foo;\nimpl Command for Foo {}\n",
                ),
            ],
            "crate::command::Command",
            &["crate::commands", "crate::builtins"],
        )
        .unwrap();
        assert!(out.is_empty(), "any one allowed location suffices: {out:?}");
    }

    #[test]
    fn a_bare_same_module_trait_name_reacts() {
        // B1: the impl is in the trait's own (disallowed) module, with a bare `Command`
        // and no `use`. Resolving the bare name against the current module is required —
        // leaving it unresolved would silently pass a real misplaced impl.
        let out = locality_findings(
            "bare-same-module",
            &[
                ("lib.rs", "pub mod command;\n"),
                (
                    "command.rs",
                    "pub trait Command {}\npub struct Foo;\nimpl Command for Foo {}\n",
                ),
            ],
            "crate::command::Command",
            &["crate::commands"],
        )
        .unwrap();
        assert_eq!(out, ["crate::command (impl for Foo)"]);
    }

    #[test]
    fn a_renamed_trait_import_reacts() {
        let out = locality_findings(
            "renamed-trait",
            &[
                ("lib.rs", "pub mod command;\npub mod domain;\n"),
                ("command.rs", "pub trait Command {}\n"),
                (
                    "domain.rs",
                    "use crate::command::Command as Cmd;\npub struct Foo;\nimpl Cmd for Foo {}\n",
                ),
            ],
            "crate::command::Command",
            &["crate::commands"],
        )
        .unwrap();
        assert_eq!(out, ["crate::domain (impl for Foo)"]);
    }

    #[test]
    fn a_super_relative_trait_import_reacts() {
        // The relative-use false negative: `use super::command::Command` populates the
        // use-map with the relative string; resolution must canonicalize it against the
        // module before matching the anchor, or a real misplaced impl silently passes.
        let out = locality_findings(
            "super-trait",
            &[
                ("lib.rs", "pub mod command;\npub mod domain;\n"),
                ("command.rs", "pub trait Command {}\n"),
                (
                    "domain.rs",
                    "use super::command::Command;\npub struct Foo;\nimpl Command for Foo {}\n",
                ),
            ],
            "crate::command::Command",
            &["crate::commands"],
        )
        .unwrap();
        assert_eq!(out, ["crate::domain (impl for Foo)"]);
    }

    #[test]
    fn a_cfg_gated_module_with_no_file_is_skipped_not_errored() {
        // A `#[cfg(feature = "x")] mod optional;` with no `optional.rs` (the feature is off)
        // is legal Rust; the whole-crate walk must skip it, never fail the gate (exit 2).
        let out = locality_findings(
            "cfg-absent-mod",
            &[
                (
                    "lib.rs",
                    "pub mod command;\n#[cfg(feature = \"never\")]\npub mod optional;\n",
                ),
                ("command.rs", "pub trait Command {}\n"),
            ],
            "crate::command::Command",
            &["crate::commands"],
        )
        .unwrap();
        assert!(
            out.is_empty(),
            "a cfg-gated absent module is skipped: {out:?}"
        );
    }

    #[test]
    fn a_reexported_trait_path_reacts() {
        // S1: the impl reaches the trait through a facade re-export; resolution must
        // follow the pub use chain to match the anchor.
        let out = locality_findings(
            "reexport-impl",
            &[
                (
                    "lib.rs",
                    "pub mod command;\npub mod facade;\npub mod domain;\n",
                ),
                ("command.rs", "pub trait Command {}\n"),
                ("facade.rs", "pub use crate::command::Command;\n"),
                (
                    "domain.rs",
                    "use crate::facade::Command;\npub struct Foo;\nimpl Command for Foo {}\n",
                ),
            ],
            "crate::command::Command",
            &["crate::commands"],
        )
        .unwrap();
        assert_eq!(out, ["crate::domain (impl for Foo)"]);
    }

    #[test]
    fn an_anchor_named_at_a_reexport_path_resolves_not_a_constitution_error() {
        // B2: the boundary names the trait at its facade path; this must resolve to the
        // real local trait (not a false exit-2) and still react to misplaced impls.
        let out = locality_findings(
            "reexport-anchor",
            &[
                (
                    "lib.rs",
                    "pub mod command;\npub mod facade;\npub mod domain;\n",
                ),
                ("command.rs", "pub trait Command {}\n"),
                ("facade.rs", "pub use crate::command::Command;\n"),
                (
                    "domain.rs",
                    "use crate::command::Command;\npub struct Foo;\nimpl Command for Foo {}\n",
                ),
            ],
            "crate::facade::Command",
            &["crate::commands"],
        )
        .unwrap();
        assert_eq!(out, ["crate::domain (impl for Foo)"]);
    }

    #[test]
    fn an_unresolvable_trait_anchor_is_a_constitution_error() {
        let err = locality_findings(
            "ghost-trait",
            &[
                ("lib.rs", "pub mod command;\n"),
                ("command.rs", "pub trait Command {}\n"),
            ],
            "crate::command::Ghost",
            &["crate::commands"],
        )
        .unwrap_err();
        assert_eq!(err, unknown_trait_error("crate::command::Ghost", "x"));
    }

    #[test]
    fn a_non_anchored_traits_impl_is_ignored() {
        let out = locality_findings(
            "other-trait",
            &[
                ("lib.rs", "pub mod command;\npub mod domain;\n"),
                ("command.rs", "pub trait Command {}\npub trait Other {}\n"),
                (
                    "domain.rs",
                    "use crate::command::Other;\npub struct Foo;\nimpl Other for Foo {}\n",
                ),
            ],
            "crate::command::Command",
            &["crate::commands"],
        )
        .unwrap();
        assert!(out.is_empty(), "only the anchored trait reacts: {out:?}");
    }

    #[test]
    fn an_inline_module_impl_is_located() {
        let out = locality_findings(
            "inline-impl",
            &[
                (
                    "lib.rs",
                    "pub mod command;\npub mod domain { use crate::command::Command; pub struct Foo; impl Command for Foo {} }\n",
                ),
                ("command.rs", "pub trait Command {}\n"),
            ],
            "crate::command::Command",
            &["crate::commands"],
        )
        .unwrap();
        assert_eq!(out, ["crate::domain (impl for Foo)"]);
    }

    #[test]
    fn a_glob_imported_trait_is_a_documented_bound() {
        let out = locality_findings(
            "glob-trait",
            &[
                ("lib.rs", "pub mod command;\npub mod domain;\n"),
                ("command.rs", "pub trait Command {}\n"),
                (
                    "domain.rs",
                    "use crate::command::*;\npub struct Foo;\nimpl Command for Foo {}\n",
                ),
            ],
            "crate::command::Command",
            &["crate::commands"],
        )
        .unwrap();
        assert!(
            out.is_empty(),
            "a glob-imported trait is out of scope, not silently matched: {out:?}"
        );
    }

    #[test]
    fn a_path_remapped_module_is_a_documented_bound() {
        let out = locality_findings(
            "path-remapped",
            &[
                (
                    "lib.rs",
                    "pub mod command;\n#[path = \"weird.rs\"]\npub mod domain;\n",
                ),
                ("command.rs", "pub trait Command {}\n"),
                (
                    "weird.rs",
                    "use crate::command::Command;\npub struct Foo;\nimpl Command for Foo {}\n",
                ),
            ],
            "crate::command::Command",
            &["crate::commands"],
        )
        .unwrap();
        assert!(
            out.is_empty(),
            "a #[path]-remapped module is out of scope, not silently matched: {out:?}"
        );
    }

    #[test]
    fn two_impls_in_one_module_are_distinct_findings_by_self_type() {
        let out = locality_findings(
            "distinct-self",
            &[
                ("lib.rs", "pub mod command;\npub mod domain;\n"),
                ("command.rs", "pub trait Command {}\n"),
                (
                    "domain.rs",
                    "use crate::command::Command;\npub struct A;\npub struct B;\nimpl Command for A {}\nimpl Command for B {}\n",
                ),
            ],
            "crate::command::Command",
            &["crate::commands"],
        )
        .unwrap();
        assert_eq!(
            out,
            ["crate::domain (impl for A)", "crate::domain (impl for B)"]
        );
    }

    #[test]
    fn a_cfg_gated_impl_is_observed_as_written() {
        // `#[cfg]` is not evaluated: syn parses every branch, so a misplaced impl behind a
        // disabled feature is still observed (a deliberate, documented over-approximation).
        let out = locality_findings(
            "cfg-gated",
            &[
                ("lib.rs", "pub mod command;\npub mod domain;\n"),
                ("command.rs", "pub trait Command {}\n"),
                (
                    "domain.rs",
                    "use crate::command::Command;\npub struct Foo;\n#[cfg(feature = \"never\")]\nimpl Command for Foo {}\n",
                ),
            ],
            "crate::command::Command",
            &["crate::commands"],
        )
        .unwrap();
        assert_eq!(out, ["crate::domain (impl for Foo)"]);
    }

    #[test]
    fn a_macro_generated_impl_is_a_documented_bound() {
        // A `make_impl!(…)` invocation is an `Item::Macro`, not an `Item::Impl` — syn does
        // not expand it, so the impl it would generate is out of scope, not silently matched.
        let out = locality_findings(
            "macro-impl",
            &[
                ("lib.rs", "pub mod command;\npub mod domain;\n"),
                ("command.rs", "pub trait Command {}\n"),
                ("domain.rs", "make_impl!(Foo);\n"),
            ],
            "crate::command::Command",
            &["crate::commands"],
        )
        .unwrap();
        assert!(
            out.is_empty(),
            "a macro-generated impl is out of scope, not silently matched: {out:?}"
        );
    }

    #[test]
    fn the_builder_carries_severity() {
        // Severity (and thus baseline/exit-code parity via the shared 璇璣 model) is plumbed
        // from the builder into each Violation by `check_trait_impl_boundary`.
        let warn = TraitImplBoundary::in_crate("app")
            .trait_("crate::command::Command")
            .only_implemented_in("crate::commands")
            .warn()
            .because("advisory first");
        assert_eq!(warn.severity(), Severity::Warn);

        let enforce = TraitImplBoundary::in_crate("app")
            .trait_("crate::command::Command")
            .only_implemented_in("crate::commands")
            .because("enforced");
        assert_eq!(enforce.severity(), Severity::Enforce);
    }

    // --- visibility boundary -------------------------------------------------

    fn vis_findings(
        name: &str,
        files: &[(&str, &str)],
        module: &str,
    ) -> Result<Vec<String>, String> {
        let dir = std::env::temp_dir().join(format!("hunyi-vis-{name}-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        let src = dir.join("src");
        for (rel, contents) in files {
            let path = src.join(rel);
            std::fs::create_dir_all(path.parent().expect("file has a parent")).expect("mkdir");
            std::fs::write(&path, contents).expect("write source");
        }
        let root = src.join("lib.rs");
        let result = visibility_findings(&src, &root, module, "x");
        let _ = std::fs::remove_dir_all(&dir);
        result
    }

    #[test]
    fn pub_items_react_and_non_pub_items_are_clean() {
        let out = vis_findings(
            "pub-mix",
            &[
                ("lib.rs", "pub mod internal;\n"),
                (
                    "internal.rs",
                    "pub fn a() {}\npub struct B;\npub trait C {}\npub(crate) fn d() {}\npub(super) fn e() {}\nfn f() {}\n",
                ),
            ],
            "crate::internal",
        )
        .unwrap();
        assert_eq!(
            out,
            ["pub fn a", "pub struct B", "pub trait C"],
            "only bare-pub items react: {out:?}"
        );
    }

    #[test]
    fn a_pub_use_and_glob_react() {
        let out = vis_findings(
            "pub-use",
            &[
                ("lib.rs", "pub mod internal;\n"),
                (
                    "internal.rs",
                    "pub use crate::db::Handle;\npub use crate::db::*;\npub(crate) use crate::db::Hidden;\n",
                ),
            ],
            "crate::internal",
        )
        .unwrap();
        assert_eq!(out, ["pub use crate::db::*", "pub use crate::db::Handle"]);
    }

    #[test]
    fn a_pub_submodule_reacts() {
        let out = vis_findings(
            "pub-mod",
            &[
                ("lib.rs", "pub mod internal;\n"),
                ("internal.rs", "pub mod sub;\nmod hidden;\n"),
                ("internal/sub.rs", "\n"),
                ("internal/hidden.rs", "\n"),
            ],
            "crate::internal",
        )
        .unwrap();
        assert_eq!(out, ["pub mod sub"]);
    }

    #[test]
    fn a_bare_pub_item_in_a_non_pub_module_still_reacts() {
        let out = vis_findings(
            "pub-in-crate-mod",
            &[
                ("lib.rs", "pub(crate) mod internal;\n"),
                ("internal.rs", "pub fn helper() {}\n"),
            ],
            "crate::internal",
        )
        .unwrap();
        assert_eq!(
            out,
            ["pub fn helper"],
            "the rule governs the declared pub keyword, not crate-reachability"
        );
    }

    #[test]
    fn a_pub_extern_crate_and_pub_trait_alias_react() {
        // Bare-`pub` item kinds beyond the common set: a public crate re-export and a
        // public trait alias are observable bare-`pub` declarations and must react.
        let out = vis_findings(
            "extern-and-alias",
            &[
                ("lib.rs", "pub mod internal;\n"),
                (
                    "internal.rs",
                    "pub extern crate serde;\npub trait Alias = Clone;\n",
                ),
            ],
            "crate::internal",
        )
        .unwrap();
        assert_eq!(out, ["pub extern crate serde", "pub trait Alias (alias)"]);
    }

    #[test]
    fn a_leading_colon_pub_use_is_rendered_and_distinct() {
        // `::external::X` and `external::X` are distinct declarations; the leading colon
        // must be rendered so they do not collide under dedup.
        let out = vis_findings(
            "leading-colon",
            &[
                ("lib.rs", "pub mod internal;\n"),
                (
                    "internal.rs",
                    "pub use ::external::X;\npub use external::X;\n",
                ),
            ],
            "crate::internal",
        )
        .unwrap();
        assert_eq!(out, ["pub use ::external::X", "pub use external::X"]);
    }

    #[test]
    fn a_macro_export_macro_is_out_of_scope() {
        let out = vis_findings(
            "macro-export",
            &[
                ("lib.rs", "pub mod internal;\n"),
                (
                    "internal.rs",
                    "#[macro_export]\nmacro_rules! m { () => {} }\npub(crate) fn helper() {}\n",
                ),
            ],
            "crate::internal",
        )
        .unwrap();
        assert!(
            out.is_empty(),
            "a #[macro_export] macro carries no pub keyword — out of declared scope: {out:?}"
        );
    }

    #[test]
    fn a_macro_invocation_pub_item_is_a_documented_bound() {
        let out = vis_findings(
            "macro-gen",
            &[
                ("lib.rs", "pub mod internal;\n"),
                ("internal.rs", "make_public!();\n"),
            ],
            "crate::internal",
        )
        .unwrap();
        assert!(
            out.is_empty(),
            "a macro-generated item is out of scope, not silently claimed: {out:?}"
        );
    }

    #[test]
    fn a_cfg_gated_pub_item_is_observed_as_written() {
        let out = vis_findings(
            "cfg-pub",
            &[
                ("lib.rs", "pub mod internal;\n"),
                (
                    "internal.rs",
                    "#[cfg(feature = \"never\")]\npub fn gated() {}\n",
                ),
            ],
            "crate::internal",
        )
        .unwrap();
        assert_eq!(out, ["pub fn gated"], "cfg is observed as-written");
    }

    #[test]
    fn an_unknown_visibility_module_is_a_constitution_error() {
        let err = vis_findings(
            "vis-unknown",
            &[("lib.rs", "pub mod internal;\n"), ("internal.rs", "\n")],
            "crate::ghost",
        )
        .unwrap_err();
        assert_eq!(err, unknown_module_error("crate::ghost", "x"));
    }

    #[test]
    fn an_inline_visibility_module_is_scanned() {
        let out = vis_findings(
            "vis-inline",
            &[("lib.rs", "pub mod internal { pub fn a() {} fn b() {} }\n")],
            "crate::internal",
        )
        .unwrap();
        assert_eq!(out, ["pub fn a"]);
    }

    #[test]
    fn the_visibility_builder_carries_severity() {
        let warn = VisibilityBoundary::in_crate("app")
            .module("crate::internal")
            .must_not_declare_pub()
            .warn()
            .because("advisory first");
        assert_eq!(warn.severity(), Severity::Warn);

        let enforce = VisibilityBoundary::in_crate("app")
            .module("crate::internal")
            .must_not_declare_pub()
            .because("enforced");
        assert_eq!(enforce.severity(), Severity::Enforce);
    }

    #[test]
    fn a_generic_self_type_is_rendered_distinctly() {
        let out = locality_findings(
            "generic-self",
            &[
                ("lib.rs", "pub mod command;\npub mod domain;\n"),
                ("command.rs", "pub trait Command {}\n"),
                (
                    "domain.rs",
                    "use crate::command::Command;\npub struct W<T>(T);\nimpl Command for W<u8> {}\nimpl Command for W<u16> {}\n",
                ),
            ],
            "crate::command::Command",
            &["crate::commands"],
        )
        .unwrap();
        assert_eq!(
            out,
            [
                "crate::domain (impl for W<u16>)",
                "crate::domain (impl for W<u8>)"
            ]
        );
    }

    // --- forbidden-marker ----------------------------------------------------

    fn marker_findings(
        name: &str,
        files: &[(&str, &str)],
        subtree: &str,
        forbidden: &[&str],
    ) -> Result<Vec<String>, String> {
        let dir = std::env::temp_dir().join(format!("hunyi-mark-{name}-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        let src = dir.join("src");
        for (rel, contents) in files {
            let path = src.join(rel);
            std::fs::create_dir_all(path.parent().expect("file has a parent")).expect("mkdir");
            std::fs::write(&path, contents).expect("write source");
        }
        let forbidden: Vec<String> = forbidden.iter().map(|s| s.to_string()).collect();
        let root = src.join("lib.rs");
        let result = forbidden_marker_findings(&src, &root, subtree, &forbidden, "x");
        let _ = std::fs::remove_dir_all(&dir);
        result
    }

    #[test]
    fn a_forbidden_derive_on_a_subtree_type_reacts_and_a_clean_type_does_not() {
        let out = marker_findings(
            "derive",
            &[
                ("lib.rs", "pub mod domain;\n"),
                (
                    "domain.rs",
                    "#[derive(serde::Serialize)]\npub struct Order;\n#[derive(Clone, Debug)]\npub struct Plain;\n",
                ),
            ],
            "crate::domain",
            &["serde::Serialize"],
        )
        .unwrap();
        assert_eq!(out, ["derive serde::Serialize on crate::domain::Order"]);
    }

    #[test]
    fn a_serde_derive_path_and_cfg_attr_derive_react_by_leaf() {
        let out = marker_findings(
            "leaf-and-cfgattr",
            &[
                ("lib.rs", "pub mod domain;\n"),
                (
                    "domain.rs",
                    "#[derive(serde_derive::Serialize)]\npub struct A;\n#[cfg_attr(feature = \"serde\", derive(serde::Serialize))]\npub struct B;\n",
                ),
            ],
            "crate::domain",
            &["serde::Serialize"],
        )
        .unwrap();
        assert_eq!(
            out,
            [
                "derive serde::Serialize on crate::domain::A",
                "derive serde::Serialize on crate::domain::B"
            ],
            "serde_derive path (leaf) and cfg_attr-wrapped derive both react: {out:?}"
        );
    }

    #[test]
    fn a_hand_impl_outside_the_subtree_reacts_via_the_self_type() {
        let out = marker_findings(
            "hand-impl",
            &[
                ("lib.rs", "pub mod domain;\npub mod wire;\n"),
                ("domain.rs", "pub struct Order;\n"),
                (
                    "wire.rs",
                    "impl serde::Serialize for crate::domain::Order {}\n",
                ),
            ],
            "crate::domain",
            &["serde::Serialize"],
        )
        .unwrap();
        assert_eq!(
            out,
            ["impl serde::Serialize for crate::domain::Order"],
            "a hand impl written outside the subtree, for a subtree type, reacts: {out:?}"
        );
    }

    #[test]
    fn a_submodule_type_is_governed_and_a_sibling_is_not() {
        let out = marker_findings(
            "subtree",
            &[
                ("lib.rs", "pub mod domain;\npub mod domainx;\n"),
                ("domain.rs", "pub mod order;\n"),
                (
                    "domain/order.rs",
                    "#[derive(serde::Serialize)]\npub struct Order;\n",
                ),
                (
                    "domainx.rs",
                    "#[derive(serde::Serialize)]\npub struct Other;\n",
                ),
            ],
            "crate::domain",
            &["serde::Serialize"],
        )
        .unwrap();
        assert_eq!(
            out,
            ["derive serde::Serialize on crate::domain::order::Order"],
            "a submodule type is governed; the prefix-colliding sibling crate::domainx is not: {out:?}"
        );
    }

    #[test]
    fn a_same_leaf_different_trait_is_a_documented_false_positive() {
        let out = marker_findings(
            "leaf-fp",
            &[
                ("lib.rs", "pub mod domain;\n"),
                (
                    "domain.rs",
                    "#[derive(rkyv::Serialize)]\npub struct Order;\n",
                ),
            ],
            "crate::domain",
            &["Serialize"],
        )
        .unwrap();
        assert_eq!(
            out,
            ["derive Serialize on crate::domain::Order"],
            "leaf-match reacts (accepted false positive; path-qualify to document intent)"
        );
    }

    #[test]
    fn an_unresolvable_glob_self_type_is_a_documented_bound() {
        let out = marker_findings(
            "glob-self",
            &[
                ("lib.rs", "pub mod domain;\npub mod wire;\n"),
                ("domain.rs", "pub struct Order;\n"),
                (
                    "wire.rs",
                    "use crate::domain::*;\nimpl serde::Serialize for Order {}\n",
                ),
            ],
            "crate::domain",
            &["serde::Serialize"],
        )
        .unwrap();
        assert!(
            out.is_empty(),
            "a glob-imported self-type cannot be placed in the subtree — a stated bound: {out:?}"
        );
    }

    #[test]
    fn a_nested_cfg_attr_derive_reacts() {
        // The review's blocker: `cfg_attr(a, cfg_attr(b, derive(X)))` must still yield X.
        let out = marker_findings(
            "nested-cfgattr",
            &[
                ("lib.rs", "pub mod domain;\n"),
                (
                    "domain.rs",
                    "#[cfg_attr(all(), cfg_attr(all(), derive(serde::Serialize)))]\npub struct Order;\n",
                ),
            ],
            "crate::domain",
            &["serde::Serialize"],
        )
        .unwrap();
        assert_eq!(out, ["derive serde::Serialize on crate::domain::Order"]);
    }

    #[test]
    fn two_same_named_types_in_different_submodules_stay_distinct() {
        // The review's baseline-collapse blocker: the finding must use the canonical path so
        // two `Order`s don't dedup into one (baselining one would else suppress the other).
        let out = marker_findings(
            "same-name",
            &[
                ("lib.rs", "pub mod domain;\n"),
                ("domain.rs", "pub mod a;\npub mod b;\n"),
                (
                    "domain/a.rs",
                    "#[derive(serde::Serialize)]\npub struct Order;\n",
                ),
                (
                    "domain/b.rs",
                    "#[derive(serde::Serialize)]\npub struct Order;\n",
                ),
            ],
            "crate::domain",
            &["serde::Serialize"],
        )
        .unwrap();
        assert_eq!(
            out,
            [
                "derive serde::Serialize on crate::domain::a::Order",
                "derive serde::Serialize on crate::domain::b::Order"
            ],
            "two same-named types must stay distinct findings: {out:?}"
        );
    }

    #[test]
    fn the_forbidden_marker_builder_carries_severity() {
        let b = ForbiddenMarkerBoundary::in_crate("app")
            .module("crate::domain")
            .must_not_acquire("serde::Serialize")
            .and_not_acquire("serde::Deserialize")
            .warn()
            .because("r");
        assert_eq!(b.forbidden(), &["serde::Serialize", "serde::Deserialize"]);
        assert_eq!(b.severity(), Severity::Warn);
    }
}
