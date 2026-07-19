## 1. Shared identity model

- [ ] 1.1 Add validated `FindingKey` and `Finding` public types in xuanji with private invariant-bearing storage, read-only accessors, canonical ordering, JSON projection, and rejection tests for ambiguous keys.
- [ ] 1.2 Change `ViolationId` to constructor-only disjoint structured/legacy identity semantics, then replace `Violation::new` with the typed-identity signature and verify equality remains transitive and presentation/metadata do not affect structured identity.

## 2. Baseline migration

- [ ] 2.1 Implement version-2 structured baseline parsing, stable serialization, sorting, and de-duplication with owner/tracker metadata tests.
- [ ] 2.2 Implement explicit version-1 parsing and exact-text legacy matching for gate, stale detection, and metadata-preserving rewrite without adding fallback `ViolationId` equality.
- [ ] 2.3 Verify malformed/unknown versions fail, v1 gates unchanged text, wording changes are new only for v1, and every write upgrades to version 2.

## 3. Dimension-owned facts

- [ ] 3.1 Introduce guibiao-owned typed fact conversions for every crate and module finding shape, migrate its emitters, and test key injectivity across rule variants and identity-bearing values.
- [ ] 3.2 Introduce hunyi-owned typed fact conversions for every semantic finding shape, migrate its emitters, and test key injectivity across single-module and whole-crate findings.
- [ ] 3.3 Introduce louke-owned typed fact conversions for every runtime audit finding shape, migrate its emitters, and test key injectivity across seam and source-file findings.

## 4. Projections and compatibility

- [ ] 4.1 Add `finding_key` to violation and stale-entry JSON while retaining human `finding`; verify legacy stale entries project a null key and text/SARIF behavior is unchanged.
- [ ] 4.2 Update public re-exports and API documentation, then verify pacta and modou retain their existing builder, runner, check, coverage, projection, baseline, and type-name usage against local crates.

## 5. Verification

- [ ] 5.1 Update baseline fixtures, examples, and capability documentation to version 2 without changing any Cargo package version.
- [ ] 5.2 Run the repository Definition of Done gates and confirm the self-law projection and example reactions remain green.
