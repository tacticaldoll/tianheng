## Context

The repository has three kinds of proof with different jobs: unit tests exhaust scanner semantics,
focused examples teach one adoption story, and `self_governance.rs` protects Tianheng's real
architecture. The gap is between them. Several public `Constitution` boundary-family insertion
paths are covered only by unit fixtures, and the self-law bypasses the composed
`check_constitution` entrypoint in favor of separate static and semantic calls.

This change must increase adopter-shaped coverage without inventing false architectural laws merely
to exercise a DSL. It must also avoid multiplying one workspace per rule variant or pinning human
presentation as a test contract.

## Goals / Non-Goals

**Goals:**

- React through every public composed boundary-family path at least once across the owned examples.
- Keep focused teaching examples focused and place breadth in one clearly labeled catalog fixture.
- Make self-governance exercise the production composed library evaluator over its genuine law.
- Make coverage drift reviewable when a boundary family is added, removed, or relocated.

**Non-Goals:**

- Exhaust every rule modifier, parser edge, or finding shape outside its dimension's unit tests.
- Add a generic test harness, new CLI command, new boundary, or new dependency.
- Add a self-law boundary that does not describe Tianheng's actual architecture.
- Freeze text, ANSI, ordering unrelated to deterministic identity, or full JSON snapshots.

## Decisions

### One catalog workspace complements focused examples

Add one excluded `examples/capability-catalog` workspace whose source contains small, named modules
for the boundary families missing from the focused examples. Its law is one composed
`Constitution`; its test inspects the returned structured report, and the shell script confirms the
real CLI projection. Existing composed, standalone, sans-I/O, and unsafe examples keep their current
stories.

Alternative: expand `examples/composed`. Rejected because the funnel example's three-stage narrative
and baseline assertions would become ambiguous under unrelated violations. Alternative: one example
per capability. Rejected because repeated Cargo resolution and harness code add maintenance weight
without a distinct adoption story.

### Cover public families, not every terminal rule

The ledger keys the published 0.2.x composition surface: static crate/module boundaries; signature,
trait-impl, visibility, forbidden-marker, dyn-trait, impl-trait, async-exposure, and unsafe semantic
families; the `sans_io_pure` composed profile; and runtime boundaries. Within static boundaries, the catalog selects rules
that exercise currently absent observation shapes (source classification,
external confinement, and inline call paths); focused examples already own module-import and
severity/baseline behavior. Unit tests remain responsible for every modifier and lexical edge.

The ledger is an explicit test-owned mapping from this frozen family set to example/test. It is not
a production registry or generic value model. A compile-time/exhaustive API mechanism is impossible
for builder methods, so this change does not claim future methods are auto-detected; adding a public
family is already an OpenSpec/API-review event that must deliberately amend the set.

### Assert structured facts, never presentation

Catalog checks match each expected family using structured `BoundaryKind`, stable rule identity,
`FindingKey` namespace/code/fields, and the declared reason/anchor where useful. They do not compare
the full report JSON or finding sentence. This catches a dropped dimension or miswired family while
preserving the presentation freedom established by version-2 identity.

### Self-law uses the composed evaluator without adding fake law

`tianheng_governs_itself` calls `check_constitution` once with the existing live Constitution and
asserts `Outcome::Clean`. Declaration-integrity tests remain separate because they observe the
declaration itself, not governed source. If dimension-specific diagnosis remains useful, it comes
from the returned violation kind/reason rather than parallel evaluator calls. The empty runtime set
still matters: the composed evaluator must run the audit and reject an undeclared production-root
probe.

## Risks / Trade-offs

- **Catalog fixture becomes noisy or slow** → Use one workspace, one metadata scan per composed
  check where possible, and keep rule-edge exhaustiveness in unit tests.
- **A contrived violation teaches bad architecture** → Label the catalog as contract coverage, not
  an onboarding design; keep all narrative examples unchanged.
- **Ledger becomes prose-only bookkeeping** → Back its current mappings with executed structured
  reactions; do not claim it can discover future builder methods automatically.
- **Self-law failure loses dimension-specific assertion text** → Structured violations already name
  kind, rule, target, file, and reason; constitution errors retain precedence.
- **Runtime audit finds macro declarations in Tianheng sources** → The existing macro-body exclusion
  is the intended reaction; an actual reachable undeclared probe must fail self-governance.
