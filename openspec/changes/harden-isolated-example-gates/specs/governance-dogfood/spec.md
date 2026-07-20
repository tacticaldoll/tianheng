## ADDED Requirements

### Requirement: Isolated examples pass repository quality gates

Every repository-owned isolated example workspace SHALL pass format checking, Clippy over all of
its targets with warnings denied, and rustdoc with warnings denied before its declared Tianheng
reaction is accepted by the examples gate. Clippy and rustdoc SHALL resolve the in-development
Tianheng family through the same execution-time local patches as the example's tests while the
committed manifest retains its adopter-facing dependency form. A deliberate Tianheng boundary
violation SHALL remain executable scan data and SHALL NOT exempt the surrounding Rust target or
reaction test from these quality gates.

#### Scenario: Every isolated workspace is quality checked

- **WHEN** the repository examples gate runs
- **THEN** each isolated example workspace passes format, all-target Clippy, and rustdoc checks
  before its reaction owner is considered successful

#### Scenario: A warning fails before reaction acceptance

- **WHEN** an isolated example target introduces a Clippy or rustdoc warning
- **THEN** the examples gate fails even if that example would still produce its expected Tianheng
  exit code or structured violation

#### Scenario: Local quality checks preserve adopter manifests

- **WHEN** Clippy, rustdoc, and tests resolve an example against the in-development workspace
- **THEN** execution-time Cargo patches provide the local crates and no committed example
  dependency is rewritten to a path dependency

#### Scenario: Deliberate drift remains live

- **WHEN** an example passes its Rust quality checks
- **THEN** its existing Tianheng reaction test still observes the deliberately violated boundary
  rather than repairing or suppressing that architectural fault

