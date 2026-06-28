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

```rust
use hunyi::{SemanticBoundary, TraitImplBoundary, VisibilityBoundary, ForbiddenMarkerBoundary};

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
