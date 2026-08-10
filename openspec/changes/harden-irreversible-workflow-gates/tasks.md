## 1. Publish remote evidence

- [x] 1.1 Add directions that distinguish an unreadable remote from a successful remote without `refs/heads/main`, observe the new diagnostic assertion fail against the current implementation, then preserve the Git failure cause in `publish_source_gate` and make both directions pass.
- [x] 1.2 Run the complete publish-source test target and confirm the existing signed-tag, source-identity, and cannot-judge directions remain unchanged.

## 2. Squash-merge commit evidence

- [x] 2.1 Add a controlled-command wrapper test that proves a live commit absent from local refs reaches the Rust gate, observe it fail against the current local-ref acquisition, then switch `merge-pr.sh` to the paginated pull-request commits API and make the direction pass.
- [x] 2.2 Add controlled directions for multi-page commit order, an API read failure, and an empty subject set; prove both failure directions stop before the gate and merge command and that no local-ref fallback is used.
- [x] 2.3 Run the complete squash-message and wrapper test targets and confirm all existing subject, body, argument-integrity, and gate-ran directions remain unchanged.

## 3. Contract and verification

- [x] 3.1 Update live implementation comments and `[Unreleased]` self-governance prose affected by the evidence-source change without broadening the wrapper into a verdict owner.
- [x] 3.2 Run formatting, targeted Clippy, the workspace tests that exercise the changed gates, and the repository's full Definition of Done; record every new guard's pre-fix failure and the final commands in the PR verification notes.
- [x] 3.3 Adversarially review the diff for incomplete API pagination, fork-head assumptions, swallowed command failures, accidental real-network execution in tests, and any public API or compatibility effect.
