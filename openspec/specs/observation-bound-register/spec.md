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

## Subject

- `openspec/specs/*/spec.md`
- `crates/kanhe/tests/bound_register.rs`
- `crates/kanhe/src/bound_register_parse.rs`
- `docs/observation-bounds.md`

The reaction runs as `cargo test -p kanhe --test bound_register`, so *violation* and *cannot-judge* below name
values of its result type rather than process statuses.
## Requirements
### Requirement: An observation bound is declared as a scenario that names itself one

An **observation bound** SHALL be declared as a `#### Scenario:` whose heading marks it as a bound, in
the spec of the capability whose reaction it bounds — a bound being a claim that an observation
deliberately stops at a named shape, so that shape is governed policy rather than a defect. The
declaring file SHALL be `openspec/specs/<capability>/spec.md`.

A bound MAY also be declared for a **classification a reaction does not make**, not only for a shape it does
not observe. Where two populations are told apart by a judgement rather than by a rule, the judgement is
carried by position and the absence of the rule is what gets declared: otherwise a reader takes the split for
something a run enforces.

The marking SHALL carry **no qualifier**. The recognizer previously admitted an optional free word before
"bound", and that slot accumulated many phrasings with no vocabulary governing any of them — one of them,
`cfg-blind`, used by two capabilities for bounds on **opposite sides** of the false-negative line, where the
direction is the whole content. A qualifier therefore read as a classification while classifying nothing. What
kind of stop a bound describes SHALL instead be carried by its typed declaration below, where the value set is
closed and a contradiction is a compile error. A heading carrying a qualifier SHALL fail, naming the heading and
the repair.

The bare singular phrases `a stated bound` and `a documented bound` SHALL remain interchangeable. They carry no
information — some specs use both forms internally — but they mislead no reader, where a qualifier did; and each
removal changes the bound's derived id, so a sweep is charged against every reference to it. Closing the harmful
half of the slot rather than all of it is a deliberate limit on that churn, not an oversight. Every repository
consumer that enumerates declared bounds SHALL call the register parser's canonical marker predicate rather than
reproduce this grammar. Article-less fragments, plural forms, and forms carrying an interposed qualifier SHALL
NOT declare a bound.

#### Scenario: Either canonical bare singular marker declares a bound

- **WHEN** a scenario heading contains `a stated bound` or `a documented bound`
- **THEN** every bound enumerator includes the scenario through the same canonical predicate

#### Scenario: A near-miss marker does not declare a bound

- **WHEN** a scenario heading contains `stated bound`, `stated bounds`, `documented bounds`, or an interposed
  qualifier but not either canonical bare singular phrase
- **THEN** every bound enumerator excludes the scenario, so one gate cannot classify a population the register
  never declared

#### Scenario: Which member holds a check is a judgement — a stated bound

- **WHEN** a check is added under `crates/shengmo/` or `crates/kanhe/`
- **THEN** nothing observes whether it landed in the right one. The split is by what a check judges — the
  law and the delivered product on one side, this repository's record on the other — and two mechanical rules
  were each measured unreliable: a text scan reads a comment naming `AGENTS.md` as governance while a check
  scanning every tracked file names nothing, and the workspace marker means both "this needs the repository as
  its subject" and "this needs a fixture". Position is the declaration; the join below catches a **capability**
  named wrongly, never a member chosen wrongly
- **UNPINNED** `BACKLOG.md` — *which governance member a check belongs to is unobserved*

#### Scenario: Whether a citation demonstrates the direction its bound declares — a stated bound

- **WHEN** a declared bound cites a test that runs, bites, and demonstrates a **different** direction from the
  one its extent predicts — a reacting distinction cited by a bound whose extent says the reaction stays silent
- **THEN** nothing reacts. `Extent::demonstrates()` names the direction a defence must show, and it reaches the
  projection's label and the contradiction classification beside it; no reader compares that prediction with what
  the cited test asserts. Reaching further means deciding what a test demonstrates from its source, which is a
  judgement over code of the same kind this repository has designed, measured and rejected over prose — and
  unlike a citation that never runs or never bites, there is no reaction here whose gap a fixture could exhibit.
  This is the sibling of *a rationale that contradicts its extent*, one step over: the prose beside an extent is
  already declared free to disagree with it, and so, until now silently, was the test beneath it
- **UNPINNED** `BACKLOG.md` — *a pin may defend a direction its bound does not declare*

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

A citation's syntax SHALL be validated before it is resolved. The cited name SHALL be an ASCII Rust
identifier, optionally raw (`r#name`); an optional crate qualifier SHALL be a crate-directory name; and at
most one `::` separator SHALL appear. Anything else SHALL fail, naming the bound id and the rejected citation.
This closes two directions **by construction** rather than by escaping. The name is interpolated into the
search pattern, so a regular-expression metacharacter would let a citation for a test that does not exist
resolve to a differently-named function — defeating the renamed-or-deleted direction this requirement exists
for. The qualifier is joined to a filesystem path, so `../` would resolve a citation against a function
outside the `crates/` boundary this requirement declares.

The restriction to ASCII is narrower than Rust's own identifier grammar and SHALL be stated as such rather
than implied: the search pattern is byte-oriented, no cited name needs otherwise, and the refusal of a
non-ASCII identifier is loud — an author sees it — where accepting one and matching it unreliably would not
be.

**Whether a cited name is a test that runs SHALL be decided by the test harness, not by the source text.**
The reaction SHALL enumerate each workspace member's registered tests and SHALL fail when the cited name is
absent from the cited crate's set. Enumeration SHALL be per package rather than per workspace, because the
enumeration carries no crate label while a citation may be crate-qualified — this repository already has one
test name registered in two crates, so a workspace-wide match would let a citation qualified to one crate be
satisfied by the other's test.

The harness is the authority because it is the only exact observation source for the claim. A text scan reads
shape, so it accepted a `#[test]` neutralised by `#[cfg(any())]`, a `#[test] fn` inside an uninvoked
`macro_rules!` body, and a definition inside a raw string or a block comment — all measured, none of which
registers a test. Enumerating those sub-cases in the scan is unbounded (`cfg`, `cfg_attr`, feature gates, a
cfg-gated `mod`, comments, strings, macros), and the previous version of this requirement declared one of them
as a residual before three more were found.

**The text scan SHALL remain as a declared fallback, and the degradation SHALL be reported.** A repository
with no root manifest cannot be enumerated — the failure matrix builds such repositories deliberately — so
there the attribute-run walk decides test-ness, and the reaction SHALL say on its own output that it did. A
gate that silently drops its strongest direction reports a weaker clean than the one it claims.

Where the enumeration itself cannot be produced — no `cargo`, or a workspace that does not build — the
reaction SHALL refuse as a **cannot-judge** rather than fall back silently, because a citation's test-ness is
then undecided rather than decided weakly.

**Every `PINNED-BY` citation SHALL be resolved wherever it appears, independent of whether its scenario
heading also marks a bound.** The marker means one thing in both places — *this test is the evidence* — and a
renamed test leaves an ordinary scenario citing nothing exactly as silently as it would a bound. Measured: the
tracked specs held 75 citations, of which 70 sat under bound headings and were resolved while 5 under ordinary
scenario headings were parsed by nothing; renaming one of the five left the whole gate suite green with a spec
citing a function that no longer existed. The register's own corpus stays bound-gated, because a citation
under an ordinary scenario declares no bound and admitting it would invent one — it is resolution, not
declaration, that follows the marker rather than the heading. This is the same rule the reference direction
below already states for its own marker.

The reaction SHALL verify that each `PINNED-BY` name resolves to exactly one Rust function **definition**
under `crates/`. Resolving to none SHALL fail: a test that was renamed or deleted leaves a citation that reads
as coverage while defending nothing. Resolving to more than one SHALL also fail: a name defined twice makes
the citation name a set rather than a reaction, so the bound's defender is not identified. This direction
supplies the **site**, which the enumeration does not carry, and the crate precision that makes a qualified
citation exact.

When the harness registers the cited test but the definition scan locates no site, the reaction SHALL report
the **line-shape limitation** — the scan requires `fn` and the name on one line — rather than reporting the
test absent, since the two directions disagree about a form rather than about existence.

The fallback walk SHALL recognize a definition as a test by an attribute run immediately above it containing
`#[test]`, read upward past interleaved attributes, to the enclosing item's boundary, with no line cap, and
stopping at a block-comment delimiter rather than interpreting one.

Requiring the cited function to be a test is not a naming convention imposed on a suite the register does not
own; it is what the citation already means. The register SHALL require nothing of the test's **name** beyond
its being an identifier.

#### Scenario: A citation whose name is not an identifier

- **WHEN** a declared bound's `PINNED-BY` contains a character no ASCII Rust identifier may hold
- **THEN** the reaction fails before resolving it, naming the bound id and the rejected citation, so a
  metacharacter cannot resolve a citation to a differently-named function

#### Scenario: A citation naming a raw identifier

- **WHEN** a declared bound's `PINNED-BY` names `r#name`
- **THEN** the reaction accepts the citation's form, because a raw identifier is a Rust identifier and the
  register imposes no naming convention of its own

#### Scenario: A citation whose crate qualifier leaves the crates directory

- **WHEN** a declared bound's `PINNED-BY` qualifier is not a plain crate-directory name — a traversal, a
  nested path, or a second `::` component
- **THEN** the reaction fails before resolving it, so a citation cannot be satisfied by a function outside
  the boundary this requirement declares

#### Scenario: A cited name the harness does not register

- **WHEN** a cited name is absent from the enumerated tests of the crate it is qualified to, or of the
  workspace when unqualified
- **THEN** the reaction fails, naming the bound id and the name, because a citation names what defends the
  bound and an unregistered function defends nothing

#### Scenario: A test neutralised by a cfg attribute

- **WHEN** a cited function carries `#[test]` and a `cfg` attribute that removes it from the build
- **THEN** the reaction fails, because the attribute run says test while the harness registers nothing

#### Scenario: A test inside an uninvoked macro body

- **WHEN** a cited function's `#[test] fn` tokens sit inside a `macro_rules!` body that nothing invokes
- **THEN** the reaction fails, because tokens that expand nowhere register no test

#### Scenario: A definition inside a string or a block comment

- **WHEN** a cited function's definition sits inside a multi-line string literal or a block comment
- **THEN** the reaction fails, because the harness registers no test for it — retiring the residual the
  previous version of this requirement declared for the block-comment case

#### Scenario: The harness cannot be enumerated

- **WHEN** the judged repository has no root manifest
- **THEN** the reaction decides test-ness by the fallback walk and reports on its own output that it did, so a
  reader of a clean result knows which direction produced it

#### Scenario: The enumeration cannot be produced at all

- **WHEN** a root manifest exists but the enumeration fails — no `cargo`, or a workspace that does not build
- **THEN** the reaction refuses as a **cannot-judge**, because test-ness is undecided rather than weakly
  decided

#### Scenario: A citation naming a test defined twice

- **WHEN** a declared bound's `PINNED-BY` name is defined by two functions under `crates/`
- **THEN** the reaction fails, naming the bound id and both definition sites, because the citation is
  ambiguous rather than merely imprecise

#### Scenario: A registered test the definition scan cannot locate

- **WHEN** the harness registers a cited test whose `fn` keyword and name sit on different source lines
- **THEN** the reaction reports the line-shape the scan requires, rather than reporting the test absent

#### Scenario: A citation satisfied only by a mention

- **WHEN** a declared bound's `PINNED-BY` name appears in the tree only inside a comment or a string, with no
  registered test of that name
- **THEN** the reaction fails, because a mention defends nothing

#### Scenario: A pinning test whose attribute run carries another attribute

- **WHEN** the fallback walk reads a definition preceded by `#[test]` and then a further attribute such as
  `#[should_panic]`
- **THEN** it resolves as a test, so the fallback reads the attribute run rather than one line

#### Scenario: A pinning test whose attribute run is longer than any cap

- **WHEN** the fallback walk reads a definition whose `#[test]` sits above more interleaved attributes than a
  fixed-window walk would read
- **THEN** it still resolves as a test, because the walk ends at the item boundary rather than at a line count

#### Scenario: An attribute written inside a block comment

- **WHEN** the fallback walk reads a definition whose `#[test]` sits inside a block comment
- **THEN** it does not resolve as a test, because the walk stops at the delimiter rather than reading
  commented text as an attribute

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

### Requirement: A pinning citation MAY declare the mutation it dies under, and every declared mutation SHALL kill it

*A cited pinning test SHALL resolve to exactly one definition in the tree* decides that a citation names a test
that **runs**. It does not decide that the test **bites**. A pin whose assertions are deleted, or whose subject
is loosened back toward the rule it was written to refuse, keeps resolving, keeps carrying `#[test]`, keeps
being registered by the harness, and keeps occupying the place of a defence. Measured in this repository, not
supposed: retiring the composition-body reaction deleted the only assertions over the anchor-counting rule, the
suite stayed green, and the rejected alternative could be restored with nothing refusing.

The question is not decidable from text, and the register already says why for the easier question one level
down — a `cfg`-removed attribute, an uninvoked macro body, a definition inside a string or a comment. Whether a
test *would fail* under a different reaction is a question about running a program. The reaction SHALL
therefore **run the cited test against a mutated tree** and read its status, never infer biting from the shape
of either the test or the reaction.

A **mutation** SHALL be declared as four fields: the cited test name, a tracked path, a `from` substring, and a
`to` substring. It SHALL be applied to a **separate checkout of HEAD**, never to the working directory, so that
an interrupted run has edited nothing of the author's.

The path a record names SHALL be both **tracked** and **contained**, and the two SHALL be tested separately
because neither implies the other. `[[ -f $tree/$file ]]` asks neither: a `../` path resolves outside the tree
and the mutation rewrites a file the reaction has no business touching. Asking git asks only tracked-ness: a
tracked **symlink** is tracked, a checkout materializes it as a symlink, and both the backup copy and the write
follow it — so the outside file is rewritten, and a run killed between write and restore leaves it destroyed.
Both were measured. Containment SHALL therefore be decided on the **resolved** path, which is the form that
answers the question the refusal's message claims to ask.

What that holds is that a **record** cannot name a path outside the checkout. It is not a boundary on where the
reaction can write, and the requirement SHALL NOT be read as one: the check precedes the build, and the build
runs the checkout's own code — a build script replacing the checked path with a symlink afterwards redirects
the write, reproduced. Re-checking would re-check the same window. That is the code-execution the reaction
grants unconditionally by testing there at all, and it is declared as a bound rather than guarded.

That checkout SHALL carry a working repository. An export of tracked content alone makes some citations
structurally **unreachable** rather than merely uncovered — a pin that reads the repository through git fails
its own control run, so no record can ever exercise it, which the coverage story would otherwise misdescribe as
work not yet done.

Carrying one has a cost that SHALL be paid rather than assumed away: the checkout shares the judged
repository's common directory. Its **hooks SHALL be disabled** for the checkout, because a `post-checkout` hook
would otherwise run inside the tree under test with write access to the judged repository's refs — measured, a
planted tag survived the reaction's own cleanup, and the same hook could have rewritten the tree so that what
ran was no longer HEAD's content. The shared common directory itself is inherent to a checkout of this kind and
is declared as a bound below.

Cleanup SHALL be claimed only where it happens. A checkout registered in the judged repository is removed on an
ordinary exit and on interrupt and terminate signals; a killed process reaches no handler, and the registration
then survives — `git worktree prune` will not clear it while the directory it names still exists. Stating that
beats asserting an automatic cleanup the reaction does not perform.

Judging HEAD is **not** the rule this family holds, and the difference is stated so neither is read as the
other: the sibling gates enumerate tracked *paths* and read the *worktree's* content, deliberately — the
whitespace gate's own header calls reading anything else a false negative. This reaction is the one that judges
HEAD's content, and the blind spot that follows is declared below rather than left to be discovered.

The reaction SHALL build that checkout with its **own target directory**, because its premise is that the
binary under test was built from the *mutated* tree. A shared directory has been seen to serve one that was
not, and a verdict over the wrong binary is not a verdict.

What is *not* claimed is which wrong verdict follows. Two reproduction attempts each landed somewhere else — a
directory holding the previous run's mutated binary fails the **control** run, and one pre-warmed from an
unmutated build simply rebuilds and reports correctly — so no false-clean direction was found, none is claimed,
and the matrix has no direction for this requirement. Requiring it on the premise rather than on an observed
outcome is the honest form: the guarantee claimed is exactly the one held.

A `from` that occurs zero times or more than once in the named file SHALL be a **cannot-judge**, not a violation.
The mutation could not be applied, which is a different fact from the pin not biting, and reporting the second
for the first lets a mutation whose anchor has rotted read as a pin that has been exercised. Requiring the
anchor to be unique is the rule the observer protocol's body reader reached by the expensive route in the same
window: an anchor matching twice names a set rather than a site.

One direction of the record-to-citation relation SHALL react and the other SHALL be disclosed, and the
difference is stated here so neither is read as the other. A mutation naming a test no declared bound cites
SHALL fail: it perturbs something this register makes no claim about, and its passing would read as coverage of
a citation that does not exist. A citation carrying no mutation SHALL NOT fail — that is the coverage this
requirement admits is partial, below.

Each record SHALL first be run **unmutated**, and the reaction SHALL refuse to judge unless that run passes.
Without the control, a cited test failing for its own reasons reads as a pin that bites the moment anything is
applied to the tree — the same `f() == f()` shape reached from the other side, where the comparison is with a
run whose outcome was never in doubt.

Where the mutated run **fails**, the control SHALL be run again after the restore, and the reaction SHALL
refuse to judge unless it passes then too. One control rules out a test that fails deterministically on its
own; it cannot rule out one whose failure it *caused*. A pin writing a marker and asserting the marker's
absence passes exactly once, so the mutated run fails for a reason the mutation had no part in and the citation
is reported as exercised by a perturbation that did nothing — a false clean, constructed and measured. Where
the restored-tree run fails, the outcome is order-dependent and no failure under the mutation can be attributed
to it. It is run only on that branch because a *surviving* pin is a violation whatever the second run says.

**Every** run of the cited test SHALL be held to having executed exactly one test, the restored-tree run
included. It was added without that rule, and a pin that rewrites its own source on a later run then left the
filter matching nothing: exit 0 over zero tests, read as the restored tree still passing. Measured. The restore
puts back the mutated file alone, so whatever the test itself wrote survives into that run.

What none of this reaches is declared below rather than implied: the number of runs is **fixed**, so a cited
test whose outcome depends on its run count with any period the sequence does not break is indistinguishable
from one the mutation killed.

The package and target the cited test runs in SHALL be **derived from where that test is defined**, not from the
file the mutation edits and not declared beside the mutation. The mappings from a definition path to a target SHALL be an
**allowlist**, and a path matching none of them SHALL be a cannot-judge. Assuming a library test for whatever did
not match ran a *different* test of the same name and reported that one's death as the citation's, while the
cited pin never ran at all — measured twice, first for a module of an integration target and then, after a
denylist closed that instance, for a binary target the denylist did not name. Refusing what cannot be mapped is
what closes the class rather than its instances. A record routinely perturbs a reaction in one file
while the pin defending it sits in another, and deriving from the edited file then runs a target the citation is
not registered in — measured here, where a recognizer in a crate's library and its pin in an integration target
selected the library. A fifth field would be a second spelling of a fact the tree already carries, and would rot.
A name whose definition the scan finds zero or several times SHALL be a cannot-judge, because a target cannot be
derived from a set.

An enumeration of citations that yields none SHALL fail loudly rather than report every mutation valid against
an empty set — the vacuity direction, in the second of this capability's two enumerations. It SHALL be carried
by the **read**, which refuses a match of nothing as a failed read, rather than by a comparison against zero
afterwards: with the read refusing first, such a comparison can never fire, and a guard that cannot fire reads
as protection while being none.

A mutation whose tree does not **compile** SHALL be a cannot-judge, for the same reason as a rotted anchor: the
perturbation was never exercised. `cargo test` exits non-zero for a compile error as well as for a failing
assertion, so a reaction distinguishing them by status alone would read a broken build as a biting pin.

A cited name the harness registers **more than once** in the selected target SHALL be a cannot-judge before any
run, for the same reason the definition scan refuses a name found in several files: a filter matching several
does not name the citation. A run SHALL also be required to have executed **exactly one** test. A filter matching nothing exits 0 having run
nothing, which by status is indistinguishable from a pin that survived its mutation — measured here, where a lib
test registers under its module path while the citation is the bare identifier.

Coverage SHALL be partial and SHALL say so. A clean run SHALL print how many **distinct cited tests** carry no
declared mutation. The population SHALL be named, because it is not the register's: the register counts
*citations*, one per declaration, and blesses one test cited by two bounds in one capability, so an unqualified
figure here would become the fifth answer that *A declared bound SHALL carry exactly one citation naming its
defence* makes the register the arbiter of. Both sides of the remainder SHALL be counted over that same
population; subtracting a record count from a name count made the disclosure read `-1`, measured. A gate that
reported only the mutations it ran would be a reaction reading as coverage, which is this requirement's own
subject one level up.

What that leaves unobserved is declared as a bound below and tracked rather than pinned
(bound: observation-bound-register/whether-a-citation-carrying-no-declared-mutation-is-defended-is-not-observed-a-stated-bound): closing it
is coverage, which grows one authored record at a time.

#### Scenario: A pin that survives its declared mutation

- **WHEN** a declared mutation is applied and the cited test passes
- **THEN** the reaction fails, naming the citation, the mutation, and the bound whose defence it is, because a
  test that cannot tell the reaction from its perturbation defends nothing while occupying the place of a
  defence

#### Scenario: A pin that dies as declared

- **WHEN** a declared mutation is applied and the cited test fails
- **THEN** that citation is reported as exercised, and the failure output is not treated as the gate's own
  failure

#### Scenario: A mutation whose anchor is absent or ambiguous

- **WHEN** a mutation's `from` occurs zero times, or more than once, in the file it names
- **THEN** the reaction refuses to judge rather than reporting either a biting or a dead pin, because the
  perturbation it describes was never applied

#### Scenario: A mutation naming a test no bound cites

- **WHEN** a mutation record names a test that appears in no declared bound's citation
- **THEN** the reaction fails, because a mutation is an assertion about a defence and there is no defence here
  to assert about

#### Scenario: The uncovered remainder is disclosed on a clean run

- **WHEN** every declared mutation kills its citation
- **THEN** the reaction still prints how many citations carry no mutation, so a clean result cannot be read as
  every pin having been exercised

#### Scenario: The mutation set is empty

- **WHEN** no mutation is declared at all
- **THEN** the reaction refuses to judge, saying the set was empty, because every property of zero mutations
  holds and reporting that as conformance is the vacuity direction this repository has re-opened most often

#### Scenario: One records file read by two parsers

- **WHEN** the set is counted by one splitting rule and processed by another
- **THEN** the reaction reports clean over a file holding nothing to run — a records file whose only remaining
  line was a TAB-indented comment counted as a declared mutation and was skipped as prose, exiting 0. The
  records SHALL therefore be parsed **once**, and a line that is neither prose nor four TAB-separated fields
  SHALL be a cannot-judge rather than skipped

#### Scenario: A mutation that does not compile

- **WHEN** the mutated tree fails to build
- **THEN** the reaction refuses to judge, because `cargo test`'s non-zero status there reports a broken build
  and not a test that failed

#### Scenario: A filter that runs no test

- **WHEN** the cited test's filter matches nothing and the harness exits 0 having run nothing
- **THEN** the reaction refuses to judge rather than reading the exit status as a pin that survived

#### Scenario: What code executed inside the checkout does outside it is not observed — a stated bound

- **WHEN** code the reaction runs inside the checkout — a cited test, or a build script the build invokes —
  writes outside that checkout, whether to the repository it was taken from or by replacing a checked path so
  the reaction's own write lands elsewhere
- **THEN** nothing reacts. Running the cited test is the reaction's whole method, so code execution inside the
  checkout is granted unconditionally, and neither consequence is separable from it: the shared common
  directory is what makes a git-reading citation reachable at all, and re-checking a resolved path after the
  build would re-check the same window that defeated it. Hooks are the one case that IS closed, because those
  run without any citation asking for them
- **UNPINNED** `BACKLOG.md` — *most pinning citations have never been seen to fail*

#### Scenario: Whether a cited test's outcome depends on its run count is not observed beyond one period — a stated bound

- **WHEN** a cited test passes and fails by a period the reaction's fixed run sequence does not break — passing
  the control, failing the mutated run, and passing the restored-tree run, for reasons of run count alone
- **THEN** the citation is reported as exercised by a perturbation that did nothing. The reaction runs the test
  a fixed number of times, so any period matching that sequence escapes it, and the number is readable in the
  reaction's own source. Closing it needs each run to be unable to observe how many times the test has run —
  a separate checkout per run — whose cost grows with the coverage this capability exists to grow
- **UNPINNED** `BACKLOG.md` — *most pinning citations have never been seen to fail*

#### Scenario: Whether a pin gutted but not committed still bites is not observed — a stated bound

- **WHEN** a cited pin's assertions are removed in the working directory and not committed
- **THEN** nothing reacts, because the checkout under test is HEAD's content. The sibling gates read the
  worktree for exactly this reason, and this reaction cannot: mutating the author's checkout is what a separate
  checkout exists to avoid, so the two properties are in tension and this one is given up deliberately
- **UNPINNED** `BACKLOG.md` — *most pinning citations have never been seen to fail*

#### Scenario: Whether a record perturbs the check or the pin's own assertions is not observed — a stated bound

- **WHEN** a record names the file its pin lives in and neutralises one of that pin's assertions
- **THEN** the pin fails and the citation is counted as exercised, because a killed pin does not say what killed
  it. Separating the two by refusing a record that edits its pin's file was measured against this tree's own
  first record, which legitimately edits the file its pin lives in, so the rule would refuse a conforming shape
- **UNPINNED** `BACKLOG.md` — *most pinning citations have never been seen to fail*

#### Scenario: Whether a citation carrying no declared mutation is defended is not observed — a stated bound

- **WHEN** a pinning citation declares no mutation
- **THEN** the reaction does not decide whether that pin bites, and says how many such citations there are on
  every clean run rather than leaving the gap to be inferred
- **UNPINNED** `BACKLOG.md` — *most pinning citations have never been seen to fail*

### Requirement: A bound stated in prose but not declared as a scenario SHALL fail

The reaction SHALL scan `openspec/specs/*` for bound-declaring prose and SHALL fail on any occurrence
outside a declared bound scenario, **subject to the exemptions and residuals stated below, which SHALL be
enumerated rather than implied**. This makes the prose already present the register's mandatory minimum, so
the register cannot be completed by declaring only the convenient bounds. Its size is measured rather than
estimated by whoever wrote it: the reaction prints what it counted on every clean run, and a figure typed
here would be a census in prose — the class `AGENTS.md` forbids, and one this sentence has already demonstrated,
having had its denominator re-swept while its numerator was left behind.

One **exemption** is deliberate and SHALL be declared here rather than only in the reaction's own comments.
Prose under a requirement whose heading names bounds is not reported, because several such requirements
state their bounds as a numbered list — `Observation bounds are stated, not silent` enumerates seven — and
requiring each item to become its own scenario would restructure three requirements and read worse. The
exemption is not free, and its price SHALL be charged: such a requirement SHALL declare at least one bound
scenario, or its list would have no reaction anywhere. What the exemption costs is that the *other* items of
such a list are unregistered, and that cost SHALL be stated in the projection's header.

The direction SHALL be described as a **floor and not a proof**, in the generated projection's own header,
and every residual known to the reaction SHALL appear there. Three are known and SHALL be named:

1. A bound worded outside the scanned pattern — "out-of-scope", "does not claim to observe" — is
   undetectable.
2. The scan is **line-oriented**, so a statement whose bound names continue onto the next line is examined
   only on the line carrying the trigger words.
3. A resolvable bound reference clears the prose it sits with **regardless of how many bounds that prose
   states**, and regardless of whether the referenced bound is one of them. The clearing form is any
   reference the reaction resolves — the parenthesised `(bound: …)` and a bare capability-qualified id
   alike, since the id is no less a reference than the wrapper around it — so naming only the
   parenthesised form here understated the residual by the wider half of what actually clears.

Residual 3 is the mechanism that let a retired `#[path]` bound survive in a capability's overview paragraph
through two sweeps, so it SHALL be recorded as the reason rather than as a curiosity. Closing it would
require reading which bounds a sentence lists, which is a semantic judgment no reaction can reach; residual
2's obvious repair — scanning paragraphs rather than lines — SHALL NOT be adopted on that account, because
it was measured against this defect and **would not have caught it**: the paragraph carries the reference
that clears it, so the repair costs twelve new offenses and buys nothing against the failure that motivated
it.

These residuals SHALL NOT be declared as bounds of this capability, for the reason already settled for the
first: nothing observes them, and a declaration no reaction can reach is the name-without-a-reaction
`PROJECT.md` forbids. The register must not make itself the exception.

#### Scenario: Spec prose states a bound that no scenario declares

- **WHEN** a spec paragraph or a bare THEN clause states that an observation stops at a shape, and no
  bound scenario declares it
- **THEN** the reaction fails, naming the file and the occurrence

#### Scenario: The same statement inside a declared bound scenario does not fail

- **WHEN** the bound-declaring prose sits inside a declared bound scenario
- **THEN** the reaction passes for that occurrence, so declaring the bound is what clears it rather than
  rewording the sentence

#### Scenario: Prose under a bounds-named requirement is exempt, and the requirement pays for it

- **WHEN** a requirement whose heading names bounds states one in prose
- **THEN** the occurrence is not reported, and the reaction instead requires that requirement to declare at
  least one bound scenario, failing when it declares none

#### Scenario: The register states every residual of its prose direction

- **WHEN** the projection is read
- **THEN** its header names all three residuals — unrecognized wording, the line-oriented scan, and a
  reference clearing prose that states more bounds than it names — and no bound of this capability claims
  any of them, since no reaction could reach one

### Requirement: Prose MAY reference a declared bound, and a reference SHALL resolve

Prose that mentions a bound SHALL be cleared by the undeclared-prose reaction when it carries an explicit
reference of the form `(bound: <capability>/<slug>)`, where `<slug>` is the declaring scenario's heading
lowercased with each run of non-alphanumeric characters replaced by a single hyphen.

**Every reference SHALL be resolved wherever it appears, independent of whether its line also states a bound.**
Resolution belongs to the id, not to the wording around it: a reference reachable only through the
bound-prose scan is un-checked the moment a sentence is reworded out of that scan's pattern, which happened
here — a repair reworded a capability's overview out of the scan's pattern while improving it, and the two
references that repair added were never resolved again. A reference in a Purpose
paragraph, a requirement's prose, or inside a declared bound scenario SHALL be resolved the same way.

Each reference SHALL resolve to exactly one declared bound across all specs, and **every** reference the prose
carries SHALL be checked rather than one of them: resolving to none SHALL fail, because a reference that points
nowhere is indistinguishable from an undeclared bound, and resolving to more than one SHALL fail, which is also
what keeps derived ids unique rather than merely assumed unique.

What a reference does **not** establish SHALL be stated wherever the reference form is described: it clears
the prose it sits with, and it does not certify that the bounds the prose states are the bound it names. A
sentence listing four inherited bounds is cleared by one reference to a fifth. Authors SHALL therefore carry
one reference for each bound the prose names, and the reaction cannot enforce that.

A reference exists because the floor's alternative is worse. Without it, a sentence that legitimately
**points at** a bound declared elsewhere — in the same file, or in another dimension's spec — must either
be rewritten to avoid the words or be restated as a second declaration of the same bound. The first
degrades prose that is doing its job; the second is exactly the restatement this register exists to end,
and the drift it produces is already recorded as a live `BACKLOG.md` item.

A reference SHALL NOT be treated as a declaration: it carries no citation of its own, contributes nothing
to the register's bound count, and cannot be the only mention of a bound anywhere.

**A bound id written bare SHALL resolve too, wherever tracked Rust or Markdown carries it.** The `(bound: …)`
form is what *clears prose*; resolution belongs to the **id**, and an id is no less a reference for being
written without the wrapper. Measured before this was proposed: three occurrences across the tree did not
resolve, every one of them naming a bound that does exist under an id it does not have. They sat in the two
places the bijection cannot look — a doc comment citing a bound, and a unit-test fixture constructing a
`BoundDecl` that mirrors one, together with the assertion round-tripping it. Neither is a *declaration*, and
the bijection compares the two declaration sides. The fixture is the sharper case: its pin name and its shape
string both carried the full wording while its id carried an abbreviated one, so a declaration had drifted from
itself inside a single constructor with nothing able to say so.

**Recognition SHALL be by shape against the enumerated capability set, never by a list written beside it.** A
reference is a **maximal run of path characters** that is exactly `<capability>/<slug>`, where `<capability>`
is a directory under `openspec/specs/` and `<slug>` is kebab-case. Reading maximal runs is what keeps a path
from being mistaken for a reference: a spec's own path is one run carrying three slashes, so it is not a
`<capability>/<slug>` pair — the same word-reading rule the adopter-narrative reaction already applies for the
same reason. Enumerating the capabilities rather than listing them is the register's own prohibition: a
capability added later must be recognized without this reaction being touched.

**That last property is held by construction, and is stated here rather than given a scenario.** The capability
set is a *parameter* of the recognizer, filled by the reaction from the tracked spec directories, so a
capability added later is covered because there is no second list to fall behind. A scenario for it would have
to assert that the reaction passes the set it passes, which is one function compared with itself — the shape
this repository refuses. What a case table *can* show is that the set governs the answer, and one does: a
recognizer shown a capability the tree does not declare finds references under it.

**This is reference resolution, not a judgement over prose.** The distinction is the one this repository has
drawn three times when rejecting a detector over sentences: a bound id has a recognizable shape and the set it
must land in is *produced* by the declarations, exactly as a path, an `--exact` identifier and a `(bound: …)`
reference already are. Nothing here decides what a sentence means.

**A reaction over tracked text reads its own text, and a fixture SHALL be written for that.** A row needing an
invented slug SHALL hang it off a capability no spec directory carries, and prose explaining that SHALL
describe the shape rather than spelling an offending token — both were observed here, one after the other,
within this reaction's own first two runs.

What it does **not** establish is the same as for the wrapped form and SHALL be stated with it: resolving says
the id names a declared bound, never that the prose around it describes that bound.

#### Scenario: A bare id in a doc comment names no declared bound

- **WHEN** tracked Rust or Markdown carries `<capability>/<slug>` with no `(bound: …)` wrapper, and no declared
  bound produces that id
- **THEN** the reaction fails, naming the file, the line and the unresolved id — the same refusal the wrapped
  form gets, because resolution belongs to the id rather than to the syntax around it

#### Scenario: A path that merely contains a capability name

- **WHEN** tracked content carries a path whose characters include a capability name followed by a slash
- **THEN** it is not read as a reference, because a reference is a maximal run of path characters that is
  exactly `<capability>/<slug>` and such a path is neither

#### Scenario: Prose referencing a declared bound is cleared

- **WHEN** a sentence mentions a bound and carries `(bound: <capability>/<slug>)` naming a declared bound
- **THEN** the reaction passes for that occurrence, and the register's bound count is unchanged

#### Scenario: A reference that resolves to nothing

- **WHEN** a reference names a `<capability>/<slug>` that no declared bound produces
- **THEN** the reaction fails, naming the file, the line, and the unresolved id, because a dangling
  reference is indistinguishable from an undeclared bound

#### Scenario: A reference on a line that states no bound

- **WHEN** a reference sits in prose that does not itself match the bound-prose pattern — a Purpose
  paragraph, or a sentence reworded away from those words
- **THEN** the reaction resolves it exactly as it would on any other line, so rewording a sentence cannot
  silently un-check the references it carries

#### Scenario: An earlier reference on the same line that resolves to nothing

- **WHEN** prose carries two references and only the later one resolves
- **THEN** the reaction fails, naming the unresolved one, because a line examined at one reference leaves the
  rest unchecked whichever one that is

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

**Staleness checking SHALL NOT be mistaken for content checking, and the companion test SHALL assert the
document's content directly.** A byte-for-byte comparison proves the document and the reaction agree; it
cannot prove either is right, because both come from one renderer. A mangled apostrophe rendered as
`author\s:` survived a full review in the tracked document for exactly that reason. The companion test
SHALL therefore assert that each disclosure the requirements oblige the header to make is literally present,
and SHALL refuse a rendered backslash, which this document's prose never wants and which is therefore a
quoting artifact rather than content.

**A hand-written census of the register SHALL NOT live in prose, and the reaction SHALL emit the figures.**
A count of a set the reaction already enumerates is a claim with no observation source — the class this
capability exists to end — and it went stale three times in one release window, the third time inside the
entry recording that the first two had. Carelessness is not the cause: four independent, deliberate counts of
this tree produced four different answers for the number of citations. A clean run SHALL therefore print the
bound, capability, **pinning-citation**, unpinned, and reference counts, so prose is written from a
measurement rather than from memory, and prose SHALL state a figure only where a reader outside the
repository needs one.

The pinning-citation figure SHALL be **labelled as such rather than as "citations"**, because this
specification defines a citation as *either* form — `PINNED-BY` or `UNPINNED` — so an unqualified figure names
two different numbers depending on which sense a reader carries. That ambiguity is the actual cause of the
four disagreeing counts, and a reaction emitting an unqualified figure would become a fifth answer rather
than the arbiter. Labelled, the printed pinning-citation and unpinned counts sum to this specification's sense
with nothing left to infer.

Where such a figure is stated, the reaction SHALL react to it: a **tracked Markdown** document writing
`N bounds across M capabilities` SHALL fail when either number disagrees with what the reaction counted. The
matched shape SHALL be narrow rather than a general number-in-prose matcher, because a heuristic over prose
figures would refuse unrelated numbers, which is how a gate earns the false positives that get it disabled.

The scan SHALL read **tracked Markdown**, through the same `git ls-files` this reaction already uses for
specs and for trackers, and that scope SHALL be stated here rather than inferred from a glob. A filesystem
walk is forbidden: it judged the worktree, so an untracked scratch note and an ignored vendored tree each
failed the reaction, which makes a local file break a developer's run while a clean checkout passes — the
checkout-dependence this family repairs wherever it appears.

**Every** figure on a line SHALL be checked, and a figure SHALL be recognized wherever it sits on that line,
including at its start. A matcher that guarded the number against a preceding digit could not match at a
line's first column, so a line-initial census — which reflowed Markdown produces routinely — was silently
skipped while the identical figure mid-line was caught; and a greedy match examined only the last figure of
two, the same partial check the reference direction was already repaired for. A longest-match extraction
SHALL be used instead, so a longer written number is read whole rather than sliced into a false agreement.

This direction's own residual SHALL be stated rather than left implicit: it is **line-oriented**, so a figure
reflowed across a line break — `… 15` ending one line and `capabilities` opening the next — is invisible to
it, exactly as the undeclared-prose scan is. Closing it would mean joining lines before matching, which costs
the line number the diagnostic needs and would match across a paragraph boundary; the residual is therefore
recorded here rather than repaired, and it SHALL NOT be declared as a bound of this capability, for the reason
already settled for the prose floor's residuals — nothing observes it.

#### Scenario: The projection is stale

- **WHEN** a bound is declared, changed, or removed without regenerating the projection
- **THEN** the reaction fails, reporting that the projection no longer matches the specs

#### Scenario: The projection is regenerated

- **WHEN** the projection is regenerated from the specs
- **THEN** it matches byte-for-byte and the reaction passes, so the document has one source of truth

#### Scenario: Unpinned bounds are counted where a reader cannot miss them

- **WHEN** the register contains bounds whose defence is a tracker rather than a test
- **THEN** the projection states their count in its header, not only in the affected entries

#### Scenario: The projection's disclosures are asserted, not only its freshness

- **WHEN** the companion test runs
- **THEN** it greps the blessed document for each disclosure the requirements oblige the header to make and
  for the absence of a rendered backslash, so a renderer typo cannot pass by agreeing with itself

#### Scenario: A clean run states the figures it counted

- **WHEN** the reaction reports clean
- **THEN** it prints the bound, capability, pinning-citation, unpinned, and reference counts, each labelled in
  the sense this specification defines, so a figure written into prose comes from a measurement rather than
  from memory

#### Scenario: A tracked document's written census disagrees with the register

- **WHEN** a tracked Markdown document states `N bounds across M capabilities` and either number differs from
  what the reaction counted
- **THEN** the reaction fails, naming the file, the line, the written figures, and the counted ones

#### Scenario: A written census that agrees passes

- **WHEN** a tracked Markdown document's stated figures match what the reaction counted
- **THEN** the reaction passes for that document, so the direction reacts to disagreement rather than to the
  shape

#### Scenario: A census written at the start of a line

- **WHEN** a stale figure is the first thing on its line, as reflowed Markdown produces
- **THEN** the reaction fails, so the figure's column cannot decide whether it is judged

#### Scenario: Two censuses on one line, the earlier one stale

- **WHEN** a line carries two figures and only the later one agrees with the count
- **THEN** the reaction fails, naming the stale one, because a line examined at one figure leaves the rest
  unchecked whichever one that is

#### Scenario: A census in a path the repository does not track

- **WHEN** a stale figure sits in an untracked file, or in a path the repository ignores
- **THEN** the reaction passes for it, because this gate judges tracked content and a local file must not
  decide a developer's run where a clean checkout would pass

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
Done and run verbatim by CI, so `crates/kanhe/tests/dod_coherence.rs` binds the two. Its failure directions SHALL
each be proven by a companion test against fixtures built to trip exactly one condition — a gate over a
coverage claim that has not been observed failing is a restatement of the register, not a defence of it.

The reaction SHALL be read-only: it SHALL NOT edit a spec, declare a bound, or rewrite the projection
except when explicitly asked to regenerate it.

Regeneration SHALL be bound by the same verdict contract as judgment — clean, violation, or cannot-judge.
Regenerating over a register that has offenses SHALL write the projection and then **fail**, because "the
document was rewritten" and "the register it describes is valid" are different claims and one verdict cannot
carry both. A register the reaction cannot judge at all SHALL fail **before** the projection is
written, so a register whose declarations it could not find cannot leave behind a document that reads as a
complete one.

The verdict contract SHALL bind **every** path out of the reaction, including a failure nobody anticipated.
A command that fails without its own handling SHALL surface as a **cannot-judge** naming where it failed, never
as the failing utility's own status, which carries no verdict this contract defines — so a consumer cannot act
on it and an operator is given no reason. Holding this per-command is not equivalent to
holding it structurally — the paths that break the contract are the ones nobody thought to wrap.

The reaction's **package enumeration** SHALL come from tracked content like every other read, and SHALL be
refused rather than judged when it fails: a directory listing that emits some entries and then fails leaves a
short list that reads as authoritative, and every citation in a package the reaction never enumerated is then
reported as one the harness does not register.

An **enumeration of the observation source that fails** SHALL be a cannot-judge, never an empty result.
The reaction reads what it judges through `git ls-files`, and a failed enumeration returns exactly what a
repository holding nothing returns, so the two MUST be told apart by the enumeration's exit status,
checked where the reaction can act on it rather than inside a subshell whose status reaches no one. The
directions this forecloses are not one: an empty census list reports clean over a document it never read,
while an empty tracker or citation list refuses every bound in the register and blames the register for a
`git` failure. A tracked path the worktree does not hold SHALL be refused on the same ground and before
the projection is written, since a tree the reaction could only partly read cannot produce a whole
register.

The repository argument SHALL be resolved to one stable physical root before any scan, directory transition,
or projection access. A relative and an absolute spelling of the same repository SHALL judge and regenerate
the same projection; entering the repository for the tracked-Markdown census SHALL NOT make later paths relative
to that repository a second time.

Before scanning tracked Markdown for a written census, the reaction SHALL enter the judged repository in a
separately checked step. Failure to enter SHALL be a **cannot-judge** and SHALL NOT be interpreted as grep's
ordinary exit-1 no-match result.

#### Scenario: Every failure direction is proven

- **WHEN** the companion test runs
- **THEN** each of the reaction's failure directions is exercised by its own fixture, and the passing
  direction is exercised too, so a gate that only ever refuses is not mistaken for a working one

#### Scenario: The local gate and CI cannot drift apart

- **WHEN** the gate is added to the Definition of Done
- **THEN** the identical command appears in CI, and `crates/kanhe/tests/dod_coherence.rs` fails if it does not

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

#### Scenario: A failed tracked-file enumeration is not an empty one

- **WHEN** `git ls-files` fails while enumerating the tracked files a direction judges — the tracked
  Markdown a written census could sit in, the tracked paths a tracker could name, or the tracked Rust
  files a citation could be defined in — and the repository otherwise holds a stale census
- **THEN** the reaction reports that it cannot judge, naming the enumeration that failed, rather than
  reading the empty result as a repository holding nothing: that reading reports clean over a census it
  never examined, and refuses every tracker and citation in the register for a failure that is not the
  register's

#### Scenario: Relative and absolute repository paths share one projection root

- **WHEN** the gate is invoked from a repository's parent with a relative path to a register carrying tracked Markdown
- **THEN** it judges and regenerates the same projection as an absolute invocation, without looking beneath a second copy of the relative repository path

#### Scenario: The repository disappears before the written-census scan

- **WHEN** tracked Markdown enumeration succeeds and the judged repository cannot then be entered for the census scan
- **THEN** the reaction refuses as a **cannot-judge** naming the directory transition, rather than reporting
  that no census was written

#### Scenario: A tracked spec absent from the worktree is refused before the projection is written

- **WHEN** a spec file `git ls-files` lists is absent from the worktree, with other spec files still
  readable
- **THEN** the reaction reports that it cannot judge, naming the absent spec, and writes no projection —
  a partial tree would otherwise produce a projection describing a partial register while agreeing with
  the verdicts drawn from the same partial read

#### Scenario: An unanticipated failure still reports within the verdict contract

- **WHEN** a command the reaction runs fails with no handling of its own — a text utility reading a spec, a
  temp file that cannot be created
- **THEN** the reaction refuses as a **cannot-judge**, naming where the failure occurred, rather than
  surfacing the failing utility's own status — a refusal the contract does not define is one no consumer can act
  on and no operator can read

#### Scenario: A partial package enumeration is refused, not judged

- **WHEN** the enumeration of the workspace's packages emits some entries and then fails
- **THEN** the reaction reports that it cannot judge rather than building its harness index from the short
  list, which would report every citation in an unenumerated package as one the harness does not register

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

### Requirement: A citation answered twice SHALL fail, whichever answer is repeated

A bound carrying more than one `UNPINNED` bullet SHALL fail, naming the bound, exactly as one carrying both a
`PINNED-BY` and an `UNPINNED` does. Two trackers are two answers to the question a citation exists to answer,
and the declaration holds one tracker, so silently keeping one of them records a bound whose owner is whichever
line happened to be last.

Repeated **`PINNED-BY`** SHALL remain accepted. That asymmetry is deliberate and already stated: several
pinning tests are several defences of one bound, while several trackers are several owners of one gap. A repair
that flattened the two would break a live declaration.

#### Scenario: A bound carries two `UNPINNED` citations

- **WHEN** a bound scenario carries more than one `UNPINNED` bullet
- **THEN** the reaction fails naming the bound id, because a bound that answers the citation question twice
  records nothing

#### Scenario: A bound carries two `PINNED-BY` citations

- **WHEN** a bound scenario carries more than one `PINNED-BY` bullet
- **THEN** the reaction accepts it and retains both, because several tests defending one bound is not two
  answers to one question

### Requirement: The package enumeration SHALL come from tracked content and refuse rather than shorten

The reaction's package enumeration SHALL read tracked content and SHALL refuse when it fails, rather than
producing a short list. A directory listing that emits some entries and then fails leaves a list that reads as
authoritative, and every citation in a package the reaction never enumerated is then reported as one the
harness does not register — a filesystem failure charged to the register it was reading.

This requirement was already written in this capability's prose and held by nothing: the enumeration read the
working directory with `read_dir` and dropped failed entries.

#### Scenario: The tracked package enumeration fails

- **WHEN** the enumeration of tracked package manifests fails
- **THEN** the reaction refuses as a cannot-judge naming the failure, rather than resolving citations against
  a set it could only partly read

#### Scenario: An untracked directory under the crates root

- **WHEN** a directory carrying a manifest exists in the working tree and in no commit
- **THEN** it is not a package this reaction enumerates, because the tracked set is what the repository holds
