## Why

This repository holds four generated documents and keeps a **hand-written list of them** in the document a
reader is told to open first.

`AGENTS.md` names `AGENTS.self-law.md`, `docs/observation-bounds.md`, `docs/observation-bound-extents.md` and
`docs/gate-shape-contract.md` in prose, across two paragraphs written at four different times. Nothing enumerates
the projections. Nothing checks that list. A fifth reaction added tomorrow projects a document, holds it fresh,
and is named nowhere — and no reaction, gate or test finds out.

That is precisely the class the last two changes closed one level down, and it is being carried by the
capability-of-capabilities: the mechanism whose whole purpose is to stop documents drifting is itself described
by a document that drifts. `crates/tianheng/src/testing.rs` already says as much about its own duplication — "the
mechanism whose whole purpose is to stop documents drifting is a poor place to duplicate" — and the same sentence
applies to its inventory.

Three facts measured on the tree rather than recalled, and they decide the shape:

1. **The surface is already self-declaring.** Every one of the four projections carries the marker **bolded and
   above its first `##` heading** — measured at lines 4, 7, 10 and 22 against first headings at 18, 10, 50 and 29
   — and each names its own generator. So the enumeration needs no hand-kept list: a document declares itself
   generated, exactly as a spec scenario declares itself a bound. Both qualifiers are load-bearing rather than
   decorative: a rule on the bare string would enumerate this capability's own spec, which quotes the marker while
   requiring it.
2. **There are two generating mechanisms, not one.** Three projections are held by a Rust call to
   `assert_projection_matches` / `assert_projection_fresh_with_preamble`; `docs/observation-bounds.md` is written
   by `scripts/check_bound_register.sh` under `BLESS=1`. A reaction that scanned only Rust call sites would
   report a perfect bijection over three quarters of the surface.
3. **The holders and the documents already agree, 4 for 4.** So this register describes the tree rather than
   migrating it — the same position `gate-shape-contract` was in, and the reason its cost was bounded.

Why now: `PROJECT.md`'s audit-cycle decision names the remedy — enumerate a claim surface, react over it, audit
against the enumeration — and this is the fourth instance of that shape. The trigger is the one the third
instance left behind: closing `gate-shape-contract` added a fourth projection **and** a hand-edit to `AGENTS.md`
to mention it, which is the maintenance step that has no reaction behind it.

## What Changes

**A new capability, `projection-register`,** whose subject is the repository's own generated documents.

- **The surface is enumerated from tracked content by the marker each document carries**, and paired with the
  generator that document names. A document enters the register the moment it says it is generated.
- **The holders are enumerated independently**, from both mechanisms, and the two sets are held equal in **both
  directions**. Document-without-holder is a document claiming a freshness nobody asserts; holder-without-document
  is a projection no reader has been told to read.
- **Every projection must be named where a reader is sent to find it** — `AGENTS.md`. That is the check whose
  absence let the last change need a hand edit.
- **The register is itself a projection**, carries the marker, names its own generator, and appears in its own
  table. A register that exempted itself would be counting everyone else's unregistered documents.
- **What it does not check is declared**, not implied: whether a stated regeneration command actually regenerates
  its document, and whether a generated document produced by a *third* mechanism exists at all.

## Capabilities

### New Capabilities

- `projection-register`: the inventory of this repository's generated documents — how the surface is enumerated,
  the correspondence between a document and the reaction that holds it fresh, the requirement that a reader can
  find each one, and the projection that keeps the inventory honest.

### Modified Capabilities

None. `self-law-projection` owns `AGENTS.self-law.md`'s content and `constitution-projection` owns its rendering;
neither is changed by an inventory over them. `observation-bound-register` owns its own projection's freshness.
This capability owns exactly one obligation those do not: that the set of such documents is known.

## Impact

- **New**: a self-governance test module under `crates/tianheng/tests/`, and its generated projection under
  `docs/`.
- **Modified**: `AGENTS.md` — the *Self-governance* paragraph gains the new projection, and this is the last time
  that edit is unchecked.
- **Not affected**: no crate's public API, no `Constitution`, no baseline format, no gate. Repository-internal
  governance; version class **PATCH**.

## What this deliberately does not do

**It does not run the regeneration commands.** A Rust test invoking `BLESS=1 cargo test` re-enters the harness
that is running it, and invoking the shell gate under `BLESS=1` writes into the tree the test is judging. Both are
refused, and the consequence is declared as a bound rather than left as an impression of coverage: a header can
name a command that no longer regenerates anything, and this register will not see it.

**It does not check freshness.** Each holder already asserts that, and duplicating it here would be a second
implementation of the one rule `assert_projection_matches` exists to be. The register's subject is *registration*;
the projection's own header will say so where a reader meets the table.
