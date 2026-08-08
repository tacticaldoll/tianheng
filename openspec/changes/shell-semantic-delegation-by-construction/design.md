# Design — the shell's semantic delegation, held by construction

## Why not a sixth reader

The retired reaction's defeat list is the argument. Name resolution, the parameter's binding site, which
definition is the subject, the caller frame — each was closed and the next one opened. The last group closes
nothing: a delegation bound to `let _`, written inside a never-invoked `macro_rules!`, or placed in a
conditionally-called closure satisfies every textual rule while the shell decides for itself. Whether a call
*happens* is not a property of the characters around it.

## What "by construction" means here, and what it does not

This repository already has the sentence: where the built-in path obtains a dimension's outcome **by invoking
that dimension's observer**, equality for that dimension holds by construction. The runtime arm is that shape.
The semantic arm was not: it called `hunyi::check_all` directly, which is what `SemanticObserver::observe`
calls, so the two agreed only while nobody put a decision between them.

Invoking the observer removes the second site. It does **not** make a shell-local guard impossible — an author
can still write one above the call — and the requirement must not be read as claiming that. What it removes is
the *second implementation*: there is one call, so there is nothing for a divergent copy to diverge from. That
is the same guarantee the runtime arm carries, stated in the same words, and it is the guarantee the retired
reaction was built to approximate.

## The cost, paid deliberately

`SemanticObserver::new` takes an owned `SemanticBoundaries`, so the arm clones the declared bundle once per
run. The runtime arm already pays the equivalent `to_vec`. Passing a reference would need the observer to
borrow, which changes a published constructor's signature for an internal call site — a worse trade than one
clone on a path that then reads the filesystem.

## What the equality reaction still observes

Two of three dimensions are now construction-held, which is exactly the shape that makes a comparison read as a
guarantee while proving nothing. The reaction already asserts, per dimension, that the fixture's boundary
reacted — added when the comparison was found able to hold vacuously in any one dimension. That assertion is
what remains for semantic and runtime, and the requirement now says so rather than leaving a reader to infer
it: an arm that quietly went vacuous would leave the whole comparison resting on static.
