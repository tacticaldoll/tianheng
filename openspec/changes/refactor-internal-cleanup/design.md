## Context

`release/0.2.3` modularized `xuanji`, `xingbiao`, `guibiao`, and `hunyi`/`louke` (doc cleanup only for the last two), plus cleaned docstring noise in `guibiao`. Two categories of internal noise remain:

1. **`louke/src/lib.rs`** (674 lines) was not split alongside its siblings. It mixes four distinct concerns: the `Tracked` supertrait + `FoldHasher` runtime machinery, the declaration DSL, the write-once registry + reaction runtime, and the constant/macro public surface.
2. **`tianheng` runner projection**: `text.rs` repeats an identical `if-empty / text_section / for / push_str block / anchor_line` skeleton across eight `*_text` functions; `document.rs` has two near-identical `dyn_trait_boundary_json` / `impl_trait_boundary_json` bodies differing only in the rule constant.

## Goals / Non-Goals

**Goals:**
- Modularize `louke/src/lib.rs` into three internal submodules without changing any public path.
- Eliminate the repeated render skeleton in `text.rs` via a private `ModuleBlockSpec` struct + `render_section` helper.
- Eliminate the duplicated body in `document.rs` via a private `shape_boundary_json` helper.
- Full DoD green: clippy, fmt, tests, self-governance, release-coherence, examples.

**Non-Goals:**
- Any public API change in `louke`, `hunyi`, or any other crate.
- Adding traits or public items to `hunyi` for `tianheng`'s benefit (preserves hunyi's independent-product space).
- Moving `merge_outcomes` to `xuanji` (it is a shell-composition detail, not a model primitive).
- Changing `runtime_text` (its extra `posture:` field makes it a permanent exception to the shared skeleton).

## Decisions

**D1: `ModuleBlockSpec` struct over a trait for text projection**

A private trait `ModuleBoundaryText` with 8 impl blocks would carry 24 lines of mechanical forwarding boilerplate (`severity_str/reason/anchor`) with no semantic content — the trait's open-polymorphism benefit is unused because `render_section` is a private helper with no external callers. `ModuleBlockSpec` is a plain value struct: the conversion from each boundary type (the `.map()` closure) holds the actual logic, the shared skeleton (`render_section`) is written once, and there is zero forwarding boilerplate. Projection is a data-transformation concern, not a polymorphic-dispatch concern.

**D2: Trait stays out of `hunyi`**

Adding a `Boundary` supertrait to `hunyi`'s public API for `tianheng`'s projection benefit would (a) expand `hunyi`'s public API surface in a crate that may grow as an independent product, and (b) constitute a non-breaking but unnecessary API addition in a 0.2.x patch. The projection concern belongs in `tianheng`'s runner layer.

**D3: `louke` submodule split mirrors `xuanji`'s pattern**

`xuanji` split into `model.rs / finding.rs / violation.rs / baseline.rs / util.rs / tests.rs` with `pub use` re-exports at the crate root. `louke` follows the same pattern: `tracked.rs / dsl.rs / registry.rs`, re-exporting everything public at `lib.rs`. `lib.rs` retains: the crate doc, `pub use` re-exports, `RUNTIME_SEAM_RULE`, `runtime_seam_rule_line`, `assert_boundary!`, `register_origin!`, and the `#[cfg(feature = "audit")]` gate.

**D4: `module_block` fn removed**

After introducing `render_section`, the private `module_block` helper in `text.rs` is no longer called. Its logic is inlined into `render_section`'s format string, parameterized via `ModuleBlockSpec::target_text` (which encodes "module X in Y", "trait X in Y", "subtree X in Y", "crate X" per boundary type). Dead code removal is correct; no caller outside `text.rs` used it.

## Risks / Trade-offs

- [`louke` macro hygiene] → `assert_boundary!` and `register_origin!` reference `$crate::__react` and other private items via `$crate::` paths. Moving those items to submodules and re-exporting them at `lib.rs` root preserves the `$crate::` path — verify that each macro-referenced symbol is in the re-export list before removing it from `lib.rs`.
- [`render_section format string`] → The `module_block` helper's existing format (`\n[{sev}] module {m} in {k}\n  rule:   {r}\n  reason: {rs}\n`) must be reproduced exactly in `render_section` (with `target_text` replacing the `"module {m} in {k}"` fragment) to keep byte-identical output for adopters using `list --format text`. Verify with the existing runner tests.
