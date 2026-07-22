## 1. Modularization & Internal Refactoring

- [ ] 1.1 Create `crates/xuanji/src/util.rs` containing `pretty_json` and its proof doc comment.
- [ ] 1.2 Create `crates/xuanji/src/model.rs` containing `Severity`, `BoundaryKind`, `Polarity`, and `Outcome`.
- [ ] 1.3 Create `crates/xuanji/src/finding.rs` containing `FindingKey` and `Finding`.
- [ ] 1.4 Create `crates/xuanji/src/violation.rs` containing `Violation` and `Report`.
- [ ] 1.5 Create `crates/xuanji/src/baseline.rs` containing `ViolationId`, `BaselineEntry`, `BaselineFormat`, `Baseline`, `baseline_id_matches`, and `apply_baseline`.
- [ ] 1.6 Create `crates/xuanji/src/tests.rs` containing the comprehensive unit test suite.
- [ ] 1.7 Refactor `crates/xuanji/src/lib.rs` into module declarations (`mod model;`, etc.) and `pub use` re-exports.

## 2. Doc Noise Reduction & Spec Alignment

- [ ] 2.1 Refactor doc comments across all `xuanji` submodules into concise, high-signal forward-looking shape descriptions.
- [ ] 2.2 Verify full alignment of invariants against `structured-violation-identity`, `violation-baseline`, and `rule-model-surface` specs.

## 3. Pre-flight DoD Verification

- [ ] 3.1 Run `cargo build --workspace`
- [ ] 3.2 Run `cargo clippy --all-targets --all-features -- -D warnings` and `cargo clippy --workspace -- -D warnings`
- [ ] 3.3 Run `cargo clippy -p louke -- -D warnings`
- [ ] 3.4 Run `cargo fmt --all --check`
- [ ] 3.5 Run `TIANHENG_WORKSPACE_TESTS=1 cargo test --workspace --all-features`
- [ ] 3.6 Run `RUSTDOCFLAGS="-D warnings" cargo doc --workspace --no-deps --all-features`
- [ ] 3.7 Run `cargo deny check`
- [ ] 3.8 Run `bash scripts/test_release_coherence.sh` and `bash scripts/check_release_coherence.sh`
- [ ] 3.9 Run `bash scripts/test_examples.sh`
