## ADDED Requirements

### Requirement: Source-policy reactions use semantic identity

Every source-policy violation SHALL identify its governed target, structured rule key (including
identity-bearing policy roles), and dimension-owned source fact. Presentation and policy rendering
SHALL remain outside identity.

#### Scenario: Source presentation changes without re-keying
- **WHEN** only the displayed source-policy wording changes
- **THEN** an existing target/rule/fact baseline match remains valid
