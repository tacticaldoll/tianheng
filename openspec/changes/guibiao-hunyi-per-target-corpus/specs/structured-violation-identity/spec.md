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
source path relative to the package's own directory, which is unique per unit, moves with neither the
checkout nor the member set, and is the thing whose contents produced the observation.

A root whose path does not lie under that directory SHALL be a **constitution error** naming it, never
labeled by the path as given. That path is the checkout's own location, so using it would make the
identity checkout-dependent — the same commit in two clones yielding two identities, and a baseline
recorded in one matching nothing in the other, which is the defect this role exists to prevent. Refusing
to judge is the Core Contract's own ordering over a silently degraded label, and it matches the runtime
dimension's refusal of a relative or empty anchor.

This is deliberately NOT the rule the runtime dimension applies to a file reached through an absolute
path literal, and the difference SHALL be stated wherever either is: that literal is **committed text**,
identical in every checkout, so keeping it verbatim is exactly what makes it stable, whereas a root path
outside the package directory is the checkout's location, so keeping it verbatim is what makes it
unstable. Same shape, opposite consequence.

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

### Requirement: A fact carries every varying coordinate of its observation's location

A fact's identity SHALL carry every coordinate of **where** the observation was made that can vary
within the governed space, and SHALL NOT carry one that cannot. The coordinates are, from outermost in:
the declaration that governs it, the compilation unit, the module, the owner or item, and the
position-free discriminator of the thing itself within that item. A coordinate SHALL be omitted only
when it cannot vary for that fact family or is already encoded in the violation's target, and the
omission SHALL be recorded with the reason rather than left as silence.

No coordinate SHALL be positional — not scan order, item ordinal, traversal index, or renderer fallback
position — and none SHALL be checkout-dependent, since either makes an identity that shifts without the
observation changing.

This derivation exists because the alternative was found not to work: every identity collision this
system has had was a missing coordinate discovered one adversarial review at a time — a second crate
declaring the same boundary, a second module implementing the same owner, a second impl block bounding
the same parameter, a second spelling of one trait, a second path differing only in undecodable bytes, a
second crate root, and a second module importing the same forbidden path. Widening a fact's schema
pre-emptively SHALL NOT be used to anticipate them: an identity is its **values**, so declaring a field
before its value is known re-keys every recorded baseline once for the field and again when the value
arrives, and a coordinate that cannot vary adds nothing to distinguish.

Each dimension's own published-identity-schema reaction SHALL be the enforcement point: a fact family
whose schema omits a coordinate that can vary SHALL fail that reaction rather than await review.

#### Scenario: A new fact family omitting a varying coordinate fails its schema reaction

- **WHEN** a fact family is added or changed so that its identity omits a coordinate of the observation's
  location that can vary for it
- **THEN** the dimension's published-identity-schema reaction fails, rather than the omission surviving
  until two observations are found to collide

#### Scenario: A coordinate that cannot vary is not added

- **WHEN** a coordinate is already encoded in the violation's target, or cannot vary for a fact family at
  all
- **THEN** the fact does not carry it, and the reason is recorded — so the identity stays as narrow as the
  observation and no baseline re-keys for a field that distinguishes nothing
