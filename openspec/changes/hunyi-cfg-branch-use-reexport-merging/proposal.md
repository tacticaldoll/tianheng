## Why

Two mutually-exclusive `#[cfg]`-gated declarations for the identical local name, in the same file,
silently collapse to one in hunyi's shared name-resolution layer — the later declaration always
wins (or, with the fix half-applied during development, whichever declaration a caller happens to
consult first), instead of both being checked (cfg-blind: observation cannot know which branch is
live at build time). Two independent maps had this defect:

- `UseMap` (`crates/hunyi/src/resolve/mod.rs`): `use ... as Name;` declared once under
  `#[cfg(unix)]` and again under `#[cfg(not(unix))]` — reproduced directly with
  `#[cfg(unix)] use crate::infra::Secret as Handle; #[cfg(not(unix))] use crate::safe::Handle; pub
  fn leak() -> Handle { .. }`: whichever declaration was NOT the forbidden one, if declared last,
  made the whole file silently pass.
- `ReexportMap`: the identical collision for `pub use ... as X;` targets, reached through a facade
  path — reproduced the same way, substituting re-exports for `use` aliases.

Both are `HashMap<String, String>` — a single value per name — so `.insert()` on a second,
mutually-exclusive declaration always overwrites the first, regardless of which one is genuinely
forbidden.

While fixing `UseMap`, an independent reproduction (not named in the original audit findings, but
using the identical fixture pattern) found the same collision in `resolve_principal`
(`crates/hunyi/src/crate_scope.rs`) — the shared principal-trait resolver dyn-trait and impl-trait's
*operand-scoped* boundaries both use. Since it consumes the same `UseMap`/`ReexportMap` through the
same single-candidate `resolve_path`/`canonicalize_through_reexports` calls `module_findings` used,
it has the identical defect, not a separately-implemented one — closed in the same change rather
than deferred, unlike a genuinely separate capability's own implementation would be.

An independent adversarial apply-stage review then refused to accept, at face value, this change's
own initial claim that its other 7 `resolve_path` callers had "no audit-verified need" for
cfg-blind treatment — it constructed live counter-examples instead, and found 3 more genuine gaps,
each independently reproduced before being folded in:

- `forbidden_marker.rs`'s derive-form and impl-form leaf matching — a mutually-exclusive
  `#[cfg]`-gated `use` alias for a forbidden derive/trait's own name only reacted when the forbidden
  alias happened to be declared first.
- `trait_impl.rs`'s anchor resolution — the identical collision on the anchored trait's own alias,
  plus the single-candidate re-export closure on both the declared anchor's facade and each impl
  site's resolved path.
- `scan.alias_targets` (`type X = <path>;`'s landing-type record) was itself a THIRD single-valued
  map (`HashMap<String, String>`), feeding both `forbidden_marker.rs`'s self-type/marker-acquisition
  gate and `scan.aliases` (the exposure pipeline's own `AliasMap`) — so a `type X = Y;` alias whose
  target `Y` was itself a cfg-collided `use` name silently dropped a candidate at this THIRD layer,
  independent of the `UseMap`/`ReexportMap` fixes above.

## What Changes

- `UseMap`/`ReexportMap` become multi-valued (`HashMap<String, Vec<String>>`), mirroring `AliasMap`'s
  already-multi-valued shape. `collect_uses`/`collect_reexports` accumulate every candidate instead
  of overwriting.
- A new `resolve_path_all` returns every `UseMap` candidate a path's head resolves to;
  `resolve_path` (unchanged signature, `Option<String>`) becomes a thin wrapper taking the first —
  preserving current behavior for its non-exposure callers (impl-locality anchor resolution,
  trait-impl anchoring, marker acquisition), which have no audit-verified need for cfg-blind
  multi-candidate treatment.
- `expand_canonical_paths` (already a multi-candidate DFS for `AliasMap`) now walks `ReexportMap`
  the same way, via the existing `rewrite_longest_alias_prefixes` helper instead of the single-valued
  `rewrite_longest_prefix`. `canonicalize_through_reexports`/`canonicalize_through_aliases` (used by
  trait-impl anchor resolution) deliberately keep taking only the first candidate.
- `exposure.rs`'s own resolution now calls `resolve_path_all` and feeds every candidate through
  `expand_canonical_paths`; its already-existing downstream `.filter(matches_forbidden)` loop needed
  no change — it was already written to react to however many canonicals arrive.
- `resolve_principal` returns every candidate the same way; `matches_forbidden_principal` checks
  each one.
- `scan.alias_targets` becomes `AliasMap` (unified with `scan.aliases`'s type), populated via
  `resolve_path_all`; `containment.rs::resolve_self_type` now returns every landing candidate via
  `expand_canonical_paths`, replacing the now-deleted single-candidate
  `canonicalize_through_single_alias_map`.
- `scan.aliases`'s own `type X = <path>;` population (the exposure pipeline's `AliasMap`) now pushes
  every `resolve_path_all` candidate instead of one `resolve_path` result.
- `forbidden_marker.rs`'s derive-form and impl-form leaf matching, and its self-type landing check,
  all check every candidate.
- `trait_impl.rs`'s anchor resolution checks every candidate on both the declared anchor's own
  re-export facade and each impl site's resolved trait path.
- The now-fully-dead single-candidate helpers (`canonicalize_through_reexports`,
  `canonicalize_through_aliases`, `canonicalize_through_single_alias_map`, `rewrite_longest_prefix`)
  are deleted.

## Capabilities

### New Capabilities

(none)

### Modified Capabilities

- `semantic-signature-coupling`: extends the existing exposure-resolution requirement to state that
  a mutually-exclusive `#[cfg]` collision in the `use`-map or re-export closure does not suppress
  either candidate.
- `semantic-dyn-trait-operand-boundary` / `semantic-impl-trait-operand-boundary`: the same cfg-blind
  principal-trait resolution requirement, since both share `resolve_principal`.
- `semantic-forbidden-marker`: extends the derive-form and impl-form matching requirements so a
  cfg-collided `use` alias for the derive/trait name, or for a `type X = <path>;` self-type alias's
  landing, does not suppress either candidate.
- `semantic-trait-impl-locality`: extends the anchor-resolution requirement so a cfg-collided `use`
  alias or re-export facade, on either the declared anchor or an impl site's own trait path, does not
  suppress either candidate.

## Impact

- Affected code: `crates/hunyi/src/resolve/mod.rs`, `crates/hunyi/src/exposure.rs`,
  `crates/hunyi/src/crate_scope.rs`, `crates/hunyi/src/shape_scan.rs`,
  `crates/hunyi/src/scan.rs`, `crates/hunyi/src/containment.rs`, `crates/hunyi/src/forbidden_marker.rs`,
  `crates/hunyi/src/trait_impl.rs`.
- No public API/DSL/builder change, no baseline format change (this fixes false negatives, not an
  identity shape — an adopter's existing baseline is unaffected either way).
- Out of scope, named explicitly rather than silently left:
  - `exposure.rs:157` — one `#[cfg]`/`cfg_if` branch's child `mod` shadowing a mutually-exclusive
    branch's genuine extern re-export — needs a different mechanism (cfg-aware child-module-name
    partitioning within one branch, not a multi-valued map) and is its own follow-up change.
  - `canonical_self_owner`/`canonical_self_owner_without_fallback`/`canonical_unsafe_owner`'s own
    single-candidate `resolve_path` calls — these render a finding's displayed identity LABEL from
    the self type as written; a wrong candidate under cfg collision is an identity-collision risk
    (change 1's bug class), not a missed reaction. Not reproduced, not touched here.
