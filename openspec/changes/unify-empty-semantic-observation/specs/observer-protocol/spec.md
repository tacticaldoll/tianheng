## MODIFIED Requirements

### Requirement: An empty semantic observer SHALL not read workspace metadata

The semantic dimension's public composed entry point SHALL return `Clean` for an empty boundary bundle without
reading the manifest. The shell and `SemanticObserver` SHALL delegate both empty and non-empty semantic bundles
to that entry point rather than maintaining independent empty-boundary guards, so every semantic composition
path has one behavior owner.

#### Scenario: Empty semantic boundaries through the public semantic entry point

- **WHEN** `check_all` receives an empty semantic boundary bundle and a path that cannot be read
- **THEN** it returns `Clean`, because there is no semantic observation to perform

#### Scenario: Empty semantic boundaries through an observer

- **WHEN** a semantic observer has no boundaries and receives a path that cannot be read
- **THEN** it returns `Clean` by delegating to the public semantic entry point

#### Scenario: Empty semantic boundaries through the shell

- **WHEN** the shell composes a constitution whose semantic boundary bundle is empty
- **THEN** it delegates that bundle to the public semantic entry point instead of deciding emptiness itself
