# observation-bound-model Specification

## Purpose

Type where a reaction's measure deliberately stops, so a declared observation bound carries a classification a
reaction can check rather than an adjective a reader must interpret — and hold the specs' declarations and the
code's in a bijection, in both directions.
## Requirements
### Requirement: A declared bound SHALL carry a typed extent whose illegal states are unrepresentable

Every declared observation bound SHALL carry an `Extent`: where the reaction's measure stops. The type SHALL
be nested rather than flat, so a shape the observation source never reached has **nowhere** to carry a claim
about how the reaction treated it. This is the difference between a model and a label — a flat enum admits
"never observed it, and it over-reacts", which is the exact contradiction that let a backlog entry predict a
false negative where the real behaviour was a fail-loud constitution error.

The value set SHALL be the one the declared bounds exhibit, not a designed hierarchy. Seven stops were read out
of the family's own declarations:

1. **out of reach** — the observation source never sees the shape (`external-crate-confinement`: comments,
   string literals and macro bodies are stripped before scanning; `runtime-origin-assertion`: source outside a
   member's library or binary targets; `semantic-dyn-trait-boundary`: a macro-generated `dyn`).
2. **reached, refusing to judge** — exit 2 rather than a guess.
3. **reached, deliberately not refusing** — `semantic-trait-impl-locality`: a cfg-gated module with an absent
   file "is skipped … rather than failing the gate with a scan error (exit 2)". The mirror of the above, and a
   real declaration, so the model carries it rather than collapsing the two.
4. **reached, over-reacting** — `crate-source-boundary`: a `git`-plus-`version` dependency is flagged "even
   though such a dependency would `cargo publish` successfully".
5. **reached, under-reacting** — a declared false negative. `inline-symbol-path-confinement`: "the system does
   NOT react (a false negative the adopter owns by narrowing)".
6. **reached, correctly not a violation** — `semantic-reexport-exposure`'s `as _`, which "binds no nameable path
   a consumer can reach", and the body-nested module and plain item, "unreachable as `crate::…`". Nothing is
   bounded at all: the reaction is right, and the declaration exists only so a reader does not misread the
   silence as an escape. This value was **absent from the first draft** and was forced by classifying the
   declared set — three bounds are exactly it, and the granularity value below could not hold them because it
   requires a bounded part.
7. **reached, reacting exactly, bounded in what the fact carries** — `semantic-dyn-trait-boundary`'s
   unrenderable sub-node: "each still *reacts* on first occurrence; only baseline-dedup granularity is
   bounded". The reaction is not bounded at all here, so an extent that implied otherwise would misreport it.

Value 2 SHALL be retained whether or not a declared bound uses it, and its membership SHALL NOT be restated in
prose. Which values carry bounds is what the extent projection renders; a sentence naming that here is a census
of a set with an enumerator, and every attempt at one has gone stale within the window that wrote it — once when
an instance arrived, once when the reaction holding the only instance was retired. **Nothing observes that
prohibition**, and it is a rule rather than a reaction deliberately: the detector it would need is one over
prose, which `AGENTS.md` records as designed and measured three times and rejected — widening the recognized
phrasing false-positives on the generated projections' own headers and on a gate's diagnostic strings. Saying
which of the two this sentence is, is what keeps it from reading as the other. The misclassification this model exists to prevent was exactly a confusion between it and
value 1: a backlog entry predicted a silent false negative for a `#[cfg_attr]` path remap where the real
behaviour was a fail-loud refusal, and the entry's own lesson is that the risk class decides urgency. A
direction that cannot be *named* cannot be predicted with, which is what earns the value its place — not how
many bounds happen to hold it.

Granularity SHALL be carried **only** by the sixth, not as an independent field on every extent. No declared
bound is both out of reach and granularity-limited, so a model offering both on every value would invite a
combination nothing exhibits.

#### Scenario: A never-reached shape cannot claim a reaction direction

- **WHEN** a declaration says the observation source never sees the shape
- **THEN** the type offers no place to record over- or under-reaction, so the contradiction is a compile
  error rather than a review finding

#### Scenario: Refusing to judge and declining to refuse are distinct

- **WHEN** one bound declares that the reaction exits 2 on a shape and another declares that it deliberately
  continues past a shape that could have errored
- **THEN** they carry different values, because collapsing them would make a declared fail-loud and a declared
  skip read alike while their adopter consequences are opposite

#### Scenario: A reaction that is exactly right is not described as bounded

- **WHEN** a bound limits only the granularity at which two occurrences are told apart, while the reaction
  fires correctly on each
- **THEN** the extent records that the reaction is as intended and names the bounded part of the fact, so the
  projection does not report a working reaction as a limited one

### Requirement: A declared false negative SHALL name who owns closing it

An under-reacting extent SHALL carry an owner: this dimension's own engine, a layer beneath it (naming which),
or the adopter. A false negative is the one direction this family treats as a defect, so a declaration of one
with nobody responsible is how it outlives its reason. The declared bounds already distinguish all three —
"inherited from the module scanner", "shared with the semantic dimension", and "a false negative the adopter
owns by narrowing" — in prose that no reaction can read.

An owner SHALL NOT be carried by the other extents. Nothing is owed for a shape nothing observes by design,
and inventing an owner there would make the field decorative wherever it is not load-bearing.

#### Scenario: An under-reacting bound is declared without an owner

- **WHEN** a declaration records that the reaction fires less than the truth
- **THEN** the type requires the owner, so the declaration cannot be completed without saying who must act

#### Scenario: An inherited bound names the layer it is inherited from

- **WHEN** a bound exists because a layer beneath the dimension cannot distinguish the shape
- **THEN** the owner names that layer, so closing it in this dimension is visibly a fork rather than a fix

### Requirement: What the pinning test demonstrates SHALL be derived from the extent

The direction a bound's pinning test must demonstrate SHALL be a function of the extent, never a field beside
it. An out-of-reach or under-reacting bound is defended by a test showing the reaction does **not** fire; an
over-reacting bound by a test showing it fires on a shape that is not really a violation; a refusing bound by
a test showing exit 2. A separately declared direction carries no information the extent does not already
determine, and two copies of one fact can disagree.

#### Scenario: The demonstrated direction cannot contradict the extent

- **WHEN** a bound declares an over-reacting extent
- **THEN** the direction its pinning test must demonstrate is derived, so a declaration cannot claim the extent
  of one kind and the evidence of another

### Requirement: A declaration's defence state SHALL match the register vocabulary

Every typed declaration SHALL carry exactly one `Defence`: either `PinnedBy { first, additional }`, with at least
one pinning-test slot, or `Unpinned { tracker }`. The two states SHALL be mutually exclusive in the type, matching
the register's `PINNED-BY` / `UNPINNED` grammar. Multiple `PINNED-BY` lines on one scenario SHALL all be retained
in declaration order. An unpinned declaration SHALL carry a tracker and no fabricated test name. The comparison
path SHALL be exercised with both states even when the live declaration set contains no unpinned entry.

#### Scenario: A bound has no pinning test yet

- **WHEN** a declaration is created without a pinning test
- **THEN** it is expressible as `Unpinned` with its tracker, and cannot simultaneously claim `PinnedBy`

#### Scenario: One bound is defended by more than one test

- **WHEN** a scenario carries several `PINNED-BY` citations
- **THEN** the typed declaration retains every test while its pinned state keeps at least one test slot by construction

#### Scenario: No live bound is currently unpinned

- **WHEN** the comparison reaction runs while every live declaration is pinned
- **THEN** a local unpinned declaration still exercises the same typed conversion used by the live comparison and preserves its tracker without fabricating a live bound

### Requirement: A dimension SHALL export its declarations as library items

Each dimension owning declared bounds SHALL expose them from its library, not from `#[cfg(test)]` code. A
declaration compiled only when its own crate is under test is invisible to every other crate, so no single
reaction could hold the specs and the code in bijection — and the protocol that follows this change requires an
observer to declare its bounds as part of joining a run, which a test-only item cannot satisfy.

The crates owning declared bounds are the three dimensions **and the shell**, which owns the bounds of the
capabilities whose reactions live in it — this capability's among them, since a capability that exempted itself from
its own bijection would count everyone else's unclassified bounds while hiding its own. Their number is deliberately
not written here: a census belongs to whatever enumerates the set, and `crates/jiaochou/tests/bound_register.rs` prints it on every
clean run. A crate with no declared bound SHALL gain no export: an empty accessor would be a name with nothing
behind it.

**Where a reaction is behind a Cargo feature, the declarations describing it SHALL be gated with it.** A bound is a
property of a *reaction*; a build that compiles none of that reaction and still exports its bounds tells a reader a
limit exists for something the crate does not contain — an unbacked claim, which is the one thing this model exists
to refuse, arriving through the export rather than through the declaration. Measured: 漏刻 declared six bounds
unconditionally while five of them describe `audit_probe_coverage`, a scanner behind its non-default `audit`
feature, and `mod observer` immediately beneath the export was already gated for exactly that reason. A dimension
whose reaction is wholly gated and which therefore has **no** declaration in a given configuration falls under the
rule above and exports nothing there.

#### Scenario: A dimension's declarations are readable from another crate

- **WHEN** a reaction in the composed shell enumerates every declared bound
- **THEN** it reads each dimension's exported declarations directly, with no test-only visibility and no
  duplicated list

#### Scenario: A build compiles none of the reaction a declaration describes

- **WHEN** a dimension's reaction sits behind a Cargo feature and that feature is off
- **THEN** the declarations describing that reaction are absent from the export, so no dependent reads a bound for
  a reaction its build does not contain — while any declaration describing an always-present path stays

### Requirement: Each standalone dimension SHALL expose its shared protocol vocabulary

Each dimension root SHALL re-export the shared types an adopter needs to name its public observation-bound and
observer surface: `BoundDecl`, `BoundId`, `Defence`, `Demonstrates`, `Extent`, `FactGranularity`, `Observer`,
`Outcome`, `Owner`, and `Reached`. The exports SHALL preserve the original `xuanji` type identities rather than
introducing dimension-specific wrappers. An adopter depending on one dimension SHALL NOT need a direct `xuanji`
dependency merely to use that dimension's public protocol.

#### Scenario: An adopter depends on one dimension

- **WHEN** an external integration test imports the complete shared protocol vocabulary from any one dimension root
- **THEN** every type resolves and can be used with that dimension's declarations and observer implementation

### Requirement: The specs' declarations and the code's SHALL be held in bijection

A reaction SHALL assert that the set of bound ids declared in `openspec/specs/*/spec.md` **equals** the set
declared in code, and SHALL name every id on either side that has no counterpart. Ids SHALL be asserted
duplicate-free before the sets are compared, since two declarations collapsing onto one id would satisfy an
equality that proves nothing.

The id SHALL be the `<capability>/<scenario-slug>` form the register already derives, so this reaction
introduces no second naming scheme and no lookup table.

Both directions are required for the same reason the register requires both of its own: a spec bound with no
declaration is an unclassified claim, and a declaration with no spec bound is a classification of something no
reader can find.

**A dimension's declarations SHALL be read through `Observer::bounds`**, not through its exported free function.
The protocol requires a participant to declare what it does not observe, and a required method nothing reads is
answered into a void: measured, `bounds()` had no call site anywhere outside a comment, so a dimension could have
answered anything without moving a verdict. Reading the bijection through it makes the register the method's
consumer, and a dimension returning the wrong set now fails here.

The **shell's** own declarations SHALL keep coming from its free function, because the shell composes dimensions
rather than being one and implements no observer. That asymmetry is stated so it does not read as the same gap
this requirement closes.

#### Scenario: A spec declares a bound with no typed declaration

- **WHEN** a bound scenario is added to a spec and no declaration is added in code
- **THEN** the reaction fails, naming the id, because the qualifier slot it used to carry is gone and an
  unclassified bound would otherwise pass silently

#### Scenario: Code declares a bound no spec states

- **WHEN** a declaration exists whose id matches no bound scenario
- **THEN** the reaction fails, naming the id, because a classification a spec reader cannot find is a fact
  recorded where nobody looks

#### Scenario: Two declarations collapse onto one id

- **WHEN** two declarations carry the same id
- **THEN** the reaction fails before comparing the sets, because set equality would hold while one bound went
  unclassified

#### Scenario: An observer answers the bounds method with the wrong set

- **WHEN** a dimension's `Observer::bounds` returns a set other than that dimension's declarations
- **THEN** the bijection fails, naming every id left unclassified, because the register reads the answer rather
  than reading past it

### Requirement: The extents SHALL be projected into a generated, staleness-checked document

The reaction SHALL emit a projection grouping every declared bound by its extent, blessed by an environment
variable and diffed on every run, in the manner `AGENTS.self-law.md` and `docs/observation-bounds.md` already
are. It SHALL lead with the count of declared false negatives, because that figure is the family's own audit
backlog and a number in a footnote is not read — the same reason the register's projection leads with its
unpinned count.

The projection SHALL state what it does not claim, in its own header. Its rendering path SHALL be exercised for
both defence states even when the checked-in projection contains no unpinned entry.

#### Scenario: The projection is stale

- **WHEN** a declaration's extent changes and the projection is not regenerated
- **THEN** the reaction fails and names the blessing command

#### Scenario: A reader can count the declared false negatives without reading code

- **WHEN** a reader opens the projection
- **THEN** the number of under-reacting bounds and their owners lead the document

#### Scenario: The live projection has no unpinned entry

- **WHEN** a local unpinned declaration is rendered independently of the live declaration set
- **THEN** the projection path emits its tracker in the register vocabulary without changing the checked-in projection

### Requirement: Observation bounds

This capability SHALL declare its own limits rather than leave them to the model's silence: it classifies where
a reaction stops and does not verify the classification against the reaction, and a capability whose subject is
honesty about what is not observed cannot be implicit about its own.

Each is a limit of the **model**, not of a dimension's engine: one on prose the type cannot read, one on a value
the evidence did not earn, and one on a combination the nesting deliberately forbids.

#### Scenario: Whether a declaration's stated cause is the real cause is not observed — a stated bound

- **WHEN** a declaration's rationale string names a cause that is not why the reaction stops
- **THEN** the model does not claim to observe it, a stated bound: the extent is typed and checkable, the
  rationale is prose, and requiring the prose to match would trade a fact for a heuristic
- **PINNED-BY** `a_rationale_that_contradicts_its_extent_is_a_stated_bound`

#### Scenario: An answer that depends on the corpus entry point has no extent of its own — a stated bound

- **WHEN** a bound's outcome differs by which entry point observed it — one declared bound reports a seam
  covered from the root file and unprobed from the directory
- **THEN** it is declared as under-reacting with the entry point as the inherited owner rather than carrying an
  extent of its own, a stated bound: one live instance does not earn a value in a set every other member has
  several of, and the direction that matters (a seam reported covered when it is not) is recorded either way
- **PINNED-BY** `an_entry_dependent_bound_is_declared_as_under_reacting`

#### Scenario: A bound both out of reach and granularity-limited cannot be expressed — a stated bound

- **WHEN** a future bound is both invisible to the observation source and limited in the granularity of the
  fact it would have produced
- **THEN** the model cannot express it, a stated bound: no declared bound exhibits the pair, and offering
  granularity on every extent would invite a combination nothing shows while weakening the nesting that makes
  the contradiction above impossible
- **PINNED-BY** `granularity_is_carried_only_by_the_as_intended_extent`

### Requirement: A bound declaration SHALL carry owned-or-borrowed strings, so a bound may be declared at runtime

Every string a bound declaration carries SHALL be owned-or-borrowed (`Cow<'static, str>`) — its id, the shape it
names, its rationale, and the layer an inherited ownership names — so a declaration whose value is computed is
expressible while one written from literals borrows every string value it carries. This contract observes string
ownership only; it SHALL NOT be presented as measuring allocations by non-string storage or by the surrounding
governance run.

`Observer::bounds` carries no default body, so declaring bounds is a condition of implementing the protocol. An
implementor whose bounds are not compile-time literals — an observer over a discovered plugin set, or over roots it
scanned — was therefore mandated to declare its limits and given no way to name them. That is the shape this
requirement exists to remove.

The family's own declarations SHALL remain literals, and **a reaction SHALL measure that** rather than the
requirement asserting it in prose. A bound is a property of the *reaction*, and this family's reactions know their
limits when they are written; the owned form exists for implementors whose reactions do not. Since every
constructor accepts anything convertible, a family declaration rewritten as a computed string compiles and
allocates on every run of the register and the projection, and nothing would name it — a normative rule with no
reaction, in the capability that exists to refuse exactly that.

A declaration SHALL therefore be able to answer, of itself, whether every string it carries borrows. The answer
SHALL reach every string, including those nested in the extent and in an inherited owner's layer name, and SHALL be
decided by exhaustive matching **within the declaring crate**, so a variant added later carrying a string of its own
fails to compile rather than being silently unmeasured.

The reaction SHALL be shown able to answer **`false`**, for each string position independently. A discriminant that
is a constant `true` measures nothing, and one written as a single short-circuiting chain can pass while examining
only its first field.

The constructors SHALL accept anything convertible, so a declaration written as a literal reads the same as before.
The accessors SHALL lend `&str` borrowed from the declaration rather than promising `&'static str`, which is what an
owned-or-borrowed value can honestly lend.

#### Scenario: A bound whose id and rationale are computed

- **WHEN** an implementor declares a bound whose id, shape or rationale is built at runtime
- **THEN** the declaration is expressible, and the bound behaves exactly as a literal one — the same extent, the
  same derived evidence, and the same refusal of a duplicate id

#### Scenario: A literal declaration is unchanged and borrows every string

- **WHEN** a bound is declared from string literals, as every one of this family's own is
- **THEN** the call site is written exactly as it was, and every string value is borrowed rather than owned,
  without making a claim about other allocations

#### Scenario: One of the family's own declarations is rewritten as a computed string

- **WHEN** any string in any of this family's declarations becomes an owned value
- **THEN** the reaction fails, naming that declaration, because the rule that they stay literal would otherwise
  hold only for as long as nobody tested it

#### Scenario: A declaration carries a computed string in exactly one position

- **WHEN** a declaration owns its id, or its shape, or its pin, or its extent's rationale, or its inherited layer
  name, and borrows the rest
- **THEN** it answers that it does not borrow every string, whichever position it was — so the discriminant cannot
  pass by examining only the first
