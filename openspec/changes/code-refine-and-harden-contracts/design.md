# Design: Code Refine and Harden Contracts

## Overview

This design outlines the systematic code doc purification and contract hardening process across the 6 Tianheng workspace crates.

## Refinement and Hardening Architecture

```
┌─────────────────────────────────────────────────────────────────────────────────┐
│              Tianheng Code Refinement & Contract Hardening                      │
├─────────────────────────────────────────────────────────────────────────────────┤
│                                                                                 │
│ ┌──────────────────────┐  ┌──────────────────────┐  ┌────────────────────────┐ │
│ │ 1. Rustdoc Audit     │  │ 2. Contract Audit    │  │ 3. DoD Pre-flight      │ │
│ │ Sweep `//!` & `///`  │  │ Ensure every intent  │  │ Run 100% green DoD     │ │
│ │ in 6 crates          │  │ has a test reaction  │  │ validation suite       │ │
│ └──────────────────────┘  └──────────────────────┘  └────────────────────────┘ │
│            │                          │                         │               │
│            └──────────────────────────┼─────────────────────────┘               │
│                                       ▼                                         │
│                    【 Outcome: 100% Fortified Contracts 】                       │
│                                                                                 │
└─────────────────────────────────────────────────────────────────────────────────┘
```

### Pillar 1: Crate Rustdoc Purification (`crates/*/src/`)
- **`xuanji`**: Verify reaction model types (`RuleKey`, `StructuredFactIdentity`, `Violation`, `Report`, `Baseline`) are accurately documented.
- **`xingbiao`**: Verify `cargo metadata` reader substrate docstrings.
- **`guibiao`**: Verify static scanner module headers (`must_not_import`, `deny_external`, `must_not_call_inline`).
- **`hunyi`**: Verify AST semantic capability headers (`signature_coupling`, `dyn_trait`, `impl_trait`, `async_exposure`, `reexport_exposure`, `external_crate`, `unsafe_confinement`, `visibility_ceiling`).
- **`louke`**: Verify runtime origin assertion (`assert_boundary!`) and audit scanner docstrings.
- **`tianheng`**: Verify shell composer prelude and composed profile (`SansIoPure`, `NoExistentialLeak`) docstrings.

### Pillar 2: Contract Hardening
- Audit existing test coverage against documented architectural invariants:
  - `xuanji` serde wire stability.
  - `guibiao` syn-free module scanner fidelity.
  - `hunyi` resolution fixpoint hop-cap and seam identity.
  - `louke` origin assertion fail-closed reaction.
- Confirm every capability has an assigned reaction test owner (already verified in `test_examples.sh`).

### Pillar 3: Release Coherence & Verification
- Update `CHANGELOG.md` under `[Unreleased]`.
- Execute full DoD pre-flight commands (`cargo build`, 3 `clippy` passes, `cargo fmt`, `cargo test`, `cargo doc`, `cargo deny`, `scripts/check_release_coherence.sh`, `scripts/test_examples.sh`).
