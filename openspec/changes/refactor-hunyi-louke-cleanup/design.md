## Context

`crates/hunyi/src/` handles `syn` AST observation (trait implementations, item visibility, inline strict checks), while `crates/louke/src/` handles runtime origin tracking and CI probe-coverage audits. This design outlines doc comment noise reduction and internal helper cleanup across both crates.

## Goals / Non-Goals

**Goals:**
- Clean up docstrings in `hunyi` and `louke` to state structural invariants clearly.
- Ensure strict compliance with `RUSTDOCFLAGS="-D warnings" cargo doc --workspace --no-deps --all-features` and all DoD pre-flight gates.

**Non-Goals:**
- No changes to public API or baseline identity structures in `hunyi` or `louke`.
- No new workspace dependencies.

## Decisions

### Decision 1: High-Signal Doc Comment Noise Reduction
- **Choice**: Update Rustdoc comments across `hunyi` and `louke` to concise, forward-looking shape descriptions.
- **Rationale**: Keeps documentation aligned with project governance and clear of historical patch commentary.

## Risks / Trade-offs

- **[Risk] Unused feature-gated items in `louke`** → **Mitigation**: Verify with `cargo clippy -p louke -- -D warnings` and default features passes.
