## 1. Example-Set Reaction

- [x] 1.1 Add a repository-only helper that derives immediate example workspaces, rejects unknown
  fulfillment claims, and fails with every unfulfilled live example.
- [x] 1.2 Add focused temporary-tree tests proving unknown claims and missing owners both fail with
  named diagnostics, and that artifact roots are unique and removed after a failed child run.

## 2. Driver and Workflow Hygiene

- [x] 2.1 Register each example only after its quality and declared reaction assertions, then close
  the live example inventory at the end of `scripts/test_examples.sh`.
- [x] 2.2 Move every examples-driver projection, command output, and generated baseline under one
  invocation-local temporary root with unconditional cleanup.
- [x] 2.3 Refresh current-version comments and BACKLOG/CHANGELOG prose without changing any example
  manifest, dependency declaration, architectural fault, or teaching scope.

## 3. Verification

- [x] 3.1 Run shell syntax/static checks, both focused ledger tests, and the complete live examples
  gate; perform adversarial apply review against every new scenario.
- [x] 3.2 Run the complete Definition of Done and strict OpenSpec validation, then confirm manifest
  dependency/version semantics, package versions, Cargo.lock, self-law, public API, identity/report
  wire, tags, and release commits are unchanged.
