## MODIFIED Requirements

### Requirement: The register reaction SHALL be a local gate CI runs identically

The reaction SHALL be a script invoked from the workspace root, listed in `AGENTS.md`'s Definition of
Done and run verbatim by CI, so `check_dod_coherence.sh` binds the two. Its failure directions SHALL
each be proven by a companion test against fixtures built to trip exactly one condition — a gate over a
coverage claim that has not been observed failing is a restatement of the register, not a defence of it.

The reaction SHALL be read-only: it SHALL NOT edit a spec, declare a bound, or rewrite the projection
except when explicitly asked to regenerate it.

Regeneration SHALL be bound by the same exit contract as judgment — 0 clean, 1 violation, 2 cannot judge.
Regenerating over a register that has offenses SHALL write the projection and then **fail**, because "the
document was rewritten" and "the register it describes is valid" are different claims and one exit code
cannot carry both. A register the reaction cannot judge at all SHALL fail **before** the projection is
written, so a register whose declarations it could not find cannot leave behind a document that reads as a
complete one.

An **enumeration of the observation source that fails** SHALL be a cannot-judge, never an empty result.
The reaction reads what it judges through `git ls-files`, and a failed enumeration returns exactly what a
repository holding nothing returns, so the two MUST be told apart by the enumeration's exit status,
checked where the reaction can act on it rather than inside a subshell whose status reaches no one. The
directions this forecloses are not one: an empty census list reports clean over a document it never read,
while an empty tracker or citation list refuses every bound in the register and blames the register for a
`git` failure. A tracked path the worktree does not hold SHALL be refused on the same ground and before
the projection is written, since a tree the reaction could only partly read cannot produce a whole
register.

#### Scenario: Every failure direction is proven

- **WHEN** the companion test runs
- **THEN** each of the reaction's failure directions is exercised by its own fixture, and the passing
  direction is exercised too, so a gate that only ever refuses is not mistaken for a working one

#### Scenario: The local gate and CI cannot drift apart

- **WHEN** the gate is added to the Definition of Done
- **THEN** the identical command appears in CI, and `check_dod_coherence.sh` fails if it does not

#### Scenario: The reaction leaves the tree unchanged

- **WHEN** the gate runs against any checkout
- **THEN** the working tree, `HEAD`, and the projection are unchanged unless regeneration was explicitly
  requested

#### Scenario: Regeneration over a register that has offenses

- **WHEN** regeneration is requested and a declared bound carries no citation
- **THEN** the projection is written and the reaction still fails, naming the offense, so a successful
  rewrite is never reported as a valid register

#### Scenario: Regeneration over a register the reaction cannot judge

- **WHEN** regeneration is requested and no declared bound is parsed at all
- **THEN** the reaction reports that it cannot judge and no projection is written, so a vacuous register
  produces no document

#### Scenario: A failed tracked-file enumeration is not an empty one

- **WHEN** `git ls-files` fails while enumerating the tracked files a direction judges — the tracked
  Markdown a written census could sit in, the tracked paths a tracker could name, or the tracked Rust
  files a citation could be defined in — and the repository otherwise holds a stale census
- **THEN** the reaction reports that it cannot judge, naming the enumeration that failed, rather than
  reading the empty result as a repository holding nothing: that reading reports clean over a census it
  never examined, and refuses every tracker and citation in the register for a failure that is not the
  register's

#### Scenario: A tracked spec absent from the worktree is refused before the projection is written

- **WHEN** a spec file `git ls-files` lists is absent from the worktree, with other spec files still
  readable
- **THEN** the reaction reports that it cannot judge, naming the absent spec, and writes no projection —
  a partial tree would otherwise produce a projection describing a partial register while agreeing with
  the verdicts drawn from the same partial read
