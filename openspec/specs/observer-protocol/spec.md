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

### Requirement: An empty semantic observer SHALL not read workspace metadata

The semantic dimension's public composed entry point SHALL return `Clean` for an empty boundary bundle without
reading the manifest. The shell and `SemanticObserver` SHALL delegate both empty and non-empty semantic bundles
to that entry point rather than maintaining independent empty-boundary guards, so every semantic composition
path has one behavior owner.

The repository reaction SHALL inspect the executed body of the shell's `evaluate_constitution` composition
function. That body SHALL access `constitution.semantic_boundaries()` exactly once, as the direct boundary
argument to `hunyi::check_all`; a missing function, an additional semantic-boundary inspection, or an indirect
shell-local decision SHALL fail rather than be treated as delegation.

The reaction SHALL distinguish a body that does not delegate from a body it could not read. **Two** conditions
make the text unsafe to judge and SHALL be refused; the rules after them narrow what the comparison reads, and
their outcome is a verdict rather than a refusal.

**The anchor SHALL be unique.** The function is located by line position, which rules out a mid-line mention
and nothing more: a whole-line copy of the signature — inside a block comment, a multi-line string, or a second
module — anchors exactly as well as the definition. A doc comment does not, because `///` becomes the trimmed
start; that exception is written down because it was asserted the other way before it was checked. Where more
than one line could anchor the read, the reaction SHALL decline. This is the half no in-body check can cover,
because every delimiter that made such an extent wrong sits outside it; measured, a commented-out copy above the
function let a body carrying an independent shell-local guard read as a conforming delegation.

**An extent carrying a literal or comment delimiter SHALL be refused.** The extent is found by counting braces,
and a string literal, a character literal, or a block comment inside the body moves it; where the extent carries
`"`, `'`, or `/*` in executed code, the reaction SHALL refuse. Stating the refusal closes a false negative
rather than describing one: this requirement's comparison is a count and a containment, and both survive a
truncated extent unharmed — a second semantic-boundary access past the cut is absent from what is compared, so
the one shape the requirement refuses reads as the delegation it demands. Measured on synthetic bodies in the
tracked shape, each returning the conforming verdict before the refusal existed; the tracked body itself carries
no such delimiter, which is why a fixture rather than the tracked file is the observation source.

**Delimiters SHALL be read in executed code only, on the same terms the brace count uses.** Text after `//`
cannot move a brace, because the count already treats it as prose, so refusing on a delimiter there would refuse
on text that cannot cause the fault — and an apostrophe in ordinary English is one reflow away from the tracked
body. By the same rule the comparison SHALL NOT read a comment as code: the required call appearing only in a
comment does not satisfy the requirement, which it did while tails were compared.

The count SHALL be over the accessor's **name** rather than a receiver spelling. A rebinding, an associated-
function call, or a reborrow reaches the semantic boundaries just as a direct receiver does, and counting one
spelling counted the spelling: measured, each walked past the comparison while an independent shell-local guard
stood in the body. The *direct* spelling remains what the delegation containment holds.

Over-refusal is the declared direction, and the character literal is why it must be said out loud: a lifetime is
spelled with the same delimiter, so a composition body that names one is refused too. That cost is accepted
because a refusal is loud and repairable in the commit that causes it, while the alternative is the silent pass
above. The reaction SHALL therefore be held to still *judging* the tracked body, so a refusal that swallowed
every input could not pass for the closure.

**This does not contradict an empty observer set being a cannot-judge**, and the difference is stated here
because the two sentences read as a contradiction otherwise. An empty *bundle* means a participant was
composed and declares nothing for its dimension — a static-only adoption is exactly that, and there is nothing
to observe, so `Clean` is the honest answer. An empty *observer set* means nothing was composed at all: the
misconfiguration is in the assembly, not in a dimension's declarations, and there is no participant whose
silence could be read as cleanliness. Unifying them fails in both directions: making an empty bundle a
cannot-judge would make every static-only adopter's composed run report exit `2`, a false refusal on the
primary use case, and reporting an empty observer set as clean is the vacuous pass this repository has
re-opened most often. The asymmetry is therefore a property of the two constructions rather than a claim a
reaction could observe, and it carries no scenario for that reason.

#### Scenario: Empty semantic boundaries through the public semantic entry point

- **WHEN** `check_all` receives an empty semantic boundary bundle and a path that cannot be read
- **THEN** it returns `Clean`, because there is no semantic observation to perform

#### Scenario: Empty semantic boundaries through an observer

- **WHEN** a semantic observer has no boundaries and receives a path that cannot be read
- **THEN** it returns `Clean` by delegating to the public semantic entry point

#### Scenario: Empty semantic boundaries through the shell

- **WHEN** the shell composes a constitution whose semantic boundary bundle is empty
- **THEN** the source-shape reaction finds exactly one semantic boundary access, passed directly to the public semantic entry point, and fails if the shell decides emptiness itself

#### Scenario: A second semantic-boundary access sits past a moved extent

- **WHEN** the composition body holds the delegation, a construct whose delimiter moves the read extent, and a
  further `constitution.semantic_boundaries()` access beyond the resulting cut
- **THEN** the reaction refuses to judge rather than reporting the delegation as satisfied, because the further
  access is the one shape this requirement refuses and a moved extent never presents it to the comparison at all

#### Scenario: The composition body carries a delimiter that can move the read extent — a stated bound

- **WHEN** the extent read for `evaluate_constitution` carries `"`, `'`, or `/*` in executed code
- **THEN** the reaction refuses to judge, naming the delimiter — a stated bound. It does not decide whether the
  body delegates, because the extent it would decide over may not be the body; separating a brace in code from
  one inside a string, a character literal, or a block comment needs the lexing this repository measured and
  rejected, so the reaction declines the verdict instead of guessing at it. The set is three delimiters and not
  four: a block comment opened above the signature closes *after* the brace it hides, so the extent is cut
  before the `*/` and carries no delimiter at all — a fourth member no shape reaches is a declared set drifting
  from its enumerator, deletable with every document still agreeing
- **PINNED-BY** `an_ambiguous_delegation_extent_is_refused_rather_than_judged`

#### Scenario: A second line could anchor the read

- **WHEN** more than one line in the source has the composition function's signature as its trimmed start — a
  commented-out copy, a copy inside a multi-line string, or a second module's definition
- **THEN** the reaction declines rather than reading the first, because it cannot know which body is the
  subject, and the delimiters that made the wrong extent wrong sit outside that extent where no in-body check
  reaches them

#### Scenario: The ambiguity refusal precedes the comparison

- **WHEN** an extent is both moved and divergent within the text that survives the cut
- **THEN** the reaction refuses rather than reporting the divergence, because a verdict formed on text the
  reaction cannot vouch for is unsound whichever way that verdict happens to fall — an ordering that only
  escalated a *passing* verdict would report the divergence and be indistinguishable on every other input

#### Scenario: A delimiter appears only inside a comment

- **WHEN** the extent's only `"`, `'`, or `/*` sits after a `//`, whether on its own line or as a tail
- **THEN** the reaction judges the body normally, because the brace count already treats that text as prose and
  a delimiter that cannot move a brace is not evidence the extent is wrong

#### Scenario: The required call appears only in a comment

- **WHEN** the body's only occurrence of the direct `hunyi::check_all` delegation is inside a comment
- **THEN** the reaction does not report delegation, because a requirement satisfied by prose is satisfied in
  appearance and failed in substance

#### Scenario: A second semantic-boundary access is reached through a rebinding

- **WHEN** the body reaches the semantic boundaries a second time through a local binding, an associated-function
  call, or a reborrow rather than through the declared receiver spelling
- **THEN** the reaction reports it, because the requirement admits one access however the receiver is written

#### Scenario: A moved extent leaves no delimiter behind

- **WHEN** a block comment opened above the signature closes after a brace it hides, so the extent is cut before
  any delimiter reaches it
- **THEN** the reaction still fails rather than reporting delegation, because the surviving text loses the
  delegation along with everything else — the residual is loud, which is a property held by a reaction rather
  than asserted here

#### Scenario: The tracked composition body is still judged

- **WHEN** the reaction reads the tracked `evaluate_constitution` body, which carries none of those delimiters
  on an executed line
- **THEN** it returns a verdict rather than a refusal, because a refusal that swallowed every input would
  satisfy the bound above while observing nothing

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
method SHALL be located by **line position** — a line whose trimmed start is the signature — so a mention of it
*within* a line of prose cannot be brace-matched from. That is all line position buys, and the spec claimed more
than it bought: a **whole-line** copy of the signature inside a block comment has the signature as its trimmed
start, so it anchors exactly as well as the definition, and a decoy conforming copy above a divergent method let
the equality pass on text that was not the method — measured. The anchor SHALL therefore also be **unique**: two
lines that could anchor the read make the subject unknown, and the reader SHALL decline rather than take the
first. And a **trailing comment** on the delegation SHALL be
prose, not a second list: the region discipline this family already holds says a comment is never executed text,
and the reaction that judges a shell gate's own text strips one before comparing for exactly this reason.
The reaction SHALL apply Rust line-comment semantics to the inspected body: a `//` line is prose, while a Rust
attribute beginning with `#` remains executed Rust text.

Where the built-in path obtains a dimension's outcome **by invoking that dimension's observer**, equality for
that dimension holds **by construction rather than by observation**, and the spec SHALL say which dimensions
those are — otherwise a reader takes a constructed equality for a measured one. The runtime dimension is
currently such a case: the built-in path delegates to `RuntimeObserver`, so its two copies of the corpus
derivation, the audit call and the `cannot read workspace` message become one. The static and semantic
dimensions remain independently implemented on both sides, and for them the reaction's equality is observed.

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
  same moved extent meeting a count-and-containment comparison accepts a divergent body instead, which is why
  the shell-delegation reaction refuses an ambiguous extent rather than inheriting this bound
- **PINNED-BY** `a_brace_in_a_block_comment_moves_the_body_extent`

#### Scenario: A Rust attribute appears in an inspected body

- **WHEN** an inspected Rust body contains a line whose trimmed start is `#`
- **THEN** the reaction retains that line as Rust source rather than dropping it as a shell comment

#### Scenario: An observer's bounds method cannot be found where the reaction looks

- **WHEN** the method is absent from the source the reaction reads
- **THEN** the reaction refuses to judge rather than passing, because a reaction that finds nothing to read has
  not observed that the obligation holds

#### Scenario: A second line could anchor the bounds method

- **WHEN** more than one line in the observer's source has the bounds-method signature as its trimmed start —
  a commented-out copy being the measured case
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
