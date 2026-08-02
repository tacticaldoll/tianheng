## Context

Reproduced directly before designing the fix: `#[cfg(unix)] use crate::infra::Secret as Handle;
#[cfg(not(unix))] use crate::safe::Handle; pub fn leak() -> Handle { .. }` under a boundary
forbidding `crate::infra` returns `Ok(["crate::infra::Secret exposed by fn crate::api::leak"])` —
but reversing the two `use` lines returns `Ok([])`: the verdict depends on source order, because
`collect_uses` (`crates/hunyi/src/resolve/mod.rs`) builds `UseMap = HashMap<String, String>` via
plain `.insert()`, so the second declaration for the same name always overwrites the first. The
identical shape reproduces for `ReexportMap` (`pub use ... as X;`) and, discovered while fixing
this, for `resolve_principal`'s dyn-trait/impl-trait principal-trait resolution.

Traced the actual multi-value consumption pattern already established for `AliasMap`
(`HashMap<String, Vec<String>>`, used for `type X = <path>;` aliases): `expand_canonical_paths`
already runs a full iterative-DFS fixpoint over `AliasMap`'s multiple targets per key, returning
every reachable canonical path, and `exposure.rs`'s own downstream matching
(`canonicals.into_iter().filter(matches_forbidden)...`) was ALREADY written to react to however many
canonicals `expand_canonical_paths` returns — the cfg-blind "check every candidate" architecture
already exists for aliases; `UseMap`/`ReexportMap` just never fed it more than one candidate.

**Round 2 (adversarial apply-stage review):** the first version of this change claimed the 7
non-`resolve_path_all` callers of `resolve_path` were all identity/anchor consumers with "no
audit-verified need for cfg-blind treatment," and left them on the single-candidate path. An
independent adversarial review of that claim did not take it at face value — it constructed live
counter-examples against each caller instead. Three of the seven callers turned out to be genuine
forbidden/anchor-matching consumers, each reproduced directly (not merely suspected) with an
order-dependent silent pass, exactly the same shape as the original `UseMap` finding:

- `forbidden_marker.rs`'s derive-form and impl-form leaf matching (two call sites) — a
  mutually-exclusive `#[cfg]`-gated `use` alias for a forbidden derive/trait's name only reacted
  when the forbidden alias happened to be declared first.
- `trait_impl.rs`'s anchor resolution — the same collision on the anchored trait's own alias, plus
  the single-candidate `canonicalize_through_reexports` on both the declared anchor's own re-export
  facade and each impl site's resolved path.
- A third map, `scan.alias_targets` (`type X = <path>;`'s landing-type record, feeding
  `forbidden_marker.rs`'s self-type/marker-acquisition check via `containment.rs::resolve_self_type`)
  had the identical defect one level removed: it was itself single-valued
  (`HashMap<String, String>`) and populated via `resolve_path`, so `type X = Y;` where `Y` is a
  cfg-collided `use` name only ever recorded one landing candidate for `X`. This fed BOTH
  `forbidden_marker.rs`'s self-type gating check and `scan.aliases` (the exposure pipeline's own
  `AliasMap`) via a second, sibling `resolve_path` call in the same `syn::Item::Type` arm.

Each of the four was reproduced directly (fixture + assertion, both declaration orders) before being
folded into this same change — not a taste call or a hypothetical, the identical "real reproduction
before design" discipline the rest of this campaign applies.

## Goals / Non-Goals

**Goals:**
- `UseMap`/`ReexportMap` become multi-valued, mirroring `AliasMap`'s existing shape, so a
  mutually-exclusive `#[cfg]` collision never silently drops a candidate regardless of declaration
  order.
- Every genuinely forbidden/anchor-matching consumer of these maps — not just the two originally
  scoped in (`exposure.rs`, `resolve_principal`) — checks every candidate and reacts if any is
  forbidden. This now includes `forbidden_marker.rs` (derive leaf, impl trait leaf, and self-type
  landing) and `trait_impl.rs` (anchor resolution, both the declared anchor's own re-export facade
  and each impl site's resolved path).
- `scan.alias_targets` becomes multi-valued (`AliasMap`, not a separate single-valued type), reusing
  `expand_canonical_paths` for its own landing-type resolution instead of the bespoke
  single-candidate `canonicalize_through_single_alias_map`, which is now dead and removed (along with
  `canonicalize_through_reexports`/`canonicalize_through_aliases`/`rewrite_longest_prefix`, its only
  other caller having also moved onto the multi-candidate path).
- Non-exposure/non-anchor consumers that remain on `resolve_path` — impl-locality's
  `canonical_self_owner`/`canonical_self_owner_without_fallback` (identity LABEL rendering, not a
  reaction decision) and `canonical_unsafe_owner` (the same, for unsafe-site identity) — keep their
  exact current behavior. These render a finding's displayed identity from the self type AS WRITTEN;
  they do not decide whether a violation fires. A label naming the wrong candidate under a cfg
  collision would be an identity-collision risk (the same class of bug change 1 fixed), not a
  false-negative reaction gap — a different bug class, not reproduced here, and out of scope for
  this change.
- Both bare `#[cfg]` and `cfg_if!` forms of the collision are covered (the latter already reads as
  real code per the separately-closed cfg_if-transparency family).

**Non-Goals:**
- `exposure.rs:157` (child `mod` shadowing a mutually-exclusive branch's extern re-export) — a
  different mechanism (cfg-aware child-module-name partitioning within one branch), not a
  multi-valued map; its own follow-up change.
- `canonical_self_owner`/`canonical_self_owner_without_fallback`/`canonical_unsafe_owner`'s own
  single-candidate `resolve_path` calls — identity-label rendering, a different bug class
  (identity-collision, not false-negative reaction) from everything else this change fixes; not
  reproduced, not touched. A candidate follow-up if ever independently reproduced.
- `collect.rs`'s one test-only `resolve_path` call site — production-inert.
- `resolve/shape.rs`'s two `resolve_path` calls inside `canonical_self_owner`/
  `canonical_self_owner_without_fallback` — see above, same identity-label reasoning.

## Decisions

- **`UseMap`/`ReexportMap` become `HashMap<String, Vec<String>>`, not a new wrapper type.** Mirrors
  `AliasMap` exactly, so the existing multi-candidate DFS machinery in `expand_canonical_paths`
  (built for aliases) is reused verbatim for re-exports by swapping `rewrite_longest_prefix` for the
  already-existing `rewrite_longest_alias_prefixes` — no new algorithm, no new hop-cap reasoning
  (the existing `aliases.len() + reexports.len() + 1` cap already accounted for `.len()` meaning key
  count, not candidate count, since `AliasMap` was already this shape before this change).
- **`scan.alias_targets` becomes `AliasMap`, unifying its type with `scan.aliases`.** Rather than
  inventing a second multi-valued map shape, `alias_targets` now shares `AliasMap`'s exact type, so
  `containment.rs::resolve_self_type` can call `expand_canonical_paths` directly instead of a
  bespoke, now-deleted single-candidate fixpoint (`canonicalize_through_single_alias_map`). One
  fixpoint implementation, one hop-cap proof, for all three maps that ever needed it (aliases,
  reexports, alias-targets).
- **`resolve_path` keeps its `Option<String>` signature; `resolve_path_all` is new.** Rather than
  changing `resolve_path`'s return type (which would force every one of its callers to adapt,
  regardless of whether their own semantics call for multi-candidate treatment), `resolve_path`
  becomes a thin `resolve_path_all(...).into_iter().next()` wrapper. This kept the blast radius
  contained to exactly the callers that needed to change.
- **Every reaction-deciding caller was individually re-examined against a live counter-example, not
  assumed safe by category.** Round 1 of this change asserted "the other callers are identity/anchor
  consumers with no audit-verified need" without constructing a counter-example for each — an
  adversarial review correctly refused to accept that claim on its own terms and reproduced 3 of the
  7 as live false negatives. The corrected discipline applied in round 2: a caller is left
  single-candidate only when a concrete attempt to construct a collision against it fails to
  reproduce (`canonical_self_owner` and siblings — tried, and the failure mode is a wrong LABEL, not
  a missed reaction, a materially different bug this change does not claim to fix).
- **`resolve_principal` and `forbidden_marker.rs`/`trait_impl.rs`'s fixes are folded into the same
  change, not deferred.** All of them consume the exact same `UseMap`/`ReexportMap`/`alias_targets`
  through the exact same single-candidate primitives `module_findings` used before this change — the
  same mechanism, independently reproduced (not merely suspected) before being fixed here. Unlike
  change `hunyi-extern-block-exposure`'s siblings (separately-implemented collectors, each needing
  its own design judgment), this is one mechanism with many call sites.

## Risks / Trade-offs

- **[Risk] A caller relying on `UseMap`/`ReexportMap`/`alias_targets`'s old single-value type fails
  to compile.** → **Mitigation**: the type change is compile-enforced — every construction/
  consumption site had to be visited to make the workspace build again; no silent behavioral drift
  possible since the compiler catches every site.
- **[Risk] The type change alone looks sufficient but isn't — a caller still takes only the first
  candidate.** → **Mitigation**: verified non-vacuous per caller, not just per type, for every one of
  the 6 fixed call sites (map accumulation ×2, `exposure.rs`, `resolve_principal`, `forbidden_marker`
  ×3 sub-sites, `trait_impl.rs`): each was individually reverted to first-candidate-only, its
  regression test confirmed to fail in exactly the predicted (order-dependent) way, then restored.
- **[Risk] An adversarial review's own claim ("only these callers need fixing") could itself be
  under- or over-scoped.** → **Mitigation**: every caller the review flagged was independently
  reproduced by the implementer (not taken on the reviewer's word), and every caller the review did
  NOT flag (`canonical_self_owner` and siblings) was independently checked for a live counter-example
  before being left alone, rather than trusted by category.
- **[Risk] Widening these matchers to multi-candidate could regress an unrelated caller that shares
  the same primitive.** → **Mitigation**: `canonicalize_through_reexports`/
  `canonicalize_through_aliases`/`canonicalize_through_single_alias_map`/`rewrite_longest_prefix` are
  now fully dead (their only callers all moved onto `expand_canonical_paths`) and have been deleted,
  so there is no lingering single-candidate code path a future caller could accidentally reach.

## Migration Plan

1. Land the `UseMap`/`ReexportMap` type change and `collect_uses`/`collect_reexports` accumulation.
2. Land `resolve_path_all`, `exposure.rs`'s consumption fix, and `resolve_principal`'s consumption
   fix.
3. Land `scan.alias_targets`'s type unification with `AliasMap`, `scan.aliases`'s population fix
   (the `type X = <path>;` indirection case), `forbidden_marker.rs`'s three fixes (derive leaf, impl
   trait leaf, self-type landing), and `trait_impl.rs`'s anchor-resolution fix — all discovered via
   adversarial review, each independently reproduced first.
4. Delete the now-dead single-candidate helpers (`canonicalize_through_reexports`,
   `canonicalize_through_aliases`, `canonicalize_through_single_alias_map`,
   `rewrite_longest_prefix`) and update their former callers' doc comments.
5. Regression tests: `UseMap` collision (both orders, plus `cfg_if!`), `ReexportMap` collision (both
   orders), the dyn-trait/impl-trait sibling gap, and — from the review — derive-leaf, impl-trait-leaf,
   self-type-landing, type-alias-indirected-exposure, and trait-impl-anchor collisions (each both
   orders).
6. Verify non-vacuous per fix layer independently: each of the 6 fixed call sites reverted to
   first-candidate-only, confirmed its own regression test fails in the predicted order-dependent
   way, restored.
7. CHANGELOG `[Unreleased]` entry. No **BREAKING** marker — false negatives closing, not an identity
   shape; no existing baseline is invalidated. No version bump (campaign-wide constraint).

## Open Questions

None outstanding. `exposure.rs:157`'s different mechanism, and the identity-label (not reaction)
concerns in `canonical_self_owner` and siblings, are explicitly out of scope, not open questions
within this change.
