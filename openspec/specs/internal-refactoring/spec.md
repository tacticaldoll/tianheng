# internal-refactoring Specification

## Purpose
Specifies invariants and bounds for non-breaking internal refactorings across Tianheng crates.
## Requirements
### Requirement: Internal Code Structure and Docstring Refactoring
The internal refactoring of `xingbiao`, `guibiao`, `hunyi`, `louke`, and the `tianheng` runner's projection layer SHALL NOT alter any public API signatures, exported symbols, JSON wire formats, or self-governance boundaries. Specifically:

- `louke/src/lib.rs` SHALL be modularized into internal submodules (`tracked`, `dsl`, `registry`) with all public items re-exported at the crate root, preserving every public path.
- The `tianheng` runner's `text.rs` text-projection layer SHALL eliminate the duplicated render skeleton across the eight `*_text` boundary-projection functions by introducing a private `ModuleBlockSpec` struct and a single `render_section` helper. The `hunyi` and `louke` public APIs SHALL remain unchanged.
- The `tianheng` runner's `document.rs` SHALL eliminate the near-identical `dyn_trait_boundary_json` / `impl_trait_boundary_json` function bodies by introducing a private `shape_boundary_json` helper.

#### Scenario: Verification of public API compatibility and DoD suite
- **WHEN** full workspace Definition of Done checks are executed after `louke` modularization and `tianheng` projection cleanup
- **THEN** all crates build, clippy passes with zero warnings, tests pass, and self-governance law holds, with no change to any public API path or JSON wire format

