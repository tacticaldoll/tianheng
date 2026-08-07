## Why

The reaction that holds the shell to delegating semantic emptiness reads `evaluate_constitution`'s body by
counting braces, and a string literal or block comment inside that body moves the extent it reads. The
truncated extent drops a second `constitution.semantic_boundaries()` access, so the count-and-contains
comparison passes on the exact shape the requirement refuses — a false negative, which the Core Contract
names as the one forbidden bug.

The gap is invisible today because a bound already claims this shape and classifies it as safe. That
classification was read off `bounds_body`'s exact one-statement comparison, where any moved extent refuses a
conforming body. The delegation reaction arrived later with a different comparison, and for it the same moved
extent accepts a divergent one. One declared bound now describes two consumers that fail on opposite sides of
the false-negative line, and it is typed for the harmless one.

## What Changes

- The shell-delegation judgement moves out of the test body into a helper over a `Source`, so it can be driven
  by fixtures rather than only by the tracked `crates/tianheng/src/runner.rs`.
- That helper **refuses to judge** when the extent it read carries a `"` or `/*` on an executed line, instead of
  asserting over text it cannot be sure is the function's body. A refusal is loud; the present silent pass is
  not.
- The declared bound splits in two, each pinned by its own test: the surviving over-reaction on `bounds_body`'s
  one-statement comparison, and a new refusal on the delegation reaction. Their `Extent` classifications become
  `Reached::OverReacts` and `Reached::RefusesToJudge` respectively.
- The scenario prose that justifies the bound — "the error direction is the safe one … refuses a **conforming**
  body rather than accepting a divergent one" — is scoped to the comparison it is actually true of.
- String-literal lexing is **not** adopted. The existing scenario records it as measured and rejected, and
  nothing here disturbs that finding.

Not **BREAKING**. The recognizer is a test-support reaction that ships in no crate, so no adopter's recorded
baseline moves. The bound identities being renamed live in `tianheng::observation_bounds()`, which is absent at
`v0.4.0` and has therefore never shipped; re-declaring them asks nothing of anyone.

## Capabilities

### New Capabilities

None. The obligation already exists; this change makes the reaction that carries it stop passing on a shape it
was never able to judge.

### Modified Capabilities

- `observer-protocol`: two requirements change.
  - *An empty semantic observer SHALL not read workspace metadata* gains the refusal — the delegation reaction
    must distinguish "the body does not delegate" from "the body could not be read", rather than reporting the
    second as the first.
  - *The built-in path SHALL keep its behaviour, and the two paths SHALL be held equal* has its brace-extent
    bound scoped to the one-statement comparison, and no longer claims a safe error direction for a consumer
    where the direction is unsafe.

## Impact

- `crates/tianheng/tests/observer_protocol.rs` — the delegation helper, the refusal, the two false-negative
  fixtures, and the two pins.
- `crates/tianheng/src/bounds.rs` — the split bound declarations. Unreleased public surface.
- `openspec/specs/observer-protocol/spec.md` — the two modified requirements, at sync.
- `docs/observation-bounds.md` and `docs/observation-bound-extents.md` — regenerated, never hand-edited.
- `CHANGELOG.md` — an `[Unreleased]` entry.

No product code changes. `crates/tianheng/src/runner.rs` is the reaction's subject and is not itself edited;
the refusal makes adding a string literal to `evaluate_constitution` a loud stop rather than a silent one, which
is a maintenance consequence the reason must state.
