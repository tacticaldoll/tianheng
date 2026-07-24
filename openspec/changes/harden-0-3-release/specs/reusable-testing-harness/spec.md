## ADDED Requirements

### Requirement: Projection blessing has one explicit environment interpretation
`GovernanceTest` projection freshness methods and Tianheng's own self-law caller SHALL enable
regeneration only for `BLESS=1` or case-insensitive `BLESS=true`. An unset, empty, `0`, `false`, or
other value SHALL compare and fail on stale content rather than write it.

#### Scenario: False-like BLESS value does not overwrite
- **WHEN** a projection is stale and `BLESS=0`, `BLESS=false`, or an empty BLESS value is present
- **THEN** projection freshness fails and leaves the artifact unchanged

#### Scenario: True-like BLESS value regenerates
- **WHEN** a projection is stale and `BLESS=1` or `BLESS=true` is present
- **THEN** the harness overwrites the artifact with the live projection and passes

### Requirement: Projection freshness behavior is executable
Repository tests SHALL execute the public projection freshness methods for a matching artifact, a
stale artifact without blessing, and a stale artifact with blessing. Documentation-only examples
SHALL NOT be the sole evidence for this requirement.

#### Scenario: Public freshness paths are covered
- **WHEN** the testing-harness suite runs
- **THEN** fresh, stale, and blessed projection paths execute through `GovernanceTest`

