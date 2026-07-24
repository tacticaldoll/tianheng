# reusable-testing-harness Specification

## Purpose

Provide a reusable, fluent `GovernanceTest` harness in the `tianheng` facade to streamline architecture testing in `cargo test`.

## Requirements

### Requirement: GovernanceTest fluent harness executes clean reactions

The `tianheng::testing` module SHALL export a `GovernanceTest` struct that wraps a `Constitution` and executes reaction checks against the local workspace manifest.

#### Scenario: Clean outcome assertion succeeds
- **WHEN** `GovernanceTest::for_constitution(constitution).assert_clean()` is invoked on a clean workspace
- **THEN** the test passes without panic

#### Scenario: Violating outcome assertion panics with report details
- **WHEN** `GovernanceTest::for_constitution(constitution).assert_clean()` is invoked on a violating workspace
- **THEN** the test panics and formats the violation report in the panic message

### Requirement: GovernanceTest enforces workspace member coverage

`GovernanceTest` SHALL provide an `assert_all_workspace_members_covered()` method that verifies every member crate in the workspace is targeted by at least one boundary in the constitution.

#### Scenario: All workspace members covered
- **WHEN** all workspace members are targeted by constitution boundaries
- **THEN** `assert_all_workspace_members_covered()` passes

#### Scenario: Uncovered workspace member fails loud
- **WHEN** a workspace member is not targeted by any boundary
- **THEN** `assert_all_workspace_members_covered()` panics and names the uncovered member crate

#### Scenario: Vacuous zero-member read fails loud
- **WHEN** the workspace metadata resolves zero members
- **THEN** `assert_all_workspace_members_covered()` panics rather than passing vacuously

### Requirement: GovernanceTest enforces Markdown projection freshness with BLESS support

`GovernanceTest` SHALL provide an `assert_projection_fresh(path)` method that compares the generated Markdown projection of the constitution against a target file path.

#### Scenario: Fresh projection passes
- **WHEN** the target file contents match the generated constitution Markdown
- **THEN** `assert_projection_fresh` passes

#### Scenario: Stale projection fails with instructions
- **WHEN** the target file contents differ from the generated constitution Markdown and `BLESS` environment variable is unset
- **THEN** `assert_projection_fresh` panics and instructs running with `BLESS=1`

#### Scenario: BLESS=1 updates stale projection
- **WHEN** `BLESS=1` (or `BLESS=true`) is present in the environment and the target file is stale
- **THEN** `assert_projection_fresh` overwrites the target file with the updated rendered Markdown and passes

### Requirement: GovernanceTest provides fixture negative testing

`GovernanceTest` SHALL provide `test_fixture(fixture_manifest_path)` and `assert_violates_fixture(fixture_manifest_path)` methods to assert that custom boundaries produce expected violations on violating fixture workspaces.

#### Scenario: Fixture violation assertion succeeds
- **WHEN** a custom boundary produces expected violations on a violating fixture via `test_fixture`
- **THEN** fixture assertion completes successfully

#### Scenario: Assert violates fixture alias succeeds
- **WHEN** a custom boundary produces expected violations on a violating fixture via `assert_violates_fixture`
- **THEN** fixture assertion completes successfully
