## Why

Tianheng's self-law constitution (`self_governance.rs`) and working agreement (`AGENTS.md`) carry three structural and prose inefficiencies:
1. Boundary `because(...)` reasons contain historical debug logs (e.g., 0.2.2 lesson details) instead of clean forward shapes, inflating LLM agent context tokens without strengthening compliance.
2. `guibiao` carries a redundant `.forbid_dependency_on(["tianheng"])` boundary alongside its `.restrict_dependencies_to(["serde_json", "xuanji", "xingbiao"])` allowlist, violating Tianheng's own "Minimalism forbids redundant reaction" rule.
3. Inline `canonicalize` call restrictions are applied to four specific submodules rather than at the crate-root level with `ScanDepth::Subtree`, leaving potential false-negative blind spots for newly added file-walking modules.

Refining the prose governance in `AGENTS.md` first and hardening `self_governance.rs` second aligns the working agreement with the enforced law and produces a cleaner, denser `AGENTS.self-law.md` projection.

## What Changes

- **Prose Governance (`AGENTS.md`)**: Update the `Writing a boundary's reason — for 潛移 (gravity)` section to formalize the Three-Layer Governance Architecture (Layer 1 Reaction Backstop, Layer 2 Qiányí Gravity Pull, Layer 3 Provenance & History) and enforce forward-shape-only `because(...)` reasons.
- **Self-Law Refinement (`crates/tianheng/tests/self_governance.rs`)**:
  - Distill all `because(...)` reasons into forward-looking, dense idiom statements; remove historical debrief text.
  - Remove the redundant `.forbid_dependency_on(["tianheng"])` boundary on `guibiao` while preserving the core ⊥ shell clause inside `guibiao`'s allowlist reason.
  - Upgrade `std::fs::canonicalize` confinement boundaries for `guibiao`, `hunyi`, and `louke` to crate-root targets (`module("crate")`) with `ScanDepth::Subtree` coverage.
- **Projection Update (`AGENTS.self-law.md`)**: Regenerate the byte-checked self-law projection via `BLESS=1 cargo test`.

## Capabilities

### New Capabilities
- None.

### Modified Capabilities
- `self-law-projection`: Refines the projected self-law requirements for distilled forward-shape reasons, minimalism in allowlists, and crate-wide `canonicalize` subtree confinement.

## Impact

- **Code & Test Suite**: Modifies `crates/tianheng/tests/self_governance.rs` and regenerates `AGENTS.self-law.md`. All workspace tests and self-governance gates remain green.
- **Documentation**: Updates `AGENTS.md` working agreement.
- **Context Overhead**: Reduces context size of `AGENTS.self-law.md` by stripping historical narrative.
