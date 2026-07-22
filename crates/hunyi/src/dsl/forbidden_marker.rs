//! Forbidden-marker declaration DSL — [`ForbiddenMarkerBoundary`] and its draft chain.

use xuanji::{RuleKey, Severity};

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
    pub(crate) anchor: Option<String>,
    pub(crate) severity: Severity,
}

impl ForbiddenMarkerBoundary {
    /// Stable semantic identity for this forbidden-marker acquisition rule.
    pub fn rule_key(&self) -> RuleKey {
        RuleKey::of(
            "tianheng.rule/hunyi/forbidden-marker",
            [("forbidden", super::canonical_path_set(&self.forbidden))],
        )
    }

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

    /// The boundary's severity.
    pub fn severity(&self) -> Severity {
        self.severity
    }
}

/// A forbidden-marker boundary awaiting its module-subtree anchor.
#[doc(hidden)]
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
#[doc(hidden)]
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
#[doc(hidden)]
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
            anchor: None,
            severity: self.severity,
        }
    }
}
