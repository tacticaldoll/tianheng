# Change: Separate repository-governance bound catalogs from Tianheng

## Why

`tianheng::observation_bounds()` is a published shell entrypoint, but every declaration it still returns describes an unpublished repository-governance capability. Product dimensions already expose their own catalogs through their observers; the shell catalog contains no product reaction of its own.

Keeping the entrypoint therefore makes repository test support look like product capability. Prefix-by-prefix exclusions only move the leak: Kanhe owns the repository consistency gates, Shengmo owns the self-law dogfood reaction, and Tianheng should expose neither as its own catalog.

## What Changes

- Remove the Tianheng shell bound module and its public `observation_bounds()` export.
- Move the remaining Kanhe-owned declarations into `kanhe::bounds::observation_bounds()`.
- Move `self-law-projection` declarations into a Shengmo-owned catalog.
- Make the repository observation-bound model consume dimension, Kanhe, and Shengmo catalogs explicitly.
- Replace the narrow `rust-repository-reactions` product-catalog guard with a guard that the Tianheng product source defines no repository bound catalog entrypoint.
- Correct the unreleased changelog: this API did not exist in v0.4.0, so retiring it before its first release requires no released adopter migration.

## Capabilities

### Modified Capabilities

- `observation-bound-model`: repository declarations are owned and enumerated by the unpublished crate whose reaction they qualify; the product shell has no repository bound catalog.

### Existing Capabilities

- `self-law-projection`: requirements do not change; its declarations move beside the Shengmo reaction they qualify.
- `observation-bound-register`: requirements do not change; its declarations move beside the Kanhe gate.
- `observer-protocol`: product protocol requirements do not change; the repository reader's declared limits move beside its Kanhe gate.
- `projection-register`: requirements do not change; its declarations move beside the Kanhe gate.
- `publish-source-integrity`: requirements do not change; its declaration moves beside the Kanhe gate.
- `release-coherence`: requirements do not change; its declarations move beside the Kanhe gate and its governed changelog records the compatibility result.
- `adopter-surface`: no promised prelude or inspection name changes; the unpromised, unreleased repository catalog entrypoint is removed from the crate root.

## Impact

- Removes `tianheng::observation_bounds()` before that entrypoint has appeared in a published release.
- Adds an unpublished Shengmo catalog and expands the unpublished Kanhe catalog; neither ships in a package.
- Leaves `BoundDecl`, `BoundId`, `Extent`, dimension observers, product reports, evaluators, and package manifests unchanged.
- Requires callers using the unreleased repository checkout directly to stop calling the retired shell entrypoint; released v0.4.0 adopters take no action.
