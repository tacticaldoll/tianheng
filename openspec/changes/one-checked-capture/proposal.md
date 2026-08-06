## Why

`BACKLOG.md` recorded a swallowed subshell status as this window's most recurring class — **nine mentions** — and
every recurrence was repaired one site at a time. This window's pre-release review found eight more, and **both
failure directions are now measured rather than argued**:

- `check_whitespace_hygiene.sh` given a `git ls-files --eol` that emitted one clean row and then exited 7 reported
  `whitespace hygiene ok (1 tracked text files)` and **exit 0** over a repository it had read one file of. The
  count fell from two to one in its own output and nothing reacted to it.
- `check_release_coherence.sh` given a `git log` truncated the same way concluded the tree was in **snapshot**
  state and reported `[Unreleased] must be empty` — **exit 1, a violation invented from a partial read**.

So the class produces a false clean *and* a false violation, from one mechanism. A vacuity guard cannot cover it:
`inspected -eq 0` was built for zero rows and a partial read gives one or more.

The eight sites are migrated (`795b808`, `1df4e88`) through one shared rule in `scripts/lib/capture.sh`. **What is
missing is the thing that stops the ninth.** Ten repairs of one class over one window is the definition of a shape
that needs a reaction rather than another repair — and `gate-shape-contract` is the reaction that already holds
this surface's shape.

## What Changes

**An eleventh property**: a gate SHALL NOT consume an observation source through `< <(…)` whose producer can fail.

- Recognized in **executed text**, on the `< <(` construct, with a builtin over data already in memory
  (`printf`, `echo`) permitted — that is exactly the two sites remaining in the tree after the migration, and
  they cannot fail on I/O.
- The projection gains a column; the surface reads 6 of 6 after the migration, so the property lands green.

**And the declared bound that covered this narrows rather than staying as it was.** `gate-shape-contract` already
declares *whether a read's status is checked in the parent shell is not observed*. Part of that is now observed, so
the bound is no longer true as written — and a bound that overstates what is unobserved is as misleading as one that
understates it: it tells an auditor a real check does not exist.

Its **heading is unchanged**, deliberately: the heading's slug is the bound's id, so renaming it would break the
citation and the typed declaration in one move. The body narrows to what remains — a status swallowed by a command
substitution or an unchecked pipeline — and the typed declaration's rationale narrows with it.

## Capabilities

### Modified Capabilities

- `gate-shape-contract`: an eleventh property over the gate surface, and the read-status bound narrowed to what the
  new property does not reach.

## Impact

- **Modified**: `crates/tianheng/tests/gate_shape_contract.rs` (one `PROPERTIES` entry, one recognizer), its
  projection, `crates/tianheng/src/bounds.rs` (the narrowed rationale), and `docs/observation-bound-extents.md`.
- **Not affected**: no gate changes — the migration already landed. No crate's public API, no `Constitution`, no
  baseline format. Version class **PATCH**.
