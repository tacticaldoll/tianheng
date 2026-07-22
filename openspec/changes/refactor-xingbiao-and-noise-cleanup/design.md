## Context

Tianheng (天衡) enforces architectural governance across static (`guibiao`),語義 (`hunyi`), and runtime (`louke`) observation dimensions, with `xuanji` (reaction model) and `xingbiao` (workspace metadata substrate) acting as shared substrates beneath them.

Recently, `xuanji` was modularized into internal submodules (`model`, `finding`, `violation`, `baseline`, `util`, `tests`), and doc comments across `guibiao`, `hunyi`, and `louke` were refactored for docstring noise reduction. `xingbiao/src/lib.rs` (314 lines) remains as a single file containing both primary metadata functions and tests (taking >50% of the file length). Furthermore, internal modules in `hunyi` and `louke` contain legacy doc comments that can be refactored into high-signal forward-looking invariant descriptions.

## Goals / Non-Goals

**Goals:**

- Extract `crates/xingbiao/src/tests.rs` out of `lib.rs` while keeping 100% of public crate root exports intact.
- Audit cross-crate utility logic for Twin-Drift bugs (ensuring `xingbiao` remains the Single Source of Truth for cargo metadata resolution).
- Refactor internal doc comment noise across `hunyi/src/collect.rs`, `hunyi/src/scan.rs`, `louke/src/audit/scan.rs`, `guibiao/src/module_scan/symbol_scan.rs`, and `guibiao/src/module_scan/use_scan.rs`.
- Verify full workspace Definition of Done (DoD) suite.

**Non-Goals:**

- Altering any public API signatures or JSON wire formats.
- Adding heavy dependencies (keeping `xingbiao` dependency-light with `serde_json` + `std` only; `syn` remains quarantined in `hunyi`).
- Weakening any self-governance boundaries in `crates/tianheng/tests/self_governance.rs`.

## Decisions

- **Decision 1: Extract `tests.rs` in `xingbiao`**:
  - *Rationale*: Keeps `lib.rs` clean and operational, matching the modularization pattern established in `xuanji`.
  - *Alternative Considered*: Keeping inline tests in `lib.rs`. Rejected because inline tests account for over 50% of the file and obscure core functions.

- **Decision 2: Forward-Looking Invariants for Docstrings**:
  - *Rationale*: Adheres to the 潛移 (gravity) principle in `PROJECT.md`—doc comments should state the forward shape the boundary protects, rather than historical bug narrative noise.
  - *Alternative Considered*: Retaining historical bug narratives. Rejected to maintain high signal-to-noise ratio across all crates.

- **Decision 3: Zero Requirement Delta Specs**:
  - *Rationale*: Internal refactoring and doc cleanup do not alter spec-level adopter or CLI behavior.

## Risks / Trade-offs

- [Risk]: Inadvertently breaking internal type visibility during `xingbiao` test extraction.
  - *Mitigation*: Run `cargo test -p xingbiao` and `cargo test --workspace --all-features` to verify.
- [Risk]: Modifying self-governance protected files without steward review.
  - *Mitigation*: Ensure no changes are made to `crates/tianheng/tests/self_governance.rs`, `crates/tianheng/src/constitution.rs`, or `deny.toml`.
