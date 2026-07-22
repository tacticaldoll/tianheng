## ADDED Requirements

### Requirement: Visibility violations identify the declared item structurally

A visibility violation SHALL encode item kind, module, owner, and item name where observed under a
structured visibility rule key. Rendered visibility/item text and declaration order SHALL NOT enter
identity.

#### Scenario: Same-named items on different owners stay distinct
- **WHEN** two violating public items share a name but differ by module or owner
- **THEN** their structured facts remain distinct
