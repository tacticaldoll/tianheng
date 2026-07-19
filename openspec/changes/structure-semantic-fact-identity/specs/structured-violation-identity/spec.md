## MODIFIED Requirements

### Requirement: Observation dimensions own fact meaning and rendering

Each observation dimension SHALL own the typed fact schemas it can observe and the conversion from
each fact to its structured key and human finding text. The shared reaction crate SHALL own only the
dimension-agnostic key envelope and SHALL NOT contain crate-, module-, semantic-, or runtime-specific
fact vocabulary. A dimension SHALL derive the key and text from the same typed fact conversion so
their relationship has one implementation source. When a fact contains separately observed,
identity-bearing components, its dimension SHALL preserve their roles as fact-specific named key
fields rather than first concatenate them into a presentation string and store that string as one
opaque field.

#### Scenario: A dimension introduces a new fact shape

- **WHEN** an observation dimension begins reacting to a new kind of observed fact
- **THEN** its schema and rendering are added in that dimension while the shared key envelope remains unchanged

#### Scenario: Shared identity does not reverse the dependency graph

- **WHEN** all dimension fact schemas are compiled with the shared reaction model
- **THEN** every observation dimension depends inward on the shared model and the shared model depends on no observation dimension

#### Scenario: Semantic presentation changes without re-identifying the fact

- **WHEN** 渾儀 changes only the human wording that combines an exposure subject and its structured public seam
- **THEN** the finding text changes while the fact code and named subject/seam key fields remain unchanged

#### Scenario: Semantic seams remain injective without rendered descriptors

- **WHEN** the same semantic subject is observed at two public seams that differ by item kind, module, owner, member, or trait-impl position
- **THEN** the two finding keys differ in a named seam field, so baselining one observation cannot mask the other
