## Context

`crates/xuanji/src/lib.rs` has grown to over 1,200 lines, placing all domain entities, baseline snapshots, JSON formatters, and test suites in a single file. As part of 0.2.x refactoring, `xuanji` will be modularized into clean internal submodules without breaking any public interfaces.

## Goals / Non-Goals

**Goals:**
- Partition `xuanji` into focused internal submodules (`model.rs`, `finding.rs`, `violation.rs`, `baseline.rs`, `util.rs`, `tests.rs`).
- Re-export all public symbols at crate root (`lib.rs`) using `pub use` to maintain 100% API backward compatibility.
- Perform doc comment noise reduction by replacing historical bug-fix comments with concise forward-looking invariant descriptions.
- Align code documentation and invariants strictly with `openspec/specs/structured-violation-identity`, `openspec/specs/violation-baseline`, and `openspec/specs/rule-model-surface`.

**Non-Goals:**
- No public API or signature changes.
- No baseline V1 or V2 JSON wire format modifications.
- No new external dependencies (`serde_json` remains sole dependency).

## Decisions

### Decision 1: Internal Submodule Breakdown with Full Crate-Root Re-export
- **Choice**: Extract types into `model.rs` (`Severity`, `BoundaryKind`, `Polarity`, `Outcome`), `finding.rs` (`FindingKey`, `Finding`), `violation.rs` (`Violation`, `Report`), `baseline.rs` (`ViolationId`, `BaselineEntry`, `Baseline`, `apply_baseline`), `util.rs` (`pretty_json`), and `tests.rs`. Re-export all in `lib.rs`.
- **Rationale**: Keeps external consumers (`guibiao`, `hunyi`, `louke`, `tianheng`, and adopters) completely unaffected while organizing internal source files cleanly.

### Decision 2: Dedicated `tests.rs` File
- **Choice**: Move the comprehensive 430+ line unit test suite into `tests.rs` (`mod tests;` in `lib.rs`).
- **Rationale**: Isolates test logic from production code, keeping each module focused on its core responsibility.

### Decision 3: Doc Noise Reduction
- **Choice**: Refactor doc comments from historical patch narratives to forward-looking structural invariants.
- **Rationale**: Align with project governance (`AGENTS.md`) and ensure zero rustdoc warnings under `RUSTDOCFLAGS="-D warnings"`.

## Risks / Trade-offs

- **[Risk] Missing re-export or visibility regression** → **Mitigation**: Run full pre-flight verification (`cargo build --workspace`, `cargo test --workspace --all-features`, `cargo doc --workspace --all-features`, `cargo deny check`, `bash scripts/test_examples.sh`).
