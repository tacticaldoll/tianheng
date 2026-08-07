## Context

All three dimension crates depend on `xuanji`, return `Vec<BoundDecl>` from `observation_bounds`, implement `Observer`, and already re-export portions of the shared reaction model. The accidental difference is at the public root: `hunyi` exposes the bound and observer vocabulary, while `guibiao` and `louke` require an adopter to add and version-match `xuanji` directly.

## Goals / Non-Goals

**Goals:**

- Make the standalone public surface symmetric across the three dimensions.
- Let an adopter name every type needed to inspect declarations or implement the observer protocol through the dimension it already depends on.
- Prove the surface from external integration tests rather than internal imports.

**Non-Goals:**

- Create dimension-specific wrappers or forks of the shared types.
- Make one dimension the canonical owner of a protocol defined by `xuanji`.
- Change the composed `tianheng` re-export path or runtime behavior.

## Decisions

1. Re-export the existing `xuanji` identities directly. Wrappers would split type identity and turn an additive surface fix into conversion infrastructure.
2. Hold the same root vocabulary in all three adopter tests: `BoundDecl`, `BoundId`, `Defence`, `Demonstrates`, `Extent`, `FactGranularity`, `Observer`, `Outcome`, `Owner`, and `Reached`. This is a required interface set, not a remembered count.
3. Keep the tests under each crate's external `tests/` surface and import only that crate. A unit test could pass through private imports that a standalone adopter cannot use.
4. Leave `tianheng`'s existing composition funnel unchanged. Once every dimension exposes the protocol, the shell's internal choice of one re-export edge no longer withholds the vocabulary from standalone users.

## Risks / Trade-offs

- [Broader public API] Re-exports become compatibility commitments. → Export only types already present in public signatures or required to construct and inspect those values.
- [Lists drift independently] Three tests could diverge. → Use the same compile-time vocabulary in each test and review their diff together; no runtime abstraction can prove a crate-root export exists.
