## MODIFIED Requirements

### Requirement: The gate surface SHALL be enumerated from tracked content

The reaction SHALL derive the self-governance surface from tracked Rust test targets under `crates/tianheng/tests/` replacing bash script check gates.

#### Scenario: The surface is read from tracked content
- **WHEN** the reaction runs in a checkout
- **THEN** it judges exactly the self-governance test boundaries registered in `crates/tianheng/tests/`, ensuring untracked drafts do not alter the verdict

#### Scenario: An empty enumeration fails rather than reporting clean
- **WHEN** the self-governance test enumeration yields zero registered boundaries
- **THEN** the reaction fails, saying the surface was empty, refusing vacuous pass reports
