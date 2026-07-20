## 1. Release-State Reaction

- [ ] 1.1 Implement deterministic release-spine discovery, numeric version ordering, and
  development, release-ready, and release-snapshot classification with named failure diagnostics.
- [ ] 1.2 Validate workspace version inheritance and internal dependency pins in every state, then
  validate state-specific changelog and workspace lock surfaces without mutating them.

## 2. Failure Proof and Workflow

- [ ] 2.1 Add temporary-repository tests for all three valid states and for missing history, version
  regression, empty development notes, stale lock entries, missing release notes, and mismatched
  snapshot subjects.
- [ ] 2.2 Add the coherence check to local Definition of Done and a dedicated CI job whose checkout
  includes complete git history.
- [ ] 2.3 Update BACKLOG, CHANGELOG, and release-procedure prose to name the built reaction and its
  development-versus-release bounds.

## 3. Verification

- [ ] 3.1 Run shell syntax/static checks, the focused state matrix, and the live development-state
  check against this branch.
- [ ] 3.2 Run the complete Definition of Done and strict OpenSpec validation.
- [ ] 3.3 Confirm the check itself changed no manifests, lockfile versions, package versions, tags,
  release commits, public APIs, or identity/baseline/report wire.
