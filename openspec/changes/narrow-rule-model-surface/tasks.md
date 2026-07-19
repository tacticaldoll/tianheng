## 1. Narrow the construction surface

- [x] 1.1 Mark every data-carrying `Rule` and `ModuleRule` variant non-exhaustive, retain `CrateBoundary::rule()`, and add the symmetric read-only `ModuleBoundary::rule()` accessor.
- [x] 1.2 Add external-view compile-pass documentation for open-ended matching and compile-fail documentation for direct variant construction.

## 2. Fold the inline rule representation

- [x] 2.1 Add the default-off `strict_external` field to `ConfineInlineSymbolPath`, remove the hidden twin variant, and update the builder modifier to mutate the single payload.
- [x] 2.2 Collapse label, polarity, text, JSON, validation, and scan dispatch matches onto the single inline variant while preserving reaction and identity parity.

## 3. Compatibility evidence

- [x] 3.1 Update and run guibiao tests proving default/strict projections, constitution errors, local precedence, reaction coverage, and no baseline re-keying.
- [x] 3.2 Verify rustdoc external-surface examples and check pacta and modou against the locally patched crates without downstream source changes.

## 4. Documentation and gates

- [x] 4.1 Update the 0.2 model-surface decision/backlog and API documentation without changing any Cargo package version or lockfile dependency graph.
- [x] 4.2 Run OpenSpec strict validation and the full repository Definition of Done, including self-law projection and example reactions.
