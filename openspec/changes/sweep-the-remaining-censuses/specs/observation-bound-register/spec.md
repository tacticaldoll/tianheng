# observation-bound-register (delta)

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
reaction it limits — the `Observation bounds` requirement some specs carry is a place bounds are
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

### Requirement: A bound stated in prose but not declared as a scenario SHALL fail

The reaction SHALL scan `openspec/specs/*` for bound-declaring prose and SHALL fail on any occurrence
outside a declared bound scenario, **subject to the exemptions and residuals stated below, which SHALL be
enumerated rather than implied**. This makes the prose already present the register's mandatory minimum, so
the register cannot be completed by declaring only the convenient bounds. Its size is measured rather than
estimated by whoever wrote it: the reaction prints what it counted on every clean run, and a figure typed
here would be a census in prose — the class `AGENTS.md` forbids, and one this sentence has already demonstrated,
having had its denominator re-swept while its numerator was left behind.

One **exemption** is deliberate and SHALL be declared here rather than only in the reaction's own comments.
Prose under a requirement whose heading names bounds is not reported, because several such requirements
state their bounds as a numbered list — `Observation bounds are stated, not silent` enumerates seven — and
requiring each item to become its own scenario would restructure three requirements and read worse. The
exemption is not free, and its price SHALL be charged: such a requirement SHALL declare at least one bound
scenario, or its list would have no reaction anywhere. What the exemption costs is that the *other* items of
such a list are unregistered, and that cost SHALL be stated in the projection's header.

The direction SHALL be described as a **floor and not a proof**, in the generated projection's own header,
and every residual known to the reaction SHALL appear there. Three are known and SHALL be named:

1. A bound worded outside the scanned pattern — "out-of-scope", "does not claim to observe" — is
   undetectable.
2. The scan is **line-oriented**, so a statement whose bound names continue onto the next line is examined
   only on the line carrying the trigger words.
3. A `(bound: …)` reference clears the prose it sits with **regardless of how many bounds that prose
   states**, and regardless of whether the referenced bound is one of them.

Residual 3 is the mechanism that let a retired `#[path]` bound survive in a capability's overview paragraph
through two sweeps, so it SHALL be recorded as the reason rather than as a curiosity. Closing it would
require reading which bounds a sentence lists, which is a semantic judgment no reaction can reach; residual
2's obvious repair — scanning paragraphs rather than lines — SHALL NOT be adopted on that account, because
it was measured against this defect and **would not have caught it**: the paragraph carries the reference
that clears it, so the repair costs twelve new offenses and buys nothing against the failure that motivated
it.

These residuals SHALL NOT be declared as bounds of this capability, for the reason already settled for the
first: nothing observes them, and a declaration no reaction can reach is the name-without-a-reaction
`PROJECT.md` forbids. The register must not make itself the exception.

#### Scenario: Spec prose states a bound that no scenario declares

- **WHEN** a spec paragraph or a bare THEN clause states that an observation stops at a shape, and no
  bound scenario declares it
- **THEN** the reaction fails, naming the file and the occurrence

#### Scenario: The same statement inside a declared bound scenario does not fail

- **WHEN** the bound-declaring prose sits inside a declared bound scenario
- **THEN** the reaction passes for that occurrence, so declaring the bound is what clears it rather than
  rewording the sentence

#### Scenario: Prose under a bounds-named requirement is exempt, and the requirement pays for it

- **WHEN** a requirement whose heading names bounds states one in prose
- **THEN** the occurrence is not reported, and the reaction instead requires that requirement to declare at
  least one bound scenario, failing when it declares none

#### Scenario: The register states every residual of its prose direction

- **WHEN** the projection is read
- **THEN** its header names all three residuals — unrecognized wording, the line-oriented scan, and a
  reference clearing prose that states more bounds than it names — and no bound of this capability claims
  any of them, since no reaction could reach one
