## Why

Release coherence deliberately permits `[Unreleased]` adopter narrative to name a future intended release while manifests remain on the current shipped version, but its matrix has no such instance. A future prose-version equality check could therefore contradict the contract while every existing case remains green.

## What Changes

- Add a development fixture whose `[Unreleased]` item names a different future version while the workspace, example requirement, lockfile, and comparison link remain on the current release.
- Assert that release coherence classifies that fixture as development and passes.
- Sharpen the existing scenario so the mismatched prose literal is the observable condition, without adding a general prose-number detector.

## Capabilities

### New Capabilities

None.

### Modified Capabilities

- `release-coherence`: Make the intended-release narrative allowance explicit as a different prose version literal backed by the failure matrix.

## Impact

Touches only the release-coherence failure matrix and specification. The release gate, manifests, package versions, public APIs, accepted Tianheng law, baselines, and adopter behavior do not change.
