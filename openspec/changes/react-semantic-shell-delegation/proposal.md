## Why

The observer-protocol spec requires the shell to delegate an empty semantic bundle to `hunyi::check_all`, but its shell scenario has no reaction: reintroducing a local `is_empty` guard in `evaluate_constitution` leaves every behavioral test green. The single-owner claim needs a source-shape observation at the level that actually changes.

## What Changes

- Add a repository test that locates `evaluate_constitution` and requires its semantic boundary accessor to appear exactly once, as the direct argument to `hunyi::check_all`.
- Refuse a missing composition function or any local semantic-boundary decision instead of treating an unreadable shape as compliant.
- Reuse the existing executed-line source view and brace-counted function-body recognition rather than adding a second lexical model.

## Capabilities

### New Capabilities

None.

### Modified Capabilities

- `observer-protocol`: Bind the shell delegation scenario to a source-shape reaction that fails when semantic emptiness is decided in the shell.

## Impact

Touches the observer-protocol repository test and its specification. No product behavior, public API, manifest, dependency, package version, baseline, or generated projection changes.
