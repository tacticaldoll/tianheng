## MODIFIED Requirements

### Requirement: A gate a wrapper asks for SHALL be observed to have run, and the name it is asked for by SHALL be pinned

A wrapper that asks for a check by test name SHALL treat *the filter matched nothing* as a failure of the
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

**The cited identity SHALL be pinned by a check.** For every tracked shell script, each `--exact <ident>`
SHALL be joined to the `--test <target>` of the same invocation, and that target SHALL register `<ident>`
exactly once. A test identifier is a reference into this repository exactly as a path is, and the reference
gate matches paths only.

**Every tracked script SHALL carry at least one such citation, and that SHALL be held per script.** A script
citing no gate renders its own verdict, which is the shape this capability's Purpose refuses and the shape its
retired predecessor described in full: `check_*.sh` gates paired with `test_*.sh` twins over a shared shell
library, 1562 lines of it. The direction that enumerates the scripts folded every citation into one list and
asserted that **list** was non-empty, so a script contributing nothing was invisible while any sibling
contributed something — the whole way back was open, and the enumeration that would have seen it was already
running.

**The consequence is stated rather than discovered: `scripts/` becomes a closed category.** A tracked script
that is not a wrapper cannot be added there while this holds, which is what the capability already claims when
it says `git ls-files scripts/` names only wrappers. Making that claim hold is the point; a convenience script
belongs somewhere this requirement does not reach, or the requirement is amended deliberately.

**Both SHALL hold, and neither substitutes for the other.** Measured rather than reasoned: `--list` includes an
`#[ignore]`d test, so the check cannot see a silenced gate, while `--exact` on one reports `0 passed; 1
ignored` and exits `0`, so the wrapper can. The check runs where the suite runs and a wrapper is run
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
- **THEN** the pinning check fails in an ordinary run, naming the script, the identifier, and the target it
  was cited against — before any wrapper is invoked

#### Scenario: A tracked script cites no gate at all

- **WHEN** a tracked shell script carries no `--exact <ident>` citation anywhere, while its siblings do
- **THEN** the check fails naming that script, because a script that defers its verdict to nothing is rendering
  one itself; the aggregate being non-empty says only that some script cites a gate, never that this one does

#### Scenario: An invocation whose identifier cannot be bound to a target

- **WHEN** a tracked script writes `--exact <ident>` with no `--test <target>` in the same invocation
- **THEN** the check refuses as a cannot-judge naming the script and the identifier: an identifier it
  cannot bind to a target is one it could not resolve, not one it resolved as fine

#### Scenario: The script enumeration fails

- **WHEN** the tracked-script enumeration fails
- **THEN** the check refuses as a cannot-judge rather than reporting clean over an empty list, since a
  failed enumeration returns exactly what a repository holding no scripts returns

#### Scenario: A tool configuration set in the environment is not observed — a stated bound

- **WHEN** a value a sanctioned wrapper refuses as an argument is exported into its environment instead
- **THEN** the wrapper does not see it, a stated bound: the allowlist classifies **arguments**, and cargo takes
  the same configuration from the environment — measured on cargo 1.96.0, `--target not-a-real-triple` and
  `CARGO_BUILD_TARGET=not-a-real-triple` produce the identical rustc-probe failure. Closing it is ordinary work
  here rather than another layer's, since the wrapper could scrub the environment before invoking the tool; it
  needs an allowlist **over the environment**, and legitimate setups export `CARGO_HOME` and `CARGO_TARGET_DIR`,
  so which set to admit is a decision this bound records instead of guessing
- **PINNED-BY** `a_tool_configuration_set_in_the_environment_is_a_stated_bound`

#### Scenario: A gate reached without the wrapper — a stated bound

- **WHEN** someone runs `cargo publish` directly, or merges in the browser
- **THEN** no repository check fires. Both assertions guard the sanctioned path; reaching further would mean observing
  the operator's shell or GitHub's servers rather than this repository. The pinning check narrows this
  without closing it: it keeps the sanctioned path sanctioned, so what is left is choosing not to use it
  rather than using it unguarded
- **UNPINNED** `BACKLOG.md` — *a merge or publish made outside the wrapper is not observed*
