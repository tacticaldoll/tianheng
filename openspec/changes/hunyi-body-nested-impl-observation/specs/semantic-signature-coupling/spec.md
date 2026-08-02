## ADDED Requirements

### Requirement: An impl nested in a const or fn body is observed

The system SHALL observe an inherent `impl` block that is written as a direct statement of the outermost body of a `const` initializer (a bare `{ … }` block expression) or of a `fn`'s own body — the "const-eval trick" idiom (`const _: () = { impl Foo { … } };`, commonly used for a compile-time trait assertion or a doctest/dogfooding scratch impl) and its fn-body-nested sibling (`fn _also() { impl Foo { … } }`) — exactly as if it were written at the enclosing module's own top level, so its public method signatures and public associated `const`/`type` items are governed like any other inherent-impl public API. Rust binds an `impl` to its self type's coherence set regardless of where the `impl` is lexically written, so wrapping it in a body does not change what it makes real: the instant `Foo` itself is module-level and reachable, `Foo::leak` is real, externally callable public API whether the `impl` sits at the module's top level or inside a body. A walker that stops at a module's own top-level items therefore has a genuine observation gap here — distinct from the correct treatment of every OTHER item kind nested in a body the same way (a `fn`, `struct`, `mod`, `trait`, or another `const`/`static` written directly in a body genuinely IS scoped to that body and unreachable as `crate::…`, the existing "a body-nested module is a stated bound" reasoning, which this requirement does not disturb or extend to any item kind but `impl`). This anchor-and-item property is shared by every single-module-anchored semantic capability that observes an inherent impl's public API (async-exposure, dyn-trait, impl-trait), not only signature-coupling, matching how this spec already states the anchor-resolution property on their behalf. Recovery is bounded to exactly this shape, stated rather than left silent: only an `impl` that is a DIRECT statement of the `const`/`fn`'s own outermost block is recovered — an `impl` nested one level FURTHER inside that body (inside an `if`/`loop`/closure/nested `fn`) is NOT recovered; a `static` initializer is NOT inspected (the const-eval trick is specifically about `const`, which forces compile-time evaluation even when the binding is never read; no audited idiom uses `static` for it); and no item kind OTHER than `impl` is recovered from a body this way.

#### Scenario: A const-wrapped inherent impl's method reacts

- **WHEN** a governed module declares `pub struct Svc; const _: () = { impl Svc { pub fn leak(&self) -> crate::infra::Db { … } } };` under `must_not_expose("crate::infra")`
- **THEN** the system emits a violation naming `crate::infra::Db exposed by fn <crate::api::Svc>::leak`, rather than reporting zero findings because the impl sits inside a const initializer

#### Scenario: A fn-body-wrapped inherent impl's method reacts

- **WHEN** the identical impl is instead written `fn _also() { impl Svc { pub fn leak(&self) -> crate::infra::Db { … } } }`
- **THEN** the system emits the identical violation, rather than reporting zero findings because the impl sits inside a fn body

#### Scenario: The same method at top level also reacts (control)

- **WHEN** the identical `impl Svc { pub fn leak… }` is written directly at the module's top level, not wrapped in any body
- **THEN** the system emits the identical violation — the control establishing that the boundary reacts on this exact fixture shape at all, so a clean result for the wrapped forms would be a false negative rather than a misconfigured fixture

#### Scenario: A plain item nested the same way stays a stated bound

- **WHEN** a governed module declares `const _: () = { pub fn also_hidden() -> crate::infra::Db { … } };` — a plain `pub fn`, not wrapped in an `impl`, directly inside the const's body
- **THEN** the system reports no exposure — only an `impl` block is recovered from a body this way; a plain item nested the same way is genuinely scoped to that body and unreachable as `crate::…`, exactly like the existing body-nested-module bound, so it stays unobserved rather than a new, unaudited claim

#### Scenario: An impl nested one level further, or static-wrapped, is a stated bound

- **WHEN** the impl is written one level further inside the body (`fn _also() { if true { impl Svc { … } } }`), or the wrapping binding is a `static` rather than a `const`
- **THEN** the system reports no exposure for that impl — a stated coverage bound rather than a silent claim of cleanliness
