## Why

The governance-dogfood spec requires the focused example matrices to run in order before the positive driver and forbids the driver from rerunning them, but no reaction currently distinguishes either drift. The prose therefore promises an orchestration shape that can change while every gate remains green.

## What Changes

- Extend the DoD-coherence reaction to require the focused example matrices and positive driver as one contiguous ordered sequence in both the local Definition of Done and CI.
- Make the reaction reject focused-matrix basenames on the positive driver's non-comment source lines.
- Add failure-matrix cases for reordered local orchestration, reordered CI orchestration, nested reruns, and an unreadable driver surface.
- Bound the specification to the authored shell shapes the reaction actually reads.

## Capabilities

### New Capabilities

None.

### Modified Capabilities

- `governance-dogfood`: Convert the focused-matrix ordering and non-recursion claim into an explicit source-shape reaction.

## Impact

Touches the DoD-coherence gate and its failure matrix plus the governance-dogfood specification. It changes repository governance only; no Rust API, accepted Tianheng law, dependency graph, package version, baseline identity, or adopter behavior changes.
