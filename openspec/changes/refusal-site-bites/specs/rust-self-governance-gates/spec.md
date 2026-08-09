## MODIFIED Requirements

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

## ADDED Requirements

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
