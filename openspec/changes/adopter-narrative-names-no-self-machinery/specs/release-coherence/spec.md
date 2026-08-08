# release-coherence

## ADDED Requirements

### Requirement: Adopter narrative SHALL NOT name this repository's own machinery

An entry under an adopter-facing heading of the `[Unreleased]` section SHALL NOT name this repository's
own machinery: a path token under `scripts/`, or a bare basename that `git ls-files scripts/` resolves
to a tracked file there.

`CHANGELOG.md` is the adopter's document, and every heading it offers — `### Added`, `### Changed`,
`### Fixed`, `### Migration` — is an adopter's vocabulary. It offers no heading that is not, so every
change to this repository's own governance machinery has been written into one of them. Measured: eleven
entries whose subject is a gate, in a directory that ships in **zero** packages.

The `[Unreleased]` section SHALL be permitted a `### Self-governance` heading,
under which naming that machinery is what belongs; a heading is adopter-facing when it is any `### `
heading **other than** that one, so a heading nobody anticipated is adopter-facing rather than exempt.

The basename form SHALL be decided against the enumerator and never against a list of gate names written
beside it. A hand-kept list lets a new script be added and never measured, which is the register's own
prohibition rather than a stylistic call.

This sits on the **decidable** side of the line this capability already draws for itself: a path citation
is a reference, and reference resolution over `CHANGELOG.md` is already mechanical. Whether an entry's
*subject* is adopter-facing is a judgement over prose — the instrument `AGENTS.md` records as designed,
measured three times and rejected — and is declared as a bound below rather than approximated.

What the rule forces is a **rewrite**, not a move, wherever the adopter-relevant fact is genuinely
present. A publish-provenance entry states the guarantee an adopter gets; naming the gate file that
enforces it is the leak. If a fact matters to an adopter, state the fact.

#### Scenario: An adopter heading names a gate

- **WHEN** an entry under `### Fixed` in `[Unreleased]` names a path under `scripts/`
- **THEN** the reaction fails, naming the section, the heading and the path

#### Scenario: The same entry under the self-governance heading

- **WHEN** that entry moves under `### Self-governance` in the same section
- **THEN** the reaction is clean, so the refusal above is about the heading it sat under rather than
  about the path being named at all

#### Scenario: A bare basename the enumerator resolves

- **WHEN** an entry under an adopter-facing heading names `check_pin_bites.sh` with no directory
- **THEN** the reaction fails, because `git ls-files scripts/` resolves that basename to a tracked file

#### Scenario: A bare basename the enumerator does not resolve

- **WHEN** an entry under an adopter-facing heading names `check_something_that_does_not_exist.sh`
- **THEN** the reaction is clean, so the rule is held to the enumerator rather than to the `check_`
  prefix

#### Scenario: A dated release section names a gate — a stated bound

- **WHEN** a dated `## [X.Y.Z] - DATE` section carries an entry naming a path under `scripts/`
- **THEN** nothing reacts. A dated section records what was true at that release, and rewriting it to
  satisfy a rule written afterwards would falsify the record — the same reason `docs/history/` is left
  alone. The blindness is the scope, deliberately chosen and pinned rather than inferred
- **PINNED-BY** `a_dated_section_naming_a_gate_is_a_stated_bound`

#### Scenario: A gate named as bare prose — a stated bound

- **WHEN** an entry under an adopter-facing heading names a gate as ordinary prose rather than as a
  backticked token
- **THEN** nothing reacts, and the leak the rule exists to stop passes unseen. Widening to a bare
  substring would fire on any sentence carrying the characters, trading a declared blindness for an
  undeclared false-positive surface
- **PINNED-BY** `a_gate_named_as_bare_prose_is_a_stated_bound`

#### Scenario: Machinery the judged repository tracks by nothing — a stated bound

- **WHEN** an entry under an adopter-facing heading names a file under `scripts/` that exists in the
  worktree and in no commit
- **THEN** nothing reacts. The enumeration is `git ls-files scripts/`, so an untracked `scripts/` reads
  as absent; closing this means judging worktree content, which this repository's gates are held not to
  do — the larger error, so the blindness is declared instead
- **PINNED-BY** `machinery_tracked_by_nothing_is_a_stated_bound`

#### Scenario: An entry about self-governance that names no machinery — a stated bound

- **WHEN** an entry under an adopter-facing heading describes this repository's own governance without
  naming any path under `scripts/`
- **THEN** nothing reacts. Reaching it needs a judgement over the entry's subject rather than over its
  references, and that instrument is the one this repository measured three times and rejected;
  widening the matcher toward it — heading keywords, phrase lists — would trade a declared, bounded
  blindness for an undeclared false-positive surface
- **UNPINNED** `BACKLOG.md` — *the self-governance residual is a judgement over an entry's subject*

#### Scenario: The enumeration cannot be read

- **WHEN** `git ls-files scripts/` fails rather than returning nothing
- **THEN** the reaction refuses to judge, because a failed read is not an empty result and treating it
  as one reports a verdict over content that was never read

#### Scenario: A repository tracking no machinery at all

- **WHEN** the enumeration succeeds and names no file, and an entry under an adopter-facing heading
  names a path under `scripts/`
- **THEN** the reaction is clean, because a repository tracking no machinery has nothing an entry could
  leak — and it SHALL reach that verdict by having nothing to match. Keying the parser on the record
  number rather than on the input file makes an empty enumeration consume the changelog itself, after
  which the section vacuity guard refuses a document the reaction never read
