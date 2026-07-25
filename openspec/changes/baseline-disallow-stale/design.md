## Context

`tianheng check` evaluates constitution boundaries against Cargo workspaces and supports gate mode via `--baseline <file>`. In gate mode, baseline entries are compared against current findings, producing a list of active violations and a list of `stale_baseline` entries. Stale entries represent violations that were previously baselined but are no longer present in the checked codebase. Currently, `stale_baseline` entries are printed as advisories (or included in JSON/SARIF output) but do not cause process exit code 1.

## Goals / Non-Goals

**Goals:**
- Add `--disallow-stale` flag to `tianheng check` CLI.
- Fail process execution with exit code 1 when `--disallow-stale` is enabled and `stale_baseline` is non-empty.
- Fail process execution with usage error exit code 2 when `--disallow-stale` is supplied without `--baseline <file>`.
- Add integration tests in `crates/tianheng/tests/baseline_cli.rs` validating exit code 1 on stale baseline entries with `--disallow-stale` and exit code 2 on missing `--baseline`.

**Non-Goals:**
- Automatically rewrite baseline files (use `--write-baseline` for re-generating baselines).
- Changing baseline identity rules or format schemas in `xuanji`.

## Decisions

### Decision 1: CLI Flag Parsing in `runner.rs`
- Add `disallow_stale: bool` flag to `CheckOptions` in `crates/tianheng/src/runner.rs`.
- Parse `--disallow-stale` in CLI argument parser.
- Validate flag combinations: if `disallow_stale` is true but `baseline` is `None`, return a usage error outcome (exit 2).

### Decision 2: Gate Outcome Evaluation
- In `evaluate_check_outcome`, when `disallow_stale` is true and `stale_baseline` is not empty, set outcome to `Outcome::Violations` (exit code 1).
- Emit human-readable warning / error summary naming stale baseline entries when `disallow_stale` triggers exit 1.

## Risks / Trade-offs

- **[Risk]** Existing scripts passing `--disallow-stale` without `--baseline` → **[Mitigation]** Exit 2 with explicit usage error text explaining that `--disallow-stale` requires `--baseline`.
- **[Risk]** JSON / SARIF schema incompatibility → **[Mitigation]** JSON already includes `stale_baseline` array; process exit code field in JSON report matches exit code 1.
