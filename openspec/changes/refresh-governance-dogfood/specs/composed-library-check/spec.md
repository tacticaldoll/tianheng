## ADDED Requirements

### Requirement: Tianheng self-governance uses the composed library evaluator

The repository's primary self-governance gate SHALL evaluate the live self-Constitution through
`check_constitution` against the workspace manifest. It SHALL therefore exercise the same ordered
static, semantic, and runtime-audit composition used by adopters, including the runtime audit when
the Constitution declares no runtime boundaries. Declaration-integrity assertions MAY remain
separate because their observation source is the Constitution itself rather than workspace code.

#### Scenario: The live self-law is clean through the adopter entrypoint

- **WHEN** Tianheng's self-governance test evaluates its declared Constitution
- **THEN** it calls the composed library evaluator and requires one clean Outcome for the workspace

#### Scenario: An undeclared reachable runtime probe cannot bypass self-governance

- **WHEN** a reachable production source under the workspace target roots contains a runtime probe absent from the self-Constitution
- **THEN** the self-governance gate receives the runtime audit reaction instead of passing because its runtime declaration set is empty
