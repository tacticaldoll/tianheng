## ADDED Requirements

### Requirement: Optional baseline metadata has a strict input type

For every version-1 and version-2 baseline entry, the parser SHALL accept `owner` and `tracker` only
when each field is absent, JSON null, or a JSON string. Absent and null fields SHALL represent no
annotation, while a string SHALL be preserved as metadata. Any other JSON type SHALL make the
baseline malformed rather than silently removing the annotation. Generated baselines SHALL continue
to omit unset metadata fields.

#### Scenario: Omitted and null metadata are absent

- **WHEN** an entry omits `owner` or `tracker`, or supplies either field as JSON null
- **THEN** the baseline parses with that annotation absent and serialization omits the unset field

#### Scenario: String metadata is preserved

- **WHEN** an entry supplies `owner` or `tracker` as a JSON string
- **THEN** the baseline preserves that exact string across parse and serialization

#### Scenario: Wrong-typed metadata invalidates the baseline

- **WHEN** a present `owner` or `tracker` is a number, boolean, array, or object
- **THEN** baseline parsing fails with an error identifying the malformed metadata field

#### Scenario: Gating fails loud on malformed metadata

- **WHEN** `tianheng check --baseline` reads a baseline containing wrong-typed optional metadata
- **THEN** the command reports an invalid baseline and exits as a scan error rather than gating with the annotation silently absent

#### Scenario: Explicit rewrite warns before metadata loss

- **WHEN** `tianheng check --write-baseline` reads a prior baseline containing wrong-typed optional metadata
- **THEN** it warns that the prior baseline could not be parsed and that metadata will not be carried forward before writing a fresh snapshot
