## Why

Each dimension publicly returns `BoundDecl` and implements `Observer`, but only `hunyi` exposes the shared bound and observer vocabulary from its own root. A standalone `guibiao` or `louke` adopter therefore cannot name the public protocol it is already consuming without discovering and adding a direct `xuanji` dependency.

## What Changes

- Re-export the same bound-declaration and observer protocol types from every dimension root.
- Add external adopter-surface tests that name the complete vocabulary through each dimension crate alone.
- Document the additive standalone-dimension surface in `[Unreleased]`.

## Capabilities

### New Capabilities

None.

### Modified Capabilities

- `observation-bound-model`: Require each standalone dimension root to expose the shared vocabulary used by its public declaration and observer surface.

## Impact

`guibiao` and `louke` gain additive public re-exports; `hunyi` is held to the same exact surface by its adopter test. No wrapper types, dependency changes, manifest versions, or composed-shell behavior change.
