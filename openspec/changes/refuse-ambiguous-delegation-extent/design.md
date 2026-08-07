## Context

`function_body` in `crates/tianheng/tests/observer_protocol.rs` finds a function's extent by counting braces
over a mask that blanks braces inside line-comment tails. It has two consumers with different comparisons:

- `bounds_body`, which requires the extent to equal exactly one statement. Any moved extent fails that
  equality, so the error direction is refusal.
- `the_shell_delegates_semantic_emptiness_to_the_public_entry_point`, which asserts
  `matches("constitution.semantic_boundaries()").count() == 1` and a containment. Both survive truncation, so
  the error direction is acceptance.

Measured with verbatim copies of the two functions compiled standalone, three constructs move the extent and
make the delegation reaction pass on a body carrying a second semantic-boundary access:

| Construct in the body | Second access | Reaction |
|---|---|---|
| none (control) | present | fails, correctly |
| `//` inside a string, block braces split across lines | present | **passes** |
| bare `}` inside a string literal | present | **passes** |
| bare `}` inside a block comment | present | **passes** |

The first route is the one an external review reported. The second and third are the plainly declared route of
the existing bound `observer-protocol/a-brace-inside-a-block-comment-or-a-string-literal-moves-the-read-body-extent-a-stated-bound`,
whose `Extent` is `Reached::OverReacts` and whose spec prose justifies acceptance with "no brace-carrying
construct survives the exact one-statement comparison". That justification was true when `bounds_body` was the
only consumer. The delegation reaction was added afterwards and the rationale was not re-derived against it, so
a bound typed on the harmless side of the false-negative line now also covers a consumer on the harmful side.

A fourth route follows from the same mechanism and appears in none of the reports: a character literal,
`let c = '}';`, moves the extent identically.

`crates/tianheng/src/bounds.rs` is absent at `v0.4.0`, so the bound identities are unreleased surface.

## Goals / Non-Goals

**Goals:**

- The delegation reaction cannot report a divergent body as delegating. Where it cannot be sure of its extent,
  it refuses.
- Each declared bound's `Extent` states what that consumer actually does, so the register stops describing an
  under-reaction as an over-reaction.
- Both closures carry a negative run seen to fail before the fix.

**Non-Goals:**

- String-literal lexing. The existing scenario records it measured and rejected, with a concrete defeat: this
  tree's own lexer suites put comment delimiters inside string literals, several nested, so a
  delimiter-counting scan opens a phantom comment at the first and swallows every definition to the next close.
  Nothing here disturbs that finding.
- Changing `bounds_body`'s comparison or its bound. Its over-reaction is genuinely safe and stays declared.
- Making `function_body` correct in general. It stays the deliberately lightweight positional recognizer the
  spec already describes; the change is to what its *consumers* do when its answer cannot be trusted.
- Editing `crates/tianheng/src/runner.rs`. It is the reaction's subject, not its target.

## Decisions

**Refuse on delimiter presence, not on a parse.** The helper scans the extent's executed lines for `"`, `'`,
`/*`, or `*/` and refuses if any appears. Alternatives considered: (a) a hand-rolled string-aware lexer —
rejected, it is exactly the measured-and-rejected approach and would re-enter the phantom-comment failure;
(b) comparing against the whole file instead of the extent — rejected, the file legitimately mentions
`semantic_boundaries` elsewhere, so the count assertion would be meaningless; (c) accepting the bound and
documenting it — rejected, the bound would then declare a false negative as policy, which is what
`docs/observation-bounds.md` exists to prevent rather than to record.

**Scan executed lines, not raw text.** `Executed` already filters `//` lines, so a quote inside a line comment
does not trip the refusal. This matters immediately: the tracked body's comments are prose-heavy and a raw scan
would refuse on the first apostrophe someone writes.

**Include `'` despite the lifetime collision.** A character literal moves the extent exactly as a string does,
so omitting it would leave the same hole one keystroke away. The cost is that a composition body naming a
lifetime is refused. Accepted: over-refusal is loud and repairable in the commit that causes it, and the
tracked body carries no lifetime today — verified, no executed line of `evaluate_constitution` holds any of the
four delimiters.

**Include `*/` as well as `/*`.** A block comment opened before the signature and closed inside the body would
otherwise present a closing delimiter with no opener in the extent. Cheap to include, and it removes a residual
that would otherwise need declaring.

**Split the bound rather than re-word one.** One `BoundId` cannot carry two `Extent` values, and the two
consumers now genuinely differ: `bounds_body` still over-reacts, the delegation reaction refuses. Two pinned
declarations state that; one reworded declaration would have to pick a side. Free to do now — the identities
have never shipped — and no longer free after the `0.5.0` squash.

**Extract the judgement before fixing it.** The reaction currently reads the tracked `runner.rs` directly, so
no fixture can drive it and no negative run is possible. A helper over `&Source` is a prerequisite for the
evidence, not a refactor for its own sake.

## Risks / Trade-offs

- **The refusal could pass vacuously by refusing everything** → the spec carries a scenario requiring the
  tracked body to still be *judged*, pinned by its own test alongside the refusal test. A refusal-only
  implementation fails it.
- **Over-refusal surfaces later as a confusing stop** when someone adds a string or lifetime to
  `evaluate_constitution` → the refusal message names the delimiter it found and says the extent may not be the
  body, so the reader meets an explanation rather than a bare failure.
- **The ambiguity scan is extent-scoped**, so in principle a construct beyond the cut is unseen → for all four
  measured routes the delimiter necessarily sits at or before the cut, because the delimiter is what causes the
  cut. Including `*/` closes the one asymmetric case. Not claimed as exhaustive; if a route is found where the
  delimiter falls outside the extent, that is a new bound and a new pin, not a silent widening.
- **The bijection in `observation_bound_model` may require one scenario per bound** → surfaces immediately when
  the model test runs; if it does, the second bound gets its own scenario, which the delta spec already
  provides.
- **Five sites move together** — typed declaration, two spec requirements, and two generated projections. A
  partial landing fails `check_bound_register.sh`, which is the intended interlock rather than a hazard.

## Migration Plan

None required. No product code changes, no adopter-visible surface moves: the recognizer ships in no crate, and
the renamed bound identities are absent at `v0.4.0`. The change must land before the `0.5.0` release squash,
after which `tianheng::observation_bounds()` is published and a bound identity becomes adopter-visible.

## Open Questions

- Whether `observation_bound_model`'s bijection accepts two bounds whose scenarios sit under different
  requirements of the same capability. Resolved by running it in task 6; the delta spec is written so either
  answer needs no spec rewrite.
