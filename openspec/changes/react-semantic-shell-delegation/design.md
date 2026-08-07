## Context

Semantic emptiness now belongs to `hunyi::check_all`; both `SemanticObserver` and the shell call that public entry point. The existing behavior tests distinguish the public entry point and observer paths, but the shell reads the same manifest for its static dimension before semantic evaluation. A shell-local empty guard therefore produces the same verdict and cannot be distinguished behaviorally.

The observer-protocol test already owns an executed-line `Source` view and a brace-counted recognizer for function bodies. This change extends that infrastructure to observe the shell composition function instead of introducing another parser.

## Goals / Non-Goals

**Goals:**

- Make the shell delegation scenario fail when `evaluate_constitution` independently inspects semantic emptiness.
- Refuse to judge when the composition function cannot be found.
- Keep one shared function-body recognizer for observer and shell source-shape checks.

**Non-Goals:**

- Change semantic evaluation behavior or `evaluate_constitution` itself.
- Freeze unrelated formatting or statement order in the runner.
- Parse arbitrary Rust syntax or govern delegation through Tianheng Constitution boundaries.

## Decisions

- Generalize `bounds_body` into a named-function body recognizer that returns executed source inside the matching braces. `bounds_body` remains a thin wrapper, preserving the existing observer reaction while avoiding a second brace walker.
- Normalize whitespace after whole-line comments have been excluded by `Source`. The semantic accessor must occur exactly once and the normalized body must contain `hunyi::check_all(constitution.semantic_boundaries(), manifest_path)`. A local guard adds another accessor occurrence; aliasing or wrapping the bundle also removes the required direct call.
- Search within `evaluate_constitution`, not the whole runner file. A mention in a test, helper, comment, or unrelated function must not satisfy the reaction.
- Prove the new guard negatively by temporarily reintroducing the former shell-local `is_empty` guard and recording the focused test failure before restoring production source.

## Risks / Trade-offs

- **Lexical observation can reject semantics-preserving refactors** → The spec deliberately claims source ownership, not only behavior; the check pins only the semantic accessor and direct call, leaving surrounding control flow and formatting free.
- **A lightweight brace recognizer is not a Rust parser** → It operates on the repository shared `Source::rust()` view and its limits stay local to the source-shape contract; malformed or missing bodies fail loud rather than pass.
