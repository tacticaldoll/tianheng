## ADDED Requirements

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
`to` substring. It SHALL be applied to a tree built from **tracked content**, never to the working directory,
which is the same rule every gate in this family holds and here also keeps an interrupted run from having
edited the author's files.

The reaction SHALL build that tree with its **own target directory**. Reusing the repository's reports every
pin as biting: cargo resolves the fingerprint against the sources the artifacts were first built from, so a
mutated scratch tree runs a binary compiled from unmutated code and reports `Finished` in hundredths of a
second. That is stated as a requirement rather than left to the implementation because it is the exact failure
this requirement exists to end, arriving through the reaction meant to end it.

A `from` that occurs zero times or more than once in the named file SHALL be **cannot judge**, not a violation.
The mutation could not be applied, which is a different fact from the pin not biting, and reporting the second
for the first lets a mutation whose anchor has rotted read as a pin that has been exercised. Requiring the
anchor to be unique is the rule the observer protocol's body reader reached by the expensive route in the same
window: an anchor matching twice names a set rather than a site.

Mutation records and pinning citations SHALL be held against each other in **both** directions. A mutation
naming a test no declared bound cites SHALL fail: it perturbs something this register makes no claim about, and
its passing would read as coverage of a citation that does not exist.

Coverage SHALL be partial and SHALL say so. A clean run SHALL print how many citations carry no declared
mutation, in the same shape the register already prints its figures and its projection already leads with the
unpinned count. A gate that reported only the mutations it ran would be a reaction reading as coverage, which
is this requirement's own subject one level up.

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

#### Scenario: The artifacts are reused from the unmutated tree

- **WHEN** the mutated tree is built against a target directory whose artifacts were produced from other
  sources
- **THEN** the reaction fails to judge rather than reading the resulting pass as a biting pin — the arrangement
  is measured to report every pin as biting, so the isolation is a stated property and not an implementation
  preference

#### Scenario: The uncovered remainder is disclosed on a clean run

- **WHEN** every declared mutation kills its citation
- **THEN** the reaction still prints how many citations carry no mutation, so a clean result cannot be read as
  every pin having been exercised

#### Scenario: The mutation set is empty

- **WHEN** no mutation is declared at all
- **THEN** the reaction fails, saying the set was empty, because every property of zero mutations holds and
  reporting that as conformance is the vacuity direction this repository has re-opened most often
