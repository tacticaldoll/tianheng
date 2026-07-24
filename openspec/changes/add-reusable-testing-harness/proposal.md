## Why

Adopters and Tianheng's own self-governance test currently write verbose, repetitive boilerplate in `cargo test` to execute architecture reactions, check workspace coverage, verify Markdown projection freshness, and validate boundary behavior. Providing an adopter-facing, reusable `GovernanceTest` harness in the `tianheng` facade streamlnes test setup, eliminates boilerplate, and enforces non-vacuous governance testing across projects.

## What Changes

- Introduce a public `tianheng::testing` module in `crates/tianheng/src/testing.rs`, exported via the `tianheng` facade.
- Provide a `GovernanceTest` fluent builder for `cargo test` integration:
  - `.assert_clean()`: asserts 0 violations returned from `check_constitution`.
  - `.assert_all_workspace_members_covered()`: asserts 100% of workspace members are targeted by the constitution (preventing vacuous passes).
  - `.assert_projection_fresh(path)`: asserts the generated Markdown projection matches the target document, supporting `BLESS=1` auto-regeneration.
  - `.assert_violates_fixture(fixture_path)`: negative assertion helper ensuring custom boundaries bite on violating fixtures.
- Refactor Tianheng's own self-governance gate (`crates/tianheng/tests/self_governance.rs`) to consume `tianheng::testing::GovernanceTest` as its primary dogfood user.

## Capabilities

### New Capabilities

- `reusable-testing-harness`: Fluent test harness (`GovernanceTest`) in the `tianheng` facade for executing clean assertions, coverage bounds, projection freshness with `BLESS=1` support, and fixture negative assertions.

### Modified Capabilities

- `governance-dogfood`: Update self-governance testing requirements to dogfood the reusable `GovernanceTest` harness.

## Impact

- **Codebase**: New module `crates/tianheng/src/testing.rs` in the `tianheng` crate facade. Refactored `crates/tianheng/tests/self_governance.rs`.
- **APIs**: Public `tianheng::testing` module exposed in crate `tianheng`. Non-breaking additive feature.
- **Dependencies**: No external crate dependencies introduced; reuses existing `guibiao`, `xingbiao`, and `serde_json` primitives.
