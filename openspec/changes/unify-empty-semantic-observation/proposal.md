## Why

Empty semantic declarations are short-circuited independently by the shell and by `SemanticObserver`, while
the public `hunyi::check_all` entry point still reads workspace metadata. The three entry paths can therefore
disagree for the same empty boundary bundle, and the observer's delegation contract is only true for non-empty
input.

## What Changes

- Make the semantic dimension's public composed entry point own the empty-boundary result.
- Have the shell and `SemanticObserver` delegate empty and non-empty semantic evaluation to that one entry
  point instead of maintaining their own guards.
- Hold the empty-boundary behavior at the shared entry point with a negative-before-positive reaction.
- Describe `SemanticObserver` as delegating to semantic evaluation, without claiming that every call reads
  workspace metadata.

## Capabilities

### New Capabilities

None.

### Modified Capabilities

- `observer-protocol`: require every semantic composition path to delegate empty-boundary handling to the
  dimension's public composed entry point.

## Impact

The change touches semantic composition in `hunyi`, the Tianheng shell's semantic arm, observer documentation,
and observer-protocol tests/specification. Non-empty evaluation, public signatures, violation identity,
dependencies, manifests, package versions, and the accepted Tianheng law remain unchanged.
