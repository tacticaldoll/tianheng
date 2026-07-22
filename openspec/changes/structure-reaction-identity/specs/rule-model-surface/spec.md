## MODIFIED Requirements

### Requirement: Boundary builders own public rule construction

The public `Rule` and `ModuleRule` enums SHALL remain readable model types, but every data-carrying
variant SHALL remain non-exhaustive. External consumers SHALL construct rules through the existing
boundary DSL. Each builder-produced rule SHALL expose a validated semantic `RuleKey` for reaction
identity and a separate human-readable presentation for projection. External consumers SHALL NOT
directly construct or mutate either a rule variant or its key.

#### Scenario: Existing adopter DSL still constructs a rule

- **WHEN** an external consumer declares an existing boundary through its builder
- **THEN** the declaration compiles and produces the same law semantics with a stable rule key

#### Scenario: Direct external construction stays closed

- **WHEN** an external consumer attempts to construct a data-carrying rule variant or mutate its key
- **THEN** compilation fails and the consumer must use the boundary DSL

### Requirement: Rule variants remain forward-compatibly inspectable

`Rule` and `ModuleRule` and their existing variant names SHALL remain public, and boundary `rule()`
accessors SHALL remain available. External consumers SHALL inspect known fields using open-ended
patterns. A reaction identity SHALL separately expose the rule's semantic key, so inspecting or
changing projection wording does not define identity. Every finite rule family and identity-bearing
parameter SHALL be cataloged; a parameter SHALL enter the key exactly when changing it changes what
the boundary permits or forbids.

#### Scenario: An external consumer matches a known field

- **WHEN** a consumer obtains a builder-produced rule and matches a known field with `..`
- **THEN** the match compiles without assuming the complete representation

#### Scenario: Presentation-only rule changes preserve identity

- **WHEN** only a rule's displayed wording or parameter formatting changes
- **THEN** the semantic rule key and existing baseline match remain unchanged

#### Scenario: A changed law has a changed key

- **WHEN** an identity-bearing parameter changes the allowed or forbidden set
- **THEN** the rule key differs and an old baseline cannot suppress the materially changed law

#### Scenario: A new rule family requires classification

- **WHEN** a new finite rule variant or identity-bearing parameter is added without a catalog entry
- **THEN** the rule compatibility reaction fails to compile or fails its test
