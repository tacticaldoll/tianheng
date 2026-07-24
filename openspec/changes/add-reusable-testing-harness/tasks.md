## 1. Implementation of `GovernanceTest` Harness

- [x] 1.1 Create `crates/tianheng/src/testing.rs` with `GovernanceTest` struct, constructors, and `assert_clean()` logic.
- [x] 1.2 Implement `assert_all_workspace_members_covered()` in `GovernanceTest` using `guibiao::check_and_cover` and `xingbiao` metadata.
- [x] 1.3 Implement `assert_projection_fresh()` with `BLESS=1` auto-regeneration and byte-level freshness assertion.
- [x] 1.4 Export `pub mod testing;` and `pub use testing::GovernanceTest;` in `crates/tianheng/src/lib.rs` and prelude.

## 2. Dogfooding & Verification

- [x] 2.1 Refactor `crates/tianheng/tests/self_governance.rs` to consume `GovernanceTest`.
- [x] 2.2 Verify full Definition of Done pre-flight suite (`cargo clippy`, `cargo test`, `cargo doc`, `cargo deny`, `scripts/check_release_coherence.sh`, `scripts/test_examples.sh`).
