# tasks: Hunyi Gives an Unrenderable Where-Clause Bound a Positional Sentinel Implementation Plan

Investigation carried from the proposal/design (see `design.md` for the measured reproduction and
the two-way decision, so neither needs re-deriving):

- Root cause: `crates/hunyi/src/collect.rs:753`'s `type_to_string(&pt.bounded_ty).unwrap_or_else(|| "_".to_string())`.
- Fix shape: `format!("_#{ordinal}.{bound_ordinal}")`, `bound_ordinal` from
  `where_clause.predicates.iter().enumerate()`, matching the sibling `trait_label` fallback
  (`format!("trait_#{ordinal}")`, three lines above) and the codebase's established
  `_#{ordinal}` sentinel discipline (`canonical_self_owner`), routed through the existing
  `reject_positional_identity` gate — no new gate, no new call-site plumbing beyond the local
  `enumerate()`.
- The generic-parameter loop (`syn::GenericParam::Type`/`Const`) is unaffected: its keys are bare
  idents that never fail to render.

- [ ] Replace the bare `"_"` fallback at `crates/hunyi/src/collect.rs:753` with the
  `_#{ordinal}.{bound_ordinal}` sentinel, threading `bound_ordinal` via
  `where_clause.predicates.iter().enumerate()`. <!-- id: 0 -->
- [ ] Add a regression test reproducing the exact two-bound trigger (`Arr<{ N + 1 }>` /
  `Arr<{ N + 2 }>`, each `: AsRef<crate::infra::Secret>`, one impl block) asserting the evaluation
  now fails loud with "cannot identify semantic fact without a stable structural label" and never
  a shared literal fact, mirroring
  `unrenderable_generic_marker_instantiations_fail_loud_without_positional_identity`. <!-- id: 1 -->
- [ ] Add a regression test proving the per-bound index is genuinely collision-free within one
  impl block (not merely detectable), mirroring
  `impl_trait_subtree_cfg_branches_never_share_an_unrenderable_owner_fallback`'s intent adapted to
  one impl block's two bounds rather than two cfg branches. <!-- id: 2 -->
- [ ] Confirm the existing renderable-bound tests
  (`trait_impl_exposure_reacts_at_the_where_position`,
  `trait_impl_exposure_reacts_at_a_where_clause_bounded_type`,
  `trait_impl_exposure_reacts_at_a_const_generic_param_type`) are unaffected — run them explicitly,
  not merely by inclusion in the full suite. <!-- id: 3 -->
- [ ] Add the adopter-facing `CHANGELOG.md` `[Unreleased]` → `### Fixed` entry, naming the measured
  identity collision, the fail-loud outcome an adopter with this rare shape now sees, and that a
  renderable where-clause bound is unaffected. <!-- id: 4 -->
- [ ] Run the full Definition of Done from the workspace root and report actual output. <!-- id: 5 -->
