## MODIFIED Requirements

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
family absent from the inventory.

#### Scenario: Every public family has a fulfilled reaction owner

- **WHEN** the dogfood suite runs against the current public `Constitution` surface
- **THEN** every inventoried boundary family has at least one owner whose real evaluator and
  structured reaction assertions completed successfully

#### Scenario: Missing family ownership fails the gate

- **WHEN** an inventoried published family has no successfully fulfilled reaction owner
- **THEN** the dogfood gate fails and names the missing family identity
