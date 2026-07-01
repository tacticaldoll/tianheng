## Why

渾儀 now governs both existential *type* forms — a written `impl Trait` return (impl-trait, v0.1.2)
and a `dyn` (dyn-trait). It does not govern the **implicit** existential a public `async fn` leaks:
`async fn` desugars to a compiler-inserted `impl Future`, so a public `async fn` at a seam exposes
an unnameable future *and* commits the seam's public contract to an async execution model. A team
layering "synchronous core, async only at the adapter edges" has no way to declare it today —
impl-trait catches the *written* `-> impl Future`, but not the `async fn` sugar (which carries
`asyncness = true` and no written `impl Trait` node).

This change adds the **implicit-existential complement** of impl-trait: `must_not_expose_async_fn()`.
It reads a pure local AST signal (`syn::Signature.asyncness`), shape-only, at declared seams — the
implicit-future sibling of impl-trait's written-future rule.

## What Changes

- **New 渾儀 capability — async-fn exposure.** A new `AsyncExposureBoundary`
  (`in_crate("…").module("…").must_not_expose_async_fn().because("…")`) reacts when a governed
  module's public API declares an `async fn` — a public free function, a public inherent method, or
  a public trait method declaration. It is **shape-only** (any public `async fn` at the seam
  reacts).
- **Owner-qualified finding identity (not a shape, not a bare name).** The finding is an
  owner-qualified item identity — `async fn <module>::name(params) -> ret` for a free fn,
  `async fn <SelfTy>::name(…)` for an inherent method, `async fn trait <Trait>::name(…)` for a
  trait method — with a stable render of params and return. This is required for correctness: a
  bare name or a future-shape would let two same-named public async fns
  (`impl A { async fn run }` / `impl B { async fn run }`, `trait T`/`trait U`) collide under the
  `(target, rule, finding)` baseline identity, letting a newly-added async leak be masked by a
  baselined one — a false negative, the one forbidden bug. The rendered return type is for human
  readability and collision-avoidance, **not** to represent the implicit future's shape.
- **Scope aligned with impl-trait.** Govern public free functions, public inherent methods, and
  public trait method declarations; **exclude** trait-*impl* methods (their `asyncness` is dictated
  by the trait declaration — governed there, avoiding double-count) and private fns/methods.
- **Additive only.** A new boundary type + `SemanticBoundaries` slot + builder + projection,
  parallel to impl-trait; `syn` stays quarantined in `hunyi`.

## Capabilities

### New Capabilities
- `semantic-async-exposure-boundary`: a module's public API must not declare an `async fn` (the
  implicit `impl Future` existential) — the implicit-future complement of the impl-trait rule.
  Shape-only; observed from `syn::Signature.asyncness`; the finding is an owner-qualified item
  identity so distinct async fns never collide under the baseline.

### Modified Capabilities
<!-- None. impl-trait / dyn-trait / signature-coupling are unchanged; this is a new sibling
     shape-exposure rule on the same syn surface, a different syntactic signal (asyncness). -->

## Impact

- **Crate:** `hunyi` (渾儀). A new `AsyncExposureBoundary` + drafts + `must_not_expose_async_fn()`,
  a `SemanticBoundaries.async_exposure` slot + `check_all` entry, an async-fn walk over the three
  public item kinds checking `sig.asyncness`, and an owner-qualified finding renderer (reusing
  `type_to_string` for params/return). `syn`-only; no new dependency.
- **Shell (`tianheng`):** `Constitution::async_exposure_boundary(...)` + slot, re-exports + prelude,
  an `async_exposure_boundary_json` + text + `list` markdown section, parallel to impl-trait.
- **Relationship to impl-trait:** complementary, no double-count. `async fn` (`asyncness = true`,
  no written `impl Trait` node) is caught here; a *written* `-> impl Future` (`asyncness = false`)
  is caught by impl-trait. A public async fn that *also* returns a written `impl Trait` would be
  flagged by both rules if both are declared — each on its own axis, correctly.
- **Stated bounds:** governs the **declared** `async fn` syntax; a hand-written `-> impl Future`
  is impl-trait's domain, and a function returning a `Box<dyn Future>` is dyn-trait's — each rule
  observes its own signal. Private items and trait-impl method bodies are out of scope (stated).
- **Admission (declarative gate is this dimension's weakest, but holds):** the intent is governing
  an **implicit existential exposure** at a *declared* seam (a "sync core / async edges" layering),
  by anchor scoping — not a blanket "do not use async" style lint. Observability and anchoring are
  strong (a pure local AST flag; a module anchor).
- **SemVer:** additive, non-breaking → folded into the ongoing **0.1.2** (no version bump).
