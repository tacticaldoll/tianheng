## Why

`check_bound_register.sh` reads its observation source — the repository's tracked files — through four
`git ls-files` enumerations, each consumed by `mapfile` from a process substitution. A process
substitution's exit status reaches no one: `set -o pipefail` covers a pipeline this shell runs, not a
subshell whose status nobody reads. So an enumeration that **fails** is indistinguishable from one that
found nothing, and the gate carries on with an empty list.

The three directions that produces, none of them the answer:

- **A false clean.** With the census enumeration failed, no tracked Markdown is examined, so a document
  writing a stale `N bounds across M capabilities` passes and an otherwise-clean gate exits `0`.
- **A false violation, twice.** With the tracked-path index failed, every `UNPINNED` tracker "names no
  path this repository tracks". With the citation enumeration failed, every `PINNED-BY` names a test "no
  function under crates/ defines". Both exit `1`, blaming the register for a `git` failure.
- **A wrong diagnosis.** The spec-file enumeration is caught by the vacuity guard, so it cannot pass —
  but it reports that no spec matched the glob, which is a claim about the repository rather than about
  the enumeration.

This is the class the sibling gate was repaired for one change ago: `check_reference_integrity.sh` now
captures its normalization's status precisely because a process substitution hid it. The same shape
survived here, in the file that states the tracked-content discipline most explicitly.

Two refusal directions also need declaring rather than merely implementing. The register's requirements
enumerate the reaction's refusals, and the absent-tracked-spec refusal added in this window is not among
them — so the reaction refuses in a direction the declared law does not name.

## What Changes

- Every `git ls-files` enumeration has its status checked **in the parent shell** before its output is
  consumed, and a failure is `cannot judge` (exit 2) rather than an empty list. One enumerator does it
  for all four call sites, so the discipline cannot be half-applied.
- Because the enumeration must be readable twice — once for its status, once for its content — it lands
  in a trap-owned temp file, the discipline this file already applies to every other lazily-created temp
  file.
- The reaction requirement gains the two scenarios: an enumeration that fails is refused rather than read
  as empty, and a tracked spec absent from the worktree is refused before the projection is written.
- The failure matrix gains the direction, with `git` stubbed to fail for the census enumeration alone.
- The residual is stated rather than left implied: process substitutions that read already-materialized
  data (the attribute-run `sed`, the id-table `awk`) are not enumerations of the observation source, and
  the comment says so instead of leaving a reader to infer the scope.

## Capabilities

### New Capabilities

None.

### Modified Capabilities

- `observation-bound-register`: the reaction requirement states that an enumeration the reaction depends
  on failing is a cannot-judge rather than an empty result, and declares the absent-tracked-spec refusal
  the window implemented without naming.

## Impact

- `scripts/check_bound_register.sh` — the enumerator gains a status check and a trap-owned buffer; four
  call sites route through it.
- `scripts/test_bound_register.sh` — the injected-enumeration-failure fixture.
- `openspec/specs/observation-bound-register/spec.md` — two scenarios and the requirement prose.
- `CHANGELOG.md` — an `[Unreleased]` entry; the gate's exit behaviour changes in a state where it
  previously reported clean.
- No Rust code, no public API, no wire format, no identity shape. `docs/observation-bounds.md` is
  unchanged: no bound's statement moves.
