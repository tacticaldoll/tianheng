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

#### Scenario: An unquoted basename in prose — a stated bound

- **WHEN** an entry under an adopter-facing heading names a gate as bare prose rather than as a token
- **THEN** nothing reacts. Recognition is by token, so widening to bare prose would match any sentence
  that happens to contain the characters; the declared blindness is narrower than the false-positive
  surface the widening would open
- **PINNED-BY** `an_unquoted_basename_in_prose_is_a_stated_bound`

#### Scenario: An entry about self-governance that names no machinery — a stated bound

- **WHEN** an entry under an adopter-facing heading describes this repository's own governance without
  naming any path under `scripts/`
- **THEN** nothing reacts. Reaching it needs a judgement over the entry's subject rather than over its
  references, and that instrument is the one this repository measured three times and rejected;
  widening the matcher toward it — heading keywords, phrase lists — would trade a declared, bounded
  blindness for an undeclared false-positive surface
- **UNPINNED** `BACKLOG.md` — *the self-governance residual is a judgement over an entry's subject*

#### Scenario: The citation shape cannot be read

- **WHEN** the enumeration of tracked files under `scripts/` cannot be read
- **THEN** the reaction refuses to judge rather than reporting every entry clean against an empty
  enumerator, which is the vacuity direction
