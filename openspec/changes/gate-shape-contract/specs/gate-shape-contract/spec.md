## ADDED Requirements

### Requirement: The gate surface SHALL be enumerated from tracked content

The reaction SHALL derive the surface it judges from `git ls-files 'scripts/check_*.sh'` and SHALL pair each
gate with the twin obtained by substituting `check_` with `test_` in its basename. It SHALL judge **tracked
content**, never the working directory: a filesystem walk makes the verdict depend on local untracked state,
which is the class that made a sibling gate's first version pass locally and fail in CI on three references.

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

Each gate SHALL install the shared backstop from `scripts/lib/exit_contract.sh`, SHALL declare the three-way
contract in its header, and SHALL accept a target directory argument so a fixture can be pointed at it.

The header declaration SHALL be recognized by **shape, not by wording**: a three-way statement whose third
term is cannot-judge, with the verdict words for 0 and 1 left to the gate. The gates word them differently —
"0 clean, 1 violation", "0 coherent, 1 incoherent", "0 publishable, 1 wrong source" — and each names its own
subject better than a shared phrase would. A reaction demanding one literal sentence would report gates as
violating this requirement while every one of them declares its contract: the invented-violation direction,
and the one a capability about gates can least afford.

#### Scenario: A gate omits the shared backstop

- **WHEN** an enumerated gate does not source and invoke `exit_contract_backstop`
- **THEN** the reaction fails, naming the gate, because an unhandled command's status then escapes as a
  foreign exit code the contract does not define

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

Requiring the twins' helper form (`expect_pass` / `expect_fail`) is legitimate where requiring a *product*
test-name convention would not be, and the difference is ownership: these twins are authored in this
repository for this purpose, so this capability may require their shape, exactly as the bound register may
require a scenario heading's form while declining to require a pinning test's name.

#### Scenario: A gate has no twin

- **WHEN** an enumerated gate has no `scripts/test_<name>.sh`
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

Each enumerated gate and its twin SHALL appear in `AGENTS.md`'s Definition of Done block, which is the single
source for the local pre-flight list. A gate present in the tree and absent from that block runs nowhere by
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

#### Scenario: The publish-time gate's absence from the Definition of Done is a stated membership bound

- **WHEN** the reaction reaches `scripts/check_publish_source.sh`, which the Definition of Done deliberately
  omits because it runs at publish time
- **THEN** the reaction accepts its absence as declared policy rather than reporting a violation, a stated
  membership bound, while still requiring its twin's membership
- **PINNED-BY** `the_publish_time_gate_is_exempt_from_dod_membership`

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

Three of the six classes this capability exists for are semantic and SHALL NOT be claimed as observed. They
are declared here rather than implied by the reaction's silence, because a bound a reader cannot see is one
the capability is lying about — and a bound that reads as coverage is worse than an unguarded gap, since it
tells a future auditor a real escape is governed policy.

#### Scenario: Whether an enumeration carries a vacuity guard is not observed — a stated semantic bound

- **WHEN** a gate iterates an enumeration with no guard against zero iterations
- **THEN** the reaction does not claim to observe it, a stated semantic bound, rather than reporting the gate
  conformant on a property it never examined
- **PINNED-BY** `a_missing_vacuity_guard_is_a_stated_semantic_bound`

#### Scenario: Whether a read's status is checked in the parent shell is not observed — a stated semantic bound

- **WHEN** a gate reads a command's output in a subshell or process substitution and never inspects that
  command's status in the parent
- **THEN** the reaction does not claim to observe it, a stated semantic bound; the backstop it does check
  narrows the damage without detecting this shape, which is why both are stated separately
- **PINNED-BY** `an_unchecked_read_status_is_a_stated_semantic_bound`

#### Scenario: Whether a gate's 1-versus-2 assignment is correct is not observed — a stated semantic bound

- **WHEN** a gate reports a genuine violation as cannot-judge, or a misconfiguration as a violation
- **THEN** the reaction does not claim to observe it, a stated semantic bound: it checks that the twin
  asserts codes, never that the codes the gate chose are the right ones, which is the judgment that let a
  `return`-instead-of-`exit` inversion ride green
- **PINNED-BY** `a_wrong_one_versus_two_assignment_is_a_stated_semantic_bound`

#### Scenario: Shell units that are not a gate or its twin are outside the surface — a stated coverage bound

- **WHEN** a shell unit under `scripts/` is neither a `check_*` gate nor its twin — a sourced function
  library, a matrix over one, the example runner, or the publish tool
- **THEN** it is outside this capability's surface, a stated coverage bound, so the projection's conformance
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
