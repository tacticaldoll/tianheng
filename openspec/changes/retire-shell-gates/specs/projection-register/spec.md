## MODIFIED Requirements

### Requirement: Every generated document and the reaction holding it fresh SHALL correspond, in both directions

Each enumerated document SHALL name the Rust self-governance unit that generates it. The correspondence is counted per blessing call site in Rust tests under `crates/tianheng/tests/`.

#### Scenario: A reaction holds a projection no document registers
- **WHEN** a Rust test unit holds a projection fresh and no enumerated document names it
- **THEN** the reaction fails, naming the unit and the path it blesses
