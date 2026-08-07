## MODIFIED Requirements

### Requirement: The register reaction SHALL be a local gate CI runs identically

Before scanning tracked Markdown for a written census, the reaction SHALL enter the judged repository in a
separately checked step. Failure to enter SHALL exit 2 cannot-judge and SHALL NOT be interpreted as grep's ordinary
exit 1 no-match result.

#### Scenario: The repository disappears before the written-census scan

- **WHEN** tracked Markdown enumeration succeeds and the judged repository cannot then be entered for the census scan
- **THEN** the reaction exits 2 naming the directory transition, rather than reporting that no census was written
