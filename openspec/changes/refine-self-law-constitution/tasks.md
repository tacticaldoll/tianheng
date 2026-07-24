## 1. Stage 1 — Prose Governance (AGENTS.md)

- [x] 1.1 Update `AGENTS.md` section `Writing a boundary's reason — for 潛移 (gravity)` to define the Three-Layer Governance Architecture and enforce forward-shape-only reason distillation.

## 2. Stage 2 — Self-Governance Hardening (self_governance.rs & AGENTS.self-law.md)

- [x] 2.1 Distill `because(...)` strings in `crates/tianheng/tests/self_governance.rs` into forward-looking shape declarations, removing historical debug text.
- [x] 2.2 Remove the redundant `.forbid_dependency_on(["tianheng"])` boundary on `guibiao` while keeping the core ⊥ shell clause inside `guibiao` allowlist's `because(...)`.
- [x] 2.3 Upgrade `canonicalize` confinement boundaries in `crates/tianheng/tests/self_governance.rs` for `guibiao`, `hunyi`, and `louke` to crate-root targets (`module("crate")`) with `ScanDepth::Subtree`.
- [x] 2.4 Regenerate `AGENTS.self-law.md` using `BLESS=1 cargo test -p tianheng self_law_projection_is_fresh`.

## 3. Stage 3 — Pre-flight Verification & Release Coherence

- [x] 3.1 Run full Definition of Done test suite (`cargo clippy`, `cargo fmt`, `cargo test`, `scripts/check_release_coherence.sh`) to verify zero regressions.
