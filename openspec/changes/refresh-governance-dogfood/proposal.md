## Why

Tianheng's examples explain the adoption funnel well, but they exercise only a curated subset of
the boundary families now shipped in 0.2.x, while the repository's self-law still evaluates its
static and semantic slices separately instead of dogfooding the public composed evaluator. A
capability can therefore remain unit-tested yet disappear from the adopter-shaped and self-hosted
paths without a focused reaction.

## What Changes

- Make the repository self-governance gate execute its live Constitution through
  `check_constitution`, including the runtime audit's always-run empty-declaration behavior, while
  retaining focused declaration-integrity assertions where they diagnose a distinct contract.
- Add one isolated, intentionally violating capability-catalog example that exercises every public
  boundary family not already represented by the focused examples, without turning the composed
  funnel example into a kitchen sink.
- Extend the examples dogfood script to assert each catalog reaction by stable structured fields
  (`kind`, `rule`, `finding_key`, and `reason`), not presentation text or ANSI layout.
- Record an explicit coverage ledger for the exact 0.2.x public boundary-family set so each shipped
  family has a reviewed adopter-shaped dogfood home; future API additions remain an OpenSpec review
  concern rather than pretending builder methods are mechanically enumerable.
- Refresh BACKLOG wording to distinguish completed 0.2.x dogfood from deliberately deferred product
  weight; add no new boundary type, CLI surface, dependency, or release-version change.

## Capabilities

### New Capabilities

- `governance-dogfood`: Defines the repository-owned reaction coverage for the public boundary
  families across self-governance and isolated runnable examples.

### Modified Capabilities

- `composed-library-check`: Requires Tianheng's own self-governance gate to exercise the same
  composed evaluator adopters call, including runtime audit ordering and error precedence.

## Impact

Affected areas are `crates/tianheng/tests/self_governance.rs`, its generated
`AGENTS.self-law.md` only if the declaration itself changes, a new excluded example workspace,
`scripts/test_examples.sh`, OpenSpec specifications, and BACKLOG/README navigation. Production APIs,
finding identity, baseline wire formats, manifests for published crates, and package versions remain
unchanged. The catalog is test-only product documentation: it may add CI time, so it must reuse one
fixture workspace and prove each reaction without duplicating dimension internals.
