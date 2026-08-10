## ADDED Requirements

### Requirement: Repository-governance bound catalogs SHALL be owned outside the product shell

Every observation-bound declaration qualifying an unpublished repository-governance reaction SHALL be returned by the unpublished crate that owns that reaction: Kanhe for repository record and coherence checks, and Shengmo for self-law dogfood. The repository observation-bound model SHALL compose those catalogs explicitly with each product dimension's observer catalog and SHALL hold the combined declarations in the existing id and specification bijection.

The published Tianheng shell SHALL NOT define an `observation_bounds` catalog entrypoint for repository declarations. Its product dimensions already declare their bounds through their observers, and an empty shell catalog would preserve a capability with no reaction.

#### Scenario: A repository declaration remains in the product shell

- **WHEN** Tianheng's tracked Rust source defines the exact `observation_bounds` catalog vocabulary
- **THEN** the repository ownership guard fails, because that entrypoint has no product-shell reaction to qualify

#### Scenario: A declaration is lost or duplicated during relocation

- **WHEN** a moved declaration appears in neither owner catalog or in both catalogs
- **THEN** the combined repository model fails its spec bijection or duplicate-id check

#### Scenario: Product dimension declarations are composed

- **WHEN** the repository model enumerates every declaration
- **THEN** it consumes each product dimension through its observer and consumes Kanhe and Shengmo through their unpublished catalogs, so the shell does not restate dimension membership

## REMOVED Requirements

### Requirement: The published shell bound catalog SHALL exclude Kanhe-owned declarations

**Reason**: Excluding one repository capability prefix leaves every other Kanhe- and Shengmo-owned declaration in a product catalog with no product-shell reaction.

**Migration**: Remove the shell catalog, return repository declarations from their unpublished owner crates, and compose those catalogs only in the repository model.
