## MODIFIED Requirements

### Requirement: An observation bound is declared as a scenario that names itself one

An **observation bound** SHALL be declared as a `#### Scenario:` whose heading marks it as a bound, in
the spec of the capability whose reaction it bounds — a bound being a claim that an observation
deliberately stops at a named shape, so that shape is governed policy rather than a defect. The
declaring file SHALL be `openspec/specs/<capability>/spec.md`.

A bound MAY also be declared for a **classification a reaction does not make**, not only for a shape it does
not observe. Where two populations are told apart by a judgement rather than by a rule, the judgement is
carried by position and the absence of the rule is what gets declared: otherwise a reader takes the split for
something a run enforces.

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

#### Scenario: Which member holds a reaction is a judgement — a stated bound

- **WHEN** a reaction is added under `crates/shengmo/` or `crates/kanhe/`
- **THEN** nothing observes whether it landed in the right one. The split is by what a reaction judges — the
  law and the delivered product on one side, this repository's record on the other — and two mechanical rules
  were each measured unreliable: a text scan reads a comment naming `AGENTS.md` as governance while a reaction
  scanning every tracked file names nothing, and the workspace marker means both "this needs the repository as
  its subject" and "this needs a fixture". Position is the declaration; the join below catches a **capability**
  named wrongly, never a member chosen wrongly
- **UNPINNED** `BACKLOG.md` — *which governance member a reaction belongs to is unobserved*
