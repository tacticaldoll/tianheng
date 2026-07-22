## MODIFIED Requirements

### Requirement: A finding has stable structured identity and human presentation

The shared reaction model SHALL represent an observed finding as both human-readable presentation
and a validated `StructuredFactIdentity`. The identity SHALL contain a non-empty semantic fact type,
a non-empty semantic shape, and zero or more uniquely named scalar string fields in canonical name
order. A semantic identifier SHALL name enduring meaning rather than a revision ordinal. Construction
SHALL reject empty identifiers/field names and duplicate field names, and SHALL NOT admit arbitrary
recursive values. Storage SHALL be private behind validated construction and read-only accessors.

#### Scenario: Presentation changes without changing fact identity

- **WHEN** a dimension renders the same observed fact with improved human wording or diagnostics
- **THEN** its presentation may change while its structured fact and violation identity remain unchanged

#### Scenario: Distinct facts carry distinct identities

- **WHEN** two observations differ in any identity-bearing observed value
- **THEN** their semantic type, shape, or named field values differ, so accepting one cannot suppress the other

#### Scenario: An ambiguous identity is rejected

- **WHEN** a caller supplies an empty type/shape/field name, duplicate field name, or recursive value
- **THEN** construction reports an error rather than normalizing or overwriting the ambiguous input

#### Scenario: A semantic identifier is not a generation number

- **WHEN** another fact family or compatible diagnostic field is added
- **THEN** existing identifiers remain unchanged and no global v3/v4 identity generation is introduced

### Requirement: Observation dimensions own fact meaning and rendering

Each observation dimension SHALL own the typed fact schemas it can observe and the conversion from
each fact to its structured identity and human presentation. The shared reaction crate SHALL own
only the dimension-agnostic envelope and SHALL NOT contain crate-, module-, semantic-, or runtime-
specific fact vocabulary. A dimension SHALL derive identity and presentation from the same typed
fact conversion. Separately observed identity-bearing components SHALL remain fact-specific named
fields rather than being concatenated into an opaque display string. Each dimension SHALL remain
usable without `tianheng` or another observation dimension.

#### Scenario: A dimension introduces a new fact shape

- **WHEN** an observation dimension begins reacting to a new kind of fact
- **THEN** its schema and rendering are added in that dimension while `xuanji` and other dimensions remain unchanged

#### Scenario: Shared identity does not reverse the dependency graph

- **WHEN** all dimension fact schemas compile with the shared reaction model
- **THEN** every observation dimension depends inward on `xuanji`, while `xuanji` depends on no observation dimension

#### Scenario: An instrument emits an independently inspectable reaction

- **WHEN** an adopter invokes 圭表, 渾儀, or 漏刻 directly
- **THEN** its Outcome exposes the same vocabulary-neutral structured identities used by the composed facade

### Requirement: Published structured identity schemas are compatibility-reacted

Every observation dimension SHALL carry an explicit compatibility reaction for every shipped fact
family and every finite typed discriminator affecting semantic type, shape, canonical field names,
or field values. Each dimension SHALL inspect at least one violation produced through its real
boundary reaction to pin target, rule key, and fact as separate identity roles. Adding a fact or
finite discriminator SHALL require an explicit catalog decision. Reactions SHALL NOT freeze human
presentation, complete report JSON, or diagnostic metadata.

Compatibility SHALL additionally be proved behaviorally: reordering declarations or inserting an
unrelated item SHALL NOT change existing identities, and distinct observed facts SHALL remain
distinct across cfg branches and unrenderable syntax. No public identity field or fallback SHALL be
derived solely from traversal position, ordinal, or collection index. A syntax-ban catalog MAY
supplement these tests but SHALL NOT replace them.

#### Scenario: Every shipped dimension fact is cataloged

- **WHEN** compatibility tests run across 圭表, 渾儀, and 漏刻 with all features
- **THEN** every fact and finite discriminator has exact expected semantic identifiers, named fields, and representative values

#### Scenario: Reordering observations preserves identity

- **WHEN** declarations are reordered or an unrelated declaration is inserted before an observed fact
- **THEN** the fact retains the same identity and baseline match

#### Scenario: Distinct unrenderable facts stay distinct

- **WHEN** two distinct facts contain syntax that cannot use the ordinary canonical renderer
- **THEN** an observed structural discriminator keeps them distinct or observation fails loud, never assigning the same positional fallback

#### Scenario: Presentation remains free to change

- **WHEN** only human wording or non-identity diagnostics change
- **THEN** compatibility reactions and baseline identity remain unchanged

### Requirement: Violations are constructed from typed identity

The public model SHALL construct a `ViolationId` from a governed target, validated semantic
`RuleKey`, and `StructuredFactIdentity`, and SHALL construct a `Violation` by attaching presentation,
boundary kind, reason, severity, and diagnostics. External callers SHALL NOT construct an id by
struct literal or mutate any identity component. `ViolationId` equality and ordering SHALL use only
the target, rule key, and fact identity. Human rule/finding presentation, reason, severity, file,
anchor, polarity, complete signature diagnostics, baseline status, owner, and tracker SHALL NOT
enter identity.

#### Scenario: A dimension emits a violation through typed identity

- **WHEN** a dimension converts an observed fact into a violation
- **THEN** it supplies the three typed identity roles rather than adjacent presentation strings

#### Scenario: Metadata and wording do not re-identify a violation

- **WHEN** only presentation, reason, severity, location, anchor, polarity, diagnostics, or annotations change
- **THEN** the new `ViolationId` compares equal to the prior identity

#### Scenario: A materially different rule is a different identity

- **WHEN** a rule parameter changes what the boundary permits or forbids for the same target and fact
- **THEN** its semantic rule key differs, so the old baseline does not suppress the new law

#### Scenario: Identity provenance cannot be forged

- **WHEN** an external caller constructs or inspects a live identity
- **THEN** validated constructors require all three roles and public access is read-only

### Requirement: Existing adopter-facing reaction entry points remain available

The adopter-written `Constitution` and boundary builders SHALL retain their existing names and
roles, as SHALL `tianheng::run`, the standalone instrument checks, and the composed pure check. The
public reaction types (`Baseline`, `BaselineEntry`, `ViolationId`, `Violation`, `Report`, and
`Outcome`) SHALL remain available with the intentional 0.3.0 identity shape changes. This
capability SHALL NOT introduce a public dimension/plugin trait or testing assertion DSL.

#### Scenario: Standalone and composed consumers inspect the same model

- **WHEN** an adopter calls a standalone instrument or `tianheng::check_constitution`
- **THEN** both return inspectable Outcomes using the same structured reaction identities
