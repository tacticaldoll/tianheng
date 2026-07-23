# Proposal: Docs Denoise History and Specs

## Why

Tianheng's orientation layer relies on the **潛移 (Qiányí)** gravity thesis: LLM agents act as autoregressive imitation engines that continue whatever idioms exist in their context. Accumulating historical artifacts, outdated terminology (such as 0.2.x finding formats), and 740+ lines of shipped `BUILT` backlog ledgers injects historical noise into the context window, causing subtle idiom drift.

Denoising historical ledgers and sweeping specs/docs for 0.3.0 consistency increases context gravity, shrinks token usage, and prevents architectural drift.

## What Changes

1. **Backlog Pruning (`BACKLOG.md`)**:
   - Extract historical `BUILT / HISTORY` entries (0.1.x ~ 0.3.0 shipped ledgers) into `docs/history/0.3.0-built-ledger.md`.
   - Keep `BACKLOG.md` lean (~200 lines), containing only Backlog Governance rules, Live Decision Index, and active `READY-PATCH` / `DESIGN-BREAKING` / `WATCH` / `ACCEPTED DEBT` / `DECLINED` items.

2. **OpenSpec Specs Sweep (`openspec/specs/*`)**:
   - Audit all 28 capability specs to ensure all prose descriptions and scenarios strictly reflect 0.3.0 semantics (`RuleKey`, `StructuredFactIdentity`, SARIF v2, async seam identity) without legacy prose ambiguity.

3. **Code Docs Sweep (`crates/*`)**:
   - Audit module-level (`//!`) and item-level (`///`) rustdoc comments across all 6 crates (`xuanji`, `xingbiao`, `guibiao`, `hunyi`, `louke`, `tianheng`) for post-0.3.0 accuracy.

4. **Contract & SOP Alignment (`PROJECT.md` & `AGENTS.md`)**:
   - Ensure cross-references between `PROJECT.md`, `AGENTS.md`, `BACKLOG.md`, and `docs/history/` are precise and single-source-of-truth.

## Non-goals

- No public API signatures, binary CLI behavior, or wire format changes.
- No editing of steward-owned protected files (`self_governance.rs`, `constitution.rs`, `deny.toml`).
- No deletion of historical context—historical records are preserved under `docs/history/`.

## Compatibility

- 100% backward compatible documentation and internal code doc maintenance.
- Passes all pre-flight DoD gates: workspace build, 3 clippy passes, rustfmt, tests (`TIANHENG_WORKSPACE_TESTS=1`), rustdoc, cargo deny, release coherence scripts, and example suite script.
