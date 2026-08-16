# observer-protocol Specification

## Purpose

Make declaring what a reaction does not observe a condition of taking part: a fixed lifecycle whose every method
has no default body, and an eager fold that composes participants into one verdict while preserving the family's
cannot-judge-supersedes invariant.

## Subject

- `crates/tianheng/src/existential.rs`
- `crates/*/src/observer.rs`
- `crates/kanhe/tests/observer_protocol.rs`

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

### Requirement: The built-in path SHALL keep its behaviour, and the two paths SHALL be held equal

`check_constitution` and the CLI SHALL keep their present composition path and observable behaviour, coverage
included. The protocol SHALL be an additional entry rather than a replacement.

A reaction SHALL assert that folding the three dimensions **through the trait** yields the same outcome as the
existing path on this workspace, and that each dimension's observer declares exactly the bound set that
dimension exports. Two composition paths that could disagree silently is the drift a seam is supposed to end.

Both obligations name a *property*, not a comparison, and the paragraphs below say how each is reacted to —
because for parts of both the two sides are one thing, and a comparison of one thing against itself is an
assertion that cannot fail. Where that is so, the reaction is over the construction that makes the property
true.

**The comparison SHALL NOT be able to hold vacuously in any one dimension.** The fixture it compares over SHALL
declare a deliberately violated boundary in **every** dimension, and the reaction SHALL assert that every
dimension reacted. A dimension whose declared set is empty contributes nothing to either side, so the two paths
agree for that dimension however wrongly one of them behaves: measured, an empty constitution is `Clean` on this
workspace, and with a static-only fixture, replacing an observer's body with `Clean` left the reaction passing.
Asserting per-dimension reaction is what keeps the fixture from silently going vacuous when the workspace
changes under it.

**An observer declares its dimension's bounds by delegating, and the reaction SHALL be over that delegation's
shape** rather than over a comparison of the two sides. Comparing an observer's `bounds()` against its
dimension's exported declarations cannot fail while `bounds()` *is* that export — measured, it is `f() == f()`,
and drifting a declaration left the reaction passing. What the requirement refuses is a **second, divergent
list**, and a second list is something written in a body; so each observer's `bounds()` SHALL hold exactly the
delegation and nothing else, recognized by position within that method rather than by the call appearing
anywhere in the file. The declarations' *content* is held by `observation-bound-model`'s extent projection and
SHALL NOT be re-asserted here.

Two things follow from *recognized by position*, and both were measured as gaps rather than reasoned about. The
method SHALL be located by a **unique occurrence** of its signature in the source, and the reader SHALL decline
otherwise. An earlier rule required the signature to begin a trimmed line, which bought only the exclusion of a
mid-line mention: a whole-line copy inside a block comment anchors exactly as well as the definition, and a decoy
conforming copy above a divergent method let the equality pass on text that was not the method — measured.
Counting occurrences does **not** subsume the mid-line mention, and the reaction SHALL require both
conditions rather than either. A source that mentions the signature mid-line and never defines it has exactly
one occurrence, so a count-only rule admits it, anchors in the prose, and returns the next function's body as
this method's — measured. Each rule refuses something the other admits, and requiring both only ever declines
more, which is this reader's declared error direction. What neither refuses is a **whole-line** copy inside a
block comment, which is declared as a bound below rather than described as closed. And a **trailing comment** on the delegation SHALL be
prose, not a second list: the region discipline this family already holds says a comment is never executed text,
and the reaction that judges a shell gate's own text strips one before comparing for exactly this reason.
The reaction SHALL apply Rust line-comment semantics to the inspected body: a `//` line is prose, while a Rust
attribute beginning with `#` remains executed Rust text.

Where the built-in path obtains a dimension's outcome **by invoking that dimension's observer**, equality for
that dimension holds **by construction rather than by observation**, and the spec SHALL say which dimensions
those are — otherwise a reader takes a constructed equality for a measured one. The list is now held to that:
a reaction reads the built-in path's own source and refuses if a dimension it declares construction-held is
not constructed there, or if a dimension it does not declare so is. This was true in only one direction until
this window: the list named runtime alone, the shell's semantic arm changed under it, and the list was
repaired by hand — a membership claim about a set with an enumerator, which is the shape this family refuses
everywhere else. What answers it is textual rather than a perturbed build: for a construction-held dimension
the built-in path does not call some *other* function that happens to agree with the observer today, it
directly constructs that dimension's own `Observer` and calls `.observe()` on it, so there is exactly one
implementation to read rather than two runs to compare. The **runtime** and **semantic** dimensions are such
cases: the built-in path invokes `RuntimeObserver::new(...).observe(...)` and
`SemanticObserver::new(...).observe(...)` directly, so for runtime its two copies of those three statements —
the corpus derivation, the audit call and the `cannot read workspace` message — become one, and for semantic
there is no second call at which the two verdicts could differ. What that does *not* settle for either is
whether the shell honours its delegation obligation, which is a different property with a bound of its own.
The **static** dimension remains independently implemented on both sides — the built-in path calls
`check_and_cover` and never constructs `StaticObserver` — and for it the reaction's equality is observed.

Where a dimension's equality is construction-held, the reaction SHALL still observe that the fixture's boundary
for that dimension **reacts at all**. Otherwise an arm that quietly went vacuous would leave the whole
comparison resting on the dimensions that did not.

#### Scenario: A whole-line occurrence that is not the definition anchors the read — a stated bound

- **WHEN** the method's definition is absent from the inspected source — the impl having moved elsewhere — and a
  whole-line copy of its signature remains anywhere in that file: inside a block comment, inside a string
  literal, or in any other position the reader does not distinguish from executed text
- **THEN** the reaction reads that copy's body and reports it as the method's. Both anchor conditions are
  satisfied — one occurrence, at a line start — and the reader knows nothing of comments or literals, so the
  class is "the unique whole-line occurrence is not the definition" rather than any one syntactic position.
  What passes is a **second, hand-maintained path that agrees today**: a *divergent* list does not, because
  `observation-bound-model` reads every dimension's declarations through `Observer::bounds` and holds them in a
  bijection with the specs, which fails on any difference of membership or content. Measured both ways. So the
  residual is narrower than a divergent list slipping through, and wider than a comment.
  **Not a defect unique to this reader.** `kanhe::region`'s own `Executed` abstraction declares the identical
  residue for the same reason — a block comment and a string literal both need nested-span lexing to tell from
  executed text, which this tree has defeated repeatedly and left declared rather than approximated. Closing
  either needs the same instrument; closing one without the other would leave the class recorded twice under
  two names for a reader to reconcile.
  This bound SHALL be **shown rather than described**: the reaction enumerates every shape it decides together
  with the decision, the reader is run against that table, and the rows where it reads a body that is not the
  method's are this bound. A sentence here that the table contradicts fails, which is what the three repair
  rounds preceding this scenario could not do
- **UNPINNED** `BACKLOG.md` — *the bounds-method reader anchors on a whole-line occurrence that is not the definition*

#### Scenario: The stated construction-held list is held against the composition path

- **WHEN** the built-in composition path's own source is read for each dimension named construction-held above
- **THEN** the reaction fails if a construction-held dimension's own `Observer` is not constructed there, or if
  a dimension not named construction-held has one constructed there instead — read directly rather than
  inferred from a mutated build, since a construction-held dimension has exactly one implementation to find,
  not two runs to compare
- **PINNED-BY** `the_construction_held_list_matches_the_built_in_composition_path`

#### Scenario: The trait-driven fold disagrees with the existing path

- **WHEN** the two paths produce different outcomes for this workspace
- **THEN** the reaction fails, because an additional entry that quietly judges differently is worse than no
  entry at all

#### Scenario: A dimension of the equality fixture stops reacting

- **WHEN** the fixture's declared boundary for some dimension no longer produces a violation of that
  dimension's kind
- **THEN** the reaction fails, because from that moment the comparison proves nothing about that dimension —
  and it fails naming the dimension, since the repair is to the fixture rather than to either path

#### Scenario: An observer's bounds method holds a list of its own

- **WHEN** an observer's bounds method holds anything other than the delegation to its dimension's exported
  declarations
- **THEN** the reaction fails, so the protocol's obligation cannot be satisfied by a second, divergent list

#### Scenario: The delegation carries a trailing comment

- **WHEN** an observer's bounds method holds the delegation followed by a comment explaining it
- **THEN** the reaction accepts it, because a comment is prose and not a list — the same region rule every other
  reaction in this repository reads its subject through

#### Scenario: That trailing comment contains a closing brace

- **WHEN** an observer's bounds method holds the delegation, a trailing comment containing `}`, and a further
  statement beneath it
- **THEN** the reaction reads the body to its real closing brace and fails on the further statement, because the
  comment tail is removed **before** the braces are counted rather than after — counted through, the body closed
  at the comment and the further statement was never presented to the comparison at all, so the one thing this
  requirement refuses passed as the delegation

#### Scenario: A brace inside a block comment or a string literal moves the read body extent — a stated bound

- **WHEN** an inspected bounds-method body carries `{` or `}` inside a block comment or a string literal
- **THEN** the reaction reads an extent that is not the method's body — a stated bound.
  It counts braces outside line comments only, and closing the gap needs the string-literal lexing this
  repository measured and rejected: this tree's own lexer suites put comment delimiters inside string literals,
  several of them nested, so a delimiter-counting scan opens a phantom comment at the first of them and swallows
  every definition to the next close. For **this** comparison the error direction is the safe one, and
  it is what the pin shows — no brace-carrying construct survives the exact one-statement comparison, so a moved
  extent refuses a **conforming** body rather than accepting a divergent one. The direction is a property of the
  comparison rather than of the extent, and it does not transfer to another reader of that extent: the
  same moved extent meeting a count-and-containment comparison would accept a divergent body instead. A reader of
  that second kind existed over the shell's composition body and is retired; the direction is recorded here so
  the next one is not written on the assumption that this bound's safety transfers to it
- **PINNED-BY** `a_brace_in_a_block_comment_moves_the_body_extent`

#### Scenario: A Rust attribute appears in an inspected body

- **WHEN** an inspected Rust body contains a line whose trimmed start is `#`
- **THEN** the reaction retains that line as Rust source rather than dropping it as a shell comment

#### Scenario: An observer's bounds method cannot be found where the reaction looks

- **WHEN** the method is absent from the source the reaction reads
- **THEN** the reaction refuses to judge rather than passing, because a reaction that finds nothing to read has
  not observed that the obligation holds

#### Scenario: A second line could anchor the bounds method

- **WHEN** the bounds-method signature occurs more than once in the observer's source — a commented-out copy
  being the measured case
- **THEN** the reader declines rather than reading the first. Here the decoy inverts this reader's declared
  error direction rather than merely moving the extent: a *conforming* copy in the comment makes the exact
  one-statement equality pass while the real method holds a second, divergent list, so the over-reaction the
  bound records becomes an acceptance

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

Because that same measurement leaves 渾儀 unable to watch this crate, the reaction holding this requirement is
**lexical**, and a lexical reaction SHALL state where it stops and SHALL check every premise it rests on:

- Its recognizer SHALL be a **named function over one line** of text, so its limit can be demonstrated by giving
  it text rather than by rewriting the crate.
- It SHALL over-approximate in the safe direction: it cannot distinguish a `pub` item in a private module from a
  publicly reachable one, and flags both. A false positive here is a sentence to write; a false negative is an
  exposure nobody governs.
- It reads every Rust source file recursively below this crate's `src` directory. Directory nesting and module
  visibility SHALL NOT remove a file from the corpus: a private nested module can still expose an item through a
  public re-export. An unreadable traversed directory or Rust source SHALL fail loud rather than shorten the corpus.
- Reading one line at a time leaves a residual the premise check cannot remove, declared as an observation bound
  below.

#### Scenario: An adopter composes observers of different concrete types

- **WHEN** two observers of unrelated types are composed into one run
- **THEN** each is folded as it is added, with no trait object in any signature and no collection holding both

#### Scenario: Composition onto a cannot-judge accumulator

- **WHEN** an observer is composed onto an accumulator that already holds a constitution error
- **THEN** that observer is not evaluated, because a verdict resting on a boundary that could not be evaluated
  is not a verdict, and evaluating further would spend work on an answer that cannot be reported

#### Scenario: A trait object appears in a nested source file

- **WHEN** a Rust source below a nested `src` directory contains a one-line public trait-object signature
- **THEN** the reaction reports it exactly as it reports the same signature in a top-level source file

#### Scenario: A trait object on a wrapped signature's continuation line is not seen — a stated bound

- **WHEN** a public signature spans several lines and names a trait object on a line that does not itself begin
  with `pub `
- **THEN** the reaction does not see it, a stated bound: the recognizer is handed one line at a time, so the
  continuation is never a candidate it declined — it is text the observation never presents. Closing it needs 渾儀
  watching this crate, which the same measurement above found unavailable. Multi-line public signatures exist
  here, so the shape is live even where no instance names a trait object
- **PINNED-BY** `a_trait_object_on_a_continuation_line_is_not_recognized`

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

### Requirement: A participant outside the family SHALL be demonstrated joining a run

A dogfood example SHALL exist in which a crate that is **not** part of the family implements `Observer`, is
composed into a run alongside the dimensions, and reacts. The protocol's claim is that a third party can take
part, and every implementor of it is a crate of this family, in this workspace, returning a literal list from its
own module — so the claim has never been executed.

The example's participant SHALL declare **computed** bounds: at least one id built at run time from what the
participant observed rather than written as a literal. `BoundId`'s owned-or-borrowed form exists precisely for an
implementor whose bounds are discovered, and until this it had no caller that was not a literal — a capability
shipped for a caller that did not exist.

The participant SHALL declare **every** bound it has, not only the one that demonstrates the mechanism. The
example is the one artefact teaching a third party how to join a run honestly, so a participant there that reacts
where its own stated reason does not require it — and says nothing — teaches the mechanism while withholding an
instance of it. Measured: its header rule read only a file's first line, so a module header below a license comment
was reported missing while the rule's reason, *that a reader learns what the file is for*, was satisfied. That
distance between a rule's wording and its reason is what `Reached::OverReacts` names, and it SHALL be declared
rather than closed where closing it would trade one edge for others and make the wording diverge from the code.

The example SHALL therefore exhibit **more than one extent**, so it demonstrates the bound *model* and not only the
call that declares a bound.

The example SHALL require **no addition to any crate's public API**. If joining a run needs an export the family
does not expose, the protocol is not usable by a third party, and that is the finding rather than a reason to add
the export.

#### Scenario: A third-party participant joins a composed run

- **WHEN** the example composes its own observer alongside the family's dimensions over its workspace
- **THEN** the run reacts, and the participant's contribution is present in the verdict rather than only the
  dimensions'

#### Scenario: The participant's bounds are computed rather than literal

- **WHEN** the participant declares its bounds
- **THEN** at least one id is built from what it observed, so the owned-or-borrowed declaration form is exercised
  by a caller outside the family

#### Scenario: The participant reacts where its own reason does not require it

- **WHEN** the participant's rule reacts to a shape its stated reason is already satisfied by
- **THEN** that is declared as an over-reaction with its own bound, so the example demonstrates the extent model
  rather than only the mechanism for declaring one

#### Scenario: Joining a run would require a new export

- **WHEN** an outside crate cannot implement or compose the protocol with the public surface alone
- **THEN** that is a defect in the public surface, reported as such, and not repaired by adding whatever export
  the example happened to need

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
