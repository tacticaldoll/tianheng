## MODIFIED Requirements

### Requirement: A cited pinning test SHALL resolve to exactly one definition in the tree

The reaction SHALL verify that each `PINNED-BY` name resolves to exactly one Rust function
**definition** under `crates/`. Resolving to none SHALL fail: a test that was renamed or deleted leaves
a citation that reads as coverage while defending nothing, which is the silent pass the register
opposes. Resolving to more than one SHALL also fail: a name defined twice makes the citation name a set
rather than a reaction, so the bound's defender is not identified.

Matching SHALL be on the definition form, never on a bare mention, so a citation cannot be satisfied by
a comment, a doc link, or a string that happens to contain the name.

#### Scenario: A citation naming a test that no longer exists

- **WHEN** a declared bound's `PINNED-BY` names a function defined nowhere under `crates/`
- **THEN** the reaction fails, naming the bound id and the unresolved test name

#### Scenario: A citation naming a test defined twice

- **WHEN** a declared bound's `PINNED-BY` name is defined by two functions under `crates/`
- **THEN** the reaction fails, naming the bound id and both definition sites, because the citation is
  ambiguous rather than merely imprecise

#### Scenario: A citation satisfied only by a mention

- **WHEN** a declared bound's `PINNED-BY` name appears in the tree only inside a comment or a string,
  with no function definition of that name
- **THEN** the reaction fails exactly as for an absent test, because a mention defends nothing

#### Scenario: One test cited by bounds in two capabilities

- **WHEN** declared bounds in two different capabilities cite the same `PINNED-BY` test
- **THEN** the reaction fails, naming every declaring capability and the shared test, because one behaviour
  has one defence and therefore one declaration; the others reference it

#### Scenario: A bound citing two tests is not a restatement

- **WHEN** one declared bound whose heading covers two shapes cites two tests
- **THEN** the reaction passes, since a bound covering two shapes is defended by two tests

#### Scenario: One capability citing one test from two bounds is not a restatement

- **WHEN** two declared bounds within a single capability cite the same test
- **THEN** the reaction passes: the restatement this direction exists for is one defence claimed by two
  capabilities, never repetition inside one

## ADDED Requirements

### Requirement: A bound shared by several capabilities SHALL be declared once and referenced elsewhere

A behaviour that bounds more than one capability SHALL be declared as a bound in exactly one of them, and
the others SHALL carry a `(bound: …)` reference to that declaration rather than a parallel declaration of
their own. The owning capability SHALL be the one that already claims the property on the others' behalf
where such a claim exists; where none does, the reaction SHALL name the capabilities and leave the choice to
the author, ownership being a judgment a reaction can demand but not compute.

**This supersedes the register's original rule that a shared bound is declared once per capability**, and
the reason for that rule is recorded so the reversal is not mistaken for drift: declaring once was rejected
because it would leave the other capabilities' specs silent about a bound they have. The reference form,
which did not exist when that was settled, keeps the bound visible in every capability that has it while
leaving one declaration to maintain — so the property the old rule protected is no longer bought at the
price of restatement.

Restatement is the failure this prevents, and it has already cost this repository twice: the
`#[path]`-remap bound was stale in two capabilities at once, and a sync left a contradicting bound beside its
own reacting scenario. One behaviour change SHALL NOT be able to leave several specs stale.

#### Scenario: A shared bound is declared in its owner and referenced elsewhere

- **WHEN** one behaviour bounds three capabilities
- **THEN** exactly one declares it, the other two carry references to that declaration, and the projection
  lists the bound once

#### Scenario: The owner is the capability that claims the property on the others' behalf

- **WHEN** a capability's spec already states a shared property on behalf of its siblings
- **THEN** the bound is declared there rather than in a sibling, so the declaration sits with the claim
