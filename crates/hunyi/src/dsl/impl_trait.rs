//! Impl-trait-boundary declaration DSL — [`ImplTraitBoundary`] and its draft chain.

use xuanji::Severity;

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
///
/// [`DynTraitBoundary`]: crate::DynTraitBoundary
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ImplTraitBoundary {
    pub(crate) crate_package: String,
    pub(crate) module: String,
    /// The forbidden trait operands. **Empty ⇒ shape-only** (any returned `impl Trait` reacts); a
    /// named set ⇒ only a returned `impl Trait` whose principal trait canonicalizes into the set.
    pub(crate) forbidden_operands: Vec<String>,
    pub(crate) reason: String,
    pub(crate) anchor: Option<String>,
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

    /// Attach a durable governance anchor (e.g. `"ADR-014"`) — a stable pointer into the
    /// project's governance, distinct from the free-text `reason`. Optional; a boundary with
    /// none projects and reacts exactly as before.
    pub fn with_anchor(mut self, anchor: &str) -> Self {
        self.anchor = Some(anchor.to_string());
        self
    }

    /// The durable governance anchor recorded with the boundary, if any.
    pub fn anchor(&self) -> Option<&str> {
        self.anchor.as_deref()
    }

    /// The boundary's severity (`enforce` or `warn`).
    pub fn severity(&self) -> Severity {
        self.severity
    }
}

/// An impl-trait boundary awaiting its module anchor.
#[doc(hidden)]
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
#[doc(hidden)]
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
    /// A returned `impl Trait` **any of whose non-auto (principal) traits** canonicalizes into
    /// `operands` is a violation; a returned `impl Trait` of any other trait passes (so a seam may
    /// allow ergonomic existentials like `impl Iterator` while forbidding `impl crate::Port`). An
    /// `operands` entry may be an exact trait path or a module prefix, and a re-exported/aliased
    /// facade matches its defining path (the same 渾儀 resolver the forbidden-type rule uses).
    ///
    /// Bounds (stated): an **empty** `operands` set degenerates to shape-only (any returned
    /// `impl Trait`) — loud, never an inert no-op. Auto-trait/lifetime bounds are never operands
    /// (a returned `impl Foo + Bar` may name several non-auto traits — forbidding any one flags it).
    /// A principal that does not resolve — a bare std trait
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
#[doc(hidden)]
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
            anchor: None,
            severity: self.severity,
        }
    }
}
