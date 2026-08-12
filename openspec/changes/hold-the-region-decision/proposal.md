## Why

`crates/kanhe/src/region.rs` exists because six defects across two reviews were one shape — *the corpus was
taken to be the whole blob when the property was about a distinguished part of it*. It replaced a helper with a
type, `Source`, so the region decision is made once and carried.

Three more instances of that shape were found in this window's adversarial round, in the two checks that never
adopted it:

- an acquisition sweep whose prefix test runs against the text right after `=$(`, so **every environment-prefixed
  acquisition is outside its corpus** — including the gate invocation both wrappers exist for;
- a Definition-of-Done comparison built from every line of the workflow file, **comments included**, so a command
  appearing only in a YAML comment would satisfy it;
- a window proving each violation-class exit sits inside the verdict branch, assembled from raw lines while the
  scan five lines above it filters comments — **two scans of one file disagreeing about the same question**.

## What Changes

- Both checks take their corpus from `region::Source` rather than re-deciding it.
- The acquisition sweep strips leading `NAME=value` tokens, so an env-prefixed acquisition enters the corpus.
- `repository-checks` states the obligation, **and declares the residual it cannot reach**: a check that should
  have distinguished a region and simply did not is invisible to any scan, because nothing can see an absence.
- A reaction refusing an inline region decision was designed and **rejected on measurement** — recorded rather
  than left to be re-proposed.
- `AGENTS.md` stops half-enumerating the publish wrapper's allowlist and points at the parser that owns it.

## Capabilities

### New Capabilities

<!-- none -->

### Modified Capabilities

- `repository-checks`: the requirement over this repository's own checks gains *the region a check judges is
  taken from the shared classifier, never re-decided at the call site*, with the scenarios for an
  environment-prefixed acquisition and for two scans of one file disagreeing — and the declared bound for the
  absence case.

## Impact

- `crates/kanhe/tests/gate_exit_classes.rs`, `crates/kanhe/tests/dod_coherence.rs` — corpus routed through
  `Source`; the acquisition prefix test widened.
- `openspec/specs/repository-checks/spec.md`, `AGENTS.md`, `CHANGELOG.md`.
- No published crate, signature, wire format, exit class, baseline or manifest is touched.

### Capabilities touched without a requirement change

- `release-coherence`: `CHANGELOG.md` and `AGENTS.md` are declared subjects of it and of `repository-checks`;
  no requirement of `release-coherence` moves.
