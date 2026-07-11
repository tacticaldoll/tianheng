//! Unsafe-confinement declaration DSL — [`UnsafeBoundary`] and its draft chain.

use xuanji::Severity;

/// An unsafe-confinement boundary: within a target crate, `unsafe` (blocks, `unsafe fn`/`impl`/
/// `trait`, `unsafe extern`) may appear **only under** the declared subtree(s); a site outside all
/// of them reacts. Declared in Rust (the single source of truth), composed at the gate.
///
/// It governs *where* `unsafe` lives (architectural intent), never *whether* it may exist — the
/// crate-wide "no `unsafe`" case is `#![forbid(unsafe_code)]`'s (stronger, compile-time). An empty
/// allowed set or one naming the crate root is a constitution error, keeping this confinement-only
/// (declarative), not a lint.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UnsafeBoundary {
    pub(crate) crate_package: String,
    pub(crate) allowed_locations: Vec<String>,
    pub(crate) reason: String,
    pub(crate) anchor: Option<String>,
    pub(crate) severity: Severity,
}

impl UnsafeBoundary {
    /// Begin an unsafe-confinement boundary in the crate named `package`.
    pub fn in_crate(package: &str) -> UnsafeCrateDraft {
        UnsafeCrateDraft {
            crate_package: package.to_string(),
        }
    }

    /// The crate this boundary governs.
    pub fn crate_package(&self) -> &str {
        &self.crate_package
    }

    /// The allowed subtree module paths where `unsafe` MAY appear.
    pub fn allowed_locations(&self) -> &[String] {
        &self.allowed_locations
    }

    /// The human-readable reason recorded with the boundary (the repair hint).
    pub fn reason(&self) -> &str {
        &self.reason
    }

    /// Attach a durable governance anchor (e.g. `"ADR-014"`) — a stable pointer into the
    /// project's governance, distinct from the free-text `reason`. Optional.
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

/// An unsafe-confinement boundary awaiting its allowed subtree(s).
#[doc(hidden)]
pub struct UnsafeCrateDraft {
    crate_package: String,
}

impl UnsafeCrateDraft {
    /// Confine `unsafe` to the given subtree module path(s) (`::`-delimited containment, so
    /// `crate::ffi` also allows `crate::ffi::raw`). A site outside all of them reacts. An **empty**
    /// set, or one naming the crate root, is a constitution error (exit 2) — use
    /// `#![forbid(unsafe_code)]` for a crate-wide ban.
    pub fn only_under<I, S>(self, locations: I) -> UnsafeBoundaryDraft
    where
        I: IntoIterator<Item = S>,
        S: Into<String>,
    {
        UnsafeBoundaryDraft {
            crate_package: self.crate_package,
            allowed_locations: locations.into_iter().map(Into::into).collect(),
            severity: Severity::Enforce,
        }
    }
}

/// A boundary awaiting severity (optional) and its reason.
#[doc(hidden)]
pub struct UnsafeBoundaryDraft {
    crate_package: String,
    allowed_locations: Vec<String>,
    severity: Severity,
}

impl UnsafeBoundaryDraft {
    /// Make this an advisory (`warn`) boundary: violations are reported but do not fail the
    /// reaction — the first rung of adoption.
    pub fn warn(mut self) -> Self {
        self.severity = Severity::Warn;
        self
    }

    /// Finish the boundary with its human-readable reason (the repair hint).
    pub fn because(self, reason: &str) -> UnsafeBoundary {
        UnsafeBoundary {
            crate_package: self.crate_package,
            allowed_locations: self.allowed_locations,
            reason: reason.to_string(),
            anchor: None,
            severity: self.severity,
        }
    }
}
