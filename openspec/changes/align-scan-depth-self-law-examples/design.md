## Context

Following the ignition of `ScanDepth` toggles on `xuanji`, `guibiao`, and `hunyi` model surfaces (#115), Tianheng adopts explicit scan depth declarations in `self_governance.rs`, `COOKBOOK.md`, and `examples/guibiao-standalone`.

## Goals / Non-Goals

**Goals:**
- Update `self_governance.rs` to explicitly chain `.depth(ScanDepth::Subtree)` on `must_not_call_inline` module boundaries as an imitable teaching convention.
- Update `COOKBOOK.md` layer purity snippet and `examples/guibiao-standalone/src/governance.rs` to demonstrate explicit `.depth(ScanDepth::Subtree)` on `ModuleBoundary`.
- Move the corresponding item in `BACKLOG.md` to `BUILT / HISTORY`.

**Non-Goals:**
- Creating non-reactive normative SHALL requirements in `openspec/specs/` (since `ScanDepth::Subtree` is already default behavior, explicit chaining is an imitable convention, not a testable structural reaction).
- Over-promising `.depth(...)` calls on profile types (such as `SansIoPureDraft`) that construct `ModuleBoundary` internally with default depth without exposing `.depth()` methods.

## Decisions

### Decision 1: Explicit `.depth(ScanDepth::Subtree)` as Teaching Convention
Chain `.depth(ScanDepth::Subtree)` explicitly on `ModuleBoundary` in `self_governance.rs`, `COOKBOOK.md`, and `guibiao-standalone`.

### Decision 2: Preserve Normative Spec Integrity & Test Verification
Keep normative `openspec/specs/` free of non-reactive syntax requirements. Verify `AGENTS.self-law.md` freshness using `cargo test -p tianheng self_law_projection_is_fresh` without `BLESS` to ensure byte-comparison assertion is fully exercised.

## Risks / Trade-offs

- None. Self-governance freshness tests and dogfood example tests remain 100% green.
