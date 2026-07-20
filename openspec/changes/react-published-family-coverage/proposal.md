## Why

The published-family dogfood ledger can prove that every family it lists still reacts, but it
cannot fail when the repository's deliberate family inventory and its executed owners diverge.
The 0.2.x catalog now supplies enough real reaction owners to close that bookkeeping loop without
inventing public capability metadata or changing production behavior.

## What Changes

- Establish one repository-owned inventory of the published 0.2.x boundary families.
- Require each inventory member to be fulfilled by at least one owner that executes the real
  evaluator and observes that family's structured reaction.
- Reject owner claims for unknown families, while allowing multiple genuine owners for one family.
- Keep the semantic decision that a new builder insertion path is a family, depth, modifier, or
  shorthand in OpenSpec/API review rather than pretending source inspection can infer it.
- Keep the inventory and coverage evidence out of public Rust APIs, violations, baselines, and
  serialized reports.

## Capabilities

### New Capabilities

None.

### Modified Capabilities

- `governance-dogfood`: Replace the frozen prose-only coverage ledger with an executable
  repository inventory compared against fulfilled adopter-shaped reaction owners.

## Impact

The change affects repository-owned governance tests, capability-catalog assertions, example test
support, and the governance-dogfood specification. It adds no runtime dependency, public API,
manifest or package-version change, evaluator behavior, finding identity, baseline wire, or report
field.
