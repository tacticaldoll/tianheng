## Why

Tianheng currently uses a rendered `finding` sentence as both presentation and the third component
of baseline identity. That makes harmless wording changes churn baselines, forces every dimension to
encode injectivity into prose, and leaves the six-positional-argument `Violation::new` constructor
open to swapping adjacent `String` values. The 0.2.0 compatibility window is open now that pacta and
modou provide concrete reference-consumer bounds, so identity can become structured without
breaking the adopter-written boundary DSL.

## What Changes

- Introduce a dimension-agnostic structured finding key: a namespaced fact code plus named string
  fields. Each observation dimension owns its fact schema and human rendering; `xuanji` owns only
  the comparable identity envelope used by the shared reaction model.
- Keep the human `finding` text in violations and reports, but derive baseline matching from the
  structured key so presentation changes do not re-identify architectural drift.
- **BREAKING**: replace the positional `Violation::new(kind, target, rule, finding, reason,
  severity)` API with construction from a typed `ViolationId` carrying the structured finding key.
- **BREAKING**: extend the public violation identity model and write baseline format version 2 with
  structured finding keys. Continue reading and matching version-1 text baselines so adopters can
  upgrade without first regenerating their accepted-debt snapshot.
- Add the structured key to the machine-readable reaction report while retaining the existing
  human `finding` string and all existing report/projection entry points.
- Preserve the adopter-written `Constitution` and boundary builders, `tianheng::run`, guibiao's
  standalone check/coverage/projection/baseline functions, and the public type names modou re-exports.
- Do not reshape `Rule` / `ModuleRule`; their growth-safe public model is a separate 0.2.0 change.

## Capabilities

### New Capabilities

- `structured-violation-identity`: Defines the common structured identity envelope, dimension-owned
  fact schemas/rendering, and the separation between stable fact identity and human finding text.

### Modified Capabilities

- `violation-baseline`: Moves baseline matching and serialization to structured finding keys,
  writes version 2, and defines version-1 read/match migration without losing owner/tracker metadata.
- `cli-check-runner`: Adds the structured finding key to JSON reaction output while preserving the
  human finding and existing exit-code/presentation contract.

## Impact

- `xuanji`: public finding/identity types, `Violation` construction, baseline codec and matching.
- `guibiao`, `hunyi`, `louke`: dimension-local fact schemas and conversion at violation emission
  sites; no cross-dimension dependency.
- `guibiao` and `tianheng` projections: additive structured-key report field; existing function
  names and human text remain.
- Baseline files: new writes use version 2; version 1 remains readable and gate-compatible.
- Reference consumers: pacta's builder/run/check usage remains source-compatible; modou keeps the
  guibiao types and functions it re-exports and calls. External callers of `Violation::new` or
  struct-literal `ViolationId` must migrate in 0.2.0.
- Dependencies and crate graph: unchanged. No version field is changed by this work.
