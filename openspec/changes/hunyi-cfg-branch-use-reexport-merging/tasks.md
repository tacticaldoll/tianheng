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
      single-valued `rewrite_longest_prefix`.
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

## 2a. Review-discovered consumers (adversarial apply-stage review, round 2)

An independent adversarial review declined to accept round 1's claim that the other 7 `resolve_path`
callers had "no audit-verified need" for cfg-blind treatment on its own terms, and constructed live
counter-examples instead. 3 of the 7 (across 6 call sites) were confirmed genuine, each independently
reproduced by the implementer before being fixed:

- [x] 2a.1 `scan.alias_targets` (`type X = <path>;`'s landing-type record) changed from
      `HashMap<String, String>` to `AliasMap`, unifying its type with `scan.aliases`. Its one
      population site now uses `resolve_path_all` and accumulates every candidate.
- [x] 2a.2 `containment.rs::resolve_self_type` changed to return `Vec<String>` (every landing
      candidate), routed through `expand_canonical_paths` instead of the bespoke single-candidate
      `canonicalize_through_single_alias_map` (now deleted, along with `canonicalize_through_reexports`,
      `canonicalize_through_aliases`, and `rewrite_longest_prefix` — all now fully dead, their only
      callers having moved onto the multi-candidate path).
- [x] 2a.3 `scan.aliases`'s own `type X = <path>;` population (the exposure pipeline's `AliasMap`,
      for the `alias_nominal_targets` complex-type case) now pushes every `resolve_path_all`
      candidate instead of one `resolve_path` result — closes a type-alias-indirected exposure false
      negative independent of the `UseMap`/`ReexportMap` fixes above.
- [x] 2a.4 `forbidden_marker.rs`'s derive-form leaf match (`derived_leaf`) and impl-form leaf match
      (`trait_leaf`) both changed to check every `resolve_path_all` candidate's leaf, falling back to
      the written leaf only when the use-map yields no candidate.
- [x] 2a.5 `forbidden_marker.rs`'s self-type landing check now checks every `resolve_self_type`
      candidate against the `defined`/`under_subtree` gate, instead of only the first.
- [x] 2a.6 `trait_impl.rs`'s anchor resolution: the declared anchor's own re-export facade now
      expands to every candidate via `expand_canonical_paths`; each impl site's trait path now
      resolves via `resolve_path_all` and expands to every re-export candidate; the match is now "any
      impl-site candidate is any declared-anchor candidate." The rendered `trait_ref` identity uses
      whichever candidate actually matched.
- [x] 2a.7 Independently re-checked (not assumed safe by category) the callers the review did NOT
      flag — `canonical_self_owner`/`canonical_self_owner_without_fallback`
      (`resolve/shape.rs`)/`canonical_unsafe_owner` (`scan.rs`) — confirmed these render a finding's
      displayed identity LABEL from the self type as written, not a reaction decision; a wrong
      candidate there is an identity-collision risk (change 1's bug class), not a missed reaction.
      Not reproduced, left untouched, documented as an explicit non-goal in proposal.md/design.md.

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
- [x] 3.5 From the round-2 review's discoveries, each reproduced and fixed with both declaration
      orders as separate permanent tests:
      - `forbidden_derive_leaf_reacts_when_the_forbidden_alias_is_declared_{first,second}`
      - `forbidden_impl_trait_leaf_reacts_when_the_forbidden_alias_is_declared_{first,second}`
      - `forbidden_marker_self_type_landing_reacts_when_the_forbidden_alias_is_declared_{first,second}`
      - `type_alias_exposure_reacts_when_the_forbidden_alias_is_declared_{first,second}`
      - `trait_impl_anchor_reacts_when_the_forbidden_alias_is_declared_{first,second}`
- [x] 3.6 Non-vacuous verification, done layer by layer rather than only at the type level, for
      every one of the 6 round-2 fixed call sites, in addition to round 1's:
      - Reverted `push_candidate` to overwrite semantics (keeping the `Vec`-valued type): confirmed
        the order-sensitive tests fail exactly as predicted, restored.
      - Reverted `exposure.rs`'s `resolve_path_all` call to single-candidate `resolve_path` (keeping
        map accumulation intact): confirmed the order-dependent `UseMap` test fails specifically,
        restored.
      - Reverted `resolve_principal`'s candidate list to `.take(1)` (keeping map accumulation
        intact): confirmed both dyn-trait/impl-trait tests fail, restored.
      - Reverted `forbidden_marker.rs`'s self-type landing check to first-candidate-only: confirmed
        only the "declared second" test fails, restored.
      - Reverted `scan.rs`'s `alias_targets` population to single-candidate `resolve_path`: confirmed
        the identical self-type-landing test fails the same way, restored.
      - Reverted `scan.rs`'s `aliases` (type-alias exposure) population to single-candidate
        `resolve_path`: confirmed only the "declared second" exposure test fails, restored.
      - Reverted `trait_impl.rs`'s candidate matching to first-candidate-only: confirmed only the
        "declared second" anchor test fails, restored.
      - Full suite green after every restore.

## 4. Documentation

- [x] 4.1 Added a CHANGELOG `[Unreleased] ### Fixed` entry. No **BREAKING** marker — false
      negatives closing, not an identity shape; no existing baseline is invalidated. Updated after
      round 2 to describe the full scope (forbidden-marker, trait-impl-locality, and the third
      `alias_targets` map), not just the original two capabilities.
- [x] 4.2 Added `MODIFIED Requirements` deltas to `semantic-signature-coupling`,
      `semantic-dyn-trait-operand-boundary`, `semantic-impl-trait-operand-boundary`,
      `semantic-forbidden-marker`, and `semantic-trait-impl-locality` — each extends its existing
      name-resolution requirement with the cfg-blind multi-candidate scenario, rather than adding a
      new requirement.

## 5. Definition of Done

- [x] 5.1 Run the full local gate list from `AGENTS.md` (build, three clippy passes, fmt, full test
      suite, both doc passes, `cargo deny check`, release-coherence scripts, `test_examples.sh`) —
      re-run after round 2's expanded scope, not just round 1's.
- [ ] 5.2 Adversarial apply-stage review (round 3): confirm round 2's fixes are complete and
      correct, not a taste call — including whether any further caller was missed.
