## Why

The self-law spec says a direct `tianheng` → `xingbiao` normal dependency reacts, but that direction is backed only by a one-time manual edit recorded in a PR. A permanent fixture must exercise the real shell boundary so future changes cannot leave the scenario as historical evidence.

## What Changes

- Add an isolated fixture package named `tianheng` whose only forbidden normal edge is a path dependency on `xingbiao`.
- Add a self-governance test that selects the live shell dependency boundary from `tianheng_constitution()` and evaluates it against that fixture.
- Record the fixture-backed reaction in the self-law-projection specification.

## Capabilities

### New Capabilities

None.

### Modified Capabilities

- `self-law-projection`: Require the direct shell-to-metadata scenario to be defended permanently by a fixture using the accepted boundary declaration.

## Impact

Touches the self-governance test, one isolated Cargo fixture, and the self-law-projection spec. The accepted constitution, generated projection, production workspace graph, public API, package versions, and runtime behavior do not change.
