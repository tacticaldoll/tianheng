# Design

## Context

Each product dimension already owns its declared observation bounds and returns them through its `Observer` implementation. Tianheng's extra free-function catalog was introduced for declarations whose reactions actually live in this repository's unpublished governance crates. After `rust-repository-reactions` moved to Kanhe, the residual catalog contains only `observation-bound-model`, `observation-bound-register`, `observer-protocol`, `projection-register`, `publish-source-integrity`, `release-coherence`, and `self-law-projection` entries.

The first six are qualified by Kanhe tests. `self-law-projection` is qualified by Shengmo's self-governance test. None is a reaction implemented by the published shell.

## Goals / Non-Goals

**Goals:**

- Make catalog ownership follow the crate containing the reaction being qualified.
- Remove the product entrypoint whose complete membership is repository-only.
- Preserve every declaration's identity, extent, owner, reason, and defence while moving it.
- Keep one combined repository bijection and generated projection over every catalog.

**Non-Goals:**

- Change any bound classification or pin.
- Change the product `Observer` protocol or dimension catalogs.
- Infer future ownership from capability-name prefixes.
- Perform the broader prose replacement of “reaction” with “repository check”; that remains a separate documentation-focused change.

## Decisions

### Remove the shell catalog rather than maintain an exclusion list

The residual membership is entirely repository-owned. An empty public function would be a target with no reaction and would preserve the misleading product capability, while another prefix denylist would need one amendment per migrated capability. Removing the entrypoint makes the absence construction-held.

### Split declarations by reaction owner

Kanhe's catalog receives the declarations qualified by Kanhe repository gates. Shengmo receives a new catalog for the declarations qualified by its self-law projection reaction. The values move unchanged; only their module ownership changes.

### Compose catalogs only in the repository model

`crates/kanhe/tests/observation_bound_model.rs` remains the sole family-wide join. It chains the three product dimension observers, Kanhe's catalog, and Shengmo's catalog. Its existing duplicate-id and spec-bijection checks prevent relocation from dropping or duplicating declarations.

### Guard the exact retired entrypoint, not repository intent

A Kanhe test reads Tianheng's tracked Rust source and rejects the exact `observation_bounds` catalog vocabulary. This is deliberately narrower than deciding whether arbitrary product code “looks like repository governance.” Before removal it fails on the current module and export; after removal it prevents that exact misleading entrypoint from returning under another Tianheng source file.

## Risks / Trade-offs

- Source vocabulary is not name resolution. The guard protects the exact retired entrypoint, not every alias or future semantic equivalent; the requirement makes no wider claim.
- Direct users of the unreleased checkout lose a root function. The latest published release does not contain it, so the change is non-breaking under this repository's adopter-action definition.
- Kanhe depends on Shengmo already, so consuming the new catalog introduces no dependency edge.

## Verification

- Before removal, the exact product-source guard fails on Tianheng's current bound module/export.
- After relocation, the combined model contains every spec declaration exactly once and both generated projections are fresh.
- A value-by-value comparison against the pre-change catalog shows every declaration moved without semantic edits.
- Product compilation, adopter-surface tests, self-governance, Kanhe gates, and the complete Definition of Done pass.
