## 1. External compilation contract

- [ ] 1.1 Add the symmetric `ModuleRule` prelude re-export and a `tianheng` integration test that imports only `prelude::*` and names every declaration/execution and reaction-inspection export.
- [ ] 1.2 Type-check representative static, semantic, runtime, selector, profile, `run`, and pure-check usage without executing external effects.

## 2. Adopter-facing contract

- [ ] 2.1 Document the purpose-only prelude tiers and explicit-root signature-coupling check in `tianheng` rustdoc and README.
- [ ] 2.2 Record the 0.2.x adopter-surface decision and mark the prelude audit resolved in `PROJECT.md` and `BACKLOG.md`.

## 3. Compatibility evidence

- [ ] 3.1 Run the focused external-view test and verify composed/sans-I/O examples plus pacta compile unchanged.
- [ ] 3.2 Confirm `ModuleRule` is the only added prelude path and no existing export, package version, manifest dependency graph, or lockfile entry changed.

## 4. Validation

- [ ] 4.1 Run OpenSpec strict validation and the full repository Definition of Done, including self-law projection and example reactions.
- [ ] 4.2 Complete apply adversarial review, resolving any finding before checking the change complete.
