//! Declaration DSL for 渾儀's semantic boundaries — the builder types each capability
//! exposes (`SemanticBoundary`, `TraitImplBoundary`, `VisibilityBoundary`,
//! `ForbiddenMarkerBoundary`, `DynTraitBoundary`, `ImplTraitBoundary`,
//! `AsyncExposureBoundary`) and their crate/module/boundary draft chains. Pure data and
//! builders — no scan, no resolution, no reaction — re-exported from the crate root so the
//! public paths (`hunyi::SemanticBoundary`, …) stay unchanged.

use xuanji::Severity;

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
    /// Opt-in depth (`semantic-trait-impl-exposure`): also observe the module's trait `impl`
    /// blocks' impl-site-authored positions. `false` keeps the v1 signature-coupling surface.
    pub(crate) including_trait_impls: bool,
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

    /// Whether the boundary also observes trait `impl` blocks (the opt-in
    /// `semantic-trait-impl-exposure` depth). `false` is the v1 signature-coupling surface.
    pub fn including_trait_impls(&self) -> bool {
        self.including_trait_impls
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
            including_trait_impls: false,
        }
    }
}

/// A boundary awaiting severity (optional) and its reason.
pub struct SemanticBoundaryDraft {
    crate_package: String,
    module: String,
    forbidden: Vec<String>,
    severity: Severity,
    including_trait_impls: bool,
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

    /// **Opt-in depth** (`semantic-trait-impl-exposure`): also observe the module's trait `impl`
    /// blocks. A bare `must_not_expose` keeps the v1 surface (trait impls out of scope); this
    /// deepens it to a trait impl's impl-site-authored positions — the trait's generic arguments,
    /// the `Self` type (bare and nested), associated-type bindings, the impl's own generics /
    /// `where`-clause, and the method **return type as written** (which RPITIT lets the impl author
    /// refine to a concrete type). Method parameters/receiver stay trait-dictated and out of scope,
    /// and implementing a forbidden *trait* is `must_not_acquire`/locality's concern, not this.
    pub fn including_trait_impls(mut self) -> Self {
        self.including_trait_impls = true;
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
            including_trait_impls: self.including_trait_impls,
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

// --- Dyn-trait-boundary declaration DSL --------------------------------------

/// A dyn-trait boundary: a module's public API must not **expose** trait-object (`dyn`)
/// syntax. The type-shape complement of [`SemanticBoundary`] (signature-coupling): where
/// that forbids an exposed *named type*, this forbids an exposed *type shape* — a `dyn`
/// node at any depth in the governed public surface. Internal `dyn` is never a violation —
/// this governs exposure across the declared seam, not internal dynamic dispatch, so it is
/// intent (by anchor scoping), not a lint. Declared in Rust and composed with the other
/// dimensions at the gate.
///
/// Two depths on one boundary type, selected by the builder:
/// - [`must_not_expose_dyn`](DynTraitModuleDraft::must_not_expose_dyn) — **shape-only**: an
///   empty operand set, so *any* exposed `dyn` reacts.
/// - [`must_not_expose_dyn_of`](DynTraitModuleDraft::must_not_expose_dyn_of) — **operand-scoped**:
///   only a `dyn` whose principal trait resolves into the named `forbidden_operands` set reacts.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DynTraitBoundary {
    pub(crate) crate_package: String,
    pub(crate) module: String,
    /// The forbidden trait operands. **Empty ⇒ shape-only** (any `dyn` reacts); a named set ⇒
    /// only a `dyn` whose principal trait canonicalizes into the set reacts.
    pub(crate) forbidden_operands: Vec<String>,
    pub(crate) reason: String,
    pub(crate) severity: Severity,
}

impl DynTraitBoundary {
    /// Begin a dyn-trait boundary in the crate named `package`.
    pub fn in_crate(package: &str) -> DynTraitCrateDraft {
        DynTraitCrateDraft {
            crate_package: package.to_string(),
        }
    }

    /// The crate this boundary governs.
    pub fn crate_package(&self) -> &str {
        &self.crate_package
    }

    /// The governed module path (e.g. `crate::core`).
    pub fn module(&self) -> &str {
        &self.module
    }

    /// The forbidden trait operands. Empty ⇒ shape-only (any `dyn` reacts); a named set ⇒
    /// only a `dyn` whose principal trait resolves into the set reacts.
    pub fn forbidden_operands(&self) -> &[String] {
        &self.forbidden_operands
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

/// A dyn-trait boundary awaiting its module anchor.
pub struct DynTraitCrateDraft {
    crate_package: String,
}

impl DynTraitCrateDraft {
    /// Anchor the boundary to a module path within the crate (e.g. `crate::core`).
    pub fn module(self, module: &str) -> DynTraitModuleDraft {
        DynTraitModuleDraft {
            crate_package: self.crate_package,
            module: module.to_string(),
        }
    }
}

/// A module-anchored boundary awaiting the rule.
pub struct DynTraitModuleDraft {
    crate_package: String,
    module: String,
}

impl DynTraitModuleDraft {
    /// Forbid the module's public API from exposing any trait-object (`dyn`) syntax. Takes no
    /// trait operand — *any* exposed `dyn` reacts (shape-only).
    pub fn must_not_expose_dyn(self) -> DynTraitBoundaryDraft {
        DynTraitBoundaryDraft {
            crate_package: self.crate_package,
            module: self.module,
            forbidden_operands: Vec::new(),
            severity: Severity::Enforce,
        }
    }

    /// Forbid the module's public API from exposing a `dyn` of any **named trait** — the
    /// operand-scoped depth of [`must_not_expose_dyn`](Self::must_not_expose_dyn). A `dyn` whose
    /// **principal trait** (its first trait bound) canonicalizes into `operands` is a violation;
    /// a `dyn` of any other trait passes. An `operands` entry may be an exact trait path
    /// (`crate::ports::Port`) or a module prefix (`crate::ports`), and a re-exported/aliased
    /// trait facade matches its defining path (resolved through the same 渾儀 resolver the
    /// forbidden-type rule uses).
    ///
    /// Bounds (stated, not silent): an **empty** `operands` set degenerates to shape-only
    /// (`must_not_expose_dyn`) — a loud "any `dyn` reacts", never an inert no-op. A principal
    /// trait that does not resolve — a bare name with no `use` (a std `dyn Fn(…)` / `dyn
    /// Iterator<…>`, a bare `dyn Send`), a macro-generated or glob/cross-crate re-exported trait
    /// — is out of the resolver's stated coverage and is not matched; a *resolvable* operand is
    /// never silently passed. Auto-trait / lifetime bounds are never operands (only the principal,
    /// first, trait is matched).
    pub fn must_not_expose_dyn_of<I, S>(self, operands: I) -> DynTraitBoundaryDraft
    where
        I: IntoIterator<Item = S>,
        S: Into<String>,
    {
        DynTraitBoundaryDraft {
            crate_package: self.crate_package,
            module: self.module,
            forbidden_operands: operands.into_iter().map(Into::into).collect(),
            severity: Severity::Enforce,
        }
    }
}

/// A boundary awaiting severity (optional) and its reason.
pub struct DynTraitBoundaryDraft {
    crate_package: String,
    module: String,
    forbidden_operands: Vec<String>,
    severity: Severity,
}

impl DynTraitBoundaryDraft {
    /// Make this an advisory (`warn`) boundary: violations are reported but do not fail the
    /// reaction — the first rung of adoption.
    pub fn warn(mut self) -> Self {
        self.severity = Severity::Warn;
        self
    }

    /// Finish the boundary with its human-readable reason (the repair hint).
    pub fn because(self, reason: &str) -> DynTraitBoundary {
        DynTraitBoundary {
            crate_package: self.crate_package,
            module: self.module,
            forbidden_operands: self.forbidden_operands,
            reason: reason.to_string(),
            severity: self.severity,
        }
    }
}

/// An impl-trait boundary: a module's public API must not **return** a written `impl Trait`
/// (return-position `impl Trait` / RPIT). The **existential** complement of [`DynTraitBoundary`]:
/// where that forbids the *dynamic-dispatch* shape (`dyn`), this forbids the *existential* shape —
/// an unnameable type the caller cannot name, store without boxing, or rely on beyond its declared
/// bounds. Governs **return positions only**: argument-position `impl Trait` (APIT) is *universal*
/// (a caller-chosen generic), not an existential leak, and is never governed; `async fn`'s implicit
/// `impl Future` is a distinct compiler-inserted existential, out of scope. Declared in Rust and
/// composed with the other dimensions at the gate.
///
/// Two depths on one boundary type, selected by the builder (mirroring [`DynTraitBoundary`]):
/// - [`must_not_expose_impl_trait`](ImplTraitModuleDraft::must_not_expose_impl_trait) —
///   **shape-only**: an empty operand set, so *any* returned `impl Trait` reacts.
/// - [`must_not_expose_impl_trait_of`](ImplTraitModuleDraft::must_not_expose_impl_trait_of) —
///   **operand-scoped**: only a returned `impl Trait` whose principal trait resolves into the named
///   `forbidden_operands` set reacts.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ImplTraitBoundary {
    pub(crate) crate_package: String,
    pub(crate) module: String,
    /// The forbidden trait operands. **Empty ⇒ shape-only** (any returned `impl Trait` reacts); a
    /// named set ⇒ only a returned `impl Trait` whose principal trait canonicalizes into the set.
    pub(crate) forbidden_operands: Vec<String>,
    pub(crate) reason: String,
    pub(crate) severity: Severity,
}

impl ImplTraitBoundary {
    /// Begin an impl-trait boundary in the crate named `package`.
    pub fn in_crate(package: &str) -> ImplTraitCrateDraft {
        ImplTraitCrateDraft {
            crate_package: package.to_string(),
        }
    }

    /// The crate this boundary governs.
    pub fn crate_package(&self) -> &str {
        &self.crate_package
    }

    /// The governed module path (e.g. `crate::core`).
    pub fn module(&self) -> &str {
        &self.module
    }

    /// The forbidden trait operands. Empty ⇒ shape-only (any returned `impl Trait` reacts); a named
    /// set ⇒ only a returned `impl Trait` whose principal trait resolves into the set reacts.
    pub fn forbidden_operands(&self) -> &[String] {
        &self.forbidden_operands
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

/// An impl-trait boundary awaiting its module anchor.
pub struct ImplTraitCrateDraft {
    crate_package: String,
}

impl ImplTraitCrateDraft {
    /// Anchor the boundary to a module path within the crate (e.g. `crate::core`).
    pub fn module(self, module: &str) -> ImplTraitModuleDraft {
        ImplTraitModuleDraft {
            crate_package: self.crate_package,
            module: module.to_string(),
        }
    }
}

/// A module-anchored boundary awaiting the rule.
pub struct ImplTraitModuleDraft {
    crate_package: String,
    module: String,
}

impl ImplTraitModuleDraft {
    /// Forbid the module's public API from **returning** a written `impl Trait` (RPIT) — any
    /// `impl Trait` at any depth in a public function/method return type (and a public trait
    /// method's declared return). Takes no trait operand — *any* returned `impl Trait` reacts
    /// (shape-only). Argument-position `impl Trait` (APIT) and `async fn`'s implicit `impl Future`
    /// are not governed (stated bounds — the former is universal, the latter a distinct form).
    pub fn must_not_expose_impl_trait(self) -> ImplTraitBoundaryDraft {
        ImplTraitBoundaryDraft {
            crate_package: self.crate_package,
            module: self.module,
            forbidden_operands: Vec::new(),
            severity: Severity::Enforce,
        }
    }

    /// Forbid the module's public API from **returning** a `impl Trait` of any **named trait** —
    /// the operand-scoped depth of [`must_not_expose_impl_trait`](Self::must_not_expose_impl_trait).
    /// A returned `impl Trait` whose **principal trait** (its first trait bound) canonicalizes into
    /// `operands` is a violation; a returned `impl Trait` of any other trait passes (so a seam may
    /// allow ergonomic existentials like `impl Iterator` while forbidding `impl crate::Port`). An
    /// `operands` entry may be an exact trait path or a module prefix, and a re-exported/aliased
    /// facade matches its defining path (the same 渾儀 resolver the forbidden-type rule uses).
    ///
    /// Bounds (stated): an **empty** `operands` set degenerates to shape-only (any returned
    /// `impl Trait`) — loud, never an inert no-op. Auto-trait/lifetime bounds are never operands
    /// (only the principal, first, trait). A principal that does not resolve — a bare std trait
    /// (`impl Iterator`/`impl Future` written bare), a macro/glob re-export — is out of the
    /// resolver's stated coverage and not matched; a *resolvable* operand is never silently passed.
    /// Return-position scoping is inherited (APIT and `async fn` are not governed).
    pub fn must_not_expose_impl_trait_of<I, S>(self, operands: I) -> ImplTraitBoundaryDraft
    where
        I: IntoIterator<Item = S>,
        S: Into<String>,
    {
        ImplTraitBoundaryDraft {
            crate_package: self.crate_package,
            module: self.module,
            forbidden_operands: operands.into_iter().map(Into::into).collect(),
            severity: Severity::Enforce,
        }
    }
}

/// A boundary awaiting severity (optional) and its reason.
pub struct ImplTraitBoundaryDraft {
    crate_package: String,
    module: String,
    forbidden_operands: Vec<String>,
    severity: Severity,
}

impl ImplTraitBoundaryDraft {
    /// Make this an advisory (`warn`) boundary: violations are reported but do not fail the
    /// reaction — the first rung of adoption.
    pub fn warn(mut self) -> Self {
        self.severity = Severity::Warn;
        self
    }

    /// Finish the boundary with its human-readable reason (the repair hint).
    pub fn because(self, reason: &str) -> ImplTraitBoundary {
        ImplTraitBoundary {
            crate_package: self.crate_package,
            module: self.module,
            forbidden_operands: self.forbidden_operands,
            reason: reason.to_string(),
            severity: self.severity,
        }
    }
}

/// An async-exposure boundary: a module's public API must not declare an `async fn`. The
/// **implicit-existential** complement of [`ImplTraitBoundary`]: an `async fn` leaks a
/// compiler-inserted `impl Future` (and commits the seam's contract to an async model), so where
/// impl-trait forbids a *written* `-> impl Future`, this forbids the `async fn` sugar (observed
/// from `syn::Signature.asyncness`). Governs public free fns, public inherent methods, and public
/// trait method declarations; trait-*impl* methods (asyncness dictated by the trait) and private
/// items are excluded. Declarative intent by anchor scoping — "this declared seam is synchronous"
/// (a sync-core/async-edges layering), not a blanket "no async".
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AsyncExposureBoundary {
    pub(crate) crate_package: String,
    pub(crate) module: String,
    pub(crate) reason: String,
    pub(crate) severity: Severity,
}

impl AsyncExposureBoundary {
    /// Begin an async-exposure boundary in the crate named `package`.
    pub fn in_crate(package: &str) -> AsyncExposureCrateDraft {
        AsyncExposureCrateDraft {
            crate_package: package.to_string(),
        }
    }

    /// The crate this boundary governs.
    pub fn crate_package(&self) -> &str {
        &self.crate_package
    }

    /// The governed module path (e.g. `crate::core`).
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

/// An async-exposure boundary awaiting its module anchor.
pub struct AsyncExposureCrateDraft {
    crate_package: String,
}

impl AsyncExposureCrateDraft {
    /// Anchor the boundary to a module path within the crate (e.g. `crate::core`).
    pub fn module(self, module: &str) -> AsyncExposureModuleDraft {
        AsyncExposureModuleDraft {
            crate_package: self.crate_package,
            module: module.to_string(),
        }
    }
}

/// A module-anchored boundary awaiting the rule.
pub struct AsyncExposureModuleDraft {
    crate_package: String,
    module: String,
}

impl AsyncExposureModuleDraft {
    /// Forbid the module's public API from declaring an `async fn` — a public free function, a
    /// public inherent method, or a public trait method declaration. Shape-only (any public
    /// `async fn` at the seam reacts). Governs the implicit `impl Future` existential; a *written*
    /// `-> impl Future` is [`ImplTraitBoundary`]'s domain (a distinct syntactic signal).
    pub fn must_not_expose_async_fn(self) -> AsyncExposureBoundaryDraft {
        AsyncExposureBoundaryDraft {
            crate_package: self.crate_package,
            module: self.module,
            severity: Severity::Enforce,
        }
    }
}

/// A boundary awaiting severity (optional) and its reason.
pub struct AsyncExposureBoundaryDraft {
    crate_package: String,
    module: String,
    severity: Severity,
}

impl AsyncExposureBoundaryDraft {
    /// Make this an advisory (`warn`) boundary: violations are reported but do not fail the
    /// reaction — the first rung of adoption.
    pub fn warn(mut self) -> Self {
        self.severity = Severity::Warn;
        self
    }

    /// Finish the boundary with its human-readable reason (the repair hint).
    pub fn because(self, reason: &str) -> AsyncExposureBoundary {
        AsyncExposureBoundary {
            crate_package: self.crate_package,
            module: self.module,
            reason: reason.to_string(),
            severity: self.severity,
        }
    }
}
