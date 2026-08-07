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

The exit contract SHALL bind **every** path out of the reaction, including a failure nobody anticipated.
A command that fails without its own handling SHALL surface as cannot-judge naming where it failed, never as
the failing utility's own status: a status outside `0`/`1`/`2` is one the contract does not define, so a
consumer cannot act on it and an operator is given no reason. Holding this per-command is not equivalent to
holding it structurally — the paths that break the contract are the ones nobody thought to wrap.

The reaction's **package enumeration** SHALL come from tracked content like every other read, and SHALL be
refused rather than judged when it fails: a directory listing that emits some entries and then fails leaves a
short list that reads as authoritative, and every citation in a package the reaction never enumerated is then
reported as one the harness does not register.

An **enumeration of the observation source that fails** SHALL be a cannot-judge, never an empty result.
The reaction reads what it judges through `git ls-files`, and a failed enumeration returns exactly what a
repository holding nothing returns, so the two MUST be told apart by the enumeration's exit status,
checked where the reaction can act on it rather than inside a subshell whose status reaches no one. The
directions this forecloses are not one: an empty census list reports clean over a document it never read,
while an empty tracker or citation list refuses every bound in the register and blames the register for a
`git` failure. A tracked path the worktree does not hold SHALL be refused on the same ground and before
the projection is written, since a tree the reaction could only partly read cannot produce a whole
register.

The repository argument SHALL be resolved to one stable physical root before any scan, directory transition,
or projection access. A relative and an absolute spelling of the same repository SHALL judge and regenerate
the same projection; entering the repository for the tracked-Markdown census SHALL NOT make later paths relative
to that repository a second time.

Before scanning tracked Markdown for a written census, the reaction SHALL enter the judged repository in a
separately checked step. Failure to enter SHALL exit 2 cannot-judge and SHALL NOT be interpreted as grep's ordinary
exit 1 no-match result.

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

#### Scenario: Relative and absolute repository paths share one projection root

- **WHEN** the gate is invoked from a repository's parent with a relative path to a register carrying tracked Markdown
- **THEN** it judges and regenerates the same projection as an absolute invocation, without looking beneath a second copy of the relative repository path

#### Scenario: The repository disappears before the written-census scan

- **WHEN** tracked Markdown enumeration succeeds and the judged repository cannot then be entered for the census scan
- **THEN** the reaction exits 2 naming the directory transition, rather than reporting that no census was written

#### Scenario: A tracked spec absent from the worktree is refused before the projection is written

- **WHEN** a spec file `git ls-files` lists is absent from the worktree, with other spec files still
  readable
- **THEN** the reaction reports that it cannot judge, naming the absent spec, and writes no projection —
  a partial tree would otherwise produce a projection describing a partial register while agreeing with
  the verdicts drawn from the same partial read

#### Scenario: An unanticipated failure still reports within the exit contract

- **WHEN** a command the reaction runs fails with no handling of its own — a text utility reading a spec, a
  temp file that cannot be created
- **THEN** the reaction reports that it cannot judge, naming where the failure occurred, and exits `2`
  rather than the failing utility's status, because a status the contract does not define is one no consumer
  can act on and no operator can read

#### Scenario: A partial package enumeration is refused, not judged

- **WHEN** the enumeration of the workspace's packages emits some entries and then fails
- **THEN** the reaction reports that it cannot judge rather than building its harness index from the short
  list, which would report every citation in an unenumerated package as one the harness does not register
