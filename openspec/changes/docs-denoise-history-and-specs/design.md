# Design: Docs Denoise History and Specs

## Overview

This design addresses context noise reduction across four distinct layers of the repository without altering any runtime binary or library API behavior.

## 4-Pillar Architecture

```
┌─────────────────────────────────────────────────────────────────────────────────┐
│                    Tianheng Denoising & Purification Architecture               │
├─────────────────────────────────────────────────────────────────────────────────┤
│                                                                                 │
│ ┌──────────────────────┐  ┌──────────────────────┐  ┌────────────────────────┐ │
│ │ 1. BACKLOG.md        │  │ 2. openspec/specs/*  │  │ 3. crates/*/src/ Docs │ │
│ │ Prune BUILT history  │  │ Audit 28 specs for   │  │ Audit Rustdoc comments │ │
│ │ to docs/history/     │  │ 0.3.0 consistency    │  │ (`//!` & `///`)        │ │
│ └──────────────────────┘  └──────────────────────┘  └────────────────────────┘ │
│            │                          │                         │               │
│            └──────────────────────────┼─────────────────────────┘               │
│                                       ▼                                         │
│                      ┌─────────────────────────────────┐                        │
│                      │ 4. PROJECT / AGENTS Alignment   │                        │
│                      └─────────────────────────────────┘                        │
└─────────────────────────────────────────────────────────────────────────────────┘
```

### Pillar 1: Backlog Partitioning
- Create `docs/history/0.3.0-built-ledger.md` to hold shipped capability ledgers for 0.1.x, 0.2.x, and 0.3.0.
- Update `BACKLOG.md` to maintain the Live Decision Index, pointing to `docs/history/0.3.0-built-ledger.md` for historical reference.

### Pillar 2: OpenSpec Capability Specs Audit
- Verify that every spec under `openspec/specs/` uses exact 0.3.0 vocabulary:
  - `RuleKey` (stable rule identity)
  - `StructuredFactIdentity` (typed fact fields)
  - Async seam identity vs presentation signature
  - Unsafe-site fact decomposition
  - Version-2 baseline and SARIF 2.1.0 format

### Pillar 3: Code Doc Sweep
- Inspect module headers (`//!`) and function/type docstrings (`///`) in:
  - `crates/xuanji`
  - `crates/xingbiao`
  - `crates/guibiao`
  - `crates/hunyi`
  - `crates/louke`
  - `crates/tianheng`
- Fix any stale references to legacy finding keys or outdated file structures.

### Pillar 4: Contract Alignment & Pre-flight Validation
- Update `CHANGELOG.md` under `[Unreleased]` with a brief maintenance note.
- Run `cargo fmt`, `cargo clippy`, `cargo test`, `cargo doc`, `cargo deny`, and coherence scripts.
