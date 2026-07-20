## Why

Version-1 baselines remain readable but match human finding text exactly, while version 2 permits
presentation changes without re-keying. The README mentions automatic upgrade on write but does not
tell legacy adopters when to perform it or which metadata survives, leaving the bounded migration
path easy to miss.

## What Changes

- Document `--write-baseline` as the existing explicit, opt-in V1-to-V2 upgrade operation.
- Advise V1 adopters to rewrite before a finding-wording change when continued suppression or
  metadata preservation matters.
- State that exact live matches preserve metadata and stale legacy entries drop with snapshot
  regeneration.
- Explicitly reject a new migration command, perpetual read warning, or time-based deprecation.
- Mark the documentation-only backlog item built.

## Capabilities

### New Capabilities

None.

### Modified Capabilities

- `violation-baseline`: require adopter documentation to expose the bounded legacy upgrade path and
  its exact-match/stale-entry consequences.

## Impact

The root adopter README, baseline specification, and backlog change. Runtime behavior, public APIs,
wire formats, dependencies, manifests, and package versions do not change.
