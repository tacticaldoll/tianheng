# Change: the example participant declares its over-reaction too

## Why

Round 2 of this window's closing review, turned on `examples/observer-participant` — the example whose whole
subject is that a participant must declare what it does not observe.

Its participant **over-reacts, and declares no bound for it.** The rule is *every module file opens with a `//!`
header*, and the recognizer reads `text.lines().next()`. A file carrying a real module header **below a license
comment** —

```rust
// SPDX-License-Identifier: MIT
//! This file DOES carry a module header, just not on line one.
```

— is reported as `missing-header`. Measured: dropping that file into the example's `src/` makes the participant
report two violations where the fixture expects one.

Relative to the rule as *worded* this is correct: the file does not open with `//!`. Relative to the rule's stated
**reason** — *"a reader opening a file learns what it is for before reading it"* — it is wrong, because such a
reader learns exactly that. That gap between a rule and its reason is what `Reached::OverReacts` exists to name,
and the example declared only its out-of-reach bound.

The example is the worst possible place for this. It is the one artefact in the repository teaching a third party
how to join a run honestly, and it was demonstrating the mechanism of declaring bounds while quietly having one it
had not declared.

## What Changes

- The participant declares a **second computed bound**, `Reached::OverReacts`, for a header below a leading
  comment — id, shape, reason and pin all built with `format!` as the first one is.
- A test pins it: the license-header file reacts, which is the bound, and the same file with its header on line one
  does not, which keeps the pin from holding for the wrong reason.
- The example's README says it now demonstrates **two extents**, not just the mechanism — a shape the observation
  never reaches, and a shape it reaches and judges too harshly.
- `observer-protocol` requires the demonstration to declare **every** bound its participant has, so the example
  cannot teach the mechanism while withholding an instance of it.

## Why declare rather than fix

Skipping a leading comment block before looking for `//!` would trade this edge for others — a `/* … */` header, a
`#![allow]` above the doc comment — and would make the rule's wording diverge from what it does. The rule stays as
worded, and the distance between its wording and its reason is *stated*, which is the whole subject of the extent
model.

## Impact

- Affected specs: `observer-protocol`
- Affected code: `examples/observer-participant/**`
- No public API change, no version bump. The register is unaffected: an example outside the workspace declares no
  bound the family's register counts.
