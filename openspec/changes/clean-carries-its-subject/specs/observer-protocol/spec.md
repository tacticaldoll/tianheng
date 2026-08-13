# observer-protocol delta

## ADDED Requirements

### Requirement: A clean verdict SHALL carry the subject it was reached over

`Outcome::Clean` SHALL carry a `Subject` recording what the observation was asked to enforce and how much of
the workspace it reached. A participant that returns clean therefore states what it observed, as it already
states what it does not observe.

**This is the dual of the bounds declaration, not a third question.** `Observer::bounds` has no default body,
so a participant cannot join a run without declaring what it deliberately does not see; nothing asked what it
did see. The obligation rides the outcome rather than a third method, because a separate call could disagree
with what `observe` actually did — the same reason this capability already gives for not asking an observer to
restate a violation's boundary kind, since two copies of one fact can disagree. Riding the outcome makes the
disagreement unrepresentable: one call produces both.

It adds no lifecycle stage. 三儀 ⊥ 三儀 forbids separate corpus, fact and reaction stages and requires each
dimension to reach its own corpus with no shared scanner; an outcome carrying evidence of the observation that
produced it shares nothing and adds no stage.

**The refused combination is relational, and zero is not the offence.** A `Subject` SHALL be unconstructible
where something was declared and nothing was reached. Reaching nothing is legitimate on its own — an empty
semantic bundle is a static-only adoption, which this capability protects deliberately — so a non-zero count
is the wrong invariant: it cannot tell *nothing to look for* from *looked for nothing*, and refusing it would
make that adoption's every run exit `2`.

What the type buys SHALL be stated rather than overclaimed. The constructor is public, because third-party
participants must return the outcome, so a participant can report a subject it did not observe. The obligation
lands on the declarer, exactly as the bounds declaration does: what becomes impossible is **forgetting**, not
lying.

#### Scenario: A participant declares boundaries and reaches no member

- **WHEN** an observer is given boundaries to enforce and observes no workspace member
- **THEN** it cannot report clean, because the subject that verdict would have to carry cannot be constructed

#### Scenario: A participant is given nothing to enforce

- **WHEN** an observer is composed with an empty bundle, as a static-only adoption composes the semantic
  dimension
- **THEN** it reports clean carrying a subject that declares nothing and reaches nothing, which is
  constructible — the run is honest rather than refused

#### Scenario: A participant observes members and finds nothing wrong

- **WHEN** an observer enforces at least one boundary over at least one workspace member and finds no violation
- **THEN** it reports clean carrying both figures, so a reader can tell that verdict from one reached over
  nothing

#### Scenario: What a subject does not establish — a stated bound

- **WHEN** a participant reports a subject larger than what it observed
- **THEN** nothing reacts. The constructor is public because an implementor must be able to return the
  outcome, so the type converts an omission into a commission and stops there; distinguishing a reported
  subject from an observed one would need the engine to walk each dimension's corpus itself, which is the
  shared scanner 三儀 ⊥ 三儀 forbids. The engine owns this narrowing: it is a limit of what a protocol can ask
  of its participants, not a limit an adopter chose
- **PINNED-BY** `a_subject_is_declared_by_the_participant_and_not_verified`

## MODIFIED Requirements

### Requirement: An empty semantic observer SHALL not read workspace metadata

The semantic dimension's public composed entry point SHALL return clean for an empty boundary bundle without
reading the manifest, and that clean verdict SHALL carry a subject declaring nothing and reaching nothing. The
shell and `SemanticObserver` SHALL delegate both empty and non-empty semantic bundles to that entry point
rather than maintaining independent empty-boundary guards, so every semantic composition path has one behavior
owner.

**The subject makes this allowance expressible rather than implicit.** Before it, an empty bundle's clean
verdict was indistinguishable from a bundle that declared boundaries and reached no member — the same value for
opposite facts, which is why a non-zero count would have refused a static-only adoption on every run.

Whether the shell honours the delegation remains unobserved, and the source-shape reaction that claimed to
observe it stays retired: it read the characters of one function body while the obligation is about what the
shell does, and four narrowings were each defeated — by resolution, by the binding site, by which definition is
the subject, by the caller frame, and by execution, which no widening reaches. The shell's semantic arm invokes
`SemanticObserver`, which makes the two paths' equality for this dimension construction-held; it does not make
an independent shell decision unwritable, so that delegation obligation stays declared as a bound.

#### Scenario: Empty semantic boundaries through the public semantic entry point

- **WHEN** `check_all` receives an empty semantic boundary bundle and a path that cannot be read
- **THEN** it returns clean carrying a subject that declares nothing and reaches nothing, because there is no
  semantic observation to perform — and the subject says so rather than leaving it indistinguishable from an
  observation that reached nothing it was asked to reach

#### Scenario: Empty semantic boundaries through an observer

- **WHEN** a semantic observer has no boundaries and receives a path that cannot be read
- **THEN** it returns that same clean verdict by delegating to the public semantic entry point

#### Scenario: Whether the shell makes an independent semantic decision is not observed — a stated bound

- **WHEN** the shell's composition arm decides semantic emptiness itself instead of leaving the decision to the
  observer it invokes
- **THEN** nothing reacts — a stated bound, and a declared false negative this repository owns. A text reader
  over the composition body was built, hardened across four review rounds and defeated at every level: name
  resolution, the parameter's binding site, the identity of the definition, the caller frame, and execution,
  which no reading of text reaches at all. Invoking the observer made the two paths' *equality* construction-held
  and left this untouched: a guard written above that call compiles, passes the whole suite, and passes every
  gate — measured on the tree that invokes the observer, not on the one that did not. The bound carries no
  pinning test because there is no reaction left to demonstrate a gap in; it is tracked instead
- **UNPINNED** `BACKLOG.md` — *the shell's semantic delegation, held by construction*
