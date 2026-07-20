## ADDED Requirements

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
pass this gate.

#### Scenario: Development with release notes is coherent

- **WHEN** post-release commits retain the released version and `[Unreleased]` contains an item and
  the matching comparison link
- **THEN** release coherence passes without requiring a release-prep version or lock rewrite

#### Scenario: Empty development notes fail

- **WHEN** post-release commits exist but `[Unreleased]` contains no list item
- **THEN** the coherence check fails and names the missing adopter-facing release narrative

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
