# Proposal: Code Refine and Harden Contracts

## Why

Following the documentation and backlog denoising pass, code-level rustdoc comments (`//!` and `///`) across all 6 workspace crates (`xuanji`, `xingbiao`, `guibiao`, `hunyi`, `louke`, `tianheng`) must be swept and purified for 0.3.0 alignment (`RuleKey`, `StructuredFactIdentity`, seam identity).

Furthermore, implicit behavioral intent embedded in doc comments must be hardened with explicit unit and conformance tests to ensure zero contract regression or loss during future agent interactions.

## What Changes

1. **Rustdoc Purification (`crates/*/src/`)**:
   - Audit module-level (`//!`) and item-level (`///`) doc comments across all 6 crates to ensure 100% precision with 0.3.0 semantics without stale terminology or legacy references.

2. **Contract Hardening & Conformance Verification**:
   - Verify that all key architectural invariants (such as `xuanji` model independence, `guibiao` syn-free scanner bounds, `hunyi` name-resolution hop caps, and `louke` runtime origin fail-closed invariants) are backed by explicit unit tests or self-governance assertions.
   - Add regression/conformance tests if any un-reacted prose intent is discovered.

3. **Internal Code Refinement**:
   - Clean up obsolete internal helpers or stale comments in `crates/hunyi/src/resolve/` and `crates/guibiao/src/`.

## Non-goals

- No altering of public API signatures, binary behavior, or wire formats.
- No macro consolidation of DSL builders (repetitive builders are designed-to-be-imitated for 潛移 gravity).
- No editing of steward-owned protected files (`self_governance.rs`, `constitution.rs`, `deny.toml`).

## Compatibility

- 100% backward compatible code doc purification and test hardening.
- Passes all pre-flight DoD gates (workspace build, 3 clippy passes, rustfmt, workspace tests, rustdoc `-D warnings`, cargo deny, release coherence scripts, example suite script).
