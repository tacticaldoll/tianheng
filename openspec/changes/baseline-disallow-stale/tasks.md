## 1. CLI Option Parsing and Validation

- [ ] 1.1 Add `--disallow-stale` CLI flag parsing to `CheckOptions` in `crates/tianheng/src/runner.rs`
- [ ] 1.2 Add usage validation: fail with `Outcome::UsageError` (exit 2) when `--disallow-stale` is passed without `--baseline`

## 2. Gate Evaluation & Reporting

- [ ] 2.1 Update `evaluate_check_outcome` in `crates/tianheng/src/runner.rs` to treat non-empty `stale_baseline` as `Outcome::Violations` (exit 1) when `disallow_stale` is enabled
- [ ] 2.2 Update human-readable and machine-readable output formatting for `disallow_stale` gate failure

## 3. Verification & Integration Testing

- [ ] 3.1 Add integration tests in `crates/tianheng/tests/baseline_cli.rs` covering exit 1 on stale baseline with `--disallow-stale` and exit 2 on `--disallow-stale` without `--baseline`
- [ ] 3.2 Verify workspace pre-flight DoD checks (`cargo clippy`, `cargo fmt`, `cargo test`)
