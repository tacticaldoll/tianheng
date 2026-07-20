## 1. Quality Matrix

- [x] 1.1 Add one examples-script quality helper that runs format check, all-target Clippy with
  warnings denied, and rustdoc with warnings denied using the current workspace's patch arguments.
- [x] 1.2 Invoke the quality helper for all six isolated example workspaces before accepting their
  existing tests and Tianheng reactions.
- [x] 1.3 Repair only formatting, Clippy, and rustdoc findings made observable by the new matrix,
  preserving each deliberate architectural violation.

## 2. Contract and Failure Proof

- [x] 2.1 Add a focused failure proof showing an isolated-example warning stops the gate before a
  successful reaction could be accepted.
- [x] 2.2 Update repository hygiene documentation and the changelog to distinguish Rust quality
  coverage from the existing behavior/reaction coverage.

## 3. Verification

- [x] 3.1 Run the complete examples gate and verify all six quality rows and all existing reactions.
- [x] 3.2 Run the complete Definition of Done from `AGENTS.md` and strict OpenSpec validation.
- [x] 3.3 Confirm committed example manifests, public APIs, identity/baseline/report wire, root
  manifests and lockfile, and package versions are unchanged.
