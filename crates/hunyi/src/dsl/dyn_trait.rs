//! Dyn-trait-boundary declaration DSL — [`DynTraitBoundary`] and its draft chain.

use xuanji::Severity;

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
///
/// [`SemanticBoundary`]: crate::SemanticBoundary
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DynTraitBoundary {
    pub(crate) crate_package: String,
    pub(crate) module: String,
    /// The forbidden trait operands. **Empty ⇒ shape-only** (any `dyn` reacts); a named set ⇒
    /// only a `dyn` whose principal trait canonicalizes into the set reacts.
    pub(crate) forbidden_operands: Vec<String>,
    pub(crate) reason: String,
    pub(crate) anchor: Option<String>,
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

/// A dyn-trait boundary awaiting its module anchor.
#[doc(hidden)]
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
#[doc(hidden)]
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
    /// **principal trait** (the sole non-auto trait, whatever its position among the bounds)
    /// canonicalizes into `operands` is a violation;
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
    /// non-auto trait is matched, regardless of its position among the bounds).
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
#[doc(hidden)]
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
            anchor: None,
            severity: self.severity,
        }
    }
}
