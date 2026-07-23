# rule-model-surface Specification

## Purpose

Define a builder-owned rule construction surface that remains forward-compatibly inspectable while
allowing rule representations to grow without parallel public variants.

## Requirements

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

### Requirement: Strict-external is one inline-rule modifier

The public builder SHALL represent `.strict_external()` as a modifier of the existing inline symbol
confinement rule rather than as a second public rule variant. The modifier SHALL preserve the
existing default-off behavior, local-precedence classification, constitution errors, human and JSON
projections, polarity, and violation identity. Adding or removing `.strict_external()` on an
otherwise identical boundary SHALL continue to leave the violation's target, rule, and finding key
unchanged.

#### Scenario: Default inline confinement stays default-off

- **WHEN** a boundary uses `must_not_call_inline(prefix)` without `.strict_external()`
- **THEN** it retains the existing default resolver behavior and omits `strict_external` from its projection

#### Scenario: The modifier preserves strict-external behavior

- **WHEN** the same builder adds `.strict_external()`
- **THEN** fully-qualified declared external paths are classified through the existing strict-external rules and the projection carries `strict_external: true`

#### Scenario: The representation fold does not re-key a violation

- **WHEN** default and strict-external forms observe a path that both already classify as the same violation
- **THEN** their target, rule, finding key, polarity, and count are identical

### Requirement: Reference consumer surfaces remain available

The guibiao check, coverage, projection, baseline, and shared-model re-export functions and types SHALL
retain their existing names. pacta's Tianheng builder/runner use and modou's guibiao
projection/baseline integration SHALL compile against the narrowed local model without source
changes.

#### Scenario: pacta and modou compile against the local crates

- **WHEN** both reference consumers are checked with their Tianheng-family dependencies patched to the local change
- **THEN** their existing builder, runner, check, coverage, projection, baseline, and type-name usage compiles unchanged

### Requirement: Boundary builders SHALL expose explicit ScanDepth toggles

The public reaction model SHALL provide a strongly-typed `ScanDepth` enum (`Shallow`, `Subtree`, `Audit`) with `#[default]` set to `Shallow`. Boundary builders SHALL expose `.depth(ScanDepth)` to allow explicit configuration of observation depth. Existing ergonomic builders (such as `.including_submodules()`) SHALL map to `.depth(ScanDepth::Subtree)` and SHALL remain fully compatible.

#### Scenario: Default boundary construction uses Shallow depth

- **WHEN** a boundary is declared without an explicit depth modifier
- **THEN** its scan depth defaults to `ScanDepth::Shallow` and existing evaluation behavior is preserved

#### Scenario: Explicit depth configuration via ScanDepth enum

- **WHEN** a boundary builder is configured with `.depth(ScanDepth::Subtree)` or `.depth(ScanDepth::Audit)`
- **THEN** the boundary retains the specified depth and evaluates matching targets accordingly

#### Scenario: Existing builder ergonomics delegate to ScanDepth

- **WHEN** an adopter calls an existing modifier like `.including_submodules()`
- **THEN** the boundary configures its depth to `ScanDepth::Subtree` without breaking caller code
