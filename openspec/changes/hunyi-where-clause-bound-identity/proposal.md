# proposal: Hunyi Gives an Unrenderable Where-Clause Bound a Positional Sentinel

## Why

`semantic-trait-impl-exposure`'s `where {bounded-type}` position keys its seam by
`type_to_string(&pt.bounded_ty)` (`crates/hunyi/src/collect.rs:753`), falling back to the bare
literal `"_"` when the bounded type cannot be rendered. A complex const-generic argument —
`Arr<{ N + 1 }>`, the brace-wrapped expression syntax needed to disambiguate a non-trivial const
generic — is exactly such a case: `path_to_string`'s generic-argument rendering
(`crates/hunyi/src/resolve/shape.rs:272-299`) is all-or-nothing, so one unrenderable argument fails
the whole path, not just that argument.

Measured with a probe reproducing an adversarial-review finding (round 2), rather than trusted from
the finding's description: a module declares one `impl` block with TWO where-clause bounds,
`Arr<{ N + 1 }>: AsRef<crate::infra::Secret>` and `Arr<{ N + 2 }>: AsRef<crate::infra::Secret>`,
under a boundary forbidding `crate::infra` with `.including_trait_impls()`. Both bounds fail to
render, both fall back to the literal `"_"`, and both therefore produce the byte-identical fact
`crate::infra::Secret exposed by impl crate::Port for crate::m::Thing (where _)`. The two-bound case
and either bound in isolation all produce that same single fact string — the second bound's
violation contributes no distinguishable trace at all, so a baseline entry recorded against one
bound silently continues to suppress after that bound is replaced by the other.

This directly contradicts two things this capability already states elsewhere. Three lines above
the bug, the sibling `trait_label` fallback for an unrenderable trait path already uses a
positional sentinel (`format!("trait_#{ordinal}")`) rather than a literal, and the codebase's
established discipline for exactly this class of failure — an unrenderable const-generic argument —
is `canonical_self_owner`'s `_#{ordinal}` sentinel, routed through the shared
`reject_positional_identity` gate (`crates/hunyi/src/finding.rs:743-759`) so unsupported syntax fails
loud instead of silently colliding; the spec itself already asserts, at the requirement documenting
this position, that where-clause bounds are "keyed by the bounded type so two distinct bounds never
collapse," and separately, under "Trait-impl exposure uses observed structural seams," that "a
traversal position or impl/item ordinal SHALL NOT substitute for an unrenderable structural role"
and that an unrenderable seam must fail safely rather than fall back positionally — but that
requirement's own enumeration never named the where-bounded-type role, which is the one role in this
capability's position set that can fail to render yet had no corresponding protection.

## What Changes

- Replace the bare `"_"` fallback for an unrenderable where-clause bounded type
  (`crates/hunyi/src/collect.rs:753`) with an internal positional sentinel `_#{ordinal}.{bound_ordinal}`
  — the item's existing, already crate-wide-continuous `ordinal` (also used for the sibling
  `trait_label` fallback in the same function) composed with a per-bound index local to this impl
  block's where-clause (via `where_clause.predicates.iter().enumerate()`), so two unrenderable
  bounds in the SAME impl block never share a sentinel either. The generic-param loop's own keys
  (a bare identifier, e.g. `T`) never fail to render and are unaffected.
- The sentinel is never published: `reject_positional_identity` already scans every structured
  fact's fields for the `_#` marker and fails the whole evaluation loud (a constitution error)
  before any dedup or publication step runs — unchanged machinery, a new call site feeding it.
- No behavior change for the common case: a where-clause bound that renders cleanly (the existing,
  tested case) is unaffected; only the previously-silent unrenderable-bound path now fails loud.
- Considered and rejected: mirroring the array-length mitigation
  (`crates/hunyi/src/resolve/shape.rs:334-348`, keep the renderable part and mark only the
  unrenderable part `_`) as `Arr<_#N>` instead of a bare sentinel. `path_to_string` propagates `None`
  for the WHOLE path via `?` the instant any one generic argument fails to render (unlike
  `canonical_self_owner`, which resolves the base path and renders its generic arguments as two
  separate steps specifically to support that partial fallback) — so producing `Arr<_#N>` here would
  need restructuring `path_to_string`/`type_to_string` itself, changing behavior at every one of
  their other call sites (self type, trait-arg, assoc bindings, method returns, forbidden-marker,
  trait-impl-locality), not just this one. The bare-sentinel style also matches this exact function's
  own adjacent `trait_label` fallback, so the fix stays consistent with its immediate sibling rather
  than importing a different capability's pattern. See `design.md` for the full comparison.

## Capabilities

### Modified Capabilities

- `semantic-trait-impl-exposure`: the `where {bounded-type}` position's failure mode gains the
  fail-loud sentinel discipline the capability's own "observed structural seams" requirement already
  demands of every other position, and that requirement's enumeration now names the role it omitted.

## Impact

- `crates/hunyi/src/collect.rs`: the where-predicate key's fallback.
- `crates/hunyi/src/tests.rs`: regression coverage for the unrenderable-bound collision (the
  reproduced two-bound trigger) and the fail-loud gate, mirroring the existing
  `impl_trait_subtree_cfg_branches_never_share_an_unrenderable_owner_fallback` /
  `unrenderable_generic_marker_instantiations_fail_loud_without_positional_identity` tests.
- `CHANGELOG.md`: `[Unreleased]` → `### Fixed`.
- Non-breaking: no public API, DSL, or wire-format change. An adopter whose where-clause bound is a
  complex const-generic expression now sees a constitution error (exit 2) naming the failure instead
  of a silently under-counted violation set — a false-negative closure, absorbable by baseline for
  the renderable case, and requiring the adopter to simplify or otherwise avoid the unrenderable
  const-generic bound shape (rare on ordinary source) for the unrenderable case.
