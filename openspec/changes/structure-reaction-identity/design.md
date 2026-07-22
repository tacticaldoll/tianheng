## Context

The 0.2 line introduced `FindingKey` and baseline version 2 to stop presentation-only edits from invalidating accepted violations. That narrowed the immediate failure, but the public identity is still assembled from generic strings, several 渾儀 observations still use rendered signatures or traversal ordinals, rule identity is still the displayed rule string, and the baseline reader carries two historical identity systems. The result is "presentation as identity" one layer lower rather than a complete observation model.

Tianheng is a pre-1.0 development tool with no owned persistent datastore. A checked-in baseline is an adopter-controlled generated projection, so 0.3.0 can take one coordinated break instead of carrying adapters through the pure model. The product nevertheless has three simultaneous faces: each instrument can be used independently, `tianheng` composes all three for CI/agents, and Rust callers can inspect reactions. Those faces must share one reaction vocabulary without making `xuanji` understand any instrument.

Pacta provides a concrete async consumer: its sync and async registries preserve the same five operation meanings while the signatures differ, and its Tianheng law confines async exposure away from the kernel/lifecycle. This supports treating the public seam as identity and the complete signature as diagnosis. Tianheng will encode that evidence in a local fixture; its tests will not require the sibling repository.

Constraints from the self-law remain binding: `xuanji` is measure-only and depends only on `serde_json`; 圭表, 渾儀, and 漏刻 do not depend on each other; only `tianheng` composes dimensions; and no law is weakened to make the migration pass.

## Goals / Non-Goals

**Goals:**

- Make a violation's identity a lossless composition of stable rule meaning and structured observed fact meaning.
- Let presentation, diagnostics, and non-reactive governance annotations evolve without baseline churn.
- Let fact families evolve locally without creating global baseline v3/v4 migrations.
- Remove traversal position from all public identities while preserving injectivity.
- Keep every instrument independently usable and keep cross-instrument composition in `tianheng`.
- Give CI tools and agents explicit semantic machine contracts and stable SARIF fingerprints.

**Non-Goals:**

- Reading, upgrading, or overwriting 0.2.x baseline files.
- A public `Dimension`, `ObservedFact`, or baseline trait; runtime plugins; or dynamic dispatch across instruments.
- A new `tianheng::testing` assertion DSL. Rust callers continue to inspect `Outcome`; a convenience harness waits for a real second consumer.
- Making baseline annotations policy or moving boundary declarations out of Rust.
- Replacing external numeric standards such as SemVer, SARIF 2.1.0, Rust edition/MSRV, Cargo metadata, or process exit codes.

## Decisions

### 1. Use a product identity algebra, not a numbered baseline generation

`ViolationIdentity` is the conjunction:

```
governed target AND semantic rule key AND structured observed fact identity
```

The target says where the law applies, the rule key says which stable prohibition/allowance reacted, and the fact says what was observed. None can be an enum alternative because removing any component permits unrelated violations to collide.

The following never enter identity: rule/finding presentation, reason, severity, file, anchor, polarity, complete signature diagnostics, baseline status, `owner`, and `tracker`.

Alternatives rejected:

- Keep `(target, rule text, FindingKey)`: rule wording remains a migration trigger.
- Put all roles in one free-form map: callers can duplicate or omit outer roles and the type system cannot express the algebra.
- Make `Baseline` an enum over instruments: a composed reaction can contain facts from all instruments simultaneously; the relationship is a product, not a sum.

### 2. `xuanji` owns an open, vocabulary-neutral fact envelope

`xuanji` exposes a validated `StructuredFactIdentity`, a validated `RuleKey`, and the composed `ViolationId`. A fact identity contains:

- a semantic fact type identifier, for example `tianheng.fact/hunyi/unsafe-site`;
- a semantic shape identifier, for example `decomposed-site`;
- canonical, uniquely named scalar fields.

Identifiers are semantic slugs, immutable in meaning. A fact may add data only where its declared shape marks that data as non-identity/open; an incompatible identity-bearing shape gets a new semantic shape identifier. Adding another fact family or dimension does not change the enclosing baseline format.

Fields remain scalar and canonically ordered. Recursive arbitrary JSON is rejected: it would make canonical equality and compatibility auditing ambiguous. Construction state stays private behind validated constructors and read-only accessors.

Each dimension owns its typed internal fact enums and the one conversion that produces identity plus presentation. `xuanji` owns no crate/module/semantic/runtime vocabulary. No public trait is introduced because there is no second external implementor whose needs can validate it; the envelope supplies future compile-time extensibility without promising a plugin system.

### 3. Rule key and rule presentation are separate

Every rule family supplies a stable semantic `RuleKey`; projections continue to render human-readable rule text and parameters. The boundary builders remain the public construction path. Rule keys are inspected through reaction identity, not by opening direct construction of `Rule`/`ModuleRule` variants.

A parameter belongs in rule identity only when changing it changes what the declared boundary forbids or permits. The implementation will catalog each rule family and explicitly classify its parameters. This prevents a cosmetic renderer change from re-keying accepted debt while ensuring a materially different law cannot reuse the old identity.

### 4. Baseline is one concrete semantic snapshot

The on-disk document carries:

```json
{
  "format": "tianheng.baseline/structured-facts",
  "violations": []
}
```

It is a concrete `Baseline`, not a trait or a `BaselineFormat` enum. It can hold standalone-instrument or composed reactions because every entry uses the same vocabulary-neutral identity. Entries remain sorted and deduplicated by identity and retain human presentation plus optional `owner`/`tracker` annotations.

Only the exact semantic format is accepted. Numeric v1/v2, an unmarked document, or another format fails as an invalid baseline and produces exit 2 in gate mode. `--write-baseline` writes a missing file or regenerates a supported semantic baseline, carrying annotations across matching identities. If an existing file is unsupported or malformed, it refuses to overwrite it. The user must move/delete it or explicitly preserve annotations before generating a new snapshot.

This is intentionally not an expand/backfill/contract migration: Tianheng owns no datastore and 0.3.0 promises no 0.2 compatibility. Fail-loud overwrite safety exists to prevent accidental loss, not to provide an adapter.

### 5. Observation schemas eliminate positional identity

Every shipped fact family gets a compatibility catalog that pins semantic type/shape identifiers, canonical field names, and representative field values. Finite typed discriminators must be exhaustively classified.

Tests must prove the behavioral property, not merely grep syntax:

- reordering declarations does not change identities;
- inserting unrelated items does not change identities;
- mutually exclusive cfg branches remain distinct when they represent distinct seams;
- two distinct observed facts never collapse merely because a renderer cannot print one component;
- presentation/signature-only changes leave identity stable where the fact meaning is unchanged.

An auxiliary catalog may reject known ordinal forms (`_#`, `trait_#`, `ordinal`, `index`) in official identity fields, but that is not the proof. No custom lint is added: a lint would police representation while the required property is reaction stability and injectivity.

### 6. Async exposure identifies the public seam

An async fact's identity is module + owner kind + canonical owner + item name (plus a trait-impl role where required to keep seams distinct). Parameter and return types, generic spelling, and the implicit future are diagnostics/presentation only. This models the architectural seam that the boundary governs: a signature-only refactor of the same operation does not create a new accepted-debt identity.

The implementation will add a local Pacta-shaped registry fixture with the same operation names across signature changes. It proves seam stability without treating Pacta's domain `Identity` middleware as Tianheng violation identity and without adding a cross-repository CI dependency.

### 7. Unsafe sites are structurally decomposed

Unsafe-site identity records the site form, module, owner kind/owner, trait, and item name where those roles exist. Anonymous unsafe blocks retain the existing deliberate per-module coalescing semantics rather than gaining an ordinal. Unrenderable syntax must use an observed structural discriminator or fail the observation; it must not fall back to traversal position.

This change will begin with an inventory of every current ordinal/fallback path in 渾儀, then migrate each fact family with an injectivity test. It does not claim perfect compiler identity: macros and other stated observation bounds remain unchanged.

### 8. Machine documents name semantics; SARIF fingerprints use canonical identity

Tianheng-owned JSON machine contracts receive explicit semantic formats: `tianheng.baseline/structured-facts` for baseline snapshots, `tianheng.reaction/structured-facts` for reaction reports, and `tianheng.constitution/declared-boundaries` for constitution projections. Existing substantive fields and exit behavior remain intact; adding a compatible field must not imply a second version axis.

SARIF remains version 2.1.0. Its partial fingerprint property changes from `tianhengViolationId/v1` to `tianheng/structured-fact-identity`, with the canonical `ViolationId` serialization used directly as the value so no extra hash can collapse distinct identities. SARIF rule presentation and messages remain diagnostic.

### 9. Standalone instruments and the composer keep distinct promises

圭表, 渾儀, and 漏刻 each emit the shared reaction model and remain directly callable. They do not implement a new common instrument trait. `tianheng` remains the sole built-in cross-dimension composer and owns CLI orchestration, combined reports, baseline gate behavior, and projections that require the intersection.

Rust architecture tests are the existing composed/standalone pure check APIs plus structured `Outcome` inspection. A future macro/assertion facade can be added as a convenience after usage evidence; it must not become a second observation engine.

## Risks / Trade-offs

- **[Semantic slugs become disguised version numbers]** → Review every slug for stated meaning; forbid ordinal-only identifiers and require a new shape only for a semantic identity incompatibility.
- **[An omitted discriminator creates false negatives]** → Require per-family injectivity cases and production-path catalog tests, including cfg-split and unrenderable cases.
- **[Over-structuring makes harmless changes breaking]** → Classify presentation/diagnostic fields explicitly and test that their changes leave identity stable.
- **[Open envelopes imply unsupported plugins]** → Document them as reaction data, keep composer inputs closed to built-in dimensions, and expose no plugin trait.
- **[Legacy overwrite loses annotations]** → Refuse to overwrite unsupported existing files and print actionable regeneration guidance.
- **[Scope expands into a testing framework]** → Limit adopter API work to identity inspection and compilation reactions; leave macros/DSL in backlog.
- **[Pacta evidence becomes environmental coupling]** → Copy only the architectural shape into a fixture and optionally run sibling verification outside the required gate.
- **[A law amendment is hidden inside the refactor]** → Run self-governance after every apply increment; any change to `self_governance.rs` requires its own explicit reviewed amendment.

## Migration Plan

1. Land this proposal as one reviewed OpenSpec documentation commit.
2. Expand `xuanji` first with the validated identity primitives and a temporary structured construction path while retaining the old construction path only as migration scaffolding. This expansion commit does not claim the final breaking invariant.
3. Migrate 圭表, 渾儀, and 漏刻 production emission in independently verifiable commits; remove ordinal fallbacks with their behavioral tests. Each intermediate checkout remains buildable and the temporary path is visible debt, not a compatibility promise.
4. After every production emitter uses structured target/rule/fact roles, contract `xuanji`: remove `FindingKey` and the old presentation-derived construction path, make the typed algebra the only live `ViolationId`, and verify no caller can regress.
5. Replace the baseline document and matching, then switch `tianheng` JSON/SARIF projections and overwrite behavior to canonical identity.
6. Run the Pacta-shaped conformance fixture, external adopter compile checks where available, self-governance, and the full Definition of Done.
7. Document the 0.3.0 break: users preserve desired annotations, move/delete the unsupported baseline, then run `tianheng check --write-baseline <file>`.

Rollback is a git revert before 0.3.0 release. There is no mixed-format runtime mode and no post-release automatic downgrade path.

## Open Questions

None.
