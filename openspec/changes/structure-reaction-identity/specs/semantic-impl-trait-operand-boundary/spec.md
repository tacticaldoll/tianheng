## ADDED Requirements

### Requirement: Impl-trait operand facts share structured shape semantics

Operand-specific impl-trait reactions SHALL use the same structured subject/seam roles as the
shape-only rule and a distinct semantic rule key for operand policy. Presentation SHALL NOT define
their relationship or identity.

#### Scenario: Shape and operand rules do not collide
- **WHEN** the same seam violates both shape-only and operand-specific laws
- **THEN** their semantic rule keys keep the violation identities distinct
