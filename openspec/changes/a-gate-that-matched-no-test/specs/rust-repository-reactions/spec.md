## ADDED Requirements

### Requirement: A gate a wrapper asks for SHALL be observed to have run, and the name it is asked for by SHALL be pinned

A wrapper that asks for a reaction by test name SHALL treat *the filter matched nothing* as a failure of the
wrapper, and the identifier it cites SHALL be held against the test that carries it. Being named where the run
is decided is not being run there.

**A filter matching nothing is not a clean gate.** `libtest` exits `0` when `--exact <name>` selects no test —
measured against a prebuilt binary, an unknown name reports `0 passed; 0 failed; N filtered out` and exits
`0`. Exit status alone therefore cannot separate *judged and found nothing wrong* from *judged nothing*, and
the two wrappers asking for a gate this way both stand in front of an act that cannot be undone. Each SHALL
require the run to report exactly one passing test, and SHALL surface what it saw when it does not.

**The assertion SHALL stand in the wrapper, before the irreversible command** — not inside the gate it guards.
A renamed or `#[ignore]`d test cannot report that it did not run, so a guard the disarming disables is not a
guard.

**The cited identity SHALL be pinned by a reaction.** For every tracked shell script, each `--exact <ident>`
SHALL be joined to the `--test <target>` of the same invocation, and that target SHALL register `<ident>`
exactly once. A test identifier is a reference into this repository exactly as a path is, and the reference
gate matches paths only.

**Both SHALL hold, and neither substitutes for the other.** Measured rather than reasoned: `--list` includes an
`#[ignore]`d test, so the reaction cannot see a silenced gate, while `--exact` on one reports `0 passed; 1
ignored` and exits `0`, so the wrapper can. The reaction runs where the suite runs and a wrapper is run
locally; the wrapper's assertion runs when a wrapper is invoked and a rename lands in a pull request long
before that.

#### Scenario: The gate's test no longer answers to the name the wrapper cites

- **WHEN** a wrapper asks for its gate by a name no test in the target carries — through a rename, a move, or
  an `#[ignore]`
- **THEN** the wrapper exits non-zero before the irreversible command, printing the run's output and saying
  the name in the script no longer names a test; it does not reach `cargo publish` or `gh pr merge`

#### Scenario: A gate that ran and refused, and a gate that did not run

- **WHEN** one wrapper's gate runs and reports a violation, and another's matches no test
- **THEN** both stop before the act and each says which happened; the second is not reported as a passing
  gate, which is what `libtest`'s exit status alone would say

#### Scenario: A renamed gate is red in the ordinary suite

- **WHEN** a test a tracked script names by `--exact` is renamed, moved to another `--test` target, or
  registered twice
- **THEN** the pinning reaction fails in an ordinary run, naming the script, the identifier, and the target it
  was cited against — before any wrapper is invoked

#### Scenario: An invocation whose identifier cannot be bound to a target

- **WHEN** a tracked script writes `--exact <ident>` with no `--test <target>` in the same invocation
- **THEN** the reaction refuses as a cannot-judge naming the script and the identifier: an identifier it
  cannot bind to a target is one it could not resolve, not one it resolved as fine

#### Scenario: The script enumeration fails

- **WHEN** the tracked-script enumeration fails
- **THEN** the reaction refuses as a cannot-judge rather than reporting clean over an empty list, since a
  failed enumeration returns exactly what a repository holding no scripts returns

#### Scenario: A gate reached without the wrapper — a stated bound

- **WHEN** someone runs `cargo publish` directly, or merges in the browser
- **THEN** nothing reacts. Both assertions guard the sanctioned path; reaching further would mean observing
  the operator's shell or GitHub's servers rather than this repository. The pinning reaction narrows this
  without closing it: it keeps the sanctioned path sanctioned, so what is left is choosing not to use it
  rather than using it unguarded
- **UNPINNED** `BACKLOG.md` — *a merge or publish made outside the wrapper is not observed*
