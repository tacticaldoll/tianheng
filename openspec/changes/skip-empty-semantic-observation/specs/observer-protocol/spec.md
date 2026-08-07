## ADDED Requirements

### Requirement: An empty semantic observer SHALL not read workspace metadata

A semantic observer with no declared boundary SHALL return `Clean` without reading the manifest. This SHALL
match the built-in composition path's empty-dimension behaviour, including when the supplied manifest does not
exist.

#### Scenario: Empty semantic boundaries with an unreadable manifest

- **WHEN** a semantic observer has no boundaries and receives a path that cannot be read
- **THEN** it returns `Clean`, because there is no semantic observation to perform
