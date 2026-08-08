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
use hunyi::{Observer, SemanticObserver};
use louke::RuntimeObserver;

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

/// One governance run, assembled observer by observer.
///
/// The fold is **eager**: each [`observe`](Run::observe) call folds that observer's outcome into the accumulator
/// immediately, so the heterogeneous set never exists as a collection and no trait object appears anywhere. An
/// earlier design held `&[&dyn Observer]` and would have needed that exposure governed; measured, it could not
/// be — no module of this crate is governed by a semantic boundary, and the `dyn`-trait DSL offers only
/// forbid-all and forbid-named-operands, so the declaration would have been a name with no reaction.
///
/// Assembly order is **semantically observable**, deterministically: it decides which cannot-judge is reported
/// when more than one observer cannot judge. That was a property of a hand-written call sequence nobody had to
/// think about; the moment the order is a caller's to choose, it is part of the contract.
///
/// ```no_run
/// use std::path::Path;
/// use tianheng::prelude::*;
///
/// # fn demo(constitution: &Constitution, manifest: &Path) -> Outcome {
/// Run::over(manifest)
///     .observe(StaticObserver::new(constitution.static_boundaries().clone()))
///     .observe(SemanticObserver::new(constitution.semantic_boundaries().clone()))
///     .verdict()
/// # }
/// ```
#[derive(Debug, Clone)]
pub struct Run<'a> {
    manifest_path: &'a Path,
    accumulated: Option<Outcome>,
}

impl<'a> Run<'a> {
    /// Begin a run over the workspace whose manifest is at `manifest_path`.
    pub fn over(manifest_path: &'a Path) -> Self {
        Self {
            manifest_path,
            accumulated: None,
        }
    }

    /// Compose one observer, folding its outcome in immediately.
    ///
    /// If the accumulator already cannot judge, the observer is **not evaluated at all**: a verdict resting on a
    /// boundary that could not be evaluated is not a verdict, so further work would be spent on an answer that
    /// cannot be reported. This is `evaluate_constitution`'s present behaviour expressed as a property of the
    /// builder rather than an `if` before each dimension.
    pub fn observe(mut self, observer: impl Observer) -> Self {
        if matches!(self.accumulated, Some(Outcome::ConstitutionError(_))) {
            return self;
        }
        let next = observer.observe(self.manifest_path);
        self.accumulated = Some(match self.accumulated.take() {
            Some(previous) => merge_outcomes(previous, next),
            None => next,
        });
        self
    }

    /// The composed verdict.
    ///
    /// A run that composed **no** observer cannot judge: reporting it clean would be a vacuous pass, which is
    /// the direction this repository has re-opened most often. It is a misconfiguration, not a clean workspace.
    ///
    /// Not the same question as a composed observer that declares **nothing** — [`hunyi::check_all`] answers an
    /// empty boundary bundle with [`Outcome::Clean`], and a static-only adoption is exactly that shape. There a
    /// participant was composed and has nothing to observe; here nothing was composed, so no participant's
    /// silence could be read as cleanliness. `observer-protocol` states why unifying the two fails in both
    /// directions.
    pub fn verdict(self) -> Outcome {
        self.accumulated.unwrap_or_else(|| {
            Outcome::ConstitutionError(
                "a run composed no observer, so there is nothing to judge; composing nothing is a \
                 misconfiguration rather than a clean workspace"
                    .to_string(),
            )
        })
    }
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
    if !matches!(outcome, Outcome::ConstitutionError(_)) {
        // **Delegated, not restated**, the route the runtime arm below already takes. This arm called 渾儀's
        // composed entry point directly, which is what `SemanticObserver::observe` does — so the two paths
        // agreed only while nobody put a decision in between them, and `observer-protocol` declared that gap
        // as a bound after a text reader over this body was defeated at every level it could be narrowed to.
        // This path now IS the observer for this dimension, so there is no second site to disagree with; the
        // cost is one clone of the declared bundle per run, paid deliberately.
        outcome = merge_outcomes(
            outcome,
            SemanticObserver::new(constitution.semantic_boundaries().clone())
                .observe(manifest_path),
        );
    }

    // Audit even an empty runtime declaration: an orphan `assert_boundary!` probe must react.
    // Once an earlier dimension errors the verdict is untrustworthy, so evaluation stops.
    if !matches!(outcome, Outcome::ConstitutionError(_)) {
        // **Delegated, not restated.** This arm held its own copy of 漏刻's three statements — the corpus and
        // anchor derivation, the audit call, and the `cannot read workspace` message — so equality between
        // this path and the protocol's for the runtime dimension depended on nobody editing one of the two
        // copies. That is the exact drift `observer-protocol` exists to end, and it was sitting inside the
        // thing being compared. This path now IS the observer for this dimension, which cannot drift; the
        // cost is one `to_vec` of the declared seams per run, paid deliberately.
        //
        // Consequence the spec states: for the runtime dimension the two paths agree **by construction**
        // rather than by observation. What still bites is the equality reaction's per-dimension assertion
        // that the fixture's runtime boundary actually reacted.
        outcome = merge_outcomes(
            outcome,
            RuntimeObserver::new(constitution.runtime_boundaries().to_vec()).observe(manifest_path),
        );
    }

    (outcome, observed_coverage)
}

/// The command-line flags `dispatch` parses, before command-specific dispatch reacts to them.
/// `format` is left as the full [`Format`] (not yet narrowed to [`ReportFormat`]) since `list`
/// and `check` accept different subsets of it, and stays an `Option` so a dispatch can tell an
/// explicitly requested format from the default: a flag that cannot apply to the requested action
/// must be rejected rather than silently ignored, which requires knowing it was supplied at all.
struct ParsedArgs {
    command: Command,
    manifest_path: Option<String>,
    baseline_path: Option<String>,
    write_baseline_path: Option<String>,
    format: Option<Format>,
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
    //
    // A value that is itself a `--`-prefixed token is the same missing value, one token later:
    // the flag the user meant to pass gets eaten as this flag's value. Taking it would drop a
    // real flag with no diagnostic, and for `--write-baseline` it reaches a silent SUCCESS —
    // writing a baseline file literally named `--warn-uncovered` and exiting 0, the one shape of
    // this mistake that does not even land on a non-zero exit. So reject it here and name the
    // token found, rather than let a downstream scan error misreport it as a bad path. The
    // `--flag=<value>` form stays the escape hatch for a value that must begin with `--`; it
    // carries its value in the same token, so no following flag can be consumed by mistake.
    macro_rules! value {
        ($flag:literal) => {
            match args.next() {
                Some(value) if !value.starts_with("--") => require_non_empty($flag, value)?,
                Some(found) => {
                    return Err(usage(&format!(
                        "{} requires a value, but the next argument is the flag '{found}'; \
                         use {}=<value> for a value that begins with '--'",
                        $flag, $flag
                    )));
                }
                None => return Err(usage(concat!($flag, " requires a value"))),
            }
        };
    }
    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--manifest-path" => {
                take_once(
                    &mut manifest_path,
                    "--manifest-path",
                    value!("--manifest-path"),
                )?;
            }
            "--baseline" => take_once(&mut baseline_path, "--baseline", value!("--baseline"))?,
            "--write-baseline" => take_once(
                &mut write_baseline_path,
                "--write-baseline",
                value!("--write-baseline"),
            )?,
            "--format" => take_once(&mut format, "--format", value!("--format"))?,
            "--warn-uncovered" => warn_uncovered = true,
            "--disallow-stale" => disallow_stale = true,
            other => {
                // The equals form deliberately does NOT reject a `--`-prefixed value — carrying the
                // value in the same token is exactly what makes it the escape hatch for one — but it
                // shares the non-empty rule, so `--flag=` is the usage error it is, and the
                // once-only rule, so the two forms cannot be combined to smuggle a second value past
                // it (`--baseline a --baseline=b`).
                if let Some(path) = other.strip_prefix("--manifest-path=") {
                    let path = require_non_empty("--manifest-path", path.to_string())?;
                    take_once(&mut manifest_path, "--manifest-path", path)?;
                } else if let Some(path) = other.strip_prefix("--baseline=") {
                    let path = require_non_empty("--baseline", path.to_string())?;
                    take_once(&mut baseline_path, "--baseline", path)?;
                } else if let Some(path) = other.strip_prefix("--write-baseline=") {
                    let path = require_non_empty("--write-baseline", path.to_string())?;
                    take_once(&mut write_baseline_path, "--write-baseline", path)?;
                } else if let Some(value) = other.strip_prefix("--format=") {
                    let value = require_non_empty("--format", value.to_string())?;
                    take_once(&mut format, "--format", value)?;
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
    // is recognized here but only honored by `list` (rejected for `check` below). The `None`
    // (flag absent) case stays `None` rather than collapsing to `Text` here: each dispatch
    // defaults it at the point of use, so it can still distinguish "text was asked for" from
    // "nothing was asked for" and reject the former where no report is produced at all.
    let format = match format.as_deref() {
        None => None,
        Some("text") => Some(Format::Text),
        Some("json") => Some(Format::Json),
        Some("markdown") => Some(Format::Markdown),
        Some("sarif") => Some(Format::Sarif),
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
    // The flags SUPPLIED are named, not merely the fact that some inapplicable flag was present. This was a single
    // sentence naming none of them, which satisfied this command's own requirement while the requirement covering
    // the same conflict inside `check` — one that cites this rule as the one it extends — requires the flag to be
    // named. The two disagreed and each implementation matched its own, so no test caught it.
    //
    // It matters most for `--manifest-path`, the flag a user types by habit: told only that "list takes only
    // --format", a reader who passed both `--manifest-path` and `--format` is being shown the flag they got right.
    //
    // Ordered by this check rather than by the command line, so the message is a function of the set supplied and
    // not of how it was typed — which is what makes it assertable.
    let mut inapplicable = Vec::new();
    for (flag, supplied) in [
        ("--manifest-path", parsed.manifest_path.is_some()),
        ("--baseline", parsed.baseline_path.is_some()),
        ("--write-baseline", parsed.write_baseline_path.is_some()),
        ("--warn-uncovered", parsed.warn_uncovered),
        ("--disallow-stale", parsed.disallow_stale),
    ] {
        if supplied {
            inapplicable.push(flag);
        }
    }
    if !inapplicable.is_empty() {
        return usage(&format!(
            "list takes only --format; {} {} check-only",
            inapplicable.join(", "),
            if inapplicable.len() == 1 { "is" } else { "are" }
        ));
    }
    let semantic = constitution.semantic_boundaries();
    let runtime = constitution.runtime_boundaries();
    // `list` honors every format it supports, so an absent `--format` simply defaults to `text`
    // here; unlike `check`'s write action, there is no `list` action a requested format cannot
    // apply to.
    match parsed.format.unwrap_or(Format::Text) {
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
    // silently falling back. `text`/`json` map to the existing boolean contract. An absent
    // `--format` defaults here, at the point of use, so the write-baseline check below still sees
    // whether one was requested at all.
    let report_format = match parsed.format.unwrap_or(Format::Text) {
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
    // `--write-baseline` records a snapshot; it emits no report at all, so a flag whose only effect
    // is on a report has nothing to act on here. `list` already rejects a check-only flag rather
    // than accepting it as a silent no-op, and `--disallow-stale` without `--baseline` is rejected
    // just above for the same reason — this is that same rule applied WITHIN `check`, between its
    // two actions, which was the one place it did not hold: `check --write-baseline out.json
    // --warn-uncovered --format sarif` recorded the baseline, exited 0, and dropped both flags with
    // no diagnostic, so an adopter could believe they had coverage advisories or a SARIF document
    // and receive neither.
    //
    // The line drawn here is "the action produces nothing this flag could affect", not "this flag
    // changes nothing observable". `--warn-uncovered` under `--format json` stays accepted: the JSON
    // report's `coverage` object already carries every uncovered crate unconditionally, so the flag
    // is redundant there rather than dropped — the consumer receives the whole fact either way.
    if parsed.write_baseline_path.is_some() {
        if parsed.warn_uncovered {
            return usage(
                "--warn-uncovered cannot apply to --write-baseline: recording a baseline emits \
                 no coverage report to raise an advisory in",
            );
        }
        if parsed.format.is_some() {
            return usage(
                "--format cannot apply to --write-baseline: recording a baseline emits no report \
                 to format (the baseline document's own shape is fixed)",
            );
        }
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

/// The one rule both flag forms share: a value must not be empty. `--flag=` and `--flag ""` are the
/// same mistake as `--flag` with nothing after it — a flag given no value — so all three are one
/// usage error (exit 2) rather than an empty string carried onward. An empty path reaches the
/// filesystem as `""` and answers `NotFound`, which reads as "cannot read baseline " against a path
/// nobody typed: the malformed invocation misreported as a missing file, the same misdirection the
/// flag-shaped-value rule exists to prevent one shape earlier. Shared by the space and equals forms
/// so the two cannot diverge on what counts as a value.
fn require_non_empty(flag: &str, value: String) -> Result<String, u8> {
    if value.is_empty() {
        return Err(usage(&format!(
            "{flag} requires a value, but was given an empty one"
        )));
    }
    Ok(value)
}

/// The second rule both flag forms share: a value-taking flag is given its value **once**. A
/// second occurrence used to overwrite the first silently, so `--baseline a --baseline b` gated
/// against `b` with no word about `a` — the invocation named two files and the runner acted on one,
/// which is the same "a flag the invocation supplied was dropped without a diagnostic" mistake the
/// flag-shaped-value rule exists to prevent, one token further out. Which value a repeat should win
/// is not knowable from the invocation, so neither is chosen: it is a usage error (exit 2) naming
/// the flag. Shared by the space and equals forms, so the two cannot be combined to smuggle a
/// second value past it.
///
/// Deliberately scoped to the value-taking flags. Repeating a boolean (`--warn-uncovered
/// --warn-uncovered`) drops nothing — the second occurrence asks for exactly what the first already
/// set — so there is no ambiguity to report, and rejecting it would be a style rule rather than a
/// misconfiguration reaction (PROJECT.md's minimalism bound: fail loud on *observable*
/// misconfiguration, not on redundancy).
fn take_once<T>(slot: &mut Option<T>, flag: &str, value: T) -> Result<(), u8> {
    if slot.is_some() {
        return Err(usage(&format!(
            "{flag} was given more than once; it takes a single value, and which of the given \
             values was meant cannot be inferred"
        )));
    }
    *slot = Some(value);
    Ok(())
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
        // A zero-length target is the one "unsupported" shape that provably holds nothing worth
        // protecting. The refusal below exists to stop an overwrite from destroying hand-authored
        // owner/tracker annotations, which cannot be reconstructed from a rerun — and zero bytes
        // cannot hold any. Refusing it therefore protects nothing while costing the adopter a manual
        // file move, and its own guidance ("preserve any desired annotations") names something that
        // is not there.
        //
        // It is also the exact shape an interrupted create leaves: `create_baseline_file` publishes
        // its directory entry before its first byte, so a crash mid-create leaves an empty file. The
        // write action's job is to record, so it records — and says that it did, because recovering
        // in silence is the other extreme. Bounded to *zero* length deliberately: whitespace, a
        // truncated `{"format":`, or any other partial content might have held annotations before it
        // was damaged, so those stay refused.
        //
        // `create_new` is false: the file exists, so this takes the overwrite path, which preserves
        // its mode and swaps atomically. Gate mode (`--baseline`) deliberately does NOT share this
        // tolerance — see `gate`, where an unreadable baseline stays exit 2. Recording may safely
        // regenerate what it owns; gating consumes a declaration the adopter wrote, and a corrupt
        // one must be reported rather than read as "nothing is baselined".
        Ok(text) if text.is_empty() => {
            eprintln!(
                "Tianheng: baseline {path} was empty, so there were no owner/tracker annotations to \
                 preserve (an interrupted write leaves exactly this); recording a fresh snapshot."
            );
            (Baseline::of(report), false)
        }
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
        Err(BaselineWriteError::TempPathCollision(tmp_path)) => {
            eprintln!(
                "Tianheng: refusing to overwrite baseline {path}: its temp file {} already \
                 exists, likely stranded by an interrupted run. Inspect and remove it, then \
                 rerun the command.",
                tmp_path.display()
            );
            EXIT_CANNOT_JUDGE
        }
        Err(BaselineWriteError::DanglingSymlink { target }) => {
            eprintln!(
                "Tianheng: refusing to write baseline {path}: it is a symlink to {}, which does \
                 not exist. Recreate the target or remove the dangling link, then rerun the \
                 command.",
                target.display()
            );
            EXIT_CANNOT_JUDGE
        }
        Err(BaselineWriteError::Io(err))
            if create_new && err.kind() == std::io::ErrorKind::AlreadyExists =>
        {
            eprintln!(
                "Tianheng: refusing to overwrite baseline {path} because it appeared while the \
                 new snapshot was being prepared. Inspect the file, then rerun the command."
            );
            EXIT_CANNOT_JUDGE
        }
        Err(BaselineWriteError::Io(err)) => {
            eprintln!("Tianheng: cannot write baseline {path}: {err}");
            EXIT_CANNOT_JUDGE
        }
    }
}

/// Best-effort flush of the directory that holds `path`, so a directory entry a just-completed
/// create or rename added is itself durable. An fsync on a *file* persists its contents; it does
/// not persist the name through which anything reaches them, which lives in the parent directory's
/// own data. Both baseline write paths call this after the step that publishes the entry.
///
/// Deliberately infallible, and this is the boundary between the two halves of the durability
/// guarantee. Flushing the written *file* is strict — it is what prevents the empty-baseline loss,
/// and fsync on a regular file just written is universally supported, so propagating its error
/// costs nothing. Flushing the *directory* only strengthens a write that has already landed, and
/// the ways it can be impossible are capability limits rather than storage faults: some FUSE and
/// network mounts answer `EINVAL`/`ENOSYS` to fsync on a directory handle, and a directory that is
/// writable but not readable (mode `0300`) cannot be opened for it at all. Turning any of those
/// into "cannot write baseline" would report failure for a baseline that is sitting correctly on
/// disk, and would regress adopters for whom this path worked before the flush existed. So a
/// runtime inability to flush a directory is treated exactly as the compile-time one is below: as
/// this platform not offering the operation, not as the write having failed.
///
/// Unix only for the compile-time half: `File::open` on a directory is not portable — Windows
/// requires `FILE_FLAG_BACKUP_SEMANTICS`, which `std` does not expose through `OpenOptions`. The
/// `File` import is scoped to the block for that reason, so it is not an unused import off unix.
/// A path with no parent component (a bare relative filename) resolves to the working directory,
/// which is where such a write actually lands.
fn sync_parent_dir(path: &Path) {
    #[cfg(unix)]
    {
        use std::fs::File;

        let parent = path
            .parent()
            .filter(|parent| !parent.as_os_str().is_empty())
            .unwrap_or_else(|| Path::new("."));
        let _ = File::open(parent).and_then(|dir| dir.sync_all());
    }
    #[cfg(not(unix))]
    let _ = path;
}

/// `create_new`'s `O_EXCL` fails on any existing directory entry, including a dangling symlink —
/// a baseline path whose symlink target does not exist reads as `NotFound` one line up (so this,
/// the create-new path, runs), then hits `AlreadyExists` here, indistinguishable from a genuine
/// concurrent creation without checking `symlink_metadata` explicitly. That distinction matters:
/// a dangling symlink is a permanent state, not a race — "rerun the command" (the concurrent-
/// creation arm's own remedy) would fail identically forever, so this earns its own diagnosis.
///
/// The written bytes are fsynced before this reports success, and the new directory entry is flushed
/// best-effort after it ([`sync_parent_dir`]), so a baseline this process said it wrote survives a
/// crash. Unlike the overwrite path, this one
/// writes in place rather than through a temp file, so it protects a *reported success* and nothing
/// more: there is no previous content to preserve, and a crash **mid-write** can leave a
/// zero-length or partial file — `create_new` publishes the entry before any byte is written, and
/// no ordering of fsyncs changes that. What the next run does with that residue depends on which of
/// the two it is, and the difference is [`write_baseline`]'s zero-length exception: a **zero-length**
/// file is recorded afresh (exit 0, announced on stderr) — zero bytes cannot hold the owner/tracker
/// annotations the refusal exists to protect, and this path is the most likely way one appears — while
/// a **partial** file is still refused as an unsupported baseline (exit 2, naming the remedy: move or
/// delete it and rerun), because it may have held annotations before being damaged and no rerun can
/// tell. So the common residue needs no manual step and the ambiguous one stays loud. Making this path
/// atomic too would mean temp-then-rename here as
/// well, at the cost of the `AlreadyExists`/dangling-symlink distinction above, which depends on
/// `create_new` landing on the real path — a deliberate trade, not an oversight.
fn create_baseline_file(path: &str, document: &str) -> Result<(), BaselineWriteError> {
    let write_result = OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(path)
        .and_then(|mut file| {
            file.write_all(document.as_bytes())
                .and_then(|()| file.sync_all())
        });
    let Err(err) = write_result else {
        sync_parent_dir(Path::new(path));
        return Ok(());
    };
    if err.kind() == std::io::ErrorKind::AlreadyExists {
        if let Ok(metadata) = std::fs::symlink_metadata(path) {
            // Dangling is claimed only when the target is genuinely **absent** — `NotFound`
            // specifically, not any metadata failure. `std::fs::metadata` follows the link, so it also
            // fails when the target EXISTS but cannot be reached: `EACCES` on a component of its path,
            // or `ELOOP`. Treating those as dangling repeats the defect this branch was narrowed to fix,
            // one error kind further in — "it is a symlink to X, which does not exist" about a target
            // that does, prescribing a remedy ("recreate the target") that cannot help. Measured: with
            // the target inside a `chmod 000` directory, `lstat` reports a symlink, the `O_EXCL` open
            // fails `EEXIST`, and `metadata` fails `EACCES`.
            //
            // This function is reached only when `read_to_string` returned `NotFound`, so for a symlink
            // the target was absent when the path was read — but it can come back before the `O_EXCL`
            // open (restored file, or the link replaced), and classifying on symlink-ness alone then
            // told the adopter "it is a symlink to X, which does not exist" about a target that does,
            // and prescribed a remedy ("recreate the target") already satisfied. Refusing was always
            // safe; only the reason was false, which is the misdiagnosis class this window corrected
            // twice elsewhere.
            //
            // Falling through loses no diagnostic: [`write_baseline`]'s `create_new && AlreadyExists`
            // arm already reports that the baseline "appeared while the new snapshot was being
            // prepared", which is exactly what happened.
            if metadata.file_type().is_symlink()
                && matches!(std::fs::metadata(path), Err(err)
                    if err.kind() == std::io::ErrorKind::NotFound)
            {
                let target = std::fs::read_link(path)?;
                return Err(BaselineWriteError::DanglingSymlink { target });
            }
        }
    }
    Err(err.into())
}

/// Why [`write_baseline_atomically`] or [`create_baseline_file`] failed. A bare `io::Error` cannot
/// distinguish a stale temp file left over from an interrupted run (a real, reachable case — a
/// killed process, or a pid reused across a fresh container), or a dangling symlink (a baseline
/// path whose target was deleted), from any other IO failure — and reporting either against the
/// baseline path with a generic message leaves the adopter nothing to act on.
#[derive(Debug)]
enum BaselineWriteError {
    /// The temp file's `create_new` open hit something already at that path. Carries the temp
    /// path itself so the caller can name the file that is actually blocking the write.
    TempPathCollision(PathBuf),
    /// The baseline path is a symlink whose target does not exist. Carries the target so the
    /// caller can say what it (no longer) points at.
    DanglingSymlink {
        target: PathBuf,
    },
    Io(std::io::Error),
}

impl From<std::io::Error> for BaselineWriteError {
    fn from(err: std::io::Error) -> Self {
        Self::Io(err)
    }
}

/// Removes a temp file when it goes out of scope unless the write committed — the cleanup half of
/// [`write_baseline_atomically`]'s guarantee, whose doc carries the full threat model.
///
/// A `bool` rather than an `Option<PathBuf>`: with the path always present, `path()` is infallible.
/// Under an `Option`, `commit` consumed the guard and took the path, so no caller could ever observe
/// `None` — and the `expect` standing over that unreachable state is the defensive
/// over-foolproofing of an impossible state the minimalism bound forbids.
struct AtomicTempFileGuard {
    path: PathBuf,
    committed: bool,
}

impl AtomicTempFileGuard {
    fn new(path: PathBuf) -> Self {
        Self {
            path,
            committed: false,
        }
    }

    fn path(&self) -> &Path {
        &self.path
    }

    /// Consumes the guard, so a committed write cannot be followed by a use of the temp path that no
    /// longer exists.
    fn commit(mut self) {
        self.committed = true;
    }
}

impl Drop for AtomicTempFileGuard {
    fn drop(&mut self) {
        if !self.committed {
            let _ = std::fs::remove_file(&self.path);
        }
    }
}

/// Overwrites an existing baseline durably: the merged document lands at a sibling temp path
/// first, is fsynced there, and only then does an atomic `rename` swap it into place. A crash,
/// interrupt, or full disk mid-write leaves the previous baseline — and the owner/tracker
/// annotations `Baseline::of_preserving` just merged in — fully intact rather than truncated.
/// `create_baseline_file` above has no pre-existing content to protect, so only this, the overwrite
/// path, needs the whole guarantee; see its own doc for what a crash can leave there.
///
/// The fsync before the rename is what makes the crash half of that true, and it is not
/// redundant with the rename's atomicity: `rename` is atomic with respect to *other observers*
/// (no one ever sees a half-swapped name), but it orders only the directory entry, never the
/// temp file's still-dirty data pages. Without the fsync, a crash shortly after a successful
/// rename can leave the baseline path present and empty — losing both the previous content and
/// the annotations merged into it, the exact loss the temp-then-rename exists to prevent. ext4
/// happens to paper over this for the replace-via-rename pattern via its `auto_da_alloc`
/// heuristic, but that is one filesystem's courtesy, disabled by `noauto_da_alloc` and absent
/// elsewhere; this crate ships to adopters on filesystems it does not choose, so the guarantee
/// is made explicitly rather than borrowed. The directory flush after the rename covers the
/// other half — that the swapped-in *name* survives — and is best-effort and unix-only, for the
/// reasons [`sync_parent_dir`] states: it strengthens a write that has already landed, so a
/// platform or filesystem that cannot flush a directory must not turn that write into a
/// reported failure.
///
/// The swap targets the file's symlink-resolved real path and carries over its existing
/// permissions: `rename` unconditionally replaces whatever sits at its destination, so renaming
/// onto `path` directly would silently replace a symlinked baseline with a plain file (orphaning
/// whatever the symlink pointed at) and reset the mode to the temp file's process-umask default,
/// silently widening permissions an adopter deliberately narrowed.
///
/// The temp file is opened with `create_new` (`O_EXCL`), never a plain create-or-truncate: a
/// predictable `<target>.tmp-<pid>` name that instead followed whatever already sat there would
/// let anything pre-planted at that path — a symlink included — receive the write and the
/// permission change that follows it, corrupting a file this process never intended to touch.
/// Measured directly: a symlink planted at the predicted temp path, right after the process was
/// launched, redirected the write and left the baseline's own path as a dangling symlink to the
/// victim. `create_new` refuses outright if anything already exists there, closing that off
/// entirely. Its mode is set to match the original file's at creation (`unix` only — permission
/// bits are not a portable concept), instead of created at the process umask default and narrowed
/// afterward — also measured: with a 0600 baseline, the temp file was briefly 0664 before the
/// follow-up `set_permissions` narrowed it.
///
/// That follow-up still has to run — `O_CREAT`'s mode is masked by the process umask, so it can only
/// *narrow*, and a baseline whose own mode is wider than the umask allows (0666 under umask 022)
/// would otherwise be silently published at 0644. It runs against the **open descriptor**
/// ([`std::fs::File::set_permissions`], an `fchmod`), never against the temp path. Applying it by
/// path would re-open the very race `create_new` was chosen to close, one step after closing it:
/// between the `O_EXCL` open and a path-based `chmod`, anything able to write the baseline's
/// directory — the same access the `create_new` reasoning above assumes — can unlink the temp file
/// and plant a symlink at that predictable name, and `chmod` follows it, stamping the baseline's mode
/// onto a file the attacker chose. A descriptor names the inode this process created, so there is no
/// second name lookup to win. Measured as the syscall, since the resulting mode is identical either
/// way and no test bound to it could tell the two apart: the path form issued
/// `chmod("<target>.tmp-<pid>", 0100666)`, the descriptor form issues `fchmod(4, 0100666)`.
///
/// The cleanup runs in [`AtomicTempFileGuard::drop`] and stays a `remove_file` of the temp path,
/// which is not the same exposure: `unlink` does not follow symlinks, so a symlink planted at that
/// name is itself what gets removed, never its target. The temp path is built by appending to the resolved
/// target's raw `OsString`, never through `Path::display()` (which lossily replaces non-UTF-8
/// bytes for human-readable formatting) — a resolved path is not guaranteed valid UTF-8, and a
/// lossy round-trip through a new string can point at a directory that does not exist, failing an
/// otherwise-valid overwrite outright. `create_new`'s `AlreadyExists` can only come from opening
/// the temp file itself — the step is split out from the rest so that outcome is reported as
/// [`BaselineWriteError::TempPathCollision`] (naming the temp path, not this process's io::Error
/// against the baseline path) rather than inferred from an error kind that could, if a later step
/// ever changed, silently stop meaning "this process created nothing." Any failure once the temp
/// file is open unconditionally cleans it up — no kind-based inference needed there, since opening
/// it is the only step that can fail without this process having created it.
fn write_baseline_atomically(path: &str, document: &str) -> Result<(), BaselineWriteError> {
    let target = std::fs::canonicalize(path)?;
    let permissions = std::fs::metadata(&target)?.permissions();
    let mut tmp_path = target.clone().into_os_string();
    tmp_path.push(format!(".tmp-{}", std::process::id()));
    let tmp_path = PathBuf::from(tmp_path);

    let mut open_options = OpenOptions::new();
    open_options.write(true).create_new(true);
    #[cfg(unix)]
    {
        use std::os::unix::fs::OpenOptionsExt;
        use std::os::unix::fs::PermissionsExt;
        open_options.mode(permissions.mode() & 0o777);
    }

    let mut file = match open_options.open(&tmp_path) {
        Ok(file) => file,
        Err(err) if err.kind() == std::io::ErrorKind::AlreadyExists => {
            return Err(BaselineWriteError::TempPathCollision(tmp_path));
        }
        Err(err) => return Err(err.into()),
    };

    let guard = AtomicTempFileGuard::new(tmp_path);

    // The fsync sits after `set_permissions` and before the `rename`, so one call flushes both
    // the bytes and the mode change (both live on the same inode) while the temp file is still
    // the only name reaching them. Syncing before the chmod would leave the mode unflushed; the
    // rename must come after both, since it is the step that publishes them.
    file.write_all(document.as_bytes())?;
    file.set_permissions(permissions)?;
    file.sync_all()?;
    std::fs::rename(guard.path(), &target)?;
    guard.commit();

    sync_parent_dir(&target);
    Ok(())
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
