# cli-check-runner Specification

## Purpose

Govern the `tianheng check` command-line contract — the runner that turns the
Rust-declared constitution into a CI reaction. It fixes the flag surface
(`--manifest-path`), the usage-error handling, and how the process exit code and
report mirror the reaction outcome (0 clean or warn-only, 1 enforce violation,
2 constitution/usage error), so a CI gate has a stable, non-bypassable contract.
## Requirements
### Requirement: Check command interface

The runner SHALL provide a `check` command that accepts the target Cargo
workspace via `--manifest-path <path>`, also accepting the `--manifest-path=<path>`
form. The runner SHALL evaluate the Rust-declared constitution against the
workspace at that path and translate the resulting outcome into a process exit
code. The runner MUST NOT require any flag other than `--manifest-path` to perform
a check.

#### Scenario: Check evaluates the target at the given manifest path

- **WHEN** the runner is invoked as `tianheng check --manifest-path <path>` where the path is a readable Cargo workspace
- **THEN** the runner evaluates the constitution against that workspace and exits with the code that mirrors the outcome

#### Scenario: The equals form of the flag is accepted

- **WHEN** the runner is invoked as `tianheng check --manifest-path=<path>`
- **THEN** the runner uses `<path>` as the target workspace, identically to the space-separated form

### Requirement: Process exit code mirrors the reaction outcome

The runner SHALL exit `0` when no enforce-severity boundary is violated, `1` when one or more enforce-severity boundaries are violated, and `2` for a constitution or scan error. Violations of warn-severity boundaries SHALL be reported but SHALL NOT by themselves cause a non-zero exit, so a warn-only run exits `0`. On any non-zero exit the runner SHALL print a human-readable report or error message. The runner MUST NOT exit `0` when it could not evaluate the constitution.

#### Scenario: Clean target exits 0

- **WHEN** the checked workspace satisfies every boundary
- **THEN** the runner reports that no boundary was violated and exits `0`

#### Scenario: Enforce violation exits 1 with a report

- **WHEN** one or more enforce-severity boundaries are violated in the checked workspace
- **THEN** the runner prints a violation report and exits `1`

#### Scenario: Warn-only violations exit 0 with an advisory

- **WHEN** the only violations are of warn-severity boundaries
- **THEN** the runner prints the violations as advisories and exits `0`

#### Scenario: Constitution error exits 2 with a message

- **WHEN** the constitution cannot be evaluated against the workspace (e.g. an unresolvable target or an unreadable workspace)
- **THEN** the runner prints a constitution error message and exits `2`, never `0`

### Requirement: Baseline flags

The runner SHALL accept two mutually exclusive baseline flags: `--baseline <file>` selects gate mode (suppress baselined violations, fail only on new ones) and `--write-baseline <file>` records the current violations as a baseline. Each SHALL also accept the `=<file>` form. Supplying both SHALL be a usage error that exits 2. In gate mode the process exit code SHALL reflect the gated outcome — 0 when the only violations are baselined or warn, 1 on a new enforce-severity violation. A baseline file that cannot be read or parsed SHALL be treated as a scan error and exit 2.

#### Scenario: Write-baseline records and exits 0

- **WHEN** the runner is invoked with `--write-baseline <file>` against a workspace with violations
- **THEN** the runner writes the baseline file and exits 0

#### Scenario: Gate against a baseline that covers all violations exits 0

- **WHEN** the runner is invoked with `--baseline <file>` and every enforce violation is recorded in that file
- **THEN** the runner exits 0

#### Scenario: Gate fails on a violation not in the baseline

- **WHEN** the runner is invoked with `--baseline <file>` and an enforce violation is absent from that file
- **THEN** the runner exits 1 and reports the new violation

#### Scenario: Supplying both baseline flags is a usage error

- **WHEN** the runner is invoked with both `--baseline` and `--write-baseline`
- **THEN** the runner prints usage guidance and exits 2

#### Scenario: An unreadable baseline file exits 2

- **WHEN** the runner is invoked with `--baseline <file>` and the file is missing or malformed
- **THEN** the runner reports a scan error and exits 2

### Requirement: Machine-readable report format

The runner SHALL accept `--format json` (and `--format=json`) and emit the outcome as a JSON document on standard output; the default format SHALL remain human-readable text, so existing invocations are unchanged. The runner SHALL additionally accept `--format sarif` as a machine/CI-consumable projection of the same outcome (defined below). An unrecognized format value SHALL be a usage error that exits 2, never a silent fallback. The `markdown` format is a `list`-only projection of the declared law and is NOT a `check` format: `check --format markdown` SHALL be a usage error that exits 2, because `check`'s machine-readable output is the JSON report, not a law summary. The JSON SHALL faithfully project the outcome: an `outcome` discriminant (`clean`, `violations`, or `constitution_error`), the `exit_code` mirroring the process exit, a `violations` array, a `stale_baseline` array (empty outside gate mode), and an `error` message (null unless a constitution error). Each violation SHALL carry its `kind`, `target`, `rule`, `finding`, `reason`, `severity`, and `baselined` flag; the `reason` SHALL serve as the repair hint with no separate invented field.

Each violation SHALL additionally carry a `file` field naming the offending source file, so an agent knows *where* to repair. The `file` SHALL be a string wherever the offending source file is a faithful byproduct of observation: a **module-import violation** names a source file where the forbidden import occurs (the static scan already reads that file to observe the import), and an **un-auditable-probe runtime violation** names the source file holding the non-literal `assert_boundary!` (the probe scan already captured that file). For every other violation kind the `file` SHALL be `null`, a faithful absence rather than an omitted-but-known location: a crate-dependency violation is an edge in the dependency graph with no single source file; a semantic violation does not currently observe a per-element source file (a stated bound — observing it would require new tracking, not yet built); and a seam-level runtime violation (a duplicate, undeclared, or unprobed seam) names a seam, not a source location, so no single file applies. The `file` SHALL NOT enter the violation's baseline identity (`target`, `rule`, `finding`), so adding or changing it SHALL NOT make an existing baselined violation count as new, and SHALL NOT change the number of violations reported (it is metadata attached after identity de-duplication, never a de-duplication key).

The `sarif` format is a **CI-consumable projection of the reaction** and, like the JSON document, is `check`-only: `list --format sarif` SHALL be a usage error that exits 2 (symmetric to `markdown` being `list`-only). It is an open, **vendor-neutral** standard (consumed by GitHub code-scanning and other tools); a vendor-specific format such as GitHub's `::error::` workflow command is deliberately NOT provided, as it would couple the tool to one CI vendor — emitting such annotations from the neutral output is a harness/CI-step convention, not a tool format. SARIF projects the **same** measure as the JSON — the same non-baselined violations, the same severities, the same exit code (it changes presentation, never the outcome or the process exit). A baselined violation SHALL NOT appear (it does not fail, consistent with the human report). `--format sarif` SHALL emit a SARIF 2.1.0 document whose `runs[].results[]` carries one result per non-baselined violation: `ruleId` the rule, `level` `error` for an enforced violation and `warning` for an advisory, and `message.text` carrying the reason and the finding (the rule is carried structurally in `ruleId`, not repeated in the message). Because a violation observes a `file` but **not a line**, a SARIF result's location SHALL carry only `physicalLocation.artifactLocation.uri` (the file) with **no `region`**; a violation with no `file` SHALL emit a SARIF result with no `locations` — never a fabricated location. A constitution error SHALL be surfaced as a SARIF tool-execution notification at `error` level under `runs[0].invocations[0]`, whose `executionSuccessful` SHALL be `false`, and SHALL exit 2, never a silent pass; a clean outcome SHALL emit a valid SARIF document with empty `results` and exit 0.

#### Scenario: JSON format emits a parseable violations document

- **WHEN** the runner checks a workspace with an enforced crate violation under `--format json`
- **THEN** standard output is a JSON document with `outcome` `"violations"`, `exit_code` 1, and a violation whose `kind` is `"crate"` naming the offending dependency

#### Scenario: A module violation carries a source file

- **WHEN** the runner checks a workspace with a module-import violation under `--format json`
- **THEN** the offending violation's `file` is a string naming a source file where the forbidden import occurs

#### Scenario: A crate violation reports a null file

- **WHEN** the runner checks a workspace with a crate-dependency violation under `--format json`
- **THEN** the offending violation's `file` is `null`, reflecting that a dependency edge has no single source file

#### Scenario: A semantic violation reports a null file under the current bound

- **WHEN** the runner checks a workspace with a semantic violation under `--format json`
- **THEN** the offending violation's `file` is `null`, the stated bound that a per-element source file is not yet observed for the semantic dimension

#### Scenario: An un-auditable-probe runtime violation carries a source file

- **WHEN** the runner checks a workspace whose runtime probe-coverage audit finds a non-literal `assert_boundary!` seam under `--format json`
- **THEN** the offending violation's `file` is a string naming the source file holding that probe, the location the probe scan already captured

#### Scenario: A seam-level runtime violation reports a null file

- **WHEN** the runner checks a workspace with a seam-level runtime violation (a duplicate, undeclared, or unprobed seam) under `--format json`
- **THEN** the offending violation's `file` is `null`, reflecting that the violation names a seam, not a source location

#### Scenario: A module importing from two files still yields one violation

- **WHEN** an importer module backed by more than one source file imports a protected module from each
- **THEN** the report still carries exactly one violation for that importer module (its identity `(target, rule, finding)` is unchanged) and the `file` names one of the offending files deterministically

#### Scenario: Adding a file does not re-baseline an accepted violation

- **WHEN** a workspace has a module violation already recorded in the active baseline, and the report now carries a `file` for it
- **THEN** the violation is still recognized as baselined (its identity `(target, rule, finding)` is unchanged) and does not fail the gate

#### Scenario: A clean workspace emits a clean JSON document

- **WHEN** the runner checks a clean workspace under `--format json`
- **THEN** standard output is a JSON document with `outcome` `"clean"`, `exit_code` 0, and an empty `violations` array

#### Scenario: The default format is unchanged

- **WHEN** the runner is invoked without `--format`
- **THEN** it prints the human-readable report exactly as before

#### Scenario: An unknown format is a usage error

- **WHEN** the runner is invoked with `--format` set to a value other than `text`, `json`, or `sarif`
- **THEN** it prints usage guidance and exits 2

#### Scenario: Check rejects the list-only markdown format

- **WHEN** the runner is invoked as `check --format markdown`
- **THEN** it prints usage guidance and exits 2, because `markdown` projects the declared law and is a `list` format, while `check`'s machine output is the JSON report

#### Scenario: Gate mode JSON reflects baseline and stale entries

- **WHEN** the runner gates against a baseline under `--format json`
- **THEN** baselined violations carry `baselined: true`, the `exit_code` reflects only new enforce violations, and baseline entries matching no current violation appear in `stale_baseline`

#### Scenario: SARIF format emits a valid results document mirroring the violations

- **WHEN** the runner checks a workspace with an enforced violation under `--format sarif`
- **THEN** standard output is a SARIF 2.1.0 document whose `runs[].results[]` has a result with `ruleId` the rule, `level` `error`, and a message carrying the reason; the process exits 1 exactly as under `--format json`

#### Scenario: A file-bearing violation projects a file-level location with no line

- **WHEN** a module-import violation (which carries a `file` but no line) is projected under `--format sarif`
- **THEN** the SARIF result's location has `physicalLocation.artifactLocation.uri` equal to the file and **no `region`** — the unobserved line is omitted, never fabricated

#### Scenario: A file-less violation projects no location

- **WHEN** a violation with a `null` file (e.g. a crate-dependency violation) is projected under `--format sarif`
- **THEN** the SARIF result carries no `locations` — a faithful absence

#### Scenario: SARIF is check-only

- **WHEN** the runner is invoked as `list --format sarif`
- **THEN** it prints usage guidance and exits 2, because SARIF projects the reaction, not the declared law (symmetric to `markdown` being `list`-only)

#### Scenario: A clean workspace under sarif

- **WHEN** the runner checks a clean workspace under `--format sarif`
- **THEN** standard output is a valid SARIF document with empty `results`, and the process exits 0

#### Scenario: A constitution error is surfaced under sarif

- **WHEN** the constitution cannot be evaluated (a constitution error) under `--format sarif`
- **THEN** the SARIF document carries `runs[0].invocations[0].executionSuccessful = false` with a tool-execution notification at `error` level carrying the message, and the process exits 2 — never a silent pass

### Requirement: Runner exposed as a reusable library entry point

The `check` runner contract — argument parsing (`--manifest-path`, `--baseline` / `--write-baseline`, `--format`), the baseline gate and write actions, the report rendering, and the exit-code mapping (`0` clean / warn-only / fully baselined, `1` enforce violation, `2` constitution/scan/usage error) — SHALL be provided by the `tianheng` library as a public entry point. The entry point SHALL accept a caller-supplied constitution and the process arguments and SHALL return the process exit code, evaluating the supplied constitution exactly as the `check` command specifies. An adopting project SHALL obtain the identical runner contract by declaring its own constitution in Rust and invoking this entry point, without reimplementing argument parsing, baseline handling, report rendering, or exit-code mapping. The entry point MUST NOT exit `0` when it could not evaluate the constitution.

#### Scenario: A project runs its own constitution through the library entry point

- **WHEN** a project declares a constitution in Rust and invokes the library runner entry point with that constitution and `check --manifest-path <path>` against a readable workspace
- **THEN** the runner evaluates that project's constitution against the workspace and returns the exit code mirroring the outcome, identically to the `tianheng` binary

#### Scenario: The entry point honors the baseline and format flags

- **WHEN** the library runner entry point is invoked with `--baseline` / `--write-baseline` or `--format json`
- **THEN** it applies the gate or write action and the report format exactly as specified for the `check` command, and returns the gated exit code

#### Scenario: A usage error from the entry point exits 2

- **WHEN** the library runner entry point is invoked with both `--baseline` and `--write-baseline`
- **THEN** it prints usage guidance and returns exit code `2`, never `0` or `1`

#### Scenario: The bundled binary is a thin caller of the entry point

- **WHEN** the `tianheng` binary is invoked as `tianheng check …`
- **THEN** it produces the same flags, reports, and exit codes as before, because it routes through the same library entry point with its own sample constitution

### Requirement: Unrecognized arguments are a usage error

The runner SHALL reject any argument it does not recognize — an unknown flag, a misspelled flag, or a stray positional token — by printing usage guidance to standard error and exiting `2`, never silently ignoring it. This SHALL hold for both the `check` and `list` commands, and matches how an unrecognized `--format` value is already handled, so that a typo such as `--write-baselin` fails loud rather than silently changing behavior. A value consumed by a recognized flag (e.g. the path after `--manifest-path`) SHALL NOT be treated as an unrecognized argument. Conversely, a value-taking flag (`--manifest-path`, `--baseline`, `--write-baseline`, `--format`) supplied with no following value SHALL also be a usage error that prints usage guidance and exits `2`, never a silent downgrade to a default or to a plain check.

#### Scenario: A value-taking flag with no value exits 2

- **WHEN** the runner is invoked as `check --manifest-path <path> --format` (or `--baseline` / `--write-baseline`) with no following value
- **THEN** it prints usage guidance and exits `2`, rather than defaulting the format or running an ordinary check

#### Scenario: An unknown flag exits 2

- **WHEN** the runner is invoked as `check --manifest-path <path> --frobnicate`
- **THEN** it prints usage guidance and exits `2`

#### Scenario: A misspelled flag fails loud instead of being ignored

- **WHEN** the runner is invoked as `check --manifest-path <path> --write-baselin <file>` (a misspelling of `--write-baseline`)
- **THEN** it prints usage guidance and exits `2`, rather than running an ordinary check and writing no baseline

#### Scenario: A stray positional token exits 2

- **WHEN** the runner is invoked as `check stray --manifest-path <path>`
- **THEN** it prints usage guidance and exits `2`

#### Scenario: An unknown flag to list exits 2

- **WHEN** the runner is invoked as `list --bogus`
- **THEN** it prints usage guidance and exits `2`

### Requirement: Workspace coverage reporting

The `check` runner SHALL report workspace coverage: how many workspace members (observed from `cargo metadata`) are the target of no boundary. The coverage line SHALL be emitted whenever the constitution was successfully evaluated — a clean or a violations outcome — in the human-readable text output, and under `--format json` as a `coverage` object carrying the workspace member count and the names of uncovered crates. On a constitution error the runner SHALL emit only the error and SHALL NOT emit coverage, because the constitution could not be evaluated and a coverage line would misrepresent crates as uncovered when the law itself is broken. Coverage SHALL be purely informational: it SHALL NOT by itself change the process exit code, because an uncovered crate is the absence of a declared boundary — neither an architectural violation (exit 1) nor a constitution error (exit 2). A workspace member SHALL count as covered if it is the target of at least one boundary, crate or module. The `list` command SHALL NOT report coverage, because it observes no workspace.

#### Scenario: Coverage line reports uncovered crates

- **WHEN** the checked workspace has four members and the constitution targets three of them
- **THEN** the runner reports that one of four workspace crates has no boundary, and the process exit code is unchanged by that fact

#### Scenario: JSON coverage projection

- **WHEN** `check` runs under `--format json` and the constitution is successfully evaluated
- **THEN** the JSON document carries a `coverage` object with the workspace member count and an array of the uncovered crate names

#### Scenario: A crate covered only by a module boundary counts as covered

- **WHEN** a workspace crate is the target of a module boundary but of no crate boundary
- **THEN** the coverage report counts it as covered

#### Scenario: Coverage is omitted on a constitution error

- **WHEN** `check` cannot evaluate the constitution (e.g. an unresolvable target), in text or under `--format json`
- **THEN** the runner reports the constitution error and exits 2, and the report carries no coverage

### Requirement: The --warn-uncovered flag raises uncovered crates to advisories

The `check` runner SHALL accept a boolean `--warn-uncovered` flag that reports each uncovered workspace crate as a warn-severity advisory. The flag SHALL NOT change the exit-code contract: because warn-severity findings do not fail, a run whose only findings are uncovered-crate advisories SHALL exit 0. The flag SHALL NOT suppress or alter any enforce-severity boundary violation. As a recognized flag it SHALL take no value; supplying it is not a usage error, consistent with how the runner rejects only unrecognized arguments.

#### Scenario: --warn-uncovered reports uncovered crates as advisories without failing

- **WHEN** `check` runs with `--warn-uncovered`, one workspace crate has no boundary, and no enforce-severity boundary is violated
- **THEN** the runner reports the uncovered crate as a warn advisory and exits 0

#### Scenario: --warn-uncovered does not mask an enforced violation

- **WHEN** `--warn-uncovered` is set and an enforce-severity boundary is violated
- **THEN** the runner still prints the violation and exits 1

### Requirement: Absent manifest path defaults to the nearest Cargo.toml

When `--manifest-path` is omitted, the `check` runner SHALL default to the nearest `Cargo.toml` discovered by walking up from the current directory (cargo-style) and evaluate the constitution against that workspace, mirroring the outcome in the exit code. An explicit `--manifest-path` SHALL override the default. The runner MUST NOT exit `0` when it could not evaluate the constitution: if no `Cargo.toml` is found from the current directory up to the root, or the resolved workspace cannot be read, the runner SHALL exit `2` (a scan error), never `0`. When no `Cargo.toml` is found, the scan-error message SHALL name the directory the search started from, so the failure is actionable in a monorepo or CI context. The `list` command is unaffected, as it observes no workspace.

#### Scenario: Absent manifest path resolves the nearest Cargo.toml

- **WHEN** the runner is invoked as `tianheng check` with no `--manifest-path` from within a Cargo workspace
- **THEN** it evaluates the constitution against the nearest `Cargo.toml` and exits with the code that mirrors the outcome

#### Scenario: An explicit manifest path still overrides the default

- **WHEN** the runner is invoked as `tianheng check --manifest-path <path>`
- **THEN** it uses `<path>`, exactly as before, regardless of the current directory

#### Scenario: No Cargo.toml found is a scan error, never a silent pass

- **WHEN** the runner is invoked as `tianheng check` with no `--manifest-path` and no `Cargo.toml` exists from the current directory up to the root
- **THEN** the runner reports that it could not find a workspace, naming the directory it searched from, and exits `2`, never `0`

### Requirement: List rejects flags that only apply to check

The `list` command observes no workspace and performs no reaction; it SHALL accept only `--format`. Supplying a flag that applies only to `check` — `--manifest-path`, `--baseline`, `--write-baseline`, or `--warn-uncovered` — to `list` SHALL be a usage error that prints usage guidance and exits `2`, never silently ignored. This complements the unrecognized-argument rule: a flag that is recognized by `check` but inapplicable to `list` SHALL be rejected rather than accepted as a silent no-op.

#### Scenario: A check-only flag supplied to list is a usage error

- **WHEN** the runner is invoked as `list --baseline <file>` (or `--write-baseline <file>`, `--manifest-path <path>`, or `--warn-uncovered`)
- **THEN** it prints usage guidance and exits `2`, never silently ignoring the flag

#### Scenario: List still accepts the format flag

- **WHEN** the runner is invoked as `list --format json` (or `list --format text`, or `list` with no flag)
- **THEN** it prints the projection and exits `0`, because `--format` is the one flag `list` honors

#### Scenario: An unknown flag to list is still a usage error

- **WHEN** the runner is invoked as `list --bogus`
- **THEN** it prints usage guidance and exits `2`, exactly as the unrecognized-argument rule already requires

### Requirement: Unified constitution declaration

The runner SHALL accept the declared law as a **single constitution object** composing every
observation dimension's boundaries — static, semantic, and runtime — rather than one argument per
dimension. The runner's entry point SHALL take that one constitution and the process arguments, and
SHALL evaluate every dimension the constitution carries, composing them into one reaction. A
constitution carrying no boundaries for a dimension SHALL react as that dimension being absent (no
observation, no violation), never as an error. The unified constitution SHALL remain
source-compatible for a caller that declares only static boundaries: constructing it by name and
adding static boundaries SHALL require no mention of any other dimension. Semantic and runtime
boundaries SHALL be folded into the same constitution through its builder, not passed as separate
runner arguments.

#### Scenario: One constitution drives the whole reaction

- **WHEN** a caller declares static, semantic, and runtime boundaries on a single constitution and invokes the runner with that one constitution
- **THEN** the runner evaluates every declared dimension and exits with the code that mirrors the composed outcome

#### Scenario: A static-only constitution needs no other dimension

- **WHEN** a caller declares only static boundaries on the constitution
- **THEN** the runner reacts against the static dimension alone, and the other dimensions contribute no violation and no error

#### Scenario: Composition and exit codes are unchanged for existing dimensions

- **WHEN** the same static and semantic boundaries are evaluated through the unified constitution as were previously supplied as separate dimension inputs
- **THEN** the exit code, the violation report, and the existing `list` projection sections are identical — the unification changes how the law is declared, never how the static and semantic dimensions react

### Requirement: Check composes the runtime CI audit

When the unified constitution carries one or more runtime boundaries, the `check` command SHALL
compose the runtime dimension's CI audit (probe coverage of the declared seams) into the same
reaction as the static and semantic dimensions, contributing to the one process exit code under the
existing 0/1/2 contract. The runner SHALL resolve the target workspace's member source directories
from the same workspace it already reads, and SHALL evaluate probe coverage across that workspace as
one corpus. A failure to resolve the workspace's sources SHALL be a constitution error (exit 2),
never a silent pass. When the constitution carries no runtime boundaries, the runner SHALL behave
exactly as before (no runtime audit performed).

#### Scenario: A declared runtime seam without a probe fails the check

- **WHEN** the constitution declares a runtime boundary whose seam has no `assert_boundary!` probe anywhere in the workspace, and the boundary's severity is enforce
- **THEN** `check` reports the unprobed seam and contributes a non-zero exit, composed with any static and semantic outcome

#### Scenario: No runtime boundaries means no runtime audit

- **WHEN** the constitution declares no runtime boundaries
- **THEN** `check` performs no runtime audit and its exit code and report are identical to a run before the runtime CI face was composed in

#### Scenario: Unresolvable workspace sources are a constitution error

- **WHEN** runtime boundaries are declared but the workspace's member source directories cannot be resolved
- **THEN** `check` exits `2` with a constitution error, never `0`

### Requirement: Human violation report foregrounds the reason

In the human-readable (text) report that `check` prints for an enforced or advisory violation, the runner SHALL **foreground the `reason`**: the reason SHALL be rendered before the mechanical fields (the boundary target, the rule, and the finding), so the agent reading the reaction leads with the principle and repair direction rather than the mechanical detail. The report SHALL surface the offending `file` when the violation carries one (the "where to repair"), and SHALL omit the file element when the violation has none (a faithful absence, never a fabricated location). The runner SHALL group violations by boundary — ordering the text report's violations by `(target, rule)` — so that multiple findings under one boundary appear together and the reason is read once per boundary.

This governs the **human text report only** — an ordering/grouping/presence invariant over already-observed fields. It does NOT change the JSON projection (the machine contract under "Machine-readable report format"), and it introduces no derived or invented field (no `repair_hint`): the reason is shown as declared, the file as observed.

#### Scenario: The reason leads the violation block

- **WHEN** `check` reports an enforced violation as text
- **THEN** within that violation's block the reason appears before the boundary target, the rule, and the finding

#### Scenario: The offending file is shown when present, omitted when absent

- **WHEN** a reported violation carries an offending `file`
- **THEN** the text report shows that file as the repair location; and **WHEN** a violation carries no file, **THEN** the report shows no file element rather than a fabricated one

#### Scenario: Violations are grouped by boundary

- **WHEN** `check` reports multiple violations spanning more than one boundary as text
- **THEN** the report orders them by `(target, rule)` so all findings under one boundary appear consecutively

#### Scenario: The JSON projection is unchanged

- **WHEN** the same outcome is emitted under `--format json`
- **THEN** the JSON content and field order are exactly as before this change — the foregrounding, file-surfacing, and grouping are presentation of the text report only

