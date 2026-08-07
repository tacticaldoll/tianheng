## MODIFIED Requirements

### Requirement: Every repository example has a fulfilled reaction owner

The repository examples gate SHALL derive its executable example inventory from every immediate
child of `examples/` that contains a `Cargo.toml`. Each inventoried example SHALL be marked fulfilled
only after that workspace's required quality checks and declared Tianheng reaction assertions
complete successfully. The gate SHALL fail when an inventoried example has no fulfilled owner or
when the driver claims an example name absent from the live inventory. This example-workspace
inventory SHALL remain independent of the published boundary-family inventory.

The focused matrices SHALL remain separate top-level Definition of Done gates. These are the matrices for
published-family coverage, example ownership and artifact cleanup, and isolated-example quality. In both the
local Definition of Done and CI's authored command streams, those three commands SHALL form one contiguous
ordered sequence immediately followed by the positive repository example driver. The DoD-coherence reaction
SHALL read and enforce that source shape.

The positive driver's executable shell text SHALL NOT directly name any focused matrix basename. The reaction
SHALL ignore full-line shell comments and reject such a basename on every other line; this is an authored-form
constraint and does not claim to resolve a command name assembled dynamically at runtime.

#### Scenario: Every live example is exercised

- **WHEN** the examples gate completes against the repository's current example directories
- **THEN** every immediate example workspace has completed its declared quality and reaction path

#### Scenario: Focused refusals precede the positive driver without direct nested reruns

- **WHEN** the repository Definition of Done exercises example dogfood
- **THEN** the local and CI command streams carry the focused matrices contiguously in their declared order
  immediately before the positive driver, and the driver's executable lines name none of those matrix basenames

#### Scenario: A focused command is reordered

- **WHEN** either authored command stream moves one focused matrix after another matrix or the positive driver
- **THEN** DoD coherence fails and names the command stream whose required contiguous sequence is absent

#### Scenario: The positive driver directly reruns a focused matrix

- **WHEN** a non-comment line in the positive driver names a focused matrix basename
- **THEN** DoD coherence fails and names both the driver and nested matrix

#### Scenario: An unowned example fails loud

- **WHEN** an immediate example workspace exists but the driver never fulfills its owner
- **THEN** the examples gate fails and names the unfulfilled example directory

#### Scenario: A nonexistent example claim fails loud

- **WHEN** the driver claims completion for a name absent from the live example inventory
- **THEN** the examples gate fails and names the unknown example

#### Scenario: Example and family completeness remain orthogonal

- **WHEN** one example fulfills several published families or two examples exercise overlapping
  families
- **THEN** example completeness counts executed workspaces while family completeness independently
  counts the reviewed public family identities
