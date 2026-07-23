## MODIFIED Requirements

### Requirement: The prelude is the composed adopter entrypoint

`tianheng::prelude::*` SHALL expose the existing declaration and execution surface: `Constitution`,
`CrateBoundary`, `ModuleBoundary`, `SemanticBoundary`, `TraitImplBoundary`, `VisibilityBoundary`,
`ForbiddenMarkerBoundary`, `DynTraitBoundary`, `ImplTraitBoundary`, `AsyncExposureBoundary`,
`UnsafeBoundary`, `RuntimeBoundary`, `SansIoPure`, `NoExistentialLeak`, `DependencyKind`, `SourceKind`,
`VisibilityCeiling`, `Severity`, and `run`. An external consumer SHALL be able to compose boundaries
from all three instruments through one `Constitution` without importing dimension crates.

#### Scenario: A consumer declares the composed law from one import

- **WHEN** an external crate imports only `tianheng::prelude::*` and builds static, semantic, and runtime boundaries into a `Constitution`
- **THEN** the declaration and its `run` entrypoint compile without importing `guibiao`, `hunyi`, or `louke`

#### Scenario: Builder selectors remain at the adoption entrypoint

- **WHEN** a declaration uses dependency kind, source kind, visibility ceiling, or severity selectors
- **THEN** their existing prelude names compile as part of the fluent declaration
