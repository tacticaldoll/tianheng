## Context

`crates/guibiao/src/` provides the token-level static observation engine for Tianheng. Over the 0.2.x series, fixes for inline modules, unconditional `#[path]` remaps, symlink traversal, and `#[cfg]` arm deduplication were added. This design outlines the internal cleanup of `module_scan/` and doc comment noise reduction.

## Goals / Non-Goals

**Goals:**
- Consolidate redundant path resolution helpers across `module_scan/reachability.rs`, `symbol_scan.rs`, and `use_scan.rs`.
- Refactor doc comments across `guibiao` source files to forward-looking shape descriptions.
- Ensure strict compliance with `RUSTDOCFLAGS="-D warnings" cargo doc --workspace --no-deps --all-features` and all DoD gates.

**Non-Goals:**
- No changes to `guibiao` public API or exported types (`Rule`, `ModuleRule`, `check`, `coverage`, `projection`, `baseline`).
- No heavy parser (`syn`) dependencies; core remains AST-free and `serde_json`-only.

## Decisions

### Decision 1: Consolidate Path Context Helpers
- **Choice**: Streamline redundant `path_vocab.rs` and `fs_walk.rs` helper usage inside `reachability.rs`.
- **Rationale**: Reduces code duplication while preserving exact path resolution semantics.

### Decision 2: High-Signal Doc Comment Noise Reduction
- **Choice**: Update docstrings to state the structural invariants protected by static checks rather than historical bug fixes.
- **Rationale**: Aligns with project governance (`AGENTS.md`) and keeps documentation clean and maintainable.

## Risks / Trade-offs

- **[Risk] Path traversal or reachability regression** → **Mitigation**: Run full test suite including all 0.2.2 edge-case regression tests and dogfood examples.
