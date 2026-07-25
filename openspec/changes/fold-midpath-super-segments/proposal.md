## Why

`guibiao`'s path resolution previously normalized `self` and `super` only when they appeared as the initial path segment (`self::...` or `super::...`). When grouped imports or nested paths construct mid-path relative segments (e.g. `use crate::a::b::{super::forbidden::Thing};` or `use crate::a::{self::b::Thing};`), `guibiao` concatenated prefix and leaf without resolving mid-path `super` or `self` segments, leaving un-collapsed paths like `crate::a::b::super::forbidden::Thing`. When evaluated against a module boundary or inline symbol confinement forbidding `crate::a::forbidden`, `path_within` string prefix comparisons returned `false`, creating a false-negative gap where forbidden imports and inline symbol calls escaped detection. Folding mid-path relative segments across both import paths and symbol-scan resolvers closes this false-negative gap without adding external dependencies.

## What Changes

- **Mid-Path Segment Folding**: Introduce `fold_canonical_segments` in `guibiao::path_vocab` to fold embedded `self` and `super` segments across all module paths (e.g., `["crate", "a", "b", "super", "c"]` -> `["crate", "a", "c"]`), enforcing over-pop protection against popping past `crate`.
- **Unified Path Normalization across Scanners**: Update `guibiao::use_scan`, `reachability`, and `symbol_scan` (`resolve_head` and `resolve_written_path`) path resolvers to use `fold_canonical_segments`, ensuring grouped imports, inline submodule imports, and inline symbol call paths all resolve to fully collapsed canonical paths.
- **Spec & Lock Tests**: Update `module-boundary` and `governance-dogfood` specifications with mid-path relative import and symbol call scenarios and add comprehensive unit/integration lock tests in `guibiao::tests`.

## Capabilities

### New Capabilities

*(None)*

### Modified Capabilities

- `module-boundary`: Add requirements for mid-path `super` and `self` segment normalization in import and symbol paths.
- `governance-dogfood`: Add test harness requirements for validating mid-path relative import and symbol call boundary enforcement.

## Impact

- **`crates/guibiao`**: Updated `path_vocab.rs`, `use_scan.rs`, `symbol_scan.rs`, `reachability.rs`, and `tests.rs`. Maintains zero heavy dependencies (`serde_json`-only).
- **Pre-flight & CI**: Standard pre-flight gates (`cargo test`, `scripts/test_examples.sh`) pass cleanly with zero breaking public API changes.
