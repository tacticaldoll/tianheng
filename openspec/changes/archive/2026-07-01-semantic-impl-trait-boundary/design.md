## Context

渾儀's dyn-trait-boundary walks a module's public surface (`collect_item_dyn_exposures`) and, at any
depth, collects `dyn` nodes via a `syn` visitor (`DynCollector.visit_type_trait_object`), rendering
each to a stable finding string with `trait_object_to_string` / `bound_to_string`. It is
position-agnostic: a `dyn` anywhere in the public surface (return, param, field, alias, …) is
dynamic-dispatch leakage.

`impl Trait` is different: its architectural hazard is **existential**, and an existential
`impl Trait` occurs on stable Rust in exactly one place — a **function/method return type** (RPIT).
`impl Trait` in *argument* position (APIT) is *universal* (sugar for `<T: Trait>`, the caller
supplies the type) and leaks nothing; a const/static/field cannot be typed `impl Trait` on stable;
`type A = impl Trait` (TAIT) is nightly-only. So the existential surface is precisely the return
positions of public fns/methods — a **narrower** walk than dyn's.

## Goals / Non-Goals

**Goals:**
- `ImplTraitBoundary` + `must_not_expose_impl_trait()`: any written `impl Trait` at any depth in
  the **return type** of a governed module's public functions/methods (and public trait method
  declarations) is a violation. Shape-only. Reuse the public-surface item walk and the `dyn`
  renderer's `bound_to_string`; the reaction/severity/baseline/exit-code/projection contract mirrors
  dyn-trait exactly. Same `syn` source, no new crate.

**Non-Goals (each a stated bound, not a silent gap):**
- **Argument-position `impl Trait` (APIT).** Universal, not existential — deliberately not governed.
- **`async fn`'s implicit `impl Future`.** A distinct existential form (compiler-inserted, not a
  written `impl Trait`; signalled by `sig.asyncness`) — a named future sibling, not this rule.
- **Nightly TAIT/ATPIT.** Out of scope (stable target).
- **Operand-scoping.** `must_not_expose_impl_trait_of([Trait])` is a future depth (the same
  `shape → named-operand` stair dyn-trait now has), born when built.

## Decisions

### Decision 1 — Govern return positions only; the existential surface is RPIT

The walk visits, for each **public** item, only the **return type** of a function/method and
collects `impl Trait` nodes within it (at any depth — `-> Option<impl Trait>`, `-> Box<impl Trait>`
all count):

```
   Item::Fn (pub)                    → sig.output
   Item::Impl (inherent, trait_ None)→ each pub method's sig.output
   Item::Trait (pub)                 → each trait method's sig.output (the trait DECLARES the RPIT)
   Item::Impl (trait_ Some)          → excluded (return shape dictated by the trait — as dyn-trait excludes it)
   struct/enum/union/const/static/type→ not walked (no stable written `impl Trait` position there)
```

APIT (`sig.inputs`) is **not** visited — the key difference from dyn's position-agnostic walk, and
what keeps the rule declarative-existential rather than a style lint against `impl Trait` sugar.

### Decision 2 — Shape-only, rendered with the shared `dyn` bound renderer

A `syn::Type::ImplTrait` node has `bounds: Punctuated<TypeParamBound>` — the same structure as
`TypeTraitObject`. So it renders through the existing `bound_to_string` as `impl {bounds}` (`impl
crate::Port`, `impl Iterator<Item = u8>`, `impl Fn(i32) -> i32`) — the same injective, stable
finding form the `dyn` renderer produces, so two structurally-different RPITs never collide under
the `(target, rule, finding)` baseline identity. No name resolution is involved (shape-only, like
dyn's shape-only path), so the walk needs no `use`-map or re-export closure.

### Decision 3 — A new boundary type, parallel to dyn-trait

`impl Trait` is a distinct syntactic node from `dyn`, so it is a distinct `ImplTraitBoundary`
(`in_crate → module → must_not_expose_impl_trait → because`), a `SemanticBoundaries.impl_trait`
slot, a `check_all` entry, and an `impl_trait_boundary_json` + markdown section — every piece
parallel to dyn-trait's, reusing the item-walk shape and the renderer. Finding is the rendered
`impl …` shape; rule label `must not expose impl trait`; identity `(module, rule, finding)`;
severity `enforce`/`warn`; unresolvable crate/module → constitution error (exit 2).

## Risks / Trade-offs

- **[`async fn` mistaken as covered]** an adopter may expect `async fn` (which leaks `impl Future`)
  to react. → Mitigation: stated plainly in the builder doc + spec as a distinct, out-of-scope
  existential form (its own future sibling); never silently claimed.
- **[APIT confusion]** an adopter may expect argument `impl Trait` to react. → Mitigation: stated —
  APIT is universal, not a leak; the rule governs existential (return) positions.
- **[Integration surface]** a new boundary type touches `SemanticBoundaries`, `check_all`, the
  Constitution, and the projection. → the same well-templated integration dyn-trait itself has;
  self-governance and projection-freshness gates guard it.

## Migration Plan

No migration. Purely additive: adopters opt in with `must_not_expose_impl_trait()`. Every existing
rule, finding, and projection is unchanged. Self-governance (`hunyi` stays `syn`-only) and the
existing 渾儀 tests must stay green; new tests cover a returned `impl Trait` reacting (incl. nested
`-> Option<impl Trait>`), an **argument** `impl Trait` **not** reacting, an `async fn` **not**
reacting (stated bound), a trait method declaration's RPIT reacting, a trait-*impl* method not
double-reacting, severity/baseline parity, and projection.

## Open Questions

- **Async-exposure sibling.** Whether the `async fn` existential warrants its own capability
  (`must_not_expose_async_fn` / a broader `must_not_expose_existential`) is deferred — recorded in
  BACKLOG as a named forward item, admitted only when built.
