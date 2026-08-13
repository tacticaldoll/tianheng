# Clean carries its subject

## Why

`Outcome::Clean` conflates two facts an operator and an agent must be able to tell apart:

- *I observed a subject and found nothing wrong.*
- *I had no subject to observe.*

Every other outcome in the model carries its evidence. `Violation` carries eleven named fields;
`ConstitutionError` carries a reason; `BoundDecl` names what a reaction deliberately does not see.
`Clean` carries nothing at all, and it is the only public value in the six shipped crates that
**asserts the result of work** rather than naming a category — a sweep of all twenty public unit
variants found nineteen classifications and this one assertion.

The dual is missing. `Observer::bounds` has no default body, so a participant cannot join a run
without declaring **what it does not observe**. Nothing asks what it **did** observe.

The gap is not theoretical, and its blast radius is larger than a log line. Traced through the
runner:

    an observer reaches nothing
      → Outcome::Clean                        nothing can contradict it
      → report = Report::empty()              runner.rs, `_ => &empty`
      → baseline.stale(report) = EVERY entry  every entry matches no current violation
      → "stale baseline entry (no longer violated)" × N
      → --disallow-stale turns exit 0 into exit 1
      → the documented remedy is to prune them

On `ConstitutionError` the same empty report is noise on a run that already failed. On `Clean` the
run **succeeded**, so the advice to delete every suppression reads as authoritative.

This repository has already hit the shape locally and closed it by hand three times — a check
returning `(Vec<Refusal>, usize)` so its caller can tell an empty corpus from a clean one,
`xingbiao` returning a named corpus beside its anchor, and five hand-written `assert!(examined > 0)`
guards, one of which was **missing**: measured, 389 tracked files, zero inspected, `ok` reported.
Three local re-inventions is the signal that the concept belongs in the protocol rather than in each
participant.

## What changes

`Outcome::Clean` gains a payload: a `Subject` recording what the observation was asked to enforce
and how much of the workspace it reached. The type refuses the one combination that is a lie —
**declared something, reached nothing** — so the failure stops being an omission a participant can
forget and becomes a commission it must write.

Zero reached is *not* itself an error: an empty semantic bundle is a static-only adoption, which this
capability already protects deliberately. The invariant is relational, not a non-zero count.

## Why now, and why exactly this much

`Observer` was introduced in the 0.5.0 window and has **never shipped**, so the population of
third-party implementors is zero — the cheapest moment a protocol can change. `Outcome::Clean`
shipped at 0.4.0 as a unit variant, so giving it a payload is breaking, and 0.5.0 is already a
breaking release.

The scope is one change, and that is a measured claim rather than restraint. The whole public
contract of the six shipped crates was swept for the same class before this was written: twenty unit
variants (nineteen are classifications), and thirty-six functions returning `bool`, `Option`, `Vec`
or `Outcome`. The `bool` returns are predicates over a value in hand; the `Option` returns mean
*absent in the data*, not *never looked*; of the `Vec` returns, three are corpus producers that flow
into this same gap, one is `bounds()` — defensible, because an empty declaration is one you had to
write — and one is `stale`, which amplifies this gap rather than adding another.

Nothing else needs this window. `Outcome`, `Report` and `Violation` are all `#[non_exhaustive]`, so
enriching what a `Subject` carries later costs nothing. The expensive half is that `Clean` carries
anything at all; the rich half stays free.

## Capabilities

- `observer-protocol` — modified. The lifecycle gains the dual of `bounds()`: a participant that
  returns `Clean` states the subject it observed. The empty-bundle allowance is unchanged and is now
  expressible rather than implicit.
- `adopter-surface` — modified. `Outcome::Clean` is part of the promised surface, and its shape
  changes; the prelude membership does not.
- `repository-checks` — accounted for, not modified. Its checks construct `Outcome` only in
  fixtures, and the vacuity guards they hand-write stay theirs: this change does not reach into
  `kanhe`, and the local guards remain correct where a check is not an `Observer`.

## Impact

Breaking for anyone matching or constructing `Outcome::Clean`. The migration is mechanical and the
compiler names every site. Six shipped crates; five production construction sites; the three
dimensions each already hold both numbers in the same function that classifies the outcome, so no
value is threaded and none is invented.
