# governance-dogfood Specification

## Purpose

Keep Tianheng's published boundary families exercised through genuine self-governance and
adopter-shaped examples without turning tutorials into exhaustive fixtures or inventing fake law.

## Subject

- `crates/shengmo/**/*.rs`
- `examples/**/*.rs`
- `examples/**/Cargo.toml`

## Requirements
### Requirement: Public boundary families have adopter-shaped reaction coverage

Every boundary family the composed shell publishes SHALL be named by at least one adopter-shaped reaction —
an isolated example workspace, or this repository's own self-law. **Both sides of that comparison SHALL be
derived, neither written down.** The families SHALL be the boundary types `crates/tianheng/src/lib.rs`
re-exports, and the owners SHALL be the tracked files under `examples/` and `crates/shengmo/` naming such a
type. The reaction SHALL fail in **both** directions: a published family no owner names, and an owned type
the shell does not publish.

**No inventory, and the requirement used to ask for one.** It named thirteen families in prose, anchored to
the `0.2.x` surface. A literal beside an enumerator is the shape this repository removes on sight: a family
added to the shell keeps its old answer in the list and nobody re-examines it, which is the failure the
inventory was for. One derivation from the shell's own re-exports replaces it, and the version anchor goes
with it — a derived set has no release to be anchored to.

Two of the thirteen were **profiles, not families**, and are not covered here.
`Constitution::sans_io_pure`'s own documentation states it is "a convenience over declaring the two
boundaries by hand; it adds no new reaction", and `no_existential_leak` is the same shape. A profile is a
bundle, so the boundaries it bundles cover it by construction; counting it as a family would have this
reaction assert coverage of a reaction that does not exist.

The requirement's third clause — that the self-governance suite dogfood `tianheng::testing::GovernanceTest` —
is **dropped rather than fixed**. That harness executes clean-reaction, workspace-member-coverage and
projection-freshness assertions; none of them is family coverage, so it was an unrelated obligation carried
inside this requirement and satisfied by nothing.

What is **not** claimed: that an owner exercises its family well. This holds that a family is reachable from
something an adopter can read and run, which is what makes a family losing its owner visible; whether the
owner's assertion is a good one belongs to the owner's own test.

#### Scenario: A published family no owner names

- **WHEN** the composed shell re-exports a boundary type that no tracked file under `examples/` or
  `crates/shengmo/` names
- **THEN** the dogfood reaction fails and names that family
- **PINNED-BY** `every_published_family_has_an_adopter_shaped_owner`

#### Scenario: An owned type the shell does not publish

- **WHEN** an example workspace or the self-law names a boundary type the composed shell does not re-export
- **THEN** the dogfood reaction fails and names it, because coverage of a surface no adopter reaches is not
  coverage of the published family set
- **PINNED-BY** `every_published_family_has_an_adopter_shaped_owner`

#### Scenario: Neither side of the comparison is empty

- **WHEN** the shell's re-exports or the owner corpus yields nothing
- **THEN** the reaction fails on the derivation itself rather than reporting every family owned, because two
  empty sets agree

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

#### Scenario: A silently-dropped patch fails the gate

- **WHEN** an example's committed family requirement no longer accepts the execution-time
  `patch.crates-io` override (e.g. a local family version bump the requirement does not satisfy),
  and Cargo silently falls back to resolving that family crate from crates.io instead
- **THEN** the examples gate fails loud, naming the exact example and crate, rather than passing
  its quality and reaction checks against the stale published crate as if the local patch had
  applied

### Requirement: Every repository example has a fulfilled reaction owner

The repository examples gate SHALL derive its executable example inventory from every immediate
child of `examples/` that contains a `Cargo.toml`. Each inventoried example SHALL be marked fulfilled
only after that workspace's required quality checks and declared Tianheng reaction assertions
complete successfully. The gate SHALL fail when an inventoried example has no fulfilled owner or
when the driver claims an example name absent from the live inventory. This example-workspace
inventory SHALL remain independent of the published boundary-family inventory.

**What this requirement no longer says, and why.** It used to require three "focused matrix" commands to sit
as one contiguous ordered sequence before a positive driver, with DoD coherence enforcing that source shape
and the driver naming no matrix basename. Those were four separate shell scripts, and the shell-to-Rust
migration consolidated them into one Rust test — `crates/shengmo/tests/examples_suite.rs` — which owns its
ordering internally, checked by the compiler rather than by grepping one document for another's basenames.
There is no longer a sequence of separate commands to order, and no separate driver that could recurse into a
matrix. The decision was taken and recorded when the consolidation landed; it did not reach this file for two
windows.

**Nothing catches this class, and saying so is more honest than naming a guard that does not.** A first draft
of this paragraph claimed the subject-resolution direction would have caught it. It would not:
`capability_subjects::declaration_offences` already refuses a subject glob matching no tracked path, and it
was green throughout — this capability's globs resolve perfectly well while a requirement under them
described a mechanism that had been deleted. What found it was reading each shell file the migration removed
and asking which requirement it had implemented, which is a judgement over prose, not a reaction.

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

### Requirement: Self-law projection freshness dogfoods the public harness

Tianheng's repository self-law freshness test SHALL execute projection comparison and regeneration
through `tianheng::testing::GovernanceTest`, including the fixed universal preamble, rather than
calling only the lower-level projection helper.

#### Scenario: Self-law uses the adopter-shaped projection gate

- **WHEN** the repository self-governance test suite verifies `AGENTS.self-law.md`
- **THEN** it invokes the public harness projection-freshness surface with the same BLESS semantics taught to adopters

### Requirement: False-negative closure reaction fixtures

The repository SHALL maintain isolated test fixtures under `crates/tianheng/tests/fixtures/` and integrated example checks for transparent macro unstripping (`cfg_if!`) and ancestor glob hazard reactions. The test harness SHALL assert that a `cfg_if!`-wrapped violation and an ancestor glob hazard violation both react with an enforced exit code 1 when checked through the shell facade.

#### Scenario: Transparent macro violation fixture reacts with exit 1

- **WHEN** `tianheng check` runs against the `cfg_if_violation` fixture manifest
- **THEN** the runner exits with status 1 and reports the structured module violation enclosed in `cfg_if!`

#### Scenario: Glob hazard violation fixture reacts with exit 1

- **WHEN** `tianheng check` runs against the `glob_hazard_violation` fixture manifest
- **THEN** the runner exits with status 1 and reports the structured Glob Hazard violation

### Requirement: Mid-path relative import dogfood coverage

The repository unit and integration test suites SHALL include lock assertions verifying that mid-path `super` and `self` imports inside grouped `use` trees and inline submodules are normalized to canonical `crate::...` paths and trigger module boundary violations when targeting forbidden subtrees.

#### Scenario: Mid-path super grouped import triggers module boundary violation

- **WHEN** the test harness evaluates a module boundary against source containing `use crate::a::b::{super::forbidden::X};`
- **THEN** the harness detects `crate::a::forbidden` and reports the expected violation

### Requirement: The example suite's declared set SHALL equal the tracked example directories

The dogfood suite SHALL hold its declared example list against the tracked contents of `examples/`, in both
directions. An example present on disk and absent from the list is exercised by **neither** of the suite's
directions nor by the workflow job that runs them, which is a false negative in the gate that runs the product
against itself — the one gate whose silence is least likely to be questioned.

The enumeration SHALL come from tracked content rather than the working directory, so an untracked scratch
directory neither fails the reaction nor is mistaken for an example.

#### Scenario: An example is added and not declared

- **WHEN** a directory under `examples/` carries a manifest and no entry in the declared list names it
- **THEN** the reaction fails, naming the directory, because it would otherwise be exercised by nothing

#### Scenario: A declared example no longer exists

- **WHEN** the declared list names a directory the tracked tree does not carry
- **THEN** the reaction fails, naming the entry, because a declaration that outlived its subject reads as
  coverage while defending nothing
