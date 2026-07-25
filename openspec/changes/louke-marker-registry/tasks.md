# tasks: Louke Custom Probe Macro Marker Registry Implementation Plan

- [ ] Refactor `crates/louke/src/audit/scan.rs` to support configurable marker sets (`&[&str]`) in `collect_probes` and `match_probe_marker`. <!-- id: 0 -->
- [ ] Expose `audit_probe_coverage_with_markers` in `crates/louke/src/audit.rs` and re-export in `crates/louke/src/lib.rs`. <!-- id: 1 -->
- [ ] Add unit tests in `crates/louke/src/audit/tests.rs` verifying custom marker recognition, unregistered marker exclusion, and word-boundary compliance. <!-- id: 2 -->
- [ ] Verify full pre-flight DoD gates (`cargo build`, `clippy -D warnings`, `cargo fmt --check`, `cargo test --workspace --all-features`, `self_governance`). <!-- id: 3 -->
