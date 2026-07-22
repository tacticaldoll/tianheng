## Why

Following recent modularization and doc comment cleanup in `xuanji`, `guibiao`, `hunyi`, and `louke`, this change modularizes `xingbiao` (星表) into submodules, audits the workspace for Twin-Drift bug patterns, and refactors internal docstrings across `guibiao`, `hunyi`, and `louke` into concise, forward-looking boundary invariant descriptions without changing public APIs or adding dependencies.

## What Changes

- **Modularize `xingbiao`**: Extract tests from `crates/xingbiao/src/lib.rs` into an independent `crates/xingbiao/src/tests.rs` module while preserving public crate root re-exports.
- **Twin-Drift & Duplication Review**: Audit cross-crate utility logic (`PathBuf` resolution, error formatting, test fixture setup helpers) to prevent twin-drift between `guibiao`, `hunyi`, `louke`, and `xingbiao`.
- **Internal Scanner Doc Comment Cleanup**: Refactor internal module docstrings in `hunyi/src/collect.rs`, `hunyi/src/scan.rs`, `louke/src/audit/scan.rs`, `guibiao/src/module_scan/symbol_scan.rs`, and `guibiao/src/module_scan/use_scan.rs` to eliminate doc noise and focus on high-signal invariant descriptions.

## Capabilities

### New Capabilities

None.

### Modified Capabilities

None.

## Impact

- Affected code: `crates/xingbiao`, `crates/guibiao`, `crates/hunyi`, `crates/louke`.
- Dependencies: Unchanged (`serde_json` + `std` for `xingbiao`; `syn` quarantined in `hunyi`).
- Compatibility: 100% backward compatible. No breaking API changes, manifest changes, or dependency additions.
