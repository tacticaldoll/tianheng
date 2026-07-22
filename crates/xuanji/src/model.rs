//! Core reaction enums and outcome models.

use crate::Report;

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

/// The repair direction a boundary-drift violation points to.
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
                if report.violations.iter().any(|violation| {
                    violation.severity == Severity::Enforce && !violation.baselined
                }) {
                    1
                } else {
                    0
                }
            }
            Outcome::ConstitutionError(_) => 2,
        }
    }
}
