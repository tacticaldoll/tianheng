## ADDED Requirements

### Requirement: Forbidden-marker acquisitions are structured facts

A marker-acquisition violation SHALL encode acquisition form, marker, module, owner/type, and item
roles where observed under a structured rule key. Rendered acquisition text and scan position SHALL
NOT define identity.

#### Scenario: Different acquisitions stay distinct
- **WHEN** two forbidden acquisitions differ by form, marker, module, or owner
- **THEN** their structured facts differ in that observed role
