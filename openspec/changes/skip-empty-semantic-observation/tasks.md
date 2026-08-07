## 1. Empty semantic contribution

- [x] 1.1 Add an unreadable-manifest test for an empty `SemanticObserver`
- [x] 1.2 Return `Clean` before metadata I/O when the semantic boundary set is empty
- [x] 1.3 Record the corrected composition parity under `[Unreleased]`

## 2. Verification

- [x] 2.1 Observe the unreadable-manifest test fail against the pre-change observer
- [x] 2.2 Run focused hunyi and observer-protocol tests, formatting, and repository hygiene gates
- [x] 2.3 Run the complete repository Definition of Done

### Verification evidence

- With the new test but without the early return, `cargo test -p hunyi
  empty_boundaries_are_clean_without_reading_a_manifest` ran the test and exited 101 because the observed outcome
  was not `Clean` for the nonexistent manifest.
- The focused hunyi guard and the full observer-protocol integration test passed after the early return, followed
  by formatting, diff, whitespace, and reference-integrity checks.
- The complete repository Definition of Done passed, including all Rust build, lint, test, documentation,
  dependency-policy, release-state, governance, and example reaction gates.
