## Why

The tests `zzz_tmp_finder_repro_nonmodrs_path_base` and `zzz_tmp_finder_repro_fn_orphan` in `crates/louke/src/audit/tests.rs` only print their outcomes using `eprintln!` and do not assert the results. Since they do not panic, they always pass regardless of whether the underlying `louke` functionality (probe coverage auditing) works correctly or not. These "phantom tests" provide a false sense of security and clutter the codebase with temporary debugging scripts.

## What Changes

- Turn the temporary repro test `zzz_tmp_finder_repro_nonmodrs_path_base` into a real regression test by adding `assert_eq!(outcome.exit_code(), 1)` (or the expected exit code) instead of just printing it, and rename it to a formal test name.
- Turn the temporary repro test `zzz_tmp_finder_repro_fn_orphan` into a real regression test similarly by asserting the outcome, and rename it.
- Remove the `zzz_tmp_` prefix from both tests to signify they are permanent assertions.

## Capabilities

### New Capabilities
None.

### Modified Capabilities
None. This is a test hygiene/internal correctness patch that does not change the declared capabilities of `louke`.

## Impact

- `crates/louke/src/audit/tests.rs`
- Increased test reliability. No public API, behavior, or reaction output is changed for end users.
