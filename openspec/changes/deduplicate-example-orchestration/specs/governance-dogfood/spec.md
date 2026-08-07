## MODIFIED Requirements

### Requirement: Every repository example has a fulfilled reaction owner

The focused matrices SHALL remain separate top-level Definition of Done gates. These are the matrices for
published-family coverage, example ownership and artifact cleanup, and isolated-example quality. The top-level
orchestration SHALL run them before the positive repository example driver, and that driver SHALL NOT recursively
invoke those matrices.

#### Scenario: Focused refusals precede the positive driver without nested reruns

- **WHEN** the repository Definition of Done exercises example dogfood
- **THEN** it runs each focused failure matrix directly before the positive example driver, and the driver does
  not invoke those matrices again
