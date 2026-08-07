## MODIFIED Requirements

### Requirement: An empty semantic observer SHALL not read workspace metadata

The semantic dimension's public composed entry point SHALL return `Clean` for an empty boundary bundle without
reading the manifest. The shell and `SemanticObserver` SHALL delegate both empty and non-empty semantic bundles
to that entry point rather than maintaining independent empty-boundary guards, so every semantic composition
path has one behavior owner.

The repository reaction SHALL inspect the executed body of the shell's `evaluate_constitution` composition
function. That body SHALL access `constitution.semantic_boundaries()` exactly once, as the direct boundary
argument to `hunyi::check_all`; a missing function, an additional semantic-boundary inspection, or an indirect
shell-local decision SHALL fail rather than be treated as delegation.

#### Scenario: Empty semantic boundaries through the public semantic entry point

- **WHEN** `check_all` receives an empty semantic boundary bundle and a path that cannot be read
- **THEN** it returns `Clean`, because there is no semantic observation to perform

#### Scenario: Empty semantic boundaries through an observer

- **WHEN** a semantic observer has no boundaries and receives a path that cannot be read
- **THEN** it returns `Clean` by delegating to the public semantic entry point

#### Scenario: Empty semantic boundaries through the shell

- **WHEN** the shell composes a constitution whose semantic boundary bundle is empty
- **THEN** the source-shape reaction finds exactly one semantic boundary access, passed directly to the public semantic entry point, and fails if the shell decides emptiness itself
