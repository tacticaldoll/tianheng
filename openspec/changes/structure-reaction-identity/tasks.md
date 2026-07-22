## 1. Shared reaction identity (`feat(xuanji)!` commit boundary)

- [ ] 1.1 Inventory every `FindingKey`, `ViolationId`, baseline-version, canonicalization, and public re-export use; record the exact migration surface in the implementation diff or commit body.
- [ ] 1.2 Add validated vocabulary-neutral `RuleKey` and structured fact identity types with private canonical storage, scalar fields, semantic identifiers, and rejection tests for invalid construction.
- [ ] 1.3 Replace live `ViolationId` construction/equality/ordering with governed target + rule key + fact, keeping all presentation and diagnostics outside identity.
- [ ] 1.4 Add compile-time and unit reactions proving `xuanji` remains observation-vocabulary-free, serde_json-only, and impossible to mutate into an invalid identity.
- [ ] 1.5 Run focused `xuanji` tests/clippy/doc checks and commit this independently verifiable breaking model increment with its migration footer.

## 2. Semantic baseline (`feat(xuanji)!` commit boundary)

- [ ] 2.1 Replace numeric `BaselineFormat` and legacy entries with the concrete `tianheng.baseline/structured-facts` snapshot and exact structured matching.
- [ ] 2.2 Preserve strict optional owner/tracker annotations, identity sorting/de-duplication, stale reporting, and annotation carry-forward for supported exact matches.
- [ ] 2.3 Reject numeric, unmarked, unknown-format, malformed, and wrong-typed baseline documents with focused parse/match/round-trip tests.
- [ ] 2.4 Run focused baseline tests/clippy/doc checks and commit the removal of legacy compatibility as a separate breaking governance increment.

## 3. 圭表 rule and fact migration (`refactor(guibiao)!` commit boundary)

- [ ] 3.1 Catalog every shipped 圭表 rule/fact family and classify each rule parameter and fact field as identity-bearing or presentation-only.
- [ ] 3.2 Emit validated semantic rule keys and dimension-owned fact identities from builder/observation paths without opening direct rule/key construction.
- [ ] 3.3 Add catalog, production-emission, reorder, insertion, and injectivity reactions proving presentation is free while changed law/fact meaning re-keys identity.
- [ ] 3.4 Run focused rule-model, standalone 圭表, and external compile checks, then commit the independently verified static increment.

## 4. Dimension-owned fact migration (one commit per independently governed instrument/fact family)

- [ ] 4.1 Inventory every 渾儀 rule parameter and ordinal/index/placeholder fallback (`_#`, `trait_#`, impl/item ordinals and equivalents), classifying each as identity-bearing, presentation-only, an observed structural role, or an explicit scan error.
- [ ] 4.2 Decompose unsafe-site facts and their rule keys into form/module/owner/trait/name roles, preserve deliberate per-module anonymous-block coalescing, and add reorder/unrenderable injectivity reactions before a focused `refactor(hunyi)!` commit.
- [ ] 4.3 Change async exposure and its rule key to module/owner-role/canonical-owner/item seam identity, retain full signatures as diagnosis, and add the self-contained Pacta-shaped signature-stability fixture before a focused `refactor(hunyi)!` commit.
- [ ] 4.4 Migrate every remaining 渾儀 rule/fact family with exhaustive schema catalogs and cfg-split/reorder/insertion tests; verify no public identity contains scan position before the remaining focused semantic commit(s).
- [ ] 4.5 Migrate all 漏刻 rule/fact families to dimension-owned typed conversion with catalog, production-emission, reorder, and injectivity reactions; verify audit-off and standalone 漏刻 builds before a focused `refactor(louke)!` commit.

## 5. Facade and machine projections (`feat(tianheng)!` commit boundary)

- [ ] 5.1 Re-export the vocabulary-neutral identity inspection types through each promised standalone surface and the composed prelude; remove obsolete legacy-key exports with external compile-contract coverage.
- [ ] 5.2 Update baseline, reaction, and constitution JSON to project their existing data plus the exact semantic formats `tianheng.baseline/structured-facts`, `tianheng.reaction/structured-facts`, and `tianheng.constitution/declared-boundaries`.
- [ ] 5.3 Derive SARIF partial fingerprints solely from canonical violation identity under the semantic property key, remove `tianhengViolationId/v1`, and prove presentation/result-order stability.
- [ ] 5.4 Make `--baseline` fail with exit 2 on unsupported files and make `--write-baseline` refuse to overwrite them with actionable preserve/move-or-delete/regenerate guidance.
- [ ] 5.5 Verify standalone instrument reactions and `check_constitution` feed the same projection/gating implementation without adding a dimension/plugin trait or testing DSL.
- [ ] 5.6 Run focused CLI, JSON, SARIF, baseline safety, adopter-surface, and composed-check tests before committing the facade increment.

## 6. Documentation and compatibility evidence (`docs` commit boundary)

- [ ] 6.1 Update adopter documentation and changelog with the 0.3.0 break, semantic identity contract, annotation-preservation steps, and explicit absence of automatic migration.
- [ ] 6.2 Update package/API documentation to distinguish standalone instruments, `tianheng` composition, structured Outcome inspection, and deferred testing/plugin conveniences.
- [ ] 6.3 Check Pacta and other available reference consumers against local crates where feasible; record exactly which external consumers were checked and keep required CI independent of sibling repositories.

## 7. Apply-stage adversarial review and release gates

- [ ] 7.1 Conduct an adversarial implementation review: try to prove presentation still leaks into identity, fact-local evolution forces a global format change, or an ordinal was merely renamed; verify every finding against production paths and fix or explicitly reject it.
- [ ] 7.2 Challenge boundaries: confirm `xuanji` remains measure-only, 三儀 remain mutually independent and standalone, `tianheng` is the only composer, and no testing/plugin promise or self-law weakening slipped into the change.
- [ ] 7.3 Run every Definition of Done command from `AGENTS.md`, including self-governance/projection freshness, release coherence, examples, docs, deny, and all feature/default-feature configurations.
- [ ] 7.4 Confirm each checked task has its named evidence, the apply commits follow governance granularity with curated provenance/migration bodies, and no task was checked before verification.
