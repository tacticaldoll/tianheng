# observation-bound-register Specification

## Purpose

Put every **observation bound** this family declares onto one enumerated, defended surface. A bound is
the claim that a reaction deliberately stops at a named shape — the one claim class no reaction defends,
and the reason a stale one is worse than ordinary stale prose: it reads as **permission**, telling a
future auditor that a real escape is governed policy. Two bounds here outlived the behaviour they
described for two releases, one of them in two capabilities at once, and nothing was watching.

So the register makes each bound name what defends it — a pinning test, or a tracker that owns closing
the gap — refuses a citation that resolves to nothing or to something that never runs, and refuses a bound
stated in prose that no scenario declares. It is projected into a generated, staleness-checked document
whose headline is the count of bounds with no test, because that count is the audit backlog and a figure
in a footnote is not read.

Where the register cannot see, it says so in that same document rather than implying completeness: the
prose scan is a floor over recognizable wording, and the restatement direction reaches a shared citation
rather than a shared behaviour. A register that overclaimed would mislead exactly where it is most
trusted — and would be committing the failure it exists to end.
## Requirements
### Requirement: An observation bound is declared as a scenario that names itself one

An **observation bound** SHALL be declared as a `#### Scenario:` whose heading marks it as a bound, in
the spec of the capability whose reaction it bounds — a bound being a claim that an observation
deliberately stops at a named shape, so that shape is governed policy rather than a defect. The
declaring file SHALL be `openspec/specs/<capability>/spec.md`.

The declaration SHALL sit under the requirement it qualifies, wherever that is, and SHALL NOT be hoisted
into a common section. 21 of the 24 bounds declared today sit under the requirement they qualify rather
than under an `Observation bounds` requirement, and moving them would separate each bound from the
reaction it limits — the `Observation bounds` requirement three specs carry is a place bounds are
gathered, never the definition of one.

Requiring the heading convention is legitimate where requiring a test-name convention is not, and the
difference is ownership: a scenario heading is authored in the spec, so the register may require its
form, while a test name pre-exists the register and is owned by its suite. A bound whose heading omits
the marking SHALL be caught by the undeclared-prose reaction below rather than silently missed.

A parallel block form SHALL NOT be introduced, because for a bound already declared as a scenario it
would state the same bound twice, which is the drift the register exists to end.

A bound's identity SHALL be derived from its location as `<capability>/<scenario-slug>`, never allocated,
so no identifier ledger is introduced and a citation cannot outlive the declaration it names.

#### Scenario: A bound is declared beside the requirement it qualifies

- **WHEN** a capability states that its observation stops at a named shape
- **THEN** that statement appears as a bound-marked scenario under the requirement it qualifies, carrying
  its own WHEN/THEN, and no second declaration of the same bound exists elsewhere in the spec

#### Scenario: A bound-marked scenario is recognized wherever it sits

- **WHEN** a bound-marked scenario sits under a requirement that is not named `Observation bounds`
- **THEN** the reaction reads it as a declared bound and requires its citation, so the register covers
  the 21 bounds already declared that way without relocating any of them

#### Scenario: A bound's id is derived rather than assigned

- **WHEN** a declared bound is cited from a diagnostic, another spec, or `BACKLOG.md`
- **THEN** the citation is `<capability>/<scenario-slug>`, requiring no lookup table and no allocation step

#### Scenario: The declaration does not disturb spec validation

- **WHEN** `openspec validate --specs --strict` runs over the specs carrying declared bounds and their
  citation bullets
- **THEN** every spec validates, so the register's syntax costs no schema change

### Requirement: A declared bound SHALL carry exactly one citation naming its defence

A declared bound SHALL carry exactly one citation bullet beside its WHEN/THEN: either
`- **PINNED-BY** \`<test fn name>\`` naming the test that pins it, or `- **UNPINNED** <tracker>` naming
what owns closing the gap. Carrying both SHALL fail, and carrying neither SHALL fail: the two are
exclusive answers to one question, and a bound that answers it twice or not at all records nothing.

#### Scenario: A declared bound carries no citation

- **WHEN** a bound scenario carries only its WHEN/THEN
- **THEN** the reaction fails, naming the bound id, because a bound with no recorded answer to "what
  defends this" is the unbacked claim the register exists to end

#### Scenario: A declared bound carries both citation forms

- **WHEN** a bound scenario carries both `PINNED-BY` and `UNPINNED`
- **THEN** the reaction fails, naming the bound id, because the bound is either defended or tracked and
  the declaration must say which

### Requirement: A cited pinning test SHALL resolve to exactly one definition in the tree

A citation's syntax SHALL be validated before it is resolved. The cited name SHALL be a Rust identifier, an
optional crate qualifier SHALL be a crate-directory name, and at most one `::` separator SHALL appear;
anything else SHALL fail, naming the bound id and the rejected citation. This closes two directions **by
construction** rather than by escaping. The name is interpolated into the search pattern, so a regular-expression
metacharacter would let a citation for a test that does not exist resolve to a differently-named function —
defeating the renamed-or-deleted direction this requirement exists for. The qualifier is joined to a
filesystem path, so `../` would resolve a citation against a function outside the `crates/` boundary this
requirement declares.

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
real test. The walk SHALL run to the enclosing item's boundary and SHALL NOT be capped at a fixed number of
lines: no attribute-run length is declared anywhere, so a cap refuses a legitimate test whose run is longer
than the cap happened to be.

The walk SHALL stop at a block-comment delimiter rather than interpret one, so a `#[test]` written inside a
block comment does not satisfy the run. It SHALL NOT strip or track comments: comment state is a forward
property of a file that an upward walk cannot know, and stripping requires lexing string literals — this
tree's own lexer fixtures carry 49 `/*` occurrences **inside string literals**, several of them nested, so a
delimiter-counting stripper would manufacture phantom comments and swallow real definitions. Stopping at the
delimiter refuses the shape without either, and its error direction is loud: a test whose attribute run
genuinely contains a block comment is refused rather than quietly accepted.

Requiring the cited function to be a test is not a naming convention imposed on a suite the register does
not own; it is what the citation already means. The register SHALL require nothing of the test's **name**
beyond its being an identifier, which is what lets the bound-pinning tests keep at least three naming
variants while some carry no "bound" in the name at all.

Matching SHALL be on the definition form, never on a bare mention: a citation SHALL NOT be satisfied by a
name appearing in a line comment, a doc link, or a string. **That claim is a floor over the mention forms
the definition pattern excludes, and SHALL be stated as one.** The pattern reads a line's shape, not its
comment state, so a function definition that is itself inside a block comment satisfies a citation. Closing
that needs the same string-literal lexing measured above as out of reach for a text-scanning gate, so the
residual SHALL be stated here and in the projection's header, and SHALL be pinned by a fixture recording the
accepted behaviour, so a later repair is not silently absorbed.

That residual SHALL NOT be declared as a bound of this capability, and the reason is a limit of the citation
form rather than a judgment: `PINNED-BY` names a Rust test under `crates/`, while this reaction's own
defences are shell fixtures, so the declaration would have to be `UNPINNED` against a tracker owning
something already measured as out of reach — permanent debt wearing an owner's name, which the unpinned
requirement forbids. That the register cannot pin a bound of its own capability SHALL be recorded as an
observation in `BACKLOG.md` rather than worked around here.

#### Scenario: A citation whose name is not an identifier

- **WHEN** a declared bound's `PINNED-BY` contains a character no Rust identifier may hold
- **THEN** the reaction fails before resolving it, naming the bound id and the rejected citation, so a
  metacharacter cannot resolve a citation to a differently-named function

#### Scenario: A citation whose crate qualifier leaves the crates directory

- **WHEN** a declared bound's `PINNED-BY` qualifier is not a plain crate-directory name — a traversal, a
  nested path, or a second `::` component
- **THEN** the reaction fails before resolving it, so a citation cannot be satisfied by a function outside
  the boundary this requirement declares

#### Scenario: A citation naming a test that no longer exists

- **WHEN** a declared bound's `PINNED-BY` names a function defined nowhere under `crates/`
- **THEN** the reaction fails, naming the bound id and the unresolved test name

#### Scenario: A citation naming a test defined twice

- **WHEN** a declared bound's `PINNED-BY` name is defined by two functions under `crates/`
- **THEN** the reaction fails, naming the bound id and both definition sites, because the citation is
  ambiguous rather than merely imprecise

#### Scenario: A citation satisfied only by a mention

- **WHEN** a declared bound's `PINNED-BY` name appears in the tree only inside a line comment or a string,
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

#### Scenario: A pinning test whose attribute run is longer than any cap

- **WHEN** a cited test carries `#[test]` above more interleaved attributes than a fixed-window walk would
  read
- **THEN** the reaction still resolves it as a test, because the walk ends at the item boundary rather than
  at a line count

#### Scenario: An attribute written inside a block comment

- **WHEN** a cited function's `#[test]` sits inside a block comment above the definition
- **THEN** the reaction fails as for any non-test definition, because the walk stops at the delimiter rather
  than reading commented text as an attribute

#### Scenario: A definition inside a block comment is not distinguished from a real one

- **WHEN** a cited function's whole definition sits inside a block comment
- **THEN** the reaction resolves it, which is the stated residual of matching on a line's form: a fixture
  records this so a later repair is not absorbed silently, and the projection's header states it where a
  register reader sees it

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

### Requirement: A bound stated in prose but not declared as a scenario SHALL fail

The reaction SHALL scan `openspec/specs/*` for bound-declaring prose and SHALL fail on any occurrence
outside a declared bound scenario. This makes the prose already present the register's mandatory
minimum, so the register cannot be completed by declaring only the convenient bounds. Its size is
measured rather than estimated: 3 of 29 specs carry an Observation-bounds requirement today while 11
more state bound prose without one.

This direction SHALL be described as a **floor and not a proof**, in the generated projection's own
header: a bound worded outside the scanned pattern is undetectable to it. That residual SHALL be stated
there and SHALL NOT be declared as a bound of this capability, because nothing can observe it — a
declaration no reaction can reach is the name-without-a-reaction `PROJECT.md` forbids, and the register
must not make itself the first exception.

#### Scenario: Spec prose states a bound that no scenario declares

- **WHEN** a spec paragraph or a bare THEN clause states that an observation stops at a shape, and no
  bound scenario declares it
- **THEN** the reaction fails, naming the file and the occurrence

#### Scenario: The same statement inside a declared bound scenario does not fail

- **WHEN** the bound-declaring prose sits inside a declared bound scenario
- **THEN** the reaction passes for that occurrence, so declaring the bound is what clears it rather than
  rewording the sentence

#### Scenario: The register states its own detection residual without declaring it a bound

- **WHEN** the projection is read
- **THEN** its header states that the undeclared-prose direction is a floor over recognizable wording, and
  no bound of this capability claims that residual, since no reaction could reach one

### Requirement: Prose MAY reference a declared bound, and a reference SHALL resolve

Prose that mentions a bound SHALL be cleared by the undeclared-prose reaction when it carries an explicit
reference of the form `(bound: <capability>/<slug>)`, where `<slug>` is the declaring scenario's heading
lowercased with each run of non-alphanumeric characters replaced by a single hyphen. The reference SHALL
resolve to **exactly one** declared bound across all specs: resolving to none SHALL fail, because a
reference that points nowhere is indistinguishable from an undeclared bound, and resolving to more than one
SHALL fail, which is also what keeps derived ids unique rather than merely assumed unique.

A reference exists because the floor's alternative is worse. Without it, a sentence that legitimately
**points at** a bound declared elsewhere — in the same file, or in another dimension's spec — must either
be rewritten to avoid the words or be restated as a second declaration of the same bound. The first
degrades prose that is doing its job; the second is exactly the restatement this register exists to end,
and the drift it produces is already recorded as a live `BACKLOG.md` item.

A reference SHALL NOT be treated as a declaration: it carries no citation of its own, contributes nothing
to the register's bound count, and cannot be the only mention of a bound anywhere.

#### Scenario: Prose referencing a declared bound is cleared

- **WHEN** a sentence mentions a bound and carries `(bound: <capability>/<slug>)` naming a declared bound
- **THEN** the reaction passes for that occurrence, and the register's bound count is unchanged

#### Scenario: A reference that resolves to nothing

- **WHEN** a reference names a `<capability>/<slug>` that no declared bound produces
- **THEN** the reaction fails, naming the file, the line, and the unresolved id, because a dangling
  reference is indistinguishable from an undeclared bound

#### Scenario: A reference that resolves to two declared bounds

- **WHEN** two declared bounds in one capability produce the same slug and a reference names it
- **THEN** the reaction fails, naming both declarations, so a derived id's uniqueness is checked rather
  than assumed

#### Scenario: A reference is not a declaration

- **WHEN** a bound is mentioned only by references and declared nowhere
- **THEN** every reference fails to resolve, so the bound cannot exist in the register as a reference alone

### Requirement: The register SHALL be projected as a generated, staleness-checked document

The register SHALL be projected into a generated document at `docs/observation-bounds.md`, grouped by
capability, carrying each bound's id, its statement, and either its pinning test or its tracker. The
document SHALL be derived from the specs and never hand-maintained, and a stale projection SHALL fail
the reaction — the discipline `AGENTS.self-law.md` already follows, for the same reason: a
hand-maintained structural document drifts from what it describes.

The projection SHALL surface the **count of unpinned bounds as its headline figure**, because that count
is the register's audit backlog and a figure in a footnote is not read.

#### Scenario: The projection is stale

- **WHEN** a bound is declared, changed, or removed without regenerating the projection
- **THEN** the reaction fails, reporting that the projection no longer matches the specs

#### Scenario: The projection is regenerated

- **WHEN** the projection is regenerated from the specs
- **THEN** it matches byte-for-byte and the reaction passes, so the document has one source of truth

#### Scenario: Unpinned bounds are counted where a reader cannot miss them

- **WHEN** the register contains bounds whose defence is a tracker rather than a test
- **THEN** the projection states their count in its header, not only in the affected entries

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
