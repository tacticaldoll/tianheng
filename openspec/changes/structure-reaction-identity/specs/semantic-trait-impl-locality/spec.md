## ADDED Requirements

### Requirement: Trait-impl locality uses structured law and fact roles

Locality violations SHALL encode the governed target, a structured locality rule key, and a fact
containing impl module, trait, and canonical self type. Rule configuration SHALL be canonically
classified as identity-bearing or presentation-only; rendered impl text SHALL NOT define identity.

#### Scenario: Two impls stay distinct
- **WHEN** two misplaced impls differ by module, trait, or self type
- **THEN** their structured facts differ in the corresponding role
