## Why

`xuanji` (璇璣) is the shared reaction model crate of Tianheng. Its implementation currently resides in a single 1,200+ line `lib.rs` file combining domain models, findings, violations, baseline snapshots, JSON formatting, and tests. To prepare the 0.2.x line for wrap-up, this change modularizes `xuanji` into single-responsibility internal submodules, performs code doc noise reduction, and confirms precise alignment with existing OpenSpec specifications without altering public API surfaces or baseline identity wire formats.

## What Changes

- **Modularize `xuanji` internals**: Extract `lib.rs` contents into `model.rs` (enums/outcomes), `finding.rs` (`FindingKey`, `Finding`), `violation.rs` (`Violation`, `Report`), `baseline.rs` (`ViolationId`, `BaselineEntry`, `Baseline`, `apply_baseline`), `util.rs` (`pretty_json`), and `tests.rs`.
- **Maintain 100% backward compatibility**: Re-export all public types and functions from `lib.rs` via `pub use`, preserving every existing import path for adopters and sibling crates.
- **Code doc noise reduction**: Clean up verbose or historical doc comments into high-signal, forward-looking shape descriptions.
- **Zero dependency impact**: Keep `serde_json` as the sole dependency.

## Capabilities

### New Capabilities
None.

### Modified Capabilities
None. (This is a non-breaking internal refactoring and documentation noise reduction; spec requirements remain intact.)

## Impact

- `crates/xuanji/src/`: File layout refactored into internal submodules.
- Public API and wire format: Completely unchanged.
