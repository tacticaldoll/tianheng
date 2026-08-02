## 1. Shared resolve-layer fix

- [x] 1.1 Changed `UseMap`/`ReexportMap` (`crates/hunyi/src/resolve/mod.rs`) from
      `HashMap<String, String>` to `HashMap<String, Vec<String>>`, mirroring `AliasMap`'s existing
      multi-valued shape.
- [x] 1.2 Added `push_candidate` and rewrote `collect_use_tree`'s four insertion sites and
      `collect_reexports`'s one insertion site to accumulate (skipping exact duplicates) instead of
      overwriting.
- [x] 1.3 Added `resolve_path_all` (returns every `UseMap` candidate); `resolve_path` becomes a thin
      `.into_iter().next()` wrapper, preserving its exact current signature and behavior for its 7
      unaffected callers.
- [x] 1.4 Updated `expand_canonical_paths`'s two `ReexportMap` lookups to use the already-existing
      `rewrite_longest_alias_prefixes` (the `AliasMap`-shaped multi-candidate walker) instead of the
      single-valued `rewrite_longest_prefix`. Updated `canonicalize_through_single_alias_map`'s one
      `ReexportMap` lookup to take the first candidate from the same walker, preserving its existing
      single-value behavior.
- [x] 1.5 Fixed the resulting compile errors: a test-only `UseMap` type annotation in `finding.rs`,
      and three raw `.insert(key, "string")` calls in `tests.rs` updated to `.insert(key,
      vec!["string"])`.

## 2. Exposure-matching consumers

- [x] 2.1 `exposure.rs`'s resolution now calls `resolve_path_all` and flat-maps every candidate
      through `expand_canonical_paths`, falling back to `bare_local_alias`/`extern_verbatim_renamed`
      only when the `use`-map yields no candidate at all (matching the original fallback-chain
      semantics). The existing downstream `.filter(matches_forbidden)` loop needed no change.
- [x] 2.2 Discovered (not in the original audit findings) and fixed the identical gap in
      `resolve_principal` (`crates/hunyi/src/crate_scope.rs`) — the shared principal-trait resolver
      dyn-trait and impl-trait's operand-scoped boundaries both use via `matches_forbidden_principal`
      (`shape_scan.rs`). Changed `resolve_principal` to return `Vec<String>` (every candidate,
      through the same `resolve_path_all` + `expand_canonical_paths` pipeline) and
      `matches_forbidden_principal` to check every candidate.
- [x] 2.3 Updated stale doc comments in `dyn_trait.rs`/`impl_trait.rs` that described the old
      single-candidate `resolve_path`/`canonicalize_through_reexports` pipeline.

## 3. Regression

- [x] 3.1 `mutually_exclusive_cfg_gated_use_aliases_both_react` +
      `..._react_regardless_of_declaration_order` — the exact `UseMap` collision, both orders.
- [x] 3.2 `mutually_exclusive_cfg_if_use_aliases_both_react` — the identical collision via `cfg_if!`
      arms rather than bare `#[cfg]`.
- [x] 3.3 `mutually_exclusive_reexport_targets_both_canonicalize_correctly` +
      `..._react_regardless_of_declaration_order` — the exact `ReexportMap` collision, both orders.
- [x] 3.4 `dyn_trait_operand_resolution_reacts_regardless_of_cfg_gated_use_alias_order` +
      `impl_trait_operand_resolution_reacts_regardless_of_cfg_gated_use_alias_order` — the
      independently-discovered `resolve_principal` sibling gap, reproduced and fixed.
- [x] 3.5 Non-vacuous verification, done layer by layer rather than only at the type level:
      - Reverted `push_candidate` to overwrite semantics (keeping the `Vec`-valued type): confirmed
        the order-sensitive tests fail exactly as predicted, restored.
      - Reverted `exposure.rs`'s `resolve_path_all` call to single-candidate `resolve_path` (keeping
        map accumulation intact): confirmed the order-dependent `UseMap` test fails specifically,
        restored.
      - Reverted `resolve_principal`'s candidate list to `.take(1)` (keeping map accumulation
        intact): confirmed both dyn-trait/impl-trait tests fail, restored.
      - Full suite green after each restore; final full non-vacuous sweep not needed beyond the
        per-layer checks already isolating each fix's own contribution.

## 4. Documentation

- [x] 4.1 Added a CHANGELOG `[Unreleased] ### Fixed` entry. No **BREAKING** marker — false
      negatives closing, not an identity shape; no existing baseline is invalidated.
- [x] 4.2 Added `MODIFIED Requirements` deltas to `semantic-signature-coupling`,
      `semantic-dyn-trait-operand-boundary`, and `semantic-impl-trait-operand-boundary` — each
      extends its existing name-resolution requirement with the cfg-blind multi-candidate scenario,
      rather than adding a new requirement.

## 5. Definition of Done

- [x] 5.1 Run the full local gate list from `AGENTS.md` (build, three clippy passes, fmt, full test
      suite, both doc passes, `cargo deny check`, release-coherence scripts, `test_examples.sh`).
- [ ] 5.2 Adversarial apply-stage review: confirm the declared reaction still bites, not a taste call.
