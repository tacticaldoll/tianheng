## Why

A gate's own shape is convention, so every new gate re-learns it by breaking.

`BACKLOG.md` records the pressure, measured on the `v0.4.0..HEAD` window rather than recalled: six
structural classes recurred across the gate surface — a swallowed subshell status, a foreign exit code or a
1-versus-2 collapse, an enumeration passing after zero iterations, a matrix asserting non-zero instead of the
code, a gate that could not be pointed at a fixture, and a matrix absent or present-but-unrun — over 28 of
that window's 49 commits. Every one was repaired one site at a time, twice leaving a sibling behind, and one
repair (the shared `ERR` backstop) *inverted* a gate whose `fail` returned rather than exited, invisibly,
because that gate's matrix asserted only a non-zero status.

The Definition of Done binds the gate **list** to CI. Nothing binds a gate's **shape** to anything, so the
seventh gate inherits the shape only if its author reads six others first.

Two facts measured while shaping this change decide what it has to be, and the second corrects the backlog
entry that motivated it:

1. **The shape is three weeks old, not a convention several authors converged on.** Probed at `v0.4.0` and at
   `c5174a6`: backstop installed 0 of 4 gates → 6 of 6; fixture-addressable 1 of 4 → 6 of 6; twin asserts
   exit codes 0 of 5 matrices → 6 of 6 gate twins. `git log -S… -- scripts/` dates each to a single commit in
   this window. A capability presenting the shape as settled law would be describing its own recency; what
   makes it law is a reaction, which is what this change adds.
2. **The property table is not uniform today.** The backlog entry claims it "is uniform **today** and
   enforced **nowhere**"; the second half holds and the first does not. The assertion that a clean run prints
   nothing on stderr — the only one that catches a backstop printing cannot-judge while still exiting 0 —
   held in 2 of 6 gate twins. So this change's reaction does not merely freeze a good state; it has real gaps
   to close first.

Why now: the trigger recorded in `BACKLOG.md` has **fired**, and `PROJECT.md`'s audit-cycle decision names
the remedy — enumerate a claim surface, react over it, audit against the enumeration — rather than another
round of one-site repairs.

## What Changes

**A new capability, `gate-shape-contract`, whose subject is the repository's own gate surface.** Its
reaction enumerates that surface from tracked content, asserts the mechanically checkable properties of each
gate and its twin, declares the properties it cannot check as observation bounds, and holds a generated
projection of the result fresh.

- **The reaction is a Rust test under `crates/`, not a seventh shell gate.** `PINNED-BY` resolves only a
  harness-registered Rust function under `crates/`, so a shell-defended capability could not pin the three
  semantic bounds this one must declare — they would land `UNPINNED`, turning the bound projection's leading
  figure from "0 of 42" into "3 of 45". Reading repository paths from a Rust test is established
  (`TIANHENG_WORKSPACE_TESTS`, six crates, `crates/xuanji/src/tests.rs` already scanning `crates/` for
  forbidden identifiers), and so is holding a blessed projection fresh (`self_law_projection_is_fresh` over
  `AGENTS.self-law.md`). A consequence worth stating: the reaction rides the existing
  `TIANHENG_WORKSPACE_TESTS=1 cargo test --workspace --all-features` line, so it adds **no** Definition of
  Done entry and **no** CI step, where a shell gate would have added two of each.
- **The surface is the `check_*` gate and its twin**, enumerated from `git ls-files`, with two exemptions
  declared rather than implied by silence: the example/library matrices and the sourced libraries under
  `scripts/lib/` are outside it, and the publish-time gate is exempt from Definition-of-Done membership.
- **The shape gaps close in this change**, so the reaction lands green and every exemption in it is a
  declared bound rather than a backlog of known-failing units wearing the word.

## Capabilities

### New Capabilities

- `gate-shape-contract`: the structural contract every repository gate and its failure matrix hold — the
  enumeration of the surface, the properties asserted over it, the properties deliberately not asserted, and
  the generated projection that keeps the result honest.

### Modified Capabilities

None. `observation-bound-register` supplies the shape and is not changed by it; `governance-dogfood`'s
subject is the published boundary families, not the repository's own verdict surface.

## Impact

- **New**: a self-governance test module under `crates/tianheng/tests/`, and its generated projection under
  `docs/`.
- **Modified**: the four gate twins missing the silent-clean-run assertion gain it.
- **Modified**: `AGENTS.md` — no Definition of Done change (see above); the *Self-governance* paragraph gains
  the new projection beside `AGENTS.self-law.md` and `docs/observation-bounds.md`.
- **Closed at sync**: the `BACKLOG.md` entry that recorded this pressure, and the entry's claim of a uniform
  table corrected to the measurement above.
- **Not affected**: no crate's public API, no `Constitution`, no baseline format. This change is
  repository-internal governance; version class PATCH.
