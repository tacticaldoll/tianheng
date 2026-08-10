# Change: Retire the repository refusal mutation sweep

## Why

Kanhe's repository checks need a typed distinction between a disagreement and an input they cannot judge. They do not need every constructor to carry runtime instrumentation, a compiler-corpus source scanner, an exemption registry, and an environment-gated process-per-site mutation sweep.

That matrix has expanded into a second governance system around repository diagnostics. It also leaked repository-only support into the published product: `tianheng::observation_bounds()` currently emits bounds whose ids belong to `rust-repository-reactions`, solely to classify the sweep's scanner and exemption limits.

## What Changes

- Keep Kanhe's shared typed refusal and ordinary constructors, but make them pure values with no mutation/recording environment.
- Delete the refusal-site scanner, exemption registry, mutation integration test, census, CI/DoD command, and publish-wrapper environment scrub that exist only for the sweep.
- Remove the sweep-specific requirements and declared bounds from `rust-repository-reactions`.
- Move the surviving `rust-repository-reactions` bounds into a Kanhe-owned catalog, remove the sweep-only bounds, and reject that capability's ids from the published Tianheng catalog.
- Preserve focused failure matrices at each repository judgement boundary.

## Capabilities

### Modified Capabilities

- `rust-repository-reactions`: retires constructor-site mutation as a repository governance requirement while retaining one typed repository-check result and focused behavior matrices.
- `observation-bound-model`: the published Tianheng bound catalog must not contain Kanhe-owned `rust-repository-reactions` declarations, while the model still consumes those declarations from Kanhe.

### Existing Capabilities

- `publish-source-integrity`: requirements do not change; `scripts/publish.sh` only stops scrubbing mutation variables that no longer exist.
- `release-coherence`: requirements do not change; its gate and focused matrix only adopt the pure shared refusal constructor, while the changelog records the catalog migration.
- `observation-bound-register`: requirements do not change; its generated projection and governed specs only reflect the relocated and retired declarations.

## Impact

- Removes repository-only Kanhe modules and one expensive env-gated CI/DoD matrix.
- Changes the membership of public `tianheng::observation_bounds()` by removing every `rust-repository-reactions` declaration; surviving repository declarations remain available only to this repository through Kanhe.
- Does not change `BoundDecl`, `Extent`, report formats, boundary evaluation, or any declaration builder signature.
- **BREAKING** for consumers that explicitly match `rust-repository-reactions` ids in the published catalog; those consumers must stop expecting repository-check declarations from the product.
