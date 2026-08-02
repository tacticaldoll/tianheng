## MODIFIED Requirements

### Requirement: Trait-impl exposure uses observed structural seams

Trait-impl exposure facts SHALL encode trait, canonical self type, associated item role/name, the
where-clause bounded type, and forbidden subject where observed. A traversal position or impl/item
ordinal SHALL NOT substitute for an unrenderable structural role: when a role's ordinary rendering
fails (for example, a where-clause bounded type carrying a complex const-generic argument, such as
`Arr<{ N + 1 }>`, which no observed shape in this capability renders), the system SHALL fail loud
(a constitution error identifying the failure) rather than fall back to a shared literal that two
structurally distinct roles could both produce.

#### Scenario: Inherent and trait-impl seams stay distinct
- **WHEN** the same subject appears in an inherent item and a trait-impl item on one self type
- **THEN** their owner/trait/item roles keep the identities distinct

#### Scenario: An unrenderable seam fails safely
- **WHEN** ordinary rendering cannot distinguish two structural seams
- **THEN** an observed discriminator separates them or scanning fails loud, never a positional fallback

#### Scenario: Distinct unrenderable where-clause bounds do not collapse by position

- **WHEN** one impl block declares two where-clause bounds each naming a structurally distinct but
  ordinarily unrenderable bounded type (for example `Arr<{ N + 1 }>: AsRef<crate::infra::Secret>`
  and `Arr<{ N + 2 }>: AsRef<crate::infra::Secret>`), each independently exposing the same forbidden
  type
- **THEN** the system does not emit one shared fact for both bounds under a common literal
  placeholder; scanning fails loud (a constitution error) rather than silently reporting only one of
  the two bounds' violations, and a renderable where-clause bounded type is unaffected by this
  fail-loud path

### Requirement: Impl-site-authored positions govern trait-impl exposure

With `.including_trait_impls()` enabled, the system SHALL observe, for each trait `impl` block whose
text appears in the governed module's source, the **impl-site-authored** positions and react to a
forbidden type that appears in any of them. The observed positions SHALL comprise exactly:

1. the trait path's generic arguments (position `trait-arg`);
2. the `Self` type, both when a forbidden type **is** the Self type and when it is **nested** within it, including the Self type's generic arguments (position `self`);
3. associated type/value bindings authored in the impl, `type Assoc = …` (position `assoc {name}`);
4. the impl block's own generic bounds and `where`-clause, keyed by the bounded type (position `where {bounded-type}`);
5. the impl method **return type as written at the impl site** (position `method {name} return`).

A forbidden type reached only through an impl-site position SHALL react even when it appears in no
signature-coupling position. A where-clause bounded type that cannot be rendered (a complex
const-generic argument) SHALL NOT be silently keyed to a shared placeholder; see "Trait-impl exposure
uses observed structural seams" for the fail-loud requirement this failure mode falls under.

#### Scenario: A forbidden type in a trait's generic argument is a violation

- **WHEN** the governed module declares `impl From<crate::infra::DbPool> for Service` and the boundary forbids exposing `crate::infra` with `.including_trait_impls()`
- **THEN** the system emits a violation naming `crate::infra::DbPool`, exposed at the `trait-arg` position

#### Scenario: A forbidden type that is the Self type is a violation

- **WHEN** the governed module declares `impl SomeTrait for crate::infra::Forbidden {}` and the boundary forbids exposing `crate::infra` with `.including_trait_impls()`
- **THEN** the system emits a violation naming `crate::infra::Forbidden`, exposed at the `self` position — the Self type is the impl seam's subject, the same coupling signature-coupling already treats as exposure for a `pub fn` parameter

#### Scenario: A forbidden type nested in the Self type is a violation

- **WHEN** the governed module declares `impl SomeTrait for Vec<crate::infra::DbPool>` and the boundary forbids exposing `crate::infra` with `.including_trait_impls()`
- **THEN** the system emits a violation naming `crate::infra::DbPool`, exposed at the `self` position

#### Scenario: A forbidden type in an associated-type binding is a violation

- **WHEN** the governed module declares `impl Iterator for Service { type Item = crate::infra::Secret; … }` and the boundary forbids exposing `crate::infra` with `.including_trait_impls()`
- **THEN** the system emits a violation naming `crate::infra::Secret`, exposed at the `assoc Item` position

#### Scenario: A forbidden trait in the impl where-clause is a violation, keyed by the bounded type

- **WHEN** the governed module declares `impl<T> SomeTrait for Service<T> where T: crate::infra::Secret {}` and the boundary forbids exposing `crate::infra` with `.including_trait_impls()`
- **THEN** the system emits a violation naming `crate::infra::Secret`, exposed at the `where T` position (impl generic bounds and `where`-clause bounds share the `where` position, keyed by the bounded type so two distinct bounds never collapse)

#### Scenario: An impl-refined method return type (RPITIT) is a violation

- **WHEN** a trait declares `fn items(&self) -> impl Iterator<Item = u8>;` and the governed module declares `impl Port for Service { fn items(&self) -> crate::infra::Iter { … } }`, refining the opaque return to a concrete type, under a boundary forbidding `crate::infra` with `.including_trait_impls()`
- **THEN** the system emits a violation naming `crate::infra::Iter`, exposed at the `method items return` position — the concrete return is authored at the impl site and is public API, so leaving it unobserved would be a false negative
