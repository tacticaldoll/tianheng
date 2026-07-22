## ADDED Requirements

### Requirement: Module violations de-duplicate by semantic identity

Module-boundary findings SHALL de-duplicate by governed target, structured rule key, and structured
observed fact. Source file, rendered import text, and traversal order SHALL NOT affect identity.

#### Scenario: Repeated imports remain one fact
- **WHEN** the same governed module observes the same violating import in multiple files or lines
- **THEN** it emits one identity while a structurally different import remains distinct
