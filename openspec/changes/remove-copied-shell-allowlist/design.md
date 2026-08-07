## Context

`term_color.rs` explains why Tianheng keeps ANSI styling dependency-free, but it embeds the current
members of the shell's dependency allowlist. The enforced declaration and generated
`AGENTS.self-law.md` projection already own that enumerable set; a code comment cannot stay synchronized
with either and has previously drifted.

## Goals / Non-Goals

**Goals:**

- Preserve the architectural explanation for the zero-dependency implementation.
- Remove the copied enumerable set and direct readers to the generated law projection.
- Make the exact duplicate declaration shape identified by review fail in authored Tianheng shell source.

**Non-Goals:**

- Change Tianheng's dependency boundary, projection, behavior, or dependency graph.
- Add a broad prose detector or repair other allowlist restatements identified as separate findings.

## Decisions

Replace the member list with “dependency allowlist self-law” and a reference to
`AGENTS.self-law.md`. This keeps the stable reason while leaving live membership to its generated source
of truth.

Add a repository-only test beside Tianheng's self-governance checks. It recursively enumerates tracked
Rust-shaped source under `crates/tianheng/src` and rejects the exact declaration token
`restrict_dependencies_to(`. The constitution declaration remains in `tests/self_governance.rs`, outside
that product-source perimeter. This observes the reported duplication class without attempting to parse
arbitrary prose or banning legitimate API exercises in other crates' tests.

## Risks / Trade-offs

- A reader must follow the projection to see current members → the projection is precisely the
  staleness-checked artifact intended for that purpose.
- A paraphrased member list could evade the exact token → the reaction intentionally governs the observed
  declaration-copy shape; broader prose policing would be a noisy detector and is out of scope.
