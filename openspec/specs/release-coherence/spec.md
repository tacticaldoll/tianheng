# Release Coherence Specification

## Purpose

Define the read-only repository reaction that keeps Tianheng's release commit spine, Cargo version
surfaces, lock snapshot, and adopter-facing changelog coherent without time-based release policy.

## Requirements

### Requirement: Repository state determines the release phase

The repository SHALL classify its release phase solely from the latest exact `release: X.Y.Z`
commit in git history, the position of `HEAD`, and the current workspace version. A later commit at
the same version SHALL be development; a strictly newer numeric `X.Y.Z` current version SHALL be
release-ready; and the exact latest release commit SHALL be a release snapshot. A current version
older than the latest release, or missing or malformed release history, SHALL fail as an observable
repository misconfiguration. Classification SHALL NOT depend on branch names, tags, wall-clock
time, warning windows, or hosted-CI-only variables.

#### Scenario: Post-release work is development

- **WHEN** `HEAD` is later than the latest exact release commit and the workspace version is
  unchanged
- **THEN** the repository is checked as active development

#### Scenario: A newer workspace version is release-ready

- **WHEN** `HEAD` is later than the latest exact release commit and the numeric `X.Y.Z` workspace
  version is strictly newer
- **THEN** the repository is checked as release-ready

#### Scenario: A version regression fails loud

- **WHEN** the workspace version is older than the latest exact release commit
- **THEN** the coherence check fails and names the current and latest release versions

#### Scenario: The release commit is a snapshot

- **WHEN** `HEAD` is the latest exact `release: X.Y.Z` commit
- **THEN** the repository is checked as a release snapshot for `X.Y.Z`

#### Scenario: Shallow or absent history fails loud

- **WHEN** no exact release commit is observable in the available git history
- **THEN** the coherence check fails and identifies release history as unavailable

### Requirement: Development carries adopter-facing release narrative

Active development SHALL retain the current released workspace version, at least one changelog list
item under `[Unreleased]`, and an `[Unreleased]` comparison link from that version to `HEAD`.
Workspace crate manifests SHALL inherit the common version and internal workspace dependency pins
SHALL equal it. Development SHALL NOT require old generated lock entries to be rewritten solely to
pass this gate. `[Unreleased]` may name the intended release in adopter-facing narrative before mechanical
release preparation advances the workspace version. The reaction SHALL judge the mutable version-bearing
surfaces it enumerates; it SHALL NOT require a version literal in `[Unreleased]` prose to equal the still-released
workspace version.

#### Scenario: Development with release notes is coherent

- **WHEN** post-release commits retain the released version and `[Unreleased]` contains an item and
  the matching comparison link
- **THEN** release coherence passes without requiring a release-prep version or lock rewrite

#### Scenario: A different intended release literal precedes mechanical version preparation

- **WHEN** `[Unreleased]` prose names a future version different from the current released workspace version,
  while workspace and example manifests, internal pins, lock entries, and the comparison link retain that current version
- **THEN** development coherence passes because prose narrative is not one of the enumerated version-bearing surfaces

#### Scenario: Empty development notes fail

- **WHEN** post-release commits exist but `[Unreleased]` contains no list item
- **THEN** the coherence check fails and names the missing adopter-facing release narrative

### Requirement: A release section SHALL be coherent with itself

The reaction reads the changelog's **state** — which version, which sections exist, whether the comparison link
is right. It SHALL also read each release section's **internal** consistency, which is a different question and
was unasked until two defects of that shape landed in one window: an `[Unreleased]` grew a second `### Changed`
heading three hundred lines from the first, and a prose claim about which prior releases carry a `### Migration`
section was wrong under every reading.

A heading SHALL NOT appear twice within one release section. Two blocks of one name split what belongs
together, and a reader of the second never learns the first exists.

A section marking a change `**BREAKING**` SHALL carry a `### Migration` section. The obligation is one-way: a
section MAY carry a migration for a break marked some other way, which this repository's own `[0.3.0]` does.

The vacuity guard SHALL be over **sections**, not headings. A changelog whose sections carry bullets directly
and no `###` sub-headings is an ordinary small changelog — this repository's own early releases are that shape —
so guarding on headings would refuse them; a changelog with no `## [` section at all is the undecidable one.

What this requirement SHALL NOT reach is the **content** of an entry: whether it is accurate, whether "no
adopter action" is true, whether a named symbol exists. Those are judgements over prose, and the detector they
would need is the one `AGENTS.md` records as designed, measured three times and rejected. The line drawn here is
between the document's grammar and its claims, and only the grammar is decidable — which is why the two defects
above were reachable and the sentence about them was not.

#### Scenario: A release section repeats a heading

- **WHEN** one `## [` section carries two `### ` headings of the same name
- **THEN** the reaction fails, naming the section and the heading

#### Scenario: A break is marked with nowhere to read what to do

- **WHEN** a release section contains a `**BREAKING**` marker and no `### Migration` heading
- **THEN** the reaction fails, naming the section

#### Scenario: A break is marked and the migration is there

- **WHEN** the same section carries both
- **THEN** the reaction is clean, so the refusal above is about the missing migration rather than about the
  marker

#### Scenario: A changelog with no release section at all

- **WHEN** the structure read from the changelog holds no `## [` section
- **THEN** the reaction refuses to judge rather than reporting every property of zero sections satisfied

### Requirement: Release-ready and snapshot surfaces agree

A release-ready repository SHALL carry an empty `[Unreleased]` section, a dated changelog section
for the current workspace version, a comparison link for that version, matching internal workspace
dependency pins, and matching `Cargo.lock` entries for every Tianheng workspace package. A release
snapshot SHALL additionally have the exact subject `release: <workspace-version>`. Any divergence
SHALL fail and name the surface and expected version. The check SHALL observe repository state only
and SHALL NOT perform a version bump, commit, merge, tag, or publish action.

#### Scenario: A coherent release candidate passes

- **WHEN** the workspace version is newer than the latest release and every changelog, pin, and
  lock surface names the new version
- **THEN** release coherence passes as release-ready

#### Scenario: A stale lock entry fails release readiness

- **WHEN** any Tianheng workspace package lock entry names a version other than the release-ready
  workspace version
- **THEN** the coherence check fails and names that package and expected version

#### Scenario: A mismatched release subject fails the snapshot

- **WHEN** `HEAD` is an exact release commit whose subject version differs from the workspace
  version
- **THEN** the coherence check fails and names both versions

#### Scenario: The check performs no release action

- **WHEN** release coherence is evaluated in any phase
- **THEN** repository files, commits, tags, packages, and external release state remain unchanged

### Requirement: Adopter narrative SHALL NOT name this repository's own machinery

An entry under an adopter-facing heading of the `[Unreleased]` section SHALL NOT name this repository's
own machinery in any of three forms: a path under `scripts/`, a bare basename that `git ls-files scripts/`
resolves to a tracked file there, or a **directory** under `scripts/` written with its trailing slash. All
three are derived from the one enumeration — the directories by stripping components from each enumerated
path — so none is a list written beside it.

`CHANGELOG.md` is the adopter's document. It carries nine kinds of heading — `### Added`, `### Changed`,
`### Fixed`, `### Migration`, `### Documentation`, `### Removed`, `### Compatibility`,
`### Compatibility evidence` and `### Self-governance` — and every one of the first eight is an adopter's vocabulary. It offered none that was
not, so every
change to this repository's own governance machinery has been written into one of them. Measured **before this rule existed**, in the window that introduced it: twenty entries named it, spread
across four adopter headings, for a directory that ships in **zero** packages. That figure is a record of a
past state rather than a census — no reaction produces it now, the section it counted has been collapsed, and
holding a record to today's enumeration would demand the record change every time the tree does.

The `[Unreleased]` section SHALL be permitted a `### Self-governance` heading,
under which naming that machinery is what belongs; a heading is adopter-facing when it is any `### `
heading **other than** that one, so a heading nobody anticipated is adopter-facing rather than exempt.

The basename form SHALL be decided against the enumerator and never against a list of gate names written
beside it. A hand-kept list lets a new script be added and never measured, which is the register's own
prohibition rather than a stylistic call. For the same reason no count of that enumeration SHALL be written
into the reaction: a census is produced, never typed, and the first draft of this reaction carried one that
the commit adding it made stale.

A name SHALL be recognised as a **word** — a maximal run of path characters, required to equal a tracked
path, basename or derived directory. That is exact matching of a lexical token rather than substring
matching: a sentence
merely containing the characters cannot match, because the run is delimited by the first character a path
cannot hold. The rule was first written to compare whole **backticked spans**, and adversarial review
reproduced three false negatives against that reading, every one a shape this repository's own changelog
already uses — a span carrying anything besides the bare path, a double-backtick span, and an inline span
wrapped across a source line. Reading words closes all three and reaches a markdown link target the span
reading never could.

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

- **WHEN** an entry under an adopter-facing heading names `publish.sh` with no directory
- **THEN** the reaction fails, because `git ls-files scripts/` resolves that basename to a tracked file

#### Scenario: A gate named inside a longer span, or as unquoted prose

- **WHEN** an entry under an adopter-facing heading writes `` `bash crates/jiaochou/tests/pin_bites.rs --fix` ``, or
  `` `` `crates/jiaochou/tests/pin_bites.rs` `` ``, or a span wrapped across a source line, or a markdown link whose
  target is the gate, or the bare name with no backticks at all
- **THEN** the reaction fails in every one of those, because the word is the unit rather than the span

#### Scenario: A bare basename the enumerator does not resolve

- **WHEN** an entry under an adopter-facing heading names `check_something_that_does_not_exist.sh`
- **THEN** the reaction is clean, so the rule is held to the enumerator rather than to the `check_`
  prefix

#### Scenario: A dated release section names a gate — a stated bound

- **WHEN** a dated `## [X.Y.Z] - DATE` section carries an entry naming a path under `scripts/`
- **THEN** nothing reacts, and the leak is real: an adopter reading `[0.4.0]` meets nine entries naming
  files they can never run. What is refused is the **repair**, not the diagnosis — rewriting a dated section
  to satisfy a rule written afterwards would falsify the record, the same reason `docs/history/` is left
  alone — so this is a declared false negative with an owner rather than a shape that is harmless. Closing it
  needs a repair that adds to the record instead of editing it
- **PINNED-BY** `a_dated_section_naming_a_gate_is_a_stated_bound`

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

#### Scenario: A basename an entry writes for another reason — a stated bound

- **WHEN** an adopter-facing entry names a file of its own whose basename the judged repository also tracks
  under `scripts/`
- **THEN** the reaction **fails**, refusing an innocent entry. The direction is the safe one — an author
  meets a refusal to argue with — and narrowing it means deciding which of two files a bare name meant,
  a judgement about the sentence rather than about the reference
- **PINNED-BY** `a_colliding_basename_is_a_stated_bound`

#### Scenario: A name reached only through a URL — a stated bound

- **WHEN** an adopter-facing entry names machinery only inside a URL
- **THEN** nothing reacts. A word is a maximal run of path characters, so a scheme and host fuse with the
  path into one run that equals no tracked name; splitting a URL into its path would make the reaction judge
  a foreign host's layout as though it were this repository's
- **PINNED-BY** `a_name_reached_only_through_a_url_is_a_stated_bound`

#### Scenario: A heading inside a fenced code block — a stated bound

- **WHEN** a `### ` line sits inside a fenced code block, followed by entries that name machinery
- **THEN** nothing reacts for those entries, because that line set the heading in force and may name the one
  exempt heading. The reaction walks the document's line grammar and does not track fences; it is latent
  rather than live, this repository's changelog carrying no fenced block at all
- **PINNED-BY** `a_heading_inside_a_fenced_block_is_a_stated_bound`

#### Scenario: The directory itself, and a derived ancestor

- **WHEN** an entry under an adopter-facing heading names `` `scripts/` ``, or `` a shared shell library `` where the
  judged repository tracks a file two levels below it
- **THEN** the reaction fails in both, because every ancestor directory is derived from the enumeration by
  stripping one component at a time

#### Scenario: A directory named without its trailing slash — a stated bound

- **WHEN** an adopter-facing entry names `scripts` or a shared shell library with no trailing slash
- **THEN** nothing reacts. Directories are derived slash-terminated, and the unslashed form is a word
  indistinguishable from ordinary prose — `scripts` is an English plural this document already uses as one.
  Admitting it for deeper names only would make the reaction judge which of its own keys read as English
- **PINNED-BY** `a_directory_named_without_its_slash_is_a_stated_bound`

#### Scenario: The enumeration fails rather than returning nothing

- **WHEN** `git ls-files scripts/` exits non-zero while an adopter-facing entry names a gate
- **THEN** the reaction refuses to judge. This is the direction whose absence is a false **negative** rather
  than a downgrade: with the refusal replaced by a plain redirect, the parser reads the empty capture cleanly
  and reports a document naming a gate as coherent

### Requirement: An enumeration SHALL NOT pass over content it failed to read

Every enumeration this judgement makes SHALL distinguish **absent** from **unreadable**, and SHALL refuse as a
cannot-judge on the second. Skipping is reserved for what genuinely is not there.

A directory entry that fails to yield SHALL be propagated rather than dropped, and a manifest that exists and
cannot be read SHALL refuse rather than be skipped. Collapsing the two lets the remaining readable members
satisfy the counters, so the run reports clean over the very file it could not read — and the counters are what
the judgement then reasons from.

Where propagating produces a refusal no fixture can construct, it SHALL be declared out of reach with a slug of
its own. A slug shared between two sites excuses whichever one was looked at.

#### Scenario: A manifest exists and cannot be read

- **WHEN** an example manifest is present but is not readable as text
- **THEN** the judgement refuses as a cannot-judge naming the path, rather than skipping it as though the
  directory held no manifest

#### Scenario: A directory holds no manifest at all

- **WHEN** a directory under the enumerated root has no `Cargo.toml`
- **THEN** it is skipped, because absence is not a failed read

#### Scenario: A directory entry cannot be yielded

- **WHEN** iterating an enumerated directory fails part-way
- **THEN** the judgement refuses rather than continuing over the entries it did receive
