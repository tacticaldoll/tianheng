# runtime-origin-assertion (delta)

## ADDED Requirements

### Requirement: An audit finding SHALL carry no repair polarity, and that SHALL be stated

An audit finding SHALL carry **no** polarity, and the reason SHALL be recorded rather than left as an absence a
reader has to interpret. `Polarity` distinguishes a **deny breach** — repair by removing the offending code — from
an **allowlist gap** — repair by removing the code *or* by widening the declared set. The audit's findings are
neither: a declared seam with no probe is repaired by probing it or by dropping the declaration, and a probe
naming an undeclared seam by declaring it or by deleting the probe. Assigning either value would name a repair
direction that does not exist.

This is the only production emission path in the family that carries none, and the difference is by construction
elsewhere: 圭表's crate and module rules answer through **exhaustive matches returning `Polarity`**, so a new rule
variant cannot compile without declaring one, and 渾儀 carries a non-optional `Polarity` on the context every
finding is emitted through. `Violation::polarity` is therefore an `Option` for exactly this dimension's audit, and
saying so is what keeps a reader from reading the `Option` as a gap — measured, a review did read it that way.

No reaction is required for the by-construction half: an exhaustive match is a stronger guard than a test, and
adding one would be a second copy of a fact the compiler already holds.

#### Scenario: A declared seam has no probe

- **WHEN** the audit reports a declared-but-unprobed seam
- **THEN** the violation carries no polarity, because probing it and dropping the declaration are both repairs
  and neither is the direction `Polarity` names

#### Scenario: A probe names an undeclared seam

- **WHEN** the audit reports a probe against a seam the constitution does not declare
- **THEN** the violation carries no polarity, for the same reason in the mirror direction

#### Scenario: A static or semantic rule is added

- **WHEN** a new rule variant is added to a dimension whose findings carry a repair direction
- **THEN** it does not compile until it declares one, so that half of the contract needs no reaction of its own
