## 1. One enumerator that can fail loudly

- [ ] 1.1 Replace `tracked_files` with `read_tracked_files <array> <pathspec…>`: write `git ls-files -z`
      to a trap-owned temp file, check the status in the parent, `cannot_judge` on failure naming the
      pathspec, and `mapfile -d ''` into the caller's array through a nameref.
- [ ] 1.2 Join the buffer to the EXIT trap with the same `${var:+"$var"}` expansion `rendered` uses, so a
      lazily-created file cannot leak on an abort.

## 2. Route every enumeration through it

- [ ] 2.1 `definitions_of`, `build_tracked_path_index`, and the census scan.
- [ ] 2.2 The spec-file list too, so its diagnosis names the enumeration rather than the repository, and
      confirm the enumeration order still matches the previous `| sort` (git lists tracked paths in index
      order, which is path-sorted).
- [ ] 2.3 State the residual in the comment: the attribute-run `sed` and the id-table `awk` read
      already-materialized data, not the observation source.

## 3. Prove the direction

- [ ] 3.1 Add the matrix fixture: `git` stubbed to fail for the census enumeration only, over a repository
      holding a stale census, asserting exit 2 and the naming of the failed enumeration.
- [ ] 3.2 Record the negative run — without the status check, that fixture exits 0 over a census it never
      read.

## 4. Declare what now reacts

- [ ] 4.1 Apply the delta at sync: the requirement prose plus the failed-enumeration and absent-spec
      scenarios.
- [ ] 4.2 Confirm `docs/observation-bounds.md` is unchanged (no bound statement moves) and the register
      still reports 42 across 15.
- [ ] 4.3 Add the `[Unreleased]` CHANGELOG entry.

## 5. Verify

- [ ] 5.1 `bash scripts/test_bound_register.sh`, `bash scripts/check_bound_register.sh`,
      `bash scripts/check_whitespace_hygiene.sh`, `bash scripts/check_reference_integrity.sh`,
      `bash scripts/check_dod_coherence.sh`, `bash scripts/check_release_coherence.sh`.
- [ ] 5.2 The Rust passes, unchanged by this change but run as the Definition of Done requires:
      `cargo clippy` (four passes), `cargo fmt --all --check`,
      `TIANHENG_WORKSPACE_TESTS=1 cargo test --workspace --all-features`, `cargo doc` with `-D warnings`.
