## 1. Internal Refactoring & Cleanups

- [ ] 1.1 Consolidate internal path context helpers in `module_scan/reachability.rs` and `fs_walk.rs`.
- [ ] 1.2 Clean up redundant scanner code in `module_scan/symbol_scan.rs` and `use_scan.rs`.

## 2. Doc Noise Reduction & Spec Alignment

- [ ] 2.1 Refactor doc comments across `guibiao` files (`lib.rs`, `model.rs`, `module_check.rs`, `module_scan/`) to forward-looking shape descriptions.
- [ ] 2.2 Verify alignment with static observation specs (`module-boundary`, `crate-source-boundary`, `crate-dependency-boundary`, `external-crate-confinement`).

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
