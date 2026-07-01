## Context

渾儀's impl-trait-boundary walks a module's public surface — public free fns, public inherent
methods, public trait method declarations (excluding trait-impl methods) — via
`collect_item_return_impl_traits`, and reacts to a written `impl Trait` in the return type. An
`async fn` carries no written `impl Trait`: it is a `syn::Signature` with `asyncness = Some(..)`,
and the `impl Future` it returns is compiler-inserted. So the async-exposure rule reads the same
public-surface item set but reacts to a different, purely-syntactic signal — `sig.asyncness` — and
does not touch the return type at all.

The finding cannot be a type shape (there is no written type) nor a bare fn name (same-named public
async fns collide across impls/traits in one module — see Decision 2). It must be an owner-qualified
item identity.

## Goals / Non-Goals

**Goals:**
- `AsyncExposureBoundary` + `must_not_expose_async_fn()`: any public `async fn` declared in the
  governed module's public surface (free fn / inherent method / trait method declaration) is a
  violation. Shape-only. Owner-qualified finding. Reuse the reaction/severity/baseline/exit-code
  contract; same `syn` source, no new crate.

**Non-Goals:**
- Governing a *written* `-> impl Future` (impl-trait's domain) or a `Box<dyn Future>` (dyn-trait's);
  each rule observes its own signal.
- Trait-*impl* method bodies (asyncness dictated by the trait declaration) and private items.
- Operand-scoping (async fn is not naturally trait-operand-scoped; not applicable).

## Decisions

### Decision 1 — React to `sig.asyncness`, over the same three public item kinds as impl-trait

```
   Item::Fn (pub)                     → if sig.asyncness.is_some()      (free async fn)
   Item::Impl (inherent, trait_ None) → each pub method: sig.asyncness  (inherent async method)
   Item::Trait (pub)                  → each method decl: sig.asyncness  (trait async method — the trait DECLARES it)
   Item::Impl (trait_ Some)           → excluded (asyncness dictated by the trait — avoid double-count)
```

A pure boolean flag: no return-type walk, no resolution, no rendering of an implicit future. This is
the strongest possible observability (an always-visible local AST signal).

### Decision 2 — Finding is an OWNER-QUALIFIED item identity (correctness, not cosmetics)

The baseline identity is `(target, rule, finding)`. A bare fn name is **not** injective — a module
may hold `impl A { pub async fn run }`, `impl B { pub async fn run }`, `pub trait T { async fn run }`,
`pub trait U { async fn run }`, all named `run`. A future-*shape* (`impl Future<Output = T>`) is
worse — every async fn returning `T` collides. Either collision lets a newly-added async leak be
masked by a baselined one — a **false negative**, the one forbidden bug. So the finding embeds the
owner kind + owner path/type + fn name + a stable render of params and return:

```
   free fn          async fn <module>::name(<params>) -> <ret>
   inherent method  async fn <SelfTy>::name(<params>) -> <ret>
   trait method     async fn trait <module>::<Trait>::name(<params>) -> <ret>
```

Params render each input's type via the existing `type_to_string` (a receiver as `self` / `&self` /
`&mut self`); the return renders `sig.output`'s type (or nothing for `-> ()`). The return is for
readability and extra collision-margin, **not** to represent the implicit future. An unrenderable
type contributes `_` — the same stated render-granularity bound the sibling shape renderers carry;
the owner kind + owner + name already disambiguate the realistic cases, so a `_` in a param type
never masks a *distinct-owner* async fn.

### Decision 3 — A new boundary type, parallel to impl-trait

`AsyncExposureBoundary` (`in_crate → module → must_not_expose_async_fn → because`), a
`SemanticBoundaries.async_exposure` slot, a `check_all` entry, an `async_exposure_boundary_json` +
text + markdown section — every piece parallel to impl-trait's. Finding as Decision 2; rule label
`must not expose async fn`; identity `(module, rule, finding)`; severity `enforce`/`warn`;
unresolvable crate/module → constitution error (exit 2).

## Risks / Trade-offs

- **[Declarative gate is this dimension's weakest]** "no async fn here" could read as a style lint.
  → Mitigation: the reason must pin the intent to *implicit existential exposure at a declared
  seam* (sync-core/async-edges layering); anchor scoping keeps it a boundary, not a blanket rule.
  Stated in the builder doc + spec.
- **[Finding collision → masking]** the whole reason for the owner-qualified identity. →
  Decision 2; tested with same-named async fns across two impls and two traits in one module.
- **[Overlap with impl-trait]** a written `-> impl Future` is *not* an `async fn` (asyncness =
  false); no double-count. A public async fn returning a written `impl Trait` is flagged by each
  rule on its own axis if both are declared. Stated.

## Migration Plan

No migration. Purely additive. Self-governance (`hunyi` stays `syn`-only) and the existing 渾儀
tests must stay green; new tests cover a public async free fn / inherent method / trait method
declaration flagging, a **trait-impl** async method **not** double-flagging, a **private** async fn
not flagging, a non-async fn not flagging, and — the crux — **two same-named async fns across two
impls (and two traits) yielding two DISTINCT findings** (no baseline masking), plus severity /
baseline / projection parity.

## Open Questions

- **Broader `must_not_expose_existential`.** Whether to later unify async-exposure with impl-trait
  under one "no existential (written or implicit) at this seam" rule is deferred — recorded in
  BACKLOG; the two syntactic signals stay distinct rules until a unification earns its admission.
