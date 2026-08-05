## MODIFIED Requirements

### Requirement: A cited pinning test SHALL resolve to exactly one definition in the tree

The reaction SHALL verify that each `PINNED-BY` name resolves to exactly one Rust function
**definition** under `crates/`, and that the resolved definition is a **test**. Resolving to none SHALL
fail: a test that was renamed or deleted leaves a citation that reads as coverage while defending
nothing, which is the silent pass the register opposes. Resolving to more than one SHALL also fail: a
name defined twice makes the citation name a set rather than a reaction, so the bound's defender is not
identified. Resolving to a function that is **not a test** SHALL fail for the same reason as an absent
one: a citation names what defends the bound, and a helper or production function of the right name
defends nothing while reading as coverage.

A definition SHALL be recognized as a test by an attribute run immediately above it containing `#[test]`,
read upward past interleaved attributes rather than only on the line before — `#[should_panic]` sits
between the attribute and the `fn` in three places in this tree, so a single-line read would refuse a
real test.

Requiring the cited function to be a test is not a naming convention imposed on a suite the register does
not own; it is what the citation already means. The register SHALL require nothing of the test's **name**,
which is what lets the bound-pinning tests keep at least three naming variants while some carry no
"bound" in the name at all.

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

#### Scenario: A citation resolving to a function that is not a test

- **WHEN** a declared bound's `PINNED-BY` resolves to exactly one function definition under `crates/` and
  that definition carries no `#[test]` in the attribute run above it
- **THEN** the reaction fails, naming the bound id and the definition site, because a function that never
  runs as a test defends nothing while occupying the place of the defence

#### Scenario: A pinning test whose attribute run carries another attribute

- **WHEN** a cited test's definition is preceded by `#[test]` and then a further attribute such as
  `#[should_panic]`
- **THEN** the reaction resolves it as a test, so the check reads the attribute run rather than one line

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

### Requirement: An unpinned bound SHALL be representable, and SHALL name its tracker

A bound with no pinning test SHALL be declarable as `UNPINNED` with a tracker reference. Requiring a
test for every bound would make the reaction block on exactly the gaps it exists to discover, whose
practical result is a smaller register rather than more tests — the trade `violation-baseline` already
settled by recording what is accepted and gating only new drift.

An `UNPINNED` citation SHALL name a tracker; a citation that merely asserts the absence of a test SHALL
fail, so accepted debt carries an owner rather than becoming anonymous. The reaction SHALL enforce this by
requiring the citation to name a **path the repository tracks**, which is the checkable part of naming an
owner: `no test exists` names none, and a tracker naming a file the repository does not track is
indistinguishable from an anonymous one, since the document it points at cannot be read.

Which section of that document owns the debt SHALL NOT be checked. That is prose the reaction cannot read,
and demanding it would trade a fact for a heuristic — the same trade the shared-bound direction refuses
below.

#### Scenario: A bound is declared without a pinning test

- **WHEN** a bound carries `UNPINNED` with a tracker reference naming a tracked path
- **THEN** the reaction passes for that bound and the projection counts it among the unpinned

#### Scenario: An unpinned citation names no tracker

- **WHEN** a bound carries `UNPINNED` with no tracker reference
- **THEN** the reaction fails, naming the bound id, because untracked debt is indistinguishable from an
  oversight

#### Scenario: An unpinned citation asserts the absence of a test instead of naming an owner

- **WHEN** a bound carries `UNPINNED` followed by text that names no path the repository tracks
- **THEN** the reaction fails, naming the bound id, because a sentence restating that no test exists
  records the gap without giving it an owner

#### Scenario: An unpinned citation naming a document that is not tracked

- **WHEN** a bound's `UNPINNED` tracker names a path absent from the repository's tracked files
- **THEN** the reaction fails, naming the bound id and the path, because a reference to a document that
  cannot be read is anonymous debt wearing an owner's name

### Requirement: The register reaction SHALL be a local gate CI runs identically

The reaction SHALL be a script invoked from the workspace root, listed in `AGENTS.md`'s Definition of
Done and run verbatim by CI, so `check_dod_coherence.sh` binds the two. Its failure directions SHALL
each be proven by a companion test against fixtures built to trip exactly one condition — a gate over a
coverage claim that has not been observed failing is a restatement of the register, not a defence of it.

The reaction SHALL be read-only: it SHALL NOT edit a spec, declare a bound, or rewrite the projection
except when explicitly asked to regenerate it.

Regeneration SHALL be bound by the same exit contract as judgment — 0 clean, 1 violation, 2 cannot judge.
Regenerating over a register that has offenses SHALL write the projection and then **fail**, because "the
document was rewritten" and "the register it describes is valid" are different claims and one exit code
cannot carry both. A register the reaction cannot judge at all SHALL fail **before** the projection is
written, so a register whose declarations it could not find cannot leave behind a document that reads as a
complete one.

#### Scenario: Every failure direction is proven

- **WHEN** the companion test runs
- **THEN** each of the reaction's failure directions is exercised by its own fixture, and the passing
  direction is exercised too, so a gate that only ever refuses is not mistaken for a working one

#### Scenario: The local gate and CI cannot drift apart

- **WHEN** the gate is added to the Definition of Done
- **THEN** the identical command appears in CI, and `check_dod_coherence.sh` fails if it does not

#### Scenario: The reaction leaves the tree unchanged

- **WHEN** the gate runs against any checkout
- **THEN** the working tree, `HEAD`, and the projection are unchanged unless regeneration was explicitly
  requested

#### Scenario: Regeneration over a register that has offenses

- **WHEN** regeneration is requested and a declared bound carries no citation
- **THEN** the projection is written and the reaction still fails, naming the offense, so a successful
  rewrite is never reported as a valid register

#### Scenario: Regeneration over a register the reaction cannot judge

- **WHEN** regeneration is requested and no declared bound is parsed at all
- **THEN** the reaction reports that it cannot judge and no projection is written, so a vacuous register
  produces no document

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
`#[path]`-remap bound went stale in two capabilities at once, and a sync left a contradicting bound beside
its own reacting scenario.

**The reaction over this requirement observes exactly one shape**: a single pinning test cited by declared
bounds in more than one capability. That is a fact rather than a heuristic, and it is a **floor**. Two
declarations of one behaviour that cite two different tests are invisible to it, and that residual SHALL be
stated in the projection's header beside the undeclared-prose floor, where a reader of the register sees it.
The requirement SHALL NOT claim that the reaction prevents every restatement, because a claim wider than
its reaction is the stale declaration this whole capability exists to end.

The residual SHALL NOT be declared as a bound of this capability, for the reason the prose floor is not:
telling two declarations of one behaviour apart from two behaviours over sibling shapes is a semantic
judgment, and nothing can observe it. Keying on statement similarity was rejected rather than overlooked —
two sibling operand dimensions in this repository declare identically-worded bounds over `dyn` and
`impl Trait` operands, each defended by its own test, and 三儀 ⊥ 三儀 requires each dimension to declare
its own; a similarity key would fail on that pair and demand the author dissolve a symmetry the
constitution requires.

The record of the two historical restatements SHALL say which direction reaches each shape, so the
requirement is not credited with a defence it does not provide: the `#[path]`-remap bound was stated as
prose in one capability and as a scenario in the other, so the **undeclared-prose** direction is what
reaches that shape.

#### Scenario: A shared bound is declared in its owner and referenced elsewhere

- **WHEN** one behaviour bounds three capabilities
- **THEN** exactly one declares it, the other two carry references to that declaration, and the projection
  lists the bound once

#### Scenario: The owner is the capability that claims the property on the others' behalf

- **WHEN** a capability's spec already states a shared property on behalf of its siblings
- **THEN** the bound is declared there rather than in a sibling, so the declaration sits with the claim

#### Scenario: The restatement direction states its own floor

- **WHEN** the projection is read
- **THEN** its header states that the restatement direction reaches a shared citation and not a shared
  behaviour, so a reader does not take the register for a proof that no bound is declared twice
