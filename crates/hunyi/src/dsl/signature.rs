//! Signature-coupling declaration DSL — [`SignatureBoundary`] and its draft chain.

use xuanji::{RuleKey, Severity};

/// A semantic boundary: the public API of a module must not **expose** any forbidden
/// type. Declared in Rust (the single source of truth), alongside — and composed with —
/// the static constitution at the gate. Each dimension owns its own declaration DSL and
/// expresses findings in the shared 璇璣 model; the shell merges them into one reaction.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SignatureBoundary {
    pub(crate) crate_package: String,
    pub(crate) module: String,
    pub(crate) forbidden: Vec<String>,
    pub(crate) reason: String,
    pub(crate) anchor: Option<String>,
    pub(crate) severity: Severity,
    /// Opt-in depth (`semantic-trait-impl-exposure`): also observe the module's trait `impl`
    /// blocks' impl-site-authored positions. `false` keeps the v1 signature-coupling surface.
    pub(crate) including_trait_impls: bool,
}

impl SignatureBoundary {
    /// Stable semantic identity for this signature-coupling rule.
    pub fn rule_key(&self) -> RuleKey {
        RuleKey::of(
            "tianheng.rule/hunyi/signature-exposure",
            [
                ("forbidden", super::canonical_path_set(&self.forbidden)),
                (
                    "including_trait_impls",
                    self.including_trait_impls.to_string(),
                ),
            ],
        )
    }

    /// Begin a semantic boundary in the crate named `package`.
    pub fn in_crate(package: &str) -> SignatureCrateDraft {
        SignatureCrateDraft {
            crate_package: package.to_string(),
        }
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

    /// Whether the boundary also observes trait `impl` blocks (the opt-in
    /// `semantic-trait-impl-exposure` depth). `false` is the v1 signature-coupling surface.
    pub fn including_trait_impls(&self) -> bool {
        self.including_trait_impls
    }
}

crate::dsl::boundary_common!(SignatureBoundary, SignatureBoundaryDraft);

/// A semantic boundary awaiting its module anchor.
#[doc(hidden)]
pub struct SignatureCrateDraft {
    crate_package: String,
}

impl SignatureCrateDraft {
    /// Anchor the boundary to a module path within the crate (e.g. `crate::domain`).
    pub fn module(self, module: &str) -> SignatureModuleDraft {
        SignatureModuleDraft {
            crate_package: self.crate_package,
            module: module.to_string(),
        }
    }
}

/// A module-anchored boundary awaiting the forbidden set.
#[doc(hidden)]
pub struct SignatureModuleDraft {
    crate_package: String,
    module: String,
}

impl SignatureModuleDraft {
    /// Forbid the module's public API from exposing the given type path or module prefix
    /// (`::`-delimited containment, so `crate::infra` also forbids `crate::infra::db::Pool`).
    pub fn must_not_expose(self, path: &str) -> SignatureBoundaryDraft {
        SignatureBoundaryDraft {
            crate_package: self.crate_package,
            module: self.module,
            forbidden: vec![path.to_string()],
            severity: Severity::Enforce,
            including_trait_impls: false,
        }
    }
}

/// A boundary awaiting severity (optional) and its reason.
#[doc(hidden)]
pub struct SignatureBoundaryDraft {
    crate_package: String,
    module: String,
    forbidden: Vec<String>,
    severity: Severity,
    including_trait_impls: bool,
}

impl SignatureBoundaryDraft {
    /// Also forbid exposing another type path / module prefix (a boundary MAY forbid more
    /// than one).
    pub fn and_not_expose(mut self, path: &str) -> Self {
        self.forbidden.push(path.to_string());
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
    pub fn because(self, reason: &str) -> SignatureBoundary {
        SignatureBoundary {
            crate_package: self.crate_package,
            module: self.module,
            forbidden: self.forbidden,
            reason: reason.to_string(),
            anchor: None,
            severity: self.severity,
            including_trait_impls: self.including_trait_impls,
        }
    }
}
