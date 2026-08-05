## ADDED Requirements

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
fail, so accepted debt carries an owner rather than becoming anonymous.

#### Scenario: A bound is declared without a pinning test

- **WHEN** a bound carries `UNPINNED` with a tracker reference
- **THEN** the reaction passes for that bound and the projection counts it among the unpinned

#### Scenario: An unpinned citation names no tracker

- **WHEN** a bound carries `UNPINNED` with no tracker reference
- **THEN** the reaction fails, naming the bound id, because untracked debt is indistinguishable from an
  oversight

### Requirement: The register reaction SHALL be a local gate CI runs identically

The reaction SHALL be a script invoked from the workspace root, listed in `AGENTS.md`'s Definition of
Done and run verbatim by CI, so `check_dod_coherence.sh` binds the two. Its failure directions SHALL
each be proven by a companion test against fixtures built to trip exactly one condition — a gate over a
coverage claim that has not been observed failing is a restatement of the register, not a defence of it.

The reaction SHALL be read-only: it SHALL NOT edit a spec, declare a bound, or rewrite the projection
except when explicitly asked to regenerate it.

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
