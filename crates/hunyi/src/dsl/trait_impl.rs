//! Trait-impl-locality declaration DSL — [`TraitImplBoundary`] and its draft chain.

use xuanji::{RuleKey, Severity};

/// A trait-impl-locality boundary: within a target crate, the named trait may be
/// implemented **only** inside the declared allowed module location(s). An
/// `impl <Trait> for <Type>` block outside them is a violation. Declared in Rust (the
/// single source of truth) and composed with the other dimensions at the gate. This
/// governs *impl locality* — the complement of exposure ([`SemanticBoundary`]) and of the
/// static import boundary. It governs only the target crate's own impl sites; it makes no
/// claim about downstream crates (that would be external trait sealing, an essential gap).
///
/// [`SemanticBoundary`]: crate::SemanticBoundary
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TraitImplBoundary {
    pub(crate) crate_package: String,
    pub(crate) trait_path: String,
    pub(crate) allowed_locations: Vec<String>,
    pub(crate) reason: String,
    pub(crate) anchor: Option<String>,
    pub(crate) severity: Severity,
}

impl TraitImplBoundary {
    /// Stable semantic identity for this trait-implementation locality rule.
    pub fn rule_key(&self) -> RuleKey {
        RuleKey::of(
            "tianheng.rule/hunyi/trait-impl-locality",
            [
                (
                    "allowed_locations",
                    super::canonical_path_set(&self.allowed_locations),
                ),
                ("trait", super::canonical_path(&self.trait_path)),
            ],
        )
    }

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

/// A trait-impl-locality boundary awaiting its trait anchor.
#[doc(hidden)]
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
#[doc(hidden)]
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
#[doc(hidden)]
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
            anchor: None,
            severity: self.severity,
        }
    }
}
