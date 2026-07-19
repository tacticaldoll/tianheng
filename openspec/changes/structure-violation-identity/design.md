## Context

`xuanji::ViolationId` currently identifies a violation by `(target, rule, finding)`, where
`finding` is also the sentence shown to a human. Every observation dimension therefore has to
make presentation prose injective, and a wording-only improvement invalidates accepted-debt
baselines. The same strings enter `Violation::new` as six positional arguments, so a dimension can
compile after accidentally exchanging the target, rule, or finding.

The 0.2.0 line is the compatibility window for correcting that public model. pacta exercises the
top-level builder, runner, and guibiao check surface. modou additionally re-exports guibiao's
`Baseline`, `ViolationId`, `Report`, `Outcome`, and `Violation`, and calls the existing check,
coverage, baseline, and projection functions. Those names and functions therefore bound this
change. No reference consumer directly constructs or matches `Rule` or `ModuleRule`, but that
public rule-model question remains a separate change so identity migration stays reviewable.

## Goals / Non-Goals

**Goals:**

- Separate stable observed-fact identity from human finding presentation.
- Keep fact vocabulary and rendering owned by the dimension that observes the fact.
- Make a dimension construct a violation through typed identity rather than a six-field positional
  string API.
- Read and gate against existing version-1 baselines, while making all new writes version 2.
- Preserve the existing builder, runner, check, coverage, projection, and public re-export names
  used by pacta and modou.

**Non-goals:**

- Reshape `Rule`, `ModuleRule`, or adopter-written `Constitution` builders.
- Introduce a general recursive value/JSON model in `xuanji`.
- Move a fact schema into `xuanji`, or add a dependency from `xuanji` to an observation dimension.
- Add a new crate, dependency, command, version bump, or release operation.

## Decisions

### A constrained shared identity envelope

`xuanji` will expose a `FindingKey` made of a non-empty dimension namespace, a non-empty fact code,
and canonically ordered, uniquely named string fields. Construction rejects empty names and
duplicate field names. It is intentionally not an arbitrary JSON value: the envelope supports the
stable scalar facts Tianheng observes without becoming a second domain model.

Each dimension owns its typed fact vocabulary and converts one fact into a `Finding`, which pairs
the `FindingKey` with the human-readable text. This keeps the dependency direction acyclic:
dimensions depend on `xuanji`; `xuanji` never imports crate-, module-, semantic-, or runtime-specific
types. A dimension's conversion is the one source for both identity and presentation, making drift
between them locally testable.

### Typed construction and identity semantics

`ViolationId` remains the public carrier of `target`, `rule`, and human `finding` so existing report
and stale-baseline consumers can keep reading those fields. External struct-literal construction is
closed; its key storage is private and exposed through a read-only
`finding_key() -> Option<&FindingKey>` accessor. The public constructor accepts `target`, `rule`, and
a typed `Finding` and always stores a key, while only the version-1 baseline parser can store `None`
to represent honestly that the artifact never contained one. `FindingKey` likewise keeps its fields
private behind validated construction and read-only accessors. `Violation::new` becomes
`Violation::new(kind, id, reason, severity)` and rejects a parsed legacy id rather than converting
historical data into an unstructured live observation. A live `Violation` therefore stores a
non-optional key. This removes the adjacent target/rule/finding strings from violation construction
while leaving file, anchor, polarity, and baseline status as metadata.

`ViolationId` equality and ordering are tagged by identity provenance: two structured ids use
`(target, rule, finding_key)` and deliberately ignore rendered text; two legacy ids use their old
`(target, rule, finding)` triple; a structured id and a legacy id are never directly equal. These
disjoint equivalence classes preserve transitivity. Presentation changes therefore do not change a
newly observed identity, while the human text remains available in reports and baseline files.

### Explicit version-1 compatibility, never fallback equality

A parsed version-1 baseline entry carries a `ViolationId` whose optional key is `None`, so existing
`Baseline::stale` and `report_json` signatures can keep returning/accepting `ViolationId` and modou's
explicit `Vec<ViolationId>` pipeline still compiles. The baseline matcher compares that legacy id
to a current violation only by the old exact `(target, rule, finding)` triple. A version-2 entry
matches only `(target, rule, finding_key)`. This cross-provenance compatibility belongs to baseline
matching; ordinary `ViolationId` equality never falls back from a structured key to text.

`Baseline::stale` and metadata preservation use the entry's version-aware matcher. Thus an unchanged
version-1 entry still suppresses and preserves `owner`/`tracker`; changing only the rendered text
before rewriting a v1 baseline intentionally appears new because the legacy artifact has no stable
key to prove equivalence.

### Version-2 baseline and additive report projection

Every newly generated baseline writes version 2. Each entry keeps `target`, `rule`, and `finding` for
humans and adds `finding_key` with `namespace`, `code`, and canonical named fields. Sorting and
de-duplication use structured identity; owner/tracker remain metadata. Re-serializing a parsed v1
snapshot without a current report preserves v1 because no structured fact was observed from which
to derive truthful keys. The runner's write action rebuilds from current violations, upgrades that
readable v1 baseline to v2, and carries metadata for entries matched under the legacy rule.

JSON reaction output keeps the existing `finding` string and adds the same `finding_key` object to
each violation and stale version-2 entry. Existing function names and the surrounding report shape
remain unchanged. Text and SARIF stay human-oriented and need no identity payload because they are
not consumed as baseline files.

## Risks / Trade-offs

- **Two representations can drift.** Each dimension will centralize key and text production in its
  typed fact conversion and test that distinct observable facts have distinct keys.
- **A weak key can mask violations.** Namespaced codes, named fields, duplicate-name rejection, and
  per-dimension injectivity tests make collisions explicit. Resolver internals and presentation
  fragments are excluded from keys.
- **Legacy matching remains wording-sensitive.** That is unavoidable because v1 stores no other
  evidence. The behavior is restricted to parsed v1 entries and ends when the file is rewritten.
- **Version 2 is not readable by an old binary.** Upgrade is forward-compatible, not bidirectional.
  Operators needing binary rollback must retain the prior v1 file or regenerate it with the old
  binary before adopting a v2 write.
- **Public struct construction breaks.** This is deliberate in 0.2.0. Read access to the existing
  target/rule/finding names remains, while constructors, private key storage, and rejection of a
  parsed legacy id at live construction establish the new invariants.

## Migration Plan

1. Add the constrained `FindingKey`, `Finding`, and typed `ViolationId` construction in `xuanji`,
   including identity and JSON tests.
2. Add the v1 parser/matcher alongside v2 parsing and writing before changing observation emitters.
3. Give every current guibiao, hunyi, and louke finding shape a dimension-owned typed fact conversion,
   then switch all violation emission sites to the typed constructor.
4. Project `finding_key` in JSON and update baseline/report fixtures and documentation.
5. Verify the workspace gates plus pacta and modou as reference consumers against the local crates.

Rollback before a v2 baseline is written is a source revert. After a v2 write, source rollback also
requires restoring the retained v1 baseline because a 0.1.x binary correctly rejects an unknown
baseline version rather than silently treating it as empty.

## Open Questions

None. The public rule model and any ergonomic DSL over fact-key construction are intentionally
deferred to separate changes with their own consumer evidence.
