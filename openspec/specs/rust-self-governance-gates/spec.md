# rust-self-governance-gates Specification

## Purpose

Hold this repository's reactions **on itself** to the shape that makes them reactions rather than
conventions: each is a Rust integration test under `crates/tianheng/tests/`, each has been seen to fail,
each says what it deliberately does not reach, and none of them is product.

This capability replaces `gate-shape-contract`, which specified the pairing of a `scripts/check_*.sh` gate
with a `scripts/test_*.sh` twin and the exit contract between them. That subject no longer exists —
`git ls-files scripts/` names one unit, `publish.sh`, which is a wrapper rather than a gate — and its
reaction had reached the vacuity its own bounds warned about, enumerating **zero** gates, projecting
`0 gates, 11 properties each`, and reporting clean over all of it.

## Requirements

### Requirement: A self-governance reaction SHALL be a Rust test that has been seen to fail

Every reaction judging this repository SHALL be a `#[test]` under `crates/tianheng/tests/`, and every
refusal it holds SHALL have been run against a tree carrying the shape it refuses, with that failure
recorded in the change that introduced it.

A Rust test's failure mode is asserted **inline** — the expected value sits beside the observation — so a
reaction needs no separate failure matrix to be defended. That is what the twin obligation bought when a gate
was a shell script and its refusal was an exit code, and it is why retiring the pairing loses no coverage.

Where a reaction judges shapes rather than one repository, it SHALL carry those shapes as fixtures it builds,
and each fixture SHALL be built **hermetically**: a fixture inheriting the judged machine's configuration
cannot demonstrate a refusal, because the shape it builds is not the shape it named.

#### Scenario: A reaction whose refusal has never been run

- **WHEN** a reaction is added with a refusal no run has produced
- **THEN** the change does not enter the specs; the refusal is not defended by a test that has only ever
  passed

#### Scenario: A fixture inherits ambient configuration

- **WHEN** a fixture is built without neutralising the machine's own configuration
- **THEN** the shape it builds may differ from the shape it declares — measured: ambient signing
  configuration turned an intentionally unsigned tag into a signed one, and a bare `git tag` demanded a
  message

### Requirement: The three-way contract SHALL survive as a type, not an exit code

A reaction that can reach both outcomes SHALL carry the distinction in its **return type**, its directions
SHALL assert which one a shape produces, and that type SHALL be one type rather than one per reaction.

A shell gate separated a violation (`1`) from a gate that cannot decide (`2`); a Rust test passes or fails,
which is why the distinction has to live somewhere a status code no longer reaches.

Collapsing the two tells a reader to go looking for a disagreement that does not exist, and a matrix reading
only "it failed" is blind to the inversion: installing a shared backstop once turned a gate's violation into
a cannot-judge, so every genuine incoherence was reported as undecidable with CI green throughout.

Where a reaction cannot usefully distinguish them — a mutation that never applied, an enumeration that could
not be read — it SHALL **fail** rather than pass, and say which it is. Passing is the direction the Core
Contract forbids.

That type SHALL be **one** type. Two reactions each defining their own `Kind`, `Refusal` and constructors is
the twin-drift class this family exists to close: the two can disagree about what a cannot-judge is while both
read as holding the same contract.

And the obligation on directions SHALL be held by a reaction rather than by discipline. "Its directions assert
the kind" was specified here and enforced by nothing; a review sweep counted **24 of 60** refusal
construction sites as surviving both perturbations, without being able to say which of those were
undistinguished and which were simply never constructed. A requirement about the shape of directions is
exactly the kind a reading cannot settle, because what a direction asserts is a question about running a
program — and so is the difference between those two answers.

#### Scenario: A reaction reaches both outcomes

- **WHEN** a reaction can both find a disagreement and fail to read its input
- **THEN** its result type names which, and its directions assert the kind rather than merely that it refused

#### Scenario: A reaction cannot decide and has no way to say so

- **WHEN** a Rust reaction meets an input it cannot judge
- **THEN** it fails, naming the input it could not read, because a reaction that reports clean over content it
  never read is the one outcome the Core Contract forbids

#### Scenario: A second gate declares its own refusal vocabulary

- **WHEN** a reaction defines a `Kind` or a refusal constructor of its own rather than using the shared one
- **THEN** it is outside the reaction that holds refusal sites to being distinguished, and the two definitions
  can drift about what each kind means while both read as the same contract

### Requirement: A reaction that runs only on request SHALL be named where the run is decided

A reaction that does not run in an ordinary `cargo test --workspace` SHALL be named **wherever its run is
decided**: on its own line in the `AGENTS.md` Definition of Done and in the CI job that holds it, when the
decision is "run it despite the cost"; or in the one path that asks for it, when the decision is "run it only
where it can answer".

The distinction is not a loophole for the second kind. `scripts/publish.sh` is where a publish-source run is
decided, because no development checkout is a release snapshot and a pre-flight run could only ever refuse. A
reaction gated behind an environment variable named in NEITHER place never runs at all, which is the shape
this requirement exists to refuse.

Two reactions are gated this way — the mutation suite, which checks out a worktree and builds it, and the
examples suite, which builds seven separate crate graphs — and neither may run inside every
`cargo test --workspace`. A third, the publish-source gate, is gated differently and deliberately: no
development checkout is a release snapshot, so it is asked for by `scripts/publish.sh` at the one moment it
can answer, which is where its run is decided and where it is named. A reaction that runs only
when someone remembers is worse than one that costs — the cost is visible and the omission is not.

#### Scenario: A gated reaction absent from the Definition of Done

- **WHEN** a reaction is gated behind an environment variable and named in neither the Definition of Done nor
  CI
- **THEN** it never runs, and the suite reports clean without it — the reads-as-coverage failure, one level up

### Requirement: A generated projection SHALL be generated, and its freshness SHALL be falsifiable

A document this repository generates SHALL be produced from the source it projects, and its freshness check
SHALL compare that production against the file. A check that reads the file and compares it to itself cannot
fail, and under `BLESS` writes the file back to itself.

Every figure such a document states SHALL be **computed**. A count typed into a generated document is the
hand-written census this family refuses: the generator compares its own literal against itself and never
notices the set moving underneath it.

A projection's absence SHALL be a failure, not a skip: a check that returns early when the file is missing
lets the document be deleted with nothing noticing.

#### Scenario: A freshness check that compares a file to itself

- **WHEN** a projection's expected content is read from the projection
- **THEN** the assertion holds by construction and defends nothing — measured on the observation-bound
  register, whose document was consequently generated by nothing and checked against nothing while its own
  header said otherwise

#### Scenario: A projection is deleted

- **WHEN** a generated document is removed from the tree
- **THEN** its freshness reaction fails rather than skipping

### Requirement: A projection is not a reaction and ships in nothing

A generated document SHALL be a derived view: it governs nothing on its own, and no crate ships it. What
governs is the reaction that produces it and the source that reaction reads.

`scripts/` and `docs/` alike ship in **zero** packages, which is what makes them self-governance rather than
product. `CHANGELOG.md`'s `### Self-governance` heading exists for the same reason, and `release-coherence`
holds an adopter-facing entry to naming none of this repository's own machinery.

#### Scenario: A projection consulted as though it governed

- **WHEN** a reader treats a generated document as the authority
- **THEN** they are reading a view; the specification the projection derives from is what a change must
  satisfy, and the projection's freshness reaction is what keeps the two together

### Requirement: A census SHALL be declared by the reaction that produces it

A figure a document states about a set this repository enumerates SHALL be **declared as a census**: the
reaction that enumerates the set names the one sentence the figures are written in and produces them, and one
sweep holds every tracked document to that declaration.

A census phrase SHALL be specific enough to name its own set, and SHALL be matchable — a phrase spanning lines
can never match a line-oriented sweep, and would be declared, enumerable and silent. Figures SHALL be read in
digits **and in words**, because this repository's prose writes counts as words; a matcher reading digits only
left two of the four censuses first declared here inert against the very documents they are for.

**What a census does not reach is declared rather than approximated.** A figure written in a sentence no
census declares is unheld, and a figure about a **past state** is a record: holding it to today's enumeration
would demand that the record change every time the tree does. Widening the match toward prose instead is the
detector `AGENTS.md` records as designed, measured three times and rejected.

#### Scenario: A declared census disagrees with what produces it

- **WHEN** a tracked document writes a declared census's phrase with figures the enumerating reaction does not
  produce
- **THEN** the sweep fails, naming the document, the line, both figures and the subject

#### Scenario: A census that can never match

- **WHEN** a census declares a phrase spanning lines, or one whose longest literal is too short to name its
  set
- **THEN** the sweep fails on the declaration itself, because a census that cannot match reads as covered
  while defending nothing

#### Scenario: A count written in a sentence no census declares — a stated bound

- **WHEN** a document writes a figure about an enumerable set in a phrasing no census names
- **THEN** nothing reacts. The declaration is the coverage; reaching further needs a judgement over prose,
  which is the instrument measured three times and rejected. `AGENTS.md` carries the other half as a rule with
  no reaction: a count of something this repository does not produce is not written
- **PINNED-BY** `a_count_in_an_undeclared_phrasing_is_a_stated_bound`

### Requirement: Every reached refusal site SHALL be distinguished in both its kind and its message

Every reached refusal site SHALL be held to both of its contracts: some direction SHALL fail when the site's
kind is swapped, and some direction SHALL fail when the site's message is replaced.

A **refusal site** is one construction of a kinded refusal, and it carries two independent contracts: the
**kind**, which is what an operator acts on before an irreversible act, and the **message**, which is what
tells them where to look.

Requiring only one of the two lets a site be observed in one contract and rot in the other — a message that
has become a sentence about something else, or a kind that has silently inverted, with the suite green.

The message perturbation SHALL **replace** the message rather than prefix it. A prefix leaves a
`contains(needle)` assertion passing, so it finds neither a direction that asserts only the kind nor
**shadowing**: two sites producing one needle, where no assertion can say which fired.

The perturbation SHALL be applied without rebuilding, by intercepting construction at the site's own caller
location. Mutating source instead would rebuild once per site per perturbation — measured here as two orders
of magnitude more expensive — and would reopen the window in which an interrupted run leaves the tree edited.

The location SHALL be read inside the `#[track_caller]` propagation chain, in the annotated constructor's own
body. Read in an unannotated helper it measures the shared constructor's interior, so every site reports one
location: the reaction would enumerate every site, intercept one, and report clean over all of it. A direction
SHALL demonstrate that two sites on different lines record different locations, because a broken propagation
chain is invisible to reading.

The enumeration SHALL be **total** and SHALL verify its own totality: no source line may carry two refusal
constructions, and no refusal value may be built outside the constructors. Either shape SHALL fail the
reaction rather than silently shrinking the set it enumerates.

The site search SHALL NOT be line-oriented. A construction may be written across lines, and one that no
direction reaches would then be invisible to the static enumeration and to the reach recording at once. Where
a construction can be written so that no search for the constructor's name finds it — an import that renames
it, a constructor taken as a value — that form SHALL **fail** rather than be followed: following a name
through a binding is resolving names, which is the compiler's work and not a scan's, and a scan that tried
would claim a reach it does not have.

The reaction SHALL be gated and named where its run is decided, and the environment that selects a
perturbation SHALL be scrubbed at the point an irreversible act is launched — not guarded by a check inside
the reaction being perturbed, which the perturbation could itself disable.

#### Scenario: A refusal site no direction distinguishes

- **WHEN** a refusal site is reached by the suite and neither swapping its kind nor replacing its message
  makes any direction fail
- **THEN** the reaction fails, naming the site, because that refusal can change kind or message with nothing
  noticing

#### Scenario: Two sites produce one needle

- **WHEN** a direction asserts a refusal by a substring that more than one site produces
- **THEN** replacing the first site's message leaves the direction passing, and the reaction names that site
  as undistinguished — shadowing is the failure a kind-only perturbation cannot see

#### Scenario: The caller location is read outside the propagation chain

- **WHEN** the location is read in a helper that does not itself carry `#[track_caller]`
- **THEN** every site records the shared constructor's own line, and the direction requiring two sites to
  record different locations fails

#### Scenario: The injection is not reached at all

- **WHEN** the reaction runs and no perturbation reaches any site
- **THEN** every per-site verdict is vacuous; a control poisoning every site at once SHALL fail and a control
  naming no site SHALL pass, or the verdicts below them say nothing

#### Scenario: A refusal value is built without a constructor

- **WHEN** a refusal is built as a struct literal, or two constructions share a source line
- **THEN** the reaction fails on its own enumeration, because a site it cannot name is a site it cannot
  perturb

#### Scenario: A construction is written across lines

- **WHEN** a refusal is constructed with its name and its arguments on different lines
- **THEN** it is still enumerated; a line-oriented search would miss it, and a wrapped construction no
  direction reaches is missed by the reach recording too — both halves blind at once

#### Scenario: A constructor is renamed or taken as a value

- **WHEN** a constructor is imported under another name, or bound to a value and called through that binding
- **THEN** the reaction fails on that form rather than following it, because resolving a name through a
  binding is the compiler's work and a scan that attempted it would claim a reach it does not have

### Requirement: A perturbation selector SHALL NOT be an exemption identity

A site's source location SHALL be used for nothing that outlives one run. It is the selector naming which site
to perturb, and it is valid only for the build that produced it: inserting a line above a site moves it.

An exemption in particular SHALL NOT be keyed on a location, nor on the message text — messages are
operator-facing prose, so rewording one would silently move an exemption. A site that genuinely cannot be
constructed SHALL declare itself **at the site**, through a named constructor form carrying a stable slug, so
that the identity moves with the site and is checked by the compiler rather than by adjacency.

The join between a slug and the bound covering it SHALL have a data model, and that model SHALL be
repository-local. `Extent` is a **shipped public type**, reached through a public `observation_bounds()`;
widening it to carry exemption membership would change a product API to serve a test. The join SHALL therefore
live in test support, as a table joined in both directions against sets that are produced: the site
enumeration, and the live bound set.

The slug SHALL be injective, and every edge of that three-way join SHALL be required in both directions: a
slug carried by two sites, a slug no registry entry covers, a registry entry naming a slug no site carries,
and a registry entry naming a `BoundId` the live bound set does not contain SHALL each fail. The membership of
the exempt set SHALL be produced by the reaction and its size declared as a census, so it cannot grow silently.

The registry-to-bound edge SHALL be a **biconditional**: a non-empty registry requires the exemption-class
bound to be declared, **and** the bound being declared requires the registry to be non-empty. One direction
alone lets the last exemption disappear while the bound survives as permanent residue — a declared false
negative about a set with no members, which reads as a limit the reaction still has.

A declared exemption SHALL be **re-run rather than trusted**: a site declared out of reach that the suite is
observed to reach SHALL fail as a stale exemption. Removing the exemption is then the retirement, and it is
earned by the observation rather than by the declaration still reading plausibly.

#### Scenario: A line is inserted above a refusal site

- **WHEN** source above a refusal site changes and every site below it moves
- **THEN** nothing that outlives the run is invalidated, because the location named only which site to perturb
  in that run

#### Scenario: An exemption's site becomes reachable

- **WHEN** a site declared out of reach is observed being constructed by some direction
- **THEN** the reaction fails; the exemption is stale, and its retirement is decided by re-running the
  observation it claimed was impossible

#### Scenario: Two sites carry one exemption slug

- **WHEN** the same slug is written at more than one site
- **THEN** the reaction fails, because an exemption that names a set rather than a site excuses whichever
  member happened to be looked at

#### Scenario: An exemption entry names a bound that does not exist

- **WHEN** the exemption registry names a `BoundId` the live bound set does not contain, or names a slug no
  site carries
- **THEN** the reaction fails; a join whose edges are checked in one direction only lets an exemption survive
  the retirement of the bound that justified it

#### Scenario: The last exemption is removed

- **WHEN** every exemption is retired and the registry becomes empty while the exemption-class bound is still
  declared
- **THEN** the reaction fails, because a declared false negative over a set with no members reads as a limit
  the reaction still has

### Requirement: The enumerated corpus SHALL be what compiles, and no refusal vocabulary SHALL sit outside it

The site enumeration SHALL be taken from **the source files the compiler reports having read** for the targets
being perturbed, not from a reimplementation of module resolution. Resolving `#[path]` by text misses
conventional `mod` declarations, `include!`, and `#[cfg_attr(…, path = …)]`, and it admits files the build
excluded by `cfg` — four errors in one, in a repository that has already shipped a false negative from
mimicking the compiler's resolution by reasoning instead of measuring against a real build.

The set of targets SHALL likewise be enumerated from the build rather than listed. A hard-coded root list is
the shape in which an entire new target falls outside the reaction.

A file the compiler reports having read SHALL be tracked, and an untracked participant SHALL fail. This is the
complement of judging tracked content rather than a departure from it: a reaction over *shipped* content
judges what is tracked, and a reaction correlating text with a **run** judges what ran, then requires that it
be reviewable.

Every file in that corpus SHALL be scanned for the **exact** refusal vocabulary the shared module defines —
its refusal type's name, its cannot-judge variant, its constructor names — appearing outside the shared
module, and any such definition SHALL fail. The claim SHALL be no wider than the scan: recognising a
vocabulary by intent rather than by name is a judgement over source, and a reaction that claimed to catch any
future kinded gate would be claiming coverage it does not have.

A site's observing targets SHALL be derived from which targets' reported sources contain that site's **file**,
not from which targets include the shared module.

The corpus is the source list of one feature set, and it SHALL be the feature set the perturbation runs — so
enumeration and run are about the same binaries by construction. This is a definition rather than a bound: a
file outside that build is also outside the binaries being perturbed, so no refusal it holds is one the suite
could have reached.

#### Scenario: A refusal site is added but not committed

- **WHEN** a new refusal site exists in the worktree, is not committed, and no direction reaches it
- **THEN** the enumeration still names it and the reaction fails; enumerating from a committed revision would
  leave it invisible to the static enumeration and to the reach recording at once, which is a false clean

#### Scenario: A judged target compiles an untracked file

- **WHEN** a file the compiler reports having read is not named by the repository's tracked set
- **THEN** the reaction fails, because a judged target is compiling content no review can see

#### Scenario: A site is included by a form module-resolution text cannot follow

- **WHEN** a refusal site reaches a target through a conventional `mod` declaration, an `include!`, or a
  conditional path attribute
- **THEN** it is still enumerated, because the corpus is what the compiler reported reading rather than what a
  second implementation of its resolution predicted

#### Scenario: A third reaction re-declares the shared refusal vocabulary

- **WHEN** the refusal type's name, its cannot-judge variant, or a constructor of that name is defined
  anywhere in the corpus other than the shared module
- **THEN** the reaction fails, naming the file

#### Scenario: A refusal vocabulary under different names is not observed — a stated bound

- **WHEN** a reaction declares the same contract under other names — a `Decision` type with `Disagrees` and
  `Unreadable`, say — and never touches the shared module; or declares the shared names inside the sources of
  the reaction performing the scan, which are exempt because a reaction over text holds the text it recognises
- **THEN** nothing reacts. The scan recognises names, and recognising a vocabulary by intent is a judgement
  over source, the instrument this repository has measured and rejected. No compile-time construction reaches
  it either: forcing a not-yet-written reaction to return the shared type requires enumerating reactions, and
  what counts as one has no mechanical definition
- **PINNED-BY** `a_refusal_vocabulary_under_other_names_is_not_observed`


### Requirement: The reach recording SHALL fail loudly, and each guard SHALL be falsified by its own defect

The record of which sites a run reaches SHALL be written and parsed strictly: a failed write or an unparseable
line SHALL fail the reaction rather than being skipped. A lost record is not self-announcing — for a site that
is a declared exemption it lands in the legal *declared and unreached* class and the run reports clean, so
recording integrity is load-bearing for a whole verdict class rather than for a count.

Each guard defending this reaction SHALL be run against **the defect it guards against**, individually. One
blanket perturbation that only some guards notice reports the rest as exercised while nothing tested them —
the reads-as-coverage failure this reaction exists to end, occurring inside it. In particular, disabling the
injection kills the guard that the injection is wired and leaves untouched the guard that it does not fire
where it was not aimed, since a poison that never fires satisfies that one vacuously.

#### Scenario: A reach record is lost

- **WHEN** a construction is not recorded, for a site that declares itself out of reach
- **THEN** the site appears legally unreached and the run is green — so the write and the parse fail loudly
  instead, and concurrent writers are serialised rather than trusted

#### Scenario: One perturbation is used to defend every guard

- **WHEN** the guards are exercised by disabling the injection alone
- **THEN** the guard requiring a nonexistent selector to stay green passes vacuously, and the classifier's
  discrimination is untested; each guard is run against its own defect instead

### Requirement: A refusal site the suite never reaches SHALL be red unless declared

A refusal site the suite never reaches SHALL fail unless it declares itself out of reach.

A site no direction constructs is not defended by anything, and it is a strictly worse fact than a site
reached but undistinguished: there is no evidence the refusal is even attainable.

Closing one SHALL be attempted in order: **construct** it through a fixture; where a preceding check makes the
branch logically unreachable, **delete** it, since a dead refusal is a smaller artefact than a bound declaring
a false negative about an impossible input; and only what survives both SHALL be declared out of reach.
Declaring first produces exemptions for code that should not exist, and the exempt set then reads as a limit
of the reaction rather than as a property of the world.

The size of that set SHALL be a declared census, produced by the same enumeration the reaction perturbs, so it
cannot grow without a document disagreeing with it. Today **6 of 58 refusal sites are declared out of reach**,
every one of them inside the tag-signature check: four describe the machine the gate runs on rather than the
repository it judges, and two are git's own two extractions of one object disagreeing with each other.

This closes the category a coverage report would otherwise absorb. A reaction that perturbed only the sites it
observed being reached, and reported the rest as "not exercised", would read as coverage of the whole
enumeration while saying nothing about the part most likely to be wrong.

#### Scenario: A refusal site nothing constructs

- **WHEN** the suite runs and a site in the enumeration is never reached
- **THEN** the reaction fails unless that site declares itself out of reach with a slug the declared bound
  covers

#### Scenario: Whether a declared out of reach refusal is genuinely unconstructible is not observed — a stated bound

- **WHEN** a site declares itself out of reach because its precondition cannot be produced in any environment
  the suite runs in — an absent `ssh-keygen`, a signature mechanism failing its own round trip
- **THEN** the reaction observes only that no direction reaches it, never that no direction *could*. Reaching
  further would require constructing the environment the declaration says is unconstructible, so the
  declaration carries a reason the reaction cannot check, and the membership of the exempt set is produced and
  counted rather than approximated
- **PINNED-BY** `a_site_declared_out_of_reach_is_only_observed_to_be_unreached`
