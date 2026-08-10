## 1. Effective command evidence

- [ ] 1.1 Extract the DoD-to-CI comparison behind a controlled document input and add a direction that removes the supply-chain step; observe it pass incorrectly against the hardcoded exemption.
- [ ] 1.2 Project `EmbarkStudios/cargo-deny-action` plus `with.command: check` to effective `cargo deny check`, remove the exemption, and make the missing-step direction fail as intended while the real workflow remains clean.
- [ ] 1.3 Add absent and wrong action-command directions so an unrecognized or misconfigured action cannot satisfy the DoD entry.

## 2. Verification and lifecycle

- [ ] 2.1 Run formatting, targeted Clippy, the DoD coherence target, and the complete repository Definition of Done; record the pre-fix false green and final commands in PR verification notes.
- [ ] 2.2 Adversarially review YAML step scoping, accidental generic action claims, swallowed parse failures, and compatibility impact.
- [ ] 2.3 Sync the delta requirement, complete the OpenSpec archive lifecycle, and remove the dated archive before creating the PR.
