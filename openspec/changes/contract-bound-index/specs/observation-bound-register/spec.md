## ADDED Requirements

### Requirement: An observation bound is declared once, in the spec of the capability it bounds

An **observation bound** SHALL be declared in the spec of the capability whose reaction it bounds, as a
`#### Bound: <id>` block placed inside the requirement it qualifies — a bound being a claim that an
observation deliberately stops at a named shape, so that shape is governed policy rather than a defect.
The declaring file SHALL be `openspec/specs/<capability>/spec.md`. `AGENTS.md` names that file the per-capability requirement truth, so the declaration SHALL NOT
introduce a separate register file, which would re-create the unlinked surfaces this register replaces.

The id SHALL be `<capability>/<slug>`, so a bound can be cited from a diagnostic, a `BACKLOG.md` entry, or
another spec without a path, and two capabilities cannot collide on a slug.

Each block SHALL carry a `statement` element giving the bound in one sentence, and SHALL carry exactly one
of a `pinned-by` element naming the test that pins the bound, or an `unpinned` element naming the tracker
that owns closing it. A block carrying both, or neither, SHALL fail the reaction: the two are exclusive
states of the same question, and a block that answers it twice or not at all records nothing.

#### Scenario: A bound is declared inside the requirement it qualifies

- **WHEN** a capability's spec states that its observation stops at a named shape
- **THEN** that statement appears as a `#### Bound: <capability>/<slug>` block inside the requirement it
  qualifies, carrying its `statement` and either its `pinned-by` test or its `unpinned` tracker

#### Scenario: A register block declares neither a pinning test nor a tracker

- **WHEN** a `#### Bound:` block carries a `statement` but neither `pinned-by` nor `unpinned`
- **THEN** the reaction fails, naming the block's id, because a bound with no recorded answer to "what
  defends this" is the unbacked claim the register exists to end

#### Scenario: A register block declares both a pinning test and a tracker

- **WHEN** a `#### Bound:` block carries both `pinned-by` and `unpinned`
- **THEN** the reaction fails, naming the block's id, because the bound is either defended or tracked and
  the block must say which

#### Scenario: The declaration does not disturb spec validation

- **WHEN** `openspec validate --specs --strict` runs over specs carrying `#### Bound:` blocks
- **THEN** every spec validates, so the register's syntax costs no schema change

### Requirement: A cited pinning test SHALL resolve to exactly one definition in the tree

The reaction SHALL verify that each `pinned-by` name resolves to exactly one Rust function **definition**
under `crates/`. Resolving to none SHALL fail: a test that was renamed or deleted leaves a register entry
that reads as coverage while defending nothing, which is the silent pass the whole register opposes.
Resolving to more than one SHALL also fail: a name defined twice makes the citation name a set rather than
a reaction, so the bound's defender is not identified.

Matching SHALL be on the definition form, never on a bare mention, so a citation cannot be satisfied by a
comment, a doc link, or a string that happens to contain the name.

#### Scenario: A citation naming a test that no longer exists

- **WHEN** a registered bound's `pinned-by` names a function defined nowhere under `crates/`
- **THEN** the reaction fails, naming the bound id and the unresolved test name

#### Scenario: A citation naming a test defined twice

- **WHEN** a registered bound's `pinned-by` name is defined by two functions under `crates/`
- **THEN** the reaction fails, naming the bound id and both definition sites, because the citation is
  ambiguous rather than merely imprecise

#### Scenario: A citation satisfied only by a mention

- **WHEN** a registered bound's `pinned-by` name appears in the tree only inside a comment or a string,
  with no function definition of that name
- **THEN** the reaction fails exactly as for an absent test, because a mention defends nothing

### Requirement: A bound stated in spec prose but left unregistered SHALL fail

The reaction SHALL scan `openspec/specs/*` for bound-declaring prose and SHALL fail on any occurrence
that does not sit inside a `#### Bound:` block. This makes the prose already present in the specs the
register's mandatory minimum, so the register cannot be completed by registering only the convenient
bounds.

This direction SHALL be described as a **floor and not a proof**, in the generated projection's own
header: a bound worded outside the scanned pattern is undetectable to it. The register SHALL therefore
carry that residual as one of its own registered bounds, rather than letting the projection imply that
every bound in the system is listed.

#### Scenario: Spec prose declares a bound that no block registers

- **WHEN** a spec paragraph states that an observation stops at a shape, and no `#### Bound:` block
  contains that paragraph
- **THEN** the reaction fails, naming the file and the occurrence

#### Scenario: The same prose inside a register block does not fail

- **WHEN** the bound-declaring prose sits inside a `#### Bound:` block
- **THEN** the reaction passes for that occurrence, so registering a bound is what clears it rather than
  rewording it

#### Scenario: The register's own detection residual is itself registered

- **WHEN** the projection is read
- **THEN** its header states that the unregistered-prose direction is a floor over recognizable wording,
  and a registered bound of the register capability carries that same residual with its own pinning test

### Requirement: The register SHALL be projected as a generated, staleness-checked document

The register SHALL be projected into a generated document at `docs/observation-bounds.md`, grouped by
capability, carrying each bound's id, statement, and either its pinning test or its tracker. The document
SHALL be derived from the specs and never hand-maintained, and a stale projection SHALL fail the
reaction — the discipline `AGENTS.self-law.md` already follows, for the same reason: a hand-maintained
structural document drifts from what it describes.

The projection SHALL surface the **count of unpinned bounds as its headline figure**, because that count
is the register's audit backlog and a figure in a footnote is not read.

#### Scenario: The projection is stale

- **WHEN** a register block is added, changed, or removed without regenerating the projection
- **THEN** the reaction fails, reporting that the projection no longer matches the specs

#### Scenario: The projection is regenerated

- **WHEN** the projection is regenerated from the specs
- **THEN** it matches byte-for-byte and the reaction passes, so the document has one source of truth

#### Scenario: Unpinned bounds are counted where a reader cannot miss them

- **WHEN** the register contains bounds whose defence is a tracker rather than a test
- **THEN** the projection states their count in its header, not only in the affected entries

### Requirement: An unpinned bound SHALL be representable, and SHALL name its tracker

A bound with no pinning test SHALL be declarable as `unpinned` with a tracker reference. Requiring a test
for every entry would make the reaction block on exactly the gaps it exists to discover, whose practical
result is a smaller register rather than more tests — the trade `violation-baseline` already settled by
recording what is accepted and gating only new drift.

An `unpinned` element SHALL name a tracker; an entry that merely asserts the absence of a test SHALL
fail, so accepted debt carries an owner rather than becoming anonymous.

#### Scenario: A bound is registered without a pinning test

- **WHEN** a bound is declared `unpinned` with a tracker reference
- **THEN** the reaction passes for that entry and the projection counts it among the unpinned

#### Scenario: An unpinned entry names no tracker

- **WHEN** a bound is declared `unpinned` with no tracker reference
- **THEN** the reaction fails, naming the bound id, because untracked debt is indistinguishable from an
  oversight

### Requirement: The register reaction SHALL be a local gate CI runs identically

The reaction SHALL be a script invoked from the workspace root, listed in `AGENTS.md`'s Definition of
Done and run verbatim by CI, so `check_dod_coherence.sh` binds the two. Its failure directions SHALL each
be proven by a companion test against fixtures built to trip exactly one condition — a gate over a
coverage claim that has not been observed failing is a restatement of the register, not a defence of it.

The reaction SHALL be read-only: it SHALL NOT edit a spec, register a bound, or rewrite the projection
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
