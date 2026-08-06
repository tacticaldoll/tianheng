## Context

Nineteen recognizers across two reactions took a bare `&str`. One file scoped its region nine times out of eleven;
the other scoped it **zero** times out of seven, and two of the six measured defects live there.

## Goals / Non-Goals

**Goals.** Make the region a property is about impossible to get wrong by accident. Close the four defects that
came from getting it wrong.

**Non-Goals.** Forbidding `&str` anywhere. A path and a needle are legitimately `&str` — `twin_of(gate: &str)` takes
a path — and the type cannot tell a path from a corpus, so a blanket ban would invent violations on half the
signatures. What is removed is the *corpus* being available as `&str`, not the type.

## Decisions

### D1 — Newtypes, not a trait

What is failing is **refusal**, not uniformity. A trait gives uniformity: `fn holds<R: Region>(r: R)` accepts any
region again, and the defect returns. Newtypes give refusal — a recognizer that wants executed text cannot be handed
a header.

This is the distinction a design critique on this window got wrong twice, proposing a trait and a macro for things
whose mechanism has no force where it was aimed. The question is not "can this be typed" but "does the type refuse
the thing that went wrong".

### D2 — No governance rule forbidding `&str`, and the reason is self-referential

A guard asserting "no recognizer takes a bare `&str`" would itself be a text reaction over Rust source — the exact
fragility this abstraction exists to replace, and one review measured that fragility on the existing `dyn` guard
(top-level files, single-line, `pub `-prefixed only). Using a weaker copy of the problem to enforce the fix would be
self-undermining. The type is the enforcement.

### D3 — `whole()` exists and is named

Some properties are genuinely about the whole blob. Removing the escape would push authors to `.header().text()`
or a fresh `read_to_string`, which is worse: the region decision would move back out of sight. Named, it greps.

### D4 — The two requirement tightenings are corrections, not extensions

`test -f scripts/check_x.sh` satisfying "this gate is invoked" is not a loose check — it is the **wrong** check.
Same for a second blessed document in an existing holder. So the requirements say invocation and per-call-site
rather than gaining an extra clause, and the projections re-render where a cell moves.

## Risks / Trade-offs

**`Prose`'s HTML-comment exclusion is line-oriented.** A comment opening mid-line after real prose hides the whole
line, including the prose before it. That direction refuses rather than admits — it can only make a document look
*less* reachable, never more — and the requirement's subject is a reader, for whom a line ending in a comment is
already ambiguous.

**The region types live in `tests/support/`, so they are test-only.** A future reaction outside `tests/` would need
them moved. Deliberately not pre-empted: two callers is not the evidence for a published module, and the move is
mechanical when a third arrives.
