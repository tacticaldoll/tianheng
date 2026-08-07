## 1. Public dimension surfaces

- [x] 1.1 Add the complete shared bound and observer vocabulary to `guibiao` and `louke` root re-exports
- [x] 1.2 Hold the same vocabulary through external adopter-surface tests for all three dimensions

## 2. Adopter narrative

- [x] 2.1 Record the additive standalone-dimension exports under `[Unreleased]`

## 3. Verification

- [x] 3.1 Observe the new guibiao and louke adopter tests fail to compile without their root re-exports
- [x] 3.2 Run the three dimension adopter tests, formatting, and repository hygiene gates
- [x] 3.3 Run the complete repository Definition of Done

### Verification evidence

- Before the re-exports, `cargo test -p guibiao --test adopter_surface` exited 101 and named all nine missing
  bound/observer types that were absent from the root while `Outcome` already resolved.
- Before the re-exports, `cargo test -p louke --test adopter_surface` exited 101 and named the same missing types.
- The guibiao, hunyi, louke default-feature, and louke all-feature adopter tests passed after the exports, followed
  by formatting, diff, whitespace, and reference-integrity checks.
- Every command in the current `AGENTS.md` Definition of Done passed, including all build, Clippy, test, rustdoc,
  cargo-deny, gate-matrix, release-coherence, bound-register, and example-family commands.
