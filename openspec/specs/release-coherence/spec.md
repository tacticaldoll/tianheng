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
