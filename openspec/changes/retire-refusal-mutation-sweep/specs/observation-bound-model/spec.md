## ADDED Requirements

### Requirement: The published shell bound catalog SHALL exclude repository-check declarations

The public `tianheng::observation_bounds()` catalog SHALL contain no bound id under the `rust-repository-reactions/` capability. Surviving declarations for that capability SHALL be returned by a Kanhe-owned catalog that the repository model gate consumes separately. That capability governs this repository's unpublished checks, so emitting its declarations from a published crate crosses repository test support into the product surface.

#### Scenario: A repository-only bound enters the published catalog

- **WHEN** `tianheng::observation_bounds()` returns a bound whose id begins `rust-repository-reactions/`
- **THEN** the repository's observation-bound model gate fails and names the leaked id, even if Kanhe also returns the same declaration

#### Scenario: Repository-only bounds are removed

- **WHEN** the mutation sweep and its declarations are retired from Kanhe and the repository spec
- **THEN** the published catalog omits the capability, Kanhe returns only its surviving declarations, and the combined spec and typed declaration sets stay in bijection
