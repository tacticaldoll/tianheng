## ADDED Requirements

### Requirement: Published structured identity schemas are compatibility-reacted

Every observation dimension SHALL carry an explicit compatibility reaction for every shipped fact
family it owns and every nested discriminator that changes a fact code, field set, or canonical
field value. The reaction SHALL pin the version-2 namespace, fact code, canonical field names, and
representative canonical field values. Each dimension SHALL additionally inspect at least one
violation produced through its real boundary reaction to pin target and rule as outer identity roles
rather than values folded into the finding key.
Adding a fact variant or identity-bearing discriminator variant SHALL require an explicit catalog
decision rather than silently bypassing the reaction. The reaction SHALL NOT freeze human finding
text, complete report JSON, or diagnostic metadata. A helper whose rendered output enters a key
SHALL be documented and tested as wire-sensitive even when it also serves readability.

#### Scenario: Every shipped dimension fact is cataloged

- **WHEN** the compatibility tests compile and run across 圭表, 渾儀, and 漏刻 with all features
- **THEN** every fact variant and identity-bearing nested discriminator owned by each dimension has an exact expected namespace, code, named fields, and representative values

#### Scenario: A new fact family cannot bypass the catalog

- **WHEN** a dimension adds a new fact or identity-bearing discriminator variant without classifying its identity schema
- **THEN** the compatibility reaction fails to compile or fails its test rather than accepting the new wire implicitly

#### Scenario: Canonicalizer polish cannot silently re-key a baseline

- **WHEN** a key-producing canonicalizer changes the byte form of a representative observed value
- **THEN** the compatibility reaction fails even if the human finding remains readable

#### Scenario: Dimension emission preserves the outer identity roles

- **WHEN** a representative boundary in each dimension emits a violation through its production reaction path
- **THEN** the catalog confirms target and rule occupy their outer `ViolationId` roles while the observed fact occupies `finding_key`

#### Scenario: Presentation remains free to change

- **WHEN** only human finding wording or non-identity diagnostic presentation changes
- **THEN** the identity compatibility reaction remains unchanged because it asserts no presentation snapshot
