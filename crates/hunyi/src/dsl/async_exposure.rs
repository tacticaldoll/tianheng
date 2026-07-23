use xuanji::{RuleKey, ScanDepth, Severity};

/// An async-exposure boundary: a module's public API must not declare an `async fn`. The
/// **implicit-existential** complement of [`ImplTraitBoundary`]: an `async fn` leaks a
/// compiler-inserted `impl Future` (and commits the seam's contract to an async model), so where
/// impl-trait forbids a *written* `-> impl Future`, this forbids the `async fn` sugar (observed
/// from `syn::Signature.asyncness`). Governs public free fns, public inherent methods, and public
/// trait method declarations; trait-*impl* methods (asyncness dictated by the trait) and private
/// items are excluded. Declarative intent by anchor scoping — "this declared seam is synchronous"
/// (a sync-core/async-edges layering), not a blanket "no async".
///
/// **Scope.** By default the boundary governs the anchored module's **own** items only (the
/// declared seam). Call [`including_submodules`](AsyncExposureBoundaryDraft::including_submodules)
/// on the rule draft to descend the anchored module's **whole subtree**, so a public `async fn` in
/// any descendant module reacts too — the sans-I/O-purity use ("no async anywhere under this
/// kernel"), where anchoring at `crate` governs the whole crate.
///
/// [`ImplTraitBoundary`]: crate::ImplTraitBoundary
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AsyncExposureBoundary {
    pub(crate) crate_package: String,
    pub(crate) module: String,
    pub(crate) reason: String,
    pub(crate) anchor: Option<String>,
    pub(crate) severity: Severity,
    /// When set, the reaction descends the anchored module's whole subtree, not just its own
    /// items. Off by default, so an existing boundary projects and reacts byte-identically.
    pub(crate) including_submodules: bool,
}

impl AsyncExposureBoundary {
    /// Stable semantic identity for this async-exposure rule.
    pub fn rule_key(&self) -> RuleKey {
        RuleKey::of(
            "tianheng.rule/hunyi/async-exposure",
            [(
                "including_submodules",
                self.including_submodules.to_string(),
            )],
        )
    }

    /// The observation scan depth for this boundary.
    pub fn scan_depth(&self) -> ScanDepth {
        if self.including_submodules {
            ScanDepth::Subtree
        } else {
            ScanDepth::Shallow
        }
    }

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

    /// Whether the reaction descends the anchored module's whole subtree (`true`) or governs only
    /// its own items (`false`, the default).
    pub fn including_submodules(&self) -> bool {
        self.including_submodules
    }
}

/// An async-exposure boundary awaiting its module anchor.
#[doc(hidden)]
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
#[doc(hidden)]
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
            including_submodules: false,
        }
    }
}

/// A boundary awaiting severity (optional) and its reason.
#[doc(hidden)]
pub struct AsyncExposureBoundaryDraft {
    crate_package: String,
    module: String,
    severity: Severity,
    including_submodules: bool,
}

impl AsyncExposureBoundaryDraft {
    /// Configure the observation scan depth / granularity level.
    pub fn depth(mut self, depth: ScanDepth) -> Self {
        self.including_submodules = !depth.is_shallow();
        self
    }

    /// Make this an advisory (`warn`) boundary: violations are reported but do not fail the
    /// reaction — the first rung of adoption.
    pub fn warn(mut self) -> Self {
        self.severity = Severity::Warn;
        self
    }

    /// Descend the anchored module's **whole subtree**: a public `async fn` in any descendant
    /// module reacts, not only one at the anchored module's own seam. Off by default (the boundary
    /// governs the declared seam alone); with it, anchoring at `crate` governs the whole crate —
    /// the sans-I/O-purity shape. Mirrors [`SemanticBoundary`]'s `including_trait_impls` opt-in:
    /// projected only when set, so a bare boundary stays byte-identical.
    ///
    /// [`SemanticBoundary`]: crate::SemanticBoundary
    pub fn including_submodules(self) -> Self {
        self.depth(ScanDepth::Subtree)
    }

    /// Finish the boundary with its human-readable reason (the repair hint).
    pub fn because(self, reason: &str) -> AsyncExposureBoundary {
        AsyncExposureBoundary {
            crate_package: self.crate_package,
            module: self.module,
            reason: reason.to_string(),
            anchor: None,
            severity: self.severity,
            including_submodules: self.including_submodules,
        }
    }
}
