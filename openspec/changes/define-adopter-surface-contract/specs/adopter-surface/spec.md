## ADDED Requirements

### Requirement: The prelude is the composed adopter entrypoint

`tianheng::prelude::*` SHALL expose the existing declaration and execution surface: `Constitution`,
`CrateBoundary`, `ModuleBoundary`, `SemanticBoundary`, `TraitImplBoundary`, `VisibilityBoundary`,
`ForbiddenMarkerBoundary`, `DynTraitBoundary`, `ImplTraitBoundary`, `AsyncExposureBoundary`,
`UnsafeBoundary`, `RuntimeBoundary`, `SansIoPure`, `DependencyKind`, `SourceKind`,
`VisibilityCeiling`, `Severity`, and `run`. An external consumer SHALL be able to compose boundaries
from all three instruments through one `Constitution` without importing dimension crates.

#### Scenario: A consumer declares the composed law from one import

- **WHEN** an external crate imports only `tianheng::prelude::*` and builds static, semantic, and runtime boundaries into a `Constitution`
- **THEN** the declaration and its `run` entrypoint compile without importing `guibiao`, `hunyi`, or `louke`

#### Scenario: Builder selectors remain at the adoption entrypoint

- **WHEN** a declaration uses dependency kind, source kind, visibility ceiling, or severity selectors
- **THEN** their existing prelude names compile as part of the fluent declaration

### Requirement: The prelude supports reaction inspection

`tianheng::prelude::*` SHALL expose `Boundary`, `BoundaryKind`, `Rule`, `ModuleRule`, `Baseline`,
`BaselineEntry`, `Finding`, `FindingKey`, `Outcome`, `Polarity`, `Report`, `Violation`, `ViolationId`,
and the pure static `check` entrypoint. These names SHALL form an inspection tier, not a second
construction path around builder-owned rule models. The declaration/execution and inspection tiers
SHALL have the same 0.2.x compatibility status; the tier names distinguish purpose only.

#### Scenario: A consumer inspects a reaction

- **WHEN** an external crate receives an `Outcome` through a prelude entrypoint
- **THEN** it can inspect reports, violations, stable identity, finding data, polarity, boundary kind, and baseline metadata using the existing prelude names

#### Scenario: Rule inspection remains builder-owned

- **WHEN** an external crate inspects `Rule` or `ModuleRule` through a builder-produced boundary
- **THEN** the prelude provides both read-side types without enabling direct construction of a non-exhaustive rule variant

#### Scenario: Module rule inspection is symmetric

- **WHEN** an external crate imports the wildcard prelude and matches the value returned by `ModuleBoundary::rule()`
- **THEN** `ModuleRule` is nameable beside `Rule` without an additional root or dimension-crate import

### Requirement: The adopter surface has an external compilation reaction

The repository SHALL compile an integration-test consumer that imports the wildcard prelude, names
every promised declaration/execution and reaction-inspection export, and type-checks representative
builder chains for the static, semantic, runtime, and composed-profile surfaces. The test SHALL NOT
depend on source-text matching or execute process and filesystem side effects merely to prove API
availability.

#### Scenario: An export is accidentally removed

- **WHEN** a promised prelude name is removed, relocated, or made unusable from an external crate
- **THEN** the integration compile contract fails in the repository test gate

#### Scenario: Runtime behavior is outside the compile contract

- **WHEN** the adopter-surface test references `run` or a check function
- **THEN** it type-checks the callable signature without invoking CLI parsing, workspace scanning, or process output

### Requirement: Focused semantic checks remain explicit

The public `check_semantic` alias for the signature-coupling semantic check SHALL remain available
from the `tianheng` crate root rather than being added to the wildcard prelude. Adopter-facing
documentation SHALL direct composed governance to `Constitution` plus `run`, pure static inspection
to prelude `check`, and focused signature-coupling inspection to the explicit root import. The full
semantic bundle and granular `#[doc(hidden)]` semantic checks SHALL NOT be misrepresented or elevated
into this adopter-surface contract.

#### Scenario: A consumer chooses the signature-coupling check

- **WHEN** an external test needs the pure signature-coupling check
- **THEN** it imports `check_semantic` explicitly from the crate root without expanding the wildcard prelude menu or implying that one check evaluates every semantic capability
