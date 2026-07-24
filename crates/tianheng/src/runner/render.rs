//! Rendering the reaction to text, coverage, and SARIF — the pure string producers the runner
//! prints, plus the thin stderr wrappers around them. Split from `runner.rs` (which keeps
//! arg-parsing, the baseline gate, and dimension composition) so the "how a reaction is
//! presented" surface is self-contained, beside the `list`-projection sibling
//! `runner/projection.rs`.

use guibiao::{Coverage, Outcome, Report, Severity};

use super::term_color::Style;

/// The human-readable `check` report goes to **stderr** as a single stream — clean
/// line, violation/advisory blocks, the baseline summary, coverage, and stale entries
/// alike — so a CI log shows them in a deterministic order rather than interleaving a
/// stderr report with a stdout coverage line. Stdout is reserved for machine output:
/// the `--format json` document and the `list` projection. (This mirrors how `cargo`
/// and `clippy` keep diagnostics on stderr and leave stdout for consumable data.)
pub(crate) fn report(outcome: &Outcome) {
    match outcome {
        Outcome::Clean => eprintln!("Tianheng: clean — no boundary violated"),
        Outcome::Violations(report) => report_violations(report),
        Outcome::ConstitutionError(message) => {
            // The exit-2 diagnostic voice, distinct from a violation (exit 1). Presentation only.
            let style = Style::detect();
            eprintln!(
                "{}",
                style.error(&format!("Tianheng constitution error: {message}"))
            );
        }
        // `Outcome` is non-exhaustive; the exit code (in guibiao) stays authoritative.
        _ => {}
    }
}

/// Print each non-baselined violation as a failure (enforce) or advisory (warn),
/// and summarize how many were suppressed by a baseline.
pub(crate) fn report_violations(report: &Report) {
    eprint!("{}", violations_text_styled(report, Style::detect()));
}

/// The human-readable violation report `check` prints, as a pure function so the foregrounding,
/// file-surfacing, and grouping invariants are unit-testable (the reaction itself,
/// [`report_violations`], just prints this to stderr).
///
/// 潛移: each block **leads with the reason** — the principle and the repair direction — then the
/// mechanical fields (target, rule, finding), then where to repair (the offending file, when
/// observed), then the verdict. Violations are **grouped by boundary** via a stable sort of a
/// local view by `(target, rule)`, so multiple findings under one boundary read consecutively and
/// the reason is read once. This borrows `&Report` immutably and never reorders the underlying vec,
/// so the JSON projection (the machine contract) is untouched.
///
/// The production path always styles by detection ([`report_violations`]); this un-styled form is
/// the byte-stable contract the unit tests assert against, so it is compiled only under `test`.
#[cfg(test)]
pub(crate) fn violations_text(report: &Report) -> String {
    violations_text_styled(report, Style::PLAIN)
}

/// The styled implementation. With [`Style::PLAIN`] the output is byte-identical to the un-styled
/// report (so `violations_text` and its unit tests stay stable); with [`Style::ACTIVE`] the
/// header carries a severity colour and the reason is emphasised — colour is layered *around* the
/// existing text, never reordering or removing a field, so the machine JSON projection and the
/// reason → boundary → rule → found → file → reaction order are untouched.
pub(crate) fn violations_text_styled(report: &Report, style: Style) -> String {
    use std::fmt::Write as _;
    if report.violations.is_empty() {
        return "Tianheng: clean — no boundary violated\n".to_string();
    }
    let baselined = report.violations.iter().filter(|v| v.baselined).count();
    let mut shown: Vec<_> = report.violations.iter().filter(|v| !v.baselined).collect();
    shown.sort_by(|a, b| (a.target(), a.rule.as_str()).cmp(&(b.target(), b.rule.as_str())));

    let mut out = String::new();
    for violation in shown {
        let (raw_header, reaction) = match violation.severity {
            Severity::Enforce => ("Tianheng violation", "CI failed."),
            Severity::Warn => ("Tianheng advisory", "warning only — CI not failed."),
            // `Severity` is non-exhaustive; an unknown future rung reports as advisory.
            _ => ("Tianheng advisory", "warning only — CI not failed."),
        };
        let header = if violation.severity == Severity::Enforce {
            style.enforce(raw_header)
        } else {
            style.warn(raw_header)
        };
        writeln!(out).unwrap();
        writeln!(out, "{header}").unwrap();
        writeln!(out).unwrap();
        writeln!(out, "Reason:\n  {}", style.reason(&violation.reason)).unwrap();
        writeln!(out, "Boundary:\n  {}", violation.target()).unwrap();
        writeln!(out, "Rule:\n  {}", violation.rule).unwrap();
        writeln!(out, "Found:\n  {}", violation.finding).unwrap();
        if let Some(file) = &violation.file {
            writeln!(out, "File:\n  {file}").unwrap();
        }
        if let Some(anchor) = &violation.anchor {
            // The durable governance pointer, surfaced after the located facts and kept out of the
            // reason-led opening; only present when the boundary declared one, so an anchor-less
            // report stays byte-unchanged.
            writeln!(out, "Anchor:\n  {anchor}").unwrap();
        }
        if let Some(polarity) = violation.polarity {
            // The repair-direction polarity, only for a boundary-drift violation (an audit-coverage
            // violation carries none, so its block shows no polarity line rather than "none").
            writeln!(out, "Repair:\n  {}", polarity.as_str()).unwrap();
        }
        writeln!(out, "Reaction:\n  {reaction}").unwrap();
    }
    if baselined > 0 {
        writeln!(
            out,
            "Tianheng: {baselined} pre-existing violation(s) suppressed by baseline"
        )
        .unwrap();
    }
    out
}

/// Print the workspace coverage summary, and — under `--warn-uncovered` — each
/// uncovered crate as a warn-severity advisory. Coverage is an observation: it is
/// reported but never changes the exit code.
pub(crate) fn report_coverage(coverage: &Coverage, warn_uncovered: bool) {
    eprint!("{}", coverage_report(coverage, warn_uncovered));
}

/// The pure text of the coverage summary — and, under `--warn-uncovered`, a per-crate advisory
/// block. Split off [`report_coverage`] (which only prints it to stderr) so the message is
/// assertable without capturing a subprocess. Every advisory block states its reaction is a
/// warning that never fails CI: coverage is an observation, not a reaction.
pub(crate) fn coverage_report(coverage: &Coverage, warn_uncovered: bool) -> String {
    let uncovered = coverage.uncovered.len();
    if uncovered == 0 {
        return format!(
            "Tianheng: coverage — all {} workspace crate(s) have a boundary\n",
            coverage.total
        );
    }
    let mut out = format!(
        "Tianheng: coverage — {uncovered} of {} workspace crate(s) have no boundary\n",
        coverage.total
    );
    if warn_uncovered {
        for crate_name in &coverage.uncovered {
            out.push_str("\nTianheng advisory\n\n");
            out.push_str(&format!("Uncovered crate:\n  {crate_name}\n"));
            out.push_str("Reason:\n  no boundary governs this workspace crate\n");
            out.push_str("Reaction:\n  warning only — CI not failed.\n");
        }
    }
    out
}

/// Project the reaction as a **SARIF 2.1.0** document (`--format sarif`) — the CI-consumable
/// surface GitHub code-scanning ingests. One `results[]` entry per non-baselined violation
/// (`ruleId` = rule, `level` = error/warning, message = reason + finding; the rule lives in
/// `ruleId`, not the message). A violation's `file` becomes `artifactLocation.uri` with **no
/// `region`** (the line is not observed — never fabricated); a file-less violation gets no
/// `locations`. A constitution error is a tool-execution notification under an invocation whose
/// `executionSuccessful` is `false` (required on any SARIF invocation). Clean → empty `results`.
/// Presentation only: the outcome and exit code are unchanged.
pub(crate) fn report_sarif(outcome: &Outcome) -> String {
    use serde_json::{Value, json};
    let mut results: Vec<Value> = Vec::new();
    let mut invocations: Vec<Value> = Vec::new();
    match outcome {
        Outcome::Violations(report) => {
            for v in report.violations.iter().filter(|v| !v.baselined) {
                let level = match v.severity {
                    Severity::Enforce => "error",
                    _ => "warning",
                };
                let mut result = json!({
                    "ruleId": v.rule,
                    "level": level,
                    "message": { "text": format!("{} (found: {})", v.reason, v.finding) },
                });
                let canonical_identity = serde_json::to_string(&v.id().to_json())
                    .expect("canonical violation identity is JSON-serializable");
                result["partialFingerprints"] = json!({
                    "tianheng/structured-fact-identity": canonical_identity,
                });
                if let Some(file) = &v.file {
                    // File-level only: artifactLocation.uri, no `region` (line is not observed).
                    result["locations"] = json!([{
                        "physicalLocation": { "artifactLocation": { "uri": file } }
                    }]);
                }
                // The result property bag carries whatever metadata applies — the durable `anchor`
                // and/or the repair-direction `polarity` (both SARIF-valid, ingester-agnostic). One
                // shared bag: the two are merged, never overwritten. Emitted only when at least one
                // applies, so a violation with neither keeps byte-unchanged SARIF.
                let mut properties = serde_json::Map::new();
                if let Some(anchor) = &v.anchor {
                    properties.insert("anchor".to_string(), json!(anchor));
                }
                if let Some(polarity) = v.polarity {
                    properties.insert("polarity".to_string(), json!(polarity.as_str()));
                }
                if !properties.is_empty() {
                    result["properties"] = Value::Object(properties);
                }
                results.push(result);
            }
        }
        Outcome::ConstitutionError(message) => {
            invocations.push(json!({
                "executionSuccessful": false,
                "toolExecutionNotifications": [{
                    "level": "error",
                    "message": { "text": message },
                }],
            }));
        }
        // Clean (and any future outcome) contributes no results.
        _ => {}
    }
    let mut run = json!({
        "tool": { "driver": { "name": "tianheng" } },
        "results": results,
    });
    if !invocations.is_empty() {
        run["invocations"] = Value::Array(invocations);
    }
    let doc = json!({
        "$schema": "https://json.schemastore.org/sarif-2.1.0.json",
        "version": "2.1.0",
        "runs": [run],
    });
    serde_json::to_string_pretty(&doc).expect("a serde_json::Value is always serializable")
}

// A GitHub-specific `::error::` workflow-command format is deliberately NOT a built-in: it would
// couple the tool to one CI vendor's proprietary protocol (and invite an open-ended gitlab/azure/…
// set). SARIF — an open, vendor-neutral standard that GitHub and others ingest — is 垂象's CI
// projection; turning it (or the JSON report) into vendor-specific annotations is a harness/CI-step
// convention, not a tool feature (see the README recipe). This keeps the machine surfaces neutral.
