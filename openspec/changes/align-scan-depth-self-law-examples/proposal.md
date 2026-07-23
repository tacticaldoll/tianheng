## Why

Now that `ScanDepth` toggles (`ScanDepth::Subtree` vs `ScanDepth::Shallow`) have landed (#115), Tianheng's self-constitution (`self_governance.rs`), `COOKBOOK.md`, and standalone teaching examples (`examples/guibiao-standalone`) adopt explicit `.depth(ScanDepth::Subtree)` declarations as an imitable teaching convention. Following Tianheng's 潛移 (gravity) principle, explicitly writing `.depth(...)` in self-governance and teaching documentation reinforces scan granularity best practices for downstream agents and human developers without introducing non-reactive normative spec requirements.

## What Changes

- **Self-Law Explicit Scan Depth**: Update `crates/tianheng/tests/self_governance.rs` `must_not_call_inline` module-level boundaries to explicitly chain `.depth(ScanDepth::Subtree)` as a teaching convention.
- **Self-Law Projection Verification**: Confirm `AGENTS.self-law.md` remains fresh via un-blessed `cargo test -p tianheng self_law_projection_is_fresh` byte comparison assertion.
- **Cookbook & Teaching Dogfood Update**: Update `COOKBOOK.md` and `examples/guibiao-standalone` to explicitly declare `.depth(ScanDepth::Subtree)` on `ModuleBoundary`.
- **Backlog Ledger Update**: Move the self-governance observation depth upgrade entry in `BACKLOG.md` from `WATCH` to `BUILT / HISTORY`.

## Capabilities

### New Capabilities
<!-- None -->

### Modified Capabilities
<!-- None: Explicit scan depth is adopted as a teaching convention in self-governance, cookbook, and examples without introducing non-reactive normative requirement changes. -->

## Impact

- **Affected Code**: `crates/tianheng/tests/self_governance.rs`, `examples/guibiao-standalone/src/governance.rs`, `COOKBOOK.md`, `BACKLOG.md`.
- **APIs**: No public API changes.
- **Verification**: `cargo test --workspace --all-features`, `bash scripts/test_examples.sh`, `bash scripts/check_release_coherence.sh`.
