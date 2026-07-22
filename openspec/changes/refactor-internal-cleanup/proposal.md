## Why

Following the modularization of `xuanji`, `guibiao`, `hunyi`/`louke`, and `xingbiao` in 0.2.3, two categories of internal noise remain: `louke/src/lib.rs` was not modularized alongside its siblings, and the `tianheng` runner's text-projection layer repeats an identical render skeleton eight times across separate functions that could share a single private helper. A small duplication in `runner/projection/document.rs` mirrors the same class of drift. All three are purely internal; no public API changes.

## What Changes

- **`louke/src/lib.rs` modularization**: extract `FoldHasher` + `TidMap` into `tracked.rs`, the declaration DSL (`RuntimeBoundary`, `RuntimeSeamDraft`, `RuntimeBoundaryDraft`, `OriginEntry`, `Posture`) into `dsl.rs`, and the registry + runtime reaction machinery (`Seam`, `OriginInfo`, `Registry`, `install`, `check_crossing`, `__react`, `set_sink`, `emit`) into `registry.rs`. Re-export all public items at `lib.rs` root — public API paths unchanged.
- **`tianheng` text projection helper**: introduce `ModuleBlockSpec` struct and a private `render_section` in `runner/projection/text.rs`. Each of the eight `*_text` functions maps its boundary slice into `Vec<ModuleBlockSpec>` then delegates to `render_section`; the duplicated skeleton disappears. The private `module_block` fn is removed (its logic inlined into `render_section`). `runtime_text` stays separate (extra `posture:` field). `hunyi` and `louke` public APIs are not touched.
- **`tianheng` document projection dedup**: introduce a private `shape_boundary_json` helper in `runner/projection/document.rs` shared by `dyn_trait_boundary_json` and `impl_trait_boundary_json`, whose bodies are currently identical modulo the rule constant.

## Capabilities

### New Capabilities
_(none)_

### Modified Capabilities
- `internal-refactoring`: extend the non-breaking internal refactoring requirement to cover `louke` modularization and `tianheng` runner projection cleanup, under the same compatibility invariant (public API, exports, wire formats, and self-governance boundaries must be unchanged).

## Impact

- `crates/louke/src/`: new files `tracked.rs`, `dsl.rs`, `registry.rs`; `lib.rs` shrinks to re-exports + constants + macros.
- `crates/tianheng/src/runner/projection/text.rs`: new `ModuleBlockSpec` struct + `render_section` fn; 8 `*_text` fns reduced to one-liners; `module_block` fn removed.
- `crates/tianheng/src/runner/projection/document.rs`: new private `shape_boundary_json` helper.
- No public API changes in any crate. No manifest version changes. No self-governance boundary changes.
