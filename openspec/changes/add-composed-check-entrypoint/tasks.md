## 1. Shared composition seam

- [ ] 1.1 Extract the existing static/semantic/runtime evaluation block into one internal evaluator that also preserves static coverage for the CLI.
- [ ] 1.2 Add public `check_constitution(&Constitution, &Path) -> Outcome` and re-export it through the prelude inspection tier.

## 2. Reaction evidence

- [ ] 2.1 Add tests for merged multi-dimension violations, deterministic constitution-error precedence, and runtime orphan-probe reaction through the library entrypoint.
- [ ] 2.2 Verify CLI exit/projection behavior remains derived from the shared evaluator and update the adopter-surface compile contract.

## 3. Adopter experience

- [ ] 3.1 Replace the composed example's per-dimension workaround with one `check_constitution` Outcome assertion covering both source-observed dimensions.
- [ ] 3.2 Document the presentation-free library check, its explicit-manifest requirement, and the baseline/coverage/presentation boundary in rustdoc, README, PROJECT, and BACKLOG.

## 4. Compatibility and validation

- [ ] 4.1 Verify pacta compiles unchanged, no existing prelude export changes, and no package version, manifest dependency graph, or lockfile entry changes.
- [ ] 4.2 Run OpenSpec strict validation and the full repository Definition of Done.
- [ ] 4.3 Complete apply adversarial review and resolve every finding before checking the change complete.
