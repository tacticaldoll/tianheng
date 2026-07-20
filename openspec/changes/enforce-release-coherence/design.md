## Context

`main` is a linear sequence of `release: X.Y.Z` snapshots. Development happens on release branches,
where fine-grained lifecycle PRs accumulate before a release-prep change updates the version and
promotes `[Unreleased]` into a dated section. The current 0.2.x review proved three drift surfaces:
adopter-facing release notes can remain empty or stale, Cargo commands can rewrite workspace lock
entries, and a release snapshot can carry lock versions from the preceding release.

Requiring lockfile equality during ordinary development would make the current branch fail for an
old snapshot defect and would conflate generated-file churn with a release invariant. The point at
which every version surface must agree is release readiness and the immutable release snapshot.

## Goals / Non-Goals

**Goals:**

- Derive release state deterministically from repository history and files.
- Require active post-release development to have adopter-facing `[Unreleased]` content.
- Require release-ready and release-snapshot states to agree across commit subject, workspace
  version, internal dependency pins, lock entries, and dated changelog entry.
- Fail with the exact divergent surface and expected version.
- Run locally and in CI without adding a parser/runtime dependency.

**Non-Goals:**

- Bump a version, create a release commit, merge `main`, tag, or publish.
- Interpret prose quality or decide whether a change is notable.
- Enforce a date window, deprecation clock, release cadence, or tag timing.
- Turn release convention into a Tianheng constitution boundary.
- Require generated lock entries to match manifests throughout ordinary development.

## Decisions

### The release spine is the state source

The check locates the most recent commit whose exact subject matches `release: X.Y.Z` and compares
that version and commit with the current workspace version and `HEAD`:

- **development** — current workspace version equals the latest release version and `HEAD` is later;
- **release-ready** — current workspace version is a strictly newer `X.Y.Z` than the latest
  release version;
- **release snapshot** — `HEAD` is the latest exact release commit.

Missing release history fails loudly because the state cannot be classified. CI therefore checks
out full history for this job. No branch name, tag, current date, or GitHub-only event variable
enters the decision, so the same repository state gives the same answer locally.
Malformed versions or a version older than the latest release fail rather than masquerading as
release-ready. The check does not infer whether a valid increase should have been patch or minor;
that remains SemVer review against the actual compatibility effect.

### State-specific invariants avoid a false permanent lock gate

Development requires at least one list item inside `[Unreleased]` and a compare link from the
current released version to `HEAD`. Workspace crate manifests must continue inheriting the common
workspace version and internal workspace dependency pins must match it in every state.

Release-ready requires `[Unreleased]` to be empty, a dated section and compare link for the new
version, and each Tianheng workspace package entry in `Cargo.lock` to equal that new version.
Release snapshot requires the same file coherence plus an exact `release: <workspace-version>` HEAD
subject. This lets normal patch development proceed on the last release version while ensuring the
next snapshot cannot repeat 0.2.0's stale-lock defect.

### A small POSIX-facing Bash check owns repository hygiene

`scripts/check_release_coherence.sh` accepts an optional repository root for focused tests and
defaults to the actual workspace. It uses `git`, `awk`, `sed`, and `grep`, already required by the
repository workflow. It does not invoke Cargo metadata because Cargo may update the very lockfile
being inspected and thereby erase the evidence before comparison.

### Temporary repositories prove each state and failure direction

A focused script builds minimal git repositories containing the relevant Cargo, lock, and changelog
surfaces. It proves a valid development, release-ready, and snapshot state, then mutates one surface
at a time to show empty development notes, stale lock versions, missing dated sections, and a
mismatched release subject fail with named diagnostics.

## Risks / Trade-offs

- **Shallow CI history makes classification impossible.** → Give the dedicated job
  `fetch-depth: 0`; missing history remains a loud configuration failure.
- **Hand-written parsing accepts TOML/Markdown structure only in Tianheng's current form.** → Keep
  the parser narrow and cover the repository layouts with fixtures; do not claim a generic release
  validator.
- **An empty `[Unreleased]` immediately after a release has no development commit to judge.** → The
  exact release snapshot is a separate valid state; the first later commit must add release notes.
- **A non-notable maintenance commit follows a release.** → This project chose an adopter-facing
  changelog for release changes; one accumulated entry is sufficient, not one entry per commit.
- **A coherent-looking downgrade is prepared.** → Compare numeric `X.Y.Z` components and reject
  any current version lower than the release spine; do not use lexicographic ordering.
- **Old 0.2.0 lock drift remains in history.** → Do not rewrite released history or bump during this
  change; enforce coherence at the next release-ready transition and snapshot.
