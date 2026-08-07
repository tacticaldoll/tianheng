## Why

The observation-bound model accepts `Defence::Unpinned`, but the live register currently contains no unpinned declaration, so the code that compares and projects that state is never executed. A future unpinned bound could therefore drift at either output boundary without this reaction noticing.

## What Changes

- Factor the typed-to-spec defence conversion into one comparison helper.
- Exercise an unpinned declaration through both the spec comparison representation and the generated extent rendering.
- Require the tracker text to survive both paths without adding a fabricated live unpinned bound.

## Capabilities

### New Capabilities

None.

### Modified Capabilities

- `observation-bound-model`: Require the unpinned defence state and tracker to be exercised through both comparison and projection paths even while the live register has no unpinned entries.

## Impact

Touches only the observation-bound model integration test and its specification. It does not add a live observation bound, alter the accepted constitution or generated projections, change public APIs, add dependencies, bump package versions, or require adopter migration.
