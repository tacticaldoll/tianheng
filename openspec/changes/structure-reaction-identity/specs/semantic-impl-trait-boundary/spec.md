## ADDED Requirements

### Requirement: Impl-trait facts preserve shape and seam separately

Impl-trait violations SHALL encode the canonical forbidden shape/subject and public seam as
separate fact roles under a structured rule key. Rendered `impl ...` presentation and traversal
position SHALL NOT enter identity.

#### Scenario: The same shape at two seams stays distinct
- **WHEN** one impl-trait shape is exposed at two public seams
- **THEN** their structured seam roles produce distinct identities
