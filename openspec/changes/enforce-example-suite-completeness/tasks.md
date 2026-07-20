## 1. Example-Set Reaction

- [ ] 1.1 Add a repository-only helper that derives immediate example workspaces, rejects unknown
  fulfillment claims, and fails with every unfulfilled live example.
- [ ] 1.2 Add focused temporary-tree tests proving unknown claims and missing owners both fail with
  named diagnostics, and that artifact roots are unique and removed after a failed child run.

## 2. Driver and Workflow Hygiene

- [ ] 2.1 Register each example only after its quality and declared reaction assertions, then close
  the live example inventory at the end of `scripts/test_examples.sh`.
- [ ] 2.2 Move every examples-driver projection, command output, and generated baseline under one
  invocation-local temporary root with unconditional cleanup.
- [ ] 2.3 Refresh current-version comments and BACKLOG/CHANGELOG prose without changing any example
  manifest, dependency declaration, architectural fault, or teaching scope.

## 3. Verification

- [ ] 3.1 Run shell syntax/static checks, both focused ledger tests, and the complete live examples
  gate; perform adversarial apply review against every new scenario.
- [ ] 3.2 Run the complete Definition of Done and strict OpenSpec validation, then confirm manifests,
  package versions, Cargo.lock, self-law, public API, identity/report wire, tags, and release commits
  are unchanged.
