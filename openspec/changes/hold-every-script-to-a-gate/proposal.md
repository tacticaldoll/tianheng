## Why

`repository-checks` says of its own retired predecessor that *"`git ls-files scripts/` names only wrappers, no
gate"*, and its Purpose says none of this repository's checks is product. Both are true today and neither is
held: the direction that enumerates the scripts folds every citation into one list and asserts that **list** is
non-empty, so a script contributing nothing is invisible as long as some sibling contributes something.

A script contributing no gate citation is precisely a script that judges by itself — the shape this window
deleted 1562 lines of. That shape had a whole capability describing it, `check_*.sh` gates paired with
`test_*.sh` twins over a `scripts/lib/` shared library, and the way back to it is not blocked by anything.

The enumeration, the extractor and the vacuity guard already exist. What is missing is the direction that runs
per member rather than over the aggregate.

## What Changes

- `kanhe::gate_identity` gains the judgement *which enumerated scripts cite no gate at all*, returning the
  shared kinded refusal like every other judgement in the crate, with its failure matrix beside it.
- `crates/kanhe/tests/gate_identity.rs` asserts it over the tracked set, so every tracked `scripts/*.sh` must
  defer its verdict to a Rust gate it names by `--exact`.
- `repository-checks` states the obligation and what it costs: `scripts/` becomes a **closed category** — a
  script that is not a wrapper cannot be added there without amending this requirement.

Not breaking, and no version moves: `scripts/` and `crates/kanhe/` ship in zero packages.

## Capabilities

### New Capabilities

<!-- none: the obligation belongs to the capability that already owns the wrapper shape -->

### Modified Capabilities

- `repository-checks`: the requirement over the wrappers' shape gains *every tracked script defers its verdict
  to a named gate*, held per script rather than over the aggregate, with the scenario for a script that cites
  none.

## Impact

- `crates/kanhe/src/gate_identity.rs` — one judgement, no new dependency, no change to `citations`,
  `logical_lines`, `registered_names` or `offences`.
- `crates/kanhe/src/tests/gate_identity.rs` — the failure matrix for it, synthetic input like its nine
  siblings.
- `crates/kanhe/tests/gate_identity.rs` — the per-member assertion over the tracked set.
- `openspec/specs/repository-checks/spec.md` — one requirement extended, one scenario added.
- `CHANGELOG.md` — one entry under `### Self-governance`.
- No published crate, public signature, wire format, exit class, baseline or manifest is touched.

### Capabilities touched without a requirement change

- `release-coherence`: `CHANGELOG.md` is its declared subject and this change writes one entry under
  `### Self-governance`. None of its requirements move.
