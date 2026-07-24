## Context

Adopters and Tianheng's own self-governance test suite manually invoke `check_constitution(&constitution, &manifest_path)` and hand-roll assertions for:
1. Status clean vs. violation.
2. 100% workspace member governance coverage (`check_and_cover`).
3. Markdown projection freshness against `AGENTS.md` / `AGENTS.self-law.md` with `BLESS=1` support.
4. Fixture negative testing (verifying rules bite).

This creates ~50+ lines of repetitive boilerplate across tests. Providing `GovernanceTest` as a fluent builder in `crates/tianheng/src/testing.rs` standardizes testing DX while keeping Rust as the single source of truth.

## Goals / Non-Goals

**Goals:**
- Provide a fluent `GovernanceTest` struct in `tianheng::testing` exposed via the `tianheng` crate facade.
- Support `assert_clean()`, `assert_all_workspace_members_covered()`, `assert_projection_fresh(path)` with `BLESS=1` regeneration, and `assert_violates_fixture(fixture_path)`.
- Refactor `crates/tianheng/tests/self_governance.rs` to dogfood `GovernanceTest`.
- Gracefully handle packaged `.crate` tarballs and CI environments using `TIANHENG_WORKSPACE_TESTS`.

**Non-Goals:**
- Creating a Low-Code / YAML / GUI testing framework (Tianheng strictly enforces Rust code as the SSOT).
- Adding `testing` to sub-crates (`guibiao`, `hunyi`, `louke`) which would violate crate dependency boundaries.

## Decisions

### Decision 1: Place `GovernanceTest` in `crates/tianheng/src/testing.rs`
- **Rationale**: `tianheng` is the imperative shell and facade. Side-effects (filesystem access, env vars `CARGO_MANIFEST_DIR`/`BLESS`/`TIANHENG_WORKSPACE_TESTS`, projection rendering) belong in the shell facade, not in sub-crates.
- **Alternatives Considered**:
  - *Sub-crate `xuanji` / `guibiao`*: Rejected because sub-crates must remain light with zero filesystem side-effects and no dependency on upper-level shell.

### Decision 2: `BLESS=1` Auto-Regeneration for Projections
- **Rationale**: Mirroring Tianheng's own `self_law_projection_is_fresh` pattern. When `BLESS=1` is set in the environment, `.assert_projection_fresh(path)` overwrites the file with updated rendered Markdown; otherwise, it asserts exact string equality.

### Decision 3: Non-Vacuous Workspace Coverage Check
- **Rationale**: `.assert_all_workspace_members_covered()` ensures workspace member count is > 0 and `coverage.uncovered` is empty. This prevents typos in targets from passing tests vacuously.

## Risks / Trade-offs

- **[Risk] Monorepo Sub-directory Execution** → Running `cargo test` in a sub-package directory sets `CARGO_MANIFEST_DIR` to the package, not the workspace root.
  - *Mitigation*: `GovernanceTest` searches upwards for `Cargo.toml` carrying `[workspace]` using `xingbiao` metadata primitives when workspace-level checks are requested.
- **[Risk] Packaged Crate Failures** → `cargo test` inside a published `.crate` tarball lacks a workspace root.
  - *Mitigation*: Gracefully skip workspace checks unless `TIANHENG_WORKSPACE_TESTS=1` is explicitly set (matching `self_governance.rs`).
