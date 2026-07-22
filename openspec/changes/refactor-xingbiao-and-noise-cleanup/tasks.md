## 1. xingbiao Submodule Extraction & Cleanup

- [ ] 1.1 Extract `crates/xingbiao/src/tests.rs` from `lib.rs`
- [ ] 1.2 Refactor `crates/xingbiao/src/lib.rs` docstrings into concise forward-looking invariant descriptions

## 2. Twin-Drift & Duplication Review

- [ ] 2.1 Audit cross-crate utility logic (`PathBuf` resolution, error formatting) to prevent twin-drift
- [ ] 2.2 Streamline test fixture setup helpers across `guibiao` and `hunyi` tests

## 3. Internal Scanner Doc Comment Cleanup

- [ ] 3.1 Refactor doc comments in `crates/hunyi/src/collect.rs` and `crates/hunyi/src/scan.rs`
- [ ] 3.2 Refactor doc comments in `crates/louke/src/audit/scan.rs`
- [ ] 3.3 Refactor doc comments in `crates/guibiao/src/module_scan/symbol_scan.rs` and `use_scan.rs`

## 4. Verification and Sync

- [ ] 4.1 Run full Definition of Done (DoD) local pre-flight checks
- [ ] 4.2 Sync delta specs (`openspec sync`) and archive while pruning dated scaffolding
