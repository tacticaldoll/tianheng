## ADDED Requirements

### Requirement: Internal Code Structure and Docstring Refactoring
The internal refactoring of `xingbiao`, `guibiao`, `hunyi`, and `louke` SHALL NOT alter any public API signatures, exported symbols, JSON wire formats, or self-governance boundaries.

#### Scenario: Verification of public API compatibility and DoD suite
- **WHEN** full workspace Definition of Done checks are executed
- **THEN** all crates build, clippy passes with zero warnings, tests pass, and self-governance law holds.
