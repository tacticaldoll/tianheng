## MODIFIED Requirements

### Requirement: Every generated document and the reaction holding it fresh SHALL correspond, in both directions

Each enumerated document SHALL name the Rust unit that generates it. The correspondence is counted per
blessing call site in the Rust tests of the member holding this repository's governance, which is where those
call sites live once the apparatus is outside every published package.

The requirement names a location because the count is over call sites, and a count over a directory that no
longer holds them would report zero and read as a register with nothing left to hold.

#### Scenario: A reaction holds a projection no document registers
- **WHEN** a Rust test unit holds a projection fresh and no enumerated document names it
- **THEN** the reaction fails, naming the unit and the path it blesses
