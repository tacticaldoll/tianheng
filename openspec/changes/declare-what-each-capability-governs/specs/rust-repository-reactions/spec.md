This capability is `rust-self-governance-gates` renamed. Sync moves the existing spec directory and its whole
requirement set **verbatim** — no requirement text changes in the move — and then applies the additions below.
The rename is reviewable as a rename; anything that is not identical is a defect in the move.

## ADDED Requirements

### Requirement: A capability SHALL declare the subject it governs

Every capability spec SHALL carry a `## Subject` section between `## Purpose` and `## Requirements`, listing
the tracked-path globs it governs. A capability that does not say what it governs cannot be joined to anything,
and a requirement's home is then decided by a name read loosely — which is how a requirement about a shell
wrapper came to be filed under a capability whose subject is Rust test files.

Membership SHALL be resolved by `git ls-files -- <glob>`. Git's pathspec is both the matcher and the meaning of
*tracked*, so no glob matcher is written here: a subject is a produced set, not a text model of one.

A capability whose subject is this repository's own reactions SHALL name the members holding them rather than
a package's `tests/` directory, since the apparatus lives outside every published package.

Every declared glob SHALL match at least one tracked path. A glob matching nothing is a claim about nothing,
and it reads as coverage while providing none.

The subject SHALL NOT be assumed to tile the repository. A tracked file no capability claims is not judged by
the join below, and the reaction SHALL say so rather than imply a coverage it does not have.

#### Scenario: A capability declares no subject

- **WHEN** a capability spec carries no `## Subject` section
- **THEN** the reaction fails, naming the capability — an undeclared subject makes every filing decision about
  it unfalsifiable

#### Scenario: A declared glob matches no tracked path

- **WHEN** a `## Subject` glob resolves to no tracked file
- **THEN** the reaction fails, naming the capability and the glob

#### Scenario: The tracked-path enumeration fails

- **WHEN** `git ls-files` fails while resolving a subject
- **THEN** the reaction refuses as a cannot-judge naming the capability and the glob, never as an empty subject
  — a failed enumeration returns exactly what a glob matching nothing returns

#### Scenario: Files no capability claims — a stated bound

- **WHEN** a tracked file is claimed by no capability's subject
- **THEN** nothing reacts to it. Subjects are declared where a capability has something to say, and requiring
  them to tile the tree would buy coverage with thirty-six claims nobody could defend. The blindness is
  declared so that a clean report is not read as a complete one, and the reaction prints how many tracked
  paths went unclaimed rather than leaving the reader to assume none did
- **PINNED-BY** `files_no_capability_claims_are_reported_rather_than_implied_judged`

### Requirement: A change SHALL name every capability whose subject it touches

A change's proposal SHALL list, in its Capabilities section, a capability claiming each file the change
actually touches. The touched set SHALL be **produced** — the change's diff against its base — and never read
from the change's own prose, because the capability list and any prose inventory come from the same decision
and comparing them is a comparison of a value with itself.

Where more than one capability claims a touched file, naming **one** of them SHALL satisfy the join. Two
capabilities may legitimately govern one file, and demanding all of them would refuse honest proposals.

The base SHALL be resolved, and a base that cannot be resolved SHALL be a cannot-judge. Reading an
unresolvable base as *nothing was touched* would report clean over every change, which is the direction this
requirement exists to close.

Where no change is active, the reaction SHALL be clean. An ordinary checkout is asking no filing question, and
a reaction that refuses one is noise rather than governance.

#### Scenario: A change touches a file whose capability it did not name

- **WHEN** a change modifies a file claimed by some capability's subject, and its proposal's Capabilities
  section names no capability claiming that file
- **THEN** the reaction fails, naming the file, the capability that claims it, and the capabilities the
  proposal did name

#### Scenario: A shell wrapper filed under a Rust-reaction capability

- **WHEN** a change modifies `scripts/publish.sh` and names only a capability whose subject is
  `crates/tianheng/tests/**/*.rs`
- **THEN** the reaction fails. This is the defect the requirement was written from, and it is the direction the
  reaction is held to

#### Scenario: The change's base cannot be resolved

- **WHEN** the branch's base cannot be determined from its upstream or from the tracked release and main refs
- **THEN** the reaction refuses as a cannot-judge naming the branch, never reporting clean

#### Scenario: No change is active

- **WHEN** `openspec/changes/` holds no active change
- **THEN** the reaction is clean, having no filing decision in front of it
