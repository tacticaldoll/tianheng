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

A shell gate separated a violation (`1`) from a gate that cannot decide (`2`); a Rust test passes or fails.
A reaction that can reach both outcomes SHALL carry the distinction in its **return type** and its directions
SHALL assert which one a shape produces.

Collapsing the two tells a reader to go looking for a disagreement that does not exist, and a matrix reading
only "it failed" is blind to the inversion: installing a shared backstop once turned a gate's violation into
a cannot-judge, so every genuine incoherence was reported as undecidable with CI green throughout.

Where a reaction cannot usefully distinguish them — a mutation that never applied, an enumeration that could
not be read — it SHALL **fail** rather than pass, and say which it is. Passing is the direction the Core
Contract forbids.

#### Scenario: A reaction reaches both outcomes

- **WHEN** a reaction can both find a disagreement and fail to read its input
- **THEN** its result type names which, and its directions assert the kind rather than merely that it refused

#### Scenario: A reaction cannot decide and has no way to say so

- **WHEN** a Rust reaction meets an input it cannot judge
- **THEN** it fails, naming the input it could not read, because a reaction that reports clean over content it
  never read is the one outcome the Core Contract forbids

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
