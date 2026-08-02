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

## Goals / Non-Goals

**Goals:**
- `UseMap`/`ReexportMap` become multi-valued, mirroring `AliasMap`'s existing shape, so a
  mutually-exclusive `#[cfg]` collision never silently drops a candidate regardless of declaration
  order.
- Exposure-matching consumers (signature-coupling's `exposure.rs`, and dyn-trait/impl-trait's shared
  `resolve_principal`/`matches_forbidden_principal`) check every candidate and react if any is
  forbidden.
- Non-exposure consumers of the same maps (impl-locality self-type resolution, trait-impl anchor
  resolution) keep their exact current behavior (first candidate) — deliberately, since no finding
  demonstrates they need cfg-blind treatment, and widening their semantics without evidence would be
  exactly the kind of unobserved reaction this project's minimalism rule forbids.
- Both bare `#[cfg]` and `cfg_if!` forms of the collision are covered (the latter already reads as
  real code per the separately-closed cfg_if-transparency family).

**Non-Goals:**
- `exposure.rs:157` (child `mod` shadowing a mutually-exclusive branch's extern re-export) — a
  different mechanism (cfg-aware child-module-name partitioning within one branch), not a
  multi-valued map; its own follow-up change.
- Widening `resolve_path`'s (the singular wrapper's) behavior for its 7 other callers
  (`crate_scope.rs`'s `resolve_self_type` is the one exception now fixed via `resolve_principal`;
  `containment.rs`, `collect.rs`, `trait_impl.rs`, `forbidden_marker.rs`, `scan.rs`'s several
  call sites) — none is an exposure-matching consumer, so each keeps taking the first candidate.

## Decisions

- **`UseMap`/`ReexportMap` become `HashMap<String, Vec<String>>`, not a new wrapper type.** Mirrors
  `AliasMap` exactly, so the existing multi-candidate DFS machinery in `expand_canonical_paths`
  (built for aliases) is reused verbatim for re-exports by swapping `rewrite_longest_prefix` for the
  already-existing `rewrite_longest_alias_prefixes` — no new algorithm, no new hop-cap reasoning
  (the existing `aliases.len() + reexports.len() + 1` cap already accounted for `.len()` meaning key
  count, not candidate count, since `AliasMap` was already this shape before this change).
- **`resolve_path` keeps its `Option<String>` signature; `resolve_path_all` is new.** Rather than
  changing `resolve_path`'s return type (which would force every one of its 9 callers to adapt,
  regardless of whether their own semantics call for multi-candidate treatment), `resolve_path`
  becomes a thin `resolve_path_all(...).into_iter().next()` wrapper. This kept the blast radius
  contained to exactly the callers that needed to change (`exposure.rs`, `resolve_principal`)
  instead of touching all 9.
- **`resolve_principal` fixed in the same change, not deferred.** Unlike change
  `hunyi-extern-block-exposure`'s sibling gaps (separately-implemented collectors in the same file,
  each needing its own reproduction and regression test before being folded in), `resolve_principal`
  consumes the exact same `UseMap`/`ReexportMap` through the exact same single-candidate
  `resolve_path`/`canonicalize_through_reexports` primitives `module_findings` used — it is the same
  mechanism, independently reproduced (not merely suspected) before being fixed here.
- **`canonicalize_through_reexports`/`canonicalize_through_aliases` (single-candidate wrappers) are
  untouched, still taking only the first result from `expand_canonical_paths`.** Their only caller,
  `trait_impl.rs`'s anchor resolution, has no audit-verified need for multi-candidate treatment.

## Risks / Trade-offs

- **[Risk] A caller relying on `UseMap`/`ReexportMap`'s old single-value type fails to compile.** →
  **Mitigation**: the type change is compile-enforced — every construction/consumption site had to
  be visited to make the workspace build again (confirmed: only `resolve/mod.rs` internals, one test
  helper type in `finding.rs`, and three raw `.insert()` calls in `tests.rs` needed updating; no
  silent behavioral drift possible since the compiler catches every site).
- **[Risk] The type change alone looks sufficient but isn't — a caller still takes only the first
  candidate.** → **Mitigation**: verified non-vacuous per caller, not just per type: reverting
  `exposure.rs`'s own `resolve_path_all` call (while keeping the map accumulation) reproduces the
  order-dependent silent pass again, and the identical check for `resolve_principal` confirms the
  same. The map-level fix and the consumption-level fix are independently load-bearing and were each
  independently verified.
- **[Risk] Widening `resolve_principal` to multi-candidate could regress trait-impl anchor
  resolution if it shared the same primitive.** → **Mitigation**: confirmed it does not —
  `trait_impl.rs` uses `canonicalize_through_reexports` directly, not `resolve_principal`, and that
  function is untouched.

## Migration Plan

1. Land the `UseMap`/`ReexportMap` type change and `collect_uses`/`collect_reexports` accumulation.
2. Land `resolve_path_all`, `exposure.rs`'s consumption fix, and `resolve_principal`'s consumption
   fix.
3. Regression tests: `UseMap` collision (both declaration orders, plus the `cfg_if!` form),
   `ReexportMap` collision (both orders), and the discovered dyn-trait/impl-trait sibling gap (both
   share one test shape via `resolve_principal`).
4. Verify non-vacuous per fix layer (map accumulation, `exposure.rs` consumption,
   `resolve_principal` consumption) — each independently, not just the type change as a whole.
5. CHANGELOG `[Unreleased]` entry. No **BREAKING** marker — false negatives closing, not an identity
   shape; no existing baseline is invalidated. No version bump (campaign-wide constraint).

## Open Questions

None outstanding. `exposure.rs:157`'s different mechanism is explicitly out of scope, not an open
question within this change.
