## 1. Doc Comment Noise Reduction & Refactoring

- [ ] 1.1 Refactor doc comments in `crates/hunyi/src/lib.rs` and `model.rs` into concise forward-looking invariant descriptions.
- [ ] 1.2 Refactor doc comments in `crates/louke/src/lib.rs` and `audit.rs` into concise forward-looking invariant descriptions.

## 2. Pre-flight DoD Verification

- [ ] 2.1 Run `cargo build --workspace`
- [ ] 2.2 Run `cargo clippy --all-targets --all-features -- -D warnings` and `cargo clippy --workspace -- -D warnings`
- [ ] 2.3 Run `cargo clippy -p louke -- -D warnings`
- [ ] 2.4 Run `cargo fmt --all --check`
- [ ] 2.5 Run `TIANHENG_WORKSPACE_TESTS=1 cargo test --workspace --all-features`
- [ ] 2.6 Run `RUSTDOCFLAGS="-D warnings" cargo doc --workspace --no-deps --all-features`
- [ ] 2.7 Run `cargo deny check`
- [ ] 2.8 Run `bash scripts/test_release_coherence.sh` and `bash scripts/check_release_coherence.sh`
- [ ] 2.9 Run `bash scripts/test_examples.sh`
