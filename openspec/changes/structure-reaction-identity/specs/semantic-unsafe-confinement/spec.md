## ADDED Requirements

### Requirement: Unsafe-site identity is structurally decomposed

Every unsafe-site fact SHALL encode its site form and enclosing module, plus each observed role
needed for that form: owner kind and canonical owner for methods, trait and self type for impls,
and item name where present. These roles SHALL be separate canonical fields rather than one rendered
finding string. Human wording and complete syntax SHALL remain presentation/diagnosis only.

An anonymous `unsafe {}` block SHALL retain deliberate per-module coalescing and SHALL NOT gain a
per-block ordinal. No unsafe fact SHALL use traversal order, item index, impl ordinal, or a
position-derived placeholder as public identity. When syntax cannot use its ordinary renderer, the
observer SHALL use an observed structural discriminator or fail loud rather than conflate distinct
sites.

#### Scenario: Unsafe impl roles stay distinct

- **WHEN** two unsafe impls differ by trait or self type in the same module
- **THEN** their structured facts differ in the corresponding trait or owner field

#### Scenario: Unsafe method owner roles stay distinct

- **WHEN** same-named unsafe methods belong to different inherent, trait, or trait-impl owners
- **THEN** owner kind, canonical owner, and trait role keep their identities distinct

#### Scenario: Anonymous blocks coalesce by observed module

- **WHEN** a module contains multiple anonymous unsafe blocks outside the allowed subtree
- **THEN** they deliberately produce one stable per-module fact without a block ordinal

#### Scenario: Reordering does not re-key an unsafe fact

- **WHEN** declarations or impl blocks are reordered or an unrelated item is inserted
- **THEN** every pre-existing unsafe-site identity remains unchanged

#### Scenario: Unrenderable sites do not collapse by position

- **WHEN** two distinct unsafe sites contain syntax outside the ordinary renderer
- **THEN** they remain structurally distinct or scanning fails loud, never sharing an ordinal fallback

## MODIFIED Requirements

### Requirement: Severity and baseline parity

An unsafe-confinement boundary SHALL carry a severity (`enforce` by default or `warn`) and SHALL gate
against the shared semantic Baseline. Its violation identity SHALL combine the confined crate
target, the stable unsafe-confinement rule key, and the structured unsafe-site fact. Human rule and
finding presentation SHALL remain available but SHALL NOT define matching. The optional baseline
owner/tracker annotations SHALL remain non-identity metadata.

#### Scenario: A warn boundary reports without failing

- **WHEN** a warn unsafe-confinement boundary is violated and no enforce boundary is violated
- **THEN** the reaction reports the violation and exits 0

#### Scenario: A baselined unsafe site does not fail

- **WHEN** an enforce boundary's only violations have exact target/rule/fact matches
- **THEN** the reaction reports them as accepted and exits 0 even if human wording changed

#### Scenario: A different unsafe site is not masked

- **WHEN** a current unsafe fact differs in any identity-bearing site role from an accepted entry
- **THEN** it is new and reacts according to severity
