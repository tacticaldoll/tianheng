## Why

`guibiao` (圭表) is Tianheng's static observation core. Following the 0.2.2 module reachability, symlink, inline `mod`, and `#[path]` remap fixes, this change refactors internal helpers in `module_scan/`, performs doc comment noise reduction across `guibiao` source files, and aligns static boundary behavior against OpenSpec specifications without altering any public APIs or introducing new external dependencies.

## What Changes

- **Refactor `module_scan/` internal helpers**: Clean up verbose path resolution logic and internal utility functions in `reachability.rs`, `symbol_scan.rs`, and `use_scan.rs`.
- **Code doc noise reduction**: Update Rustdoc comments across `crates/guibiao/src/` to high-signal forward-looking descriptions of boundary invariants.
- **Spec alignment**: Confirm precise alignment of scanner behavior against static boundary specifications (`module-boundary`, `crate-source-boundary`, `crate-dependency-boundary`, `external-crate-confinement`).
- **Zero dependency & API impact**: Retain `serde_json`-only dependency footprint (alongside `xuanji` and `xingbiao`) and 100% public API compatibility.

## Capabilities

### New Capabilities
None.

### Modified Capabilities
None. (Non-breaking internal refactoring and documentation noise reduction; spec requirements remain intact.)

## Impact

- `crates/guibiao/src/`: Code cleanup, helper consolidation, and docstring noise reduction.
- Public API and wire format: Completely unchanged.
