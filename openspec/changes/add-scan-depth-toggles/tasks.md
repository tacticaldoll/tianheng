# Tasks: `add-scan-depth-toggles`

- [ ] Add `ScanDepth` enum to `xuanji::model` with `Shallow` (default), `Subtree`, and `Audit` variants <!-- id: 0 -->
- [ ] Re-export `ScanDepth` in `xuanji`, `guibiao`, `hunyi`, `louke`, and `tianheng::prelude` <!-- id: 1 -->
- [ ] Add `.depth(ScanDepth)` builder method to boundary builders across `guibiao` and `hunyi` <!-- id: 2 -->
- [ ] Bridge existing ergonomic modifiers (`.including_submodules()`) to set `ScanDepth::Subtree` <!-- id: 3 -->
- [ ] Update `self_governance.rs` and verify `AGENTS.self-law.md` projection freshness <!-- id: 4 -->
- [ ] Run full Definition of Done pre-flight checks (`cargo test`, `cargo clippy`, `cargo fmt`, `check_release_coherence.sh`) <!-- id: 5 -->
