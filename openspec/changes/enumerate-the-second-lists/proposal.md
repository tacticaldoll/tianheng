## Why

Three hand-kept lists sit beside something this repository already enumerates, with nothing holding them equal.
All three currently agree, which is what makes them worth closing now rather than after one drifts:

- `TYPES` in the squash-message gate is *"the Conventional Commit types `AGENTS.md` admits"* — a second copy of
  a list the contract states in prose. Diverge them and the gate refuses a subject the contract admits, or
  admits one it forbids.
- `EXAMPLES` in the dogfood suite is a second copy of `examples/`. A new example is silently unexercised by both
  its directions **and** by the CI job that runs them — a false negative in the gate that runs the product
  against itself. One example was added this window and kept in step by hand.
- The publish wrapper's arrival matrix proves five of the parser's thirteen forwarded arguments actually reach
  cargo, while the specification requires each admitted argument to be measured against the tool at a named
  version.

The identical shape is already held for one list — the wrappers, by `every_gate_running_wrapper_is_named`,
whose own documentation names this exact risk. That makes these the third, fourth and fifth instance, and one
change rather than three is what a class through repeated doors asks for.

## What Changes

- Each list is held against its enumerator, **both directions**: nothing enumerated is unnamed, and nothing
  named is absent.
- `repository-checks` states the obligation over a check's own constants; `governance-dogfood` carries the
  example-directory scenario.

## Capabilities

### Modified Capabilities

- `repository-checks`: a constant a check judges by SHALL be held against whatever enumerates its set, in both
  directions, wherever an enumerator exists.
- `governance-dogfood`: the example suite's declared set SHALL equal the tracked example directories.

## Impact

- `crates/kanhe/src/merge_message_gate.rs` and its repository check — the type list.
- `crates/shengmo/tests/examples_suite.rs` — the example list.
- `crates/kanhe/tests/publish_workflow.rs` — the arrival matrix.
- Two spec files, `CHANGELOG.md`.
- No published crate, signature, wire format, exit class, baseline or manifest is touched.

### Capabilities touched without a requirement change

- `release-coherence`: `CHANGELOG.md` and `AGENTS.md` are subjects of it; no requirement of it moves.
