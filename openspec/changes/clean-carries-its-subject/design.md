# Design — Clean carries its subject

## The invariant is relational, not a non-zero count

The obvious shape — `Clean { examined: NonZeroUsize }` — is wrong here, and the counterexample is
already in the tree. `hunyi::check_all` returns `Clean` for an empty bundle **deliberately**, in its
own words: *"a participant was composed and declares nothing for this dimension, which a static-only
adoption does deliberately … refusing would make that adoption's every run exit 2."*

So there are three states, not two:

| state | honest verdict |
|---|---|
| no observer composed | `ConstitutionError` — already closed by `Run::verdict` |
| composed, nothing declared to observe | `Clean` — deliberate, a static-only adoption |
| composed, declarations exist, corpus never reached | **the gap** |

A bare count collapses rows two and three: both are zero. What separates them is not how much was
reached but whether anything was **asked for**. Hence:

    Subject { declared, reached }        declared > 0  ⟹  reached > 0

Zero is not the offence. Declaring something and reaching nothing is.

## What the two numbers mean, and why they are honest for all three dimensions

`declared` is how many boundaries the dimension was given to enforce. `reached` is how many
workspace members it actually observed — the same unit for all three, because all three take their
corpus from `cargo metadata`'s member list.

Both are already in scope at the point each dimension classifies its outcome. This was measured
before the design was fixed, because a design that needs a value threaded from elsewhere is a
design that invites inventing one:

| dimension | `declared` | `reached` | in the same function? |
|---|---|---|---|
| `guibiao::evaluate` | `constitution.boundaries()` | `workspace_member_names(metadata)` | yes |
| `hunyi::check_all` | `boundaries` | derived from `metadata` | yes |
| `louke::audit_probe_coverage` | `declared` parameter | `roots` parameter | yes |

Nothing is threaded and nothing is invented. If any dimension had needed a number it does not hold,
that would have been the design failing rather than the dimension, and the change would have stopped
there.

## Why it rides `Clean` and not a third method

`Observer` asks two questions and the module's own law explains what a third would cost: *"adding a
**stage** breaks every implementor … adding an **answer** to an existing question must break
nothing … Only a new question should force re-examination."*

A `fn subject(&self)` method would be a second call, and it could disagree with what `observe`
actually did — the same objection that killed the proposed "identify your boundary kind" method,
because *"a `Violation` already carries its own `BoundaryKind`, so an observer restating it would be
a second copy of one fact, and two copies can disagree."*

Riding the outcome makes disagreement unrepresentable: the subject is produced by the same call that
produced the verdict.

It also stays inside 三儀 ⊥ 三儀. That law forbids **separate corpus, fact and reaction stages**; it
does not forbid an outcome carrying evidence of the observation that produced it. No stage is added,
no scanner is shared, and each dimension still counts its own corpus independently.

## What the type can and cannot buy

Across a public trait boundary the constructor must be public, so a third party can write
`Subject::of(1, 1)` and lie. The type does not make honesty enforceable, and claiming otherwise
would be the design overselling itself.

What it buys is that **omission becomes commission**. Forgetting the guard stops being possible;
only deliberate misreporting remains. That is exactly the level `bounds()` already operates at, and
the protocol says so: *"The enforcement has to land on the declarer."*

## Growth is free, so v1 is minimal

`Outcome`, `Report` and `Violation` are all `#[non_exhaustive]`. `Subject` will be too, with private
fields and accessors. The breaking half is that `Clean` carries a payload at all; **what** it carries
can grow without another breaking release.

That decides three open questions in the same direction, without spending the window on any of them:

- Should `Violations` carry a subject too? Later, free — `Report` is `#[non_exhaustive]`.
- Should `reached` be identifiable rather than counted, so an agent can ask whether its own file was
  observed? Later, free — `Subject` is `#[non_exhaustive]`.
- Should the composed run refuse when every participant reached nothing? Later, free — that is
  runner behaviour, not a type change.

Doing the minimal invariant now and the richness later is not caution; it is the same rule the
protocol already states about answers versus questions.

## Alternatives rejected

**A sibling variant, `Outcome::NothingObserved`.** Non-breaking, so it needs no window — and it does
not close the gap. It is optional: a participant can still return bare `Clean`, which is the
omission this change exists to make impossible.

**A `Corpus` type, and the corpus as a lifecycle stage.** Forbidden by 三儀 ⊥ 三儀, which requires
each dimension to implement its lexical hygiene independently with no shared scanner. The module
documentation already records why: *"A lifecycle no implementor honours reads as governance while
governing nothing."*

**Requiring a corpus narrowing to cite a declared bound.** The other half of what this class costs,
and it is not a protocol change: `bounds()` already asks the question, and what is missing is
something that requires the answer. That belongs to the repository's own checks and is tracked
separately.
