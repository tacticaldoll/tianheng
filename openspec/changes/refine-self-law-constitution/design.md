## Context

Tianheng governs itself via `crates/tianheng/tests/self_governance.rs`, which generates `AGENTS.self-law.md`.
Currently, three governance style and enforcement issues exist:
1. `AGENTS.md` does not explicitly formalize the Three-Layer Governance Architecture or mandate that `because(...)` reasons must be forward-looking shape declarations without historical debug logs.
2. `self_governance.rs` has redundant boundaries (`guibiao.forbid_dependency_on(["tianheng"])`) and long, backward-looking `because(...)` strings.
3. `canonicalize` restrictions in `self_governance.rs` are targeted at specific submodules (`hunyi::module_resolve`, `guibiao::reachability`, `hunyi::scan`, `louke::audit::scan`) rather than crate-wide subtree confinement, leaving potential unmonitored call sites in new modules.

## Goals / Non-Goals

**Goals:**
- Update `AGENTS.md` working agreement to formalize the Three-Layer Governance Architecture and reason distillation discipline.
- Refine `crates/tianheng/tests/self_governance.rs`:
  - Distill `because(...)` strings to forward-looking, dense idiom statements.
  - Remove redundant `guibiao.forbid_dependency_on(["tianheng"])` while keeping the core ⊥ shell clause inside `guibiao` allowlist's `because`.
  - Upgrade `std::fs::canonicalize` boundaries for `guibiao`, `hunyi`, and `louke` to `module("crate").depth(ScanDepth::Subtree)` targets.
- Regenerate `AGENTS.self-law.md` using `BLESS=1 cargo test -p tianheng self_law_projection_is_fresh`.
- Ensure all CI gates and self-governance tests remain 100% green.

**Non-Goals:**
- Weaken any architectural invariants or relax dependency boundaries.
- Alter runtime behavior of `guibiao`, `hunyi`, `louke`, or `tianheng`.

## Decisions

### Decision 1: Two-Stage Execution Flow (AGENTS.md → self_governance.rs)
- **Rationale**: `AGENTS.md` is the Working Agreement for humans and agents. Establishing the Three-Layer Governance Architecture in `AGENTS.md` first ensures that the changes to `self_governance.rs` are executed as a direct application of agreed-upon operating protocol.
- **Alternatives Considered**: Modifying `self_governance.rs` without updating `AGENTS.md` (rejected: misses updating the human/agent protocol).

### Decision 2: Distill `because(...)` to Forward-Shape Only
- **Rationale**: Provenance and historical lessons (e.g. 0.2.2 lesson details) belong in `PROJECT.md` Decisions and `CHANGELOG.md`. Moving them out of `because(...)` strings reduces token bloat in `AGENTS.self-law.md` while optimizing for LLM continuation/imitation.
- **Alternatives Considered**: Keeping historical debriefs in `because(...)` (rejected: inflates LLM context without improving compliance).

### Decision 3: Remove Redundant `guibiao.forbid_dependency_on(["tianheng"])`
- **Rationale**: Tianheng's minimalism principle states that allowlists strictly supersede denylists. The allowlist `.restrict_dependencies_to(["serde_json", "xuanji", "xingbiao"])` already forbids `tianheng`. Merging the "functional core ⊥ imperative shell" clause into the allowlist's `because` preserves explicit documentation in `AGENTS.self-law.md` without double-reacting in code.
- **Alternatives Considered**: Retaining the redundant `forbid_dependency_on` boundary (rejected: contradicts Tianheng's own minimalism rule).

### Decision 4: Upgrade `canonicalize` Restriction to Crate Subtree
- **Rationale**: Applying `ModuleBoundary::in_crate("<crate>").module("crate").must_not_call_inline("std::fs").ending_with(["canonicalize"]).depth(ScanDepth::Subtree)` for `guibiao`, `hunyi`, and `louke` prevents any current or future submodule in those crates from calling `std::fs::canonicalize` directly without going through `xingbiao`.
- **Alternatives Considered**: Maintaining individual submodule lists (rejected: leaves blind spots for newly added modules).

## Risks / Trade-offs

- **[Risk] Test `dimension_boundaries_declare_the_mutual_independence_law` fails if `because(...)` drops the `三儀 ⊥ 三儀` string.**
  - *Mitigation*: Retain the exact string `"三儀 ⊥ 三儀"` in the distilled `because(...)` reasons for `guibiao`, `hunyi`, and `louke` allowlists.
- **[Risk] `AGENTS.self-law.md` staleness test fails in CI.**
  - *Mitigation*: Run `BLESS=1 cargo test -p tianheng self_law_projection_is_fresh` to update `AGENTS.self-law.md` and verify with `cargo test`.
