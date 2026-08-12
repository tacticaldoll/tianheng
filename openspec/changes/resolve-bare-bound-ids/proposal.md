## Why

A bound id written inside a `(bound: …)` reference in a spec is resolved. The same id written **bare** — in a
Rust doc comment above the very test that defends it — resolves against nothing, and two have been wrong for
some time.

Both were derived from a requirement's prose instead of the declaring scenario's heading, which is the same
mistake twice in two different crates. One sits in `crates/xuanji/src/tests.rs`, a **published** crate. The
bijection that holds declared ids against spec scenarios never sees them, because it compares the two
*declaration* sides and a doc comment is neither.

This is not the prose instrument this repository has rejected three times. A bound id is not a sentence to be
judged — it is a **reference with a recognizable shape**, and the set it must resolve into is produced by
`observation_bounds()`. Reference resolution here is already mechanical for paths, for `--exact` identifiers,
and for `(bound: …)`. This is the fourth reference kind and the only one nothing resolves.

## What Changes

- The register resolves a **bare** `<capability>/<slug>` wherever it appears in tracked Rust or Markdown, not
  only inside a `(bound: …)` reference in a spec.
- Recognition is by **shape against the enumerated capability set**: a maximal run of path characters that is
  exactly `<capability>/<kebab-slug>` where the capability is a directory under `openspec/specs/`. A path that
  merely contains a capability name is a longer run and is not a reference — the same word-reading rule the
  adopter-narrative reaction already uses.
- The two stale citations are corrected: `crates/kanhe/tests/capability_subjects.rs` and two occurrences in
  `crates/xuanji/src/tests.rs`.
- A typed census riding along: `crates/kanhe/tests/observation_bound_model.rs` says the reaction reads *"the
  family's four sets"* while it chains five. Four was right before the repository catalogs moved out of the
  shell. It carries no reaction and is repaired as prose, which is said plainly rather than implied.

Not breaking, and no version moves.

## Capabilities

### New Capabilities

<!-- none: the register already owns bound-id references and their resolution -->

### Modified Capabilities

- `observation-bound-register`: the requirement *Prose MAY reference a declared bound, and a reference SHALL
  resolve* gains the bare form and the corpus it is resolved over, with the scenarios for a bare id that does
  not resolve and for a path that merely contains a capability name.

## Impact

- `crates/kanhe/src/bound_register_parse.rs` — the bare-reference recognizer.
- `crates/kanhe/tests/bound_register.rs` — the direction over tracked Rust and Markdown.
- `crates/kanhe/src/tests/bound_register_parse.rs` — its failure matrix.
- `crates/kanhe/tests/capability_subjects.rs`, `crates/xuanji/src/tests.rs` — the stale citations.
- `crates/kanhe/tests/observation_bound_model.rs` — the stale census.
- `openspec/specs/observation-bound-register/spec.md`, `CHANGELOG.md`.
- No published signature, wire format, exit class, baseline or manifest is touched. The `xuanji` edit is a
  `#[cfg(test)]` doc comment.

### Capabilities touched without a requirement change

- `release-coherence`: `CHANGELOG.md` is its declared subject; one entry under `### Self-governance`, no
  requirement moves.
- `repository-checks`: `crates/kanhe/**/*.rs` is its declared subject; the files edited there are corrections
  and a new recognizer, and none of its requirements move.
