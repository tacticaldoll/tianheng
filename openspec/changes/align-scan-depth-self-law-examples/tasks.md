## 1. Self-Law & Teaching Convention Alignment

- [x] 1.1 Update `crates/tianheng/tests/self_governance.rs` module-level boundaries (`must_not_call_inline`) to explicitly chain `.depth(ScanDepth::Subtree)`.
- [x] 1.2 Verify `AGENTS.self-law.md` freshness via un-blessed `cargo test -p tianheng self_law_projection_is_fresh` byte comparison test.

## 2. Cookbook & Dogfood Example Update

- [x] 2.1 Update `COOKBOOK.md` layer purity snippet to include `.depth(ScanDepth::Subtree)`.
- [x] 2.2 Update `examples/guibiao-standalone/src/governance.rs` to demonstrate explicit `.depth(ScanDepth::Subtree)` configuration on `ModuleBoundary`.

## 3. Backlog Ledger Update

- [x] 3.1 Move self-governance observation depth upgrade entry in `BACKLOG.md` from `WATCH` to `BUILT / HISTORY`.

## 4. Definition of Done & Pre-flight Gate

- [x] 4.1 Run workspace unit tests & clippy: `TIANHENG_WORKSPACE_TESTS=1 cargo test --workspace --all-features` and `cargo clippy --workspace -- -D warnings`.
- [x] 4.2 Run examples dogfood suite: `bash scripts/test_examples.sh`.
- [x] 4.3 Run release coherence check: `bash scripts/check_release_coherence.sh`.
