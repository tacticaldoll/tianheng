## 1. Language-aware regions

- [x] 1.1 Add a guard distinguishing Rust and shell executed lines
- [x] 1.2 Replace the language-blind region with explicit `rust()` and `shell()` regions
- [x] 1.3 Migrate every current executed-region recognizer to its source language

## 2. Verification

- [x] 2.1 Observe the guard fail with language-blind filtering
- [x] 2.2 Run the owning governance integration tests and repository hygiene gates
- [x] 2.3 Run the complete repository Definition of Done

### Verification evidence

- With the Rust region temporarily assigned shell's `#` comment marker, `cargo test -p tianheng --test
  source_regions` ran the final guard and exited 101 because it could no longer observe `#[cfg(test)]`.
- The source-region guard and the gate-shape, observer-protocol, and projection-register integration suites passed
  with the correct mapping, followed by OpenSpec validation and repository hygiene gates.
- The complete repository Definition of Done passed, including all Rust, dependency-policy, release-state,
  governance, and isolated example reactions.
