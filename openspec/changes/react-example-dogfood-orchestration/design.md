## Context

`check_dod_coherence.sh` already parses the local Definition of Done and CI command streams, but it compares membership only. The four example-dogfood commands may therefore be reordered while every command remains present. It also never reads the positive driver, so that driver may rerun a focused matrix without moving the gate's verdict.

## Goals / Non-Goals

**Goals:**

- React when either authored command stream loses the contiguous focused-matrices-before-driver sequence.
- React when a non-comment source line in the positive driver names a focused matrix basename.
- Derive the expected command sequence and recursion checks from one ordered shell array plus one driver path.
- Prove every new refusal in the existing DoD-coherence failure matrix.

**Non-Goals:**

- Executing the expensive example driver from the coherence gate.
- Parsing arbitrary shell expansion or proving runtime call graphs.
- Changing any example's quality or reaction behavior.
- Combining the intended-release prose finding into this change.

## Decisions

Keep the reaction in `check_dod_coherence.sh`, because that gate already owns the relationship between the local Definition of Done and CI and already reads both command streams. A new gate would duplicate those parsers and require another DoD/CI pair solely to govern four lines inside the surface this gate already owns.

Represent the focused matrices as one ordered shell array and the positive driver as one scalar. Generate the expected `bash <path>` sequence from those values for both local and CI checks, and derive the basenames used by the direct-recursion check from the same array. This prevents three hand-maintained lists inside the reaction.

Require a contiguous sequence, not mere increasing order. The spec says the focused refusals run directly before the positive driver; allowing unrelated commands between them would preserve order while falsifying that authored shape.

Define non-recursion at the observable source level: after dropping full-line shell comments, no remaining driver source line may contain a focused matrix basename. This catches direct invocations including workspace-prefixed paths without claiming that every remaining line is executable or resolving dynamically assembled command names.

## Risks / Trade-offs

- [The source check can reject a basename in a non-comment string or heredoc body] → This is the deliberately narrow authored-form contract; keep those names out of driver code entirely and place explanatory references in comments.
- [A generic contiguous matcher could accept a later duplicate sequence] → Duplicate execution still contains the required sequence and is not this finding; existing command membership and future duplication policy remain separate.
- [The expanded failure-matrix fixture could make older cases fail for the new reason] → Give every base fixture a valid driver and valid sequence, then perturb exactly one property per new case.
