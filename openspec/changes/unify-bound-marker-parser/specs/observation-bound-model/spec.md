## MODIFIED Requirements

### Requirement: The specs' declarations and the code's SHALL be held in bijection

The typed bound-model gate SHALL assert that the set of bound ids declared in `openspec/specs/*/spec.md` equals the set declared in code and SHALL name every unmatched id. Its spec-side set SHALL be enumerated with the observation-bound register's canonical marker predicate; it SHALL NOT carry a second marker implementation. Slug derivation SHALL remain independently implemented and compared with the register projection.

#### Scenario: The model consumes the register's declaration grammar

- **WHEN** a heading is accepted or rejected by the canonical bound-marker predicate
- **THEN** the model makes the same membership decision by calling that predicate, while independently deriving the accepted heading's id

#### Scenario: A widened model-only marker would fail the bijection

- **WHEN** the model is perturbed to admit a plural near-miss that the canonical predicate rejects
- **THEN** the model gate fails because it invents a spec declaration with no typed counterpart
