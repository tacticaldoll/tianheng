# gate-shape-contract (delta)

## MODIFIED Requirements

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
- **THEN** the reaction does not claim to observe it, a stated bound: it checks that the twin asserts codes,
  never that the codes the gate chose are the right ones, which needs each gate's meaning rather than its text.
  **Narrowed** from a wider claim that named this as what let a `return`-instead-of-`exit` inversion ride green.
  That was wrong about its own history: the inversion produced both directions above in one gate, and what let it
  pass CI was the matrix asserting a non-zero status rather than a code — which the `exit codes` property now
  refuses, and whose remedy cites this instance. What remains is only the semantic judgment
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
