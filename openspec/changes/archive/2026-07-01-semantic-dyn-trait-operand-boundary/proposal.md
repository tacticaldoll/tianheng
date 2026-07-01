## Why

渾儀's dyn-trait-boundary (`must_not_expose_dyn()`, v0.1.2) is **shape-only**: it forbids
*any* `dyn` node in a module's public surface. That is the right reaction for "this seam
carries no dynamic dispatch at all", but it cannot express the finer, common architectural
intent "this seam must not leak a `dyn` of **this particular trait**" — e.g. a core module may
expose `dyn std::error::Error` freely but must never expose `dyn crate::Port` across its public
API. Today that intent is inexpressible: you either forbid all `dyn` (too broad) or none.

This change adds the **named-operand depth** of the shape-only rule — the next rung on the same
`name → shape → named-operand` stair the static dimension just climbed with crate-source-boundary.
It is a refinement on the **same `syn` observation** the shape-only rule already performs (the
`dyn` nodes in the governed public surface), adding only resolution of each `dyn` node's principal
trait against a declared forbidden set. No new observation source, no new crate, no shell-contract
change. It is the type-shape analogue of signature-coupling's forbidden *named type* set, applied
to the trait *inside* a `dyn`.

## What Changes

- **New 渾儀 capability — operand-scoped dyn exposure.** A dyn-trait boundary may name a closed
  set of **forbidden trait operands** via `must_not_expose_dyn_of([...])`: a `dyn` node in the
  governed public surface whose **principal trait** resolves (canonicalized) to a member of the
  set is a violation; a `dyn` of any other trait passes. The shape-only `must_not_expose_dyn()`
  is unchanged (it reacts to any `dyn`); the operand variant is a distinct, narrower rule.
- **Operand resolution reuses `hunyi::resolve`.** A `dyn`'s principal trait path is resolved and
  canonicalized through re-export chains exactly as signature-coupling canonicalizes its forbidden
  type set (`canonical_path_str` → `resolve_path` → `canonicalize_through_reexports`), so a
  re-exported/aliased trait facade matches its defining path — closing the same re-export false
  negative the sibling rules close. Auto-trait / marker bounds (`Send`, `Sync`, lifetimes) are not
  operands: only the single principal (non-auto) trait of a trait object is matched.
- **Parity, not a parallel path.** The operand variant folds into the same reaction, projection
  (`list` text/json/markdown), baseline identity, and exit-code contract as the shape-only rule;
  the finding stays the rendered `dyn …` shape (which already names the trait). Only the
  per-boundary operand filter is new.
- **Additive only.** A new builder method plus an operand field on the existing `DynTraitBoundary`
  (empty ⇒ shape-only, unchanged behavior; a named set ⇒ operand-scoped); no existing rule,
  signature, or projection behavior changes.

## Capabilities

### New Capabilities
- `semantic-dyn-trait-operand-boundary`: a dyn-trait boundary may forbid the exposure of a `dyn`
  of a **named trait** (a closed forbidden-operand set), resolving each `dyn` node's principal
  trait through the shared 渾儀 resolver. The named-operand refinement of the shape-only
  `semantic-dyn-trait-boundary`; reuses its public-surface `dyn` walk, the resolver, and the
  reaction/projection/baseline contract — only the operand match is new.

### Modified Capabilities
<!-- None. semantic-dyn-trait-boundary's requirements do not change: `must_not_expose_dyn()`
     stays shape-only (any dyn reacts). This is a new, narrower sibling rule on the same
     boundary type, not a change to the shape-only rule's stated behavior. -->

## Impact

- **Crate:** `hunyi` (渾儀) only. A new builder `must_not_expose_dyn_of([...])`, an operand field
  on `DynTraitBoundary`, principal-trait extraction from a `dyn` node (`syn::TypeTraitObject` →
  first `TraitBound`), and operand canonicalization + matching reusing `hunyi::resolve`. The `syn`
  dependency stays quarantined here; no new dependency.
- **Observation:** a **finer read of the same surface** — the `dyn` nodes the shape-only rule
  already collects, now with their principal trait resolved and matched against a declared set.
  The public-surface walk is unchanged.
- **Shell (`tianheng`):** re-export the new builder; the operand set projects through the existing
  generic dyn-trait projection (an added `forbidden`/operand param), no new projector.
- **Stated bounds (inherited from the shape-only rule + resolver):** a `dyn` whose principal trait
  is macro-generated, glob/cross-crate re-exported, or otherwise unresolvable is out of the
  resolver's stated coverage (the same syntactic bound signature-coupling states); a resolvable
  principal trait always reacts. Auto-trait markers are never operands.
- **SemVer:** additive, non-breaking → folded into the ongoing **0.1.2** (no version bump).
