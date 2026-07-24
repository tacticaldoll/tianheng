## ADDED Requirements

### Requirement: Workspace coverage follows declared crate targets across dimensions

`GovernanceTest::assert_all_workspace_members_covered()` SHALL count every workspace member named
by a static or semantic boundary's declared crate target. A runtime boundary SHALL NOT cover a
workspace member because its declared target is a seam and carries no crate identity. The harness
MUST NOT invent a crate association from source location, probe path, or seam spelling.

#### Scenario: Semantic-only crate is covered

- **WHEN** a workspace member is targeted only by one or more Hunyi semantic boundaries
- **THEN** the coverage assertion treats that member as covered

#### Scenario: Runtime seam does not invent crate coverage

- **WHEN** a workspace member has no static or semantic boundary and the constitution declares a
  runtime seam
- **THEN** the member remains uncovered because the seam declaration names no crate
