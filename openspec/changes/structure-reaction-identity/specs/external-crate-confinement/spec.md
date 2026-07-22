## ADDED Requirements

### Requirement: Confinement facts preserve both governed and observed roles

An external-crate-confinement violation SHALL place the confined crate in the governed target role,
the confinement law in a structured rule key, and the offending importer in a structured fact.
Those roles SHALL NOT be concatenated into presentation.

#### Scenario: Two confined crates stay distinct
- **WHEN** the same importer violates confinement for two different governed crates
- **THEN** their target roles yield distinct identities and accepting one cannot mask the other
