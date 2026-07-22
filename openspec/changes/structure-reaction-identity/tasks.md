## 1. Expand the shared identity model (`feat(xuanji)` commit boundary)

- [x] 1.1 Inventory every `FindingKey`, `ViolationId`, baseline-version, canonicalization, production constructor, and public re-export use; record the migration surface in the implementation commit body.
- [x] 1.2 Add validated vocabulary-neutral `RuleKey` and `StructuredFactIdentity` types with private canonical storage, scalar fields, semantic identifiers, and rejection tests.
- [x] 1.3 Add an explicitly temporary structured `ViolationId` construction path beside the old path so instruments can migrate one at a time without breaking intermediate checkouts; do not expose this bridge as a compatibility promise.
- [x] 1.4 Add compile-time and unit reactions proving the new primitives are immutable and `xuanji` remains observation-vocabulary-free and serde_json-only.
- [x] 1.5 Run focused `xuanji` tests/clippy/doc checks and commit the additive expansion without claiming the final breaking invariant.

## 2. 圭表 rule and fact migration (`refactor(guibiao)!` commit boundary)

- [x] 2.1 Catalog every shipped 圭表 rule/fact family and classify each rule parameter and fact field as identity-bearing or presentation-only.
- [x] 2.2 Emit validated semantic rule keys and dimension-owned fact identities from every production builder/observation path while keeping the existing builder roles.
- [x] 2.3 Add catalog, production-emission, reorder, insertion, and injectivity reactions proving presentation is free while changed law/fact meaning re-keys identity.
- [x] 2.4 Run focused rule-model, standalone 圭表, and external compile checks, then commit the independently verified static increment.

## 3. 渾儀 rule and fact migration (focused `refactor(hunyi)!` commit boundaries)

- [x] 3.1 Inventory every 渾儀 rule parameter and ordinal/index/placeholder fallback (`_#`, `trait_#`, impl/item ordinals and equivalents), classifying each as identity-bearing, presentation-only, an observed structural role, or an explicit scan error.
- [x] 3.2 Decompose unsafe-site facts and their rule keys into form/module/owner/trait/name roles, preserve deliberate per-module anonymous-block coalescing, and add reorder/unrenderable injectivity reactions before its focused commit.
- [x] 3.3 Change async exposure and its rule key to module/owner-role/canonical-owner/item seam identity, retain full signatures as diagnosis, and add the self-contained Pacta-shaped signature-stability fixture before its focused commit.
- [x] 3.4 Migrate every remaining 渾儀 rule/fact production path with exhaustive schema catalogs and cfg-split/reorder/insertion tests; verify no public identity contains scan position before the remaining focused semantic commit(s).

## 4. 漏刻 rule and fact migration (`refactor(louke)!` commit boundary)

- [x] 4.1 Catalog all 漏刻 rule/fact families and classify rule parameters and fact fields as identity-bearing or presentation-only.
- [x] 4.2 Migrate every production emission path to dimension-owned structured identity with catalog, production-emission, reorder, and injectivity reactions.
- [x] 4.3 Verify audit-off, audit-on, and standalone 漏刻 builds/tests, then commit the runtime increment independently.

## 5. Contract the shared model (`feat(xuanji)!` commit boundary)

- [ ] 5.1 Confirm every workspace production emitter uses the structured constructor; add a reaction that fails if the temporary presentation-derived path remains in production use.
- [ ] 5.2 Make governed target + `RuleKey` + `StructuredFactIdentity` the only live `ViolationId` construction/equality/ordering path; remove `FindingKey`, legacy identity provenance, and the temporary bridge.
- [ ] 5.3 Keep presentation and diagnostics on `Finding`/`Violation` outside identity and prove wording, reason, severity, file, anchor, polarity, and diagnostic changes do not re-key.
- [ ] 5.4 Run focused `xuanji`, workspace compile, clippy, and doc checks; adversarially verify presentation cannot enter identity; then commit the breaking contraction with its migration footer.

## 6. Semantic baseline (`feat(xuanji)!` commit boundary)

- [x] 6.1 Replace numeric `BaselineFormat` and legacy entries with the concrete `tianheng.baseline/structured-facts` snapshot and exact structured matching.
- [x] 6.2 Preserve strict optional owner/tracker annotations, first-occurrence de-duplication, identity sorting, stale reporting, and annotation carry-forward for supported exact matches.
- [x] 6.3 Reject numeric, unmarked, unknown-format, malformed, and wrong-typed baseline documents with focused parse/match/round-trip tests.
- [x] 6.4 Run focused baseline tests/clippy/doc checks and commit the removal of baseline compatibility as a separate breaking governance increment.

## 7. Facade and machine projections (`feat(tianheng)!` commit boundary)

- [ ] 7.1 Re-export the vocabulary-neutral identity inspection types through each promised standalone surface and the composed prelude; add external compile-contract coverage.
- [ ] 7.2 Update baseline, reaction, and constitution JSON to project their existing data plus the exact semantic formats `tianheng.baseline/structured-facts`, `tianheng.reaction/structured-facts`, and `tianheng.constitution/declared-boundaries`.
- [ ] 7.3 Derive SARIF partial fingerprints solely from canonical violation identity under `tianheng/structured-fact-identity`, remove `tianhengViolationId/v1`, and prove presentation/result-order stability.
- [ ] 7.4 Make `--baseline` fail with exit 2 on unsupported files and make `--write-baseline` refuse to overwrite them with actionable preserve/move-or-delete/regenerate guidance.
- [ ] 7.5 Verify standalone instrument reactions and `check_constitution` feed the same projection/gating implementation without adding a dimension/plugin trait or testing DSL.
- [ ] 7.6 Run focused CLI, JSON, SARIF, baseline safety, adopter-surface, and composed-check tests before committing the facade increment.

## 8. Documentation and compatibility evidence (`docs` commit boundary)

- [ ] 8.1 Update adopter documentation and changelog with the 0.3.0 break, semantic identity contract, annotation-preservation steps, and explicit absence of automatic migration.
- [ ] 8.2 Update package/API documentation to distinguish standalone instruments, `tianheng` composition, structured Outcome inspection, and deferred testing/plugin conveniences.
- [ ] 8.3 Check Pacta and other available reference consumers against local crates where feasible; record exactly which external consumers were checked and keep required CI independent of sibling repositories.

## 9. Apply-stage adversarial review and release gates

- [ ] 9.1 Conduct an adversarial implementation review: try to prove presentation still leaks into identity, fact-local evolution forces a global format change, or an ordinal was merely renamed; verify every finding against production paths and fix or explicitly reject it.
- [ ] 9.2 Challenge boundaries: confirm `xuanji` remains measure-only, 三儀 remain mutually independent and standalone, `tianheng` is the only composer, and no testing/plugin promise or self-law weakening slipped into the change.
- [ ] 9.3 Run every Definition of Done command from `AGENTS.md`, including self-governance/projection freshness, release coherence, examples, docs, deny, and all feature/default-feature configurations.
- [ ] 9.4 Confirm each checked task has its named evidence, the apply commits follow governance granularity with curated provenance/migration bodies, and no task was checked before verification.
