# Change: correct what a declared bound says let an inversion ride green

## Why

`gate-shape-contract`'s bound *Whether a gate's 1-versus-2 assignment is correct is not observed* names its cause
like this:

> it checks that the twin asserts codes, never that the codes the gate chose are the right ones, **which is the
> judgment that let a `return`-instead-of-`exit` inversion ride green**

The final clause is **wrong about its own history**, and it is the kind of wrong that matters: a bound reads as
*permission*, so a reader deciding whether to trust this gate surface is told the residual is wider than it is.

The instance it refers to is `86e8592`, in this window. Read back, it produced **both** of the bound's named
directions in one gate:

- every refusal was `1`, so a shallow clone with no release spine reported *"the release surfaces disagree"* — a
  **misconfiguration reported as a violation**;
- and the exit-contract backstop converted every genuine incoherence into `2` — a **violation reported as
  cannot-judge**.

But what let it ride green was neither of those. It was that *the matrix asserted a non-zero status rather than a
code* — recorded in that commit's own words: "the one property that would have caught it was the one it lacked."
And that property now exists. `gate-shape-contract`'s **`exit codes`** property requires every twin to assert the
expected code rather than merely non-zero, and its remedy already cites this very instance.

So the residual is narrower than the bound claims. What is genuinely unobserved is only whether a code a twin
asserts *exactly* is the semantically right one — which no lexical reaction can decide, because it needs each
gate's meaning.

## What Changes

- The bound's stated cause is corrected: the enabling mechanism is now a checked property, and the residual is
  the semantic judgment alone. The bound is **not** removed — narrowing what a bound claims is the direction that
  costs nothing and misleads nobody.
- **BACKLOG:** the bound's re-evaluation is recorded — one instance, both directions, its enabler closed, and a
  trigger that the old text could not have distinguished: an instance where a twin asserts an **exact** code and
  that code is the wrong one.
- **BACKLOG:** the class this window kept finding is filed with a **measured** cost rather than an estimated one:
  *every normative SHALL either has a reaction or is a declared bound*.

## Impact

- Affected specs: `gate-shape-contract`
- Affected code: none — the reaction already holds what the corrected text describes
- No public API change, no version bump. The bounds projection changes by exactly this bound's rationale.
