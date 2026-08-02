## MODIFIED Requirements

### Requirement: Public-signature observation governs exposure

The system SHALL observe the **public** API surface of the governed module anchor and react to forbidden types that appear in *exposed* positions. The exposed surface SHALL comprise: public function parameter and return types; public struct, enum, and union field types; public type-alias targets; public trait method signatures and associated types; public const/static types; a `pub fn`'s signature or a `pub static`'s type declared inside an `extern` block (the FFI declaration is a real item in the enclosing module's own namespace, exactly as public as a same-shaped ordinary `fn`/`static`, and Rust cannot declare both under one name in one module, so there is no identity collision in observing it identically); the generic bounds and `where`-clauses of public items where a bound names a trait by a literal, directly resolvable path; the public method signatures **and public associated `const`/`type` items** of **inherent `impl` blocks** for types defined in the module; and **named public re-exports** (specified in `semantic-reexport-exposure`). Within every observed **bound** position — a public item's generic-parameter bounds and `where`-clauses, a **trait's supertraits**, and a public **associated type's bounds and generic parameters** — a forbidden type appearing as a **generic argument** of the bound (e.g. the `crate::infra::Secret` in `AsRef<crate::infra::Secret>`) SHALL be observed with the same full-recursion coverage as any other type position, not only the bound's head trait path; comparing only the head would silently drop a resolvable forbidden type (the forbidden false negative). A public **associated type's default target** (`type Bar = crate::infra::Secret;`) is likewise an observed type position. Each exposed position SHALL be **seam-qualified injectively** so two distinct seams exposing the same forbidden type never collapse to one `(target, rule_key, fact)` baseline entry and mask a new leak — and this injectivity SHALL hold at **enum-variant field** granularity: each field of a tuple or struct variant carries a per-member seam (`variant {module}::{Enum}::{Variant}::{index|name}`, the same `::`-delimited member form struct/union fields use), mirroring struct/union fields. Trait `impl` blocks remain out of scope for a bare `must_not_expose` (governable via the opt-in `.including_trait_impls()` depth). A forbidden type used only in a non-public position SHALL NOT be a violation.

#### Scenario: A forbidden type in a public return is a violation

- **WHEN** the governed module declares `pub fn pool() -> infra::DbPool` and the boundary forbids exposing `crate::infra`
- **THEN** the system emits a violation naming the exposed type `crate::infra::DbPool`

#### Scenario: A forbidden type used only internally is clean

- **WHEN** the governed module imports and uses `crate::infra::DbPool` only inside private function bodies and non-public items, exposing it in no public signature
- **THEN** the system reports no violation, even though a static import boundary would flag the import

#### Scenario: Two forbidden fields of one enum variant stay distinct findings

- **WHEN** the governed module declares `pub enum E { V(crate::infra::Pool, crate::infra::Pool) }` under `must_not_expose("crate::infra")`
- **THEN** the system emits two distinct findings (`… variant crate::domain::E::V::0` and `… variant crate::domain::E::V::1`), so baselining the first does not mask the second — the same per-member injectivity struct fields already carry

#### Scenario: A forbidden type in a trait supertrait's generic argument is a violation

- **WHEN** the governed module declares `pub trait Facade: AsRef<crate::infra::Secret> {}` under `must_not_expose("crate::infra")`
- **THEN** the system emits a violation naming the exposed type `crate::infra::Secret`, because a supertrait bound's generic argument is walked with full recursion, not only the bound's head trait `AsRef`

#### Scenario: A forbidden type in an associated-type bound or GAT parameter is a violation

- **WHEN** the governed module declares `pub trait Facade { type Bar: Into<crate::infra::Secret>; type Gat<T: crate::infra::Marker>; }` under `must_not_expose("crate::infra")`
- **THEN** the system emits violations naming `crate::infra::Secret` (the associated-type bound's generic argument) and `crate::infra::Marker` (the GAT generic-parameter bound), the same full-recursion coverage other positions carry

#### Scenario: A forbidden type in an associated-type default is a violation

- **WHEN** the governed module declares `pub trait Facade { type Bar = crate::infra::Secret; }` under `must_not_expose("crate::infra")`
- **THEN** the system emits a violation naming `crate::infra::Secret`, because a public associated type's default target is an observed type position

#### Scenario: A forbidden type in an inherent-impl public associated const is a violation

- **WHEN** the governed module declares `impl Foo { pub const K: crate::infra::Secret = …; }` under `must_not_expose("crate::infra")`
- **THEN** the system emits a violation naming `crate::infra::Secret`, seam-qualified to `Foo`'s associated const — an inherent-`impl` public associated `const`'s type is an observed position, not only its method signatures

#### Scenario: A forbidden type in an inherent-impl public associated type target is a violation

- **WHEN** the governed module declares `impl Foo { pub type T = crate::infra::Secret; }` under `must_not_expose("crate::infra")`
- **THEN** the system emits a violation naming `crate::infra::Secret`, because an inherent-`impl` public associated `type`'s target is an observed position

#### Scenario: A non-public inherent-impl associated item is not exposed

- **WHEN** the governed module declares `impl Foo { const K: crate::infra::Secret = …; type T = crate::infra::Secret; }` (both private) under `must_not_expose("crate::infra")`
- **THEN** the system reports no violation, because only `pub` associated items of an inherent `impl` are exposed

#### Scenario: A forbidden type in an extern block's pub fn signature is a violation

- **WHEN** the governed module declares `extern "C" { pub fn handle() -> crate::infra::Secret; }` under `must_not_expose("crate::infra")`
- **THEN** the system emits a violation naming `crate::infra::Secret`, exactly as it would for a same-shaped ordinary `pub fn`

#### Scenario: A forbidden type in an extern block's pub static is a violation

- **WHEN** the governed module declares `extern "C" { pub static S: crate::infra::Secret; }` under `must_not_expose("crate::infra")`
- **THEN** the system emits a violation naming `crate::infra::Secret`, exactly as it would for a same-shaped ordinary `pub static`

#### Scenario: A non-public extern block item is not exposed

- **WHEN** the governed module declares `extern "C" { fn handle() -> crate::infra::Secret; static S: crate::infra::Secret; }` (both without `pub`) under `must_not_expose("crate::infra")`
- **THEN** the system reports no violation, because only `pub` items inside an `extern` block are exposed
