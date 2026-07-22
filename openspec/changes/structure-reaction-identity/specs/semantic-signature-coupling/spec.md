## ADDED Requirements

### Requirement: Signature exposure facts use structural seam roles

Every signature-coupling fact SHALL separately encode the forbidden subject and the public seam
roles that make the exposure distinct. Semantic member positions such as tuple-field indices MAY be
identity-bearing observations; scan order, item ordinal, and renderer fallback position SHALL NOT.

#### Scenario: Two exposed seams stay distinct
- **WHEN** the same forbidden subject appears at two public seams
- **THEN** their structured seam roles differ and accepting one does not mask the other

#### Scenario: Reordering does not alter a seam
- **WHEN** unrelated items are inserted or declarations reordered
- **THEN** pre-existing exposure identities remain unchanged
