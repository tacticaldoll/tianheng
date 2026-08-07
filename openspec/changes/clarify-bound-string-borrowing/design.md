## Context

The observation-bound model deliberately uses `Cow<'static, str>` so runtime-computed declarations are
expressible and the family's literal string values remain borrowed. `borrows_every_string()` exhaustively visits
those string positions. A `BoundDecl` also contains non-string storage: `Defence::PinnedBy.additional` is a
`Vec`, which may allocate independently of whether its elements are borrowed. The current prose crosses that
observation boundary by presenting the string predicate as an allocation audit for a governance run.

## Goals / Non-Goals

**Goals:**

- Make every affected surface describe exactly what `borrows_every_string()` observes.
- Retain the requirement that every string position is checked exhaustively.
- Preserve the existing negative matrix for owned strings in each position.

**Non-Goals:**

- Promise or implement allocation-free `BoundDecl` construction.
- Replace the multi-pin `Vec` or add an allocation-measurement facility.
- Change the observation-bound reaction, public API, or serialized output.

## Decisions

### Name string ownership directly

Use “borrows every string it carries” for the positive contract and “owns a string value” for its negative.
Explicitly state that non-string containers and surrounding-run allocations are outside this predicate.

Keeping “allocates nothing” with a qualifier was rejected because readers can still reasonably apply it to the
whole declaration. Adding allocation instrumentation was rejected because no requirement asks the model to
measure allocations and `borrows_every_string()` has a complete, useful string-ownership contract already.

### Preserve the implementation and reaction

Rename the broad test and narrow comments, but keep the pointer-identity assertion and the exhaustive
`borrows_every_string()` matrix unchanged. This is a contract-accuracy correction, not a behavioral change, so
no new guard is introduced.

## Risks / Trade-offs

- **Readers may infer that borrowing strings is a performance guarantee** → The API docs and canonical spec
  explicitly name the unobserved non-string and surrounding allocations.
- **A future allocation goal may be mistaken for this contract** → Such a goal requires its own observation
  source and reaction rather than extending a string-ownership predicate by prose.
