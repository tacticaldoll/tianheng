## Why

`guibiao`'s static observation core strips macro invocation bodies (`strip_macro_bodies`) to avoid executing complex macro expansion DSLs. However, transparent control-flow macros such as `cfg_if!` wrap human-authored, static `use` and `mod` declarations without transforming identifier identities; blindly stripping `cfg_if!` bodies creates false-negative gaps where architecture-governing imports go undetected. Furthermore, ancestor wildcard glob imports (`use forbidden_parent::*;`) under a `must_not_import` boundary present a potential false-negative hazard if ancestor wildcard paths are ignored. Closing these false-negative gaps ensures that static boundaries remain strict, non-bypassable backstops without compromising `guibiao`'s dependency-light (`serde_json`-only) performance.

## What Changes

- **Transparent Control-Flow Macro Unstripping**: Extend `guibiao::lexer` to recognize transparent control-flow macros (`cfg_if!`) during macro body stripping, preserving their inner structural braces so that enclosed `use` and `mod` declarations are observed by `use_scan`, `reachability`, and `symbol_scan`.
- **Ancestor Glob Hazard Reaction**: Update `guibiao::use_scan` and module boundary checks to preserve wildcard glob flags and detect when an observed wildcard glob import's base path (`crate::a`) is an ancestor of a forbidden target path (`crate::a::b`), reacting fail-closed on wildcard globs while preserving clean status for plain non-glob ancestor module imports (`use crate::a;`).
- **Dogfood Fixture Coverage**: Add new test fixtures (`cfg_if_violation`, `glob_hazard_violation`) to `crates/tianheng/tests/fixtures/` and integrate them into `self_governance.rs` to ensure continuous regression prevention.

## Capabilities

### New Capabilities

*(None)*

### Modified Capabilities

- `module-boundary`: Add requirements for transparent control-flow macro body inspection (`cfg_if!`) and ancestor glob import fail-closed hazard detection.
- `governance-dogfood`: Add test harness requirements for validating transparent macro unstripping and ancestor glob hazard reactions.

## Impact

- **`crates/guibiao`**: Updated `lexer.rs`, `use_scan.rs`, and `module_check.rs` parsing and evaluation logic; zero new dependencies (maintains `serde_json`-only invariant).
- **`crates/tianheng`**: Added test fixtures under `tests/fixtures/`.
- **Pre-flight & CI**: Standard pre-flight gates (`cargo test`, `scripts/test_examples.sh`) run new fixture checks seamlessly with zero breaking public API changes.
