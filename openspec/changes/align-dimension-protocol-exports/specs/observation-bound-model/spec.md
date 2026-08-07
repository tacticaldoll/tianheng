## ADDED Requirements

### Requirement: Each standalone dimension SHALL expose its shared protocol vocabulary

Each dimension root SHALL re-export the shared types an adopter needs to name its public observation-bound and
observer surface: `BoundDecl`, `BoundId`, `Defence`, `Demonstrates`, `Extent`, `FactGranularity`, `Observer`,
`Outcome`, `Owner`, and `Reached`. The exports SHALL preserve the original `xuanji` type identities rather than
introducing dimension-specific wrappers. An adopter depending on one dimension SHALL NOT need a direct `xuanji`
dependency merely to use that dimension's public protocol.

#### Scenario: An adopter depends on one dimension

- **WHEN** an external integration test imports the complete shared protocol vocabulary from any one dimension root
- **THEN** every type resolves and can be used with that dimension's declarations and observer implementation
