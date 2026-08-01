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

#### Scenario: The declaring crate is an identity-bearing observed value when it can vary

- **WHEN** a boundary kind can be declared against more than one crate in a workspace, and two
  crates each declare the identical rule against the identical governed target
- **THEN** the crate each was declared against is itself an identity-bearing observed value, so the
  two observations' identities differ and one being accepted does not suppress the other — unless a
  dimension already encodes the declaring crate in the target or another identity role it uses for
  that boundary kind, in which case no additional field is needed to satisfy this scenario

#### Scenario: A boundary kind that already encodes its crate in another identity role is not double-counted

- **WHEN** a boundary kind's identity already varies by crate through its target (or another
  identity role), because the boundary is inherently crate-scoped rather than module-path-scoped
- **THEN** this requirement does not obligate that boundary kind's fact to carry the same crate a
  second time as a redundant field
