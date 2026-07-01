# 渾儀 / hunyi

**渾環察象,形義無隱。** — *The armillary discerns the figure; neither form nor meaning can hide.*

**The semantic observation dimension of [Tianheng](https://github.com/tacticaldoll/tianheng) — the armillary.**

渾儀 (the armillary sphere) observes *meaning* via the **AST** (`syn`) — what the static
`use`-scan structurally cannot see: public signatures, `impl Trait for Type`, visibility, and
attributes/derives. It is the semantic companion to the static import boundary. The heavy
`syn` dependency is **quarantined here**, never in the static core or the reaction model.

Built capabilities (each passing Tianheng's capability-admission test — declarative, no
*essential* gap, anchorable):

- **Signature-coupling** (flagship) — a module's public API must not *expose* a forbidden
  type (depending on it internally is fine; leaking it across the public surface is the
  violation).
- **Trait-impl locality** — a trait may only be implemented in declared locations.
- **Visibility** — a module must not declare bare `pub` items.
- **Forbidden-marker** — a module's types must not acquire a forbidden trait/derive.
- **Dyn-trait** — a module's public API must not *expose* trait-object (`dyn`) syntax (the
  type-shape complement of signature-coupling: internal `dyn` is fine; leaking dynamic
  dispatch across the declared seam is the violation). Two depths: `must_not_expose_dyn()` is
  **shape-only** (any exposed `dyn` reacts), and `must_not_expose_dyn_of([...])` is
  **operand-scoped** (only a `dyn` whose principal trait resolves into the named set reacts —
  e.g. forbid `dyn crate::Port` while allowing `dyn std::error::Error`). An empty operand set
  degenerates to shape-only (any `dyn`), never a no-op; auto-trait markers (`Send`) are never
  operands; a principal trait outside the resolver's coverage (a bare std trait, macro/glob
  re-export) is the stated bound, never a silent pass of a resolvable operand.
- **Impl-trait** — a module's public API must not *return* a written `impl Trait` (RPIT), the
  **existential** complement of dyn-trait's dynamic dispatch: an RPIT at a seam leaks an
  unnameable type the caller cannot name or store, and silently commits to its auto-traits.
  Two depths: `must_not_expose_impl_trait()` is **shape-only** (any returned `impl Trait` reacts),
  and `must_not_expose_impl_trait_of([...])` is **operand-scoped** (only a returned `impl Trait`
  whose principal trait resolves into the named set reacts — e.g. allow `impl Iterator` but forbid
  `impl crate::Port`), an empty set degenerating to shape-only. Governs **return positions only**:
  argument-position `impl Trait` (APIT) is universal, not a leak, and `async fn`'s implicit
  `impl Future` is a distinct, out-of-scope existential form — both stated bounds, never silent
  misses; auto-trait markers are never operands.
- **Async-exposure** — a module's public API must not declare an `async fn`, the **implicit**
  existential complement of impl-trait: an `async fn` leaks a compiler-inserted `impl Future` and
  commits the seam to async. `must_not_expose_async_fn()` is shape-only (any public `async fn` at
  the seam reacts), over public free fns / inherent methods / trait method declarations (trait-impl
  methods and private items excluded). It governs the `async fn` sugar; a *written* `-> impl Future`
  is impl-trait's domain. The finding is an **owner-qualified item identity** (`async fn <Ty>::name(…)`)
  so two same-named async fns never collide under the baseline. Declarative = "this seam is
  synchronous" by anchor scoping (a sync-core/async-edges layering), not a blanket "no async".

```rust
use hunyi::{
    SemanticBoundary, TraitImplBoundary, VisibilityBoundary, ForbiddenMarkerBoundary,
    DynTraitBoundary, ImplTraitBoundary, AsyncExposureBoundary,
};

// exposure: my-app's public API must not leak crate::infra::DbPool
let expose = SemanticBoundary::in_crate("my-app")
    .module("crate::api")
    .must_not_expose("crate::infra::DbPool")
    .because("the API must not leak the database pool");

// impl locality: only crate::commands::* may impl Command
let locality = TraitImplBoundary::in_crate("my-app")
    .trait_("crate::Command")
    .only_implemented_in("crate::commands")
    .because("commands are registered in one place");

// visibility: the `internal` module exposes no `pub`
let visibility = VisibilityBoundary::in_crate("my-app")
    .module("crate::internal")
    .must_not_declare_pub()
    .because("internal is crate-private by contract");

// forbidden marker: domain types must not derive Serialize
let marker = ForbiddenMarkerBoundary::in_crate("my-app")
    .module("crate::domain")
    .must_not_acquire("serde::Serialize")
    .because("domain types must not be wire-coupled");

// dyn-trait (shape-only): the core's public seam must be statically dispatched
let dyn_boundary = DynTraitBoundary::in_crate("my-app")
    .module("crate::core")
    .must_not_expose_dyn()
    .because("the core public API must not leak dynamic dispatch");

// dyn-trait (operand-scoped): forbid leaking a dyn of a *named* trait, allowing others
let dyn_operand_boundary = DynTraitBoundary::in_crate("my-app")
    .module("crate::core")
    .must_not_expose_dyn_of(["crate::ports::Port"])
    .because("the core seam must not leak a dyn Port (a std dyn Error is fine)");

// impl-trait (shape-only): the core's public seam must not return an existential impl Trait (RPIT)
let impl_trait_boundary = ImplTraitBoundary::in_crate("my-app")
    .module("crate::core")
    .must_not_expose_impl_trait()
    .because("the core seam must return named types, not an unnameable existential");

// impl-trait (operand-scoped): forbid returning an existential of a *named* trait, allowing others
let impl_trait_operand_boundary = ImplTraitBoundary::in_crate("my-app")
    .module("crate::core")
    .must_not_expose_impl_trait_of(["crate::ports::Port"])
    .because("the core seam may return impl Iterator but never leak an existential Port");

// async-exposure: the core's public seam must be synchronous (no implicit impl Future)
let async_boundary = AsyncExposureBoundary::in_crate("my-app")
    .module("crate::core")
    .must_not_expose_async_fn()
    .because("the core seam is synchronous; async lives at the adapter edges");
```

**Stated bounds** (never silently passed): local `pub use` re-export chains — including
multi-hop and `as`-aliased ones — *are* followed to the item they name. What `syn`'s syntactic
view cannot see is **glob** imports, **cross-crate** re-exports, **macro**-generated names, and
types knowable only through **inference** (e.g. a return-position `impl Trait` hiding a concrete
type). These gaps are *stated*, not silently passed; an unresolvable anchor is a constitution
error, never a silent pass. Explicitly **rejected** as false-negative engines: `Send`/`Sync`
(inferred auto-traits), external trait sealing, and transitive effect-purity.

Most adopters consume the semantic dimension through the
[`tianheng`](https://crates.io/crates/tianheng) shell, which composes these boundaries into
one reaction.

## License

Licensed under either of Apache-2.0 or MIT, at your option.
