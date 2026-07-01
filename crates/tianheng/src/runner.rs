//! The runner — the CI reaction, as a reusable library entry point.
//!
//! [`run`] turns a caller-supplied [`Constitution`] and the process arguments into
//! a process exit code, providing the whole `tianheng check` contract: flag parsing
//! (`--manifest-path`, `--baseline` / `--write-baseline`, `--format`), the baseline
//! gate and write actions, the human and JSON reports, and the exit-code mapping
//! (`0` clean / warn-only / fully baselined, `1` enforce violation, `2`
//! constitution / scan / usage error). An adopting project declares its own
//! constitution in Rust and gets this contract from one line:
//!
//! ```no_run
//! use tianheng::prelude::*;
//! fn constitution() -> Constitution { Constitution::new("my-project") }
//! fn main() -> std::process::ExitCode {
//!     tianheng::run(&constitution(), std::env::args())
//! }
//! ```
//!
//! IO (filesystem, stdout/stderr) is quarantined here; the `guibiao` crate stays the
//! pure functional core (the model plus [`check`](crate::check)), and must not depend on
//! this shell — a crate-level invariant (see `tests/self_governance.rs`). The numeric
//! work lives in the private [`dispatch`], so the exit code is unit-testable; [`run`] is
//! a thin [`ExitCode`] wrapper.

use std::path::PathBuf;
use std::process::ExitCode;

use guibiao::{
    Baseline, Coverage, Outcome, Report, Severity, ViolationId, apply_baseline, check_and_cover,
    constitution_json, constitution_text, report_json, workspace_member_src_dirs,
};
use hunyi::{
    DynTraitBoundary, ForbiddenMarkerBoundary, ImplTraitBoundary, SemanticBoundary,
    TraitImplBoundary, VisibilityBoundary,
};
use louke::{RuntimeBoundary, audit_probe_coverage};
use serde_json::Value;

use crate::Constitution;

/// Which runner command was requested. `check` reacts against a workspace; `list`
/// projects the declared constitution and never reacts.
#[derive(PartialEq, Eq)]
enum Command {
    Check,
    List,
}

/// The requested output format. `text` (default) and `json` apply to both commands;
/// `markdown` is a `list`-only projection of the declared law — `check`'s machine-readable
/// output is the JSON report, never a law summary, so `check --format markdown` is a usage
/// error (exit 2).
#[derive(PartialEq, Eq, Clone, Copy)]
enum Format {
    Text,
    Json,
    Markdown,
    Sarif,
}

/// The `check` output format — the `Format` values `check` accepts, with `markdown` (a `list`-only
/// law projection) excluded by construction. `sarif` is the CI-consumable projection of the
/// reaction (an open, vendor-neutral standard); like `json` it changes presentation only, never
/// the outcome or exit code.
#[derive(PartialEq, Eq, Clone, Copy)]
enum ReportFormat {
    Text,
    Json,
    Sarif,
}

/// Run the unified constitution's boundaries against a Cargo workspace and return the
/// process exit code. The one [`Constitution`] carries every dimension — static (圭表),
/// semantic (渾儀), and the runtime (漏刻) CI probe-coverage audit — which this gate composes
/// into one reaction. A dimension with no declared boundaries contributes nothing.
/// `args` are the full process arguments (the program name is skipped internally, like a
/// real `main`). Pass `std::env::args()` from a binary.
pub fn run<I, S>(constitution: &Constitution, args: I) -> ExitCode
where
    I: IntoIterator<Item = S>,
    S: Into<String>,
{
    ExitCode::from(dispatch(constitution, args))
}

/// The runner's work, returning the exit code as a number so it is assertable
/// without a subprocess and without inspecting an opaque [`ExitCode`].
fn dispatch<I, S>(constitution: &Constitution, args: I) -> u8
where
    I: IntoIterator<Item = S>,
    S: Into<String>,
{
    let mut manifest_path: Option<String> = None;
    let mut baseline_path: Option<String> = None;
    let mut write_baseline_path: Option<String> = None;
    let mut format: Option<String> = None;
    let mut warn_uncovered = false;
    let mut args = args.into_iter().map(Into::into).skip(1).peekable();

    // The command is the first positional token; an absent or unrecognized leading
    // token stays `check` (backward compatible). Flags following it never select
    // the command.
    let command = match args.peek().map(String::as_str) {
        Some("list") => {
            args.next();
            Command::List
        }
        Some("check") => {
            args.next();
            Command::Check
        }
        _ => Command::Check,
    };

    // A value-taking flag must be given its value; an absent value is a usage error
    // (exit 2), never a silent downgrade to the default or to a plain check
    // (PROJECT.md: misconfiguration fails loud).
    macro_rules! value {
        ($flag:literal) => {
            match args.next() {
                Some(value) => value,
                None => return usage(concat!($flag, " requires a value")),
            }
        };
    }
    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--manifest-path" => manifest_path = Some(value!("--manifest-path")),
            "--baseline" => baseline_path = Some(value!("--baseline")),
            "--write-baseline" => write_baseline_path = Some(value!("--write-baseline")),
            "--format" => format = Some(value!("--format")),
            "--warn-uncovered" => warn_uncovered = true,
            other => {
                if let Some(path) = other.strip_prefix("--manifest-path=") {
                    manifest_path = Some(path.to_string());
                } else if let Some(path) = other.strip_prefix("--baseline=") {
                    baseline_path = Some(path.to_string());
                } else if let Some(path) = other.strip_prefix("--write-baseline=") {
                    write_baseline_path = Some(path.to_string());
                } else if let Some(value) = other.strip_prefix("--format=") {
                    format = Some(value.to_string());
                } else {
                    // An unknown flag, a misspelling, or a stray positional is a
                    // misconfiguration — fail loud (exit 2), never silently ignore
                    // it (PROJECT.md).
                    return usage(&format!("unrecognized argument '{other}'"));
                }
            }
        }
    }

    // `--format` is parsed for both commands so the flag contract stays uniform; `markdown`
    // is recognized here but only honored by `list` (rejected for `check` below).
    let format = match format.as_deref() {
        None | Some("text") => Format::Text,
        Some("json") => Format::Json,
        Some("markdown") => Format::Markdown,
        Some("sarif") => Format::Sarif,
        Some(other) => {
            return usage(&format!(
                "unknown --format '{other}' (expected text, json, markdown, or sarif)"
            ));
        }
    };

    // `list` is a projection, not a reaction: it observes nothing (no
    // `--manifest-path`), cannot fail a boundary, and always exits 0. It accepts
    // only `--format`; a check-only flag supplied to `list` is a usage error, not a
    // silent no-op (PROJECT.md: never silently ignore a flag).
    if command == Command::List {
        if manifest_path.is_some()
            || baseline_path.is_some()
            || write_baseline_path.is_some()
            || warn_uncovered
        {
            return usage("list takes only --format; other flags are check-only");
        }
        let semantic = constitution.semantic_boundaries();
        let runtime = constitution.runtime_boundaries();
        match format {
            Format::Json => {
                println!(
                    "{}",
                    serde_json::to_string_pretty(&list_document(constitution))
                        .expect("a serde_json::Value is always serializable")
                );
            }
            Format::Markdown => {
                // Rendered from the same `list_document` value the JSON projection emits, so the
                // Markdown provably carries no less than the JSON and covers exactly the same
                // dimensions — a pure projection, never a reaction.
                print!("{}", list_markdown(&list_document(constitution)));
            }
            Format::Text => {
                println!("{}", constitution_text(constitution.static_boundaries()));
                print!("{}", semantic_text(&semantic.signature));
                print!("{}", trait_impl_text(&semantic.trait_impl));
                print!("{}", visibility_text(&semantic.visibility));
                print!("{}", forbidden_marker_text(&semantic.forbidden_marker));
                print!("{}", dyn_trait_text(&semantic.dyn_trait));
                print!("{}", impl_trait_text(&semantic.impl_trait));
                print!("{}", runtime_text(runtime));
            }
            // SARIF projects the *reaction*, not the declared law, so it is `check`-only —
            // symmetric to `markdown` being `list`-only.
            Format::Sarif => {
                return usage(
                    "list supports --format text|json|markdown; sarif projects the reaction \
                     (a check output), not the declared law",
                );
            }
        }
        return 0;
    }

    // The command is `check`. `markdown` is a `list`-only projection of the declared law;
    // `check`'s machine output is the JSON report, so reject it loud (exit 2) rather than
    // silently falling back. `text`/`json` map to the existing boolean contract.
    let report_format = match format {
        Format::Text => ReportFormat::Text,
        Format::Json => ReportFormat::Json,
        Format::Sarif => ReportFormat::Sarif,
        Format::Markdown => {
            return usage(
                "check supports --format text|json|sarif; markdown is a list-only \
                 projection of the declared law",
            );
        }
    };

    // From here on the command is `check`: it requires a workspace to observe.
    // An absent `--manifest-path` defaults to the nearest `Cargo.toml`, cargo-style.
    // Defaulting the target location is not a silent pass: if none is found the run
    // exits 2 (a scan error), never 0.
    let manifest_path = match manifest_path {
        Some(path) => PathBuf::from(path),
        None => match nearest_manifest() {
            Some(path) => path,
            None => {
                let from = std::env::current_dir()
                    .map(|dir| dir.display().to_string())
                    .unwrap_or_else(|_| "the current directory".to_string());
                eprintln!(
                    "Tianheng: no Cargo.toml found from {from} up to the root; \
                     pass --manifest-path <path>"
                );
                return 2;
            }
        },
    };
    if baseline_path.is_some() && write_baseline_path.is_some() {
        return usage("--baseline and --write-baseline are mutually exclusive");
    }

    // One `cargo metadata` read feeds both the static reaction outcome and coverage; the
    // semantic dimension reads its own (it has no coverage notion). The two outcomes compose
    // into one: a constitution error from either supersedes (the run's verdict is
    // untrustworthy), and otherwise the violations merge into a single report. Coverage
    // stays static-only.
    let (static_outcome, observed_coverage) =
        check_and_cover(constitution.static_boundaries(), &manifest_path);
    // Compose the dimensions in order. A constitution error from any dimension supersedes
    // (the run's verdict is untrustworthy), so once one errors we stop scanning the rest;
    // otherwise each dimension's violations merge into one report.
    let mut outcome = static_outcome;
    if !matches!(outcome, Outcome::ConstitutionError(_))
        && !constitution.semantic_boundaries().is_empty()
    {
        // The whole 渾儀 dimension composes via one entry (one `cargo metadata` read);
        // a constitution error from any semantic boundary supersedes.
        outcome = merge_outcomes(
            outcome,
            hunyi::check_all(constitution.semantic_boundaries(), &manifest_path),
        );
    }
    // 漏刻 (runtime) CI face: probe-coverage of the declared runtime seams, scanned across the
    // workspace's member source roots (resolved here so `louke` stays std-only). Guarded like the
    // semantic block — once a dimension errors, the verdict is untrustworthy, so we stop. The
    // src-dir resolution can itself fail (an unreadable workspace) → fold it as a constitution
    // error (`dispatch` returns `u8`, so we cannot use `?`).
    if !matches!(outcome, Outcome::ConstitutionError(_))
        && !constitution.runtime_boundaries().is_empty()
    {
        match workspace_member_src_dirs(&manifest_path) {
            Ok(src_dirs) => {
                outcome = merge_outcomes(
                    outcome,
                    audit_probe_coverage(constitution.runtime_boundaries(), &src_dirs),
                );
            }
            Err(message) => {
                outcome = merge_outcomes(outcome, Outcome::ConstitutionError(message));
            }
        }
    }

    if let Some(path) = write_baseline_path {
        return write_baseline(&outcome, &path);
    }

    // Coverage is an observation, not a reaction: surfaced only when the constitution
    // was successfully evaluated, omitted on a constitution error (where the error is
    // the story), and never affecting the exit code.
    let coverage = match outcome {
        Outcome::ConstitutionError(_) => None,
        _ => observed_coverage,
    };

    if let Some(path) = baseline_path {
        return gate(
            &mut outcome,
            &path,
            report_format,
            coverage.as_ref(),
            warn_uncovered,
        );
    }

    match report_format {
        ReportFormat::Json => println!("{}", report_json(&outcome, &[], coverage.as_ref())),
        ReportFormat::Sarif => println!("{}", report_sarif(&outcome)),
        ReportFormat::Text => {
            report(&outcome);
            if let Some(coverage) = &coverage {
                report_coverage(coverage, warn_uncovered);
            }
        }
    }
    outcome.exit_code()
}

/// Print usage to stderr and return exit 2 — a usage mistake is not architectural
/// drift.
fn usage(message: &str) -> u8 {
    eprintln!(
        "usage:\n  \
         tianheng check --manifest-path <path/to/Cargo.toml> \
         [--baseline <file> | --write-baseline <file>] [--format text|json|sarif] \
         [--warn-uncovered]\n  \
         tianheng list [--format text|json|markdown]"
    );
    eprintln!("error: {message}");
    2
}

/// Walk up from the current directory to the nearest `Cargo.toml`, cargo-style, so
/// `check` can default its target like `cargo` does when `--manifest-path` is omitted.
fn nearest_manifest() -> Option<PathBuf> {
    let mut dir = std::env::current_dir().ok()?;
    loop {
        let candidate = dir.join("Cargo.toml");
        if candidate.is_file() {
            return Some(candidate);
        }
        if !dir.pop() {
            return None;
        }
    }
}

/// Record the current violations as a baseline. Recording is not judging, so this
/// returns 0; but a constitution that could not be evaluated cannot be pinned.
fn write_baseline(outcome: &Outcome, path: &str) -> u8 {
    if let Outcome::ConstitutionError(message) = outcome {
        eprintln!("Tianheng constitution error: {message}");
        eprintln!("refusing to write a baseline from a constitution that could not be evaluated");
        return 2;
    }
    let empty = Report::empty();
    let report = match outcome {
        Outcome::Violations(report) => report,
        _ => &empty,
    };
    let baseline = Baseline::of(report);
    match std::fs::write(path, baseline.to_json()) {
        Ok(()) => {
            eprintln!(
                "Tianheng: wrote {} violation(s) to baseline {path}",
                report.violations.len()
            );
            0
        }
        Err(err) => {
            eprintln!("Tianheng: cannot write baseline {path}: {err}");
            2
        }
    }
}

/// Gate against a baseline: suppress recorded violations, fail only on new ones,
/// and report stale baseline entries. An unreadable baseline is a scan error.
fn gate(
    outcome: &mut Outcome,
    path: &str,
    format: ReportFormat,
    coverage: Option<&Coverage>,
    warn_uncovered: bool,
) -> u8 {
    // A constitution error is the whole story: report it before reading the baseline, so
    // it is never masked by a missing or unreadable baseline file (both exit 2, but the
    // constitution error is the actionable one).
    if let Outcome::ConstitutionError(message) = outcome {
        match format {
            ReportFormat::Json => println!("{}", report_json(outcome, &[], None)),
            ReportFormat::Sarif => println!("{}", report_sarif(outcome)),
            ReportFormat::Text => eprintln!("Tianheng constitution error: {message}"),
        }
        return 2;
    }

    let baseline = match std::fs::read_to_string(path) {
        Ok(text) => match Baseline::from_json(&text) {
            Ok(baseline) => baseline,
            Err(err) => {
                eprintln!("Tianheng: invalid baseline {path}: {err}");
                return 2;
            }
        },
        Err(err) => {
            eprintln!("Tianheng: cannot read baseline {path}: {err}");
            return 2;
        }
    };

    if let Outcome::Violations(report) = outcome {
        apply_baseline(report, &baseline);
    }

    let empty = Report::empty();
    let report = match &*outcome {
        Outcome::Violations(report) => report,
        _ => &empty,
    };
    let stale: Vec<ViolationId> = baseline.stale(report).into_iter().cloned().collect();
    match format {
        ReportFormat::Json => println!("{}", report_json(outcome, &stale, coverage)),
        ReportFormat::Sarif => println!("{}", report_sarif(outcome)),
        ReportFormat::Text => {
            report_violations(report);
            for entry in &stale {
                eprintln!(
                    "Tianheng: stale baseline entry (no longer violated): {} / {} / {}",
                    entry.target, entry.rule, entry.finding
                );
            }
            if let Some(coverage) = coverage {
                report_coverage(coverage, warn_uncovered);
            }
        }
    }
    outcome.exit_code()
}

/// The human-readable `check` report goes to **stderr** as a single stream — clean
/// line, violation/advisory blocks, the baseline summary, coverage, and stale entries
/// alike — so a CI log shows them in a deterministic order rather than interleaving a
/// stderr report with a stdout coverage line. Stdout is reserved for machine output:
/// the `--format json` document and the `list` projection. (This mirrors how `cargo`
/// and `clippy` keep diagnostics on stderr and leave stdout for consumable data.)
fn report(outcome: &Outcome) {
    match outcome {
        Outcome::Clean => eprintln!("Tianheng: clean — no boundary violated"),
        Outcome::Violations(report) => report_violations(report),
        Outcome::ConstitutionError(message) => {
            eprintln!("Tianheng constitution error: {message}");
        }
        // `Outcome` is non-exhaustive; the exit code (in guibiao) stays authoritative.
        _ => {}
    }
}

/// Print each non-baselined violation as a failure (enforce) or advisory (warn),
/// and summarize how many were suppressed by a baseline.
fn report_violations(report: &Report) {
    eprint!("{}", violations_text(report));
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
fn violations_text(report: &Report) -> String {
    use std::fmt::Write as _;
    if report.violations.is_empty() {
        return "Tianheng: clean — no boundary violated\n".to_string();
    }
    let baselined = report.violations.iter().filter(|v| v.baselined).count();
    let mut shown: Vec<_> = report.violations.iter().filter(|v| !v.baselined).collect();
    shown.sort_by(|a, b| {
        (a.target.as_str(), a.rule.as_str()).cmp(&(b.target.as_str(), b.rule.as_str()))
    });

    let mut out = String::new();
    for violation in shown {
        let (header, reaction) = match violation.severity {
            Severity::Enforce => ("Tianheng violation", "CI failed."),
            Severity::Warn => ("Tianheng advisory", "warning only — CI not failed."),
            // `Severity` is non-exhaustive; an unknown future rung reports as advisory.
            _ => ("Tianheng advisory", "warning only — CI not failed."),
        };
        writeln!(out).unwrap();
        writeln!(out, "{header}").unwrap();
        writeln!(out).unwrap();
        writeln!(out, "Reason:\n  {}", violation.reason).unwrap();
        writeln!(out, "Boundary:\n  {}", violation.target).unwrap();
        writeln!(out, "Rule:\n  {}", violation.rule).unwrap();
        writeln!(out, "Found:\n  {}", violation.finding).unwrap();
        if let Some(file) = &violation.file {
            writeln!(out, "File:\n  {file}").unwrap();
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
fn report_coverage(coverage: &Coverage, warn_uncovered: bool) {
    let uncovered = coverage.uncovered.len();
    if uncovered == 0 {
        eprintln!(
            "Tianheng: coverage — all {} workspace crate(s) have a boundary",
            coverage.total
        );
        return;
    }
    eprintln!(
        "Tianheng: coverage — {uncovered} of {} workspace crate(s) have no boundary",
        coverage.total
    );
    if warn_uncovered {
        for crate_name in &coverage.uncovered {
            eprintln!();
            eprintln!("Tianheng advisory");
            eprintln!();
            eprintln!("Uncovered crate:\n  {crate_name}");
            eprintln!("Reason:\n  no boundary governs this workspace crate");
            eprintln!("Reaction:\n  warning only — CI not failed.");
        }
    }
}

/// Project the reaction as a **SARIF 2.1.0** document (`--format sarif`) — the CI-consumable
/// surface GitHub code-scanning ingests. One `results[]` entry per non-baselined violation
/// (`ruleId` = rule, `level` = error/warning, message = reason + finding; the rule lives in
/// `ruleId`, not the message). A violation's `file` becomes `artifactLocation.uri` with **no
/// `region`** (the line is not observed — never fabricated); a file-less violation gets no
/// `locations`. A constitution error is a tool-execution notification under an invocation whose
/// `executionSuccessful` is `false` (required on any SARIF invocation). Clean → empty `results`.
/// Presentation only: the outcome and exit code are unchanged.
fn report_sarif(outcome: &Outcome) -> String {
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
                if let Some(file) = &v.file {
                    // File-level only: artifactLocation.uri, no `region` (line is not observed).
                    result["locations"] = json!([{
                        "physicalLocation": { "artifactLocation": { "uri": file } }
                    }]);
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

/// Compose the static and semantic outcomes into one reaction. A constitution error from
/// either dimension supersedes any violation — a boundary that could not be evaluated makes
/// the run's verdict untrustworthy — and otherwise the two reports' violations merge into a
/// single report, gated, baselined, and reported together. The static outcome is checked
/// first, so a static error wins deterministically when both error.
fn merge_outcomes(static_outcome: Outcome, semantic_outcome: Outcome) -> Outcome {
    if matches!(static_outcome, Outcome::ConstitutionError(_)) {
        return static_outcome;
    }
    if matches!(semantic_outcome, Outcome::ConstitutionError(_)) {
        return semantic_outcome;
    }
    let mut violations = Vec::new();
    if let Outcome::Violations(report) = &static_outcome {
        violations.extend(report.violations.iter().cloned());
    }
    if let Outcome::Violations(report) = &semantic_outcome {
        violations.extend(report.violations.iter().cloned());
    }
    if violations.is_empty() {
        Outcome::Clean
    } else {
        Outcome::Violations(Report::new(violations))
    }
}

/// The text projection of the semantic boundaries, appended to the `list` output. Empty when
/// there are none, so a static-only project's `list` output is unchanged.
fn semantic_text(boundaries: &[SemanticBoundary]) -> String {
    if boundaries.is_empty() {
        return String::new();
    }
    let noun = if boundaries.len() == 1 {
        "boundary"
    } else {
        "boundaries"
    };
    let mut out = format!("Semantic {noun} ({}):\n", boundaries.len());
    for boundary in boundaries {
        out.push_str(&format!(
            "\n[{}] module {} in {}\n  rule:   must not expose: {}\n  reason: {}\n",
            boundary.severity().as_str(),
            boundary.module(),
            boundary.crate_package(),
            boundary.forbidden().join(", "),
            boundary.reason(),
        ));
    }
    out
}

/// The JSON projection of one semantic boundary, mirroring a static boundary's shape (`kind`,
/// `target`, `crate`, `rule`, `severity`, `reason`) plus the `forbidden` set.
fn semantic_boundary_json(boundary: &SemanticBoundary) -> Value {
    serde_json::json!({
        "kind": "semantic",
        "target": boundary.module(),
        "crate": boundary.crate_package(),
        "rule": "must not expose",
        "severity": boundary.severity().as_str(),
        "forbidden": boundary.forbidden(),
        "reason": boundary.reason(),
    })
}

/// The text projection of the trait-impl-locality boundaries, appended to `list`. Empty when
/// there are none, so a project not using the dimension sees unchanged output.
fn trait_impl_text(boundaries: &[TraitImplBoundary]) -> String {
    if boundaries.is_empty() {
        return String::new();
    }
    let noun = if boundaries.len() == 1 {
        "boundary"
    } else {
        "boundaries"
    };
    let mut out = format!("Trait-impl-locality {noun} ({}):\n", boundaries.len());
    for boundary in boundaries {
        out.push_str(&format!(
            "\n[{}] trait {} in {}\n  rule:   may only be implemented in: {}\n  reason: {}\n",
            boundary.severity().as_str(),
            boundary.trait_(),
            boundary.crate_package(),
            boundary.allowed_locations().join(", "),
            boundary.reason(),
        ));
    }
    out
}

/// The JSON projection of one trait-impl-locality boundary, mirroring the others' shape
/// (`kind`, `target` = the trait, `crate`, `rule`, `severity`, `reason`) plus the
/// `allowed_locations` set.
fn trait_impl_boundary_json(boundary: &TraitImplBoundary) -> Value {
    serde_json::json!({
        "kind": "semantic",
        "target": boundary.trait_(),
        "crate": boundary.crate_package(),
        "rule": "must only be implemented in the declared location(s)",
        "severity": boundary.severity().as_str(),
        "allowed_locations": boundary.allowed_locations(),
        "reason": boundary.reason(),
    })
}

/// The text projection of the visibility boundaries, appended to `list`. Empty when there
/// are none, so a project not using the dimension sees unchanged output.
fn visibility_text(boundaries: &[VisibilityBoundary]) -> String {
    if boundaries.is_empty() {
        return String::new();
    }
    let noun = if boundaries.len() == 1 {
        "boundary"
    } else {
        "boundaries"
    };
    let mut out = format!("Visibility {noun} ({}):\n", boundaries.len());
    for boundary in boundaries {
        out.push_str(&format!(
            "\n[{}] module {} in {}\n  rule:   must not declare pub items\n  reason: {}\n",
            boundary.severity().as_str(),
            boundary.module(),
            boundary.crate_package(),
            boundary.reason(),
        ));
    }
    out
}

/// The JSON projection of one visibility boundary, mirroring the others' shape (`kind`,
/// `target` = the module, `crate`, `rule`, `severity`, `reason`).
fn visibility_boundary_json(boundary: &VisibilityBoundary) -> Value {
    serde_json::json!({
        "kind": "semantic",
        "target": boundary.module(),
        "crate": boundary.crate_package(),
        "rule": "must not declare pub items",
        "severity": boundary.severity().as_str(),
        "reason": boundary.reason(),
    })
}

/// The text projection of the forbidden-marker boundaries, appended to `list`. Empty when
/// there are none, so a project not using the dimension sees unchanged output.
fn forbidden_marker_text(boundaries: &[ForbiddenMarkerBoundary]) -> String {
    if boundaries.is_empty() {
        return String::new();
    }
    let noun = if boundaries.len() == 1 {
        "boundary"
    } else {
        "boundaries"
    };
    let mut out = format!("Forbidden-marker {noun} ({}):\n", boundaries.len());
    for boundary in boundaries {
        out.push_str(&format!(
            "\n[{}] subtree {} in {}\n  rule:   must not acquire: {}\n  reason: {}\n",
            boundary.severity().as_str(),
            boundary.module(),
            boundary.crate_package(),
            boundary.forbidden().join(", "),
            boundary.reason(),
        ));
    }
    out
}

/// The JSON projection of one forbidden-marker boundary (`kind`, `target` = the subtree,
/// `crate`, `rule`, `severity`, `reason`) plus the `forbidden` trait set.
fn forbidden_marker_boundary_json(boundary: &ForbiddenMarkerBoundary) -> Value {
    serde_json::json!({
        "kind": "semantic",
        "target": boundary.module(),
        "crate": boundary.crate_package(),
        "rule": "must not acquire trait",
        "severity": boundary.severity().as_str(),
        "forbidden": boundary.forbidden(),
        "reason": boundary.reason(),
    })
}

/// The text projection of the dyn-trait boundaries, appended to `list`. Empty when there are
/// none, so a project not using the dimension sees unchanged output.
fn dyn_trait_text(boundaries: &[DynTraitBoundary]) -> String {
    if boundaries.is_empty() {
        return String::new();
    }
    let noun = if boundaries.len() == 1 {
        "boundary"
    } else {
        "boundaries"
    };
    let mut out = format!("Dyn-trait {noun} ({}):\n", boundaries.len());
    for boundary in boundaries {
        out.push_str(&format!(
            "\n[{}] module {} in {}\n  rule:   must not expose dyn\n  reason: {}\n",
            boundary.severity().as_str(),
            boundary.module(),
            boundary.crate_package(),
            boundary.reason(),
        ));
    }
    out
}

fn impl_trait_text(boundaries: &[ImplTraitBoundary]) -> String {
    if boundaries.is_empty() {
        return String::new();
    }
    let noun = if boundaries.len() == 1 {
        "boundary"
    } else {
        "boundaries"
    };
    let mut out = format!("Impl-trait {noun} ({}):\n", boundaries.len());
    for boundary in boundaries {
        out.push_str(&format!(
            "\n[{}] module {} in {}\n  rule:   must not expose impl trait\n  reason: {}\n",
            boundary.severity().as_str(),
            boundary.module(),
            boundary.crate_package(),
            boundary.reason(),
        ));
    }
    out
}

/// The JSON projection of one dyn-trait boundary, mirroring a semantic boundary's shape (`kind`,
/// `target`, `crate`, `rule`, `severity`, `reason`). An operand-scoped boundary additionally
/// carries the `forbidden` operand set; a shape-only boundary (empty set) emits no such field.
fn dyn_trait_boundary_json(boundary: &DynTraitBoundary) -> Value {
    let mut object = serde_json::json!({
        "kind": "semantic",
        "target": boundary.module(),
        "crate": boundary.crate_package(),
        "rule": "must not expose dyn",
        "severity": boundary.severity().as_str(),
        "reason": boundary.reason(),
    });
    // The operand set surfaces only for an operand-scoped boundary; a shape-only boundary
    // (empty set) projects unchanged, with no `forbidden` param.
    let operands = boundary.forbidden_operands();
    if !operands.is_empty() {
        object["forbidden"] = serde_json::json!(operands);
    }
    object
}

fn impl_trait_boundary_json(boundary: &ImplTraitBoundary) -> Value {
    let mut object = serde_json::json!({
        "kind": "semantic",
        "target": boundary.module(),
        "crate": boundary.crate_package(),
        "rule": "must not expose impl trait",
        "severity": boundary.severity().as_str(),
        "reason": boundary.reason(),
    });
    // The operand set surfaces only for an operand-scoped boundary; a shape-only boundary
    // (empty set) projects unchanged, with no `forbidden` param.
    let operands = boundary.forbidden_operands();
    if !operands.is_empty() {
        object["forbidden"] = serde_json::json!(operands);
    }
    object
}

/// The `list --format json` document: the static constitution's projection augmented with one
/// array per non-empty dimension, so the document covers every declared law and never silently
/// omits one. A dimension with no boundaries adds no key (a static-only project's document is
/// byte-identical to before the other dimensions existed).
fn list_document(constitution: &Constitution) -> Value {
    let semantic = constitution.semantic_boundaries();
    let runtime = constitution.runtime_boundaries();
    let mut document: Value =
        serde_json::from_str(&constitution_json(constitution.static_boundaries()))
            .expect("constitution_json emits a valid document");
    if !semantic.signature.is_empty() {
        document["semantic_boundaries"] = Value::Array(
            semantic
                .signature
                .iter()
                .map(semantic_boundary_json)
                .collect(),
        );
    }
    if !semantic.trait_impl.is_empty() {
        document["trait_impl_boundaries"] = Value::Array(
            semantic
                .trait_impl
                .iter()
                .map(trait_impl_boundary_json)
                .collect(),
        );
    }
    if !semantic.visibility.is_empty() {
        document["visibility_boundaries"] = Value::Array(
            semantic
                .visibility
                .iter()
                .map(visibility_boundary_json)
                .collect(),
        );
    }
    if !semantic.forbidden_marker.is_empty() {
        document["forbidden_marker_boundaries"] = Value::Array(
            semantic
                .forbidden_marker
                .iter()
                .map(forbidden_marker_boundary_json)
                .collect(),
        );
    }
    if !semantic.dyn_trait.is_empty() {
        document["dyn_trait_boundaries"] = Value::Array(
            semantic
                .dyn_trait
                .iter()
                .map(dyn_trait_boundary_json)
                .collect(),
        );
    }
    if !semantic.impl_trait.is_empty() {
        document["impl_trait_boundaries"] = Value::Array(
            semantic
                .impl_trait
                .iter()
                .map(impl_trait_boundary_json)
                .collect(),
        );
    }
    if !runtime.is_empty() {
        document["runtime_boundaries"] =
            Value::Array(runtime.iter().map(runtime_boundary_json).collect());
    }
    document
}

/// The text projection of the runtime (漏刻) boundaries, appended to `list`. Empty when there
/// are none, so a project not using the dimension sees unchanged output.
fn runtime_text(boundaries: &[RuntimeBoundary]) -> String {
    if boundaries.is_empty() {
        return String::new();
    }
    let noun = if boundaries.len() == 1 {
        "boundary"
    } else {
        "boundaries"
    };
    let mut out = format!("Runtime {noun} ({}):\n", boundaries.len());
    for boundary in boundaries {
        out.push_str(&format!(
            "\n[{}] seam {}\n  rule:    only origins: {}\n  posture: {}\n  reason:  {}\n",
            boundary.severity().as_str(),
            boundary.seam(),
            boundary.allowed_origins().join(", "),
            boundary.posture().as_str(),
            boundary.reason(),
        ));
    }
    out
}

/// The JSON projection of one runtime boundary (`kind` = runtime, `target` = the seam, `rule`,
/// `severity`, `posture`, `reason`) plus the `allowed_origins` set. `posture` is projected so a
/// `panic_on_violation` boundary does not project identically to a default event-only one.
fn runtime_boundary_json(boundary: &RuntimeBoundary) -> Value {
    serde_json::json!({
        "kind": "runtime",
        "target": boundary.seam(),
        "rule": "only declared origins may cross the seam",
        "severity": boundary.severity().as_str(),
        "posture": boundary.posture().as_str(),
        "allowed_origins": boundary.allowed_origins(),
        "reason": boundary.reason(),
    })
}

/// Render a constitution as the human- and agent-readable Markdown summary of its declared law —
/// the same projection `list --format markdown` prints, returned as a `String` for library
/// callers (e.g. to generate an agent-context artifact). It composes the same internal projector,
/// so it carries no less than the JSON and never reacts; it adds nothing of its own (no preamble,
/// no trailing newline), so it equals the CLI output byte for byte.
///
/// **Format stability.** This Markdown layout is intended for display, review, and LLM context.
/// It is **not** a machine-stable contract and **may evolve in any compatible release** to improve
/// readability or imitability (e.g. foregrounding a boundary's `reason`). Consumers that need a
/// stable, machine-parseable projection MUST use the JSON projection (`list --format json`)
/// instead — depending on the exact Markdown shape is unsupported.
///
/// ```
/// use tianheng::prelude::*;
/// let c = Constitution::new("my-project").boundary(
///     CrateBoundary::crate_("my-core")
///         .deny_external_dependencies()
///         .because("my-core stays dependency-light"),
/// );
/// let md = tianheng::constitution_markdown(&c);
/// assert!(md.contains("# Constitution: my-project"));
/// assert!(md.contains("my-core stays dependency-light"));
/// // Write it where an agent will read it, e.g.:
/// // std::fs::write("AGENTS.my-project-law.md", md)?;
/// ```
pub fn constitution_markdown(constitution: &Constitution) -> String {
    list_markdown(&list_document(constitution))
}

/// The `list --format markdown` projection: an agent-readable summary of the *whole* declared
/// law. It is rendered from the very [`Value`] [`list_document`] emits, so it provably carries
/// no information absent from the JSON and covers exactly the same dimensions (the spec's
/// "no less than the JSON" guarantee holds by construction, not by parallel maintenance). Like
/// `list` as a whole it observes nothing and never reacts. A dimension with no declared
/// boundaries contributes no section, mirroring the text and JSON projections.
fn list_markdown(document: &Value) -> String {
    let name = document
        .get("constitution")
        .and_then(Value::as_str)
        .unwrap_or("(unnamed)");
    let mut out = format!("# Constitution: {name}\n");
    // The dimension sections in projection order; each key matches `list_document`'s, and a
    // section absent or empty there is skipped here, so the two projections stay in lockstep.
    for (key, heading) in [
        ("boundaries", "Static boundaries"),
        (
            "semantic_boundaries",
            "Semantic boundaries (signature-coupling)",
        ),
        ("trait_impl_boundaries", "Trait-impl-locality boundaries"),
        ("visibility_boundaries", "Visibility boundaries"),
        ("forbidden_marker_boundaries", "Forbidden-marker boundaries"),
        ("dyn_trait_boundaries", "Dyn-trait boundaries"),
        ("impl_trait_boundaries", "Impl-trait boundaries"),
        ("runtime_boundaries", "Runtime boundaries"),
    ] {
        let Some(Value::Array(items)) = document.get(key) else {
            continue;
        };
        if items.is_empty() {
            continue;
        }
        out.push_str(&format!("\n## {heading}\n"));
        for item in items {
            out.push_str(&boundary_markdown(item));
        }
    }
    out
}

/// One boundary as a Markdown block, with the declared `reason` **foregrounded**: the `target`
/// is the heading; then — when present — the `reason` as a leading blockquote (the block's
/// principle, set apart from the mechanical metadata); then the `rule` with its parameters (the
/// reaction's mechanical shape); then the kind/severity classification, and the owning crate for a
/// module boundary. Every field is read from the JSON projection, so an agent reads the same law
/// the JSON carries.
///
/// The reason leads deliberately (see PROJECT.md, 潛移): it is the gravity-bearing content a model
/// imitates and the repair hint on a violation. The only layout property pinned is this ordering
/// (reason → rule → classification); the exact rendering stays free to evolve under Contract B (see
/// [`constitution_markdown`]). A boundary with no reason emits no blockquote and no orphan blank line.
fn boundary_markdown(boundary: &Value) -> String {
    let field = |key: &str| boundary.get(key).and_then(Value::as_str).unwrap_or("");
    let mut out = format!("\n### `{}`\n", field("target"));

    let reason = field("reason");
    if !reason.is_empty() {
        out.push_str(&format!("\n> {reason}\n\n"));
    }

    out.push_str(&format!("- **rule**: {}", field("rule")));
    let params = boundary_params(boundary);
    if !params.is_empty() {
        out.push_str(&format!(" ({params})"));
    }
    out.push('\n');

    let mut context = format!("- **kind**: {}", field("kind"));
    let severity = field("severity");
    if !severity.is_empty() {
        context.push_str(&format!(" · **severity**: {severity}"));
    }
    if let Some(krate) = boundary.get("crate").and_then(Value::as_str) {
        context.push_str(&format!(" · **crate**: {krate}"));
    }
    out.push_str(&context);
    out.push('\n');
    out
}

/// The rule parameters of a boundary — every JSON field that is not one of the structural keys
/// (kind/target/crate/rule/severity/reason) — rendered inline. This generically surfaces each
/// dimension's specifics (a forbidden set, allowed locations, allowed origins, a posture, a
/// dependency kind) without hard-coding any dimension, so a new dimension's parameters appear
/// in the Markdown the moment they appear in the JSON.
fn boundary_params(boundary: &Value) -> String {
    const STRUCTURAL: [&str; 6] = ["kind", "target", "crate", "rule", "severity", "reason"];
    let Some(object) = boundary.as_object() else {
        return String::new();
    };
    object
        .iter()
        .filter(|(key, _)| !STRUCTURAL.contains(&key.as_str()))
        .map(|(key, value)| format!("{key}: {}", inline_value(value)))
        .collect::<Vec<_>>()
        .join("; ")
}

/// Render a JSON value compactly for a Markdown parameter: a string as itself, an array as a
/// comma-joined list, a scalar via its display, an object via its JSON text. Each rendering is
/// a pure function of the value, so the projection is stable and diffable; within a boundary,
/// `boundary_params` walks the object in serde_json's default `Map` order — lexicographic by
/// key (a `BTreeMap`), not declaration order — which is likewise deterministic.
fn inline_value(value: &Value) -> String {
    match value {
        Value::String(text) => text.clone(),
        Value::Array(items) => items
            .iter()
            .map(inline_value)
            .collect::<Vec<_>>()
            .join(", "),
        Value::Bool(boolean) => boolean.to_string(),
        Value::Number(number) => number.to_string(),
        Value::Null => "null".to_string(),
        Value::Object(_) => value.to_string(),
    }
}

#[cfg(test)]
mod tests {
    use super::{
        constitution_markdown, dispatch, dyn_trait_text, list_document, list_markdown,
        merge_outcomes, report_json, report_sarif, runtime_text, semantic_text, trait_impl_text,
        violations_text, visibility_text,
    };
    use crate::prelude::*;
    use serde_json::Value;
    use std::path::PathBuf;

    fn violation(target: &str, rule: &str, finding: &str, file: Option<&str>) -> Violation {
        Violation::new(
            BoundaryKind::Crate,
            target.to_string(),
            rule.to_string(),
            finding.to_string(),
            format!("reason-for-{target}"),
            Severity::Enforce,
        )
        .with_file(file.map(str::to_string))
    }

    fn enforce_violation(kind: BoundaryKind, finding: &str) -> Violation {
        Violation::new(
            kind,
            "target".to_string(),
            "rule".to_string(),
            finding.to_string(),
            "reason".to_string(),
            Severity::Enforce,
        )
    }

    #[test]
    fn merge_combines_violations_from_both_dimensions() {
        let static_outcome = Outcome::Violations(Report::new(vec![enforce_violation(
            BoundaryKind::Crate,
            "serde",
        )]));
        let semantic_outcome = Outcome::Violations(Report::new(vec![enforce_violation(
            BoundaryKind::Semantic,
            "crate::infra::DbPool",
        )]));
        let merged = merge_outcomes(static_outcome, semantic_outcome);
        match merged {
            Outcome::Violations(report) => assert_eq!(report.violations.len(), 2),
            other => panic!("expected merged violations, got {other:?}"),
        }
    }

    #[test]
    fn merge_is_clean_only_when_both_are_clean() {
        assert_eq!(
            merge_outcomes(Outcome::Clean, Outcome::Clean),
            Outcome::Clean
        );
    }

    #[test]
    fn a_semantic_constitution_error_supersedes_static_violations() {
        let static_outcome = Outcome::Violations(Report::new(vec![enforce_violation(
            BoundaryKind::Crate,
            "serde",
        )]));
        let semantic_outcome = Outcome::ConstitutionError("module 'crate::ghost' not found".into());
        let merged = merge_outcomes(static_outcome, semantic_outcome);
        assert!(matches!(merged, Outcome::ConstitutionError(_)));
        assert_eq!(
            merged.exit_code(),
            2,
            "a constitution error supersedes (exit 2)"
        );
    }

    #[test]
    fn a_static_constitution_error_wins_when_both_error() {
        let merged = merge_outcomes(
            Outcome::ConstitutionError("bad static crate".into()),
            Outcome::ConstitutionError("bad semantic module".into()),
        );
        assert!(
            matches!(merged, Outcome::ConstitutionError(message) if message == "bad static crate"),
            "the static error is checked first and wins deterministically",
        );
    }

    #[test]
    fn semantic_text_lists_each_boundary() {
        let boundary = SemanticBoundary::in_crate("app")
            .module("crate::domain")
            .must_not_expose("crate::infra")
            .because("the domain API must not leak infrastructure types");
        let text = semantic_text(&[boundary]);
        assert!(text.contains("module crate::domain in app"), "{text}");
        assert!(text.contains("must not expose: crate::infra"), "{text}");
    }

    #[test]
    fn dyn_trait_text_lists_each_boundary() {
        let boundary = DynTraitBoundary::in_crate("app")
            .module("crate::core")
            .must_not_expose_dyn()
            .because("the core seam is statically dispatched");
        let text = dyn_trait_text(&[boundary]);
        assert!(text.contains("module crate::core in app"), "{text}");
        assert!(text.contains("must not expose dyn"), "{text}");
        assert!(
            text.contains("the core seam is statically dispatched"),
            "{text}"
        );
    }

    #[test]
    fn dyn_trait_boundary_projects_into_list_document_and_markdown() {
        let c = Constitution::new("app").dyn_trait_boundary(
            DynTraitBoundary::in_crate("app")
                .module("crate::core")
                .must_not_expose_dyn()
                .because("the core seam is statically dispatched"),
        );
        let doc = list_document(&c);
        let arr = doc
            .get("dyn_trait_boundaries")
            .and_then(Value::as_array)
            .expect("dyn_trait_boundaries projected");
        assert_eq!(arr.len(), 1);
        assert_eq!(arr[0]["rule"], "must not expose dyn");
        assert!(
            arr[0].get("forbidden").is_none(),
            "shape-only: no forbidden set"
        );
        let md = list_markdown(&doc);
        assert!(md.contains("## Dyn-trait boundaries"), "{md}");
        assert!(md.contains("must not expose dyn"), "{md}");
        assert!(
            md.contains("the core seam is statically dispatched"),
            "{md}"
        );
    }

    #[test]
    fn impl_trait_boundary_projects_into_list_document_and_markdown() {
        let c = Constitution::new("app").impl_trait_boundary(
            ImplTraitBoundary::in_crate("app")
                .module("crate::core")
                .must_not_expose_impl_trait()
                .because("the core seam must return named types, not an existential"),
        );
        let doc = list_document(&c);
        let arr = doc["impl_trait_boundaries"].as_array().expect("projected");
        assert_eq!(arr.len(), 1);
        assert_eq!(arr[0]["rule"], "must not expose impl trait");
        assert_eq!(arr[0]["target"], "crate::core");
        let md = list_markdown(&doc);
        assert!(md.contains("## Impl-trait boundaries"), "{md}");
        assert!(md.contains("must not expose impl trait"), "{md}");
    }

    #[test]
    fn operand_scoped_impl_trait_boundary_projects_its_forbidden_operands() {
        let c = Constitution::new("app").impl_trait_boundary(
            ImplTraitBoundary::in_crate("app")
                .module("crate::core")
                .must_not_expose_impl_trait_of(["crate::ports::Port"])
                .because("the core seam must not return an existential Port"),
        );
        let doc = list_document(&c);
        let arr = doc["impl_trait_boundaries"].as_array().expect("projected");
        assert_eq!(arr[0]["rule"], "must not expose impl trait");
        assert_eq!(arr[0]["forbidden"][0], "crate::ports::Port");
        let md = list_markdown(&doc);
        assert!(
            md.contains("forbidden: crate::ports::Port"),
            "the operand set surfaces as a param:\n{md}"
        );
    }

    #[test]
    fn operand_scoped_dyn_boundary_projects_its_forbidden_operands() {
        let c = Constitution::new("app").dyn_trait_boundary(
            DynTraitBoundary::in_crate("app")
                .module("crate::core")
                .must_not_expose_dyn_of(["crate::ports::Port"])
                .because("the core seam must not leak a dyn Port"),
        );
        let doc = list_document(&c);
        let arr = doc["dyn_trait_boundaries"].as_array().expect("projected");
        assert_eq!(arr[0]["rule"], "must not expose dyn");
        assert_eq!(
            arr[0]["forbidden"][0], "crate::ports::Port",
            "an operand-scoped boundary projects its forbidden operand set"
        );
        let md = list_markdown(&doc);
        assert!(
            md.contains("forbidden: crate::ports::Port"),
            "the operand set surfaces as a generic param:\n{md}"
        );
    }

    #[test]
    fn trait_impl_text_lists_each_boundary() {
        let boundary = TraitImplBoundary::in_crate("app")
            .trait_("crate::command::Command")
            .only_implemented_in("crate::commands")
            .and_in("crate::builtins")
            .because("Command impls live with the registry");
        let text = trait_impl_text(&[boundary]);
        assert!(
            text.contains("trait crate::command::Command in app"),
            "{text}"
        );
        assert!(
            text.contains("may only be implemented in: crate::commands, crate::builtins"),
            "{text}"
        );
    }

    #[test]
    fn visibility_text_lists_each_boundary_and_is_empty_when_none() {
        // The empty-guard protects existing `list` output: a project not using the
        // dimension gets byte-identical projection (no section emitted).
        assert_eq!(visibility_text(&[]), "");
        let boundary = VisibilityBoundary::in_crate("app")
            .module("crate::internal")
            .must_not_declare_pub()
            .because("internal is an impl detail");
        let text = visibility_text(&[boundary]);
        assert!(text.contains("module crate::internal in app"), "{text}");
        assert!(text.contains("must not declare pub items"), "{text}");
    }

    #[test]
    fn merge_folds_a_trait_impl_violation_into_the_report() {
        // The three-dimension composition reuses the same binary merge: a trait-impl
        // finding lands in the one aggregated report alongside static and semantic ones.
        let static_outcome = Outcome::Violations(Report::new(vec![enforce_violation(
            BoundaryKind::Crate,
            "serde",
        )]));
        let trait_impl_outcome = Outcome::Violations(Report::new(vec![enforce_violation(
            BoundaryKind::Semantic,
            "crate::domain (impl for Foo)",
        )]));
        let merged = merge_outcomes(static_outcome, trait_impl_outcome);
        match merged {
            Outcome::Violations(report) => assert_eq!(report.violations.len(), 2),
            other => panic!("expected merged violations, got {other:?}"),
        }
    }

    fn fixture(name: &str) -> String {
        PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("tests/fixtures")
            .join(name)
            .join("Cargo.toml")
            .to_string_lossy()
            .into_owned()
    }

    /// The Tianheng workspace manifest, two levels up. `None` when it is absent — e.g. inside a
    /// published `.crate` tarball, which has no workspace root — so the workspace-dependent
    /// dispatch tests below SKIP rather than fail when the crate is tested standalone. In the
    /// repo the path exists, so they run as a real end-to-end gate.
    fn workspace_manifest() -> Option<PathBuf> {
        let path = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../Cargo.toml");
        if path.exists() {
            return Some(path);
        }
        // Absent. In the repo/CI the workspace root always exists, so CI sets
        // TIANHENG_WORKSPACE_TESTS=1 to turn a missing manifest (a checkout/layout regression)
        // into a LOUD failure rather than a silent skip of the gate. Without the env (e.g. a
        // packaged .crate tested standalone) the absence is legitimate, so skip.
        assert!(
            std::env::var_os("TIANHENG_WORKSPACE_TESTS").is_none(),
            "workspace manifest expected but absent while TIANHENG_WORKSPACE_TESTS is set — \
             the dispatch gate must not silently skip in CI"
        );
        None
    }

    fn example_constitution() -> Constitution {
        Constitution::new("example").boundary(
            CrateBoundary::crate_("example-core")
                .deny_external_dependencies()
                .because("example-core must stay dependency-light"),
        )
    }

    fn run_args(args: &[&str]) -> u8 {
        dispatch(&example_constitution(), args.iter().map(|s| s.to_string()))
    }

    // Most runner unit tests below need no fixture: each asserts an exit code decided
    // during argument parsing, before any workspace is observed. The reaction paths that
    // require a real workspace are exercised against one directly: `tests/self_governance.rs`
    // drives the static `check` end-to-end against Tianheng's own workspace, and the
    // dispatch tests below (e.g. `the_trait_impl_dimension_is_wired_through_dispatch`) drive
    // each dimension through `dispatch` + real `cargo metadata`. The per-dimension finding
    // logic is unit-tested in its own crate's pure heart (`hunyi`).

    #[test]
    fn the_trait_impl_dimension_is_wired_through_dispatch() {
        // End-to-end proof the new dimension is composed into `dispatch` (not only
        // unit-tested in isolation): an unresolvable trait anchor must flow through dispatch
        // and real `cargo metadata` to a constitution error (exit 2). The static
        // constitution is empty (clean), so the exit-2 can only come from the trait-impl
        // dimension — proving it is actually evaluated.
        let Some(manifest) = workspace_manifest() else {
            return; // no workspace root (e.g. a packaged crate) — skip this end-to-end test
        };
        let boundary = TraitImplBoundary::in_crate("xuanji")
            .trait_("crate::NoSuchTrait")
            .only_implemented_in("crate::nowhere")
            .because("wiring check");
        let code = dispatch(
            &Constitution::new("wiring").trait_impl_boundary(boundary),
            [
                "tianheng".to_string(),
                "check".to_string(),
                "--manifest-path".to_string(),
                manifest.to_string_lossy().into_owned(),
            ],
        );
        assert_eq!(
            code, 2,
            "an unresolvable trait anchor reaches exit 2 through dispatch"
        );
    }

    #[test]
    fn the_visibility_dimension_is_wired_through_dispatch() {
        // End-to-end proof the visibility dimension is composed into `dispatch`: an
        // unresolvable module anchor flows through dispatch + real `cargo metadata` to a
        // constitution error (exit 2). Empty static constitution, so exit-2 can only come
        // from the visibility dimension.
        let Some(manifest) = workspace_manifest() else {
            return; // no workspace root (e.g. a packaged crate) — skip this end-to-end test
        };
        let boundary = VisibilityBoundary::in_crate("xuanji")
            .module("crate::no_such_module")
            .must_not_declare_pub()
            .because("wiring check");
        let code = dispatch(
            &Constitution::new("wiring").visibility_boundary(boundary),
            [
                "tianheng".to_string(),
                "check".to_string(),
                "--manifest-path".to_string(),
                manifest.to_string_lossy().into_owned(),
            ],
        );
        assert_eq!(
            code, 2,
            "an unresolvable visibility module anchor reaches exit 2 through dispatch"
        );
    }

    #[test]
    fn the_runtime_dimension_is_wired_through_dispatch() {
        // End-to-end proof the 漏刻 CI face is composed into `dispatch`: a declared runtime seam
        // with no probe anywhere in the workspace flows through dispatch + real `cargo metadata`
        // (member-src-dir resolution) + the probe-coverage audit to an enforce violation (exit 1).
        // The static and semantic dimensions are empty, so the exit-1 can only come from the
        // runtime audit — proving it is actually evaluated against the workspace.
        let Some(manifest) = workspace_manifest() else {
            return; // no workspace root (e.g. a packaged crate) — skip this end-to-end test
        };
        let args = || {
            [
                "tianheng".to_string(),
                "check".to_string(),
                "--manifest-path".to_string(),
                manifest.to_string_lossy().into_owned(),
            ]
        };
        let boundary = RuntimeBoundary::at("a-seam-no-probe-covers")
            .only_origins(["app::domain"])
            .because("wiring check");
        let code = dispatch(&Constitution::new("wiring").runtime(boundary), args());
        assert_eq!(
            code, 1,
            "a declared-but-unprobed runtime seam reaches exit 1 through dispatch"
        );
        // Causation: with NO runtime boundary the audit is skipped, so the same workspace exits
        // 0 — proving the exit-1 above is caused by the runtime dimension, not pre-existing drift.
        assert_eq!(
            dispatch(&Constitution::new("wiring"), args()),
            0,
            "an empty constitution over the same workspace is clean (the audit is skipped)"
        );
    }

    #[test]
    fn the_runtime_audit_reports_the_declared_unprobed_seam() {
        // Specificity (robust to noise): resolve the workspace's member src roots and run the
        // audit directly, asserting the *named* declared-unprobed seam surfaces — so this cannot
        // pass for the wrong reason (Direction-B / un-auditable noise elsewhere).
        let Some(manifest) = workspace_manifest() else {
            return; // no workspace root (e.g. a packaged crate) — skip this end-to-end test
        };
        let src_dirs = crate::workspace_member_src_dirs(&manifest).expect("resolve src dirs");
        let boundary = RuntimeBoundary::at("a-seam-no-probe-covers")
            .only_origins(["app::domain"])
            .because("wiring check");
        let outcome = crate::audit_probe_coverage(&[boundary], &src_dirs);
        match outcome {
            Outcome::Violations(report) => assert!(
                report
                    .violations
                    .iter()
                    .any(|v| v.target == "a-seam-no-probe-covers"
                        && v.finding.contains("no assert_boundary! probe")),
                "the declared-unprobed seam must be the reported finding: {:?}",
                report.violations
            ),
            other => panic!("expected a violation naming the unprobed seam, got {other:?}"),
        }
    }

    #[test]
    fn list_document_covers_every_populated_dimension() {
        // The previous json-list test ran only an empty SemanticBoundaries, so the projection's
        // per-dimension key insertion was never exercised (a blind spot). Build one boundary of
        // every dimension and assert each lands in the document — and that an empty dimension
        // adds no key (the static-only projection stays byte-identical).
        let empty = Constitution::new("empty");
        let doc = list_document(&empty);
        assert!(
            doc.get("semantic_boundaries").is_none(),
            "empty adds no key: {doc}"
        );
        assert!(
            doc.get("runtime_boundaries").is_none(),
            "empty adds no key: {doc}"
        );

        let full = Constitution::new("full")
            .boundary(
                CrateBoundary::crate_("core")
                    .deny_external_dependencies()
                    .because("core stays light"),
            )
            .signature_boundary(
                SemanticBoundary::in_crate("app")
                    .module("crate::domain")
                    .must_not_expose("crate::infra")
                    .because("no infra leak"),
            )
            .trait_impl_boundary(
                TraitImplBoundary::in_crate("app")
                    .trait_("crate::Command")
                    .only_implemented_in("crate::commands")
                    .because("impls live with the registry"),
            )
            .visibility_boundary(
                VisibilityBoundary::in_crate("app")
                    .module("crate::internal")
                    .must_not_declare_pub()
                    .because("internal is private"),
            )
            .forbidden_marker_boundary(
                ForbiddenMarkerBoundary::in_crate("app")
                    .module("crate::domain")
                    .must_not_acquire("serde::Serialize")
                    .because("domain is not wire"),
            )
            .runtime(
                RuntimeBoundary::at("domain-entry")
                    .only_origins(["app::domain"])
                    .because("only domain crosses"),
            );
        let doc = list_document(&full);
        // Each populated dimension is a non-empty array whose first entry carries the kind and
        // target the projection contract promises (deep-checked, not merely present).
        for (key, kind, target) in [
            ("semantic_boundaries", "semantic", "crate::domain"),
            ("trait_impl_boundaries", "semantic", "crate::Command"),
            ("visibility_boundaries", "semantic", "crate::internal"),
            ("forbidden_marker_boundaries", "semantic", "crate::domain"),
            ("runtime_boundaries", "runtime", "domain-entry"),
        ] {
            let arr = doc[key]
                .as_array()
                .unwrap_or_else(|| panic!("{key} must be an array: {doc}"));
            assert!(!arr.is_empty(), "{key} must be non-empty: {doc}");
            assert_eq!(arr[0]["kind"], kind, "{key}[0] kind: {}", arr[0]);
            assert_eq!(arr[0]["target"], target, "{key}[0] target: {}", arr[0]);
        }

        // And the text projection of the runtime section is non-empty and names the seam.
        let text = runtime_text(full.runtime_boundaries());
        assert!(text.contains("seam domain-entry"), "{text}");
    }

    /// A multi-dimension constitution to exercise the Markdown projection across every
    /// dimension at once (mirrors the JSON test's `full`).
    fn full_constitution() -> Constitution {
        Constitution::new("full")
            .boundary(
                CrateBoundary::crate_("core")
                    .deny_external_dependencies()
                    .because("core stays light"),
            )
            .signature_boundary(
                SemanticBoundary::in_crate("app")
                    .module("crate::domain")
                    .must_not_expose("crate::infra")
                    .because("no infra leak"),
            )
            .trait_impl_boundary(
                TraitImplBoundary::in_crate("app")
                    .trait_("crate::Command")
                    .only_implemented_in("crate::commands")
                    .because("impls live with the registry"),
            )
            .visibility_boundary(
                VisibilityBoundary::in_crate("app")
                    .module("crate::internal")
                    .must_not_declare_pub()
                    .because("internal is private"),
            )
            .forbidden_marker_boundary(
                ForbiddenMarkerBoundary::in_crate("app")
                    .module("crate::domain")
                    .must_not_acquire("serde::Serialize")
                    .because("domain is not wire"),
            )
            .runtime(
                RuntimeBoundary::at("domain-entry")
                    .only_origins(["app::domain"])
                    .because("only domain crosses"),
            )
    }

    #[test]
    fn list_markdown_covers_every_dimension_with_target_rule_and_reason() {
        // The Markdown is rendered from `list_document`, so this also proves it carries no less
        // than the JSON: every dimension's target, rule parameter, and declared reason appear.
        let md = list_markdown(&list_document(&full_constitution()));
        assert!(md.contains("# Constitution: full"), "{md}");
        // A section heading per non-empty dimension.
        for heading in [
            "## Static boundaries",
            "## Semantic boundaries",
            "## Trait-impl-locality boundaries",
            "## Visibility boundaries",
            "## Forbidden-marker boundaries",
            "## Runtime boundaries",
        ] {
            assert!(md.contains(heading), "missing {heading} in:\n{md}");
        }
        // Each dimension's target, a rule parameter, and its reason (the agent-actionable triple).
        for needle in [
            "core",                // static target
            "core stays light",    // static reason
            "crate::domain",       // semantic target
            "crate::infra",        // semantic forbidden param
            "no infra leak",       // semantic reason
            "crate::Command",      // trait-impl target
            "crate::commands",     // trait-impl allowed_locations param
            "crate::internal",     // visibility target
            "serde::Serialize",    // forbidden-marker param
            "domain-entry",        // runtime seam target
            "app::domain",         // runtime allowed_origins param
            "only domain crosses", // runtime reason
        ] {
            assert!(md.contains(needle), "missing '{needle}' in:\n{md}");
        }
    }

    #[test]
    fn constitution_markdown_equals_the_cli_projection_byte_for_byte() {
        // The public helper MUST add nothing of its own — no preamble, no trailing newline — so
        // it equals what the `list --format markdown` branch prints (`list_markdown(&list_document)`,
        // via `print!`). This guards Contract A's "same renderer, no parallel projection path":
        // a stray newline or wrapper here would silently drift the agent artifact from the CLI.
        let c = full_constitution();
        assert_eq!(constitution_markdown(&c), list_markdown(&list_document(&c)));
    }

    #[test]
    fn markdown_foregrounds_the_reason_before_rule_and_classification() {
        // Contract B / 潛移: the reason leads the block. This asserts the ORDERING INVARIANT ONLY
        // (reason before rule before kind/severity). It deliberately does NOT assert the blockquote
        // rendering — the spec frees "the blockquote choice, wording, spacing" — so the layout stays
        // free to evolve; never a byte-for-byte snapshot.
        let c = Constitution::new("t").boundary(
            CrateBoundary::crate_("core")
                .deny_external_dependencies()
                .because("the gravity-bearing principle text"),
        );
        let md = constitution_markdown(&c);
        let r = md
            .find("the gravity-bearing principle text")
            .expect("reason");
        let rule = md.find("**rule**").expect("rule");
        let kind = md.find("**kind**").expect("kind");
        assert!(
            r < rule && rule < kind,
            "reason must lead, then rule, then classification:\n{md}"
        );
    }

    #[test]
    fn markdown_projects_a_dependency_source_boundary_with_its_allowed_sources() {
        // The source rule projects through the generic static-boundary path (no per-rule
        // markdown code): its label and the `allowed_sources` param surface as params.
        let c = Constitution::new("t").boundary(
            CrateBoundary::crate_("infra")
                .restrict_dependency_sources_to([SourceKind::Registry, SourceKind::Path])
                .because("infra must publish to crates.io"),
        );
        let md = constitution_markdown(&c);
        assert!(
            md.contains("restrict dependency sources to"),
            "the source rule label surfaces:\n{md}"
        );
        assert!(
            md.contains("allowed_sources: registry, path"),
            "the allowed source kinds surface as a generic param:\n{md}"
        );
    }

    #[test]
    fn markdown_reasonless_boundary_has_no_blockquote_or_orphan_blank_line() {
        // No reason → no blockquote, and the heading is immediately followed by the rule bullet
        // (no orphan blank line where the blockquote would have been).
        let c = Constitution::new("t").boundary(
            CrateBoundary::crate_("core")
                .deny_external_dependencies()
                .because(""),
        );
        let md = constitution_markdown(&c);
        assert!(!md.contains("\n> "), "no blockquote when no reason:\n{md}");
        assert!(
            md.contains("### `core`\n- **rule**"),
            "heading immediately followed by the rule bullet:\n{md}"
        );
    }

    #[test]
    fn report_text_leads_with_reason_and_shows_the_offending_file() {
        let report = Report::new(vec![violation(
            "crate::core",
            "must not import crate::adapter",
            "crate::adapter::Db",
            Some("src/core/mod.rs"),
        )]);
        let text = violations_text(&report);
        let reason = text.find("Reason:").expect("reason");
        let boundary = text.find("Boundary:").expect("boundary");
        let rule = text.find("Rule:").expect("rule");
        let found = text.find("Found:").expect("found");
        let file = text.find("File:").expect("file");
        let reaction = text.find("Reaction:").expect("reaction");
        assert!(
            reason < boundary && boundary < rule && rule < found && found < file && file < reaction,
            "order must be reason → boundary → rule → found → file → reaction:\n{text}"
        );
        assert!(
            text.contains("File:\n  src/core/mod.rs"),
            "the offending file is shown as the repair location:\n{text}"
        );
    }

    #[test]
    fn report_text_omits_the_file_element_when_absent() {
        let report = Report::new(vec![violation("crate::x", "rule", "finding", None)]);
        let text = violations_text(&report);
        assert!(
            !text.contains("File:"),
            "no file element when the violation carries none:\n{text}"
        );
    }

    #[test]
    fn report_text_groups_violations_by_boundary() {
        // Input order is intentionally unsorted; the text groups by (target, rule).
        let report = Report::new(vec![
            violation("z-crate", "r1", "f", None),
            violation("a-crate", "r1", "f", None),
            violation("a-crate", "r0", "f", None),
        ]);
        let text = violations_text(&report);
        assert!(
            text.find("Boundary:\n  a-crate").unwrap() < text.find("Boundary:\n  z-crate").unwrap(),
            "the a-crate group precedes z-crate:\n{text}"
        );
        assert!(
            text.find("\n  r0").unwrap() < text.find("\n  r1").unwrap(),
            "within a-crate, r0 precedes r1:\n{text}"
        );
    }

    #[test]
    fn json_projection_is_unchanged_by_the_text_grouping() {
        // The text sort is presentation-only: the JSON keeps the input (detection) order.
        let outcome = Outcome::Violations(Report::new(vec![
            violation("z-crate", "r", "f", None),
            violation("a-crate", "r", "f", None),
        ]));
        let json = report_json(&outcome, &[], None);
        assert!(
            json.find("z-crate").unwrap() < json.find("a-crate").unwrap(),
            "JSON keeps input order (z before a), unaffected by the text grouping:\n{json}"
        );
    }

    #[test]
    fn sarif_projects_violations_with_file_level_locations_and_no_region() {
        let outcome = Outcome::Violations(Report::new(vec![
            violation(
                "crate::core",
                "must not import crate::adapter",
                "crate::adapter::Db",
                Some("src/core/mod.rs"),
            ),
            violation("dep-crate", "deny external", "serde", None),
        ]));
        let doc: serde_json::Value =
            serde_json::from_str(&report_sarif(&outcome)).expect("valid SARIF JSON");
        assert_eq!(doc["version"], "2.1.0");
        assert_eq!(doc["runs"][0]["tool"]["driver"]["name"], "tianheng");
        let results = doc["runs"][0]["results"].as_array().expect("results array");
        assert_eq!(results.len(), 2, "one result per non-baselined violation");
        // With a file: error level, ruleId in place, file-level location with NO region.
        assert_eq!(results[0]["level"], "error");
        assert_eq!(results[0]["ruleId"], "must not import crate::adapter");
        assert!(
            results[0]["message"]["text"]
                .as_str()
                .unwrap()
                .contains("reason-for-crate::core")
        );
        assert_eq!(
            results[0]["locations"][0]["physicalLocation"]["artifactLocation"]["uri"],
            "src/core/mod.rs"
        );
        assert!(
            results[0]["locations"][0]["physicalLocation"]["region"].is_null(),
            "no region — the line is not observed, never fabricated"
        );
        // File-less violation: no locations at all.
        assert!(
            results[1]["locations"].is_null(),
            "a file-less violation projects no location"
        );
    }

    #[test]
    fn sarif_clean_is_empty_and_constitution_error_marks_execution_unsuccessful() {
        let clean: serde_json::Value =
            serde_json::from_str(&report_sarif(&Outcome::Clean)).unwrap();
        assert!(
            clean["runs"][0]["results"].as_array().unwrap().is_empty(),
            "clean → empty results"
        );
        let err: serde_json::Value =
            serde_json::from_str(&report_sarif(&Outcome::ConstitutionError("bad law".into())))
                .unwrap();
        assert_eq!(
            err["runs"][0]["invocations"][0]["executionSuccessful"],
            serde_json::Value::Bool(false),
            "a constitution error marks the invocation unsuccessful (required by SARIF)"
        );
        assert!(
            err["runs"][0]["invocations"][0]["toolExecutionNotifications"][0]["message"]["text"]
                .as_str()
                .unwrap()
                .contains("bad law")
        );
    }

    #[test]
    fn sarif_exits_like_json() {
        // Presentation only: the same outcome exits identically under each machine format.
        for format in ["json", "sarif"] {
            assert_eq!(
                run_args(&[
                    "tianheng",
                    "check",
                    "--manifest-path",
                    &fixture("violating"),
                    "--format",
                    format,
                ]),
                1,
                "violating fixture exits 1 under --format {format}"
            );
            assert_eq!(
                run_args(&[
                    "tianheng",
                    "check",
                    "--manifest-path",
                    &fixture("clean"),
                    "--format",
                    format,
                ]),
                0,
                "clean fixture exits 0 under --format {format}"
            );
        }
    }

    #[test]
    fn list_rejects_the_check_only_sarif_format() {
        // SARIF projects the reaction, not the law — check-only, like markdown is list-only.
        assert_eq!(run_args(&["tianheng", "list", "--format", "sarif"]), 2);
    }

    #[test]
    fn list_markdown_empty_constitution_has_a_title_but_no_sections() {
        // An empty dimension adds no section, mirroring the text and JSON projections.
        let md = list_markdown(&list_document(&Constitution::new("empty")));
        assert!(md.contains("# Constitution: empty"), "{md}");
        assert!(
            !md.contains("\n## "),
            "no dimension sections expected:\n{md}"
        );
    }

    #[test]
    fn list_accepts_markdown_format() {
        // `list --format markdown` is a pure projection: it observes no workspace and exits 0.
        assert_eq!(run_args(&["tianheng", "list", "--format", "markdown"]), 0);
        assert_eq!(run_args(&["tianheng", "list", "--format=markdown"]), 0);
    }

    #[test]
    fn check_rejects_the_list_only_markdown_format() {
        // markdown is a list-only projection of the declared law; check's machine output is the
        // JSON report, so check --format markdown is a usage error (exit 2), not a silent fallback.
        assert_eq!(
            run_args(&[
                "tianheng",
                "check",
                "--manifest-path",
                &fixture("clean"),
                "--format",
                "markdown",
            ]),
            2
        );
    }

    #[test]
    fn the_runtime_projection_distinguishes_posture() {
        // A `.panic_on_violation()` boundary must NOT project identically to a default event-only
        // one — posture is part of the declared law, and the projection is faithful.
        let event = Constitution::new("c").runtime(
            RuntimeBoundary::at("s")
                .only_origins(["app::a"])
                .because("default event"),
        );
        let panicking = Constitution::new("c").runtime(
            RuntimeBoundary::at("s")
                .only_origins(["app::a"])
                .panic_on_violation()
                .because("opt-in panic"),
        );
        let ej = list_document(&event)["runtime_boundaries"][0].clone();
        let pj = list_document(&panicking)["runtime_boundaries"][0].clone();
        assert_eq!(ej["posture"], "event", "default posture is event: {ej}");
        assert_eq!(pj["posture"], "panic", "opt-in posture is panic: {pj}");
        assert_ne!(ej, pj, "posture must make the two projections differ");
        assert!(
            runtime_text(panicking.runtime_boundaries()).contains("posture: panic"),
            "the text projection names the posture too"
        );
    }

    #[test]
    fn both_baseline_flags_exit_2() {
        assert_eq!(
            run_args(&[
                "tianheng",
                "check",
                "--manifest-path",
                &fixture("clean"),
                "--baseline",
                "a.json",
                "--write-baseline",
                "b.json",
            ]),
            2
        );
    }

    #[test]
    fn unknown_format_exits_2() {
        assert_eq!(
            run_args(&[
                "tianheng",
                "check",
                "--manifest-path",
                &fixture("clean"),
                "--format",
                "yaml",
            ]),
            2
        );
    }

    #[test]
    fn flag_missing_its_value_is_a_usage_error() {
        // The foot-gun: a value-taking flag with no following token must fail loud
        // (exit 2), not silently downgrade (--format -> text and exit 0, --baseline
        // / --write-baseline -> a plain check). The trailing flag errors during
        // parsing, before any workspace is observed, so no fixture is needed.
        for flag in [
            "--manifest-path",
            "--baseline",
            "--write-baseline",
            "--format",
        ] {
            assert_eq!(
                run_args(&[
                    "tianheng",
                    "check",
                    "--manifest-path",
                    &fixture("clean"),
                    flag
                ]),
                2,
                "{flag} without a value must exit 2",
            );
        }
    }

    #[test]
    fn list_needs_no_manifest_path_and_exits_0() {
        assert_eq!(run_args(&["tianheng", "list"]), 0);
    }

    #[test]
    fn list_json_exits_0() {
        assert_eq!(run_args(&["tianheng", "list", "--format", "json"]), 0);
    }

    #[test]
    fn list_unknown_format_is_a_usage_error() {
        assert_eq!(run_args(&["tianheng", "list", "--format", "yaml"]), 2);
    }

    #[test]
    fn misspelled_flag_fails_loud_instead_of_being_ignored() {
        // The foot-gun: a typo'd --write-baseline must not silently run a plain
        // check (and write no baseline).
        assert_eq!(
            run_args(&[
                "tianheng",
                "check",
                "--manifest-path",
                &fixture("violating"),
                "--write-baselin",
                "out.json",
            ]),
            2
        );
    }

    #[test]
    fn unknown_flag_exits_2() {
        assert_eq!(
            run_args(&[
                "tianheng",
                "check",
                "--manifest-path",
                &fixture("clean"),
                "--frobnicate",
            ]),
            2
        );
    }

    #[test]
    fn stray_positional_exits_2() {
        assert_eq!(
            run_args(&[
                "tianheng",
                "check",
                "stray",
                "--manifest-path",
                &fixture("clean")
            ]),
            2
        );
    }

    #[test]
    fn list_unknown_flag_exits_2() {
        assert_eq!(run_args(&["tianheng", "list", "--bogus"]), 2);
    }

    #[test]
    fn list_rejects_check_only_flags() {
        // `list` observes no workspace, so a check-only flag is a usage error (exit 2),
        // never a silent no-op. Each is rejected during parsing/dispatch, no fixture.
        for args in [
            &["tianheng", "list", "--manifest-path", "Cargo.toml"][..],
            &["tianheng", "list", "--baseline", "b.json"][..],
            &["tianheng", "list", "--write-baseline", "b.json"][..],
            &["tianheng", "list", "--warn-uncovered"][..],
        ] {
            assert_eq!(
                run_args(args),
                2,
                "a check-only flag supplied to list must exit 2: {args:?}",
            );
        }
    }
}
