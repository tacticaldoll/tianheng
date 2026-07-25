//! CI face: probe-coverage audit.
//!
//! The `audit_probe_coverage` entry and its source scanner are the CI face, compiled only under
//! the non-default `audit` feature — a prod dependency on louke compiles none of it; the
//! `tianheng` shell enables it. Why a feature, not a 5th crate: PROJECT.md. The whole module is
//! gated at its declaration in `lib.rs`, so nothing inside needs a per-item
//! `#[cfg(feature = "audit")]`.

use std::collections::HashSet;
use std::path::PathBuf;
use xuanji::RuleKey;

use crate::finding::RuntimeFact;
use crate::registry::UNDECLARED_SEAM_REPAIR_HINT;
use crate::{BoundaryKind, Outcome, Report, RuntimeBoundary, Severity, Violation, ViolationId};

mod scan;
use scan::{DEFAULT_MARKERS, Probe, collect_probes_with_markers};

#[derive(Clone, Copy)]
enum AuditRule {
    UniqueSeamDeclaration,
    DeclaredSeamProbed,
    ProbeDeclaredSeam,
    LiteralProbeSeam,
}

impl AuditRule {
    fn rule_type(self) -> &'static str {
        match self {
            Self::UniqueSeamDeclaration => "tianheng.rule/louke/unique-seam-declaration",
            Self::DeclaredSeamProbed => "tianheng.rule/louke/declared-seam-probed",
            Self::ProbeDeclaredSeam => "tianheng.rule/louke/probe-declared-seam",
            Self::LiteralProbeSeam => "tianheng.rule/louke/literal-probe-seam",
        }
    }

    fn key(self) -> RuleKey {
        RuleKey::of(self.rule_type(), std::iter::empty::<(&str, &str)>())
    }
}

fn audit_violation(
    target: &str,
    rule: &str,
    rule_key: RuleKey,
    fact: RuntimeFact,
    reason: String,
    severity: Severity,
) -> Violation {
    let finding = fact.into_finding();
    Violation::new(
        BoundaryKind::Runtime,
        ViolationId::new(target, rule_key, finding.key().clone()),
        rule,
        finding.text(),
        reason,
        severity,
    )
}

#[cfg(test)]
use scan::scan_source;

/// **CI face.** Audit probe coverage against the **declared `RuntimeBoundary` objects** using
/// the default `["assert_boundary"]` probe macro marker.
pub fn audit_probe_coverage(declared: &[RuntimeBoundary], source_inputs: &[PathBuf]) -> Outcome {
    audit_probe_coverage_with_markers(declared, source_inputs, DEFAULT_MARKERS)
}

/// **CI face with custom markers.** Audit probe coverage against the **declared `RuntimeBoundary`
/// objects** using a custom list of probe macro marker names (e.g. `["assert_boundary", "my_seam"]`).
pub fn audit_probe_coverage_with_markers(
    declared: &[RuntimeBoundary],
    source_inputs: &[PathBuf],
    markers: &[&str],
) -> Outcome {
    let mut probes = Vec::new();
    for input in source_inputs {
        if let Err(message) = collect_probes_with_markers(input, markers, &mut probes) {
            return Outcome::ConstitutionError(message);
        }
    }
    let probed_set: HashSet<&str> = probes
        .iter()
        .filter_map(|p| match p {
            Probe::Literal(seam) => Some(seam.as_str()),
            Probe::Unauditable { .. } => None,
        })
        .collect();
    let declared_set: HashSet<&str> = declared.iter().map(RuntimeBoundary::seam).collect();
    let mut violations = Vec::new();

    // Duplicate declared seam: the prod `install` fails loud on it (a duplicate would silently
    // shadow the earlier boundary); catch it at CI too — one enforce violation per duplicated
    // seam — so the misconfiguration surfaces before it reaches a running binary.
    let mut seen_decl = HashSet::new();
    let mut dup_reported = HashSet::new();
    for boundary in declared {
        let seam = boundary.seam();
        if !seen_decl.insert(seam) && dup_reported.insert(seam) {
            violations.push(
                audit_violation(
                    seam,
                    "each runtime seam must be declared exactly once",
                    AuditRule::UniqueSeamDeclaration.key(),
                    RuntimeFact::DuplicateSeam {
                        seam: seam.to_string(),
                    },
                    "a duplicate declaration would silently shadow the earlier boundary at install"
                        .to_string(),
                    Severity::Enforce,
                )
                .with_anchor(boundary.anchor().map(String::from)),
            );
        }
    }

    // Declared but never probed: the boundary is never enforced at runtime. Reacts at the
    // declaring boundary's severity (a warn boundary is advisory, not a CI failure).
    let mut seen = HashSet::new();
    for boundary in declared {
        let seam = boundary.seam();
        if !probed_set.contains(seam) && seen.insert(seam) {
            violations.push(
                audit_violation(
                    seam,
                    "every declared runtime seam must be probed",
                    AuditRule::DeclaredSeamProbed.key(),
                    RuntimeFact::UnprobedSeam {
                        seam: seam.to_string(),
                    },
                    "a RuntimeBoundary with no probe is never enforced at runtime".to_string(),
                    boundary.severity(),
                )
                .with_anchor(boundary.anchor().map(String::from)),
            );
        }
    }
    // Probed but never declared: the probe references an undeclared seam, which panics at
    // runtime — catch the typo at CI instead of crashing production.
    let mut seen_probe = HashSet::new();
    for probe in &probes {
        if let Probe::Literal(seam) = probe {
            if !declared_set.contains(seam.as_str()) && seen_probe.insert(seam.as_str()) {
                violations.push(audit_violation(
                    seam,
                    "every probe must reference a declared seam",
                    AuditRule::ProbeDeclaredSeam.key(),
                    RuntimeFact::UndeclaredProbe { seam: seam.clone() },
                    format!("an undeclared seam panics at runtime — {UNDECLARED_SEAM_REPAIR_HINT}"),
                    Severity::Enforce,
                ));
            }
        }
    }
    // Un-auditable probes: a non-literal seam argument cannot be traced to a declared seam.
    // React rather than silently skip (a silent skip is a false negative). One reaction per
    // (file, owner-qualified enclosing item, expression text) — deduped, sorted — so two
    // textually distinct non-literal probes in the same file are distinct findings and
    // baselining one cannot mask another; two byte-identical occurrences in the same file and
    // the same enclosing item still collapse to one (a stated bound: at that granularity no
    // further source content distinguishes them, mirroring `module-boundary`'s "same import on
    // multiple lines is one violation").
    let mut unauditable: Vec<(&str, &str, &str)> = probes
        .iter()
        .filter_map(|p| match p {
            Probe::Unauditable { file, owner, expr } => {
                Some((file.as_str(), owner.as_str(), expr.as_str()))
            }
            Probe::Literal(_) => None,
        })
        .collect();
    unauditable.sort_unstable();
    unauditable.dedup();
    for (file, owner, expr) in unauditable {
        // The offending source file, owner, and expression are in hand here (the probe scan
        // captured them). Project the file into the `file` field as well as the finding text: it
        // is a genuine observation, so reporting `null` would be a dishonest null. This is the
        // one runtime violation with a source location — the seam-level ones above name a seam,
        // not a file.
        violations.push(
            audit_violation(
                "<un-auditable probe>",
                "an assert_boundary! seam must be a string literal to be auditable",
                AuditRule::LiteralProbeSeam.key(),
                RuntimeFact::UnauditableProbe {
                    file: file.to_string(),
                    owner: owner.to_string(),
                    expr: expr.to_string(),
                },
                "spell the seam as a string literal so probe coverage can be verified".to_string(),
                Severity::Enforce,
            )
            .with_file(Some(file.to_string())),
        );
    }
    if violations.is_empty() {
        Outcome::Clean
    } else {
        Outcome::Violations(Report::new(violations))
    }
}

#[cfg(test)]
mod tests;
