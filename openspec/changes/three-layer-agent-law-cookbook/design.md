## Context

Tianheng provides the `GovernanceTest` fluent builder and `constitution_markdown()` projection generator to allow self-governed projects to export their declared constitution as an agent-loadable Markdown artifact.

Currently, the concept of assembling this artifact is documented in code comments (`gate.rs`, `SELF_LAW_PREAMBLE`), but adopters lack a single unified term and step-by-step COOKBOOK guide that explains the Three-Layer Agent Law pattern end-to-end.

## Goals / Non-Goals

**Goals:**
- Define the canonical term **Three-Layer Agent Law** (Preamble + Projection Body + Rust Law Source).
- Document the preamble discipline (universals/vocabulary only, no crate-specific architectural claims) in [`COOKBOOK.md`](file:///home/qaz/work/tianheng/COOKBOOK.md).
- Provide a clear, copy-pasteable recipe using `GovernanceTest::for_constitution` with `.assert_projection_fresh_with_preamble()` and `BLESS=1` workflow.
- Update `BACKLOG.md` to record promotion of this item.

**Non-Goals:**
- Creating a macro/generator API for preambles (declined under drift law: documentation concern, not code contract).
- Modifying `GovernanceTest` rust API signatures.

## Decisions

### Decision 1: Unified Naming — Three-Layer Agent Law
Name the structure **Three-Layer Agent Law** across documentation and specs:
1. **Layer 1: Universal Preamble** — meta-instructions & vocabulary only (`SELF_LAW_PREAMBLE` discipline).
2. **Layer 2: Projection Body** — rendered by `constitution_markdown()` and fresh-checked by test.
3. **Layer 3: Law Source** — Rust `constitution()` code in tests/src governed by CODEOWNERS.

*Rationale*: Connects the three existing implementations into one clear mental model for adopters.

### Decision 2: COOKBOOK Recipe Integration
Add a new recipe under a dedicated section in [`COOKBOOK.md`](file:///home/qaz/work/tianheng/COOKBOOK.md): "Publish an imitable Agent Law for your codebase".

*Rationale*: `COOKBOOK.md` is the primary adopter-facing guide for governance patterns.

## Risks / Trade-offs

- [Risk: Adopter puts architectural claims in Preamble] → Mitigation: COOKBOOK recipe explicitly highlights the preamble discipline with bad/good examples and explains why un-reacted prose in preambles leads to rotted agent rules.
