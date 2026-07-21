## Context

The `louke` runtime dimension has two "phantom tests" (`zzz_tmp_finder_repro_nonmodrs_path_base` and `zzz_tmp_finder_repro_fn_orphan`) which are missing assertion logic. They only print their outcomes to standard error and return success, which creates a false sense of security and clutters the test output.

## Goals / Non-Goals

**Goals:**
- Turn phantom tests into real, assertive regression tests.
- Verify that `louke` actually catches the boundary violations described in the tests.

**Non-Goals:**
- Re-architecting `louke`'s `audit_probe_coverage` logic.
- Sweeping other crates (already confirmed there are no other `zzz_` phantom tests).

## Decisions

- **Decision 1:** We will use `assert_eq!(outcome.exit_code(), 1)` on the outcome of `audit_probe_coverage()`. This directly validates the reaction matches the expected policy enforcement.
- **Decision 2:** Rename the tests to remove the `zzz_tmp_` prefix to indicate they are no longer temporary reproduction scripts, but permanent regression tests.

## Risks / Trade-offs

- **Risk:** The tests might legitimately fail once the assertion is added if `louke` has an actual bug in path resolution.
  - **Mitigation:** We will first add the assertions. If they fail, it reveals a real bug in `louke` which we must fix before merging, fulfilling the purpose of this exact patch.
