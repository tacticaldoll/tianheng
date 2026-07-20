## Why

The 0.2.x readiness review found an empty `[Unreleased]`, stale compatibility prose, and a release
snapshot whose workspace lock entries still named 0.1.10. These are now observed release-integrity
failures, so the release spine needs a repository-state reaction rather than another reminder.

## What Changes

- Add a release-coherence check that derives development, release-ready, and release-snapshot state
  from the linear `release: X.Y.Z` git spine and the current workspace version.
- Require active development after a release to carry adopter-facing `[Unreleased]` content.
- Require release-ready and release-snapshot states to align the release subject, workspace version,
  internal dependency pins, workspace lock entries, and dated CHANGELOG section.
- Run the check in local Definition of Done and a CI job with sufficient git history.
- Add focused temporary-repository tests for valid states and each failure direction.

## Capabilities

### New Capabilities

- `release-coherence`: Repository hygiene reaction relating the release commit spine, Cargo version
  surfaces, lockfile snapshot, and adopter-facing changelog without time-based policy.

### Modified Capabilities

None.

## Impact

The change affects repository scripts, CI checkout depth, contribution/release procedure, BACKLOG,
CHANGELOG, and a new repository-governance specification. It adds no Tianheng constitution
boundary, dependency, public Rust API, evaluator behavior, wire change, package-version bump, tag,
or release action.
