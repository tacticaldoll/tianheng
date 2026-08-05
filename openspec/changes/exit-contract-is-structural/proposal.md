## Why

The register's exit contract — `0` clean, `1` violation, `2` cannot judge — is declared, and the reaction
does not hold it. Two faces of one class, both measured rather than argued:

- **A failure aborts with a foreign exit code and no diagnostic.** `parse_spec` reads each spec through
  `sed | awk`, and under `set -e` with `pipefail` a failing `sed` aborts the script with **sed's** status.
  Measured on this repository with `sed` stubbed to exit 4: the gate exits **4**, printing nothing at all.
  A consumer reading exit codes sees a status the contract does not define, and the operator sees no
  reason. Every unguarded command in the file can do this; `parse_spec` is the one a review found.
- **A partial enumeration is read as a complete one.** The harness's package list comes from
  `mapfile -t members < <(cd … && find … | sort)`, whose status the parent never sees. The
  previously-added guard catches only a *totally* empty result, so `find` emitting some members and then
  failing yields a short list that reads as authoritative. Measured with `find` stubbed to print one
  directory and exit 3: the gate reports `24 registered test names across 1 package(s)` and exits **1**
  with a cascade of false violations against citations in the packages it never enumerated.

The second is the same swallowed-status class two earlier changes closed for the reads of the observation
source; this call site was classified as guarded, and the guard was weaker than the classification.

## What Changes

- **The exit contract becomes structural.** An `ERR` trap maps any unguarded failure to `cannot judge`
  (exit 2) with the failing line, so the contract holds for every command in the file — the ones present
  now and the ones a later change adds — rather than for the ones someone remembered to wrap. Verified
  against the shapes this file relies on: a failure inside `if`, `||`, `&&`, an arithmetic guard, or a
  captured pipeline with its own handler does not fire it, under `set -E`, even inside a function.
- **`parse_spec` keeps its own refusal**, because a backstop that says "line 610" is worse than a
  diagnosis that names the spec it could not read. The trap is the floor, not the answer.
- **`find` is retired.** Packages are enumerated from **tracked** manifests through the same
  `read_tracked_files` every other read uses, so the member list is status-checked, comes from tracked
  content like the rest of the gate, and drops one external tool. A directory holding no tracked
  `Cargo.toml` is not a package, which is what `cargo test -p` would have discovered one step later.
- The matrix proves all three: an unhandled failure exits 2 rather than the utility's code, an unreadable
  spec names itself, and a partial member enumeration refuses instead of judging on a short list.

## Capabilities

### New Capabilities

None.

### Modified Capabilities

- `observation-bound-register`: the reaction requirement states that the exit contract binds **every**
  path, including a failure nobody anticipated, and that the package enumeration is tracked content like
  every other read.

## Impact

- `scripts/check_bound_register.sh` — the `ERR` trap, the `parse_spec` refusal, the member enumeration.
- `scripts/test_bound_register.sh` — three directions.
- `openspec/specs/observation-bound-register/spec.md` — requirement prose and two scenarios.
- `CHANGELOG.md` — an `[Unreleased]` entry.
- No Rust code, no public API, no wire format. `docs/observation-bounds.md` unchanged: no bound moves.
