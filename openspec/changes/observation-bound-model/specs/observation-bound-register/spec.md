## MODIFIED Requirements

### Requirement: An observation bound is declared as a scenario that names itself one

An **observation bound** SHALL be declared as a `#### Scenario:` whose heading marks it as a bound, in
the spec of the capability whose reaction it bounds — a bound being a claim that an observation
deliberately stops at a named shape, so that shape is governed policy rather than a defect. The
declaring file SHALL be `openspec/specs/<capability>/spec.md`.

The marking SHALL carry **no qualifier**. The recognizer previously admitted an optional free word before
"bound", and that slot accumulated many phrasings with no vocabulary governing any of them — one of them,
`cfg-blind`, used by two capabilities for bounds on **opposite sides** of the false-negative line, where the
direction is the whole content. A qualifier therefore read as a classification while classifying nothing. What
kind of stop a bound describes SHALL instead be carried by its typed declaration below, where the value set is
closed and a contradiction is a compile error. A heading carrying a qualifier SHALL fail, naming the heading and
the repair.

The two marker words SHALL remain interchangeable. They carry no information — some specs use both bare forms
internally — but they mislead no reader, where a qualifier did; and each removal changes the bound's derived id,
so a sweep is charged against every reference to it. Closing the harmful half of the slot rather than all of it
is a deliberate limit on that churn, not an oversight.

The **prose** recognizer SHALL remain permissive, admitting the qualified forms this requirement forbids in a
heading. It is the register's detection floor rather than a requirement on authored form: narrowing it would
stop it reporting a bound stated in prose with a qualifier, which is the direction that stops the register being
completed by declaring only the convenient bounds. Requiring an authored form and narrowing detection are
opposite acts, and only the first is legitimate here.

The declaration SHALL sit under the requirement it qualifies, wherever that is, and SHALL NOT be hoisted
into a common section. Nearly every bound declared today sits under the requirement it qualifies rather
than under an `Observation bounds` requirement, and moving them would separate each bound from the
reaction it limits — the `Observation bounds` requirement three specs carry is a place bounds are
gathered, never the definition of one.

Requiring the heading convention is legitimate where requiring a test-name convention is not, and the
difference is ownership: a scenario heading is authored in the spec, so the register may require its
form, while a test name pre-exists the register and is owned by its suite. A bound whose heading omits
the marking SHALL be caught by the undeclared-prose reaction below rather than silently missed.

A parallel block form SHALL NOT be introduced, because for a bound already declared as a scenario it
would state the same bound twice, which is the drift the register exists to end.

A bound's identity SHALL be derived from its location as `<capability>/<scenario-slug>`, never allocated,
so no identifier ledger is introduced and a citation cannot outlive the declaration it names. That derived id
SHALL also be the key its typed declaration carries, so the two sides bind with no second naming scheme.

A declared bound SHALL carry, beside its citation, a **typed declaration** in the owning dimension's library,
keyed on its derived id and classified under `observation-bound-model`. The scenario states the bound for a
reader; the declaration states what kind of stop it is for a reaction. Neither alone is the declaration: a
scenario without a typed declaration is an unclassified claim, and a declaration without a scenario is a
classification no spec reader can find. `observation-bound-model` owns holding the two sets equal.

#### Scenario: A bound is declared beside the requirement it qualifies

- **WHEN** a capability states that its observation stops at a named shape
- **THEN** that statement appears as a bound-marked scenario under the requirement it qualifies, carrying
  its own WHEN/THEN, and no second declaration of the same bound exists elsewhere in the spec

#### Scenario: A bound-marked scenario is recognized wherever it sits

- **WHEN** a bound-marked scenario sits under a requirement that is not named `Observation bounds`
- **THEN** the reaction reads it as a declared bound and requires its citation, so the register covers
  every bound already declared that way without relocating any of them

#### Scenario: A heading carries a qualifier before the marker

- **WHEN** a bound scenario's heading marks itself with a qualified phrase
- **THEN** the reaction fails, naming that heading and the repair, rather than declining to recognize it as a
  bound at all — an unrecognized heading would fall through to the undeclared-prose direction and be reported
  as something else, so the qualified form is refused explicitly

#### Scenario: Prose stating a bound with a qualifier is still detected

- **WHEN** a spec states a bound in prose using a qualified phrase, outside any declared bound scenario
- **THEN** the undeclared-prose reaction still reports it, because that recognizer is a detection floor and
  narrowing it in step with the heading requirement would hide exactly the bounds an author did not declare

#### Scenario: A bound's id is derived rather than assigned

- **WHEN** a declared bound is cited from a diagnostic, another spec, or `BACKLOG.md`
- **THEN** the citation is `<capability>/<scenario-slug>`, requiring no lookup table and no allocation step

#### Scenario: The declaration does not disturb spec validation

- **WHEN** `openspec validate --specs --strict` runs over the specs carrying declared bounds and their
  citation bullets
- **THEN** every spec validates, so the register's syntax costs no schema change
