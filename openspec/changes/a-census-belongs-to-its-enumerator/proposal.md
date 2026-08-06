# Change: a census belongs to whatever enumerates the set

## Why

Hand-written figures drifted **in every kind of place they can live** during this one release window, and each was
found by hand rather than by a reaction:

| Where | Wrote | Truth |
| --- | --- | --- |
| a **code doc** in `xuanji/src/tests.rs` | fifty-three declarations | fifty-four |
| a **BACKLOG** entry | 55 declared bounds | 56 |
| a **CHANGELOG** sentence, no time anchor | 54 classified bounds | 56 |
| **three files at once** | "the eight files under `src/runner/`" | correct, and stale on the next file |
| a **BACKLOG** entry | "all five gate matrices" | six gates today |
| the **version-horizon paragraph** — the one that assigns the release number | "of the 44 commits … the two other product-code touches … nothing else is packaged" | 78 commits; 20 packaged sources touched, 9 of them new |
| a **generated projection's template** | "Three of the six classes … a fourth bound is about coverage" | five declared bounds; one added this window went unlisted |

The last row is the sharp one. `docs/gate-shape-contract.md` is generated and staleness-checked, and its bound
disclosure still went wrong — because the figure and the list are **string literals in the generator**. The
freshness check compares the generator's own text with itself, so it is `f() == f()` for exactly those figures.
**A projection cannot self-correct a number it does not compute.**

And a detector over prose is not the answer. It was designed and measured three times, each time refuted:

- widening the recognized phrasing to `N declared bounds` false-positives on both generated projections' own
  headers, on the register's diagnostic string, and on six expected-output literals in its failure matrix;
- widening the corpus from tracked Markdown to `scripts/` false-positives on the fixture censuses that matrix
  writes deliberately;
- and the one instance that occurred in a code doc was spelled **in words**, which no digit-based matcher reads.

Most numbers here describe a *shape* — "two files of one module yield one violation" — not a census, so a matcher
over numbers is mostly false positives.

## What Changes

- **`AGENTS.md` gains the rule**: a figure saying how many members a set in this repository currently has is
  **produced, never typed** — printed by the reaction that enumerates it, or *computed* into a generated
  projection. Where nothing enumerates the set, anchor the figure to a past moment or drop it. It records why a
  prose detector is the wrong instrument, so the rejected design is not re-proposed.
- **`gate-shape-contract`'s projection derives its bound disclosure** from the specification and is held to it in
  **both** directions, with the figure as the list's length. A single array carries only the *explanations*.
- The drifted figures are corrected — and where the claim never needed a number, it loses it rather than gaining a
  fresher one.

## Impact

- Affected specs: `gate-shape-contract`, `observation-bound-model`
- Affected code: `crates/tianheng/tests/gate_shape_contract.rs`, `docs/gate-shape-contract.md` (regenerated)
- Affected docs: `AGENTS.md`, `BACKLOG.md`, `CHANGELOG.md`
- No public API change, no version bump. `PROJECT.md` and `README.md` were swept and hold no census — their
  numbers describe shapes, and `PROJECT.md`'s "twenty pull requests, ten review rounds" is anchored to the
  pre-register period in the past perfect.
