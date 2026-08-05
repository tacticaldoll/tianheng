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
into a common section. Nearly every bound declared today sits under the requirement it qualifies rather
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
  every bound already declared that way without relocating any of them

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
reaction SHALL exit **cannot judge** rather than fall back silently, because a citation's test-ness is then
undecided rather than decided weakly.

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
- **THEN** the reaction exits cannot-judge, because test-ness is undecided rather than weakly decided

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

### Requirement: A bound stated in prose but not declared as a scenario SHALL fail

The reaction SHALL scan `openspec/specs/*` for bound-declaring prose and SHALL fail on any occurrence
outside a declared bound scenario, **subject to the exemptions and residuals stated below, which SHALL be
enumerated rather than implied**. This makes the prose already present the register's mandatory minimum, so
the register cannot be completed by declaring only the convenient bounds. Its size is measured rather than
estimated: 3 of 30 specs carry an Observation-bounds requirement today while 11 more state bound prose
without one.

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
3. A `(bound: …)` reference clears the prose it sits with **regardless of how many bounds that prose
   states**, and regardless of whether the referenced bound is one of them.

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

An **enumeration of the observation source that fails** SHALL be a cannot-judge, never an empty result.
The reaction reads what it judges through `git ls-files`, and a failed enumeration returns exactly what a
repository holding nothing returns, so the two MUST be told apart by the enumeration's exit status,
checked where the reaction can act on it rather than inside a subshell whose status reaches no one. The
directions this forecloses are not one: an empty census list reports clean over a document it never read,
while an empty tracker or citation list refuses every bound in the register and blames the register for a
`git` failure. A tracked path the worktree does not hold SHALL be refused on the same ground and before
the projection is written, since a tree the reaction could only partly read cannot produce a whole
register.

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

#### Scenario: A failed tracked-file enumeration is not an empty one

- **WHEN** `git ls-files` fails while enumerating the tracked files a direction judges — the tracked
  Markdown a written census could sit in, the tracked paths a tracker could name, or the tracked Rust
  files a citation could be defined in — and the repository otherwise holds a stale census
- **THEN** the reaction reports that it cannot judge, naming the enumeration that failed, rather than
  reading the empty result as a repository holding nothing: that reading reports clean over a census it
  never examined, and refuses every tracker and citation in the register for a failure that is not the
  register's

#### Scenario: A tracked spec absent from the worktree is refused before the projection is written

- **WHEN** a spec file `git ls-files` lists is absent from the worktree, with other spec files still
  readable
- **THEN** the reaction reports that it cannot judge, naming the absent spec, and writes no projection —
  a partial tree would otherwise produce a projection describing a partial register while agreeing with
  the verdicts drawn from the same partial read

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
