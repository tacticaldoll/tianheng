## Context

`observation-bound-model` typed where each reaction's measure stops and holds the specs' declarations and the
code's in a bijection. Its projection now leads with a figure that was previously unsayable: 11 of 45 declared
bounds are declared false negatives, 7 owned by a dimension's own engine.

What it cannot do is oblige anyone to declare. Measured at the tip of `release/0.5.0`:
`grep -c 'pub trait' crates/tianheng/src crates/xuanji/src` returns zero, and `evaluate_constitution` hand-wires
`check_and_cover`, `hunyi::check_all` and `audit_probe_coverage`. There is no seam, so there is no obligation.

## Goals / Non-Goals

**Goals**

- Make declaring one's bounds a condition of participating, enforced by the compiler rather than by review.
- Give the shell one composition entry that folds a set of observers, preserving the existing verdict invariant.
- Dogfood it: the three dimensions implement the protocol rather than being exempt from it.

**Non-Goals**

- Rewriting the built-in composition path. `check_constitution` and the CLI keep their behaviour; a behaviour
  change does not belong under a change whose subject is a seam.
- Adjudicating a participant's verdict, or checking that its declared bounds are complete. Both declared as
  bounds of this capability.
- A staged pipeline where one dimension consumes another's output. Forbidden by 三儀 ⊥ 三儀, and D2 below.

## Decisions

### D1 — Trait methods with no default, because the enforcement must land on the declarer

`enum` + a hand-maintained `ALL` array is the shape this repository already uses well (`SeamKind`), and it is
the wrong shape here. The difference is **which side breaks**: adding a variant breaks the consumer's `match`,
while every existing declaration keeps its old classification and nobody re-examines it. The enforcement lands
on the reader.

A trait method with no default body lands it on the declarer: adding a stage breaks every implementor, family
and third-party alike. That asymmetry is the whole design, and its other half matters as much — adding an
**answer** to an existing question must break nothing, which is `#[non_exhaustive]` on the extent enums. Only a
new *question* should force re-examination.

**Rejected — a typestate builder.** It was the first sketch and it is closed in the wrong dimension: a chain of
inherent methods on a concrete type can be *called* from outside but never *implemented* from outside, so an
adopter's own observer could not join. The unrepresentability it bought lives in the *data* instead — nested
extent enums — which is what makes those values returnable from a trait method at all.

### D2 — Three methods, because the five-stage lifecycle was a fiction the law forbids

The sketch was `corpus` → `observe` → `react`, mirroring an observation pipeline. Measured against the family's
own law, that shape cannot be honoured: 三儀 ⊥ 三儀 requires each dimension to implement lexical hygiene
**independently, by design**, with "no shared scanner" — the conformance tests say so in those words. No
dimension exposes a corpus step or a fact step separately, and every implementor would collapse the three into
one call.

So the seam is what each dimension actually has: identify yourself, observe a workspace, declare your limits.
Recording this as a decision rather than quietly shrinking the sketch matters, because a lifecycle no
implementor honours reads as governance while governing nothing.

The same law is why the composition is a **fan-out fold and never a pipe**: no observer receives another's
output. The shell composes; the dimensions do not.

### D3 — The fold preserves an existing invariant, and declares what was incidental

`merge_outcomes` already holds it: "A constitution error from either side supersedes any violation — a boundary
that could not be evaluated makes the run's verdict untrustworthy — and otherwise the two reports' violations
merge into a single report. `first` is checked first, so its error wins deterministically when both error." The
caller short-circuits on the accumulated error.

The protocol keeps exactly that. What it **adds** is a declaration: assembly order is semantically observable,
because it decides which cannot-judge is reported. Today that is a property of a hand-written call sequence and
nobody has had to think about it; the moment order is an adopter's to choose, it must be stated.

An empty observer set is a misconfiguration, not a clean run — the vacuity direction this repository has
re-opened six times in one window.

### D4 — An additional entry, not a rewrite

`check_and_cover` produces the static outcome **and** the coverage advisory from one `cargo metadata` read, and
the CLI presents coverage without letting it change the reaction. A protocol returning only an outcome cannot
carry that, and splitting the call would double the read.

So `check_constitution` keeps its path, and the trait is an additional composition entry whose observers
delegate to the outcome-only faces that already exist — `guibiao::check`, `hunyi::check_all`, and 漏刻's audit
with the roots and anchor its own observer derives.

The cost is two composition paths, and it is paid rather than accepted: a reaction asserts they agree on this
workspace. Two paths that could disagree silently is precisely the drift a seam is supposed to end, so the
guard against it is not optional.

### D5 — The eager fold removes the trait object, because governing it was not available

The first design held `&[&dyn Observer]` and declared the trait-object exposure as a boundary of the shell,
reasoning that composition already lives there — `sans_io.rs` says a thing spanning two dimensions "lives in the
shell". Two measurements killed it:

- **No module of `tianheng` is governed by a semantic boundary.** The self-governance constitution's only
  semantic self-boundary is `sans_io_pure` on 璇璣. So a "declared exposure" in the shell would have been
  observed by nothing.
- **The `dyn`-trait DSL has no allow-except form.** `must_not_expose_dyn()` forbids all — it would refuse the
  protocol's own signature, with no way to exempt it — and `must_not_expose_dyn_of(operands)` forbids named
  traits, which would never see `dyn Observer` unless it were itself named, at which point the protocol could
  not compose. Neither polarity can express "no trait object except this one".

A declared exposure no reaction could refuse is a name without a reaction, which this family forbids outright.
So the trait object is **removed rather than governed**, and the way out is better than the thing it replaces:
the fold is **eager**. `Run::over(..).observe(a).observe(b).verdict()` folds each observer as it arrives, each
call monomorphized, the accumulator carrying only the outcome so far. The heterogeneous collection never exists,
so there is nothing to expose.

It also carries the short-circuit for free rather than as a check: composing onto an accumulator that already
cannot judge does not evaluate the observer at all, which is exactly `evaluate_constitution`'s present behaviour
expressed as a property of the builder instead of an `if` before each dimension.

Recorded at this length because the rejected design was two decisions deep and looked principled — the lesson is
that a boundary declaration must be checked against the DSL that would have to carry it, not only against the
architecture that motivates it.

## Risks / Trade-offs

**The obligation is to declare, never to declare completely.** A third-party observer can satisfy the trait with
a partial list, and no reaction can enumerate the limits of a reaction it did not write. Declared as a bound
rather than implied away. What the protocol does buy is that *saying nothing* stops being possible, which is the
state it was built to end.

**The fold trusts each participant's verdict.** It composes outcomes and does not adjudicate them; second-guessing
would require a second implementation of every dimension. Also declared as a bound.

**Two composition paths exist.** Mitigated by the equality reaction rather than by care, and the alternative —
rewriting the built-in path in this change — would hide a behaviour change under a seam.

**No trait object means no heterogeneous storage.** An adopter who genuinely needs to hold observers in a
collection before composing them must box them behind their own trait object, in their own crate, where their own
governance applies. Accepted: the alternative is an exposure this family cannot govern in the crate that composes
its governors.

**A published trait is a compatibility surface.** Adding a stage later is a breaking change for adopters, by
design: that is the enforcement working as intended, not an accident to be softened. It should therefore happen
rarely and deliberately, which is what "fixing the lifecycle" means.
