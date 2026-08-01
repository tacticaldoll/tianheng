## ADDED Requirements

### Requirement: Identity is scoped to its governing crate

Identity SHALL include the target crate whenever a boundary kind can be declared against more than
one crate in a workspace. A dimension's fact construction for any such boundary kind SHALL carry the
boundary's declared crate package as a named identity-bearing field, separate from the human-anchored
module or target path, so that two crates declaring the same governed path under the same rule
produce distinct `ViolationId`s rather than one collapsing into the other.

#### Scenario: Two crates with the identical boundary shape stay distinct

- **WHEN** two workspace members each declare a boundary of the same kind, same rule, and same
  governed module path, and each crate independently violates it
- **THEN** the composed report contains one violation per crate, each carrying its own file and
  crate-scoped identity, and neither is dropped by dedup

#### Scenario: A baseline accepted for one crate does not suppress another crate's violation

- **WHEN** a baseline records an accepted violation observed in one crate, and a different crate
  later produces a violation with the same module path, rule, and un-scoped fact fields
- **THEN** the gate treats the second crate's violation as new and unaccepted, because its
  crate-scoped identity differs from the baselined entry

#### Scenario: A dimension without multi-crate boundaries is unaffected

- **WHEN** a dimension's boundary kind can only ever be declared against a single, fixed crate
- **THEN** this requirement does not obligate that dimension to add a crate-scoped field it has no
  varying value for
