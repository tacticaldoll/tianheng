## Why

The 0.2 line has a real composed adopter using `tianheng::prelude::*`, but the prelude's compatibility
promise is only prose in the backlog and incidental example compilation. Without an explicit,
external-view contract, a cleanup can remove or relocate a builder, selector, runner, or reaction
inspection type while Tianheng's internal tests remain green.

## What Changes

- Define the prelude as two documented usage tiers: declaration/execution and reaction inspection.
  Both remain ordinary public API; tiers explain purpose rather than weakening SemVer.
- Add the missing `ModuleRule` prelude re-export, symmetric with `Rule`, so
  `ModuleBoundary::rule()` is inspectable through the recommended wildcard entrypoint.
- Add an integration compile contract that imports only `tianheng::prelude::*` and type-checks the
  representative supported declaration chains, selector values, `run`, and reaction-inspection
  model while naming every promised export.
- Document the explicit root path for the signature-coupling semantic check and the prelude path for the
  composed adopter workflow, without removing or renaming any existing re-export. Granular hidden
  checks remain outside this contract.
- Record the scoped 0.2.x compatibility commitment in adopter-facing docs and the project decision
  log. No package version or dependency change is part of this change.

## Capabilities

### New Capabilities

- `adopter-surface`: Defines the composed prelude entrypoint, its declaration/execution and reaction-
  inspection tiers, and an external compilation reaction that keeps the documented surface honest.

### Modified Capabilities

None.

## Impact

- Affects `tianheng` public-surface documentation, an external-view integration test, the adopter
  README, `PROJECT.md`, and the corresponding backlog item.
- Adds one public re-export path for the existing `ModuleRule` type, but no new type, runtime
  behavior, dependency, or generated shell.
- Keeps pacta's wildcard-prelude source unchanged and keeps guibiao/modou surfaces out of scope.
- Makes future prelude narrowing or relocation an explicit OpenSpec/SemVer decision instead of an
  accidental cleanup.
