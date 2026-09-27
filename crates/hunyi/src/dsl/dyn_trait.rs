//! Dyn-trait-boundary declaration DSL — [`DynTraitBoundary`] and its draft chain.

use xuanji::{RuleKey, Severity};

/// The target matching policy of a dyn-trait boundary: shape-only, principal-trait operand-scoped,
/// or auto-trait bound-scoped.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum DynTraitTarget {
    Any,
    Principal(Vec<String>),
    AutoBounds(Vec<String>),
}

/// A dyn-trait boundary: a module's public API must not **expose** trait-object (`dyn`)
/// syntax. The type-shape complement of [`SignatureBoundary`] (signature-coupling): where
/// that forbids an exposed *named type*, this forbids an exposed *type shape* — a `dyn`
/// node at any depth in the governed public surface. Internal `dyn` is never a violation —
/// this governs exposure across the declared seam, not internal dynamic dispatch, so it is
/// intent (by anchor scoping), not a lint. Declared in Rust and composed with the other
/// dimensions at the gate.
///
/// Three policies on one boundary type, selected by the builder:
/// - [`must_not_expose_dyn`](DynTraitModuleDraft::must_not_expose_dyn) — **shape-only**: an
///   empty operand set, so *any* exposed `dyn` reacts.
/// - [`must_not_expose_dyn_of`](DynTraitModuleDraft::must_not_expose_dyn_of) — **operand-scoped**:
///   only a `dyn` whose principal trait resolves into the named `forbidden_operands` set reacts.
/// - [`must_not_expose_dyn_bounded_by`](DynTraitModuleDraft::must_not_expose_dyn_bounded_by) —
///   **auto-bound-scoped**: only a `dyn` whose own bounds carry one of the named auto traits reacts.
///
/// [`SignatureBoundary`]: crate::SignatureBoundary
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DynTraitBoundary {
    pub(crate) crate_package: String,
    pub(crate) module: String,
    pub(crate) target: DynTraitTarget,
    pub(crate) reason: String,
    pub(crate) anchor: Option<String>,
    pub(crate) severity: Severity,
}

impl DynTraitBoundary {
    /// Stable semantic identity for this dyn-trait exposure rule.
    pub fn rule_key(&self) -> RuleKey {
        match &self.target {
            DynTraitTarget::Any => RuleKey::of(
                "tianheng.rule/hunyi/dyn-trait-exposure",
                [("forbidden_operands", "[]".to_string())],
            ),
            DynTraitTarget::Principal(operands) => RuleKey::of(
                "tianheng.rule/hunyi/dyn-trait-exposure",
                [("forbidden_operands", super::canonical_path_set(operands))],
            ),
            DynTraitTarget::AutoBounds(bounds) => {
                let json = crate::resolve::auto_bound_leaves(bounds, "dyn-trait")
                    .map(|leaves| {
                        serde_json::to_string(&leaves.into_iter().collect::<Vec<_>>())
                            .expect("serialized leaves")
                    })
                    .unwrap_or_else(|_| super::canonical_path_set(bounds));
                RuleKey::of(
                    "tianheng.rule/hunyi/dyn-trait-auto-bound",
                    [("forbidden_auto_bounds", json)],
                )
            }
        }
    }

    /// Begin a dyn-trait boundary in the crate named `package`.
    pub fn in_crate(package: &str) -> DynTraitCrateDraft {
        DynTraitCrateDraft {
            crate_package: package.to_string(),
        }
    }

    /// The governed module path (e.g. `crate::core`).
    pub fn module(&self) -> &str {
        &self.module
    }

    /// The forbidden trait operands. Empty ⇒ shape-only or auto-bound; a named set ⇒
    /// only a `dyn` whose principal trait resolves into the set reacts.
    pub fn forbidden_operands(&self) -> &[String] {
        match &self.target {
            DynTraitTarget::Principal(operands) => operands,
            _ => &[],
        }
    }

    /// The forbidden auto-trait bounds.
    pub fn forbidden_auto_bounds(&self) -> &[String] {
        match &self.target {
            DynTraitTarget::AutoBounds(bounds) => bounds,
            _ => &[],
        }
    }

    /// The human-readable reason recorded with the boundary (the repair hint).
    pub fn reason(&self) -> &str {
        &self.reason
    }
}

crate::dsl::boundary_common!(DynTraitBoundary, DynTraitBoundaryDraft);

/// A dyn-trait boundary awaiting its module anchor.
#[doc(hidden)]
pub struct DynTraitCrateDraft {
    crate_package: String,
}

impl DynTraitCrateDraft {
    /// Anchor the boundary to a module path within the crate (e.g. `crate::core`).
    /// Written from the crate root — `crate` or `crate::a::b`, a raw identifier read as its plain
    /// form; any other spelling, or a module the crate does not declare, is a constitution error (exit 2).
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
            target: DynTraitTarget::Any,
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
    /// never silently passed. Auto-trait / lifetime bounds are never principal operands (only
    /// the non-auto trait is matched, regardless of its position among the bounds). A forbidden
    /// operand whose leaf names an auto trait is a constitution error; remove that entry.
    pub fn must_not_expose_dyn_of<I, S>(self, operands: I) -> DynTraitBoundaryDraft
    where
        I: IntoIterator<Item = S>,
        S: Into<String>,
    {
        DynTraitBoundaryDraft {
            crate_package: self.crate_package,
            module: self.module,
            target: DynTraitTarget::Principal(operands.into_iter().map(Into::into).collect()),
            severity: Severity::Enforce,
        }
    }

    /// Forbid the module's public API from exposing a `dyn` trait object carrying any of the
    /// named **auto-trait bounds** (e.g. `Send`, `Sync`).
    pub fn must_not_expose_dyn_bounded_by<I, S>(self, bounds: I) -> DynTraitBoundaryDraft
    where
        I: IntoIterator<Item = S>,
        S: Into<String>,
    {
        DynTraitBoundaryDraft {
            crate_package: self.crate_package,
            module: self.module,
            target: DynTraitTarget::AutoBounds(bounds.into_iter().map(Into::into).collect()),
            severity: Severity::Enforce,
        }
    }
}

/// A boundary awaiting severity (optional) and its reason.
#[doc(hidden)]
pub struct DynTraitBoundaryDraft {
    crate_package: String,
    module: String,
    pub(crate) target: DynTraitTarget,
    severity: Severity,
}

impl DynTraitBoundaryDraft {
    /// Finish the boundary with its human-readable reason (the repair hint).
    pub fn because(self, reason: &str) -> DynTraitBoundary {
        DynTraitBoundary {
            crate_package: self.crate_package,
            module: self.module,
            target: self.target,
            reason: reason.to_string(),
            anchor: None,
            severity: self.severity,
        }
    }
}
