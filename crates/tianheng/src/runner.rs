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

use std::path::{Path, PathBuf};
use std::process::ExitCode;
use std::{fs::OpenOptions, io::Write};

use guibiao::{
    Baseline, BaselineEntry, Coverage, Outcome, Report, apply_baseline, check_and_cover,
    constitution_text, report_json, report_json_with_stale_policy, stale_policy,
};
use louke::audit_probe_coverage;
use xingbiao::{cargo_metadata, member_root_files};

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
use render::{
    disallow_stale_message, report, report_coverage, report_sarif, report_sarif_with_stale,
    report_violations,
};
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

/// Evaluate every dimension in a unified [`Constitution`] against the workspace at
/// `manifest_path`, returning one inspectable reaction without CLI presentation.
///
/// This is the library counterpart to [`run`]: it observes static boundaries, the full semantic
/// bundle, and runtime probe coverage through the same composition path the CLI uses. The manifest
/// path is explicit; this function performs cargo-metadata and source-file observation, but does not
/// parse arguments, discover a manifest from the current directory, print output, apply or write a
/// baseline, or emit coverage advisories. Use [`run`] for those gate and presentation concerns.
pub fn check_constitution(constitution: &Constitution, manifest_path: &Path) -> Outcome {
    evaluate_constitution(constitution, manifest_path).0
}

/// The one composition seam beneath the library check and CLI runner. Coverage remains static-only
/// and is returned separately for CLI advisory presentation; it never changes the reaction.
fn evaluate_constitution(
    constitution: &Constitution,
    manifest_path: &Path,
) -> (Outcome, Option<Coverage>) {
    // One `cargo metadata` read feeds both the static reaction outcome and coverage; the semantic
    // dimension reads its own (it has no coverage notion). A constitution error from any dimension
    // supersedes the accumulated verdict, and otherwise violations merge into one report.
    let (static_outcome, observed_coverage) =
        check_and_cover(constitution.static_boundaries(), manifest_path);
    let mut outcome = static_outcome;
    if !matches!(outcome, Outcome::ConstitutionError(_))
        && !constitution.semantic_boundaries().is_empty()
    {
        outcome = merge_outcomes(
            outcome,
            hunyi::check_all(constitution.semantic_boundaries(), manifest_path),
        );
    }

    // Audit even an empty runtime declaration: an orphan `assert_boundary!` probe must react.
    // Once an earlier dimension errors the verdict is untrustworthy, so evaluation stops.
    if !matches!(outcome, Outcome::ConstitutionError(_)) {
        match cargo_metadata(manifest_path) {
            Ok(metadata) => {
                let roots = member_root_files(&metadata);
                outcome = merge_outcomes(
                    outcome,
                    audit_probe_coverage(constitution.runtime_boundaries(), &roots),
                );
            }
            Err(message) => {
                outcome = merge_outcomes(
                    outcome,
                    Outcome::ConstitutionError(format!(
                        "cannot read workspace '{}': {message}",
                        manifest_path.display()
                    )),
                );
            }
        }
    }

    (outcome, observed_coverage)
}

/// The command-line flags `dispatch` parses, before command-specific dispatch reacts to them.
/// `format` is left as the full [`Format`] (not yet narrowed to [`ReportFormat`]) since `list`
/// and `check` accept different subsets of it.
struct ParsedArgs {
    command: Command,
    manifest_path: Option<String>,
    baseline_path: Option<String>,
    write_baseline_path: Option<String>,
    format: Format,
    warn_uncovered: bool,
    disallow_stale: bool,
}

/// Parse `dispatch`'s process arguments into [`ParsedArgs`], or `Err(exit code)` on a usage
/// error (an absent flag value, an unrecognized argument, or an unknown `--format`) — a
/// misconfiguration fails loud (exit 2), never a silent downgrade to a default (PROJECT.md).
fn parse_args<I, S>(args: I) -> Result<ParsedArgs, u8>
where
    I: IntoIterator<Item = S>,
    S: Into<String>,
{
    let mut manifest_path: Option<String> = None;
    let mut baseline_path: Option<String> = None;
    let mut write_baseline_path: Option<String> = None;
    let mut format: Option<String> = None;
    let mut warn_uncovered = false;
    let mut disallow_stale = false;
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
                None => return Err(usage(concat!($flag, " requires a value"))),
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
            "--disallow-stale" => disallow_stale = true,
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
                    return Err(usage(&format!("unrecognized argument '{other}'")));
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
            return Err(usage(&format!(
                "unknown --format '{other}' (expected text, json, markdown, or sarif)"
            )));
        }
    };

    Ok(ParsedArgs {
        command,
        manifest_path,
        baseline_path,
        write_baseline_path,
        format,
        warn_uncovered,
        disallow_stale,
    })
}

/// The `list` command's whole reaction: a projection, not a reaction — it observes nothing (no
/// `--manifest-path`), cannot fail a boundary, and always exits 0. It accepts only `--format`; a
/// check-only flag supplied to `list` is a usage error, not a silent no-op (PROJECT.md: never
/// silently ignore a flag).
fn dispatch_list(constitution: &Constitution, parsed: &ParsedArgs) -> u8 {
    if parsed.manifest_path.is_some()
        || parsed.baseline_path.is_some()
        || parsed.write_baseline_path.is_some()
        || parsed.warn_uncovered
        || parsed.disallow_stale
    {
        return usage("list takes only --format; other flags are check-only");
    }
    let semantic = constitution.semantic_boundaries();
    let runtime = constitution.runtime_boundaries();
    match parsed.format {
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
    EXIT_OK
}

/// Resolve `check`'s target manifest: the given `--manifest-path`, or the nearest `Cargo.toml`
/// up from the current directory, cargo-style. Defaulting the target location is not a silent
/// pass: if none is found this is a scan error (exit 2), never 0.
fn resolve_manifest_path(manifest_path: Option<String>) -> Result<PathBuf, u8> {
    match manifest_path {
        Some(path) => Ok(PathBuf::from(path)),
        None => match nearest_manifest() {
            Some(path) => Ok(path),
            None => {
                let from = std::env::current_dir()
                    .map(|dir| dir.display().to_string())
                    .unwrap_or_else(|_| "the current directory".to_string());
                eprintln!(
                    "Tianheng: no Cargo.toml found from {from} up to the root; \
                     pass --manifest-path <path>"
                );
                Err(EXIT_CANNOT_JUDGE)
            }
        },
    }
}

/// Print `check`'s final report in the requested format — the tail every non-baseline,
/// non-write-baseline `check` run reaches. Never affects the exit code (the caller computes
/// that from `outcome` itself).
fn print_report(
    report_format: ReportFormat,
    outcome: &Outcome,
    coverage: Option<&Coverage>,
    warn_uncovered: bool,
) {
    match report_format {
        ReportFormat::Json => println!("{}", report_json(outcome, &[], coverage)),
        ReportFormat::Sarif => println!("{}", report_sarif(outcome)),
        ReportFormat::Text => {
            report(outcome);
            if let Some(coverage) = coverage {
                report_coverage(coverage, warn_uncovered);
            }
        }
    }
}

/// The runner's work, returning the exit code as a number so it is assertable
/// without a subprocess and without inspecting an opaque [`ExitCode`].
fn dispatch<I, S>(constitution: &Constitution, args: I) -> u8
where
    I: IntoIterator<Item = S>,
    S: Into<String>,
{
    let parsed = match parse_args(args) {
        Ok(parsed) => parsed,
        Err(code) => return code,
    };

    if parsed.command == Command::List {
        return dispatch_list(constitution, &parsed);
    }

    // The command is `check`. `markdown` is a `list`-only projection of the declared law;
    // `check`'s machine output is the JSON report, so reject it loud (exit 2) rather than
    // silently falling back. `text`/`json` map to the existing boolean contract.
    let report_format = match parsed.format {
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
    if parsed.baseline_path.is_some() && parsed.write_baseline_path.is_some() {
        return usage("--baseline and --write-baseline are mutually exclusive");
    }
    if parsed.disallow_stale && parsed.baseline_path.is_none() {
        return usage("--disallow-stale requires --baseline");
    }

    let manifest_path = match resolve_manifest_path(parsed.manifest_path) {
        Ok(path) => path,
        Err(code) => return code,
    };

    let (mut outcome, observed_coverage) = evaluate_constitution(constitution, &manifest_path);

    if let Some(path) = parsed.write_baseline_path {
        return write_baseline(&outcome, &path);
    }

    // Coverage is an observation, not a reaction: surfaced only when the constitution
    // was successfully evaluated, omitted on a constitution error (where the error is
    // the story), and never affecting the exit code.
    let coverage = match outcome {
        Outcome::ConstitutionError(_) => None,
        _ => observed_coverage,
    };

    if let Some(path) = parsed.baseline_path {
        return gate(
            &mut outcome,
            &path,
            report_format,
            coverage.as_ref(),
            parsed.warn_uncovered,
            parsed.disallow_stale,
        );
    }

    print_report(
        report_format,
        &outcome,
        coverage.as_ref(),
        parsed.warn_uncovered,
    );
    outcome.exit_code()
}

/// Print usage to stderr and return exit 2 — a usage mistake is not architectural
/// drift.
fn usage(message: &str) -> u8 {
    eprintln!(
        "usage:\n  \
         tianheng check --manifest-path <path/to/Cargo.toml> \
         [--baseline <file> | --write-baseline <file>] [--format text|json|sarif] \
         [--warn-uncovered] [--disallow-stale]\n  \
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
    // Metadata-preserving merge applies only to a supported semantic baseline. Unsupported or
    // unreadable content is preserved byte-for-byte: presentation cannot reconstruct identity, and
    // overwriting would silently destroy annotations the adopter may still need to carry manually.
    let (baseline, create_new) = match std::fs::read_to_string(path) {
        Ok(text) => match Baseline::from_json(&text) {
            Ok(existing) => (Baseline::of_preserving(report, &existing), false),
            Err(err) => {
                eprintln!(
                    "Tianheng: refusing to overwrite unsupported baseline {path} ({err}). Preserve \
                     any desired owner/tracker annotations, move or delete the unsupported file, \
                     then run `tianheng check --write-baseline {path}` again."
                );
                return EXIT_CANNOT_JUDGE;
            }
        },
        Err(err) if err.kind() == std::io::ErrorKind::NotFound => (Baseline::of(report), true),
        Err(err) => {
            eprintln!(
                "Tianheng: refusing to overwrite unreadable baseline {path} ({err}). Preserve any \
                 desired owner/tracker annotations, move or delete the unsupported file, then run \
                 `tianheng check --write-baseline {path}` again."
            );
            return EXIT_CANNOT_JUDGE;
        }
    };
    let document = baseline.to_json();
    let write_result = if create_new {
        create_baseline_file(path, &document)
    } else {
        write_baseline_atomically(path, &document)
    };
    match write_result {
        Ok(()) => {
            eprintln!(
                "Tianheng: wrote {} violation(s) to baseline {path}",
                report.violations.len()
            );
            EXIT_OK
        }
        Err(err) if create_new && err.kind() == std::io::ErrorKind::AlreadyExists => {
            eprintln!(
                "Tianheng: refusing to overwrite baseline {path} because it appeared while the \
                 new snapshot was being prepared. Inspect the file, then rerun the command."
            );
            EXIT_CANNOT_JUDGE
        }
        Err(err) => {
            eprintln!("Tianheng: cannot write baseline {path}: {err}");
            EXIT_CANNOT_JUDGE
        }
    }
}

fn create_baseline_file(path: &str, document: &str) -> std::io::Result<()> {
    OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(path)
        .and_then(|mut file| file.write_all(document.as_bytes()))
}

/// Overwrites an existing baseline durably: the merged document lands at a sibling temp path
/// first, then an atomic `rename` swaps it into place. A crash, interrupt, or full disk mid-write
/// leaves the previous baseline — and the owner/tracker annotations `Baseline::of_preserving` just
/// merged in — fully intact rather than truncated. `create_baseline_file` above has no pre-existing
/// content to protect (a crash there simply leaves no file, and the adopter reruns the command), so
/// only this, the overwrite path, needs the guarantee.
///
/// The swap targets the file's symlink-resolved real path and carries over its existing
/// permissions: `rename` unconditionally replaces whatever sits at its destination, so renaming
/// onto `path` directly would silently replace a symlinked baseline with a plain file (orphaning
/// whatever the symlink pointed at) and reset the mode to the temp file's process-umask default,
/// silently widening permissions an adopter deliberately narrowed — both measured regressions
/// against this function's own "leaves the previous baseline fully intact" claim. The temp path is
/// built from the resolved target, not the original `path`, so it always shares the target's
/// directory (and therefore its filesystem, keeping the rename atomic) even when `path` is a
/// symlink elsewhere. A write or permissions failure best-effort cleans up the temp file rather
/// than leaving it stranded.
fn write_baseline_atomically(path: &str, document: &str) -> std::io::Result<()> {
    let target = std::fs::canonicalize(path)?;
    let permissions = std::fs::metadata(&target)?.permissions();
    let tmp_path = format!("{}.tmp-{}", target.display(), std::process::id());
    let write_result = std::fs::write(&tmp_path, document)
        .and_then(|()| std::fs::set_permissions(&tmp_path, permissions))
        .and_then(|()| std::fs::rename(&tmp_path, &target));
    if write_result.is_err() {
        let _ = std::fs::remove_file(&tmp_path);
    }
    write_result
}

/// Gate against a baseline: suppress recorded violations, fail only on new ones,
/// and report stale baseline entries. An unreadable baseline is a scan error.
fn gate(
    outcome: &mut Outcome,
    path: &str,
    format: ReportFormat,
    coverage: Option<&Coverage>,
    warn_uncovered: bool,
    disallow_stale: bool,
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
    let stale: Vec<BaselineEntry> = baseline.stale(report).into_iter().cloned().collect();
    let policy = stale_policy(outcome, &stale, disallow_stale);

    match format {
        ReportFormat::Json => println!(
            "{}",
            report_json_with_stale_policy(outcome, &stale, coverage, disallow_stale)
        ),
        ReportFormat::Sarif => println!(
            "{}",
            report_sarif_with_stale(outcome, &stale, disallow_stale)
        ),
        ReportFormat::Text => {
            report_violations(report);
            for entry in &stale {
                eprintln!(
                    "Tianheng: stale baseline entry (no longer violated): {} / {} / {}",
                    entry.id.target(),
                    entry.rule,
                    entry.finding
                );
            }
            if policy.stale_disallowed {
                eprintln!("Tianheng: {}", disallow_stale_message(stale.len()));
            }
            if let Some(coverage) = coverage {
                report_coverage(coverage, warn_uncovered);
            }
        }
    }
    policy.exit_code
}

/// Fold two outcomes into one reaction. Reused across the composition chain — static + semantic,
/// then the accumulated outcome + the runtime probe-coverage audit, then + a workspace-source
/// constitution error. A constitution error from either side supersedes any violation — a boundary
/// that could not be evaluated makes the run's verdict untrustworthy — and otherwise the two reports'
/// violations merge into a single report, gated, baselined, and reported together. `first` is checked
/// first, so its error wins deterministically when both error.
fn merge_outcomes(first: Outcome, second: Outcome) -> Outcome {
    if matches!(first, Outcome::ConstitutionError(_)) {
        return first;
    }
    if matches!(second, Outcome::ConstitutionError(_)) {
        return second;
    }
    let mut violations = Vec::new();
    if let Outcome::Violations(report) = &first {
        violations.extend(report.violations.iter().cloned());
    }
    if let Outcome::Violations(report) = &second {
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
