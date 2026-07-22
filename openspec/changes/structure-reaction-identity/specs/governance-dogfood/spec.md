## ADDED Requirements

### Requirement: Dogfood reacts to semantic identity schemas

Tianheng's governance dogfood SHALL exercise production-emitted target, rule key, and structured
fact roles for every shipped dimension. It SHALL pin semantic identifiers and identity-bearing
fields without pinning human presentation or whole report documents.

#### Scenario: A schema drifts silently
- **WHEN** a fact/rule identity field or canonical value changes without an explicit catalog update
- **THEN** the dogfood compatibility reaction fails

#### Scenario: Presentation changes freely
- **WHEN** only rule/finding wording or diagnostics change
- **THEN** the identity dogfood remains green
