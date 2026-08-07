## MODIFIED Requirements

### Requirement: Development carries adopter-facing release narrative

Active development SHALL retain the current released workspace version while `[Unreleased]` may name the intended
release in adopter-facing narrative. The reaction SHALL judge the mutable version-bearing surfaces it enumerates;
it SHALL NOT require a version literal in `[Unreleased]` prose to equal the still-released workspace version.

#### Scenario: Intended release narrative precedes mechanical version preparation

- **WHEN** `[Unreleased]` names the intended release while workspace and example manifests retain the current
  released version
- **THEN** development coherence passes until release preparation advances the enumerated version-bearing
  surfaces together
