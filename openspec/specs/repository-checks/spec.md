# repository-checks Specification

## Purpose

Hold this repository's checks **on itself** to the shape that makes them checks rather than conventions: each
is a Rust integration test in unpublished Kanhe or Shengmo, each has been seen to fail, each says what it
deliberately does not reach, and none of them is product.

This capability replaces `gate-shape-contract`, which specified the pairing of a `scripts/check_*.sh` gate
with a `scripts/test_*.sh` twin and the exit contract between them. That subject no longer exists —
`git ls-files scripts/` names only wrappers, no gate — and its
check had reached the vacuity its own bounds warned about, enumerating **zero** gates, projecting
`0 gates, 11 properties each`, and reporting clean over all of it.

## Subject

- `crates/shengmo/**/*.rs`
- `crates/kanhe/**/*.rs`
- `scripts/*.sh`
- `AGENTS.md`
- `.github/workflows/ci.yml`

## Requirements

### Requirement: Repository governance vocabulary SHALL preserve product ownership

Live project prose and self-descriptive source comments SHALL use **product** only for crates whose manifests
permit publication. **Reaction** SHALL name observable boundary behavior implemented by those product crates:
observation, structured outcome or report, process exit class, or runtime event.

An unpublished Rust judgement or test over this repository SHALL be called a **repository check** or **gate**.
A Shengmo gate MAY invoke product reactions as dogfood, but the gate itself SHALL NOT be described as a separate
reaction. Shell scripts and CI SHALL be described as **workflow orchestration** that invokes gates or
irreversible commands and SHALL NOT be assigned a verdict of their own.

#### Scenario: A Kanhe test judges repository records

- **WHEN** prose describes a Kanhe integration test comparing this repository's documents, code, or workflow
  registration
- **THEN** it calls that executable a repository check or gate, not a product reaction

#### Scenario: Shengmo runs the delivered product against this workspace

- **WHEN** a Shengmo test invokes Tianheng's published observation and outcome path against this repository
- **THEN** prose calls the test a dogfood gate and calls the behavior it invokes the product reaction

#### Scenario: A shell wrapper invokes a Rust judgement

- **WHEN** a shell script sequences a Kanhe gate before merge or publish
- **THEN** prose assigns the judgement and verdict to the Rust gate and calls the shell wrapper workflow
  orchestration

#### Scenario: Product specifications describe boundary behavior

- **WHEN** a publishable crate observes a governed shape and produces an outcome, report, exit class, or runtime
  event
- **THEN** its specification retains reaction vocabulary

### Requirement: A self-governance check SHALL be a Rust test that has been seen to fail

Every check judging this repository SHALL be a `#[test]` living **outside every published package**, and
every refusal it holds SHALL have been run against a tree carrying the shape it refuses, with that failure
recorded in the change that introduced it.

**A refusal about the reading failing is not a refusal about the subject.** Where a refusal can be reached
only by breaking the tool that reads — a process that will not run, a directory that will not enumerate,
output that is not the format its producer emits — a direction over it must simulate that tool, and a
fixture that simulates a tool tests the simulation. Such a site SHALL be declared unheld rather than given a
fabricated fixture, because a fixture that passes for the wrong reason is a false green and a false green is
worse than a declared gap. The distinction is not difficulty: every site so declared is a **cannot-judge**,
which the compiler established before it was claimed.

**The second clause is held by a register rather than by attention.** A refusal SHALL carry the identity of
the branch that produced it, and a direction observing that branch SHALL name the same identity, so the two
are compared by running. Identity in the message alone could not be measured: a message is a template and a
direction asserts a rendering of it, and five textual predicates written against that gap were each wrong in
a different direction. The corpus is this repository's own check crate; sites not yet carrying an identity
are counted in a produced projection that falls to zero.

Shipping in zero packages is what this capability already gives as the criterion separating governance from
product — the reason `scripts/` and `docs/` count as governance. Measured before this change, the checks
themselves failed it: `cargo package --list -p tianheng` carried all 50 files under `tests/`, so every
check judging this repository's changelog, specs, scripts and documents reached every adopter, where it
could only detect no workspace and return.

Outside every published package is a floor, not the whole answer: it says where a check must **not** live and
nothing about where it belongs. Checks SHALL therefore be held apart by **what they judge** — the law this
repository declares over itself and the dogfood gates that run the delivered product's reactions against this
workspace in one member, the checks that collate its record against itself in another. Measured when only the
floor was applied: 13 of 17 targets in a member whose stated identity was the law judged neither a product
contract nor an architecture, which is the dilution the move set out to end.

The location is not cosmetic. A repository's own law living under a published package's `tests/` lends its
name to everything beside it, and a governance document came to state that twenty checks reaching no
shipped API "run Tianheng's product reactions against the workspace". Position is what makes the two populations
separable at all.

A Rust test's failure mode is asserted **inline** — the expected value sits beside the observation — so a
check needs no separate failure matrix to be defended. That is what the twin obligation bought when a gate
was a shell script and its refusal was an exit code, and it is why retiring the pairing loses no coverage.

#### Scenario: A refusal site is registered and no direction observes it

- **WHEN** a refusal is constructed through the registered form and no direction names its identity
- **THEN** the register refuses. Registering a site is the commitment that a direction observes it, which is
  what keeps the migration from outrunning the coverage it exists to measure

#### Scenario: A direction cites a site no refusal produces

- **WHEN** a direction names a refusal identity that no site constructs
- **THEN** the register refuses. Both directions are held, because either alone is satisfiable by doing
  nothing: a register nobody cites passes a one-way check, and so does a citation of a site that has since
  moved

#### Scenario: Two refusal sites share one identity

- **WHEN** two branches are registered under the same identity
- **THEN** the register refuses, because one direction's citation would then vouch for a branch it never
  reached — the same non-injective identity this repository has already recorded once, where a finding not
  qualified by its owner let a baseline mask a new violation

#### Scenario: No refusal site is untriaged

- **WHEN** a refusal is constructed by anything that does not carry a site identity
- **THEN** the register refuses. The count reached zero and the site-less constructors were deleted, so this
  is held by the compiler and reported as zero on every clean run; the reaction remains because a
  constructor re-introduced is the shape it exists to see

#### Scenario: A construction shape the register's reader does not model — a stated bound

- **WHEN** a registered or unregistered constructor is referenced by a bare name rather than called directly
  — a binding taken by value and called through the alias, or a reference to the name that a local binding
  of the same spelling has shadowed
- **THEN** the reference is read as a construction, whichever it actually names. **This bound used to be
  wider.** The register's reader was text over Rust and not exhaustive over the language: a byte char
  literal, a raw string, or a closure whose parameter list spanned two lines could desynchronise a
  character-by-character scan entirely, producing a site the reader neither parsed nor counted as
  unparseable — invisible to both of its readings at once, which was the unsafe direction this bound named,
  since a missed citation fails loud while a missed construction reports clean over a site nothing holds.
  Reading this repository's own Rust with a real parser instead of scanning it closes that floor: every
  syntactically valid construction is seen by construction, not by an arm added the day a shape was found
  wrong. **What remains is not lexical.** Whether a bare reference names the constructor taken by value or a
  local variable that happens to share its spelling is not written down anywhere a parse tree carries —
  answering it needs name resolution, which a reader of syntax alone does not have
- **UNPINNED** `BACKLOG.md` — *a bare reference to a registered constructor's name cannot be told from a local variable sharing its spelling without name resolution*

#### Scenario: A refusal constructed outside the register's corpus is not triaged — a stated bound

- **WHEN** a refusal is constructed by a gate implemented under `crates/kanhe/tests`, where the judgement
  and the directions over it share a file
- **THEN** nothing triages it. The register reads `crates/kanhe/src`, and a construction there is either
  held by a direction or declared unheld; a construction beside its own directions is neither, because
  *which direction observes this branch* has no answer when every direction in the file can see it. Reaching
  further means deciding what a file that is both judgement and test is being asked, which is a question
  about where those gates should live rather than about this register
- **UNPINNED** `BACKLOG.md` — *a gate that is its own test is outside the refusal register*

#### Scenario: A site no direction holds is declared, not left

- **WHEN** a refusal site is registered and no direction observes it
- **THEN** it SHALL be declared unheld — with why, an owner and a tracker — or the register refuses. There
  is no third state: a site is held or declared. The declaration is the escape hatch and is deliberately
  expensive, because an escape hatch nothing forces you through is the prose that drifted

#### Scenario: An input the wrapper never supplied is not a message that disagrees

- **WHEN** the merge gate's harness is invoked with a subject but without one of the other judged inputs
- **THEN** it refuses as a cannot-judge naming the input. A merge is being made once the subject is there,
  so a missing input is the wrapper supplying an incomplete set. Read with a default, absence arrived as
  emptiness, and the gate answers emptiness on its own terms — an empty **body** is a violation — so an
  input never supplied was reported at the exit class reserved for a gate that ran and disagreed. An empty
  value that *was* supplied keeps its own meaning

#### Scenario: The constructors are the only way to build a refusal

- **WHEN** a refusal is built as a struct literal rather than through a constructor
- **THEN** it does not compile. The register counts calls, so a literal would produce a registered site that
  is unheld by any direction, undeclared, and unreported, while the projection said no other construction
  exists. The field the register is about is private, which makes the compiler refuse the shape rather than
  a reader detect it

#### Scenario: A registered construction this reader cannot parse is not counted as absent

- **WHEN** a registered refusal is constructed in a shape this register's reader does not parse — the
  constructor taken by name and called through the binding, or a site arriving as a parameter
- **THEN** the register refuses for that module. Each shape was invisible to **both** of its readings: no
  parsed site, and not counted as untriaged either, because the untriaged count reads the site-less
  constructors. A real refusal site was then neither held, nor declared, nor reported missing. The parse is
  counted against the calls, which turns *did not see it* into *cannot answer for this module*. A site
  written as a raw string literal no longer belongs to this list: the register's reader parses this
  repository's own Rust with a real parser, and a raw string decodes exactly like a plain one — there is no
  special case left to write for it

#### Scenario: A violation may not be declared unheld

- **WHEN** a refusal that refuses as a **violation** is declared unheld
- **THEN** the register refuses. The declaration exists because a refusal about the *reading* failing can
  only be reached by breaking the machine, and its fixture would test that break. A refusal about the
  **subject** has no such excuse: its fixture is the defect it names, and a shape that cannot be built is
  one the branch is not about. Without this the declaration is available to any branch whose fixture is
  merely inconvenient, which is the half of the escape hatch a table cannot close by describing itself

#### Scenario: A declaration names a site, and a declared site is not observed

- **WHEN** a declaration names a refusal no site produces, or names one a direction does observe
- **THEN** the register refuses. A declaration about nothing is prose about nothing, one level up from the
  drift this register ends; and a declared site a direction observes is **held**, so the declaration
  understates what the repository has

#### Scenario: A check inside a published package

- **WHEN** a check judging this repository lives under a package that `cargo publish` would ship
- **THEN** it reaches adopters who cannot run it, and it is filed as governance while its location makes it
  product — the two answers this criterion exists to keep from disagreeing

#### Scenario: The packaged self-test's subject

- **WHEN** the packaged crate's tests are run from its tarball
- **THEN** what runs exercises the packaged code, rather than governance checks detecting no workspace and
  returning — a skip proves a skip is real, and a tarball of mostly skips proves little else

### Requirement: The three-way contract SHALL survive as a type, not an exit code

A repository judgement that can reach both outcomes SHALL carry the distinction in one shared Rust return
type. Focused behavior matrices SHALL assert the result kind and actionable message for the externally
meaningful shapes they exercise.

A shell gate separated a violation (`1`) from a gate that cannot decide (`2`); a Rust test passes or fails,
which is why the distinction has to live somewhere a status code no longer reaches.

Collapsing the two tells a reader to go looking for a disagreement that does not exist, and a matrix reading
only "it failed" is blind to the inversion: installing a shared backstop once turned a gate's violation into
a cannot-judge, so every genuine incoherence was reported as undecidable with CI green throughout.

Where a judgement cannot read enough input to decide, it SHALL **fail** rather than pass, and say which input
was unverifiable. Passing is the direction the Core Contract forbids.

That type SHALL be **one** type. Two judgements each defining their own `Kind`, `Refusal` and constructors is
the twin-drift class this family exists to close: the two can disagree about what a cannot-judge is while both
read as holding the same contract.

The shared value and constructors SHALL remain ordinary repository-check code. They SHALL NOT carry runtime
mutation, reach recording, caller-location identity, or an exemption protocol; those make internal
construction sites into governance identities instead of holding the behavior an operator can observe.

#### Scenario: A repository judgement reaches both outcomes

- **WHEN** a repository judgement can both find a disagreement and fail to read its input
- **THEN** its result type names which, and its directions assert the kind rather than merely that it refused

#### Scenario: A repository judgement cannot decide

- **WHEN** a repository judgement meets an input it cannot judge
- **THEN** it fails, naming the input it could not read, because a judgement that reports clean over content it
  never read is the one outcome the Core Contract forbids

### Requirement: A repository check that runs only on request SHALL be named where the run is decided

A repository check that does not run in an ordinary `cargo test --workspace` SHALL be named **wherever its run is
decided**: on its own line in the `AGENTS.md` Definition of Done and in the CI job that holds it, when the
decision is "run it despite the cost"; or in the one path that asks for it, when the decision is "run it only
where it can answer".

The distinction is not a loophole for the second kind. `scripts/publish.sh` is where a publish-source run is
decided, because no development checkout is a release snapshot and a pre-flight run could only ever refuse. A
check gated behind an environment variable named in NEITHER place never runs at all, which is the shape
this requirement exists to refuse.

The pin-bites matrix and examples suite are gated by cost and named on their own command lines; neither may run
inside every `cargo test --workspace`. The publish-source gate is gated differently and deliberately: no
development checkout is a release snapshot, so it is asked for by `scripts/publish.sh` at the one moment it
can answer, which is where its run is decided and where it is named. A check that runs only
when someone remembers is worse than one that costs — the cost is visible and the omission is not.

#### Scenario: A retired env-gated check remains in one command surface

- **WHEN** an env-gated repository check is deleted but its command remains in the Definition of Done or CI
- **THEN** command coherence fails or the stale invocation fails, so both surfaces are retired together

### Requirement: Definition-of-Done coherence SHALL compare effective CI commands

Every command in AGENTS.md's Definition of Done SHALL have an effective counterpart in CI. Commands expressed
by `run:` SHALL be compared directly after the existing normalization. The repository's
`EmbarkStudios/cargo-deny-action` step SHALL contribute `cargo deny <command>` from its declared `with.command`
value. A DoD command SHALL NOT be exempted merely because CI normally expresses it through an action.

The action projection is intentionally limited to the cargo-deny action whose command semantics this repository
uses; the check SHALL NOT claim to interpret arbitrary GitHub Actions.

#### Scenario: Cargo deny is supplied by its action

- **WHEN** the DoD contains `cargo deny check` and CI contains an `EmbarkStudios/cargo-deny-action` step whose
  `with.command` is `check`
- **THEN** the coherence check recognizes the effective command and does not report it missing

#### Scenario: The supply-chain step is absent or misconfigured

- **WHEN** the DoD contains `cargo deny check` and CI omits the cargo-deny action or gives it a different or
  absent command
- **THEN** the coherence check fails and names `cargo deny check` as missing from CI

### Requirement: A hand-maintained pin SHALL carry the window it is good for

A pin this repository maintains by hand SHALL declare the window it is good for, and a reaction SHALL hold
that declaration. A pin nobody refreshes rots, and pinning the interpreter that executes a digest-pinned
dependency tree traded a repointable major for exactly that.

**The two halves of the rot are not equal and SHALL NOT be answered together.** Falling behind *within* a
major is bounded by the declaration around the pin — `engines.node` with both ends and `engine-strict=true`,
so npm stops rather than warns — and what remains is bounded risk on a tree resolved by digest. Running the
interpreter **past the point its major is maintained** is the half with teeth, and nothing reacted to it: the
only thing that would notice was someone remembering.

The declaration SHALL sit beside the pin it bounds.
The declaration SHALL name the **major it speaks for** as well as the date, so a pin moved without it refuses
rather than inheriting a window chosen for something else. The reaction SHALL refuse when no window is
declared, when more than one is — a reader that takes one leaves the others binding nothing — when the major
declared is not the major pinned, and when the date has been reached.

**The declaration SHALL be a commitment of this repository rather than an assertion about the tool.** Nothing
here can hold a vendor's release calendar: it is not in this tree, and every reaction runs offline. A claim
about the world needs something holding it, and this one would have nothing; a claim about what this tree will
do needs only the file it is written in. The date is *chosen* with the vendor's schedule in view, and that
choice is the one unheld thing left — which is why the declaration is a decision this repository owns rather
than a fact it claims about the vendor.

**The reading SHALL be a pure function of the workflow text and the day.** A bound whose only demonstration is
the calendar reaching it is a bound nobody has seen work, and this one is written to sit dormant for years.
Every direction it refuses in SHALL be constructed against a supplied day, and the day SHALL be taken in UTC,
so the date a reader is refused on is the same date everywhere.

#### Scenario: The pinned interpreter is inside its declared window

- **WHEN** the workflow declares one window naming the major it pins, and the date it names is ahead of today
- **THEN** the reaction passes
- **PINNED-BY** `the_pinned_interpreter_is_within_its_declared_support_window`

#### Scenario: The declared window has been reached

- **WHEN** the date the workflow declares is today or earlier
- **THEN** the reaction refuses, and names the three places that move together — the workflow pin, the
  package manifest's engine range, and the declaration itself
- **PINNED-BY** `the_window_reader_decides_every_shape_of_the_declaration`

#### Scenario: The pin moves and the window does not

- **WHEN** the major the workflow pins is not the major the declaration speaks for
- **THEN** the reaction refuses, rather than reading a window chosen for a major this workflow no longer runs
- **PINNED-BY** `the_window_reader_decides_every_shape_of_the_declaration`

#### Scenario: The window is absent, doubled, or unreadable

- **WHEN** the workflow declares no window, declares more than one, or declares one whose fields are not a
  major and a `YYYY-MM-DD` date
- **THEN** the reaction refuses and says what to write, rather than passing over a declaration it could not
  read
- **PINNED-BY** `the_window_reader_decides_every_shape_of_the_declaration`

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
- **THEN** its freshness check fails rather than skipping

### Requirement: A projection is not a check and ships in nothing

A generated document SHALL be a derived view: it governs nothing on its own, and no crate ships it. What
governs is the check that produces it and the source that check reads.

`scripts/` and `docs/` alike ship in **zero** packages, which is what makes them self-governance rather than
product. `CHANGELOG.md`'s `### Self-governance` heading exists for the same reason, and `release-coherence`
holds an adopter-facing entry to naming none of this repository's own machinery.

#### Scenario: A projection consulted as though it governed

- **WHEN** a reader treats a generated document as the authority
- **THEN** they are reading a view; the specification the projection derives from is what a change must
  satisfy, and the projection's freshness check is what keeps the two together

### Requirement: A census SHALL be declared by the check that produces it

A figure a document states about a set this repository enumerates SHALL be **declared as a census**: the
check that enumerates the set names the one sentence the figures are written in and produces them, and one
sweep holds every tracked **Markdown** document to that declaration.

A census phrase SHALL be specific enough to name its own set, and SHALL be matchable — a phrase spanning lines
can never match a line-oriented sweep, and would be declared, enumerable and silent. Figures SHALL be read in
digits **and in words**, because this repository's prose writes counts as words; a matcher reading digits only
left two of the four censuses first declared here inert against the very documents they are for.

**What a census does not reach is declared rather than approximated.** A figure written in a sentence no
census declares is unheld, and a figure about a **past state** is a record: holding it to today's enumeration
would demand that the record change every time the tree does. Widening the match toward prose instead is the
detector `AGENTS.md` records as designed, measured three times and rejected.

#### Scenario: A declared census disagrees with what produces it

- **WHEN** a tracked Markdown document writes a declared census's phrase with figures the enumerating check does
  not produce
- **THEN** the sweep fails, naming the document, the line, both figures and the subject

#### Scenario: A census that can never match

- **WHEN** a census declares a phrase spanning lines, or one whose longest literal is too short to name its
  set
- **THEN** the sweep fails on the declaration itself, because a census that cannot match reads as covered
  while defending nothing

#### Scenario: A count written in a sentence no census declares — a stated bound

- **WHEN** a document writes a figure about an enumerable set in a phrasing no census names
- **THEN** no repository check fires. The declaration is the coverage; reaching further needs a judgement over prose,
  which is the instrument measured three times and rejected. `AGENTS.md` carries the other half as a rule with
  no check: a count of something this repository does not produce is not written
- **PINNED-BY** `a_count_in_an_undeclared_phrasing_is_a_stated_bound`

#### Scenario: A figure written in words at one hundred or above is not matched — a stated bound

- **WHEN** a tracked Markdown document writes a declared census's phrase with a figure spelled in words at one
  hundred or above
- **THEN** the sweep does not match it, a stated bound: the word reader covers the units, the tens, and one
  compound of the two, which stops at ninety-nine. Extending it upward buys nothing measurable — the figures
  this repository writes in words are the small ones, and a set large enough to need three-digit words is one
  whose prose writes digits. The residual is stated rather than closed because a word reader that silently
  stops matching reads as covered, which is the failure this requirement's own sweep exists to refuse
- **PINNED-BY** `a_word_form_at_one_hundred_or_above_is_a_stated_bound`

#### Scenario: A census written outside Markdown is not observed — a stated bound

- **WHEN** a tracked file that is not Markdown — a Rust doc comment, a shell comment, a manifest — writes a
  declared census's phrase with the wrong figures
- **THEN** the sweep does not see it, a stated bound: the corpus is tracked Markdown, and widening it was
  measured rather than reasoned about. This repository's Rust sources carry census phrases **as fixture input**,
  where the figures are a parser's expected output and deliberately arbitrary; admitting them would report a test
  asserting its own parser as a drifted document. The narrow corpus is what keeps the sweep's every report
  actionable, and the residual is a figure in a code comment, which `AGENTS.md` measured and left to the reviewer
- **PINNED-BY** `a_census_outside_markdown_is_a_stated_bound`

### Requirement: A squash message SHALL be judged before the merge that records it

A proposed squash subject and body SHALL be judged by a check before `gh pr merge` runs, and the sanctioned
path to that merge SHALL be a wrapper that cannot be reached without the judgement.

The subject SHALL equal the pull request's title exactly, and SHALL NOT carry a trailing `(#N)`. The rule is
already written; what is new is that something holds it. Measured **when this requirement was written**, nine
subjects carried that serial, the most recent on the commit that landed a check for a requirement enforced by
nothing. That figure is a record of the moment it was taken rather than a census: nothing produces it, and the
set it counts can still grow through the declared bound below, where a merge made outside the wrapper is not
observed.

The judgement SHALL be a Rust check returning the shared kinded refusal, so that a title that could not be
read is separated from a subject that disagrees, and so that its own construction sites are swept like every
other. Only the wrapper SHALL be shell, and it SHALL carry no verdict.

The refusals SHALL be ordered from most specific to least: a subject carrying a serial also differs from its
title and is also still conventional-shaped, and reporting the general fact for the specific one sends a reader
to compare two strings that differ by exactly the thing the rule names.

**What the gate judged SHALL be what the act records, for every judged input and not for the message alone.**
An input the gate received as a **value** SHALL reach `gh pr merge` as that value, never as a path or other
reference the tool re-resolves after the gate has run. The interval between the two holds a whole `cargo test`
run, and what lands cannot be amended — a squash commit's hash is cited by the pull request's merge record, so
correcting the commit afterwards decouples the two. This is the local half of the pin the head requirement
makes remotely: a pull request that moved is refused, and an input that moved on disk is never read a second
time.

The obligation is stated over the whole set because three of the four inputs already satisfied it while the
fourth did not, and nothing said they were one set: the subject travelled as a value, the repository was
resolved once and named on every call, the head was captured before the commit set and supplied as
`--match-head-commit`, and the live commit subjects were pinned through that head — while the body was handed
over as the path it had been read from. The wrapper's own allowlist already refuses a **caller's** body flag
in every spelling, naming this same split as its reason, which is what makes the wrapper composing one itself
a defect rather than a gap.

**The law applied SHALL be the judged repository's own.** The gate is loaded from the wrapper's own tree while
every input is resolved from the working directory. Those are one tree whenever the wrapper is run the way its
own refusals say to run it, and its refusal of a `--repo` selector enumerated the gate among the things read
"from the repository it is run in" — while nothing held them together. Invoked by absolute path from another
checkout they come apart in silence, and the wrapper would judge one repository's pull request by another
repository's law and then merge it. The two worktrees SHALL be compared before any evidence is read.

#### Scenario: The gate and the evidence would come from different repositories

- **WHEN** the wrapper is invoked from a checkout other than the one holding it, so its gate would be loaded
  from one worktree while the pull request resolves from another
- **THEN** it refuses as a cannot-judge before reading any evidence, naming both trees: applying one
  repository's law to another repository's pull request is a judgement about neither

#### Scenario: A squash subject carries the pull request's number

- **WHEN** a proposed subject ends in `(#N)`
- **THEN** the merge is refused, naming the serial rather than the fact that the subject differs from the title

#### Scenario: A squash subject is not the pull request's title

- **WHEN** a proposed subject differs from the title in any other way
- **THEN** the merge is refused; the title is what review saw, and a subject that says something else makes the
  record disagree with what was approved

#### Scenario: The pull request's title cannot be read

- **WHEN** the title is unavailable
- **THEN** the check refuses as a cannot-judge rather than as a disagreement, because an unread title is not
  a wrong subject

#### Scenario: The body file changes between the gate and the merge

- **WHEN** the file a body was read from is modified after the gate judged that body and before the merge runs
- **THEN** the merge records the value the gate judged, because the wrapper hands the tool that value and never
  the path it came from

#### Scenario: A hook is proposed for this rule — a stated bound

- **WHEN** someone reaches for a `commit-msg` hook, or for the repository's squash-title setting
- **THEN** neither holds it: a squash merge runs on GitHub's servers so no local commit exists and no hook
  runs, and both values of that setting append the serial. Nor can a merge made in the browser be reached by a
  wrapper. The compliance point is one string passed at merge time, and this check guards the sanctioned
  path to it rather than every path
- **PINNED-BY** `a_merge_made_outside_the_wrapper_is_not_observed`

### Requirement: What reaches a sanctioned irreversible act SHALL be an allowlist, not a denylist

**Only the gate's own verdict SHALL be able to exit the violation class, and that SHALL hold by construction
rather than by a sweep.** Under `set -e` any unguarded failure exits with the tool's status, so requiring
every statement to be guarded makes the obligation as large as the script. Two sweeps were widened trying to
hold it — first by tool name, then by command substitution — and a bare `cd` walked through both, because the
axis was never which shape a statement has. A wrapper SHALL therefore install an `ERR` trap reporting the
unjudged class, with `set -E` so it reaches failures inside functions, leaving exactly one statement able to
exit `1`: the arm carrying the gate's verdict. Measured on bash 5: a bare failure traps, a `||`-guarded
command does not, a failure in an `if`/`while`/`!`/`&&` condition does not, and an explicit `exit 1` is not
intercepted.

#### Scenario: An unguarded command fails

- **WHEN** any command a wrapper runs without a guard fails
- **THEN** the wrapper exits the unjudged class naming what happened in its own voice, rather than exiting the
  class that means a gate ran and refused — which, unguarded, it does with no output at all
- **PINNED-BY** `an_unguarded_failure_exits_the_unjudged_class`

Every wrapper standing in front of an irreversible act — the squash merge and the registry publish — SHALL
forward only arguments it names. An argument it does not name, including a spelling of a known flag and a flag a
future version of the tool adds, SHALL be refused before any evidence is read or gate is run, rather than passed
on to the act.

The admitted set SHALL be decided by **three** questions.

First: **does the argument move what the gate judged, or what the act records?** An argument that changes the
message, the strategy, the repository, the source tree, the set of crates, what the tool verifies, or what gets
packaged SHALL be refused; one that changes only whether and how the act proceeds MAY be forwarded.

Second: **does the tool honour it as the wrapper composes the invocation** — beside the arguments the wrapper
supplies itself? An argument the tool discards, or honours as something other than the judged act, SHALL be
refused or the composition corrected, because a forwarded argument that changes nothing is a promise the wrapper
does not keep. Neither may an argument defer the act past the evidence: the gate judges what exists when it runs,
so an argument that performs the act **later** SHALL be refused, since the record it produces need not be the one
that was judged. This classification SHALL be measured against the tool at a **named version** and that version
recorded beside the classification, since a tool's combination behaviour is not readable from its `--help`.

Third: **does the argument perform a further act after the judged one?** The first two questions ask about the
act itself; neither refuses an argument that leaves the judged act untouched and then does something else. The
merge wrapper admitted `--delete-branch` on that gap for a window, sharing an arm with `--admin` and carrying no
sentence of its own, while the criterion beside it admitted only arguments that change whether the merge
proceeds. Deleting the head branch is a post-merge act with an effect no rerun undoes: a pull request stacked on
that branch is auto-closed, and GitHub refuses to reopen it once the branch is gone and its head has moved —
which this repository has already paid for. An argument whose effect outlives the act and which the wrapper
cannot undo SHALL be refused, and its refusal SHALL name the consequence rather than the rule, because a
refusal an operator cannot act on is one they work around.

**A value position SHALL NOT be a place a refused argument may sit.** An arm that reads a following value
SHALL refuse a flag-shaped one, because the tool does not consume it: measured on cargo 1.96.0,
`cargo publish --package --no-verify` packages **without verifying** — byte-identical to passing `--no-verify`
alone — and exits 0 with no complaint about a package by that name. Every refusal a wrapper argues for is
otherwise reachable through the one selector it admits.

This SHALL hold for **every** value-taking arm rather than the one measured to leak. How the tool handles a
flag-shaped value differs per flag and per version — some consume it and fail later, some are refused by the
argument parser — and a wrapper standing in front of an irreversible act does not rest on the tool failing
correctly. It SHALL be decided by **shape**, a leading `-`, rather than against the refusal list: what makes
the value wrong is that the tool reads it as an argument of its own, so an argument nobody has classified is
refused for the same reason as a named one.

**The refusal SHALL state the wrapper's own property, not the tool's mechanism.** Saying *the tool reads
this value as an argument of its own* is true of one case and false of most: measured on cargo 1.96.0,
`--jobs --allow-dirty` has cargo consume the value and fail later with `could not parse --allow-dirty`, and
`--registry --config` is refused by clap with `a value is required for '--registry <REGISTRY>'`. Three
mechanisms, one sentence, so the sentence was wrong twice. What holds for every arm and does not expire with a
version is the wrapper's own rule: it does not accept a value beginning with `-`.

#### Scenario: A negative value cargo documents is refused by the shape rule — a stated bound

- **WHEN** a caller passes `--jobs -1`, which cargo documents — *If negative, it sets the maximum number of
  parallel jobs to the number of logical CPUs plus provided value* — and measured, `cargo publish --jobs -1
  --dry-run` packages and verifies normally
- **THEN** the wrapper refuses it, and nothing will admit it short of a per-arm rule. A leading digit means a
  job count for one arm and nothing for `--package` or `--registry`, so admitting it means the shape question
  is asked differently per arm — which is the arrangement one shape check exists to replace, and the
  arrangement whose per-arm reasoning the refusal wording above was just corrected for repeating. The engine
  owns the narrowing: the caller passes the count instead, one arithmetic step
- **PINNED-BY** `a_refused_flag_cannot_sit_in_an_admitted_arguments_value_position`

**A direction over an argument allowlist SHALL cross its axes.** Sending each refused argument alone, and each
admitted argument with a well-formed value, leaves the interaction between arguments owned by nothing — which
is where the refusal above walked through. The reaction SHALL hold every value-taking arm against every class
of refused argument, and SHALL take that set of arms **from the wrapper** rather than copying it alongside:
an arm takes a value exactly when it asks for one, so the request is the marker and no second list of flag
names exists to fall behind. The set read SHALL be held against a declared literal in both directions, so a
new arm fails until someone has looked at whether it may take a value at all.

**The reading SHALL rest on two properties of an arm that must agree, and SHALL fail on what it cannot
attribute.** Taking the guard request as the sole marker makes the check its own evidence: an arm spelled in a
shape the reader does not recognise asks to be guarded, is dropped from the set, is absent from the literal
too, and both directions hold over two sets that agree by both missing it — measured with an arm spelled
`-j)`, whose value-position refusal then never ran and nothing said so. So the reaction SHALL also read
whether the arm **consumes** the following argument, which is what taking a value is and owes nothing to the
wrapper's own helper, and SHALL require the two to name the same arms: one without the other is a refused flag
reaching the tool, or a refusal never exercised. **The two readings SHALL scan disjoint surfaces**, since a
guard request's own arguments name the value it guards: the canonical call carries the very token consumption
is read from, so an arm using it satisfied both tests on one line and the two properties agreed by
construction. The direction that dies is over-refusal — a non-value arm given a guard by mistake refuses the
*following flag* — which is the half a shape rule owes the caller. A guard request the reader cannot attribute to an arm SHALL
stop it, because a reading that shrinks its own subject reports on a set that does not describe the wrapper.

#### Scenario: A refused flag is written where a value belongs

- **WHEN** a refused flag is passed as the value of an admitted value-taking argument
- **THEN** the wrapper refuses, and the tool is never reached
- **PINNED-BY** `a_refused_flag_cannot_sit_in_an_admitted_arguments_value_position`

Each admitted argument SHALL be accepted in one spelling, with its value as a separate argument, because parsing
a tool's short, glued and equals forms is what a denylist has to get exhaustively right and an allowlist does
not. A misconfigured invocation SHALL exit `2`, the usage-error class, rather than `1`, which is what a gate that
ran and refused exits.

**The second question was missing, and both wrappers had an instance.** The publish wrapper admitted `--package`
while writing `--workspace` unconditionally; cargo maps that combination to *all packages* and says nothing, so
the selector this wrapper admitted precisely so a partly completed publish could resume instead published the
whole workspace. The merge wrapper admitted `--auto` and `--disable-auto`: the first merges after the gate has
read the evidence, so a commit pushed in between changes the set while the captured subject and body do not; the
second is not a merge, so the wrapper would run its gate, reach the tool, and exit `0` having recorded nothing.
An argument the wrapper supplies as a default SHALL be supplied as a default and not written over an argument the
caller gave.

**Enumerating what to forbid is the shape that failed, four times across both wrappers.** At the merge: a
`--repo` flag, a positional pull-request URL, and every short spelling of the flags the long-form arms named —
the last the sharpest, since `gh` accepts `-t` for `--subject` and `-F` for `--body-file`, the wrapper splices
forwarded arguments after its own, and `gh` reads the last occurrence of a repeated flag, so one unlisted
spelling replaced the message the gate had just approved. At the publish: everything but `--manifest-path` was
forwarded, so `--no-verify`, `--allow-dirty`, `--exclude`, `--config` naming a whole configuration file, and a
flag no cargo has all reached `cargo publish` with the wrapper exiting `0`. Both scripts carried the sentence *a
guard catching one would be a guard catching neither* while arguments walked past them. Refusing arms MAY remain
for the diagnostics they carry, but they SHALL decide nothing the default refusal would not.

#### Scenario: An argument the wrapper does not name

- **WHEN** a sanctioned wrapper is given any argument outside its admitted set, in any spelling
- **THEN** it refuses with a usage error before reading evidence or running its gate, and says that it forwards
  only what cannot change what the act records

#### Scenario: An argument that acts after the merge

- **WHEN** a caller passes an argument that leaves the judged merge untouched and then performs a further act
  whose effect the wrapper cannot undo — deleting the head branch
- **THEN** it is refused, and the refusal names what the act does to a pull request stacked on that branch, so
  the operator can tell when it is safe to do by hand
- **PINNED-BY** `only_an_allowlisted_flag_reaches_the_merge`

#### Scenario: An admitted argument reaches the act

- **WHEN** a sanctioned wrapper is given an argument that changes only whether and how the act proceeds
- **THEN** it is forwarded, so the refusal above is a rule about what moves the record rather than a wrapper
  that refuses its own arguments. Every admitted argument SHALL be shown to arrive, not one of them

#### Scenario: An argument the tool would discard

- **WHEN** an admitted argument would be voided or overridden by one the wrapper supplies itself
- **THEN** the composition names the caller's argument instead of the wrapper's default, and the direction
  holding it asserts the **selection the tool would honour** rather than the string the wrapper typed — a
  controlled executable logs arguments and cannot see a flag the real tool discards

#### Scenario: An argument that performs the act later

- **WHEN** an argument would defer the act past the moment the gate read its evidence
- **THEN** it is refused, because the gate's verdict covers the evidence that existed when it ran and not the
  record a later act would produce

#### Scenario: An admitted argument given no value

- **WHEN** a value-taking argument is passed with nothing after it
- **THEN** the wrapper names that argument and refuses, rather than exiting on the shift arithmetic with no
  diagnostic at all

### Requirement: A wrapper's exit class SHALL agree with the gate it fronts

A sanctioned wrapper SHALL exit `1` — the violation class — **only** where a gate ran and reported a
disagreement. Every other stop SHALL exit `2`: a misconfigured invocation, an input the wrapper could not read,
and a gate that did not run. The classification SHALL be chosen in one place per wrapper rather than at each site.

**A gate that did not run belongs to the unjudged class, however loudly its message says so.** The distinction is
already typed where the judgement lives: `refusal::Kind` separates a source that disagrees from one that could
not be read, and the merge gate returns cannot-judge for an unavailable title and for unavailable commit
subjects — *which is not the same fact as a subject that disagrees*. A wrapper reporting those as `1` tells an
operator to go looking for a disagreement that does not exist, which is the collapse the sibling publish gate
already refuses.

**The class SHALL travel on a channel of its own, not in the gate's prose.** The wrapper SHALL name a file for
the gate to report its class on; the gate SHALL write it at the moment it has a verdict and before it fails; and
the wrapper SHALL read that file rather than searching the gate's output. An absent, empty or unrecognised value
SHALL be the unjudged class, so a run that reached no verdict — a compile failure included — is unjudged by
construction rather than by a default. The variable name and the class spelling SHALL each be defined once and
compared against the wrappers by a repository check, and a direction SHALL hold that each gate reports before it
fails — the scalars can agree while no gate ever writes, which leaves every failing gate reading as unjudged.

**A channel that was opened and cannot be written SHALL fail the gate loudly**, naming the path and the error.
Absence is the unjudged class, and a discarded write outcome gives absence a second cause — a verdict the gate
reached and lost — which makes *unjudged by construction* false while leaving it stated. The gate SHALL NOT
report a class it could not deliver, and it SHALL NOT continue as though it had.

**What this SHALL NOT be read as claiming.** A refused verdict whose write fails still reaches the wrapper as
the unjudged class, because the channel is absent either way; the wrapper's exit class is unchanged by this
requirement. What the requirement buys is that the gate says which of the two facts the operator has, in its own
output, rather than failing on the refusal alone and leaving the absent channel to be read as a run that never
judged.

#### Scenario: The channel is opened and cannot be written

- **WHEN** a wrapper names a channel the gate cannot write to, and the gate reaches a verdict
- **THEN** the gate fails naming the channel and the error, rather than continuing with the class discarded

#### Scenario: A refused verdict whose class could not be delivered

- **WHEN** the verdict that could not be delivered is a refusal
- **THEN** the wrapper still reads the unjudged class and exits `2` — the channel is absent either way, so the
  gate's own output is what distinguishes a verdict lost from a verdict never reached

Reading the class out of the gate's output was the first attempt and it was the wrong channel twice over. It put
the delimiter in the shell and the variant name in Rust, so a check pinning the rendering's *arguments* stayed
green while a changed format string made the pattern match nothing — every violation then reporting as unjudged,
verbatim the failure that check's own prose said it prevented. And the stream searched carries arbitrary tooling
output, in which a class could be read from text no judgement wrote.

**Every input SHALL be read once, guarded, before the gate.** A test that a file exists is not a read: an
unreadable body file left the gate's body variable empty, and the gate refuses an empty body as a
disagreement — so a file the wrapper could not open was reported to the operator as a record they had written
wrongly. Reading once and handing the gate the value also closes the window between the test and the use.

**A wrapper SHALL leave no temporary file behind, on the path that completes the act as well as on the paths that
do not.** Cleanup SHALL NOT rest on an EXIT trap alone: a trap does not run when `exec` replaces the shell image,
so the one path that completes the act is the one path a trap never cleans. The trap SHALL remain, because it is
what covers the failure paths, and `exec` SHALL remain, because the tool's exit status becoming the script's is
deliberate — so the removal belongs immediately before the `exec`, where the file's purpose is spent. A direction
holding this SHALL observe an isolated temporary directory as a whole rather than one known name, so a temporary
file added later is covered without the direction being touched.

A direction holding any of these SHALL NOT skip on the subject's own behaviour. A skip for an environment that
cannot produce the condition SHALL be decided by a probe of the direction's own; deciding it from the wrapper's
exit status swallowed exactly the defect, since a wrapper that wrongly succeeds looks like an environment that
could not fail.

**Every acquisition SHALL be guarded.** An unguarded command substitution under `set -e` exits with the *tool's*
status and only the tool's stderr, so the class reported is neither of the two the wrapper defines and the
operator receives the tool's words for a fact about the wrapper. Measured: a failing commits read left the merge
wrapper exiting `91` in silence.

A direction holding any of these stops SHALL assert the **class**, not merely that the wrapper failed. Asserting
non-zero cannot see `1` from `2`, which is how five could-not-read conditions were split across both classes while
every direction covering them passed.

#### Scenario: An input the wrapper could not read

- **WHEN** a wrapper cannot read its body file, the repository identity, the pull request's number, head, or
  commit subjects
- **THEN** it exits `2` with its own diagnostic, because an input that could not be read is not a disagreement

#### Scenario: A gate that did not run

- **WHEN** a wrapper's gate invocation selects no passing test, or fails without rendering a verdict
- **THEN** it exits `2`, since no judgement formed and there is no disagreement to report

#### Scenario: A gate that ran and refused

- **WHEN** the gate renders a disagreement
- **THEN** the wrapper exits `1`, and that is the only site in the wrapper that may

#### Scenario: The channel a wrapper reads and the one a gate writes disagree

- **WHEN** a wrapper's variable name or class spelling differs from the judgement's, or a gate fails without
  reporting on the channel it was given
- **THEN** a repository check fails naming which, because every violation would otherwise be reported as unjudged

#### Scenario: An input that exists and cannot be read

- **WHEN** a file a wrapper was given is present and unreadable
- **THEN** the wrapper exits `2` naming the read it could not make, and the gate is never asked to judge a value
  that was never read

#### Scenario: The act completes

- **WHEN** a wrapper reaches the irreversible command and it succeeds
- **THEN** no temporary file it created remains, even though an EXIT trap would not have run

#### Scenario: An acquisition fails

- **WHEN** an external tool a wrapper reads evidence from exits non-zero
- **THEN** the wrapper reports it in its own words and its own class, rather than exiting with the tool's status

### Requirement: The merge SHALL be pinned to the head the gate read its evidence from

The squash wrapper SHALL obtain the pull request's head commit and require the merge to match it, so a pull
request that moved between the gate's verdict and the merge is refused rather than merged against a body that no
longer states its commits. The wrapper SHALL supply that pin itself; a caller-supplied one SHALL be refused,
because the tool takes the last spelling of a repeated flag and a chosen SHA would replace exactly the link the
pin exists to make.

**The head SHALL be read before the commit set.** Read first, a commit pushed in between leaves the commit set
ahead of the pinned head and the merge is refused — it fails closed. Read after, the pin would carry the new
commit while the gate judged the older set, so the merge would proceed and record a body missing it — it fails
open. The two orders are the same two calls and opposite guarantees, so the order is part of the requirement
rather than of the implementation.

A head that cannot be read SHALL stop the wrapper before the gate and the merge. An unreadable head is not a head
that has not moved, and merging unpinned because the pin could not be built is the vacuity direction in front of a
record that cannot be amended.

#### Scenario: The pull request moves between the gate and the merge

- **WHEN** a commit is pushed to the pull request after the gate has read its commit subjects
- **THEN** the merge is refused, because the head no longer matches the one the evidence came from

#### Scenario: A caller supplies the pin

- **WHEN** the wrapper is given a head-matching argument of its own
- **THEN** it is refused, naming that the wrapper supplies the head the gate read and a caller's would replace it

#### Scenario: The head cannot be read

- **WHEN** the pull request's head commit cannot be obtained
- **THEN** the wrapper stops before the gate and the merge, saying the merge could not be pinned

### Requirement: The squash wrapper SHALL judge the complete live pull-request commit set

Before invoking the squash-message gate, the sanctioned merge wrapper SHALL resolve the accepted pull-request
selector to one canonical numeric pull-request identity, then obtain every commit subject from that live pull
request rather than deriving the set from local remote-tracking refs. The acquisition SHALL include all pages,
SHALL derive the subject from the first line of each full commit message without headline truncation, and SHALL
work when the pull request head belongs to a fork. Failure to resolve the identity or acquire the live set, or an
acquired set containing no subjects, SHALL stop the workflow before the gate and before `gh pr merge`; it SHALL
NOT construct an endpoint from the unresolved selector or fall back to a local subset.

#### Scenario: Local remote-tracking refs are stale

- **WHEN** the live pull request contains a commit absent from the local base-to-head ref range
- **THEN** the wrapper supplies the live commit's full subject to the squash-message gate, so a default body
  containing it cannot escape as an unrecognized shape

#### Scenario: Pull-request commits span multiple API pages

- **WHEN** the live pull request's commits require more than one response page
- **THEN** the wrapper supplies subjects from every page to the gate in pull-request order

#### Scenario: The live commit set cannot be acquired

- **WHEN** the pull-request commits read fails or yields no commit subjects
- **THEN** the wrapper exits non-zero before invoking the squash-message gate or `gh pr merge`, without
  substituting local refs

#### Scenario: The accepted selector does not resolve to one canonical number

- **WHEN** `gh pr view` does not return a positive numeric pull-request identity for the accepted selector
- **THEN** the wrapper exits non-zero before constructing the commits endpoint, invoking the squash-message
  gate, or invoking `gh pr merge`

### Requirement: A capability SHALL declare the subject it governs

Every capability spec SHALL carry a `## Subject` section between `## Purpose` and `## Requirements`, listing
the tracked-path globs it governs. A capability that does not say what it governs cannot be joined to anything,
and a requirement's home is then decided by a name read loosely — which is how a requirement about a shell
wrapper came to be filed under a capability whose subject is Rust test files.

Membership SHALL be resolved by `git ls-files -- <glob>`. Git's pathspec is both the matcher and the meaning of
*tracked*, so no glob matcher is written here: a subject is a produced set, not a text model of one.

A capability whose subject is this repository's own checks SHALL name the members holding them rather than
a package's `tests/` directory, since the apparatus lives outside every published package.

Every declared glob SHALL match at least one tracked path. A glob matching nothing is a claim about nothing,
and it reads as coverage while providing none.

The subject SHALL NOT be assumed to tile the repository. A tracked file no capability claims is not judged by
the join below, and the check SHALL say so rather than imply a coverage it does not have.

#### Scenario: A capability declares no subject

- **WHEN** a capability spec carries no `## Subject` section
- **THEN** the check fails, naming the capability — an undeclared subject makes every filing decision about
  it unfalsifiable

**A bullet the reader cannot understand SHALL be refused, never dropped.** The form read is one backticked
glob and nothing else. A `- ` bullet the reader cannot parse used to fall out of a `filter_map`, so the
capability's declared subject shrank by exactly the bullets that failed to parse and the filing join then
missed every file those globs claimed — a capability quietly governing less than it says, which is the
condition this requirement exists to make falsifiable, produced by the reader enforcing it. This is the same
obligation `adopter-surface` states for the prelude's members, for the same reason: a reader that narrows a
claim by the amount it failed to read reports the narrowed claim as the whole one.

#### Scenario: A declared glob matches no tracked path

- **WHEN** a `## Subject` glob resolves to no tracked file
- **THEN** the check fails, naming the capability and the glob

#### Scenario: A subject bullet the reader cannot parse

- **WHEN** a `## Subject` bullet is not one backticked glob — prose after the closing backtick, no backticks,
  an unterminated one
- **THEN** the check refuses as a **cannot-judge** naming the bullet, rather than reading past it: the section
  may claim exactly the right files and this reader cannot say, while a shorter glob list would be the silent
  narrowing itself

**The same obligation reaches the section, and the reader SHALL answer how many there are.** Taking the text
after the first `## Subject` marker makes no choice about the count: a spec carrying two sections had the
second one's globs dropped, so the capability governed less than it says while reading as a complete
declaration — the identical narrowing the bullet rule above closes, one level up from it, and correct only
while a second section happened not to exist. Several sections SHALL be refused naming the count, which is
what an author acts on; reusing the bullet wording would send them looking for a bullet that parses fine.

#### Scenario: A capability declares its subject twice

- **WHEN** a capability spec carries more than one `## Subject` section
- **THEN** the check refuses as a **cannot-judge** naming how many, rather than reading the first: which
  section declares what the capability governs would otherwise be decided by file order, dropping every glob
  the others claim

#### Scenario: The tracked-path enumeration fails

- **WHEN** `git ls-files` fails while resolving a subject
- **THEN** the check refuses as a cannot-judge naming the capability and the glob, never as an empty subject
  — a failed enumeration returns exactly what a glob matching nothing returns

#### Scenario: Files no capability claims — a stated bound

- **WHEN** a tracked file is claimed by no capability's subject
- **THEN** no repository check fires. Subjects are declared where a capability has something to say, and requiring
  them to tile the tree would buy coverage with a claim per capability nobody could defend. The blindness is
  declared so that a clean report is not read as a complete one, and the check prints how many tracked
  paths went unclaimed rather than leaving the reader to assume none did
- **PINNED-BY** `files_no_capability_claims_are_reported_rather_than_implied_judged`

### Requirement: A change SHALL name every capability whose subject it touches

A change's proposal SHALL list, in its Capabilities section, a capability claiming each file the change
actually touches. The touched set SHALL be **produced** — the change's diff against its base — and never read
from the change's own prose, because the capability list and any prose inventory come from the same decision
and comparing them is a comparison of a value with itself.

**Every** capability claiming a touched file SHALL be accounted for, not one of them. Naming one was measured
unable to catch the defect this requirement was written from: the publish wrapper is claimed both by the
capability governing what must be true before a publish and by the capability governing this repository's
checks, so a change naming only the second passed while filing a wrapper's requirement under a
repository-check subject.

Accounting for a capability is **not** listing it as modified: a Capabilities section naming it while stating
why its requirements do not change satisfies this. So requiring all of them refuses no honest proposal — it
requires the proposal to say what it is doing, which is the discipline the join exists to make routine.

The base SHALL be resolved, and a base that cannot be resolved SHALL be a cannot-judge. Reading an
unresolvable base as *nothing was touched* would report clean over every change, which is the direction this
requirement exists to close.

Where no change is active, the check SHALL be clean. An ordinary checkout is asking no filing question, and
a check that refuses one is noise rather than governance.

#### Scenario: A change touches a file whose capability it did not name

- **WHEN** a change modifies a file claimed by some capability's subject, and its proposal's Capabilities
  section names no capability claiming that file
- **THEN** the check fails, naming the file, the capability that claims it, and the capabilities the
  proposal did name

#### Scenario: A shell wrapper filed under a repository-check capability

- **WHEN** a change modifies `scripts/publish.sh` and names only a capability whose subject is
  `crates/kanhe/tests/**/*.rs`
- **THEN** the check fails. This is the defect the requirement was written from, and it is the direction the
  check is held to

#### Scenario: The change's base cannot be resolved

- **WHEN** the branch's base cannot be determined from its upstream or from the tracked release and main refs
- **THEN** the check refuses as a cannot-judge naming the branch, never reporting clean

#### Scenario: No change is active

- **WHEN** `openspec/changes/` holds no active change
- **THEN** the check is clean, having no filing decision in front of it

The capability list SHALL be read from **one** `## Capabilities` section. Several is the same unanswered
count the subject reader now refuses, one document over: reading the first drops the capabilities the others
name, and the join would then report a change as having accounted for a capability it never listed.

#### Scenario: A proposal lists its capabilities twice

- **WHEN** a proposal carries more than one `## Capabilities` section
- **THEN** the check refuses as a cannot-judge naming how many, rather than reading the first

### Requirement: A check SHALL take the region it judges from the shared classifier

A repository check deciding a property about **executed** text SHALL obtain its corpus from `kanhe::region`,
which classifies a format once and carries the decision in the type. It SHALL NOT re-decide the region at the
call site by filtering comment markers inline.

The rule exists because the shape keeps costing defects. Six were recorded when the classifier was written, all
one shape — *the corpus was taken to be the whole blob when the property was about a distinguished part of it*.
Two more were found afterwards in a check that never adopted it, and one of the two is two scans of a single
file disagreeing about the same question five lines apart. A helper was the first answer and reached most
callers but not all; the type is the second, and adoption is what this requirement adds.

**An acquisition SHALL be recognized past whatever precedes the tool name.** A sweep testing the text
immediately after a command substitution opens is blind to an environment-prefixed acquisition, which is the
form the central gate invocation takes in both sanctioned wrappers. The tool is what the property is about; the
assignments in front of it are not.

**Two scans of one file that disagree about the region are a defect regardless of whether either currently
admits a wrong answer**, because the region is a property of the format and not of the scan. That is stated
here as a definition rather than given a scenario: it is as invisible as the absence declared below, and for
the same reason — seeing it would need the reaction this requirement records as measured and rejected. A
scenario would read as a claim that something looks.

**Selecting comments is not re-deciding a region, and SHALL NOT be read as a violation of this.** A check whose
subject *is* the commentary — that a doc comment directs a reader somewhere — necessarily recognizes comment
lines, and so does a check parsing a data format whose own syntax marks comments. The rule is about a property
over executed text being decided on unclassified text, not about the marker appearing.

#### Scenario: An acquisition prefixed by environment assignments

- **WHEN** a wrapper acquires a value as `var=$(NAME=value tool …)`
- **THEN** the sweep recognizes it as an acquisition of `tool`, because what precedes the tool name is not what
  the property is about

#### Scenario: A check whose subject is the commentary

- **WHEN** a check recognizes comment lines in order to judge what a comment says, or to parse a data format
  whose syntax marks comments
- **THEN** nothing reacts: the region was not re-decided, it is the subject

#### Scenario: A check that should distinguish a region and does not — a stated bound

- **WHEN** a check judges a property over executed text on unclassified text — having written no region
  decision at all, or having written one that a neighbouring scan of the same file contradicts
- **THEN** no reaction sees either. An absence is not a shape, and nothing can scan for a filter that was never
  written; a disagreement between two scans is visible only to something that can already recognize what a
  region decision is, which is the same reaction. A reaction refusing an inline region decision was designed and measured against this repository:
  of the sites carrying the marker, only some are this class — the rest select commentary deliberately or parse
  a data format whose syntax marks comments — so it would refuse more legitimate sites than defects, which is
  how a gate earns being turned off. The classifier's adoption is what narrows this, and the narrowing is not a
  closure. The bound is declared on the construction rather than on an instance: a candidate one was reported
  and **refuted by measurement**, and a bound resting on a refuted instance would be worse than none
- **UNPINNED** `BACKLOG.md` — *a check that never wrote a region decision is invisible*

**The classifier's shell region approximates the shell's rule, and SHALL say so where the rule is stated.** It
cuts a marker at line start or after whitespace and nowhere else, which agrees with bash on the shapes this
repository writes and diverges on two it does not. Three sibling paragraphs restated the rule and two
overclaimed — one calling it the shell's own, one scoping the string-literal residue to the Rust region while
both regions run the identical rule. The rule now has one owner and the divergences are declared below.

#### Scenario: A shell comment opened by a metacharacter stays in the executed region — a stated bound

- **WHEN** an executed shell line writes the comment marker straight after an unquoted metacharacter, where
  bash opens a comment
- **THEN** the region keeps it, a stated bound: the rule tests for whitespace or line start, so commentary can
  satisfy a property about executed text. The direction is the over-including one and closing it needs
  word-splitting the classifier does not do; no tracked script carries the shape on an executed line
- **PINNED-BY** `a_shell_marker_after_a_metacharacter_stays_in_the_region`

#### Scenario: A whitespace-preceded shell marker inside quotes is cut — a stated bound

- **WHEN** an executed shell line carries the marker inside a quoted string with whitespace before it, where
  bash keeps it as string content
- **THEN** the region cuts it, a stated bound: executed text is deleted, which is the direction the Core
  Contract forbids, and the residue was for a window recorded as reaching the Rust region alone. Closing it
  needs the quote tracking the TOML region has, rewritten for the shell's own quoting; no tracked script
  carries the shape on an executed line
- **PINNED-BY** `a_shell_marker_inside_quotes_is_cut_from_the_region`

**A command a tracked document hands a reader SHALL name a target that exists.** The obligation above is
about the commands a *wrapper* runs; this is the same claim reaching the audience that cannot debug it. The
instance: `COOKBOOK.md` told an adopter to run the examples suite under the `tianheng` package, where that
target lives in `shengmo`, so cargo answered that no test target of that name exists in that package — it
arrived in the `0.5.0` window when the shell suite migrated, while `AGENTS.md` and `BACKLOG.md` both carried
the correct package. The set of targets SHALL be **produced** by `cargo metadata`, never modelled by mapping a package and a
target name onto a path under that crate's `tests/` directory: that mapping reimplements cargo's target
resolution in string form, which this repository has already shipped a false negative from doing.

The corpus is tracked Markdown. A Rust source carries these pairs as **fixture input** — a parser direction
plants a package-and-target pair as text — and admitting them would report a test asserting its own parser as a broken
command. Measured when this was written: 35 occurrences across the tree, two of them those fixtures, and one
review-reported defect; running the check found **four more**, every one in an example README.

#### Scenario: A document names a package that does not carry the target

- **WHEN** tracked Markdown writes a `cargo test` invocation naming a package and a `--test` target that
  `cargo metadata` does not report as a pair
- **THEN** the check fails naming the document and the command, because a reader following it meets an error
  rather than a gate

### Requirement: A constant a check judges by SHALL be held against its enumerator

A list a check judges against SHALL be compared with whatever enumerates its set, wherever such an enumerator
exists in this repository. The comparison SHALL run **in both directions**: nothing the enumerator produces is absent from the list, and
nothing the list names is absent from the enumerator. A one-directional comparison catches an omission and
misses an entry that has outlived its subject, which are the two ways one list falls behind another.

**Where no enumerator exists, the list SHALL say so rather than say nothing.** The neighbouring attribution
constant does this already: it states which half of its rule is unenumerable and why the open half stays a
reviewer's obligation. A list that is fully enumerable and states nothing reads as though nothing could hold
it.

This is the third door one class has come through. The wrapper list is already held, and its own documentation
names the risk it closes; the constants below were the same shape with nothing behind them. Adding a guard per
instance is what a repeated class refuses — the rule is stated once and each instance answers it.

#### Scenario: A list the contract also states

- **WHEN** a check's constant enumerates something a governance document also enumerates, and one gains or
  loses a member
- **THEN** the comparison fails, naming the side that has the member and the side that does not

#### Scenario: A list whose enumerator is a directory or a parser

- **WHEN** a check's constant enumerates something a tracked directory or an executable allowlist produces, and
  the two disagree in either direction
- **THEN** the comparison fails, naming what is unmatched and on which side

#### Scenario: A constant with no enumerator

- **WHEN** a check's constant enumerates something nothing in the repository produces
- **THEN** nothing reacts, and the constant states that its set is not enumerable and what stays a reviewer's
  obligation — a silence that is written is not the same as a silence that was never considered

### Requirement: A token with a constant owner SHALL be spelled once inside that owner's reach

Where a constant exists so that one token has one owner, that value SHALL appear **exactly once** in the
executed text of every module able to reach the constant, and that one appearance SHALL be the declaration
itself. A second spelling is a second owner: the two can disagree about one token, and a change to the
constant leaves the copy behind.

The corpus SHALL be **the reach of the constant** — the owning crate and every workspace member depending on
it — and that set SHALL be derived from the manifests rather than listed, or listed and held against them
both ways under the requirement above. A member outside that reach spells the token itself because the
dependency edge would close a cycle; that is a fact about the graph, so it sits outside the subject rather
than inside it as an exemption, and the check SHALL say so where a reader meets the constant.

Exemption SHALL be by **declaration**, never by file. Skipping the owner's whole file exempts more than the
declaration it means to, and a second constant carrying the same value beside the first then reads as clean —
which is the corpus-narrower-than-the-claim shape this check exists to refuse, in the check itself.

An enumeration that cannot be read SHALL refuse rather than shrink the corpus: a module this check could not
open is one it did not compare, which is not a module without a second spelling.

#### Scenario: A second spelling inside the reach

- **WHEN** a module able to reach the constant carries the constant's value as a literal
- **THEN** the check refuses, naming every site and the constant that owns the token

#### Scenario: A second declaration beside the first

- **WHEN** the owner's own file carries a second constant with the same value
- **THEN** the check refuses, because the exemption is the declaration and not the file

#### Scenario: The declared corpus and the dependency graph disagree

- **WHEN** a member is added to the graph and not to the corpus, or left in the corpus after it stopped
  depending on the owner
- **THEN** the comparison fails in whichever direction disagrees, naming both sets

#### Scenario: A spelling outside the constant's reach

- **WHEN** a crate that cannot depend on the owner spells the token itself
- **THEN** it is outside the corpus rather than an exemption inside it, and the constant's own documentation
  states which crates those are and why the edge cannot exist

### Requirement: A gate a wrapper asks for SHALL be observed to have run, and the name it is asked for by SHALL be pinned

A wrapper that asks for a check by test name SHALL treat *the filter matched nothing* as a failure of the
wrapper, and the identifier it cites SHALL be held against the test that carries it. Being named where the run
is decided is not being run there.

**A filter matching nothing is not a clean gate.** `libtest` exits `0` when `--exact <name>` selects no test —
measured against a prebuilt binary, an unknown name reports `0 passed; 0 failed; N filtered out` and exits
`0`. Exit status alone therefore cannot separate *judged and found nothing wrong* from *judged nothing*, and
the two wrappers asking for a gate this way both stand in front of an act that cannot be undone. Each SHALL
require the run to report exactly one passing test, and SHALL surface what it saw when it does not.

**The assertion SHALL stand in the wrapper, before the irreversible command** — not inside the gate it guards.
A renamed or `#[ignore]`d test cannot report that it did not run, so a guard the disarming disables is not a
guard.

**The cited identity SHALL be pinned by a check.** For every tracked shell script, each `--exact <ident>`
SHALL be joined to the `--test <target>` of the same invocation, and that target SHALL register `<ident>`
exactly once. A test identifier is a reference into this repository exactly as a path is, and the reference
gate matches paths only.

**Every tracked script SHALL carry at least one such citation, and that SHALL be held per script.** A script
citing no gate renders its own verdict, which is the shape this capability's Purpose refuses and the shape its
retired predecessor described in full: `check_*.sh` gates paired with `test_*.sh` twins over a shared shell
library, 1562 lines of it — a figure measured when that shell was deleted and standing as a record of that
moment, not a census: no reaction produces it, and the set it counted no longer exists. The direction that
enumerates the scripts folded every citation into one list and
asserted that **list** was non-empty, so a script contributing nothing was invisible while any sibling
contributed something — the whole way back was open, and the enumeration that would have seen it was already
running.

**The consequence is stated rather than discovered: `scripts/` becomes a closed category.** A tracked script
that is not a wrapper cannot be added there while this holds, which is what the capability already claims when
it says `git ls-files scripts/` names only wrappers. Making that claim hold is the point; a convenience script
belongs somewhere this requirement does not reach, or the requirement is amended deliberately.

**Both SHALL hold, and neither substitutes for the other.** Measured rather than reasoned: `--list` includes an
`#[ignore]`d test, so the check cannot see a silenced gate, while `--exact` on one reports `0 passed; 1
ignored` and exits `0`, so the wrapper can. The check runs where the suite runs and a wrapper is run
locally; the wrapper's assertion runs when a wrapper is invoked and a rename lands in a pull request long
before that.

#### Scenario: The gate's test no longer answers to the name the wrapper cites

- **WHEN** a wrapper asks for its gate by a name no test in the target carries — through a rename, a move, or
  an `#[ignore]`
- **THEN** the wrapper exits non-zero before the irreversible command, printing the run's output and saying
  the name in the script no longer names a test; it does not reach `cargo publish` or `gh pr merge`

#### Scenario: A gate that ran and refused, and a gate that did not run

- **WHEN** one wrapper's gate runs and reports a violation, and another's matches no test
- **THEN** both stop before the act and each says which happened; the second is not reported as a passing
  gate, which is what `libtest`'s exit status alone would say

#### Scenario: A renamed gate is red in the ordinary suite

- **WHEN** a test a tracked script names by `--exact` is renamed, moved to another `--test` target, or
  registered twice
- **THEN** the pinning check fails in an ordinary run, naming the script, the identifier, and the target it
  was cited against — before any wrapper is invoked

#### Scenario: A tracked script cites no gate at all

- **WHEN** a tracked shell script carries no `--exact <ident>` citation anywhere, while its siblings do
- **THEN** the check fails naming that script, because a script that defers its verdict to nothing is rendering
  one itself; the aggregate being non-empty says only that some script cites a gate, never that this one does

#### Scenario: An invocation whose identifier cannot be bound to a target

- **WHEN** a tracked script writes `--exact <ident>` with no `--test <target>` in the same invocation
- **THEN** the check refuses as a cannot-judge naming the script and the identifier: an identifier it
  cannot bind to a target is one it could not resolve, not one it resolved as fine

#### Scenario: The script enumeration fails

- **WHEN** the tracked-script enumeration fails
- **THEN** the check refuses as a cannot-judge rather than reporting clean over an empty list, since a
  failed enumeration returns exactly what a repository holding no scripts returns

#### Scenario: A tool configuration set in the environment is not observed — a stated bound

- **WHEN** a value a sanctioned wrapper refuses as an argument is exported into its environment instead
- **THEN** the wrapper does not see it, a stated bound: the allowlist classifies **arguments**, and cargo takes
  the same configuration from the environment — measured on cargo 1.96.0, `--target not-a-real-triple` and
  `CARGO_BUILD_TARGET=not-a-real-triple` produce the identical rustc-probe failure. Closing it is ordinary work
  here rather than another layer's, since the wrapper could scrub the environment before invoking the tool; it
  needs an allowlist **over the environment**, and legitimate setups export `CARGO_HOME` and `CARGO_TARGET_DIR`,
  so which set to admit is a decision this bound records instead of guessing
- **PINNED-BY** `a_tool_configuration_set_in_the_environment_is_a_stated_bound`

#### Scenario: A gate reached without the wrapper — a stated bound

- **WHEN** someone runs `cargo publish` directly, or merges in the browser
- **THEN** no repository check fires. Both assertions guard the sanctioned path; reaching further would mean observing
  the operator's shell or GitHub's servers rather than this repository. The pinning check narrows this
  without closing it: it keeps the sanctioned path sanctioned, so what is left is choosing not to use it
  rather than using it unguarded
- **UNPINNED** `BACKLOG.md` — *a merge or publish made outside the wrapper is not observed*

The subject SHALL be re-read against the pull request's title **after** the gate, and a title that moved
SHALL be a **cannot-judge**. The wrapper judges three inputs and pins two of them by construction — the body
travels as the value the gate judged, and the commit set through `--match-head-commit`, which the server
decides atomically. The title was captured once, so an edit during the gate left the merge recording a
subject that is no longer the title. The class is cannot-judge rather than violation because the gate did not
find the subject wrong: it found it right, against a title that no longer exists.

#### Scenario: A title edited while the gate ran

- **WHEN** the pull request's title differs between the wrapper's evidence read and its post-gate re-read
- **THEN** the wrapper stops before `gh pr merge`, exits `2`, and names both titles
- **PINNED-BY** `a_title_edited_while_the_gate_ran_stops_before_the_merge`
- **PINNED-BY** `an_unchanged_title_still_reaches_the_merge`

#### Scenario: A title edited inside the re-read itself — a stated bound

- **WHEN** the pull request's title changes between the wrapper's post-gate re-read of it and `gh pr merge`
- **THEN** nothing observes it, and the merge records the subject the gate approved against a title that has
  since moved. The wrapper judges three inputs and pins two of them by construction — the body travels as
  the value the gate judged, and the commit set is pinned through `--match-head-commit`, which GitHub
  decides atomically. `gh` offers no `--match-title`, so the third can only be re-read, which shrinks the
  exposure from a whole `cargo test` to one API call rather than closing it. Closing it needs a
  server-decided precondition this tool does not offer
- **UNPINNED** `BACKLOG.md` — *the title race the wrapper can only narrow*

### Requirement: The squash-message gate SHALL refuse a shape by what it is, not by what it resembles

The gate SHALL refuse a message for what it is rather than for what it resembles: a refusal at the merge is a
false refusal as costly as a miss, blocking a legal record at the one moment nothing can be undone. Two of its
checks read a resemblance rather than the thing.

The **breaking marker** SHALL be read from the Conventional Commit head — the text before `": "` — never from
anywhere in the subject. `fix(tianheng): preserve bang! in summaries` announces no migration, and the ability
to read the head already sits in the same judgement, which strips a trailing `!` before matching the type.

A **bare commit list** SHALL be recognised by what its bullets say. The pull request's own commit subjects
SHALL be supplied to the judgement, and a body SHALL be a bare list when every bullet is one of them. A body
of `- Why: …` / `- Contract: …` is self-contained and its shape is not the question.

Tightening the recogniser instead — requiring a bullet to look like a Conventional Commit — SHALL NOT be used:
every commit in this repository is conventional, so it would refuse a hand-written body of `- fix: …` bullets
while a branch carrying one non-conventional subject slipped through.

An **agent attribution mark** SHALL be matched without regard to ASCII case, and by the shape the mark has. The
recognition SHALL travel beside each mark rather than as one rule applied to all of them, because they are not the
same kind of thing:

- a **trailer** — a `Key: Value` mark such as the co-authored trailer or the generated-with footer — SHALL be
  recognised at the start of a line, so a body that *names* one inside a sentence is not carrying it. This gate
  would otherwise refuse the commit message of any change about this rule, which is the false refusal this
  requirement forbids. **A line-start match SHALL also require the mark to END there**, and the two line marks
  end differently because only one of them is a key: a trailer key ends at its `:`, and a footer phrase ends at
  a word boundary — `Generated with Claude Code` carries no colon, so demanding one would stop refusing the
  real mark. Without a boundary the prefix runs on and a line beginning `Co-authored-bystander …` is refused
  while carrying no attribution at all, which is this same false refusal reached from the other end;
- a **glyph** with no legitimate use in this repository's messages SHALL be recognised wherever it appears.
  Reading it by position would let a subject carrying it mid-line pass, which is a miss rather than a false
  refusal. Prose about the rule names such a glyph in words.

The gate holds the marks `AGENTS.md` names. That document also forbids "any other tool-authorship mark", which is
not enumerable, so the open clause SHALL remain a reviewer's obligation and SHALL be stated as such rather than
implied by a list that reads as complete.

Case-sensitivity was the live defect: the canonical spellings are not the ones the check listed — git writes the
trailer with only its first letter capitalised and GitHub renders it that way — so the form most likely to appear
was the form not caught. Measured before the widening, two canonical spellings were accepted.

#### Scenario: An attribution mark in the case the tool actually writes

- **WHEN** a squash message carries a line that is an attribution trailer in any ASCII case
- **THEN** the gate refuses it as a violation

#### Scenario: A sentence naming an attribution mark

- **WHEN** a body names a trailer mark inside a sentence, as prose about the rule
- **THEN** the gate accepts it, because naming a mark is not carrying one

#### Scenario: A glyph mid-line

- **WHEN** a subject or body carries the forbidden glyph anywhere, not at the start of a line
- **THEN** the gate refuses it, because reading that mark by position would be a miss

Where the commit subjects cannot be read, the judgement SHALL refuse as a cannot-judge rather than fall back
to the shape, because falling back is the false refusal being removed.

#### Scenario: A summary containing an exclamation mark

- **WHEN** a subject carries `!` after the `": "` and its head does not end in one
- **THEN** the message is accepted without a `BREAKING CHANGE:` footer, while a head ending in `!` still
  requires one

#### Scenario: A terse body written entirely as bullets

- **WHEN** every non-blank line of the body is a bullet and none of them is one of the pull request's commit
  subjects
- **THEN** the message is accepted: the body is self-contained, and its formatting is not what the rule is
  about

#### Scenario: GitHub's default body

- **WHEN** the body's bullets are the pull request's commit subjects
- **THEN** the merge is refused, which is the default this rule exists to replace

#### Scenario: The commit subjects cannot be read

- **WHEN** the wrapper supplies no commit subjects
- **THEN** the judgement refuses as a cannot-judge naming what it could not read, rather than falling back to
  the shape it was refusing before

### Requirement: The prelude promise SHALL be held against the contract compiled from outside

A repository check SHALL hold the composed wildcard prelude's promise against the contract compiled from
outside — `adopter-surface` declares the promise, and a repository check is what holds it. The check SHALL
read the promise from the prelude's own block, read the external-view integration test compiled against it,
and refuse when a promised member is mentioned nowhere. It SHALL refuse as a cannot-judge when the promise
parses to nothing or the contract yields no identifier, since both make every direction hold vacuously.

**An input the reader cannot read SHALL be refused naming what it could not read, never returned as an empty
promise.** The two reach the same exit class and are not the same fact: one is a malformed source to repair,
the other a prelude that genuinely promises nothing, and a reader that reports the first as the second sends
its operator to the wrong repair. One arm collapsed them — a `pub use super::{` statement reaching no `};`
returned an empty member set, which the vacuity guard above then reported as *the promise parsed to no
member*, discarding whatever earlier statements in the same block had already contributed. The direction is
safe (it refuses either way) and the diagnostic was not.

#### Scenario: Whether a mention compiles anything is not observed — a stated bound

- **WHEN** the contract names a promised member only in a comment
- **THEN** the check counts it as named, a stated bound: deciding that a mention is load-bearing is a
  judgement over text, the instrument this repository has designed, measured and rejected, and what makes a
  mention bite is the compiler rather than this check. A comment-only mention still fails the reviewer reading
  the diff, which is the layer that owns it
- **PINNED-BY** `a_member_named_only_in_a_comment_is_counted_as_named`

#### Scenario: A re-export statement the reader cannot terminate

- **WHEN** the prelude block carries a `pub use super::{` statement that reaches no `};` — `pub use
  super::{A, B} ;`, which is legal Rust — after a statement whose members were read
- **THEN** the check refuses as a cannot-judge naming the statement it could not terminate, rather than
  returning the empty member set that the vacuity guard reports as a promise of nothing
- **PINNED-BY** `an_unterminated_reexport_statement_is_refused_rather_than_read_as_an_empty_promise`

#### Scenario: A longer word that starts like an attribution mark is not one

- **WHEN** a body line begins with a longer word that merely starts like a mark — `Co-authored-bystander …`,
  `Generated withheld …`
- **THEN** the gate does not refuse it, because the mark must end where it ends: a trailer key at its `:`, a
  footer phrase at a word boundary. Refusing it is the same false refusal the line-start rule exists to
  prevent, reached from the other end
- **PINNED-BY** `a_longer_word_that_starts_like_a_mark_is_not_carrying_it`

### Requirement: Whitespace hygiene SHALL be held over every tracked text file, and hold itself to this family's contract

No tracked text file SHALL carry trailing whitespace on a line, end with a blank line, or lack its final
newline. Membership SHALL be produced by `git ls-files --eol`, and what counts as binary SHALL be git's own
classification rather than a rule written here — the same deference the subject requirement makes to git's
pathspec.

**This requirement is written late, and that is the finding.** The check existed, ran in the Definition of
Done and in CI, and was described by no requirement in any capability — the one reaction in this family
without one, while its file sat inside this capability's declared subject. Nothing catches that: subjects
claim files, not the requirements those files hold, and *reactions without requirements* is the sweep this
repository runs by hand.

The check SHALL answer with the shared kinded refusal, so that **a file it could not read is separated from a
file that disagrees**. It had collapsed the two in the direction that reports clean: an unreadable tracked
file was passed over, so a file nobody could open counted as hygienic — the identical condition
`census::sweep` refuses, in its own words, because *an unread document is not a document without one*.

The check SHALL assert that it inspected at least one file, before reaching its verdict. Every path that
leaves a file uninspected drops it into a silence indistinguishable from cleanliness, and one such path
depends on `git ls-files --eol` writing exactly one tab before the path. Measured with that separator changed
and the parse failure passed over: 389 tracked files, **zero** inspected, and the check reported clean.

#### Scenario: A tracked file cannot be read

- **WHEN** a file `git ls-files` names cannot be opened
- **THEN** the check refuses as a cannot-judge naming the path and the error, because an unread file is not a
  file without offences
- **PINNED-BY** `an_unreadable_tracked_file_is_refused_rather_than_skipped`

#### Scenario: A listing line the reader cannot parse

- **WHEN** a `git ls-files --eol` line carries no tab before its path
- **THEN** the check refuses as a cannot-judge, rather than passing over it — a line it cannot parse is a
  tracked file it did not inspect
- **PINNED-BY** `a_listing_line_without_a_path_separator_is_refused`

#### Scenario: No file was inspected

- **WHEN** every listing line takes a path that leaves its file uninspected
- **THEN** the check fails on the vacuity, naming how many listing lines it read, rather than reporting the
  empty offence set as cleanliness

#### Scenario: A file carries whitespace this repository does not keep

- **WHEN** a tracked text file has trailing whitespace, a blank line at its end, or no final newline
- **THEN** the check reports it as a violation naming the path, and the line for trailing whitespace
- **PINNED-BY** `each_offence_shape_is_named_when_it_is_shown`

### Requirement: An amendment to the self-law is named before it lands

The set of boundaries `shengmo::law::constitution()` declares SHALL be declared in this repository as text,
and held against the projection `AGENTS.self-law.md` renders **in both directions**. The declared identity
SHALL carry each boundary's **heading, reason, rule and severity**, so that widening an allowlist, lowering a
severity from `enforce` to `warn`, or rewriting a reason each move the set exactly as adding or removing a
boundary does. A repeated boundary on either side SHALL be refused before the comparison, because a set built
over one would fold it away and hold over a law neither side fully saw.

What this requirement establishes is that an amendment is **named**, not that it was accepted. One actor can
change the law, regenerate the projection and edit the declaration in a single change and satisfy it. Human
acceptance rests on a steward decision, which a single-steward repository has no mechanical second party to
carry; that limit SHALL be recorded as a judgement boundary rather than described as satisfied by this check.

The subject SHALL be the projection's tracked text rather than `constitution()` itself: a check calling the
law it is judging compares the law against itself and cannot fail.

`.github/CODEOWNERS` states that the review requirement is the reaction and that a merge cannot relax the law
without a human accepting it, then states that designation alone only auto-requests review. Measured, `main`
carries `require_code_owner_reviews: false` and `required_approving_review_count: 0`; and enabling it would
not close the gap, because a pull request's author cannot approve their own, so for a single-steward
repository that rule cannot fire. Two crate boundaries reached the projection under a commit body stating the
law itself did not change, and nothing refused them.

#### Scenario: A boundary reaches the law without being named

- **WHEN** `constitution()` gains a boundary and the projection is regenerated
- **THEN** the check fails naming the projected boundary the declaration does not carry, because an amendment
  nobody named is the one this requirement exists to refuse
- **PINNED-BY** `the_law_declares_no_boundary_this_repository_has_not_named`

#### Scenario: A severity is lowered from enforce to warn

- **WHEN** a boundary keeps its heading, reason and rule while its severity changes
- **THEN** the check fails naming that boundary, because turning a run-failing violation into an advisory is
  the relaxation that moves the fewest characters
- **PINNED-BY** `the_law_declares_no_boundary_this_repository_has_not_named`

#### Scenario: A boundary's reason is rewritten

- **WHEN** a boundary keeps its target, rule and severity while the sentence a reader takes its meaning from
  changes
- **THEN** the check fails naming that boundary
- **PINNED-BY** `the_law_declares_no_boundary_this_repository_has_not_named`

#### Scenario: The same boundary appears twice

- **WHEN** either the declaration or the projection carries one boundary twice
- **THEN** the check fails on the repeat before comparing, rather than folding it away
- **PINNED-BY** `the_law_declares_no_boundary_this_repository_has_not_named`

#### Scenario: An allowlist is widened and the projection re-blessed

- **WHEN** a boundary keeps its target and its rule permits more than the declaration records
- **THEN** the check fails naming that boundary, because relaxing the law widens a boundary far more often
  than it deletes one
- **PINNED-BY** `the_law_declares_no_boundary_this_repository_has_not_named`

#### Scenario: A declaration outlives its boundary

- **WHEN** a boundary is removed from the law and the declaration still carries it
- **THEN** the check fails naming the declared boundary the projection does not render, which is the entry a
  one-directional comparison would keep certifying
- **PINNED-BY** `the_law_declares_no_boundary_this_repository_has_not_named`

#### Scenario: The projection cannot be parsed

- **WHEN** the projection renders no boundary section, or a section carries no rule line, or a rule line sits
  under no section
- **THEN** the check refuses as unreadable rather than comparing an empty set, because a projection this
  reader cannot parse is not a law with no boundaries
- **PINNED-BY** `a_projection_this_reader_cannot_parse_is_not_a_law_with_no_boundaries`

### Requirement: The merge wrapper reads what CI said, not only what ran locally

The wrapper standing in front of `gh pr merge` SHALL read the pull request's check conclusions and refuse to
reach the tool unless every check agrees. It SHALL separate four states: a check that disagreed, a check that
has not finished, a check that finished and produced **no evidence**, and a head no workflow has claimed — an
unfinished run is not a failed one, and merging on *not success* would refuse a pull request nobody has
answered yet.

**A check that did not run agreed with nothing.** `NEUTRAL` and `SKIPPED` classified as agreement, beside
`SUCCESS`, with no measurement — while the `EXPECTED` classification was reasoned onto the unfinished side because
*reading it as agreement would merge past a required status that never arrived*. The identical argument
covers a check that did not run, and it is measured on this repository rather than argued from GitHub's
vocabulary: no job in `.github/workflows/ci.yml` carries `if:`, `needs:`, `paths:`, `paths-ignore:` or
`continue-on-error:`, so a skip here cannot mean *legitimately not applicable* — it can only mean the workflow
changed or the run was interfered with. It SHALL therefore be its own refusal rather than folded into the
unfinished one, because the operator action differs: an unfinished check is waited for and a skipped one is
investigated. Where a job legitimately may skip, moving it back to agreement SHALL carry the measurement that
earns it — which job, and why that skip is evidence.

**A refusal SHALL state what to do about the state it met, not what is true of the tree.** This one said
*no job in this repository's workflow carries `if:`, `needs:`, `paths:` or `continue-on-error:`* — true when
written, and false the moment anyone adds one, at which point the wrapper tells an operator something false
about the tree they are standing in. It was justified on the ground that the classification filtered on it as
well. **It did not**: the classification reads a check's conclusion and nothing else, and the wrapper named
the workflow in exactly that one sentence. A claim about the world needs something holding it; a claim about
what to do next does not, and buys the same thing.

Removing it ends a class rather than closing an instance. A reader was built to hold that sentence, and seven
review rounds found five positions in it — two failing open — each of which was a hole in something
load-bearing only because the sentence was load-bearing. The reader SHALL be kept as a **convenience** and
stated as one: it decides **when** an operator learns a job may now skip, not **whether**, since a skipping
job reports `SKIPPED` and the wrapper refuses regardless. Its remaining blind spots SHALL be
recorded at that severity rather than as false negatives — **per mechanism, since the five keys reach the
rollup by two of them**. A job key moves a check's conclusion, so the check appears as `SKIPPED` and the
refusal happens whatever the reader did. A trigger filter stops the workflow running, so its checks are
**absent** from the rollup; that refuses only while one workflow exists, because an empty rollup takes the
*no workflow has claimed this head* arm. A second workflow file makes a missed trigger filter a false
negative again.

**A severity resting on a count SHALL hold that count**, rather than stating the conclusion and leaving the
condition unwritten. The reaction SHALL fail when the workflow directory stops holding exactly one file,
naming what the second file changes and where the severity is stated, so the question is re-priced when it
arises instead of after. **Every key SHALL be read at the position it can occupy**, and there are two classes. `if:`, `needs:` and
`continue-on-error:` sit on a **job**: a `steps:` entry may carry `if:` or `continue-on-error:` without the
job's own conclusion moving, so refusing those would refuse correct code. `paths:` and `paths-ignore:` are
**trigger** conditions and sit under `on:`, quoted or not — YAML 1.1 reads a bare `on` as a boolean, so both
spellings name the block.

**Under `on:` includes the flow form**, where the whole block sits on the key's own line. Entering a block
and reading it SHALL NOT be exclusive: a reader that sets its scope from a top-level key and then moves to the
next line never examines the rest of that line, so `on: {push: {paths: ['src/**']}}` carries a real filter
past a premise that reports itself intact. That direction is **open**, which is the one this requirement
exists to close: the depth and scope defects above each refused too much rather than too little.

**The rule is general and SHALL be applied at every level, which the first statement of it was not.** Saying
it of the top-level key alone left the same open direction one level down — a block-form `on:` whose event is
written in flow form, `push: {branches: [main], paths: […]}`, which is the more ordinary of the two spellings.
The reaction SHALL therefore ask *what does this line open, and what does it still carry* through one
implementation used wherever that question arises, rather than through a branch that happens to have been
corrected.

**A key SHALL be recognised in key position, not as a substring.** The reaction asked that question in three
spellings, and the one that was not positional reacted to a trailing comment: `# no paths: filter here` named
a filter that is a word in a sentence. Splitting a flow body on its separators puts every key at the start of
its own segment, and a block-form line is the degenerate case of the same rule — so one implementation answers
for both forms at both levels, which is what stops a fourth spelling appearing.

**The job side SHALL NOT be given the same treatment**, and the asymmetry is a decision rather than an
oversight. A flow-form `jobs: {alpha: {…}}` leaves no line ending in a colon at the name depth, so no job is
found and the set equality reports them missing — measured, `missing ["examples"]`. Reading that line the same
way would turn a loud failure into a quiet pass unless the flow body were parsed, which is a YAML parser
rather than a line reader. Failing loudly on a shape this repository's workflow does not use is the better of
the two.

Reading the trigger pair at any depth instead SHALL NOT be treated as harmless breadth. It was, justified as
*those two keys have no other meaning anywhere in it* — a claim about one file's current content rather than
about the keys, and the same kind of assumption the indentation rule had just been rewritten to remove.
Measured: a step input named `paths`, the shape several published actions take, made the reaction refuse and
tell a maintainer that a job can now legitimately skip, about an input that moves no job's conclusion. The
reaction fails closed, so the cost is a false refusal rather than a merge — which is why it is scoped rather
than deleted. It SHALL hold the job names it reads against a declared set **in both
directions**, since a read that loses jobs otherwise satisfies *none of them carries a forbidden key* over
whatever it happened to reach. A count of what was read is not sufficient: it catches a reader that found
nothing and not one that found fewer.

**The block's indentation SHALL be read out of the document rather than assumed.** YAML fixes no width — only
consistency within a mapping — so a job whose keys sit deeper than another document's is the same document,
and a reader keyed to one width loses **keys** rather than names: the job is still found, the set equality
still holds, and the forbidden key is never examined. That is the one loss the equality cannot catch, so it is
removed rather than guarded. The reader SHALL be exercised by a fixture over the shapes the tracked workflow
does not currently have, including the two that must **not** react — a `steps:` entry's own `if:`, and a
sequence item written at a job key's depth.

#### Scenario: A second workflow file appears

- **WHEN** `.github/workflows/` holds more than one file
- **THEN** the check fails, because a missed trigger filter stops costing a delay and starts costing a merge:
  a filtered-out workflow contributes nothing to a rollup the others make non-empty and green
- **PINNED-BY** `a_missed_path_filter_costs_a_delay_only_while_one_workflow_exists`

#### Scenario: A job acquires a key that lets it skip

- **WHEN** a job in the workflow carries `if:`, `needs:` or `continue-on-error:`, or the workflow carries a
  path filter
- **THEN** the check fails naming the key and its line, so the decision is made deliberately rather than met
  at a merge — either `SKIPPED` moves back beside agreement with the measurement that earns it, or the key
  goes. The merge is refused either way; what this changes is when the operator finds out
- **PINNED-BY** `no_workflow_job_can_legitimately_skip`

#### Scenario: The workflow is written at a different indentation

- **WHEN** a job's keys, or the job names themselves, sit at a depth other than the one the tracked workflow
  happens to use
- **THEN** the reader still finds them, because it derives both depths from the document — a reader keyed to
  one width loses keys without losing names, which the set equality cannot see
- **PINNED-BY** `the_workflow_reader_decides_every_shape_of_the_block`

#### Scenario: A trigger block is written in flow form

- **WHEN** the trigger block sits on its own key's line — `on: {push: {paths: […]}}` — with or without quotes
  on the key
- **THEN** the filter is still found, because entering the block and reading it are the same line's work; and
  a flow-form list carrying no filter still reacts to nothing
- **PINNED-BY** `the_workflow_reader_decides_every_shape_of_the_block`

#### Scenario: An event under a block-form trigger is written in flow form

- **WHEN** `on:` opens a block and one of its events carries its filter inline — `push: {paths: […]}`
- **THEN** the filter is found, by the same rule the top-level key uses, since the rule is about lines rather
  than about one position in the file
- **PINNED-BY** `the_workflow_reader_decides_every_shape_of_the_block`

#### Scenario: A key is named in a comment

- **WHEN** a trailing comment on a trigger line names one of the keys in prose
- **THEN** nothing reacts, because the key is recognised in key position rather than as a substring
- **PINNED-BY** `the_workflow_reader_decides_every_shape_of_the_block`

#### Scenario: A job body is written in flow form

- **WHEN** a job is written as `alpha: {name: A, if: x}`
- **THEN** no job is read and the set equality names it missing, rather than the key being read out of a body
  the reader cannot parse — the loud failure is chosen over a quiet pass
- **PINNED-BY** `the_workflow_reader_decides_every_shape_of_the_block`

#### Scenario: A key of one class appears where the other class lives

- **WHEN** a step carries an input named `paths`, or a job carries a key the trigger block owns
- **THEN** nothing reacts, because each key is read only where it can change whether a job runs — a reaction
  that refuses a step input names a skip that cannot happen and sends the maintainer to look for it
- **PINNED-BY** `the_workflow_reader_decides_every_shape_of_the_block`

**A fixture standing for a green suite SHALL carry only agreeing conclusions.** The default rollup fixture
carried a `SKIPPED` beside a `SUCCESS`, so every success-path direction over this wrapper asserted the
classification above as an unwritten premise; withdrawing it failed four directions at once, none of them
about CI. A fixture that encodes the property under test makes the suite agree with itself in place of the
subject.

All four SHALL be **derived from one read** of the rollup. Asking a separate filter per state makes the
third unreachable by construction: a pull request with no checks at all produces the empty answer the
disagreement filter gives for *nothing disagreed* and the zero the unfinished filter gives for *nothing is
pending*, so neither refuses and the merge runs — the same false-negative direction this requirement exists
to close, arriving through the guard that closes it. The read SHALL leave the tool's stderr on the terminal
rather than folding it into the value the states are derived from, since a notice on a **successful** call
would otherwise be reported as a check that disagreed.

The refusal SHALL name **which** check disagreed. Both refusals are cannot-judge, exit `2`: a suite this
wrapper could not get agreement from is not a gate that ran and refused.

The Definition of Done is the **local** pre-flight list and CI runs a superset of it, so a green local run is
not a green suite. Measured: a single let-chain the default toolchain accepts and the declared MSRV refuses
was red in CI and green locally through **nineteen consecutive merges**, on a job the local list does not
carry because it installs a toolchain and rebuilds the workspace.

#### Scenario: A check disagreed

- **WHEN** a pull request carries a check whose conclusion is a failure, an error, a cancellation, a timeout,
  or any class this wrapper has not met
- **THEN** the wrapper refuses as a cannot-judge naming that check, and `gh pr merge` is never reached
- **PINNED-BY** `a_pull_request_whose_checks_disagree_stops_before_the_merge`

#### Scenario: A check finished and produced no evidence

- **WHEN** a pull request carries a check whose conclusion is neutral or skipped
- **THEN** the wrapper refuses as a cannot-judge naming that check and saying it produced no evidence — not
  that it disagreed, and not that it has not finished, since the action a skip asks for is to look at why it
  did not run rather than to wait
- **PINNED-BY** `a_check_that_produced_no_evidence_stops_before_the_merge`

#### Scenario: A check has not finished

- **WHEN** a pull request carries a check with no conclusion yet
- **THEN** the wrapper refuses as a cannot-judge saying the run is unfinished rather than that it disagreed
- **PINNED-BY** `a_pull_request_whose_checks_have_not_finished_stops_before_the_merge`

#### Scenario: No workflow has claimed the head

- **WHEN** a pull request carries no checks at all
- **THEN** the wrapper refuses as a cannot-judge saying nothing has checked it, because a pull request nothing
  has checked is not one that checked out
- **PINNED-BY** `a_pull_request_no_workflow_has_claimed_stops_before_the_merge`

#### Scenario: The check rollup cannot be read

- **WHEN** reading the pull request's check conclusions fails
- **THEN** the wrapper refuses as a cannot-judge, because a rollup it could not read is not a suite that
  agreed

The rollup is a **union of two node shapes** and the read SHALL cover both. A check run carries a conclusion
and a name; an external commit status carries a state and a context, and neither of the first two. A read of
one shape answers the empty conclusion for every node of the other, so a **failed** commit status classifies
as unfinished and is reported under a name no check has — the same wrong sentence the stderr rule above
already forbids, arriving through the shape the read never covered. A state with no counterpart among the
conclusions SHALL be classified by what the operator must do about it: a status still **expected** is
unfinished, since it is required and not yet posted, and reading it as agreement would merge past a required
status that never arrived.

#### Scenario: A failed external commit status disagrees

- **WHEN** a pull request carries a commit status whose state is a failure, alongside check runs that passed
- **THEN** the wrapper refuses as a cannot-judge naming that status by its context, rather than classifying it
  as unfinished under a name no check carries
- **PINNED-BY** `a_failed_commit_status_disagrees_rather_than_reading_as_unfinished`

#### Scenario: A commit status still expected has not finished

- **WHEN** a pull request carries a commit status whose state is *expected* — required, and never posted
- **THEN** the wrapper refuses as a cannot-judge saying the run is unfinished, so the operator waits rather
  than hunting a disagreement
- **PINNED-BY** `a_commit_status_still_expected_is_unfinished_rather_than_agreement`

#### Scenario: An admitted flag does not carry a red suite past the wrapper

- **WHEN** the wrapper is invoked with `--admin` on a pull request whose checks disagree
- **THEN** it refuses as a cannot-judge and `gh pr merge` is never reached. `--admin` bypasses required
  **reviews**, which a single-steward repository needs because a pull request's author cannot approve their
  own; it SHALL NOT be read as bypassing this requirement, which runs before the tool. The flag was admitted
  on the ground that whether CI agreed stayed outside this wrapper, and that ground was withdrawn when this
  requirement was written — the arm's own reasoning outlived its premise because no direction observed the
  two together
- **PINNED-BY** `the_admin_flag_does_not_carry_a_red_suite_past_this_wrapper`

### Requirement: A merge records a message about work the pull request carries

The wrapper standing in front of `gh pr merge` SHALL read how many files the pull request changes and refuse
to reach the tool when that count is zero. A count it cannot read SHALL be its own refusal, never treated as
a count of some.

Measured: this wrapper merged a squash whose message asserted seven repairs across five files and whose tree
was byte-identical to its parent's. The content had been committed onto the release branch itself while the
branch the pull request named still pointed at an already-merged commit, so every other guard was satisfied —
the live commit set was non-empty, the message gate judged it against that set, CI was green because nothing
had changed, and the head pin named a real commit. The message is curated separately from the tree and
travels through `argv`, which is what lets the squash message be the record; the pull request's diff is the
only thing tying the two together, and nothing read it.

#### Scenario: The pull request changes no file

- **WHEN** the pull request about to be merged has a changed-file count of zero
- **THEN** the wrapper refuses as a cannot-judge saying the message describes work that is not in it, and
  `gh pr merge` is never reached
- **PINNED-BY** `a_pull_request_that_changes_no_file_stops_before_the_merge`

#### Scenario: The changed-file count cannot be read

- **WHEN** the changed-file count is unreadable or is not a number
- **THEN** the wrapper refuses as a cannot-judge, because a count it cannot read is not a count of zero and
  not a count of some
- **PINNED-BY** `an_unreadable_changed_file_count_stops_before_the_merge`

### Requirement: A test target that spawns a process itself is named

Every test target that spawns a process itself SHALL be named in a declared set together with what it
spawns, and that set SHALL be held against the tree in both directions. *Every* means every test target this
workspace compiles, not one crate's. A path SHALL NOT be declared twice, since a repeat would shrink the compared set and weaken
the comparison without saying so.

The corpus SHALL be every integration test target the workspace compiles, matched by the shape cargo builds
as its own binary. A reaction's reach has three axes — what it looks for, what it counts as a hit, and where
it looks — and this requirement said *every test target* while the corpus was one crate's directory from the
form when the finding happened to sit there. Two axes were closed by construction while the set equality went
on passing over a corpus the requirement does not describe, and four targets outside that directory spawned
processes, two of them running `git` directly.

The detector SHALL recognize the **capability** rather than a spelling of it, and where the spawn goes
through this repository's shared process module the detector SHALL name that **module**, not the functions it
exports. Each narrower form was one spelling short of a requirement that was already correct: a
shared-builder marker, then a literal program name, then a program passed as a value, then a second entry
point added to the same module. The last of those was **live rather than hypothetical** — a test target whose
only spawn went through that entry point was undeclared while both older markers passed over it, and naming
the module found it. A detector keyed on how something is written will keep trailing a requirement about what
is done; spawning a process has one syntactic form and needs no knowledge of the program.

The detector SHALL read executed text, and SHALL recognize the call by position: not preceded by a quote, so
the check does not match its own marker literals, and not preceded by an identifier character, so a
different type's constructor is not read as a spawn.

The purpose recorded beside each path is prose with no producer — a reader's aid for whoever adds the next
one. What this requirement holds is membership.

#### Scenario: A test target gains a spawn

- **WHEN** a tracked test target spawns a process itself and is not named
- **THEN** the check fails naming it
- **PINNED-BY** `no_test_target_spawns_a_process_unnamed`

#### Scenario: A declared target stops spawning

- **WHEN** a named target no longer spawns a process
- **THEN** the check fails, because a name that outlives its reason certifies nothing
- **PINNED-BY** `no_test_target_spawns_a_process_unnamed`

### Requirement: The isolation a builder claims SHALL be decided by a run, not by a list of names

The shared command builder SHALL clear `GIT_CONFIG_PARAMETERS`. It is a channel **parallel** to
`GIT_CONFIG_COUNT`: git parses it independently of the count, so occupying index 0 — which is what closes the
indexed channel and, through it, the ignore channel — does nothing to it. Measured on git 2.53.0 against the
builder's full environment, `'core.excludesFile=/tmp/x'` on that channel makes `config --get core.excludesFile`
answer `/tmp/x` and `status --porcelain --untracked-files=all` stop reporting a file that path excludes — the
read `publish-source-integrity#worktree-is-not-clean` rests on, in front of an act that cannot be undone.

**It is ambient in the ordinary sense, not the hypothetical one.** git exports it itself: measured, a
`pre-commit` hook under `git -c probe.key=SET commit` sees `GIT_CONFIG_PARAMETERS=['probe.key'='SET']` and
sees it unset without the `-c`. A gate run from a hook, an alias, or `bisect run` inherits whatever
configuration that invocation set.

**The reaction SHALL ask the question of a run rather than of a name list.** Three rounds widened this builder
by name — the config files, the repository selectors, the object-directory pair — each after someone measured
the variable, and a list grown that way is as complete as the last person's memory. The reaction SHALL run the
builder in a **child process that inherits an ambient environment**, and classify every configuration git
reports against the origins the builder admits. A channel nobody has named then carries a setting from none of
them, and is named by what arrived rather than by anyone having thought of it. Inheritance SHALL be the
delivery, since setting the variable on the builder's own command would overwrite the removal and test the
case's last statement.

**Every channel the builder clears SHALL be delivered by the reaction, not only named in it.** A member added
to the cleared set with no input constructing it is the same list-grown-by-memory one level up: measured, the
whole suite stayed green with `GIT_CONFIG` deleted from that set, because nothing ever put it in the child's
environment. The reaction SHALL carry each member ambiently, so removing one from the builder makes it appear
in what is classified.

**The observation port SHALL be the whole listing, not one class of origin.** Filtering to command-line
entries reads a channel that **adds** to what git reports and cannot see one that **replaces** it: measured
with `GIT_CONFIG` naming a file, git lists that file alone, command-line entries number zero, and a case
written that way fails on emptiness saying the read is not about the builder — naming neither the setting that
arrived nor where to close it, which is the half of this requirement that does the work. Every line SHALL be
classified against the origins the builder admits: what it wrote, and the repository's own config, which it
does not claim to govern. **Both** admissions SHALL match **exactly** rather than by a fragment the ambient
side chooses, and the exactness SHALL cover the whole entry — its origin, its key and its value — because a
channel that reaches this listing chooses all three. Measured on each in turn: a substring test for the
repository's own config admitted every line of a file `GIT_CONFIG` named `…/foreign.git/config`, that
variable's ordinary use, the ignore setting this surface exists to own included; and a substring test for the
builder's own entry admitted an ambient `core.excludesfile=/dev/null-evil`, whose value merely begins with the
builder's, on the same channel the builder writes to — the case passing green while an unnamed channel set the
one key this whole surface exists to hold. Both are the shape this specification refuses elsewhere for naming
a subcommand by a fragment of a diagnostic string. The builder writes one setting from one constant, so its
admitted entry SHALL be **derived** from that constant rather than spelled again here. The absence of the builder's own setting SHALL be asserted **after** that
classification, so a replacing channel is named by its content rather than by what is missing.

`GIT_CONFIG` SHALL be cleared alongside, for a different reason that SHALL be recorded rather than folded in:
it does not move a judgement's reads — measured, `status --porcelain --untracked-files=all` reports an
excluded file with and without it — but it redirects a **write**, so `git config` in a fixture builder lands
outside the fixture and the commit after it fails for want of an identity. Fail-loud, like the
object-directory pair, and cleared because an `env_remove` costs nothing.

A control SHALL establish that the ambient channel is readable on the machine, so the assertion cannot hold
because the variable was never live. The settings the control and the subject carry SHALL NOT name
`core.excludesFile`: the sibling ignore sweep decides whether a file closed that channel by whether the file
names the setting, so an attack string spelling it would read as this file having neutralised it and would
take the channel-control exception with it — measured.

#### Scenario: A configuration channel nobody named is open

- **WHEN** any environment channel delivers configuration that the builder did not write
- **THEN** the case fails naming the setting that arrived and where to close it, whether or not that channel
  appears in the builder's own list
- **PINNED-BY** `no_ambient_configuration_reaches_a_hermetic_command`

### Requirement: A judgement SHALL close the ambient channel that moves which repository git answers about

The shared command builder SHALL clear `GIT_DIR`, `GIT_WORK_TREE` and `GIT_INDEX_FILE`, so a judgement's reads
are about the repository its working directory names. These are not an ignore channel and the difference is
the severity: an ignore file changes **an answer**, while a repository selector changes **the subject** — the
`HEAD` subject, the worktree's cleanliness and the release tag all come from whatever repository the variable
names, while the act the gate stands in front of packages the directory on disk. The gate and the act are then
about two different trees, before an upload that can be yanked and never replaced.

They SHALL be **cleared** rather than set: git's own default is their absence, so removing them restores
discovery from the working directory, and there is no value meaning *the one this process chose*.

**The set SHALL be what measurement admits rather than every `GIT_*` git defines.** Measured against two
repositories whose `HEAD` subjects and tags differ: with `GIT_DIR` naming the second, `log -1 --format=%s` and
`for-each-ref refs/tags` both answer it; with `GIT_WORK_TREE` naming it, `status --porcelain` reports that
tree against this index. `GIT_NAMESPACE` was measured in the same run and `for-each-ref refs/tags` still
answered this repository, so it is **not** in the set — an entry that closes nothing reads as a defence that
was never there.

**Every variable considered SHALL carry its measurement, admitted or not.** `GIT_OBJECT_DIRECTORY` and
`GIT_ALTERNATE_OBJECT_DIRECTORIES` are outside the set for reasons that differ and are both recorded: measured
against the tag-object read the signature check reconstructs, the first **replaces** the store so this
repository's own tag object goes missing and the command refuses — a gate in front of an irreversible act may
fail closed — while the second **appends** one, so the local object still answers and nothing moves. A
negative measurement kept is what stops the set from growing by resemblance, and what stops the same variable
being re-measured every review.

**The direction's own reach SHALL be stated in this requirement rather than only beside the case.** What a
construction case establishes is that the builder marks the variables for removal, and what it cannot
establish is the composition — that a variable inherited from a real environment is absent in the child —
because constructing that needs this process's environment mutated, which is unsafe in this edition and racy
against a parallel run. That residue belongs here, where the register reads, and not only in the case's
header: the first form of this repair left an unmeasured stop in a private doc comment saying it was filed
when nothing was, which is the defect the paragraph below describes arriving through the repair for it.

**A stop that is not declared is a defect rather than policy, which is how this requirement came to exist.**
The builder left these open and said so in a doc-comment table, with the justification *nothing in this tree
sets them — zero occurrences, repository-wide*. That corpus cannot decide it: the channel is ambient, so the
variable arrives from outside the tree the sweep read — the reader-narrower-than-its-claim shape this
repository spends four rules closing, in the justification for leaving a channel open. No
`openspec/specs/*` scenario carried the stop either, so it reached neither observation register, and a
reader following this repository's own instruction to check the register before reporting a behaviour as a
defect would have found nothing.

#### Scenario: The builder stops clearing a repository selector

- **WHEN** the shared command builder no longer clears `GIT_DIR`, `GIT_WORK_TREE` or `GIT_INDEX_FILE`
- **THEN** the case fails naming the selector, having first demonstrated on an unisolated control that the
  variable does move the subject — so the assertion cannot hold because the key was never readable
- **PINNED-BY** `a_repository_selector_cannot_reach_a_hermetic_command`

### Requirement: A judgement SHALL close the ambient channel of any read whose answer an ignore file changes

A repository check SHALL name `core.excludesFile` as neutralised whenever it runs a git subcommand whose
answer an ignore file **outside** the repository changes. Neutralising the config *files* SHALL NOT be treated
as sufficient: `$XDG_CONFIG_HOME/git/ignore` is the default excludes path git uses when no config file names
one, so emptying the files leaves the default in force. The setting has to be named. Measured both directions
rather than reasoned about — an ignore query answers *ignored*, and a fixture's `add -A` leaves the matching
file untracked, until it is named.

The shared command builder SHALL name it, so the property holds for every caller rather than for whichever
call site was last edited. That the builder did not was found by a read that had been fixed on its own: the
same channel was silently omitting files from fixtures across the crate. The builder's guarantee SHALL be held
by a case comparing a command that closes the channel against one that does not, and the control SHALL leave
the channel open — a control that closed it would compare a value against itself.

**Why this and not process isolation in general.** Bare spawns survive across this crate's test targets and
most of them read a fact no configuration moves. The subcommands this requirement names are the ones whose
answer is ambient — an ignore query directly, and a `status` or `ls-files` asked which files are untracked.
A requirement covering every spawn would name a rule nothing in the tree meets; this one names the half that
can move a verdict, and the tree meets it.

**The direction is what makes it consequential.** For an ignore query, the ambient answer is *ignored*, and
`ignored` means the offence is **not** reported. So the failure mode is an under-refusal whose verdict depends
on who runs the gate — which is what happened: the reference-integrity gate asked `check-ignore` through a
bare spawn on the real repository, so an entry in whoever's personal ignore file quietly excused a stale path
reference. The builder was reachable throughout and nothing required it.

The check SHALL accept **either** closure — the file naming the setting, or the file starting no process of
its own, so that every command it runs is the builder's. Requiring the explicit flag on top of the builder
would refuse correct code and call the redundancy a repair. It SHALL recognize the subcommand as a **complete
argument literal**, because a fragment matched a diagnostic string and two doc comments that run nothing. It
SHALL refuse to report clean when no file in the corpus matches, since a rename that takes the last call site
out of reach otherwise reads as compliance.

Where a file must leave the channel open to pin it, the check SHALL name that file and SHALL hold the
exception against **the direction that earns it**, not against the file's continuing to spawn — measured: a
different test in the same file spawns bare for a different property, and the first form of the guard went on
passing with the control converted away.

What this holds is **file granularity**: the neutraliser in the same file's executed text, not in the same
call. A per-call rule would refuse the one site that was already right, where a single wrapper closes the
channel for every judgement in that file. A subcommand composed at run time is not seen, and
`.git/info/exclude` is inside the repository, so no setting reaches it — the row the publish gate classifies
rather than refuses.

#### Scenario: A judgement asks an ignore question through an unisolated read

- **WHEN** a tracked Rust file's executed text runs a subcommand whose answer an ambient ignore file changes
  through a `Command` it builds itself, and never names `core.excludesFile`
- **THEN** the check fails naming the file, because that verdict depends on who runs it
- **PINNED-BY** `no_judgement_reads_an_ambient_ignore_file`

#### Scenario: The builder stops closing the channel

- **WHEN** the shared command builder no longer names `core.excludesFile`
- **THEN** the case comparing an isolated command against an unisolated control fails, because the two
  answers stop differing
- **PINNED-BY** `an_ignore_file_outside_the_repository_cannot_reach_a_hermetic_command`

#### Scenario: The excused control stops pinning anything

- **WHEN** the direction that earns the named exception is renamed or removed
- **THEN** the check fails rather than going on excusing that file
- **PINNED-BY** `no_judgement_reads_an_ambient_ignore_file`

#### Scenario: The check loses its reach

- **WHEN** no file in the corpus matches any named subcommand
- **THEN** the check fails rather than reporting clean, because the reach was lost and not the risk
- **PINNED-BY** `no_judgement_reads_an_ambient_ignore_file`

### Requirement: A reader SHALL refuse input it cannot understand rather than skip it

A judgement dividing a text into a fixed number of fields SHALL answer *how many arrived* and SHALL refuse a
count it did not ask for. Dropping what it cannot parse and destructuring the survivors states a verdict over
an input the reader never read: measured, `filter_map(|part| part.parse().ok())` over `2028--4-30` yielded
three values from four fields, so a reader asking for three succeeded and the date read as `2028-04-30`.

The division SHALL distinguish a separator whose repetition is a **defect** from whitespace a writer spaces
freely. Collapsing runs is right for the second and wrong for the first, and one rule serving both is what
made a repeated delimiter invisible.

The refusal SHALL be a **cannot-judge** — a field count the reader did not expect is a fact about the input,
not a subject disagreeing with what it is judged against — and SHALL name the count that arrived, since none
and one are different facts. What to write **instead** SHALL come from the caller: a shared reader does not
know the form its caller wanted, and a refusal an operator cannot act on is one they work around.

The two scenarios below are pinned by unit directions in `crates/kanhe/src/tests/`, which is the first time
this capability cites one. The reader is library code under `crates/kanhe/src`, so its matrix sits beside it
as `AGENTS.md`'s *What lives where* requires; the repository check that runs it over the real workflow is the
third scenario's, one layer up.

#### Scenario: A field count the reader did not expect

- **WHEN** a text divides into more or fewer fields than the reader asked for
- **THEN** the reader refuses as a cannot-judge, naming the count that arrived
- **PINNED-BY** `a_field_count_this_reader_did_not_expect_is_refused_either_way`

#### Scenario: A repeated delimiter is a field, not a collapse

- **WHEN** a character separator occurs twice in succession
- **THEN** the empty field between them is counted, so a reader asking for three fields refuses four
- **PINNED-BY** `a_character_separator_keeps_the_empty_field_a_collapsing_reader_would_drop`

#### Scenario: The interpreter support window reads its declaration through it

- **WHEN** the support window's declaration carries one field, or three
- **THEN** the reaction refuses through the shared reader, and adds the form to write
- **PINNED-BY** `the_window_reader_decides_every_shape_of_the_declaration`
