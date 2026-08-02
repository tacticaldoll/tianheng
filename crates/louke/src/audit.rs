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
use scan::{DEFAULT_MARKERS, Probe, collect_probes_with_markers, common_ancestor};

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
        ViolationId::new(target, rule_key, finding.fact().clone()),
        rule,
        finding.text(),
        reason,
        severity,
    )
}

#[cfg(test)]
use scan::scan_source;

/// **CI face.** Audit probe coverage against the **declared `RuntimeBoundary` objects** using
/// the default `["assert_boundary"]` probe macro marker list. Delegates to
/// [`audit_probe_coverage_with_markers`].
pub fn audit_probe_coverage(declared: &[RuntimeBoundary], source_inputs: &[PathBuf]) -> Outcome {
    audit_probe_coverage_with_markers(declared, source_inputs, DEFAULT_MARKERS)
}

/// **CI face with custom probe markers.** Audit probe coverage against the **declared
/// `RuntimeBoundary` objects** (the authoritative seam set — the constitution, not a source scan
/// for declarations) by scanning the workspace's source inputs for probe macro invocations
/// matching any identifier in `markers` (defaulting to `["assert_boundary"]` via
/// [`audit_probe_coverage`]).
///
/// A file input is treated as an exact Cargo target root and walked through reachable modules;
/// a directory input retains the legacy recursive corpus for source compatibility. Reacts,
/// with the static dimensions' exit-code contract, in three directions:
///
/// - **declared-but-unprobed** — a declared seam with no literal probe → a `Violation` at the
///   declaring boundary's severity (a `warn` boundary yields an advisory). Closes the
///   otherwise-essential "declared but never enforced" gap.
/// - **probed-but-undeclared** — a literal probe whose seam is not in the declared set → an
///   enforce `Violation` (a typo against the declared seams).
/// - **un-auditable probe** — a probe macro whose seam argument is not a string literal
///   (e.g. a `const`) cannot be traced to a declared seam → an enforce `Violation` naming the
///   site, never a silent skip (a silent skip would be a false negative). Its identity's `file`
///   field is labeled relative to the common ancestor of every `source_inputs` root passed to this
///   call (the real caller's actual checkout root, by construction), never the raw absolute path
///   — so a recorded baseline stays valid across a different clone location or CI runner.
///
/// Declarations come from the passed objects, so an unconventionally spelled `RuntimeBoundary::at`
/// can no longer hide a seam. The probe scan is build/CI-time only (std-only, comment- and
/// string-literal-aware including raw/byte strings); source outside a member's lib/bin target
/// subtree is out of scope (the same bound as the semantic dimension). It does NOT observe the
/// live install registry — install-vs-constitution consistency is the prod face's runtime
/// fail-closed concern; this verifies coverage against the declared seams and the source.
///
/// **Stated bound (lexical, not semantic):** the scan is textual and does not evaluate `cfg`.
/// A probe behind a non-production `#[cfg(...)]` (e.g. `#[cfg(test)]`) is still counted as
/// covering its seam, so a seam whose *only* probe is compiled out of the production binary
/// would be reported covered. Keep a seam's production probe out of non-production `cfg`s.
///
/// **`#[path]` relocation (followed, union rather than pick-one):** an **unconditional**
/// `#[path = "…"] mod name;` is followed to its author-chosen file and its probes are counted — the
/// base is the directory a conventional `mod name;` would use, and the loaded file is mod-rs-like,
/// so its own children resolve from its directory. A **`cfg_attr`-wrapped** `#[path]` (one or more
/// on the same declaration, each gated by its own platform predicate) is cfg-conditional, so `cfg`
/// never resolves the module — `cfg_attr` never removes the `mod` item the way a bare `#[cfg]`
/// does — but its own target is followed too, resolved the identical way an unconditional `#[path]`
/// is: EVERY candidate that exists on disk is counted, unioned with the conventional `name.rs` if it
/// too exists, since cfg-blind observation cannot know which one a given build actually compiles.
/// Absence is tolerated only when NEITHER the conventional file NOR any `cfg_attr` target resolves
/// anywhere, and the declaration carries no other cfg-conditional gate (a bare `#[cfg]` or
/// transparent-macro-arm membership) — that combination is a genuinely broken reference on every
/// configuration, so it fails loud (a constitution error), never a silent pass.
///
/// Compiled only with the non-default `audit` feature (the CI face); see the module note above.
pub fn audit_probe_coverage_with_markers(
    declared: &[RuntimeBoundary],
    source_inputs: &[PathBuf],
    markers: &[&str],
) -> Outcome {
    if markers.is_empty() {
        return Outcome::ConstitutionError("custom probe markers list cannot be empty".to_string());
    }
    for &marker in markers {
        if !scan::is_valid_macro_marker(marker) {
            return Outcome::ConstitutionError(format!(
                "custom probe marker '{marker}' is not a valid Rust macro identifier"
            ));
        }
    }
    // Every un-auditable probe's `file` identity field is labeled relative to this anchor (the
    // common ancestor of every root passed here — the real caller's actual checkout/workspace
    // root by construction) rather than as a raw absolute path, so the identity — and any baseline
    // recorded against it — stays checkout-independent. See `scan::labeled`/`common_ancestor`.
    let anchor = common_ancestor(source_inputs);
    let mut probes = Vec::new();
    for input in source_inputs {
        if let Err(message) = collect_probes_with_markers(input, &anchor, markers, &mut probes) {
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
    // (marker, file, owner-qualified enclosing item, expression text) — deduped, sorted — so two
    // textually distinct non-literal probes in the same file are distinct findings and
    // baselining one cannot mask another; two byte-identical occurrences in the same file and
    // the same marker and enclosing item still collapse to one (a stated bound: at that
    // granularity no further source content distinguishes them, mirroring `module-boundary`'s
    // "same import on multiple lines is one violation").
    let mut unauditable: Vec<(&str, &str, &str, &str)> = probes
        .iter()
        .filter_map(|p| match p {
            Probe::Unauditable {
                marker,
                file,
                owner,
                expr,
            } => Some((
                marker.as_str(),
                file.as_str(),
                owner.as_str(),
                expr.as_str(),
            )),
            Probe::Literal(_) => None,
        })
        .collect();
    unauditable.sort_unstable();
    unauditable.dedup();
    for (marker, file, owner, expr) in unauditable {
        // The offending source file, owner, and expression are in hand here (the probe scan
        // captured them). Project the file into the `file` field as well as the finding text: it
        // is a genuine observation, so reporting `null` would be a dishonest null. This is the
        // one runtime violation with a source location — the seam-level ones above name a seam,
        // not a file.
        violations.push(
            audit_violation(
                "<un-auditable probe>",
                "a configured probe marker's seam must be a string literal to be auditable",
                AuditRule::LiteralProbeSeam.key(),
                RuntimeFact::UnauditableProbe {
                    marker: marker.to_string(),
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
