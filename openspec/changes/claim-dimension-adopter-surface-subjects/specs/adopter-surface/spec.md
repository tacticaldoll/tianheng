## MODIFIED Requirements

### Requirement: The adopter surface has an external compilation reaction

The repository SHALL compile integration-test consumers for the wildcard composed prelude and for
each standalone instrument's promised check/reaction surface. The tests SHALL name the structured
identity inspection types, the promised harness and depth selector, and type-check representative
builder/check chains including `NoExistentialLeak`. They SHALL NOT invoke CLI, filesystem, or
process side effects merely to prove API availability, and SHALL NOT imply an unimplemented plugin
protocol.

Each standalone instrument's own promised check/reaction surface is compile-reacted by that
dimension's own `tests/adopter_surface.rs` — `crates/guibiao/tests/adopter_surface.rs`,
`crates/hunyi/tests/adopter_surface.rs`, and `crates/louke/tests/adopter_surface.rs` — alongside the
composed shell's `crates/tianheng/tests/adopter_surface.rs`. This capability's declared subject SHALL
name all four: a standalone dimension's own adoption compile check is this capability's concern by
the same argument that put the shell's file there, not a file some other capability happens to leave
unclaimed.

#### Scenario: A composed export is accidentally removed

- **WHEN** a promised prelude name is removed, relocated, or unusable
- **THEN** the composed external compile contract fails

#### Scenario: A standalone reaction surface drifts

- **WHEN** an instrument can no longer emit or expose the common reaction identity independently
- **THEN** its external compile contract fails

#### Scenario: Runtime behavior is outside the compile contract

- **WHEN** the compile consumer references a run or check function
- **THEN** it type-checks the signature without executing observation or presentation side effects

#### Scenario: A dimension's own standalone-surface test is this capability's subject

- **WHEN** a change touches `crates/guibiao/tests/adopter_surface.rs`,
  `crates/hunyi/tests/adopter_surface.rs`, or `crates/louke/tests/adopter_surface.rs`
- **THEN** it is filed under `adopter-surface`, because that dimension's own external compilation
  reaction is what this requirement obligates and this capability's declared subject names the file
