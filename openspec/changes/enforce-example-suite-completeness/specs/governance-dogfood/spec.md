## ADDED Requirements

### Requirement: Every repository example has a fulfilled reaction owner

The repository examples gate SHALL derive its executable example inventory from every immediate
child of `examples/` that contains a `Cargo.toml`. Each inventoried example SHALL be marked fulfilled
only after that workspace's required quality checks and declared Tianheng reaction assertions
complete successfully. The gate SHALL fail when an inventoried example has no fulfilled owner or
when the driver claims an example name absent from the live inventory. This example-workspace
inventory SHALL remain independent of the published boundary-family inventory.

#### Scenario: Every live example is exercised

- **WHEN** the examples gate completes against the repository's current example directories
- **THEN** every immediate example workspace has completed its declared quality and reaction path

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

### Requirement: Example-run artifacts are invocation-isolated

The repository examples gate SHALL write its temporary machine projections, command output, and
generated baseline beneath one invocation-local temporary directory and SHALL remove that directory
on every exit. It SHALL NOT use fixed shared `/tmp` output paths whose contents can collide across
concurrent runs.

#### Scenario: Concurrent runs do not share artifacts

- **WHEN** two examples-gate invocations run concurrently on one host
- **THEN** each invocation reads and writes only its own temporary artifact directory

#### Scenario: Failure still cleans temporary artifacts

- **WHEN** any quality or reaction assertion terminates the examples gate early
- **THEN** the invocation-local artifact directory is removed by the exit cleanup
