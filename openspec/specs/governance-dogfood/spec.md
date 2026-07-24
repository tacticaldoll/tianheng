# governance-dogfood Specification

## Purpose

Keep Tianheng's published boundary families exercised through genuine self-governance and
adopter-shaped examples without turning tutorials into exhaustive fixtures or inventing fake law.
## Requirements
### Requirement: Public boundary families have adopter-shaped reaction coverage

The repository SHALL maintain an executable, reviewable inventory mapping the published 0.2.x
`Constitution` boundary-family set to at least one repository-owned self-governance or isolated
example reaction. That set is: static crate and module boundaries; signature, trait-impl,
visibility, forbidden-marker, dyn-trait, impl-trait, async-exposure, and unsafe semantic
boundaries; the `sans_io_pure` and `no_existential_leak` composed profiles; and runtime boundaries.
A family SHALL count as fulfilled only after its owner executes the real evaluator against source
or Cargo metadata and observes its declared structured reaction; construction, projection, an exit
code that another family could cause, or a free-standing coverage claim SHALL NOT count. The
dogfood gate SHALL fail when any inventoried family has no fulfilled owner or an owner claims a
family absent from the inventory. The self-governance suite SHALL dogfood `tianheng::testing::GovernanceTest`
to execute reaction, workspace member coverage, and projection freshness assertions.

#### Scenario: Every public family has a fulfilled reaction owner

- **WHEN** the dogfood suite runs against the current public `Constitution` surface via `tianheng::testing::GovernanceTest`
- **THEN** every inventoried boundary family has at least one owner whose real evaluator and
  structured reaction assertions completed successfully

#### Scenario: Missing family ownership fails the gate

- **WHEN** an inventoried published family has no successfully fulfilled reaction owner
- **THEN** the dogfood gate fails and names the missing family identity

### Requirement: Breadth stays separate from teaching examples

The repository SHALL exercise boundary families without a genuine home in Tianheng's self-law or an existing focused example
in one isolated capability-catalog workspace. The catalog SHALL identify itself
as contract coverage rather than an architecture recommendation. Existing standalone, composed
funnel, sans-I/O, and unsafe-confinement examples SHALL retain their focused narratives.

#### Scenario: Catalog breadth does not overload the funnel

- **WHEN** missing boundary families are added to adopter-shaped dogfood
- **THEN** they live in the capability catalog while the composed example continues to demonstrate only the staged three-instrument funnel and its existing contract axes

### Requirement: Dogfood assertions preserve presentation freedom

Dogfood SHALL identify expected reactions through structured boundary kind, validated `RuleKey`,
dimension-owned `StructuredFactIdentity`, and declared reason or anchor where needed. It SHALL NOT pin an entire
JSON report, ANSI output, or human finding sentence. The examples script SHALL execute the catalog
through the public shell in addition to its library-level structured assertions.

#### Scenario: Wording polish does not invalidate capability coverage

- **WHEN** human finding wording or terminal styling changes without changing structured identity
- **THEN** the capability dogfood remains green while a missing or miswired structured reaction fails

#### Scenario: The real shell retains every catalog family

- **WHEN** the examples script runs the capability catalog through Tianheng's check command
- **THEN** its structured output contains the expected family identities and the declared exit class

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

### Requirement: Dogfood reacts to semantic identity schemas

Tianheng's governance dogfood SHALL exercise production-emitted target, rule key, and structured
fact roles for every shipped dimension. It SHALL pin semantic identifiers and identity-bearing
fields without pinning human presentation or whole report documents.

#### Scenario: A schema drifts silently
- **WHEN** a fact/rule identity field or canonical value changes without an explicit catalog update
- **THEN** the dogfood compatibility reaction fails

#### Scenario: Presentation changes freely
- **WHEN** only rule/finding wording or diagnostics change
- **THEN** the identity dogfood remains green

