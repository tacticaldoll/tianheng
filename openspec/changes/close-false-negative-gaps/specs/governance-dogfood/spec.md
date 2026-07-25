## ADDED Requirements

### Requirement: False-negative closure reaction fixtures

The repository SHALL maintain isolated test fixtures under `crates/tianheng/tests/fixtures/` and integrated example checks for transparent macro unstripping (`cfg_if!`) and ancestor glob hazard reactions. The test harness SHALL assert that a `cfg_if!`-wrapped violation and an ancestor glob hazard violation both react with an enforced exit code 1 when checked through the harness.

#### Scenario: Transparent macro violation fixture reacts with exit 1

- **WHEN** the test harness checks the `cfg_if_violation` fixture manifest against its module boundary
- **THEN** the harness exits with status 1 and reports the structured module violation enclosed in `cfg_if!`

#### Scenario: Glob hazard violation fixture reacts with exit 1

- **WHEN** the test harness checks the `glob_hazard_violation` fixture manifest against its module boundary
- **THEN** the harness exits with status 1 and reports the structured Glob Hazard violation
