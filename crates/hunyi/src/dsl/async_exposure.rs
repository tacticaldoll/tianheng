//! Async-exposure declaration DSL — [`AsyncExposureBoundary`] and its draft chain.

use xuanji::Severity;

/// An async-exposure boundary: a module's public API must not declare an `async fn`. The
/// **implicit-existential** complement of [`ImplTraitBoundary`]: an `async fn` leaks a
/// compiler-inserted `impl Future` (and commits the seam's contract to an async model), so where
/// impl-trait forbids a *written* `-> impl Future`, this forbids the `async fn` sugar (observed
/// from `syn::Signature.asyncness`). Governs public free fns, public inherent methods, and public
/// trait method declarations; trait-*impl* methods (asyncness dictated by the trait) and private
/// items are excluded. Declarative intent by anchor scoping — "this declared seam is synchronous"
/// (a sync-core/async-edges layering), not a blanket "no async".
///
/// [`ImplTraitBoundary`]: crate::ImplTraitBoundary
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AsyncExposureBoundary {
    pub(crate) crate_package: String,
    pub(crate) module: String,
    pub(crate) reason: String,
    pub(crate) anchor: Option<String>,
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
    ///
    /// [`ImplTraitBoundary`]: crate::ImplTraitBoundary
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
            anchor: None,
            severity: self.severity,
        }
    }
}
