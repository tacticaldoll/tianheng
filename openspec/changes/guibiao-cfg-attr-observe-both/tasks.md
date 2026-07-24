## 1. Internal Scanner & Data Structure Refactoring

- [x] 1.1 Update `PathAttrKind` in `crates/guibiao/src/module_scan/reachability.rs` to support `ConditionalRemaps(Vec<PathRemapSpec>)`
- [x] 1.2 Refactor `attr_prefix_path_kind` to collect all `cfg_attr(..., path = "...")` occurrences across the attribute prefix
- [x] 1.3 Preserve direct `#[path = "..."]` precedence over conditional `cfg_attr` paths when both are present

## 2. Union-Scan Reachability Traversal

- [x] 2.1 Update module reachability walk in `crates/guibiao/src/module_scan/reachability.rs` to iterate over candidate conditional target paths
- [x] 2.2 Implement graceful missing-file resolution (`path.exists()` check) for `ConditionalRemaps` candidates without raising `Exit 2` scan errors
- [x] 2.3 Ensure deduplication (`try_visit` / `canonicalize`) across multiple candidate module paths

## 3. Unit Tests & Definition of Done Verification

- [x] 3.1 Add unit test scenarios in `reachability.rs` verifying union-scan over existing `cfg_attr(path)` targets, missing target skipping, nested `cfg_attr`, and direct `#[path]` precedence
- [x] 3.2 Verify all workspace unit tests and self-governance suite (`TIANHENG_WORKSPACE_TESTS=1 cargo test --workspace --all-features`)
- [x] 3.3 Run full Definition of Done pre-flight checks (`clippy`, `rustfmt`, `cargo deny`, `test_examples.sh`, `check_release_coherence.sh`)
