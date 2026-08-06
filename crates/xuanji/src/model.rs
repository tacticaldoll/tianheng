//! Core reaction enums and outcome models.

use crate::{Report, Violation};

/// How strongly a boundary reacts.
///
/// `Enforce` fails the reaction (exit 1); `Warn` reports the violation as advisory
/// without failing — the first rung of adoption before full enforcement.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
#[non_exhaustive]
pub enum Severity {
    /// Violations fail the reaction (exit 1). Default.
    #[default]
    Enforce,
    /// Violations are reported as advisory but do not fail the reaction.
    Warn,
}

impl Severity {
    /// The projection label (`"enforce"` / `"warn"`), single source for report and constitution renderings.
    pub fn as_str(&self) -> &'static str {
        match self {
            Severity::Enforce => "enforce",
            Severity::Warn => "warn",
        }
    }
}

/// Which kind of boundary produced a violation.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
pub enum BoundaryKind {
    /// Crate dependency boundary.
    Crate,
    /// Module structural boundary.
    Module,
    /// Semantic AST boundary — 渾儀 (`hunyi`).
    Semantic,
    /// Runtime boundary — 漏刻 (`louke`).
    Runtime,
}

impl BoundaryKind {
    /// The projection label (`"crate"` / `"module"` / `"semantic"` / `"runtime"`).
    pub fn as_str(&self) -> &'static str {
        match self {
            BoundaryKind::Crate => "crate",
            BoundaryKind::Module => "module",
            BoundaryKind::Semantic => "semantic",
            BoundaryKind::Runtime => "runtime",
        }
    }
}

/// The depth or granularity level of a boundary observation.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
#[non_exhaustive]
pub enum ScanDepth {
    /// Default shallow scan: current module/signature level (<10ms).
    #[default]
    Shallow,
    /// Deep subtree scan: recursive submodule traversal.
    Subtree,
}

impl ScanDepth {
    /// The projection label (`"shallow"` / `"subtree"`), single source for report and constitution renderings.
    pub fn as_str(&self) -> &'static str {
        match self {
            ScanDepth::Shallow => "shallow",
            ScanDepth::Subtree => "subtree",
        }
    }

    /// Returns true if this depth is `Shallow` (useful for default/filtering comparisons).
    pub fn is_shallow(&self) -> bool {
        matches!(self, Self::Shallow)
    }
}

/// The repair direction a boundary-drift violation points to.
///
/// # When a violation carries none
///
/// A polarity names a repair *direction*, so it belongs only to a finding that has one, and
/// [`Violation::polarity`](crate::Violation) being an `Option` is not a gap.
///
/// 圭表's crate and module rules answer through **exhaustive matches returning `Polarity`**, and 渾儀 emits every
/// finding through a context carrying a **non-optional** one — so for those dimensions a new rule variant cannot
/// compile without declaring a direction. That is a stronger guard than a reaction, and a reaction asserting the
/// same thing would be a second copy of a fact the compiler already holds, able to disagree with it.
///
/// The one production path that carries `None` is 漏刻's **probe audit**, and there it is correct rather than
/// missing: a declared seam with no probe is repaired by probing it *or* by dropping the declaration, and a probe
/// naming an undeclared seam by declaring it *or* by deleting the probe. Neither is a deny breach or an allowlist
/// gap, so assigning either value would name a direction that does not exist.
///
/// Written down because an `Option` with no stated rule reads as an omission — measured, a review read it exactly
/// that way and filed it as a rule kind missing its direction. See `runtime-origin-assertion`'s requirement *An
/// audit finding SHALL carry no repair polarity, and that SHALL be stated*.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
pub enum Polarity {
    /// Forbids a specific target; repair is to remove the offending code (`forbid_*` / `must_not_*`).
    DenyBreach,
    /// Permits a set; repair is to remove code or declare intent by widening the set (`restrict_*_to`).
    AllowlistGap,
}

impl Polarity {
    /// The projection label (`"deny_breach"` / `"allowlist_gap"`).
    pub fn as_str(&self) -> &'static str {
        match self {
            Polarity::DenyBreach => "deny_breach",
            Polarity::AllowlistGap => "allowlist_gap",
        }
    }
}

/// The reaction's outcome.
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub enum Outcome {
    /// No enforce-severity boundary was violated (exit 0).
    Clean,
    /// One or more boundaries were violated; carries the full report.
    Violations(Report),
    /// Constitution could not be evaluated — misconfiguration or scan error (exit 2).
    ConstitutionError(String),
}

impl Outcome {
    /// `0` for clean, warn-only, or fully baselined; `1` when a non-baselined enforce violation exists; `2` for constitution error.
    pub fn exit_code(&self) -> u8 {
        match self {
            Outcome::Clean => 0,
            Outcome::Violations(report) => {
                if report.violations.iter().any(Violation::is_active_enforce) {
                    1
                } else {
                    0
                }
            }
            Outcome::ConstitutionError(_) => 2,
        }
    }
}
