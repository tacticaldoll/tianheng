## MODIFIED Requirements

### Requirement: A self-governance reaction SHALL be a Rust test that has been seen to fail

Every reaction judging this repository SHALL be a `#[test]` living **outside every published package**, and
every refusal it holds SHALL have been run against a tree carrying the shape it refuses, with that failure
recorded in the change that introduced it.

Shipping in zero packages is what this capability already gives as the criterion separating governance from
product — the reason `scripts/` and `docs/` count as governance. Measured before this change, the reactions
themselves failed it: `cargo package --list -p tianheng` carried all 50 files under `tests/`, so every
reaction judging this repository's changelog, specs, scripts and documents reached every adopter, where it
could only detect no workspace and return.

Outside every published package is a floor, not the whole answer: it says where a reaction must **not** live
and nothing about where it belongs. Reactions SHALL therefore be held apart by **what they judge** — the law
this repository declares over itself and the reactions that run the delivered product against this workspace
in one member, the reactions that collate its record against itself in another. Measured when only the floor
was applied: 13 of 17 targets in a member whose stated identity was the law judged neither a product contract
nor an architecture, which is the dilution the move set out to end.

The location is not cosmetic. A repository's own law living under a published package's `tests/` lends its
name to everything beside it, and a governance document came to state that twenty reactions reaching no
shipped API "run Tianheng's own reactions against the workspace". Position is what makes the two populations
separable at all.

A Rust test's failure mode is asserted **inline** — the expected value sits beside the observation — so a
reaction needs no separate failure matrix to be defended. That is what the twin obligation bought when a gate
was a shell script and its refusal was an exit code, and it is why retiring the pairing loses no coverage.

#### Scenario: A reaction inside a published package

- **WHEN** a reaction judging this repository lives under a package that `cargo publish` would ship
- **THEN** it reaches adopters who cannot run it, and it is filed as governance while its location makes it
  product — the two answers this criterion exists to keep from disagreeing

#### Scenario: The packaged self-test's subject

- **WHEN** the packaged crate's tests are run from its tarball
- **THEN** what runs exercises the packaged code, rather than governance reactions detecting no workspace and
  returning — a skip proves a skip is real, and a tarball of mostly skips proves little else
