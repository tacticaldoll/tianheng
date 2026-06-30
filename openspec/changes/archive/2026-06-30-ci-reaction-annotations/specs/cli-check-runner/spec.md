## MODIFIED Requirements

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
