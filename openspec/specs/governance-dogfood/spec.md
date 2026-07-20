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
boundaries; the `sans_io_pure` composed profile; and runtime boundaries. A family SHALL count as
fulfilled only after its owner executes the real evaluator against source or Cargo metadata and
observes its declared structured reaction; construction, projection, an exit code that another
family could cause, or a free-standing coverage claim SHALL NOT count. The dogfood gate SHALL fail
when any inventoried family has no fulfilled owner or an owner claims a family absent from the
inventory.

#### Scenario: Every public family has a fulfilled reaction owner

- **WHEN** the dogfood suite runs against the current public `Constitution` surface
- **THEN** every inventoried boundary family has at least one owner whose real evaluator and
  structured reaction assertions completed successfully

#### Scenario: Missing family ownership fails the gate

- **WHEN** an inventoried published family has no successfully fulfilled reaction owner
- **THEN** the dogfood gate fails and names the missing family identity

#### Scenario: Unknown ownership claims fail the gate

- **WHEN** an example or self-governance owner claims a family identity absent from the inventory
- **THEN** the dogfood gate fails and names the unknown family identity

#### Scenario: The inventory does not classify future methods automatically

- **WHEN** a future change proposes another public boundary-family insertion path
- **THEN** its OpenSpec and API review deliberately classifies it as a family, depth, modifier, or
  shorthand, and only a new family extends the executable inventory

### Requirement: Breadth stays separate from teaching examples

The repository SHALL exercise boundary families without a genuine home in Tianheng's self-law or an existing focused example
in one isolated capability-catalog workspace. The catalog SHALL identify itself
as contract coverage rather than an architecture recommendation. Existing standalone, composed
funnel, sans-I/O, and unsafe-confinement examples SHALL retain their focused narratives.

#### Scenario: Catalog breadth does not overload the funnel

- **WHEN** missing boundary families are added to adopter-shaped dogfood
- **THEN** they live in the capability catalog while the composed example continues to demonstrate only the staged three-instrument funnel and its existing contract axes

### Requirement: Dogfood assertions preserve presentation freedom

Dogfood SHALL identify expected reactions through structured boundary kind, stable rule identity,
dimension-owned `FindingKey`, and declared reason or anchor where needed. It SHALL NOT pin an entire
JSON report, ANSI output, or human finding sentence. The examples script SHALL execute the catalog
through the public shell in addition to its library-level structured assertions.

#### Scenario: Wording polish does not invalidate capability coverage

- **WHEN** human finding wording or terminal styling changes without changing structured identity
- **THEN** the capability dogfood remains green while a missing or miswired structured reaction fails

#### Scenario: The real shell retains every catalog family

- **WHEN** the examples script runs the capability catalog through Tianheng's check command
- **THEN** its structured output contains the expected family identities and the declared exit class
