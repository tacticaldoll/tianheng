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
  requirement forbids;
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

#### Scenario: Whether a mention compiles anything is not observed — a stated bound

- **WHEN** the contract names a promised member only in a comment
- **THEN** the check counts it as named, a stated bound: deciding that a mention is load-bearing is a
  judgement over text, the instrument this repository has designed, measured and rejected, and what makes a
  mention bite is the compiler rather than this check. A comment-only mention still fails the reviewer reading
  the diff, which is the layer that owns it
- **PINNED-BY** `a_member_named_only_in_a_comment_is_counted_as_named`

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
