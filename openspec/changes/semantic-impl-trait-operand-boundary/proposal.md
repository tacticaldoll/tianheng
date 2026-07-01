## Why

渾儀's impl-trait-boundary (`must_not_expose_impl_trait()`, v0.1.2) is **shape-only**: it forbids a
public seam from returning *any* written `impl Trait`. That is right for "this seam returns only
named types", but cannot express the finer, common intent "this seam may return ergonomic
existentials (`impl Iterator`, `impl Future`) but must never return an existential of **this**
abstraction" — e.g. a core module may freely return `-> impl Iterator<Item = u8>` yet must never
leak `-> impl crate::ports::Port` (a domain port hidden behind an unnameable existential). Today
that intent is inexpressible: forbid all RPIT (too broad) or none.

This is the **named-operand depth** of the shape-only impl-trait rule — the same
`name → shape → named-operand` stair the dyn-trait boundary already climbed (v0.1.2 operand-scoped
dyn). It is a refinement on the **same `syn` observation** the shape-only rule performs (the
returned `impl Trait` nodes), adding only resolution of each node's principal trait against a
declared forbidden set, reusing signature-coupling's resolver exactly as operand-scoped dyn does.
No new observation source, no new crate.

## What Changes

- **New 渾儀 capability — operand-scoped impl-trait exposure.** An impl-trait boundary may name a
  closed set of **forbidden trait operands** via `must_not_expose_impl_trait_of([...])`: a returned
  `impl Trait` whose **principal trait** (its first trait bound) resolves (canonicalized) into the
  set is a violation; a returned `impl Trait` of any other trait passes. The shape-only
  `must_not_expose_impl_trait()` is unchanged (any returned `impl Trait` reacts).
- **Operand resolution reuses `hunyi::resolve`.** The principal trait path is resolved and
  canonicalized through re-export chains (`canonical_path_str` → `resolve_path(BareFallback::Ignore)`
  → `canonicalize_through_reexports` → `matches_forbidden`, exact-or-module-prefix), so a
  re-exported/aliased trait facade matches its defining path — identical to operand-scoped dyn.
- **Parity, not a parallel path.** The operand variant folds into the same reaction, projection,
  baseline identity, and exit-code contract as the shape-only rule; the finding stays the rendered
  `impl …` shape. Only the per-boundary operand filter is new. Return-position-only scoping is
  unchanged (argument-position `impl Trait` / APIT and `async fn` remain out of scope).
- **Additive only.** A new builder method + an operand field on the existing `ImplTraitBoundary`
  (empty ⇒ shape-only, unchanged; a named set ⇒ operand-scoped); no existing rule changes.

## Capabilities

### New Capabilities
- `semantic-impl-trait-operand-boundary`: a public seam must not return a `impl Trait` of a
  **named trait** (a closed forbidden-operand set), resolving each returned `impl Trait`'s
  principal trait through the shared 渾儀 resolver. The named-operand refinement of the shape-only
  `semantic-impl-trait-boundary`; reuses its return-position walk, the resolver, and the
  reaction/projection/baseline contract — only the operand match is new.

### Modified Capabilities
<!-- None. semantic-impl-trait-boundary's requirements do not change: must_not_expose_impl_trait()
     stays shape-only (any returned impl Trait reacts). This is a new, narrower sibling rule on the
     same boundary type. -->

## Impact

- **Crate:** `hunyi` (渾儀) only. An operand field on `ImplTraitBoundary`, a
  `must_not_expose_impl_trait_of([...])` builder, principal-trait capture on the return-position
  collector, and operand canonicalization + matching reusing `hunyi::resolve` (the exact machinery
  operand-scoped dyn uses). `syn`-only; no new dependency.
- **Observation:** a **finer read of the same return-position surface** — the returned `impl Trait`
  nodes the shape-only rule collects, now with their principal trait resolved and matched.
- **Shell (`tianheng`):** the operand set projects through the impl-trait projection (an added
  `forbidden` param), no new projector.
- **Stated bounds (inherited):** empty operand set ⇒ any returned `impl Trait` (loud,
  safe-by-direction, never a silent no-op); auto-trait markers are never operands; an unresolvable
  principal (bare std trait like `impl Iterator`/`impl Future` written bare, macro/glob re-export)
  is the stated resolver bound, never a silent pass of a *resolvable* operand. Return-position-only
  and the APIT/`async fn` bounds are inherited from the shape-only rule unchanged.
- **SemVer:** additive, non-breaking → folded into the ongoing **0.1.2** (no version bump).
