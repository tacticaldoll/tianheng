//! Visibility-boundary declaration DSL — [`VisibilityBoundary`] and its draft chain.

use xuanji::Severity;

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
    pub(crate) anchor: Option<String>,
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
            anchor: None,
            severity: self.severity,
        }
    }
}
