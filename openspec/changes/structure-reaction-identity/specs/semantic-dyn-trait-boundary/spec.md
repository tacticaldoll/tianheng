## ADDED Requirements

### Requirement: Dyn-trait facts preserve shape and seam separately

Dyn-trait violations SHALL encode the canonical forbidden shape/subject and public seam as separate
fact roles under a structured rule key. Stated renderer-granularity bounds MAY coalesce the same
subject at the same seam, but traversal position SHALL NOT be used to claim injectivity.

#### Scenario: The same shape at two seams stays distinct
- **WHEN** one dyn-trait shape is exposed at structurally different public seams
- **THEN** the seam fields produce distinct identities
