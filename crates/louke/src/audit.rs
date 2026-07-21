//! CI face: probe-coverage audit.
//!
//! The `audit_probe_coverage` entry and its source scanner are the CI face, compiled only under
//! the non-default `audit` feature — a prod dependency on louke compiles none of it; the
//! `tianheng` shell enables it. Why a feature, not a 5th crate: PROJECT.md. The whole module is
//! gated at its declaration in `lib.rs`, so nothing inside needs a per-item
//! `#[cfg(feature = "audit")]`.

use std::collections::HashSet;
use std::path::PathBuf;

use crate::finding::RuntimeFact;
use crate::{BoundaryKind, Outcome, Report, RuntimeBoundary, Severity, Violation, ViolationId};

mod scan;
use scan::{Probe, collect_probes};

#[cfg(test)]
use scan::scan_source;

/// **CI face.** Audit probe coverage against the **declared `RuntimeBoundary` objects** (the
/// authoritative seam set — the constitution, not a source scan for declarations) by scanning
/// the workspace's source inputs for `assert_boundary!` probes. A file input is treated as an
/// exact Cargo target root and walked through reachable modules; a directory input retains the
/// legacy recursive corpus for source compatibility. Reacts, with the static
/// dimensions' exit-code contract, in both directions plus an un-auditable case:
///
/// - **declared-but-unprobed** — a declared seam with no literal probe → a `Violation` at the
///   declaring boundary's severity (a `warn` boundary yields an advisory). Closes the
///   otherwise-essential "declared but never enforced" gap.
/// - **probed-but-undeclared** — a literal probe whose seam is not in the declared set → an
///   enforce `Violation` (a typo against the declared seams).
/// - **un-auditable probe** — an `assert_boundary!` whose seam argument is not a string literal
///   (e.g. a `const`) cannot be traced to a declared seam → an enforce `Violation` naming the
///   site, never a silent skip (a silent skip would be a false negative).
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
/// **`#[path]` relocation (followed, with a narrowed bound):** an **unconditional**
/// `#[path = "…"] mod name;` is followed to its author-chosen file and its probes are counted — the
/// base is the directory a conventional `mod name;` would use, and the loaded file is mod-rs-like,
/// so its own children resolve from its directory. A **`cfg_attr`-wrapped** `#[path]` is
/// cfg-conditional and is **not** followed (following it cfg-blind could read a file rustc does not
/// compile in this configuration): such a module's probes are not counted — a stated bound, so keep
/// a cfg-relocated module's production probes in a conventionally located module instead.
///
/// Compiled only with the non-default `audit` feature (the CI face); see the module note above.
pub fn audit_probe_coverage(declared: &[RuntimeBoundary], source_inputs: &[PathBuf]) -> Outcome {
    let mut probes = Vec::new();
    for input in source_inputs {
        if let Err(message) = collect_probes(input, &mut probes) {
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
                Violation::new(
                    BoundaryKind::Runtime,
                    ViolationId::new(
                        seam,
                        "each runtime seam must be declared exactly once",
                        RuntimeFact::DuplicateSeam {
                            seam: seam.to_string(),
                        }
                        .into_finding(),
                    ),
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
                Violation::new(
                    BoundaryKind::Runtime,
                    ViolationId::new(
                        seam,
                        "every declared runtime seam must be probed",
                        RuntimeFact::UnprobedSeam {
                            seam: seam.to_string(),
                        }
                        .into_finding(),
                    ),
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
                violations.push(Violation::new(
                    BoundaryKind::Runtime,
                    ViolationId::new(
                        seam,
                        "every probe must reference a declared seam",
                        RuntimeFact::UndeclaredProbe { seam: seam.clone() }.into_finding(),
                    ),
                    "an undeclared seam panics at runtime — declare the RuntimeBoundary or fix the probe's seam name".to_string(),
                    Severity::Enforce,
                ));
            }
        }
    }
    // Un-auditable probes: a non-literal seam argument cannot be traced to a declared seam.
    // React rather than silently skip (a silent skip is a false negative). One reaction per
    // file (deduped, sorted) so the finding names where to look and the baseline id is stable.
    let mut unauditable_files: Vec<&str> = probes
        .iter()
        .filter_map(|p| match p {
            Probe::Unauditable { file } => Some(file.as_str()),
            Probe::Literal(_) => None,
        })
        .collect();
    unauditable_files.sort_unstable();
    unauditable_files.dedup();
    for file in unauditable_files {
        // The offending source file is in hand here (the probe scan captured it). Project it
        // into the `file` field as well as the finding text: it is a genuine observation, so
        // reporting `null` would be a dishonest null. This is the one runtime violation with a
        // source location — the seam-level ones above name a seam, not a file.
        violations.push(
            Violation::new(
                BoundaryKind::Runtime,
                ViolationId::new(
                    "<un-auditable probe>",
                    "an assert_boundary! seam must be a string literal to be auditable",
                    RuntimeFact::UnauditableProbe {
                        file: file.to_string(),
                    }
                    .into_finding(),
                ),
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
