## ADDED Requirements

### Requirement: Trait-impl exposure uses observed structural seams

Trait-impl exposure facts SHALL encode trait, canonical self type, associated item role/name, and
forbidden subject where observed. A traversal position or impl/item ordinal SHALL NOT substitute for
an unrenderable structural role.

#### Scenario: Inherent and trait-impl seams stay distinct
- **WHEN** the same subject appears in an inherent item and a trait-impl item on one self type
- **THEN** their owner/trait/item roles keep the identities distinct

#### Scenario: An unrenderable seam fails safely
- **WHEN** ordinary rendering cannot distinguish two structural seams
- **THEN** an observed discriminator separates them or scanning fails loud, never a positional fallback
