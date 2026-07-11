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
    Baseline, Coverage, Outcome, Report, ViolationId, apply_baseline, check_and_cover,
    constitution_text, report_json, workspace_member_src_dirs,
};
use louke::audit_probe_coverage;

use crate::Constitution;

/// The non-`Outcome` CLI exit codes. They mirror [`Outcome::exit_code`]'s contract — `0` clean,
/// `1` violation, `2` cannot-judge (constitution/scan/usage error) — for the CLI paths that never
/// build an `Outcome`: a usage error, a missing manifest, a baseline-write failure. A violation
/// always flows through an `Outcome`, so `1` never appears as a bare return here. Named so every
/// runner path speaks the one 0/1/2 contract rather than a bare literal that could silently drift
/// from `exit_code()`.
const EXIT_OK: u8 = 0;
const EXIT_CANNOT_JUDGE: u8 = 2;

mod projection;
use projection::*;
pub use projection::{constitution_markdown, projection_gate};

mod render;
use render::{report, report_coverage, report_sarif, report_violations};
mod term_color;
use term_color::Style;

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
                print!("{}", async_exposure_text(&semantic.async_exposure));
                print!("{}", unsafe_text(&semantic.unsafe_confinement));
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
        return EXIT_OK;
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

    // A contradictory flag pair is a pure usage error, independent of any workspace — check it
    // before resolving the manifest, so an also-absent `--manifest-path` (whose "no Cargo.toml
    // found" diagnostic would otherwise fire first) cannot mask the real misconfiguration.
    if baseline_path.is_some() && write_baseline_path.is_some() {
        return usage("--baseline and --write-baseline are mutually exclusive");
    }

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
                return EXIT_CANNOT_JUDGE;
            }
        },
    };

    // One `cargo metadata` read feeds both the static reaction outcome and coverage; the
    // semantic dimension reads its own (it has no coverage notion). The two outcomes compose
    // into one: a constitution error from either supersedes (the run's verdict is
    // untrustworthy), and otherwise the violations merge into a single report. Coverage
    // stays static-only.
    let (static_outcome, observed_coverage) =
        check_and_cover(constitution.static_boundaries(), &manifest_path);
    // Compose the dimensions in order, stopping at the first that raises a constitution error
    // (its supersede/merge semantics are stated above and in `merge_outcomes`).
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
    //
    // Run whenever the constitution evaluated — even with an **empty** declared-boundary set:
    // `audit_probe_coverage(&[], …)` reacts to an `assert_boundary!` probe left in source after its
    // `RuntimeBoundary` was deleted (an undeclared seam), catching at CI the orphan that would
    // otherwise panic in production. On a workspace with no probes it is a no-op, so a pure
    // static/semantic run is undisturbed.
    if !matches!(outcome, Outcome::ConstitutionError(_)) {
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
    EXIT_CANNOT_JUDGE
}

/// Walk up from the current directory to the nearest `Cargo.toml`, cargo-style, so
/// `check` can default its target like `cargo` does when `--manifest-path` is omitted.
/// The shell reads the cwd; the walk itself is the pure [`nearest_manifest_from`].
fn nearest_manifest() -> Option<PathBuf> {
    nearest_manifest_from(std::env::current_dir().ok()?)
}

/// The pure ascent: from `start`, return the first ancestor (including `start`) that holds a
/// `Cargo.toml`, or `None` once the root is passed. Split out from [`nearest_manifest`] so the
/// walk is testable without touching the process-global cwd.
fn nearest_manifest_from(start: PathBuf) -> Option<PathBuf> {
    let mut dir = start;
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
        eprintln!(
            "{}",
            Style::detect().error(&format!("Tianheng constitution error: {message}"))
        );
        eprintln!("refusing to write a baseline from a constitution that could not be evaluated");
        return EXIT_CANNOT_JUDGE;
    }
    let empty = Report::empty();
    let report = match outcome {
        Outcome::Violations(report) => report,
        _ => &empty,
    };
    // Metadata-preserving merge: carry each surviving entry's owner/tracker forward by identity, so
    // re-running --write-baseline never silently wipes hand-added governance records. A missing file
    // is the normal first write (no warning); an existing-but-unreadable/unparseable file falls back
    // to a fresh baseline but WARNS, so the metadata loss is visible rather than silent.
    let baseline = match std::fs::read_to_string(path) {
        Ok(text) => match Baseline::from_json(&text) {
            Ok(existing) => Baseline::of_preserving(report, &existing),
            Err(err) => {
                eprintln!(
                    "Tianheng: existing baseline {path} could not be parsed ({err}); writing a \
                     fresh baseline — owner/tracker metadata is not carried forward"
                );
                Baseline::of(report)
            }
        },
        Err(err) if err.kind() == std::io::ErrorKind::NotFound => Baseline::of(report),
        Err(err) => {
            eprintln!(
                "Tianheng: existing baseline {path} could not be read ({err}); writing a fresh \
                 baseline — owner/tracker metadata is not carried forward"
            );
            Baseline::of(report)
        }
    };
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
            ReportFormat::Text => eprintln!(
                "{}",
                Style::detect().error(&format!("Tianheng constitution error: {message}"))
            ),
        }
        return EXIT_CANNOT_JUDGE;
    }

    let baseline = match std::fs::read_to_string(path) {
        Ok(text) => match Baseline::from_json(&text) {
            Ok(baseline) => baseline,
            Err(err) => {
                eprintln!("Tianheng: invalid baseline {path}: {err}");
                return EXIT_CANNOT_JUDGE;
            }
        },
        Err(err) => {
            eprintln!("Tianheng: cannot read baseline {path}: {err}");
            return EXIT_CANNOT_JUDGE;
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

#[cfg(test)]
mod tests;
