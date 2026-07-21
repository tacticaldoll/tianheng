## 1. Update `louke` Tests

- [ ] 1.1 In `crates/louke/src/audit/tests.rs`, find `zzz_tmp_finder_repro_nonmodrs_path_base`.
- [ ] 1.2 Rename it to `finder_repro_nonmodrs_path_base` (removing `zzz_tmp_`).
- [ ] 1.3 Add `assert_eq!(outcome.exit_code(), 1);` replacing the print statements.
- [ ] 1.4 In the same file, find `zzz_tmp_finder_repro_fn_orphan`.
- [ ] 1.5 Rename it to `finder_repro_fn_orphan` (removing `zzz_tmp_`).
- [ ] 1.6 Add `assert_eq!(outcome.exit_code(), 1);` replacing the print statements.

## 2. Verification

- [ ] 2.1 Run `cargo test -p louke --all-targets` to verify the modified tests pass.
- [ ] 2.2 Run `cargo clippy -p louke -- -D warnings` to verify no warnings were introduced.
