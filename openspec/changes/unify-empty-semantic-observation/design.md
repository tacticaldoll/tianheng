## Context

`SemanticBoundaries::is_empty()` is currently consulted in two consumers: Tianheng's built-in composition path
skips `hunyi::check_all`, and `SemanticObserver::observe` returns `Clean` directly. The composed semantic entry
point itself does not short-circuit, so a direct caller with empty boundaries still invokes `cargo metadata` and
can return a constitution error. That disagrees with both consumers and leaves the observer's stated delegation
conditional on input shape.

The accepted law keeps 渾儀 responsible for semantic observation and forbids sharing a scanner across
dimensions. Empty semantic behavior therefore belongs inside 渾儀's existing composed entry point, not in the
shell.

## Goals / Non-Goals

**Goals:**

- Give every semantic composition path the same empty-boundary outcome.
- Keep one owner for the empty-boundary decision.
- Preserve the single metadata read and existing outcomes for every non-empty semantic declaration.
- Make `SemanticObserver` documentation true for both empty and non-empty calls.

**Non-Goals:**

- Change static or runtime composition.
- Change metadata scanning for any non-empty semantic capability.
- Change public signatures, boundary identities, baselines, law, dependencies, or package versions.
- Add a general abstraction for dimension emptiness; the dimensions have deliberately independent reactions.

## Decisions

### `hunyi::check_all` owns the empty-boundary result

Place the `SemanticBoundaries::is_empty()` guard before `read_metadata` in the public composed semantic entry
point. Then remove the equivalent guards from the shell and `SemanticObserver`, so both paths delegate all
semantic boundary bundles to the same function.

Keeping the guard in either consumer was rejected because it preserves multiple semantic entry behaviors and
makes direct `check_all` callers observe a different result. Introducing a shell helper or a shared cross-crate
utility was rejected because the decision is specific to semantic observation and already has a natural owner.

### The reaction observes the changed public entry point

Add a `hunyi` test that calls `check_all` with an empty boundary bundle and a manifest path that does not exist.
It must return `Clean`. Before the change this reaches metadata and fails, so the test observes the behavior that
moves rather than merely re-proving the already-clean observer wrapper.

The existing `SemanticObserver` empty-manifest test remains useful as its public contract check, but it is not
claimed as the new guard because it passed before centralization.

### Observer prose describes delegation, not an unconditional read

Document that `SemanticObserver` delegates to the dimension's composed entry point, which reads its own metadata
when semantic boundaries require observation. This retains 三儀 independence without falsely saying an empty
observation performs I/O.

## Risks / Trade-offs

- **A direct caller may have used empty `check_all` as manifest validation** → That behavior is outside the
  semantic evaluation contract; callers that need workspace validation must request it explicitly rather than
  declaring no semantic observation.
- **A future consumer may reintroduce its own empty guard** → The specification assigns the decision to the
  public semantic entry point, and source review can reject a second owner without adding a brittle prose/code
  detector.
