//! Visibility-boundary declaration DSL — [`VisibilityBoundary`] and its draft chain.

use xuanji::{RuleKey, Severity};

use crate::rules::{VISIBILITY_MODULE_RULE, VISIBILITY_RULE, VISIBILITY_SUPER_RULE};

/// The maximum declared visibility a governed module's direct items may carry. An item whose
/// declared-visibility rank is strictly above the ceiling reacts; at or below it passes.
/// (`Public` is deliberately not a ceiling — it would never react.)
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum VisibilityCeiling {
    /// Allow up to `pub(crate)`; react on bare `pub`. The `must_not_declare_pub` case.
    Crate,
    /// Allow up to `pub(super)`; react on `pub` and `pub(crate)`.
    Super,
    /// Allow only module-private (private / `pub(self)`); react on any `pub`-family keyword.
    Module,
}

impl VisibilityCeiling {
    fn key_label(self) -> &'static str {
        match self {
            VisibilityCeiling::Crate => "crate",
            VisibilityCeiling::Super => "super",
            VisibilityCeiling::Module => "module",
        }
    }

    /// The ceiling's rank on the `pub`(3) > `pub(crate)`(2) > `pub(super)`(1) > private(0) scale;
    /// an item reacts iff its own rank is strictly greater.
    pub(crate) fn rank(self) -> u8 {
        match self {
            VisibilityCeiling::Crate => 2,
            VisibilityCeiling::Super => 1,
            VisibilityCeiling::Module => 0,
        }
    }

    /// The rule label for this ceiling. `Crate` keeps the legacy `must_not_declare_pub` string
    /// verbatim (so its findings and baselines never churn); `Super`/`Module` are distinct. Public
    /// so the shell's projection renders the same label the reaction stamps (one source).
    pub fn rule(self) -> &'static str {
        match self {
            VisibilityCeiling::Crate => VISIBILITY_RULE,
            VisibilityCeiling::Super => VISIBILITY_SUPER_RULE,
            VisibilityCeiling::Module => VISIBILITY_MODULE_RULE,
        }
    }
}

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
    pub(crate) ceiling: VisibilityCeiling,
}

impl VisibilityBoundary {
    /// Stable semantic identity for this visibility-ceiling rule.
    pub fn rule_key(&self) -> RuleKey {
        RuleKey::of(
            "tianheng.rule/hunyi/visibility-ceiling",
            [("ceiling", self.ceiling.key_label())],
        )
    }

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

    /// The boundary's maximum-visibility ceiling.
    pub fn ceiling(&self) -> VisibilityCeiling {
        self.ceiling
    }
}

/// A visibility boundary awaiting its module anchor.
#[doc(hidden)]
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
#[doc(hidden)]
pub struct VisibilityModuleDraft {
    crate_package: String,
    module: String,
}

impl VisibilityModuleDraft {
    /// Forbid the module from declaring any bare-`pub` direct item — sugar for
    /// [`max_visibility`](Self::max_visibility)`(VisibilityCeiling::Crate)`, byte-identical in
    /// behavior, rule string, and findings.
    pub fn must_not_declare_pub(self) -> VisibilityBoundaryDraft {
        self.max_visibility(VisibilityCeiling::Crate)
    }

    /// Forbid the module from declaring any direct item more visible than `ceiling`.
    pub fn max_visibility(self, ceiling: VisibilityCeiling) -> VisibilityBoundaryDraft {
        VisibilityBoundaryDraft {
            crate_package: self.crate_package,
            module: self.module,
            severity: Severity::Enforce,
            ceiling,
        }
    }
}

/// A boundary awaiting severity (optional) and its reason.
#[doc(hidden)]
pub struct VisibilityBoundaryDraft {
    crate_package: String,
    module: String,
    severity: Severity,
    ceiling: VisibilityCeiling,
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
            ceiling: self.ceiling,
        }
    }
}
