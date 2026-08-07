## Why

`check_bound_register.sh` accepts a repository argument, but keeps a relative spelling after entering that repository for the tracked-Markdown census. Later projection paths then prepend the same relative root again, so a valid fixture can read or write `<repo>/<repo>/docs/observation-bounds.md` instead of the judged tree. The gate must resolve one stable repository root before any directory transition.

## What Changes

- Resolve the judged repository to one physical absolute root before any scan or projection operation.
- Add a failure-matrix direction that invokes the gate through a relative repository argument after tracked Markdown has forced the directory transition.
- Require relative and absolute invocations to judge and regenerate the same projection path.

## Capabilities

### New Capabilities

None.

### Modified Capabilities

- `observation-bound-register`: Require every accepted repository spelling to remain anchored to the same judged root through census scanning and projection access.

## Impact

Touches the observation-bound register shell gate, its companion failure matrix, and its specification. No Rust API, manifest, dependency, package version, or generated document format changes.
