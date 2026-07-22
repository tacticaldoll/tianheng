## MODIFIED Requirements

### Requirement: The prelude supports reaction inspection

`tianheng::prelude::*` SHALL expose the existing boundary, rule, baseline, report, violation, and
Outcome inspection surface, plus the vocabulary-neutral `RuleKey` and `StructuredFactIdentity`
types used by live `ViolationId`. The obsolete public `FindingKey` SHALL be removed as an
intentional 0.3.0 break. These names SHALL form an inspection tier, not a second construction path around validated
identity or builder-owned rules. Standalone instrument APIs SHALL expose the same reaction model
without requiring the composed facade.

The public surface SHALL NOT promise a `Dimension`/`ObservedFact` plugin trait, runtime plugin
loading, or a `tianheng::testing` assertion DSL in this change. Rust architecture tests SHALL use
the existing pure standalone/composed checks and inspect structured `Outcome` values.

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
- **THEN** it invokes an existing pure check and asserts against structured Outcome data without a new testing DSL

### Requirement: The adopter surface has an external compilation reaction

The repository SHALL compile integration-test consumers for the wildcard composed prelude and for
each standalone instrument's promised check/reaction surface. The tests SHALL name the structured
identity inspection types and type-check representative builder/check chains. They SHALL NOT invoke
CLI, filesystem, or process side effects merely to prove API availability, and SHALL NOT imply an
unimplemented plugin protocol.

#### Scenario: A composed export is accidentally removed

- **WHEN** a promised prelude name is removed, relocated, or unusable
- **THEN** the composed external compile contract fails

#### Scenario: A standalone reaction surface drifts

- **WHEN** an instrument can no longer emit or expose the common reaction identity independently
- **THEN** its external compile contract fails

#### Scenario: Runtime behavior is outside the compile contract

- **WHEN** the compile consumer references a run or check function
- **THEN** it type-checks the signature without executing observation or presentation side effects
