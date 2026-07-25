## Context

`guibiao` is Tianheng's static observation core. To maintain zero heavy dependencies (`serde_json`-only) and sub-millisecond scanning speeds, `guibiao` avoids AST parsers (`syn`) and macro expanders. Instead, `strip_macro_bodies` strips macro definitions (`macro_rules!`) and macro invocations (`ident! { ... }`).

However, two false-negative gaps exist under this static approach:
1. **Transparent Control-Flow Macros (`cfg_if!`)**: Developers frequently wrap conditional `use` and `mod` statements in `cfg_if! { ... }`. Blindly stripping `cfg_if!` hides human-authored imports, causing false negatives.
2. **Ancestor Wildcard Glob Imports (`use forbidden_parent::*;`)**: If a boundary forbids `crate::a::b`, an adopter writing wildcard glob `use crate::a::*;` expands all items of `crate::a` into local scope, implicitly pulling in `crate::a::b`.

By distilling **Observation Patterns** (Transparent Wrapper Pattern and Wildcard Ancestral Containment Pattern), we can close these false-negative gaps cleanly without adding dependencies or macro expanders, and without mis-flagging plain non-glob ancestor module imports (`use crate::a;`).

## Goals / Non-Goals

**Goals:**
- Unstrip transparent control-flow macros (`cfg_if!`) in `guibiao::lexer` so `use_scan`, `reachability`, and `symbol_scan` inspect their structural contents.
- Implement Ancestor Glob Hazard detection in `guibiao::use_scan` and `module_check` so wildcard imports of a forbidden module's ancestor trigger fail-closed reactions, while plain non-glob ancestor module imports (`use crate::a;`) remain clean.
- Expand dogfood test fixtures (`cfg_if_violation`, `glob_hazard_violation`) to guarantee continuous verification.

**Non-Goals:**
- Building a macro expansion or evaluation engine in `guibiao`.
- Adding `syn` or heavy AST dependencies to `guibiao` (maintains `serde_json`-only invariant).
- Flagging plain non-glob ancestor module imports (`use crate::a;`) when child `crate::a::b` is forbidden (avoids false-positive over-reaction).

## Decisions

### Decision 1: Transparent Control-Flow Macro Unstripping Pattern
- **Rationale**: `cfg_if!` acts as a structural control-flow wrapper rather than a code-generating macro. It contains standard Rust item declarations inside `{ ... }`.
- **Implementation**: Introduce `is_transparent_macro_name(ident)` in `guibiao::lexer`. When a macro invocation name matches `cfg_if`, `strip_macro_bodies_tracked` preserves its structural body delimiters and tokens rather than replacing them with whitespace.

### Decision 2: Precise Ancestor Glob Hazard Pattern
- **Rationale**: A wildcard import `use crate::parent::*;` brings all public items of `crate::parent` into scope, including `crate::parent::forbidden_child`. A plain import `use crate::parent;` does not.
- **Implementation**: In `use_scan.rs`, return `ImportedPath { path, is_glob }`. In `module_check.rs`, for `MustNotImport`, trigger a violation if `path_within(import.path, forbidden)` OR (`import.is_glob && path_within(forbidden, import.path)`).

### Decision 3: Dedicated Dogfood Test Fixtures
- **Rationale**: Self-governance and pre-flight gates require concrete fixtures to verify that reactions bite and never rot.
- **Implementation**: Add `cfg_if_violation/` and `glob_hazard_violation/` under `crates/tianheng/tests/fixtures/` and integrate them into `self_governance.rs`.
