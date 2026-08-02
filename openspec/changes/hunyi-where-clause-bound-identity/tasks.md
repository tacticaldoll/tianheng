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

- [x] Replace the bare `"_"` fallback at `crates/hunyi/src/collect.rs:753` with the
  `_#{ordinal}.{bound_ordinal}` sentinel, threading `bound_ordinal` via
  `where_clause.predicates.iter().enumerate()`. <!-- id: 0 -->
- [x] Add a regression test reproducing the exact two-bound trigger (`Arr<{ N + 1 }>` /
  `Arr<{ N + 2 }>`, each `: AsRef<crate::infra::Secret>`, one impl block) asserting the evaluation
  now fails loud with "cannot identify semantic fact without a stable structural label" and never
  a shared literal fact, mirroring
  `unrenderable_generic_marker_instantiations_fail_loud_without_positional_identity`
  (`trait_impl_exposure_unrenderable_where_bound_fails_loud_without_positional_identity`), plus the
  single-bound counterpart proving the fail-loud outcome does not depend on a second bound being
  present (`trait_impl_exposure_unrenderable_where_bound_fails_loud_even_alone`). <!-- id: 1 -->
- [x] Add a regression test proving the per-bound index is genuinely collision-free within one
  impl block (not merely detectable) — a black-box test cannot distinguish this, since
  `reject_positional_identity` fails loud on EITHER a genuinely unique or a reused sentinel with the
  identical message, so this calls `collect_trait_impl_exposures` directly and asserts the two
  bounds' `where`-position keys differ before the gate ever runs
  (`trait_impl_exposure_where_bound_sentinels_never_share_a_bound_ordinal`). <!-- id: 2 -->
- [x] Confirm the existing renderable-bound tests
  (`trait_impl_exposure_reacts_at_the_where_position`,
  `trait_impl_exposure_reacts_at_a_where_clause_bounded_type`,
  `trait_impl_exposure_reacts_at_a_const_generic_param_type`) are unaffected — run them explicitly,
  not merely by inclusion in the full suite. <!-- id: 3 -->
- [x] Add the adopter-facing `CHANGELOG.md` `[Unreleased]` → `### Fixed` entry, naming the measured
  identity collision, the fail-loud outcome an adopter with this rare shape now sees, and that a
  renderable where-clause bound is unaffected. <!-- id: 4 -->
- [ ] Run the full Definition of Done from the workspace root and report actual output. <!-- id: 5 -->
