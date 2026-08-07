## MODIFIED Requirements

### Requirement: The release tag's signature SHALL be verified, not shape-matched

The extracted signature SHALL be proven to be the exact suffix of the tag object before payload reconstruction.
A mismatch SHALL exit 2 cannot-judge; it SHALL NOT reach cryptographic verification as an exit-1 invalid signature.

#### Scenario: Extracted signature and tag object disagree

- **WHEN** Git's extracted non-empty SSH signature is not the exact suffix of the tag object read by the gate
- **THEN** the gate exits 2 because it cannot reconstruct the signed payload reliably
