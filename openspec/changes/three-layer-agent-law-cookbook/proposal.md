## Why

Adopters need a clear, end-to-end recipe and terminology for exposing their project's declared constitution as an imitable Agent Law (`AGENTS.md` / `AGENTS.self-law.md`). While `GovernanceTest` and `constitution_markdown()` exist in code, adopters lack a unified name for the 3-layer law model (Preamble + Generated Projection + Law Source) and a COOKBOOK recipe teaching the preamble discipline and staleness test end-to-end.

## What Changes

- **Terminology & Concept**: Define the **Three-Layer Agent Law** structure (1. Universal Preamble, 2. Projection Body from `constitution_markdown()`, 3. Law Source in Rust) as a first-class concept in documentation.
- **Cookbook Recipe**: Add a dedicated recipe in [`COOKBOOK.md`](file:///home/qaz/work/tianheng/COOKBOOK.md) showing adopters how to assemble, test, and maintain an imitable Agent Law file with `GovernanceTest`.
- **Spec Sync**: Update `self-law-projection` delta spec to formalize the Three-Layer Agent Law concept and its adoption recipe requirement.

## Capabilities

### New Capabilities
<!-- None -->

### Modified Capabilities
- `self-law-projection`: Formalize the Three-Layer Agent Law unified concept and its end-to-end adoption recipe requirement in COOKBOOK.md.

## Impact

- Documentation ([`COOKBOOK.md`](file:///home/qaz/work/tianheng/COOKBOOK.md), [`BACKLOG.md`](file:///home/qaz/work/tianheng/BACKLOG.md)).
- OpenSpec specification (`openspec/specs/self-law-projection/spec.md`).
- No breaking API changes; additive documentation & spec refinement.
