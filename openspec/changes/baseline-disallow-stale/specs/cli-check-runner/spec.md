## MODIFIED Requirements

### Requirement: Baseline flags

The runner SHALL accept two mutually exclusive baseline flags: `--baseline <file>` selects gate mode (suppress baselined violations, fail only on new ones) and `--write-baseline <file>` records the current violations as a baseline. Each SHALL also accept the `=<file>` form. Supplying both SHALL be a usage error that exits 2. In gate mode the process exit code SHALL reflect the gated outcome — 0 when the only violations are baselined or warn, 1 on a new enforce-severity violation. A baseline file that cannot be read or parsed SHALL be treated as a scan error and exit 2.

The runner SHALL additionally accept `--disallow-stale` (and `--disallow-stale` with no value) in gate mode (`--baseline <file>`). When `--disallow-stale` is enabled and one or more stale baseline entries are present, the runner SHALL treat stale baseline entries as a gate failure and exit `1`. Supplying `--disallow-stale` without `--baseline <file>` SHALL be a usage error that exits `2`.

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

#### Scenario: Disallow-stale fails on stale baseline entry

- **WHEN** the runner is invoked with `--baseline <file>` and `--disallow-stale`, and the baseline file contains a stale entry no longer present in the workspace
- **THEN** the runner reports the stale baseline entry as a failure and exits 1

#### Scenario: Disallow-stale without baseline is a usage error

- **WHEN** the runner is invoked with `--disallow-stale` without `--baseline`
- **THEN** the runner reports a usage error and exits 2
