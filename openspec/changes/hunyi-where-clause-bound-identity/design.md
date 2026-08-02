## Context

`collect_trait_impl_exposures` (`crates/hunyi/src/collect.rs:674-799`) observes a trait `impl`
block's own generic-parameter bounds and `where`-clause, keying each bound's seam by the bounded
type's rendered string (`TraitImplPosition::Where(key)`). The generic-parameter loop's key is always
a bare identifier (`tp.ident.to_string()` / `cp.ident.to_string()`) and never fails to render. The
`where`-predicate loop's key is `type_to_string(&pt.bounded_ty)`, which CAN fail: a complex
const-generic argument such as `{ N + 1 }` (the brace-wrapped form Rust requires to disambiguate a
non-trivial const generic from the `<`/`>` shift-operator grammar) parses to an `Expr` shape
`expr_to_string` does not handle (only `Expr::Lit` and `Expr::Path` render), so
`generic_argument_to_string` returns `None` for that argument, and `path_to_string`'s
`rendered?.join(...)` propagates that `None` out through the whole path via `?` — not just the one
argument, the entire `Arr<{ N + 1 }>` path. The current fallback is `unwrap_or_else(|| "_".to_string())`
— a bare literal, indistinguishable from any OTHER unrenderable bound in the same impl block.

## Measured reproduction

Verified before writing this proposal, using the crate's own in-process test harness
(`findings_including_trait_impls`, `crates/hunyi/src/tests.rs`) rather than trusting the finding's
description:

```rust
pub struct Thing;
pub struct Arr<const N: usize>;
pub const N: usize = 1;
impl crate::Port for Thing
where
    Arr<{ N + 1 }>: AsRef<crate::infra::Secret>,
    Arr<{ N + 2 }>: AsRef<crate::infra::Secret>
{}
```

under `must_not_expose("crate::infra").including_trait_impls()`, anchored at the declaring module.

| bounds present | facts produced |
| --- | --- |
| both `Arr<{N+1}>` and `Arr<{N+2}>` | `["crate::infra::Secret exposed by impl crate::Port for crate::m::Thing (where _)"]` |
| `Arr<{N+1}>` only | the same single string, byte-identical |
| `Arr<{N+2}>` only | the same single string, byte-identical |

All three cases produce the identical fact. The two-bound case does not produce two facts; either
bound alone produces the same fact the pair produces together. A baseline entry recorded against one
bound (or against the two-bound state) cannot distinguish a change from the first bound to the
second — the `(target, rule_key, fact)` identity the baseline keys on is unchanged.

The renderable case is unaffected by the bug and stays unaffected by this fix: the existing test
`trait_impl_exposure_reacts_at_a_where_clause_bounded_type` (a bounded type
`crate::infra::Assoc`, which renders cleanly) already asserts a distinct, correct key and is not
touched.

## Goals / Non-Goals

**Goals:**
- An unrenderable where-clause bounded type's fallback stops being a bare, collision-prone literal.
- Two unrenderable bounds in the SAME impl block get distinguishable internal identities, not just
  two unrenderable bounds in different impl blocks (the existing `ordinal` alone is only unique
  across impl blocks/items, not across multiple bounds inside one impl block's where-clause).
- The failure is caught by the existing `reject_positional_identity` gate and reported as a
  constitution error, matching every other unrenderable-structural-role case this capability
  already protects (owner/self-type, trait-arg).
- `openspec/specs/semantic-trait-impl-exposure/spec.md`'s "Trait-impl exposure uses observed
  structural seams" requirement gets its role enumeration corrected to name the where-bounded-type
  role it omitted — the gap that let this go two adversarial-review rounds as unverified.

**Non-Goals:**
- A partial-render style (`Arr<_#N>`, keeping the renderable base and marking only the unrenderable
  generic argument) — see Decision 1.
- Touching the OTHER bare-`"_"` fallbacks already present elsewhere in 渾儀
  (`crates/hunyi/src/finding.rs:895,903`; `crates/hunyi/src/resolve/shape.rs:244,248`). Checked, not
  assumed clean: `render_sig_tail`'s two call sites are explicitly documented as diagnosis-only text
  that "does not re-key" async-exposure identity (the structured identity names the seam
  independently of the rendered tail); `bound_to_string`'s two call sites are the documented,
  intentional "stated rendering bound" shared with `dyn`-trait-object rendering (one finding, never
  zero, for an unrenderable trait-object bound component) — a DIFFERENT, already-decided tradeoff
  from this one, which is about seam-identity collision, not subject-text granularity. Neither
  contradicts an adjacent doc comment or a spec guarantee the way `collect.rs:753` did.
- Resolving the bounded type's path through `uses`/`module` (alias resolution) the way
  `canonical_self_owner` resolves the Self type's base path. The where-key's renderable case already
  uses the bare written form (`type_to_string`, no resolution) — see the existing
  `crate::infra::Assoc`-bounded test, which asserts the written, not resolved, string. Adding
  resolution here would be a separate, unrequested behavior change to the renderable path.

## Decisions

### Decision 1: A bare positional sentinel, not a partial render

Two ways to close the gap were on the table:

1. **Bare sentinel** (adopted): `format!("_#{ordinal}.{bound_ordinal}")`, matching
   `canonical_self_owner`'s and this same function's own `trait_label`'s fallback style
   (`format!("trait_#{ordinal}")`, three lines above the bug).
2. **Partial render**: `Arr<_#N>`, matching the array-length mitigation
   (`crates/hunyi/src/resolve/shape.rs:334-348`) that keeps `[elem; _]` rather than erasing the whole
   array type.

The array-length case can do a partial render because `syn::Type::Array`'s handler calls
`type_to_string(&a.elem)` and `expr_to_string(&a.len)` as two INDEPENDENT calls, keeping the first
result regardless of the second. `path_to_string`'s generic-argument handling is not structured that
way: `let rendered: Option<Vec<String>> = args.args.iter().map(generic_argument_to_string).collect();`
followed by `rendered?.join(...)` means ONE failing argument aborts the WHOLE path via `?` — the base
identifier `Arr` is never reached as a separate, salvageable value at this layer.
`canonical_self_owner` gets its own partial-render behavior (`format!("{base}<_#{ordinal}>")`) only
because it does NOT go through `path_to_string`'s monolithic path: it calls `resolve_path` for the
base and `render_last_segment_args` for the arguments as two separate steps, specifically built to
support that split.

Achieving `Arr<_#N>` for the where-clause key would therefore mean restructuring
`path_to_string`/`type_to_string` to split base-rendering from argument-rendering everywhere they are
used — self type, trait-arg, assoc bindings, method returns, forbidden-marker, trait-impl-locality —
not a change scoped to this one position. That is a materially larger change for a benefit this
position does not need: `reject_positional_identity` already turns ANY unrenderable structural role
into a constitution error before publication, so the sentinel's own readability is not
adopter-facing — nobody sees `_#{ordinal}.{bound_ordinal}` in a report; they see "cannot identify
semantic fact without a stable structural label" naming the file (via the constitution-error path
that wraps it). The bare-sentinel style is what this capability already does for its most
structurally similar case (`trait_label`, in the very same function), so it is also the
locally-consistent choice, not merely the cheaper one.

### Decision 2: A per-bound index composed with the item ordinal, not the item ordinal alone

The existing `ordinal` parameter is unique per top-level item (impl block) across the whole module,
continuously threaded across `#[cfg]` branches (the property `impl_trait_subtree_cfg_branches_never_share_an_unrenderable_owner_fallback`
already pins for the Self-type position). It is constant, though, across every bound WITHIN one impl
block's where-clause — reusing it bare for the where-key fallback would give two unrenderable bounds
in the SAME impl block (this proposal's own reproduction) an identical sentinel. Functionally,
`reject_positional_identity` would still fail the whole evaluation loud either way, because it
checks only for the `_#` marker's presence, not for cross-fact uniqueness — but publishing a
non-unique sentinel would contradict the discipline every other `_#{ordinal}`-style fallback in this
codebase upholds (genuine collision-freedom, not merely detectability), and would be one identity
principal short if `reject_positional_identity`'s behavior — or a consumer that reads structured
identity before that gate runs — ever changes. `where_clause.predicates.iter().enumerate()` supplies
a per-bound index local to the impl block's where-clause at zero extra cost (no new counter threaded
through the call chain), composed with `ordinal` as `_#{ordinal}.{bound_ordinal}`.

## Risks / Trade-offs

- **[Trade-off] A new constitution error for an adopter with this shape.** Rare on ordinary source —
  a where-clause bound naming a complex const-generic-indexed type is not an idiom seen anywhere in
  this repo's own fixtures or examples — and the alternative (the status quo) is a silent
  false-negative-enabling collision, the class every prior fix in this position's neighborhood
  (owner/self-type, trait-arg) already treats as worse than a loud stop.
- **[Bound, stated not silent] The generic-parameter loop's own keys are unaffected.** `T`'s and a
  const-param's own ident always render; only the where-PREDICATE bounded-type key can fail, so only
  that loop gains the per-bound index.
