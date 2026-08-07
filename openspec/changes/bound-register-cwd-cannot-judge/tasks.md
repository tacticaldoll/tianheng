## 1. Census working directory

- [x] 1.1 Add a failure-matrix fixture that removes the repository before the census scan
- [x] 1.2 Separate the repository transition from grep's captured result
- [x] 1.3 Record the fail-loud correction under `[Unreleased]`

## 2. Verification

- [x] 2.1 Observe the new matrix direction fail against the combined `cd && grep` capture
- [x] 2.2 Run the bound-register matrix, OpenSpec validation, and repository hygiene gates
- [x] 2.3 Run the complete repository Definition of Done

### Verification evidence

- With the new matrix direction and the old combined `cd && grep` capture, `bash scripts/test_bound_register.sh`
  exited 1: the gate itself returned violation status 1 after failing to enter the removed fixture, instead of the
  required cannot-judge status 2.
- The repaired gate and its complete failure matrix passed, followed by OpenSpec validation and repository hygiene.
- The complete repository Definition of Done passed, including all Rust, dependency-policy, release-state,
  governance, and isolated example reactions.
