## ADDED Requirements

### Requirement: Shipped prelude additions are explicit compile-reacted promises

The composed wildcard prelude SHALL expose `NoExistentialLeak`, `ScanDepth`, and `GovernanceTest`
alongside the existing adopter surface. The external compilation reaction SHALL name each type and
type-check `Constitution::no_existential_leak(...)` without executing workspace observation.
`GovernanceTest` SHALL be the promised architecture-test harness; older prose denying any testing
harness promise MUST NOT remain in this capability.

#### Scenario: Composed existential profile is compile-reacted

- **WHEN** an external-view integration test imports only `tianheng::prelude::*`
- **THEN** it can name `NoExistentialLeak` and build a constitution through
  `.no_existential_leak(...)`

#### Scenario: Harness and depth selector are compile-reacted

- **WHEN** the wildcard prelude contract is compiled
- **THEN** `GovernanceTest` and `ScanDepth` resolve as public types
