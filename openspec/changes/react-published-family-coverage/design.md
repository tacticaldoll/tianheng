## Context

The governance-dogfood spec names twelve published 0.2.x boundary families and requires a real
reaction owner for each. Those owners are split across isolated example workspaces and Tianheng's
self-governance tests. `scripts/test_examples.sh` is already the repository gate that crosses the
isolated-workspace boundaries, executes the real library and shell reactions, and is run by both
the Definition of Done and CI. Today its structured assertions prove only the identities written
beside each example; there is no executable comparison against the complete published-family set.

Rust source or rustdoc inspection cannot reliably infer this set: public methods also represent
modifiers, operand-scoped depths, and convenience shorthands. Whether a new insertion path is a
new family remains a product-contract decision made during OpenSpec/API review.

## Goals / Non-Goals

**Goals:**

- Make the deliberately published 0.2.x family inventory executable and reviewable.
- Count a family only after its owner has executed a real evaluator and checked the relevant
  structured reaction.
- Fail on both missing inventory coverage and claims for unknown family identities.
- Keep focused examples focused while allowing one example to fulfill several families.

**Non-Goals:**

- Infer family semantics from builder methods, enum variants, source text, or rustdoc.
- Add public Rust family metadata, a generic capability registry, or a new dependency.
- Add family data to production violations, identity, baselines, JSON, SARIF, or projections.
- Require one fixture per modifier, shorthand, or rule variant.

## Decisions

### The cross-workspace examples gate owns the executable ledger

`scripts/test_examples.sh` will declare stable, namespaced repository-only family IDs and record
fulfilled IDs after the existing example block has successfully checked the corresponding real
reaction. Its final gate compares the deduplicated fulfilled set with the declared inventory.

This location already owns cross-workspace execution and is part of the project DoD. Putting the
ledger in a Rust product crate would either make test governance public or fail to see isolated
examples. A separate parser over test source would only move the same hand-maintained claims behind
an unreliable heuristic.

### Claims are coupled to evidence blocks, not free-standing prose

Each `fulfill_family` call will sit after the commands and structured assertions that prove its
owner. A claim made before evidence succeeds cannot be reached; a failing evaluator or missing
identity exits before fulfillment. The final comparison fails if a declared family was never
fulfilled. `fulfill_family` itself rejects names absent from the inventory so stale or misspelled
claims cannot silently accumulate.

The claim remains a small manual bridge because classification is semantic. The gate guarantees
completeness relative to the deliberately reviewed inventory; OpenSpec/API review guarantees that
a genuinely new family is added to that inventory.

### Family IDs describe insertion paths, not every rule

The frozen inventory follows the existing contract granularity: `guibiao/crate`,
`guibiao/module`, the eight semantic insertion paths under `hunyi/*`, `tianheng/sans-io-pure`, and
`louke/runtime`. Operand filters, subtree depth, severity, and shorthands do not create IDs unless a
future OpenSpec deliberately reclassifies them.

### The gate tests both failure directions without duplicating production fixtures

The script-level helper will have focused repository tests or a self-test mode proving that an
unknown claim fails and an unfulfilled inventory member fails. The normal dogfood run proves the
positive path against all real owners. These checks test the ledger mechanism without inventing a
fake architectural boundary.

## Risks / Trade-offs

- **A contributor adds a genuinely new family but omits the inventory entry.** → The semantic
  distinction cannot be inferred mechanically; proposal/API review and an explicit task retain
  this last judgment boundary. Once declared, CI enforces ownership.
- **A claim is placed after an assertion that is too broad.** → Require stable structured kind,
  rule, and/or `FindingKey` evidence appropriate to that family; exit code alone is insufficient
  where another family could keep the example red.
- **The shell ledger becomes a generic test framework.** → Keep only inventory, fulfillment, and
  comparison helpers; example construction and assertions remain in their focused workspaces.
- **Multiple owners obscure responsibility.** → Deduplicate for completeness but print fulfilled
  family IDs in the gate output so overlap remains reviewable.

