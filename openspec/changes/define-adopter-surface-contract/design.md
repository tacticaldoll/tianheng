## Context

`tianheng::prelude` currently exports 31 names. The composed example, the sans-I/O example, the
generated demo constitution, and pacta all use the wildcard entrypoint, but no single external-view
test names the whole intended surface. The backlog therefore cannot distinguish an intentional API
decision from an accidental re-export deletion.

The prelude is not only a builder menu. pacta inspects `Outcome`, while adoption examples use
selector enums and per-dimension checks to prove reactions. Removing every non-builder name would
make the prelude aesthetically smaller but would break the actual adopter whose presence opened the
0.2 window.

The 0.2 rule-model change also made `ModuleBoundary::rule()` public, but the prelude exports only
crate-side `Rule`; its existing root-level sibling `ModuleRule` is missing. The recommended wildcard
entrypoint is therefore asymmetric for reaction inspection.

## Goals / Non-Goals

**Goals:**

- State what every existing prelude name is for, add the missing symmetric `ModuleRule`, and keep
  all 32 names reachable in 0.2.x.
- Make the contract react at compile time from an integration crate's external view.
- Preserve the one-import composed adoption path while documenting specialized checks at the root.
- Keep the test readable enough that an intentional future amendment produces an obvious diff.

**Non-Goals:**

- Removing, renaming, or deprecating public API, or adding a new public type.
- Promising that every builder can gain no new methods or variants.
- Moving per-dimension engines or checks into the shell.
- Treating documentation tiers as different SemVer levels.
- Changing package versions, dependencies, runtime behavior, or the constitution.

## Decisions

### Classify by use, not stability

The README and module docs will describe two equally compatible usage tiers:

1. **Declaration and execution:** `Constitution`; terminal static, semantic, runtime, and profile
   boundary types; `DependencyKind`, `SourceKind`, `VisibilityCeiling`, and `Severity`; and `run`.
2. **Reaction inspection:** `Boundary`, `BoundaryKind`, `Rule`, `ModuleRule`, `Baseline`, `BaselineEntry`,
   `Finding`, `FindingKey`, `Outcome`, `Polarity`, `Report`, `Violation`, `ViolationId`, and the pure
   static `check` entrypoint.

The distinction is navigational. Both tiers are public 0.2.x API. `ModuleRule` is added beside
`Rule` so both boundary accessors are usable through this entrypoint. This avoids inventing a weaker
"convenience" promise that Rust's public API and the real wildcard adopter cannot honestly support.

Alternative considered: trim the inspection tier from the prelude during the 0.2 break. Rejected
because pacta consumes `Outcome` from the wildcard prelude and because inspectable reaction data is
part of governing by reaction, not an implementation detail.

### Use an external-view integration compile contract

Add `crates/tianheng/tests/adopter_surface.rs`. As an integration test crate it sees exactly what a
dependency sees. It imports `tianheng::prelude::*`, names every promised export, and type-checks
representative declaration chains across all three instruments plus the sans-I/O profile. It also
type-checks `run` and the read-side reaction types without executing process or filesystem effects.

Alternative considered: source-text inspection of the `pub use` list. Rejected because it would
bind formatting and aliases while failing to prove that the types and methods are usable externally.

Alternative considered: rely on examples and pacta compilation. Rejected as the sole guard because
their imports are demand-shaped and do not name the whole deliberate contract; they remain valuable
reference-consumer evidence in addition to the focused test.

### Keep focused checks at the crate root

The prelude retains its existing pure static `check` because it is already adopted and complements
`run`. The public `check_semantic` alias for hunyi's signature-coupling `check` remains an explicit
root import and is documented for that focused inspection, not added to the wildcard surface. The
full semantic bundle stays the shell's `check_all` path, and the seven granular `check_*` re-exports
remain `#[doc(hidden)]` and outside this contract. This preserves the composed funnel rather than
turning the prelude into a per-capability engine menu.

## Risks / Trade-offs

- **The test freezes too much accidental API.** → Bind the current 31-name wildcard contract plus
  the missing symmetric `ModuleRule`; do not name hidden draft types or granular semantic checks.
- **The two tiers are misread as strong versus weak compatibility.** → State beside both lists that
  the distinction is purpose-only and both follow the same 0.2.x compatibility promise.
- **Compile-only helpers become dead-code noise.** → Use one ordinary test to build and inspect
  inert declarations, and small type assertions only where execution would cause effects.
- **A future legitimate break is made harder.** → The failure is intentional evidence; amend the
  OpenSpec capability and test together in the next SemVer-breaking line.

## Migration Plan

No adopter migration is required because the only export change is additive. Land the `ModuleRule`
re-export, documentation, and compile contract together. Rollback removes only that path and the new
test/spec/docs; no data or runtime state is created.

## Open Questions

None. The current export list and the two reference consumers provide the scope evidence.
