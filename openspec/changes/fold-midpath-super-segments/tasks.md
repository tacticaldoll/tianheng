## 1. Mid-Path Path Normalization (guibiao)

- [ ] 1.1 Implement `fold_canonical_segments` in `crates/guibiao/src/module_scan/path_vocab.rs` with stack-based `self`/`super` resolution and over-pop protection
- [ ] 1.2 Update `normalize_module_path` and `resolve_self_super` to use `fold_canonical_segments`
- [ ] 1.3 Update `symbol_scan` resolvers (`resolve_head` and `resolve_written_path`) to fold crate-rooted paths via `fold_canonical_segments`
- [ ] 1.4 Add unit tests in `guibiao::module_scan::path_vocab` testing mid-path `super`/`self` segment folding and over-pop boundaries

## 2. Scanner Integration & Lock Tests (guibiao)

- [ ] 2.1 Verify `use_scan`, `reachability`, and `symbol_scan` resolve grouped imports and inline symbol calls with mid-path `super` to fully collapsed canonical paths
- [ ] 2.2 Add unit tests in `guibiao::tests` asserting `must_not_import` and `must_not_call_inline` detect forbidden imports and calls containing mid-path `super` / `self`
- [ ] 2.3 Run full Definition of Done pre-flight gates (`cargo test`, `clippy`, `test_examples.sh`) to verify zero regressions
