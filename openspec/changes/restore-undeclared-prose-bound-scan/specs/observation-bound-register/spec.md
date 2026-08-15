## MODIFIED Requirements

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
