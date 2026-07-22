## MODIFIED Requirements

### Requirement: The finding is an owner-qualified item identity

An async-exposure fact SHALL identify the governed public seam by module, owner kind, canonical
owner, and item name, with a trait-impl role when required to distinguish two seams on the same
owner. The complete parameter/return signature, generic spelling, and implicit future SHALL remain
human diagnostic presentation and SHALL NOT enter violation identity. A signature-only change to
the same named seam SHALL therefore preserve its baseline identity, while two different owners or
item names SHALL remain distinct.

No unrenderable owner SHALL fall back to a traversal ordinal, item index, or branch-local position.
If ordinary canonical rendering cannot distinguish two structurally distinct seams, the observation
SHALL use an observed structural discriminator or fail loud rather than collapse them.

#### Scenario: Two same-named async methods across owners stay distinct

- **WHEN** `impl A { pub async fn run(&self) {} }` and `impl B { pub async fn run(&self) {} }` are observed
- **THEN** two fact identities name their distinct canonical owners

#### Scenario: Trait and inherent seams stay distinct

- **WHEN** inherent and trait-impl async methods share the same owner and item name
- **THEN** owner kind and trait-impl role keep the public seams distinct

#### Scenario: A signature-only change preserves the seam

- **WHEN** parameters, return type, or generic spelling change while module, owner role, canonical owner, and item name remain the same
- **THEN** presentation changes but structured fact and baseline identity remain unchanged

#### Scenario: Pacta-shaped operations preserve identity across sync and async signatures

- **WHEN** a local fixture models Pacta's same registry operation with two signature shapes
- **THEN** Tianheng emits one stable seam identity while retaining each complete signature as diagnosis

#### Scenario: Cfg branches do not share a positional fallback

- **WHEN** distinct cfg branches contain structurally distinct but ordinarily unrenderable owners at the same local item position
- **THEN** their identities remain distinct or observation fails loud, never assigning equal ordinal-derived identities
