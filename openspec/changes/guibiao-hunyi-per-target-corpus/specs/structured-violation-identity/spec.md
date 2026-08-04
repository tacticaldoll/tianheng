## ADDED Requirements

### Requirement: The compilation unit is an identity-bearing observed value when a package has more than one

The compilation unit an observation came from SHALL be an identity-bearing observed value wherever a
dimension observes more than one of a package's units, so the same fact observed in two units yields two identities
and accepting one SHALL NOT suppress the other. A package may build more than one crate root — a library
beside a `bin`, several `[[bin]]` targets, or both — and each is its own compilation unit with its own
module graph.

Without it the two collapse, because every root of a package denotes the module path `crate` and shares
the package name: a violation accepted in one root would silently mask the same violation appearing later
in another — the baseline-masking false negative, arriving through the corpus rather than through a
renderer.

The role SHALL be **declaration-derived and stable**, never positional: not the order targets appear in
metadata, not an index. A target's **name** SHALL NOT be used alone, because it is not unique within a
package — a package may build a library and a `bin` of the same name. The role SHALL be the unit's root
source path relative to the package's manifest directory, which is unique per unit, moves with neither
the checkout nor the member set, and is the thing whose contents produced the observation. A root whose
path does not lie under that directory SHALL keep the path as given, the same rule the runtime dimension
applies to a file reached through an absolute path literal — stated, not silently degraded.

A dimension that observes exactly one compilation unit per package is unaffected and SHALL NOT add the
role, exactly as the declaring-crate requirement above does not obligate a boundary kind that already
varies by crate.

#### Scenario: The same violation in two roots of one package stays two identities

- **WHEN** a package builds both a library root and a `bin` root, the identical forbidden construct is
  written in each, and one boundary governs them
- **THEN** the two observations carry different identities, so a baseline accepting the one in the `bin`
  root does not suppress the one that later appears in the library root

#### Scenario: A target name alone does not distinguish a unit

- **WHEN** a package builds a library target and a `bin` target that share the package's own name
- **THEN** the identity role still distinguishes them, because it is derived from each unit's root source
  path rather than from the target name the two have in common
