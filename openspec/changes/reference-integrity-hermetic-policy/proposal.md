## Why

The reference-integrity gate lets ambient `GOVERNANCE_DOCUMENTS` replace the repository's required governance
set. The failure matrix uses that escape to reach a zero-corpus fixture, but the same environment variable can
silently weaken a real run and make required-document absence depend on the caller's process state.

## What Changes

- Make the real governance-document set literal and immune to ambient environment variables.
- Replace the matrix's environment override with an explicit, non-empty fixture-only argument.
- Reject that fixture override for Tianheng's own workspace and reject unknown arguments as cannot-judge.
- Add a dedicated `reference-integrity` capability spec for the existing gate reaction.

## Capabilities

### New Capabilities

- `reference-integrity`: tracked in-repository references and required governance surfaces are judged under a
  hermetic real-workspace policy, with fixture narrowing explicit and confined.

## Impact

The local gate's test seam changes from an ambient environment variable to an explicit CLI argument. Normal gate
invocation is unchanged. Published crates, manifests, package versions, and adopter APIs are unaffected.
