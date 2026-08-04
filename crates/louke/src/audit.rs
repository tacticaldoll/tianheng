//! CI face: probe-coverage audit.
//!
//! The `audit_probe_coverage` entry and its source scanner are the CI face, compiled only under
//! the non-default `audit` feature — a prod dependency on louke compiles none of it; the
//! `tianheng` shell enables it. Why a feature, not a 5th crate: PROJECT.md. The whole module is
//! gated at its declaration in `lib.rs`, so nothing inside needs a per-item
//! `#[cfg(feature = "audit")]`.

use std::collections::HashSet;
use std::path::{Path, PathBuf};
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
/// [`audit_probe_coverage_with_markers`]; see it for what `anchor` is and why the caller supplies
/// it rather than this function deriving one.
pub fn audit_probe_coverage(
    declared: &[RuntimeBoundary],
    source_inputs: &[PathBuf],
    anchor: &Path,
) -> Outcome {
    audit_probe_coverage_with_markers(declared, source_inputs, anchor, DEFAULT_MARKERS)
}

/// **CI face with custom probe markers.** Audit probe coverage against the **declared
/// `RuntimeBoundary` objects** (the authoritative seam set — the constitution, not a source scan
/// for declarations) by scanning the workspace's source inputs for probe macro invocations
/// matching any identifier in `markers` (defaulting to `["assert_boundary"]` via
/// [`audit_probe_coverage`]).
///
/// A file input is treated as an exact Cargo target root and walked through reachable modules;
/// a directory input retains the legacy recursive corpus for source compatibility. See
/// `runtime-origin-assertion`'s "CI face — every declared seam is probed" requirement for the
/// full three-direction reaction (declared-but-unprobed, probed-but-undeclared, un-auditable
/// probe) and its lexical-not-semantic `cfg` bound; its "Root-aware audit excludes unreachable
/// source files" requirement for this file-input mode's module-graph walk and `#[path]`/
/// `cfg_attr`-wrapped-`#[path]` union-scan rules; and its "An un-auditable probe's identity
/// distinguishes distinct offending expressions" requirement for why the `file` field is labeled
/// relative to `anchor`, and its "A file reached through an absolute path literal keeps the path the
/// literal wrote" requirement for the one construct that is deliberately not relativized.
///
/// `anchor` is the directory every observed file's `file` identity label is made relative to — the
/// caller's own checkout/workspace root (`xingbiao::workspace_root`, for the `tianheng` shell). It
/// is a **parameter rather than something derived here**, and that is load-bearing: a raw absolute
/// label makes a baseline recorded in one checkout match nothing in another, while an anchor
/// computed from `source_inputs` themselves — their longest common prefix — trades that for a label
/// that shifts whenever the input set does. Adding one workspace member outside the current shared
/// prefix would relabel every other member's findings (`a/src/lib.rs` becoming
/// `crates/a/src/lib.rs`), so every recorded entry goes stale and re-fires as new at once: the same
/// loss, from a different cause. Only a caller knows a directory that stays put across both, so
/// only a caller can supply it. `anchor` MUST be absolute, and a relative or empty one is a
/// constitution error (exit 2) rather than a silently degraded label: `strip_prefix` cannot remove a
/// relative prefix from an absolute source path (and succeeds trivially against `""`), so either
/// would leave every label in its raw absolute form — checkout-dependent identity again, from an
/// argument that looked accepted.
///
/// Being absolute is what this function can check; being an actual ANCESTOR of the observed files is
/// the caller's own responsibility, and is why the parameter exists. A file that does not lie under
/// `anchor` keeps its path as observed — the documented fallback the absolute-`#[path]` bound below
/// depends on — so an absolute anchor that is simply unrelated to the roots degrades per file rather
/// than erroring. `xingbiao::workspace_root` is an ancestor of every member root by construction.
///
/// Declarations come from the passed objects, so an unconventionally spelled `RuntimeBoundary::at`
/// can no longer hide a seam. It does NOT observe the live install registry —
/// install-vs-constitution consistency is the prod face's runtime fail-closed concern; this
/// verifies coverage against the declared seams and the source.
///
/// Compiled only with the non-default `audit` feature (the CI face); see the module note above.
pub fn audit_probe_coverage_with_markers(
    declared: &[RuntimeBoundary],
    source_inputs: &[PathBuf],
    anchor: &Path,
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
    // Every un-auditable probe's `file` identity field is labeled relative to the caller's
    // `anchor` rather than as a raw absolute path, so the identity — and any baseline recorded
    // against it — stays both checkout-independent and stable across a change to the observed
    // member set. See `scan::labeled` and this function's own doc for why the anchor is given.
    //
    // A non-absolute anchor cannot do that job and so is refused rather than accepted: stripping
    // it from an absolute source path fails, the label silently keeps its absolute form, and the
    // identity is checkout-dependent again — the exact defect the anchor exists to close, reached
    // by an argument that looked accepted. `Path::strip_prefix` succeeds against `""` too, which is
    // why an empty anchor is refused by the same rule rather than blessed as a "no anchor" opt-out:
    // it produces the same silently checkout-dependent identity, and this crate does not offer an
    // argument whose effect is to reintroduce the bug. A caller with no stable directory to name
    // has no correct value to pass here, so it hears that (exit 2) instead of a plausible label.
    if !anchor.is_absolute() {
        return Outcome::ConstitutionError(format!(
            "probe-label anchor '{}' is not an absolute path. Every observed file's identity is \
             labeled relative to it, and a relative anchor can never prefix an absolute source \
             path, so every label would silently stay checkout-dependent. Pass the checkout root — \
             `xingbiao::workspace_root(&metadata)` for a Cargo workspace",
            anchor.display()
        ));
    }
    let mut probes = Vec::new();
    for input in source_inputs {
        if let Err(message) = collect_probes_with_markers(input, anchor, markers, &mut probes) {
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
    violations.extend(duplicate_seam_violations(declared));
    violations.extend(unprobed_seam_violations(declared, &probed_set));
    violations.extend(undeclared_probe_violations(&probes, &declared_set));
    violations.extend(unauditable_probe_violations(&probes));
    if violations.is_empty() {
        Outcome::Clean
    } else {
        Outcome::Violations(Report::new(violations))
    }
}

/// Push `violation()` into `violations` the first time `key` is inserted into `seen` — the
/// react-once-per-key guard shared by [`unprobed_seam_violations`] and
/// [`undeclared_probe_violations`]. [`duplicate_seam_violations`] is deliberately NOT built on
/// this: duplicate-seam detection needs TWO sets with different roles (a first-seen tracker
/// gating the reaction, a reported-once tracker deduping it) — a genuinely different shape this
/// single-set guard would flatten incorrectly if forced onto it.
fn react_once<T: Eq + std::hash::Hash>(
    seen: &mut HashSet<T>,
    key: T,
    violations: &mut Vec<Violation>,
    violation: impl FnOnce() -> Violation,
) {
    if seen.insert(key) {
        violations.push(violation());
    }
}

/// Duplicate declared seam: the prod `install` fails loud on it (a duplicate would silently
/// shadow the earlier boundary); catch it at CI too — one enforce violation per duplicated
/// seam — so the misconfiguration surfaces before it reaches a running binary.
fn duplicate_seam_violations(declared: &[RuntimeBoundary]) -> Vec<Violation> {
    let mut violations = Vec::new();
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
    violations
}

/// Declared but never probed: the boundary is never enforced at runtime. Reacts at the
/// declaring boundary's severity (a warn boundary is advisory, not a CI failure).
fn unprobed_seam_violations(
    declared: &[RuntimeBoundary],
    probed_set: &HashSet<&str>,
) -> Vec<Violation> {
    let mut violations = Vec::new();
    let mut seen = HashSet::new();
    for boundary in declared {
        let seam = boundary.seam();
        if !probed_set.contains(seam) {
            react_once(&mut seen, seam, &mut violations, || {
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
                .with_anchor(boundary.anchor().map(String::from))
            });
        }
    }
    violations
}

/// Probed but never declared: the probe references an undeclared seam, which panics at
/// runtime — catch the typo at CI instead of crashing production.
fn undeclared_probe_violations(probes: &[Probe], declared_set: &HashSet<&str>) -> Vec<Violation> {
    let mut violations = Vec::new();
    let mut seen_probe = HashSet::new();
    for probe in probes {
        if let Probe::Literal(seam) = probe {
            if !declared_set.contains(seam.as_str()) {
                react_once(&mut seen_probe, seam.as_str(), &mut violations, || {
                    audit_violation(
                        seam,
                        "every probe must reference a declared seam",
                        AuditRule::ProbeDeclaredSeam.key(),
                        RuntimeFact::UndeclaredProbe { seam: seam.clone() },
                        format!(
                            "an undeclared seam panics at runtime — {UNDECLARED_SEAM_REPAIR_HINT}"
                        ),
                        Severity::Enforce,
                    )
                });
            }
        }
    }
    violations
}

/// Un-auditable probes: a non-literal seam argument cannot be traced to a declared seam.
/// React rather than silently skip (a silent skip is a false negative). One reaction per
/// (marker, file, owner-qualified enclosing item, expression text) — deduped, sorted — so two
/// textually distinct non-literal probes in the same file are distinct findings and
/// baselining one cannot mask another; two byte-identical occurrences in the same file and
/// the same marker and enclosing item still collapse to one (a stated bound: at that
/// granularity no further source content distinguishes them, mirroring `module-boundary`'s
/// "same import on multiple lines is one violation").
fn unauditable_probe_violations(probes: &[Probe]) -> Vec<Violation> {
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
    unauditable
        .into_iter()
        .map(|(marker, file, owner, expr)| {
            // The offending source file, owner, and expression are in hand here (the probe scan
            // captured them). Project the file into the `file` field as well as the finding text:
            // it is a genuine observation, so reporting `null` would be a dishonest null. This is
            // the one runtime violation with a source location — the seam-level ones above name a
            // seam, not a file.
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
            .with_file(Some(file.to_string()))
        })
        .collect()
}

#[cfg(test)]
mod tests;
