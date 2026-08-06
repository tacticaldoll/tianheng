# gate-shape-contract Specification

## Purpose

Make the structural contract every repository gate and its failure matrix hold into a reaction rather than a
convention: the surface enumerated from tracked content, the mechanically checkable properties asserted over
it, the properties deliberately not asserted declared as observation bounds, and a generated projection that
keeps the result from rotting into prose.
## Requirements
### Requirement: The gate surface SHALL be enumerated from tracked content

The reaction SHALL derive the surface it judges from `git ls-files` under `scripts/`, taking every tracked
shell unit whose basename begins with `check_`, and SHALL pair each gate with the twin obtained by substituting
`check_` with `test_` in that basename. It SHALL judge **tracked content**, never the working directory: a
filesystem walk makes the verdict depend on local untracked state, which is the class that made a sibling
gate's first version pass locally and fail in CI on three references.

The selection SHALL be made on the enumerated basenames rather than by a `check_*` pathspec. Git matches
pathspec wildcards without `FNM_PATHNAME`, so `scripts/check_*.sh` already reaches into subdirectories: the
glob would be describing something other than what it appears to say, and a reader auditing the surface would
be reading a rule that does not hold.

An enumeration that yields no gate SHALL fail loudly rather than report every property of zero gates
satisfied. This is the vacuity direction, and it is the one this repository has re-opened most often — six
occurrences in one window — so it is a requirement here rather than an implementation detail.

#### Scenario: The surface is read from tracked content

- **WHEN** the reaction runs in a checkout
- **THEN** it judges exactly the gates `git ls-files` reports, so an untracked draft gate in the working
  directory is neither judged nor able to change the verdict

#### Scenario: A new gate is judged the moment it is tracked

- **WHEN** a gate is added under `scripts/check_*.sh` and tracked
- **THEN** it enters the surface with no edit to the reaction or to any list, and its missing properties are
  named individually

#### Scenario: An empty enumeration fails rather than reporting clean

- **WHEN** the enumeration yields zero gates
- **THEN** the reaction fails, saying the surface was empty, because every property of zero gates holds and
  reporting that as conformance is the silent pass this capability exists to refuse

### Requirement: Every enumerated gate SHALL hold the family's exit contract in a checkable form

Each gate SHALL install the shared backstop from `scripts/lib/exit_contract.sh`, SHALL pass it a label that is
the gate's own name, SHALL declare the three-way contract in its header, and SHALL accept a target directory
argument so a fixture can be pointed at it.

The header declaration SHALL be recognized by **shape, not by wording**: a three-way statement whose third
term is cannot-judge, with the verdict words for 0 and 1 left to the gate. The gates word them differently —
"0 clean, 1 violation", "0 coherent, 1 incoherent", "0 publishable, 1 wrong source" — and each names its own
subject better than a shared phrase would. A reaction demanding one literal sentence would report gates as
violating this requirement while every one of them declares its contract: the invented-violation direction,
and the one a capability about gates can least afford.

The label SHALL be **derived from the gate's basename** rather than compared against a kept table: `check_` and
`.sh` removed, underscores read as spaces, so `scripts/check_bound_register.sh` names itself `bound register`. A
table would be a second declaration of the gate's name and would rot exactly as the thing it checks.

That label is the gate's self-identification in the one diagnostic a reader gets when the shell aborts a gate
instead of the gate refusing. The same diagnostic prints `${BASH_SOURCE[0]}` and `$LINENO`, which expand in the
failing gate's own frame, so a wrong label does not lose the location — it **contradicts** it, and a
contradiction is read in whichever direction the reader trusts first. The hazard is copy-paste, which is how
this surface came to exist: six gates carrying one shape, each written by reading a sibling.

The label SHALL be written as a **literal**, and a gate whose label is built by expansion SHALL be refused with
that as the stated reason rather than as a mismatch. The reaction reads a gate's text and does not evaluate it,
so it cannot confirm a computed label; reporting an unconfirmed label as correct is the direction this family
refuses, and reporting it as "you wrote X, the basename asks for Y" would be a false statement about a gate that
wrote neither.

Requiring a literal is a requirement on **authored form**, legitimate here by the same ownership argument that
lets this capability require the twins' helper names: these gates are authored in this repository for this
purpose. It is worth stating plainly that the form being required is not the better one — a gate deriving its own
label from `$0` could not disagree with its filename at all, where a literal can. The literal is required because
the property has to be checkable by reading, and a rule admitting derivations would be a rule about which
spellings of a derivation the reaction recognizes.

#### Scenario: A gate omits the shared backstop

- **WHEN** an enumerated gate does not source and invoke `exit_contract_backstop`
- **THEN** the reaction fails, naming the gate, because an unhandled command's status then escapes as a
  foreign exit code the contract does not define

#### Scenario: A gate's backstop label is not its own name

- **WHEN** an enumerated gate passes `exit_contract_backstop` a label that is not its basename with `check_`
  and `.sh` removed and underscores read as spaces
- **THEN** the reaction fails, naming the gate, the label it wrote and the label its basename asks for, because
  the reader who trips this has by construction just copied a sibling and is looking at neither name

#### Scenario: A gate's backstop label is not a literal

- **WHEN** an enumerated gate builds its label by expansion rather than writing it
- **THEN** the reaction fails, saying the label could not be read as a literal, because a reaction that reads
  text cannot confirm a computed label and must not report an unconfirmed one as correct

#### Scenario: A gate that installs no backstop has no label to check

- **WHEN** an enumerated gate does not invoke `exit_contract_backstop` at all
- **THEN** the reaction reports both the missing installation and the missing label, each naming a real absence,
  exactly as an absent twin reports the matrix properties it cannot hold

#### Scenario: A gate's header declares the contract in its own verdict words

- **WHEN** an enumerated gate's header states a three-way contract ending in cannot-judge, using verdict
  words of its own for 0 and 1
- **THEN** the reaction accepts it, because the property is that the contract is declared, not that it is
  declared in one sentence

#### Scenario: A gate cannot be pointed at a fixture

- **WHEN** an enumerated gate takes no target directory argument
- **THEN** the reaction fails, naming the gate, because a gate that only ever judges its own checkout cannot
  be observed refusing, and a guard is not a guard until it has been seen to fail

### Requirement: Every enumerated gate SHALL have a companion failure matrix holding five properties

Each gate SHALL have a twin, and each twin SHALL: assert expected exit **codes** rather than merely non-zero;
hold at least one passing direction and at least one refusing direction; assert that the gate left the
repository it judged unchanged; and assert that a clean run prints nothing on stderr.

Each is a class this window observed, not a checklist assembled for symmetry. Asserting non-zero rather than
the code let a genuine incoherence collapse from 1 into 2 and ride green through CI. A matrix with no passing
direction cannot distinguish a working gate from one that refuses everything. The silent-clean-run assertion
is the only one that catches the shape where the shared backstop printed cannot-judge once per clean file
while the exit code stayed 0 — invisible to every check reading only the code.

Every property over a twin or a gate SHALL be read from the **region of the text it is about**: executed text for a
property about what runs, the header for a property about what a file declares of itself, prose for a property about
what a reader is sent to. Two properties once read the whole file while their own helper's documentation said they
were about executed text, so `expected_status` in a header comment satisfied a property about an assertion — the
wrong check rather than a loose one.

Requiring the twins' helper form (`expect_pass` / `expect_fail`) is legitimate where requiring a *product*
test-name convention would not be, and the difference is ownership: these twins are authored in this
repository for this purpose, so this capability may require their shape, exactly as the bound register may
require a scenario heading's form while declining to require a pinning test's name.

Two of the five are recognized through an authored form on that same argument, and the forms are named here so
an author meets the requirement rather than discovering it as an invented violation. The silent-clean-run
assertion SHALL capture stderr alone — a redirection of the form `2>&1 >/dev/null` — and SHALL test the
variable that capture assigned for emptiness. The unchanged-repository assertion SHALL name itself in its
refusal, saying the gate `mutated` what it judged; the comparison behind it cannot be recognized mechanically,
since the twins compare a porcelain listing, a `HEAD`, a tag list and a directory walk in four combinations.

#### Scenario: A gate has no twin

- **WHEN** an enumerated gate has no companion twin beside it, named by substituting `test_` for `check_`
- **THEN** the reaction fails, naming the gate, because a gate nobody has watched refuse is protection
  claimed rather than protection observed

#### Scenario: A twin asserts non-zero instead of the code

- **WHEN** a twin's refusing directions assert only that the gate exited non-zero
- **THEN** the reaction fails, naming the twin, because a violation reported as cannot-judge — or the reverse
  — is then indistinguishable from the verdict the gate owed

#### Scenario: A twin has no passing direction

- **WHEN** a twin holds refusing directions only
- **THEN** the reaction fails, naming the twin, because a gate that refuses everything satisfies such a
  matrix completely

#### Scenario: A twin does not assert a silent clean run

- **WHEN** a twin never asserts that a clean run's stderr is empty
- **THEN** the reaction fails, naming the twin, because a gate can print cannot-judge on every clean input
  while exiting 0, and no assertion on the code can see it

#### Scenario: A twin does not assert the gate is read-only

- **WHEN** a twin never asserts the judged repository is unchanged after the gate runs
- **THEN** the reaction fails, naming the twin, because a gate that edits what it judges makes its own next
  verdict unreproducible

### Requirement: Both files SHALL be reachable from the Definition of Done, except the publish-time gate

Each enumerated gate and its twin SHALL be **invoked** by `AGENTS.md`'s Definition of Done block — appearing in
command position, not merely mentioned in it. That block is the single source for the local pre-flight list.

Mentioned-is-not-invoked is a measured defect, not a hypothetical: `test -f scripts/check_whitespace_hygiene.sh` in
the block satisfied the membership check while executing nothing. A gate the block names and never runs is the
"matrix present but unrun" class wearing the appearance of coverage, which is worse than the absence, because the
projection then reports it reachable. A gate present in the tree and absent from that block runs nowhere by
default, which is the "matrix present but unrun" class this window recorded three times.

`scripts/check_publish_source.sh` is exempt from the gate half of this requirement and SHALL be declared as
such: it runs from `scripts/publish.sh` at publish time, because no development checkout is a release
snapshot. Its twin is in the block; the gate is not.

The exemption SHALL be checked **live**, not merely honoured. A hand-written exception that has stopped
applying is an exception that rots silently, and this one rots in the flattering direction: were the
publish-time gate ever added to the Definition of Done, an exemption that only ever *permits* would keep
permitting, and the next reader would inherit a licence with no live instance behind it.

#### Scenario: A gate or twin is absent from the Definition of Done

- **WHEN** an enumerated gate or its twin does not appear in the Definition of Done block
- **THEN** the reaction fails, naming the file, because a gate nothing invokes is a comment

#### Scenario: The publish-time gate's absence from the Definition of Done is excused by name

The exemption is deliberately **not** declared through the observation-bound mechanism. A bound says a reaction
stops at a shape; this says one named instance is excused from a requirement. Declaring it as a bound would put
something that is not an observation limit into the register whose leading figure counts exactly those — and
`observation-bound-model` would then demand a typed extent for it, which no value in that model honestly fits.

- **WHEN** the reaction reaches `scripts/check_publish_source.sh`, which the Definition of Done deliberately
  omits because it runs at publish time
- **THEN** the reaction accepts its absence as declared policy rather than reporting a violation, while still
  requiring its twin's membership

#### Scenario: The membership exemption has stopped applying

- **WHEN** `scripts/check_publish_source.sh` appears in the Definition of Done block
- **THEN** the reaction fails, saying the exemption is stale and must be retired, because an exception with
  no live instance behind it reads as licence to the next author

### Requirement: The contract SHALL be projected into a generated, staleness-checked document

The reaction SHALL emit a projection of the surface and its conformance, blessed by an environment variable
and diffed on every run, exactly as `AGENTS.self-law.md` and `docs/observation-bounds.md` are. A
hand-maintained table of this shape is the drift class this repository has closed twice; the projection is
what stops the capability's own description of the surface from rotting.

The projection SHALL state what it does not claim, in its own header rather than only in the reaction's
comments. A projection implying completeness would mislead exactly where it is most trusted.

#### Scenario: The projection is stale

- **WHEN** the surface or a gate's conformance changes and the projection is not regenerated
- **THEN** the reaction fails and names the blessing command, so the document cannot drift from what was
  measured

#### Scenario: The projection names the properties it does not check

- **WHEN** a reader opens the projection
- **THEN** its header enumerates the semantic properties declared as bounds below, so a reader can see what
  conformance in this document does and does not mean

### Requirement: Observation bounds

Each bound declared here SHALL also carry a **typed declaration** classifying where its measure stops, keyed on
its derived id, per `observation-bound-model`. That capability landed after this one was proposed and its
bijection refuses an unclassified bound, so the obligation is stated here rather than discovered at sync.

Three of the six classes this capability exists for are semantic and SHALL NOT be claimed as observed. They
are declared here rather than implied by the reaction's silence, because a bound a reader cannot see is one
the capability is lying about — and a bound that reads as coverage is worse than an unguarded gap, since it
tells a future auditor a real escape is governed policy.

A bound SHALL be **narrowed** when a reaction begins to reach part of it, and its heading SHALL NOT move when that
happens. The heading's slug is the bound's id, so renaming it would break the citation and the typed declaration in
one edit and move a row in two projections for a reason unrelated to the bound's content. A bound that overstates
what is unobserved misleads in the same way as one that understates it: it tells an auditor a real check does not
exist.

#### Scenario: Whether an enumeration carries a vacuity guard is not observed — a stated bound

- **WHEN** a gate iterates an enumeration with no guard against zero iterations
- **THEN** the reaction does not claim to observe it, a stated bound, rather than reporting the gate
  conformant on a property it never examined
- **PINNED-BY** `a_missing_vacuity_guard_is_a_stated_semantic_bound`

#### Scenario: Whether a read's status is checked in the parent shell is not observed — a stated bound

- **WHEN** a gate reads a command's output through a **command substitution** whose status nobody inspects, or
  through a pipeline whose non-final stage fails, and never inspects that status in the parent
- **THEN** the reaction does not claim to observe it, a stated bound. This is what remains after the
  process-substitution property above: that construct **is** now observed, so the bound is narrowed to the shapes
  whose detection would need control flow rather than text — whether a caller inspects `$?` after a `$(…)` is not a
  property of the source. The backstop the reaction also checks narrows the damage without detecting either shape
- **PINNED-BY** `an_unchecked_read_status_is_a_stated_semantic_bound`

#### Scenario: Whether a gate's 1-versus-2 assignment is correct is not observed — a stated bound

- **WHEN** a gate reports a genuine violation as cannot-judge, or a misconfiguration as a violation
- **THEN** the reaction does not claim to observe it, a stated bound: it checks that the twin
  asserts codes, never that the codes the gate chose are the right ones, which is the judgment that let a
  `return`-instead-of-`exit` inversion ride green
- **PINNED-BY** `a_wrong_one_versus_two_assignment_is_a_stated_semantic_bound`

#### Scenario: Shell units that are not a gate or its twin are outside the surface — a stated bound

- **WHEN** a shell unit under `scripts/` is neither a `check_*` gate nor its twin — a sourced function
  library, a matrix over one, the example runner, or the publish tool
- **THEN** it is outside this capability's surface, a stated bound, so the projection's conformance
  covers the gate surface and not everything under `scripts/`
- **PINNED-BY** `units_outside_the_gate_pairing_are_outside_the_surface`

#### Scenario: An excluded unit carries the gate contract

The exclusion is by *naming*, so it must not become a place a gate can hide. A unit outside the pairing that
installs the shared backstop is a gate wearing another name.

- **WHEN** a unit outside the gate-and-twin pairing installs `exit_contract_backstop` — the library that
  *defines* it excepted
- **THEN** the reaction fails, naming the unit, because the surface would otherwise be evaded by a rename
  rather than argued as a spec change

### Requirement: The reaction SHALL refuse to skip silently in CI

The reaction SHALL follow the repository's established discipline for a test that reads repository paths:
outside a checkout it returns without asserting, and when `TIANHENG_WORKSPACE_TESTS` is set an absent layout
SHALL be a loud failure. A governance reaction that quietly does nothing in CI is the shape the whole
capability argues against.

#### Scenario: The layout is absent while the workspace-tests marker is set

- **WHEN** the reaction cannot locate the repository layout and `TIANHENG_WORKSPACE_TESTS` is set
- **THEN** it fails loudly, naming what was expected, rather than returning as it would outside a checkout

### Requirement: A gate SHALL NOT consume a fallible observation source through process substitution

Each gate SHALL read an observation source by materializing it, checking the producer's status in the **parent
shell**, and only then consuming it. A `while … done < <(producer)` SHALL be refused when its producer can fail,
because the status of a process substitution never reaches the parent — so a producer that emits some rows and then
fails leaves the gate judging a partial read.

Both directions of that failure are measured. A `git ls-files --eol` truncated after one clean row made a gate report
`whitespace hygiene ok (1 tracked text files)` at **exit 0** over a repository it had read one file of. A `git log`
truncated after one release record made another gate conclude snapshot state and report `[Unreleased] must be empty`
at **exit 1** — a violation invented from a partial read. A vacuity guard reaches neither: it was built for zero rows
and a partial read gives one or more.

A producer that is a **shell builtin over data already in memory** — `printf` or `echo` re-splitting a variable —
SHALL be permitted, having no I/O to fail at. The permission SHALL be granted by naming the builtin rather than by
listing the call sites, because a list of sites rots on the next edit and would make the property about where code
is rather than what it does.

#### Scenario: A gate consumes a fallible producer through process substitution

- **WHEN** an enumerated gate contains `done < <(git ls-files)`, or any process substitution whose producer is not a
  shell builtin, in executed text
- **THEN** the reaction fails, naming the gate, because that producer's failure cannot reach the parent and a
  partial read would be judged as a whole one

#### Scenario: A builtin re-splitting a held variable is permitted

- **WHEN** the producer is `printf` or `echo` over a variable already in memory
- **THEN** the reaction accepts it, because there is no I/O to fail at and requiring a temporary file would make the
  gate longer without making it safer
