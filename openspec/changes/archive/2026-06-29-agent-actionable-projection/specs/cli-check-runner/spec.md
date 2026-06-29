## MODIFIED Requirements

### Requirement: Machine-readable report format

The runner SHALL accept `--format json` (and `--format=json`) and emit the outcome as a JSON document on standard output; the default format SHALL remain human-readable text, so existing invocations are unchanged. An unrecognized format value SHALL be a usage error that exits 2, never a silent fallback. The `markdown` format is a `list`-only projection of the declared law and is NOT a `check` format: `check --format markdown` SHALL be a usage error that exits 2, because `check`'s machine-readable output is the JSON report, not a law summary. The JSON SHALL faithfully project the outcome: an `outcome` discriminant (`clean`, `violations`, or `constitution_error`), the `exit_code` mirroring the process exit, a `violations` array, a `stale_baseline` array (empty outside gate mode), and an `error` message (null unless a constitution error). Each violation SHALL carry its `kind`, `target`, `rule`, `finding`, `reason`, `severity`, and `baselined` flag; the `reason` SHALL serve as the repair hint with no separate invented field.

#### Scenario: JSON format emits a parseable violations document

- **WHEN** the runner checks a workspace with an enforced crate violation under `--format json`
- **THEN** standard output is a JSON document with `outcome` `"violations"`, `exit_code` 1, and a violation whose `kind` is `"crate"` naming the offending dependency

#### Scenario: A clean workspace emits a clean JSON document

- **WHEN** the runner checks a clean workspace under `--format json`
- **THEN** standard output is a JSON document with `outcome` `"clean"`, `exit_code` 0, and an empty `violations` array

#### Scenario: The default format is unchanged

- **WHEN** the runner is invoked without `--format`
- **THEN** it prints the human-readable report exactly as before

#### Scenario: An unknown format is a usage error

- **WHEN** the runner is invoked with `--format` set to a value other than `text` or `json`
- **THEN** it prints usage guidance and exits 2

#### Scenario: Check rejects the list-only markdown format

- **WHEN** the runner is invoked as `check --format markdown`
- **THEN** it prints usage guidance and exits 2, because `markdown` projects the declared law and is a `list` format, while `check`'s machine output is the JSON report

#### Scenario: Gate mode JSON reflects baseline and stale entries

- **WHEN** the runner gates against a baseline under `--format json`
- **THEN** baselined violations carry `baselined: true`, the `exit_code` reflects only new enforce violations, and baseline entries matching no current violation appear in `stale_baseline`
