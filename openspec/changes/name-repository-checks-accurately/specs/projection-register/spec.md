## MODIFIED Requirements

### Requirement: Every generated document and the check holding it fresh SHALL correspond, in both directions

Every generated document SHALL name the repository check that holds it fresh, and every registered freshness check SHALL hold a generated document. The generated register SHALL call these holders **checks**, not reactions, because they are unpublished Rust gates over repository projections rather than product boundary behavior. The existing two-way correspondence, duplicate refusal, and fail-loud enumeration behavior SHALL remain unchanged.

#### Scenario: A generated document has no freshness check

- **WHEN** a generated document is registered without a repository check that compares it to its source
- **THEN** the projection-register gate fails and names the unheld document

#### Scenario: Generated prose assigns product vocabulary to a freshness gate

- **WHEN** the projection register is generated
- **THEN** it labels the Rust holders as checks and never teaches that projection freshness is a product reaction
