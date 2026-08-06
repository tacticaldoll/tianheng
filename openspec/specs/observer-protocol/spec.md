# observer-protocol Specification

## Purpose

Make declaring what a reaction does not observe a condition of taking part: a fixed lifecycle whose every method
has no default body, and an eager fold that composes participants into one verdict while preserving the family's
cannot-judge-supersedes invariant.
## Requirements
### Requirement: An observer SHALL declare what it does not observe in order to join a run

The `Observer` trait SHALL carry a method returning the observation bounds its reaction declares, and that
method SHALL have **no default body**. A participant cannot be composed into a run without answering it.

This is the whole point of the protocol rather than a convenience on it. 天衡's promise is honesty about what a
reaction does not see; before this, that honesty was a convention the family kept about itself, and a
third-party observer could have joined — had a seam existed — while saying nothing about its limits.

No method on the trait SHALL carry a default body. Adding a **stage** therefore breaks every implementor, family
and third-party alike, which is the direction that must break: a declaration written before the question existed
has not answered it. Adding an **answer** to an existing question SHALL NOT break an implementor, and that is
what `#[non_exhaustive]` on the extent enums already provides.

#### Scenario: An observer omits its bounds declaration

- **WHEN** a type is written to implement `Observer` without answering the bounds method
- **THEN** it does not compile, so it cannot be composed into a run

#### Scenario: A new stage is added to the lifecycle

- **WHEN** the protocol gains a further method with no default
- **THEN** every existing implementor fails to compile until it answers, because a stage nobody re-examined is
  a question the older declarations never addressed

#### Scenario: A new extent value is added

- **WHEN** the bound model gains a value
- **THEN** no implementor breaks, because an existing bound's answer to an existing question is still valid

### Requirement: The lifecycle SHALL be the seam each dimension actually has

The trait SHALL ask an observer to observe a workspace and return one outcome, and to declare its bounds. It
SHALL NOT split observation into separate corpus, fact and reaction stages, and SHALL NOT ask an observer to
restate its boundary kind.

A third method — "identify your boundary kind" — was designed and dropped because **nothing reacts to it**: a
`Violation` already carries its own kind, so an observer restating it would be a second copy of one fact, and two
copies can disagree. That is the same reason a bound's demonstrated direction is derived from its extent rather
than declared beside it.

That split was designed and then rejected against the family's own law: 三儀 ⊥ 三儀 requires each dimension to
implement its own lexical hygiene with **no shared scanner**, so no dimension exposes those stages separately
and every implementor would collapse them into one. A lifecycle no implementor honours is a shape that reads as
governance while governing nothing.

#### Scenario: A dimension implements the protocol over its existing entry point

- **WHEN** a dimension already exposes an outcome-only face taking a manifest path
- **THEN** its observer delegates to that face, adding no second evaluation path and no second reading of the
  workspace

### Requirement: The fold SHALL be ordered, and SHALL stop at the first cannot-judge

Composing observers SHALL fold their outcomes in assembly order: a constitution error from any observer
supersedes every accumulated violation and stops evaluation, and otherwise violations merge into one report.

This is `merge_outcomes`' existing invariant, and the reason is not symmetry — a boundary that could not be
evaluated makes the run's verdict untrustworthy, so reporting violations beside it would present a partial
verdict as a whole one.

Assembly order SHALL therefore be declared **semantically observable**: it decides which cannot-judge is
reported when more than one observer cannot judge. Deterministic and stated, never incidental.

#### Scenario: One observer cannot judge and a later one would find violations

- **WHEN** an observer returns a constitution error and a later observer would report violations
- **THEN** the fold reports the constitution error and does not evaluate the later observer, because a verdict
  resting on a boundary that could not be evaluated is not a verdict

#### Scenario: Two observers cannot judge

- **WHEN** two observers would each return a constitution error
- **THEN** the earlier in assembly order is reported, deterministically

#### Scenario: Every observer is clean

- **WHEN** no observer reports a violation or an error
- **THEN** the fold reports one clean outcome, and an empty observer set SHALL NOT be reported as clean —
  composing nothing is a misconfiguration, not a passing run

### Requirement: The built-in path SHALL keep its behaviour, and the two paths SHALL be held equal

`check_constitution` and the CLI SHALL keep their present composition path and observable behaviour, coverage
included. The protocol SHALL be an additional entry rather than a replacement.

A reaction SHALL assert that folding the three dimensions **through the trait** yields the same outcome as the
existing path on this workspace, and that each dimension's observer declares exactly the bound set that
dimension exports. Two composition paths that could disagree silently is the drift a seam is supposed to end.

#### Scenario: The trait-driven fold disagrees with the existing path

- **WHEN** the two paths produce different outcomes for this workspace
- **THEN** the reaction fails, because an additional entry that quietly judges differently is worse than no
  entry at all

#### Scenario: An observer's declared bounds differ from its dimension's

- **WHEN** an observer's bounds method returns a set other than its dimension's exported declarations
- **THEN** the reaction fails, so the protocol's obligation cannot be satisfied by a second, divergent list

### Requirement: Composition SHALL introduce no trait object

`Observer` SHALL name no `dyn` in its own signature, and composition SHALL introduce none either. Assembly
SHALL fold each observer **as it arrives**, so the heterogeneous collection never exists: each `observe` call is
monomorphized and the accumulator carries only the outcome so far.

A collection-based entry taking `&[&dyn Observer]` was designed first and rejected on measurement. The exposure
would have needed governing, and it cannot be: no module of the composed shell is governed by a semantic
boundary today, and the `dyn`-trait DSL offers only *forbid all* and *forbid named operands* — there is no
allow-except form, so *forbid all* would refuse the protocol's own signature while *forbid named* would never
see it. A declared exposure that no reaction could refuse is a name without a reaction, which this family
forbids. Removing the trait object is therefore not a preference over governing it; governing it was not
available.

The eager fold also carries the short-circuit for free: composing onto an accumulator that already cannot judge
SHALL NOT evaluate the observer at all.

#### Scenario: An adopter composes observers of different concrete types

- **WHEN** two observers of unrelated types are composed into one run
- **THEN** each is folded as it is added, with no trait object in any signature and no collection holding both

#### Scenario: Composition onto a cannot-judge accumulator

- **WHEN** an observer is composed onto an accumulator that already holds a constitution error
- **THEN** that observer is not evaluated, because a verdict resting on a boundary that could not be evaluated
  is not a verdict, and evaluating further would spend work on an answer that cannot be reported

### Requirement: Observation bounds

This capability SHALL declare its own limits, because a protocol whose subject is the obligation to declare
limits cannot be silent about the ones it leaves open. Both concern what the fold **trusts** about a participant
it did not write.

#### Scenario: Whether an observer's declared bounds are complete is not observed — a stated bound

- **WHEN** an observer declares some of its limits and omits others
- **THEN** the protocol does not claim to observe the omission, a stated bound: the trait compels a
  declaration, never a complete one, and no reaction can enumerate the limits of a reaction it did not write
- **PINNED-BY** `an_observer_may_under_declare_its_bounds`

#### Scenario: Whether an observer's own verdict is correct is not observed — a stated bound

- **WHEN** a composed observer returns an outcome that misjudges the workspace it read
- **THEN** the fold merges it as given, a stated bound: it composes verdicts and does not adjudicate them, and
  a protocol that second-guessed each participant would need a second implementation of every dimension
- **PINNED-BY** `the_fold_does_not_adjudicate_a_participant_s_verdict`
