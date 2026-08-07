## Why

Long-lived CI comments, example documentation, and manifest comments restate mutable dependency versions that
already live in the example manifests. Several have drifted, and the observer-participant prose additionally
claims its local-patched surface is already published even though the APIs arrive in the intended release.

## What Changes

- Refer to each example's manifest requirement instead of copying its current numeric value into prose.
- Keep historical/provenance versions and the manifest requirements themselves unchanged.
- Clarify that `[Unreleased]` may name the intended release before release preparation advances workspace versions.
- Correct observer-participant to claim the public surface it actually uses, not a previously published surface.

## Capabilities

### Modified Capabilities

- `release-coherence`: release narrative may name the intended release independently of the still-released
  workspace version; the reaction judges only its enumerated mutable surfaces.

## Impact

Documentation and comments stop drifting from their manifest sources of truth. No package versions, dependency
requirements, runtime behavior, or public APIs change.
