## Context

`AGENTS.md` is the source of the local DoD command list, CI repeats that list, and the existing gate-shape
reaction requires every enumerated gate and twin to appear there. The three focused example matrices already
appear immediately before `test_examples.sh` in both places. Their second invocation inside `test_examples.sh`
duplicates work and splits orchestration ownership between two layers.

## Goals / Non-Goals

**Goals:**

- Give the top-level DoD/CI list sole ownership of matrix-before-driver ordering.
- Preserve direct visibility and gate-shape reachability for every focused matrix.
- Keep `test_examples.sh` focused on positive isolated-workspace quality and reaction paths.

**Non-Goals:**

- Combine the three focused matrices into one script.
- Change example coverage, quality gates, reaction assertions, or artifact cleanup.
- Add a new shell call-graph detector when the existing gate-shape reaction already keeps every focused matrix
  directly reachable from DoD.

## Decisions

### Remove nested calls, not top-level commands

Removing the DoD/CI commands would violate the existing gate-shape contract and hide each focused proof behind
another script. Removing the nested calls preserves the explicit reaction inventory and makes the established
top-level order authoritative.

### Verify at the moved observation level

Before the refactor, running `test_examples.sh` emits the focused matrices' success markers before any example.
Afterward, the driver does not emit those markers, while running the three matrices followed by the driver still
passes. The existing gate-shape reaction continues to guard that every matrix remains directly reachable from
DoD; review of the driver observes the non-recursion half without adding a general shell call-graph detector for
one runtime-cost class.

## Risks / Trade-offs

- Invoking `test_examples.sh` alone no longer runs failure matrices. Its header and purpose are the positive
  example reactions; the complete acceptance sequence remains the documented DoD.
- The non-recursion rule is durable specification, but not a new Tianheng self-law boundary: reintroducing
  duplication would waste gate time rather than create an architectural false negative, so a general detector
  would outweigh the failure class.
