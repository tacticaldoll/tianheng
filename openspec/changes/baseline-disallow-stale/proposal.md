## Why

`tianheng check` running in gate mode (`--baseline <file>`) suppresses baselined violations and reports any stale baseline entries (entries present in the baseline file but no longer occurring in the workspace). Currently, stale entries are reported as advisories without forcing process exit code 1. In strict CI environments, team policy may require that baseline files remain ratcheted and strictly minimal (no obsolete or stale baseline entries), so CI fails if a baseline file contains stale entries that were not cleaned up.

## What Changes

- Add `--disallow-stale` flag (and `--disallow-stale` equal form) to `tianheng check` CLI.
- When `--baseline <file>` is used together with `--disallow-stale` and any stale baseline entry is present, `tianheng check` SHALL report the stale entries and exit `1`.
- Supplying `--disallow-stale` without `--baseline` SHALL be a usage error that exits `2`.

## Capabilities

### New Capabilities

### Modified Capabilities

- `cli-check-runner`: Add `--disallow-stale` flag requirement, usage error rule for missing `--baseline`, and process exit code scenario when stale baseline entries are disallowed.

## Impact

- `crates/tianheng/src/runner.rs` (CLI argument parsing and check runner gate evaluation)
- CLI contract (`tianheng check --baseline <file> --disallow-stale`)
