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

/// What an observation was asked to enforce, and how much of the workspace it reached.
///
/// The evidence a clean verdict carries. Every other outcome in this model already carries its own: a
/// [`Violation`] names eleven things about itself, a constitution error names a reason, and
/// [`BoundDecl`](crate::BoundDecl) names what a reaction deliberately does not see. `Clean` carried nothing,
/// which made *I observed a subject and found nothing wrong* and *I had no subject* the same value.
///
/// **The refused combination is relational, and reaching nothing is not the offence.** `declared == 0` with
/// `reached == 0` is a real and protected shape — a static-only adoption composes the semantic dimension with
/// an empty bundle, and refusing that would make its every run exit `2`. What cannot be constructed is
/// *declared something, reached nothing*: a non-zero count would collapse those two, because it cannot tell
/// **nothing to look for** from **looked for nothing**.
///
/// What this buys is stated rather than overclaimed. The constructor is public, because a third-party
/// participant must be able to return an outcome, so nothing here stops one reporting a subject larger than it
/// observed. It converts an **omission** into a **commission**: forgetting becomes impossible and only
/// deliberate misreporting remains — the level [`Observer::bounds`](crate::Observer::bounds) already operates
/// at, where the enforcement lands on the declarer.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
pub struct Subject {
    declared: usize,
    reached: usize,
}

impl Subject {
    /// The subject of an observation, or `None` where it declared something and reached nothing.
    ///
    /// A caller meeting `None` has not found a clean workspace; it has found that it could not observe the one
    /// it was pointed at, which is a constitution error and not a verdict.
    pub fn of(declared: usize, reached: usize) -> Option<Self> {
        (declared == 0 || reached > 0).then_some(Self { declared, reached })
    }

    /// A participant composed with nothing to enforce — reached nothing because there was nothing to reach.
    ///
    /// Named rather than left to `of(0, 0)`, because the shape it describes is a deliberate adoption and not
    /// an edge case a reader should have to recognise from two zeroes.
    pub fn nothing_declared() -> Self {
        Self {
            declared: 0,
            reached: 0,
        }
    }

    /// How many boundaries the observation was asked to enforce.
    pub fn declared(&self) -> usize {
        self.declared
    }

    /// How much of its own corpus the observation reached.
    ///
    /// **The unit is the dimension's, and the figure is not comparable across dimensions.** A static
    /// observation reaches workspace members; a semantic or runtime one reaches compilation roots. Forcing one
    /// unit would make two of the three report a number they did not measure, and the invariant this type
    /// holds does not need them to agree: it asks whether anything was reached, never how much relative to
    /// anyone else.
    pub fn reached(&self) -> usize {
        self.reached
    }
}

/// The reaction's outcome.
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub enum Outcome {
    /// No enforce-severity boundary was violated (exit 0), over the subject it names.
    Clean(Subject),
    /// One or more boundaries were violated; carries the full report.
    Violations(Report),
    /// Constitution could not be evaluated — misconfiguration or scan error (exit 2).
    ConstitutionError(String),
}

impl Outcome {
    /// `0` for clean, warn-only, or fully baselined; `1` when a non-baselined enforce violation exists; `2` for constitution error.
    pub fn exit_code(&self) -> u8 {
        match self {
            Outcome::Clean(_) => 0,
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
