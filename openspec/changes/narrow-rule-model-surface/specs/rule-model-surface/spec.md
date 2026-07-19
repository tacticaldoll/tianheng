## ADDED Requirements

### Requirement: Boundary builders own public rule construction

The public `Rule` and `ModuleRule` enums SHALL remain readable model types, but every data-carrying
variant SHALL be non-exhaustive. External consumers SHALL construct crate and module rules through
the existing boundary DSL rather than a variant struct expression. The DSL names and sequencing for
`Constitution`, `CrateBoundary`, `ModuleBoundary`, every existing rule method, `.strict_external()`,
`.because(...)`, severity, anchor, and `run` SHALL remain available.

#### Scenario: Existing adopter DSL still constructs a rule

- **WHEN** an external consumer declares any existing crate or module boundary through its builder
- **THEN** the declaration compiles and produces the same rule, reaction, and projection as before

#### Scenario: Direct external variant construction is closed

- **WHEN** an external consumer attempts to construct a data-carrying `Rule` or `ModuleRule` variant with a struct expression
- **THEN** compilation fails because the variant is non-exhaustive and the consumer must use the boundary DSL

### Requirement: Rule variants remain forward-compatibly inspectable

`Rule` and `ModuleRule` and their existing variant names SHALL remain public.
`CrateBoundary::rule()` SHALL remain available and `ModuleBoundary` SHALL add the symmetric
read-only `rule()` accessor, so both model enums are obtainable from builder-produced boundaries. An
external consumer SHALL be able to inspect known fields of a known variant by using an open-ended
`Variant { known_field, .. }` pattern and the wildcard arm required by the enum's non-exhaustiveness.
Adding a field to that variant later SHALL NOT break such a match.

#### Scenario: An external consumer matches a known field

- **WHEN** an external consumer obtains a crate or module rule from a built boundary's `rule()` accessor and matches a known variant field with `..`
- **THEN** the match compiles and reads that field without assuming the variant's complete representation

#### Scenario: Module rules gain a read-only path

- **WHEN** an external consumer builds a `ModuleBoundary`
- **THEN** `ModuleBoundary::rule()` returns its `ModuleRule` by shared reference without exposing mutable representation or a second construction path

#### Scenario: A closed-field match has an explicit migration

- **WHEN** a consumer previously matched every named field of a rule variant without `..`
- **THEN** adding `..` preserves its read-side behavior under the narrowed model contract

### Requirement: Strict-external is one inline-rule modifier

The public builder SHALL represent `.strict_external()` as a modifier of the existing inline symbol
confinement rule rather than as a second public rule variant. The modifier SHALL preserve the
existing default-off behavior, local-precedence classification, constitution errors, human and JSON
projections, polarity, and violation identity. Adding or removing `.strict_external()` on an
otherwise identical boundary SHALL continue to leave the violation's target, rule, and finding key
unchanged.

#### Scenario: Default inline confinement stays default-off

- **WHEN** a boundary uses `must_not_call_inline(prefix)` without `.strict_external()`
- **THEN** it retains the existing default resolver behavior and omits `strict_external` from its projection

#### Scenario: The modifier preserves strict-external behavior

- **WHEN** the same builder adds `.strict_external()`
- **THEN** fully-qualified declared external paths are classified through the existing strict-external rules and the projection carries `strict_external: true`

#### Scenario: The representation fold does not re-key a violation

- **WHEN** default and strict-external forms observe a path that both already classify as the same violation
- **THEN** their target, rule, finding key, polarity, and count are identical

### Requirement: Reference consumer surfaces remain available

The guibiao check, coverage, projection, baseline, and shared-model re-export functions and types SHALL
retain their existing names. pacta's Tianheng builder/runner use and modou's guibiao
projection/baseline integration SHALL compile against the narrowed local model without source
changes.

#### Scenario: pacta and modou compile against the local crates

- **WHEN** both reference consumers are checked with their Tianheng-family dependencies patched to the local change
- **THEN** their existing builder, runner, check, coverage, projection, baseline, and type-name usage compiles unchanged
