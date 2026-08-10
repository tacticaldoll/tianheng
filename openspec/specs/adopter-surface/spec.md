# adopter-surface Specification

## Purpose

Define the composed wildcard entrypoint and its external compilation reaction so Tianheng's
documented adoption path remains usable and semantically honest across the 0.2 line.
## Subject

- `crates/tianheng/src/lib.rs`
- `crates/tianheng/src/sans_io.rs`
- `crates/tianheng/tests/adopter_surface.rs`

## Requirements
### Requirement: The prelude is the composed adopter entrypoint

`tianheng::prelude::*` SHALL expose the existing declaration and execution surface: `Constitution`,
`CrateBoundary`, `ModuleBoundary`, `SignatureBoundary`, `TraitImplBoundary`, `VisibilityBoundary`,
`ForbiddenMarkerBoundary`, `DynTraitBoundary`, `ImplTraitBoundary`, `AsyncExposureBoundary`,
`UnsafeBoundary`, `RuntimeBoundary`, `SansIoPure`, `NoExistentialLeak`, `GovernanceTest`,
`ScanDepth`, `DependencyKind`, `SourceKind`, `VisibilityCeiling`, `Severity`, and `run`. An
external consumer SHALL be able to compose boundaries from all three instruments through one
`Constitution` without importing dimension crates.

#### Scenario: A consumer declares the composed law from one import

- **WHEN** an external crate imports only `tianheng::prelude::*` and builds static, semantic, and runtime boundaries into a `Constitution`
- **THEN** the declaration and its `run` entrypoint compile without importing `guibiao`, `hunyi`, or `louke`

#### Scenario: Builder selectors remain at the adoption entrypoint

- **WHEN** a declaration uses dependency kind, source kind, visibility ceiling, or severity selectors
- **THEN** their existing prelude names compile as part of the fluent declaration

### Requirement: The prelude supports reaction inspection

`tianheng::prelude::*` SHALL expose the existing boundary, rule, baseline, report, violation, and
Outcome inspection surface, plus the vocabulary-neutral `RuleKey` and `StructuredFactIdentity`
types used by live `ViolationId`. The obsolete public `FindingKey` SHALL be removed as an
intentional 0.3.0 break. These names SHALL form an inspection tier, not a second construction path around validated
identity or builder-owned rules. Standalone instrument APIs SHALL expose the same reaction model
without requiring the composed facade.

The public surface SHALL NOT promise a `Dimension`/`ObservedFact` plugin trait or runtime plugin
loading. Rust architecture tests MAY use the promised `GovernanceTest` harness or invoke the
existing pure standalone/composed checks and inspect structured `Outcome` values.

#### Scenario: A consumer inspects a composed reaction

- **WHEN** an external crate checks a unified `Constitution`
- **THEN** it can inspect target, rule key, structured fact, presentation, metadata, and outcome without decoding CLI text

#### Scenario: A consumer inspects a standalone reaction

- **WHEN** an external crate calls an instrument's public check directly
- **THEN** it can inspect the same vocabulary-neutral reaction identity without importing `tianheng`

#### Scenario: Rule inspection remains builder-owned

- **WHEN** an external crate inspects a builder-produced rule and its emitted reaction
- **THEN** it can read rule presentation and semantic key without directly constructing an invalid rule or identity

#### Scenario: Architecture tests use the reaction model

- **WHEN** an adopter wants a Rust test for an architectural boundary
- **THEN** it uses `GovernanceTest` or invokes an existing pure check and asserts against structured Outcome data

### Requirement: The adopter surface has an external compilation reaction

The repository SHALL compile integration-test consumers for the wildcard composed prelude and for
each standalone instrument's promised check/reaction surface. The tests SHALL name the structured
identity inspection types, the promised harness and depth selector, and type-check representative
builder/check chains including `NoExistentialLeak`. They SHALL NOT invoke CLI, filesystem, or
process side effects merely to prove API availability, and SHALL NOT imply an unimplemented plugin
protocol.

#### Scenario: A composed export is accidentally removed

- **WHEN** a promised prelude name is removed, relocated, or unusable
- **THEN** the composed external compile contract fails

#### Scenario: A standalone reaction surface drifts

- **WHEN** an instrument can no longer emit or expose the common reaction identity independently
- **THEN** its external compile contract fails

#### Scenario: Runtime behavior is outside the compile contract

- **WHEN** the compile consumer references a run or check function
- **THEN** it type-checks the signature without executing observation or presentation side effects

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

### Requirement: Every promised prelude member is named by the external compilation reaction

The composed wildcard prelude is the adopter's entrypoint, so every name it re-exports SHALL be mentioned by
the external-view integration test compiled against it. The relation SHALL be **containment, not equality**:
the reaction legitimately names root imports and its own helpers, which the prelude does not promise, and
requiring equality would refuse it for being a test. A promised member SHALL be named in whatever form its
kind admits — a type through a type assertion, a trait through a bound, a function item through its own call
shape — because requiring one form would demand either a hand-kept list per kind or a contract that cannot
name its trait at all. The promise SHALL be read from the prelude's own block rather than from any
`pub use super::{…}` in the shell, which carries several that are not the promise.

#### Scenario: A prelude addition the contract never mentions reacts

- **WHEN** the prelude promises a member that appears nowhere in the external compilation reaction
- **THEN** the repository check fails and names each unmentioned member

#### Scenario: The contract may name more than the prelude promises

- **WHEN** the reaction names an item reached by a root import rather than through the prelude
- **THEN** that is not a disagreement, because the promise is what the prelude carries

#### Scenario: An input that cannot be read is refused rather than reported clean

- **WHEN** the prelude block parses to no member, or the reaction yields no identifier at all
- **THEN** the check refuses as cannot-judge, because a promise of nothing and an unread contract both make
  every direction hold vacuously

Where the holding check's own observation stops is `repository-checks`'s to declare, since the limit belongs
to the check rather than to the promise.

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
