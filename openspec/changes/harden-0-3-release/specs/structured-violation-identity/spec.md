## ADDED Requirements

### Requirement: Every live violation identity component is read-only
The governed target stored by `Violation` SHALL be private behind a read-only accessor, matching its
private rule key and structured fact. No external caller with mutable access to a `Violation` SHALL
be able to rewrite any component returned by `Violation::id()`.

#### Scenario: External inspection cannot mutate target identity
- **WHEN** an external consumer inspects a live violation
- **THEN** it can read the target through an accessor but cannot assign a replacement target

### Requirement: Identity migration and testing harness remain separate capabilities
The structured-identity capability SHALL NOT define or require a plugin protocol or testing DSL.
This statement SHALL NOT deny the separately specified `tianheng::testing::GovernanceTest`
capability shipped by the facade.

#### Scenario: Release documentation describes both capabilities
- **WHEN** adopter-facing release notes summarize the identity migration and reusable testing harness
- **THEN** they state that identity introduces no plugin protocol while accurately listing the testing harness

