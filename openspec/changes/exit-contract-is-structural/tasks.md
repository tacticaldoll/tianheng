## 1. Make the exit contract structural

- [x] 1.1 Install an `ERR` trap under `set -E` that reports cannot-judge with `$LINENO` and exits 2, placed
      where it covers the whole run, and state in its comment that it reports where rather than what.
- [x] 1.2 Confirm on this repository that the passing path is unaffected — every deliberate non-zero return
      in the file (`grep -q` misses, `[[ ]] && continue`, `((…)) || cannot_judge`, captured pipelines with
      handlers) still behaves, which the matrix's passing directions assert.

## 2. Name the failures worth naming

- [x] 2.1 `parse_spec` captures its own status and refuses with the spec's path, so the backstop is not the
      diagnosis for a read that has a name.

## 3. Retire the filesystem walk

- [x] 3.1 Enumerate packages from tracked `crates/*/Cargo.toml` through `read_tracked_files`, dropping
      `find` and its swallowed pipeline status; keep the empty-list refusal.
- [x] 3.2 Confirm the member set is unchanged on this repository (six packages).

## 4. Prove all three directions

- [x] 4.1 An unhandled failure exits 2, not the utility's status — `mktemp` stubbed to fail.
- [x] 4.2 An unreadable spec names itself — `sed` stubbed to fail for the CR-stripping expression.
- [x] 4.3 A partial package enumeration refuses — `git ls-files` stubbed to fail for the manifest pathspec.
- [x] 4.4 Record each negative run: without its repair, what exit code and message the fixture produced.

## 5. Declare and record

- [x] 5.1 Apply the delta at sync: requirement prose plus the two scenarios.
- [x] 5.2 `docs/observation-bounds.md` unchanged; register still reports 42 across 15.
- [x] 5.3 `[Unreleased]` CHANGELOG entry.

## 6. Verify

- [x] 6.1 `test_bound_register.sh`, `check_bound_register.sh`, `check_whitespace_hygiene.sh`,
      `test_reference_integrity.sh`, `check_reference_integrity.sh`, `check_dod_coherence.sh`,
      `check_release_coherence.sh`.
- [x] 6.2 The Rust passes as the Definition of Done requires, though unchanged by this change.
