## 1. Typed semantic observation model

- [ ] 1.1 Replace the kind-plus-descriptor envelope and rendering-only catalog with typed semantic fact variants that generate byte-identical text and fact-specific named keys.
- [ ] 1.2 Model every public seam shape as private structured data and carry it through path and shape collection without pre-rendering it.
- [ ] 1.3 Make semantic collectors and emitters pass typed facts through sorting, deduplication, file attribution, and `ViolationId` construction.

## 2. Contract and regression coverage

- [ ] 2.1 Add tests proving display-only wording changes leave live semantic keys unchanged and every identity-bearing fact value changes the key.
- [ ] 2.2 Add tests covering every public seam variant, including same-subject cross-seam injectivity and fail-loud unstamped exposures.
- [ ] 2.3 Verify existing semantic reports and version-1 baseline migration stay byte-compatible while version-2 semantic keys use named fields.

## 3. Governance and verification

- [ ] 3.1 Update the project decision and backlog horizon to record that structured semantic facts are built and remove the obsolete deferral.
- [ ] 3.2 Run the repository Definition of Done, strict OpenSpec validation, and adversarially review the implementation against the declared identity reaction.
