## Context

渾儀's shape-only impl-trait-boundary (v0.1.2) walks a module's public surface **return positions**
(`collect_item_return_impl_traits` → `impl_traits_in_return`, visiting `sig.output` only) and
collects returned `impl Trait` nodes via `ImplTraitCollector`, rendering each with
`impl_trait_to_string` / `bound_to_string`. It is shape-only: any returned `impl Trait` reacts, no
resolution.

Operand-scoped dyn (v0.1.2) is the exact template for adding a named-operand depth: it captures each
`dyn` node's principal trait (`trait_object_principal_path` — the first `TypeParamBound::Trait`) and
filters via signature-coupling's resolver (`resolve_path(BareFallback::Ignore)` →
`canonicalize_through_reexports` → `matches_forbidden`). A `TypeImplTrait`'s `bounds` are the same
`Punctuated<TypeParamBound>` a trait object carries, so the principal extraction and matching are
identical — only the walk (return positions) and the render prefix (`impl` vs `dyn`) differ.

## Goals / Non-Goals

**Goals:**
- `must_not_expose_impl_trait_of([trait paths])`: a returned `impl Trait` whose principal trait
  canonicalizes into the forbidden set is a violation; a returned `impl Trait` of any other trait
  passes. Reuse the return-position walk, the shared resolver, and the
  reaction/projection/baseline/exit-code contract; the only new logic is principal capture +
  operand matching. Same `syn` source, no new crate.

**Non-Goals:**
- Changing `must_not_expose_impl_trait()` — it stays shape-only.
- Changing the return-position scoping — argument-position `impl Trait` (APIT) and `async fn`'s
  implicit `impl Future` remain out of scope (inherited from the shape-only rule).
- Matching auto-trait markers as operands, or resolving a bare/unresolvable principal.

## Decisions

### Decision 1 — Mirror operand-scoped dyn exactly; empty ⇒ shape-only

`ImplTraitBoundary` gains `forbidden_operands: Vec<String>`. `must_not_expose_impl_trait()`
constructs it empty (shape-only, unchanged); `must_not_expose_impl_trait_of([...])` constructs it
non-empty. The reaction:

```
   a returned impl Trait is a finding  ⇔  forbidden_operands.is_empty()                 // shape-only
                                         ∨ forbidden_operands ∋ canon(principal trait)  // operand-scoped
```

**Empty ⇒ any** is safe by direction (`must_not_expose_impl_trait_of([])` → forbid any returned
`impl Trait`, a loud over-reaction, never a silent no-op), identical to operand-scoped dyn. The
operand findings function itself guards `forbidden.is_empty()` so the invariant holds regardless of
caller, and the check routes an empty set to the cheaper resolution-free shape-only path.

### Decision 2 — Principal trait via the shared extraction; render prefix `impl`

The `ImplTraitCollector` captures, per returned `impl Trait`, both the rendered shape (`impl …`) and
the principal trait path (the first `TypeParamBound::Trait` of the node's `bounds` — the same
extraction `trait_object_principal_path` performs for `dyn`; factor a shared
`principal_trait_path(bounds)` helper used by both). Resolution and matching are byte-for-byte the
operand-scoped dyn pipeline. Auto-trait markers are never operands (only the principal, first,
trait); an unresolvable principal — a bare std trait (`impl Iterator`, `impl Future` written bare),
a macro/glob re-export — is dropped, the inherited resolver-coverage bound, never a silent pass of a
*resolvable* operand. The operand findings path needs the module's `use`-map + re-export closure
(the shape-only path stays resolution-free).

### Decision 3 — Finding is the rendered `impl …` shape (parity); projection carries the operand set

The finding stays the rendered shape (`impl crate::ports::Port`) — it already names the trait, keeps
baseline identity `(target, rule, finding)` byte-identical to the shape-only rule, and the rule
label is unchanged (`must not expose impl trait`). The `list` JSON/markdown projection gains a
`forbidden` parameter listing the operand set when non-empty (mirroring operand-scoped dyn); a
shape-only, empty-set boundary projects unchanged.

### Decision 4 — Reuse the check / severity / baseline path unchanged

The operand variant is the same `ImplTraitBoundary`, so it flows through the existing
`check_impl_trait_*` reaction, `Severity`, `Baseline` gating, and exit-code contract (0/1/2 —
unresolvable crate/module stays a constitution error). No new reaction plumbing.

## Risks / Trade-offs

- **[Empty-operand surprise]** `_of([])` forbids any returned `impl Trait`. → safe-by-direction
  (loud over-reaction, not a silent pass); stated in the builder doc + spec.
- **[Bare-std principal unresolved]** `impl Iterator` / `impl Future` written without a `use` won't
  resolve and so can't be matched by a `crate::` operand. → the inherited, stated resolver bound;
  the primary use case (a local `crate::…` trait) resolves. Never a silent pass of a resolvable
  operand.
- **[Confusion with shape-only]** an adopter may expect `_of` to also catch other returned
  `impl Trait`s. → documented: shape-only forbids all; `_of` forbids the named subset.

## Migration Plan

No migration. Purely additive: adopters opt in with `must_not_expose_impl_trait_of([...])`; the
shape-only rule and every existing finding are unchanged. Self-governance and the existing 渾儀 tests
must stay green; new tests cover principal matching (exact + module prefix), a re-exported operand,
a non-matching returned `impl Trait` passing, the empty-⇒-any degeneracy, and projection/severity
parity. Rollback removes the builder and the operand field.

## Open Questions

- None. This is a direct composition of two shipped depths (impl-trait shape-only + operand-scoped
  dyn); the design is fully determined by their union.
