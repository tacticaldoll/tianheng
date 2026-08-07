## MODIFIED Requirements

### Requirement: The release tag's signature SHALL be verified, not shape-matched

Temporary signature material SHALL be owned by cleanup before its directory is acquired. If acquisition creates
and reports a directory before failing, the gate SHALL exit cannot-judge and SHALL remove that directory.

#### Scenario: Signature workspace acquisition fails after creating a directory

- **WHEN** temporary-workspace acquisition creates and reports its directory but returns failure
- **THEN** the gate exits `2` cannot-judge and removes the partially acquired directory
