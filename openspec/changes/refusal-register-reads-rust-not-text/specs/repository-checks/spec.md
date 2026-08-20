## MODIFIED Requirements

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
