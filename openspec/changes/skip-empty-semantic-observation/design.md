## Context

`evaluate_constitution` already avoids semantic evaluation when the constitution contains no semantic boundaries. The trait-driven path always invokes `SemanticObserver::observe`, whose current implementation delegates to `check_all`; `check_all` reads metadata before evaluating its boundary slice. The mismatch is observable only when that unnecessary read fails or costs work, which existing parity fixtures excluded by making every dimension non-empty.

## Goals / Non-Goals

**Goals:**

- Make empty semantic participation contribute the same clean outcome on both composition paths.
- Prove that no manifest read occurs, not merely that a valid manifest eventually produces no finding.
- Keep non-empty semantic observation byte-for-byte on the existing `check_all` path.

**Non-Goals:**

- Add a general `Observer::is_vacuous` protocol method.
- Change how an entirely empty composed run is classified.
- Remove the built-in path's existing empty-dimension short circuit.

## Decisions

1. Put the early return inside `SemanticObserver::observe`. The semantic dimension owns what an empty boundary set means; adding a protocol method would break every third-party observer for a fact only this implementation needs.
2. Return `Outcome::Clean` before calling `check_all`. This matches the built-in path's contribution semantics while keeping the composed run's separate no-observer refusal unchanged.
3. Test with a guaranteed-nonexistent manifest path. A test using a real workspace would pass before and after the fix and prove only the surrounding contract, not the removed I/O.

## Risks / Trade-offs

- [Hidden future setup in `check_all`] An empty observer will skip any future unconditional setup added there. → The contract explicitly says no semantic observation means no workspace I/O; future setup must be justified by a non-empty boundary.
- [Parity test remains broad] The existing cross-path fixture still uses non-empty dimensions. → The focused semantic unit test owns the empty cell with a discriminatory unreadable path.
