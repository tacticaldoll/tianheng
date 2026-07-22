## Why

Tianheng 0.2.x still lets rendered rule/finding text and scan-order fallbacks leak into machine identity, so improving an observation can silently re-key a baseline or collapse distinct facts. The 0.3.0 breaking window should replace that compromise once, while preserving the product shape: each instrument observes independently, `xuanji` owns vocabulary-neutral reaction, and `tianheng` alone composes the instruments.

## What Changes

- **BREAKING** Replace numbered baseline generations with one semantic machine contract whose format identifies structured observed facts rather than migration order.
- **BREAKING** Remove v1/v2 baseline parsing and automatic upgrade behavior. Unsupported or unmarked baselines fail loud; `--write-baseline` refuses to overwrite them so governance annotations are not silently lost.
- Define violation identity as the conjunction of governed target, stable semantic rule key, and an open structured observed-fact identity. Human rule/finding wording, reason, severity, source location, anchor, polarity, diagnostics, and baseline annotations do not participate.
- Give each shipped observation dimension ownership of its typed fact schemas, canonical identity projection, and presentation. Keep the shared `xuanji` envelope free of 圭表/渾儀/漏刻 vocabulary and keep each instrument independently usable.
- Remove positional/scan-order identity fallbacks. Identity compatibility is reacted through schema catalogs plus reorder/insertion and injectivity tests, not a custom lint.
- Identify async exposure by its public seam (module, owner kind, canonical owner, and item name); retain the complete signature as diagnosis rather than identity. Use a Tianheng-local Pacta-shaped fixture to prove signature-only changes do not re-key the seam.
- Preserve optional baseline `owner` and `tracker` as non-identity governance annotations within the new semantic format.
- Emit `tianheng.baseline/structured-facts`, `tianheng.reaction/structured-facts`, and `tianheng.constitution/declared-boundaries` for Tianheng-owned machine documents, plus the SARIF partial-fingerprint key `tianheng/structured-fact-identity` derived only from canonical violation identity. External standards such as SARIF 2.1.0 and crate SemVer remain numerically versioned.
- Keep the current composed library/CLI surfaces. A future architecture-testing convenience layer and third-party dimension protocol remain enabled directions, not 0.3.0 API promises.

## Capabilities

### New Capabilities

None.

### Modified Capabilities

- `structured-violation-identity`: Replace the generic version-2 finding key with the semantic target/rule/fact identity algebra, dimension-owned schemas, open semantic identifiers, and non-positional compatibility reactions.
- `violation-baseline`: Replace v1/v2 compatibility and migration with one semantic structured-facts format, strict legacy rejection, overwrite safety, and retained non-identity annotations.
- `rule-model-surface`: Separate stable semantic rule keys from human-readable rule presentation without exposing direct construction of invalid rule variants.
- `semantic-async-exposure-boundary`: Make the public seam—not its complete signature or scan position—the async observation identity.
- `semantic-unsafe-confinement`: Fully decompose unsafe-site identity and prohibit scan-order fallbacks while preserving fact injectivity.
- `crate-dependency-boundary`: Remove version-1 presentation compatibility and express dependency/kind/feature observations as structured facts.
- `crate-source-boundary`: Move source-policy baseline matching to semantic rule and fact identity.
- `module-boundary`: Move module finding de-duplication to semantic rule and fact identity.
- `external-crate-confinement`: Preserve confined-crate/importer injectivity in the new identity algebra.
- `semantic-signature-coupling`: Preserve public-seam injectivity without rendered or positional identity fallbacks.
- `semantic-reexport-exposure`: Preserve exported-path seam identity in structured fact roles.
- `semantic-trait-impl-exposure`: Replace positional fallback identity with observed structural seam roles.
- `semantic-trait-impl-locality`: Move impl-locality matching and rule parameters to the new identity algebra.
- `semantic-dyn-trait-boundary`: Preserve shape/seam injectivity and stated rendering bounds under structured facts.
- `semantic-dyn-trait-operand-boundary`: Preserve operand-rule parity under structured facts.
- `semantic-impl-trait-boundary`: Preserve shape/seam injectivity under structured facts.
- `semantic-impl-trait-operand-boundary`: Preserve operand-rule parity under structured facts.
- `semantic-forbidden-marker`: Move marker-acquisition matching to structured facts.
- `semantic-visibility-boundary`: Move public-item matching to structured facts.
- `governance-dogfood`: React against semantic identity schemas without freezing presentation.
- `cli-check-runner`: Project the new structured identities and semantic machine-document formats, and derive SARIF fingerprints from canonical violation identity.
- `adopter-surface`: Expose the vocabulary-neutral reaction identity needed to inspect standalone and composed outcomes without promising a public dimension/plugin trait or a testing DSL.

## Impact

- Public Rust API changes in `xuanji` and re-exports from the instrument/facade crates are intentionally breaking for 0.3.0.
- All existing baseline files must be regenerated after explicitly preserving any desired `owner`/`tracker` annotations; no in-process adapter is retained.
- Identity emission changes across `guibiao`, `hunyi`, and `louke`; composition remains in `tianheng`, and no observation dependency is added to `xuanji`.
- JSON reports/baselines and Tianheng's SARIF partial-fingerprint property change machine contract. SARIF itself remains 2.1.0.
- Pacta informs the async-seam contract, but Tianheng CI remains self-contained and does not depend on a sibling checkout.
