## Why

The built-in composition path skips an empty semantic dimension, but `SemanticObserver::observe` still reads Cargo metadata before discovering that it has no boundaries. With an unreadable manifest, the two composition paths therefore return different verdicts for the same empty semantic participation.

## What Changes

- Return `Clean` from `SemanticObserver` before any metadata read when its boundary set is empty.
- Add a discriminatory unit test using a manifest path that does not exist.
- Specify the empty semantic observer behavior as part of observer-protocol parity and document the fix under `[Unreleased]`.

## Capabilities

### New Capabilities

None.

### Modified Capabilities

- `observer-protocol`: Require an empty semantic observer to contribute `Clean` without performing workspace I/O.

## Impact

Only `hunyi::SemanticObserver::observe`, its tests, the observer-protocol specification, and adopter-facing release notes change. Non-empty semantic observation, public signatures, manifests, and package versions remain unchanged.
