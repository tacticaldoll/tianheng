# structured-violation-identity Specification

## Purpose

Define the shared structured identity model that separates a violation's stable observed fact from
its human presentation while keeping fact meaning inside the observation dimension that owns it.

## Requirements

### Requirement: A finding has stable structured identity and human presentation

The shared reaction model SHALL represent an observed finding as both a human-readable `finding`
string and a structured `finding_key`. The key SHALL consist of a non-empty dimension namespace, a
non-empty fact code, and zero or more uniquely named string fields in canonical name order.
Construction SHALL reject empty namespace/code/field names and duplicate field names rather than
silently create an ambiguous key. Key storage SHALL be private behind validated construction and
read-only accessors, so a caller cannot mutate a valid key into an invalid one. The key SHALL NOT
admit arbitrary recursive values.

#### Scenario: Presentation changes without changing fact identity

- **WHEN** a dimension renders the same observed fact with improved human wording
- **THEN** the `finding` string may change while its `finding_key` and violation identity remain unchanged

#### Scenario: Distinct facts carry distinct keys

- **WHEN** two findings differ in any identity-bearing observed value
- **THEN** their namespaced fact code or named key fields differ, so accepting one cannot suppress the other

#### Scenario: An ambiguous key is rejected

- **WHEN** a caller constructs a finding key with an empty namespace, empty code, empty field name, or duplicate field name
- **THEN** construction reports an error rather than normalizing or overwriting the ambiguous input

### Requirement: Observation dimensions own fact meaning and rendering

Each observation dimension SHALL own the typed fact schemas it can observe and the conversion from
each fact to its structured key and human finding text. The shared reaction crate SHALL own only the
dimension-agnostic key envelope and SHALL NOT contain crate-, module-, semantic-, or runtime-specific
fact vocabulary. A dimension SHALL derive the key and text from the same typed fact conversion so
their relationship has one implementation source.

#### Scenario: A dimension introduces a new fact shape

- **WHEN** an observation dimension begins reacting to a new kind of observed fact
- **THEN** its schema and rendering are added in that dimension while the shared key envelope remains unchanged

#### Scenario: Shared identity does not reverse the dependency graph

- **WHEN** all dimension fact schemas are compiled with the shared reaction model
- **THEN** every observation dimension depends inward on the shared model and the shared model depends on no observation dimension

### Requirement: Violations are constructed from typed identity

The public model SHALL construct a `ViolationId` from its target, rule, and typed finding, and SHALL
construct a `Violation` from its boundary kind, `ViolationId`, reason, and severity. A newly observed
id SHALL carry a structured key; only an id parsed from a version-1 baseline SHALL carry no key.
External callers SHALL NOT be able to construct an id by struct literal or mutate its key; they
SHALL read the optional key through an accessor while retaining public read access to target, rule,
and human finding. Live violation construction SHALL reject a parsed legacy id rather than admit a
current observation without a structured key.
Two structured ids SHALL compare and sort by `(target, rule, finding_key)`, two legacy ids by their
old `(target, rule, finding)` triple, and ids from different identity provenances SHALL NOT compare
equal. Human finding text, reason, severity, file, anchor, polarity, and baseline status SHALL NOT
enter structured identity. The public models SHALL continue exposing human `finding` text.

#### Scenario: A dimension emits a violation through typed identity

- **WHEN** a dimension converts an observed fact into a violation
- **THEN** it supplies a typed finding inside `ViolationId` rather than passing target, rule, and finding as adjacent positional strings to `Violation::new`

#### Scenario: Metadata and wording do not re-identify a violation

- **WHEN** only a violation's finding wording, reason, severity, file, anchor, polarity, or baseline status changes
- **THEN** its newly observed `ViolationId` compares equal to the prior identity

#### Scenario: Legacy and structured identity remain disjoint

- **WHEN** a legacy id and a structured id carry the same target, rule, and human finding
- **THEN** ordinary `ViolationId` equality reports them different, leaving cross-version text matching exclusively to baseline operations

#### Scenario: A caller cannot forge legacy provenance

- **WHEN** an external caller constructs a newly observed violation identity
- **THEN** the public constructor requires a typed finding and the caller cannot remove or replace the stored key through public fields

#### Scenario: Historical identity cannot become a live observation

- **WHEN** a caller attempts to construct a live violation from an id read from a version-1 baseline
- **THEN** construction fails loudly rather than producing a current violation or version-2 entry with a null key

### Requirement: Existing adopter-facing reaction entry points remain available

The adopter-written `Constitution` and boundary builders SHALL retain their existing names and roles,
as SHALL `tianheng::run` and guibiao's public check, coverage, projection, and baseline functions.
The public types that those functions expose (`Baseline`, `BaselineEntry`, `ViolationId`,
`Violation`, `Report`, and `Outcome`) SHALL remain available, with identity construction changing
only as required to establish the structured-key invariant. This capability SHALL NOT reshape
`Rule` or `ModuleRule`.

#### Scenario: Reference consumers use the unchanged reaction surface

- **WHEN** pacta and modou are checked against the local 0.2.0 crates after migrating any direct violation construction
- **THEN** their builder, runner, check, coverage, projection, baseline, and public re-export usage resolves through the same named entry points
