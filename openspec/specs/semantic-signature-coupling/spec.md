# semantic-signature-coupling Specification

## Purpose
The flagship semantic reaction: a module's public API must not **expose** a forbidden type.
Depending on a type internally is fine; naming it across the public surface — in a `pub`
signature, field, type alias, const/static, trait method, or a named public re-export — is the
leak. The complement of import-governance, and the case that provably earns the AST (`syn`):
a type named via a fully-qualified path with no `use` is invisible to a token scanner but caught
here — including an **inline external-crate path** (`-> dep::spi::Foo`), resolved via the crate's
external-crate name set (v0.1.4), with the governed module's own child modules excluded so a local
`mod dep` is not misread as the dependency. Trait-impl positions are out of scope for a bare
boundary (see the opt-in `semantic-trait-impl-exposure`); named public re-exports are in scope by
default (see `semantic-reexport-exposure`).
## Requirements
### Requirement: Semantic boundary declared in Rust

A semantic boundary SHALL be expressed as Rust code and is part of the single source of truth. Because the dependency architecture forbids a dimension from depending on another dimension's engine, each dimension owns its own declaration DSL and the static and semantic declarations are **composed at the gate** rather than held in one unified `Constitution` object (a unified object would have to live below both engines and drag the declaration DSL into the dimension-agnostic reaction model). A `SemanticBoundary` SHALL name a governed anchor — **a module path within a target crate** — a forbidden-type set, a human-readable reason, and a severity. A *type*-path anchor is out of scope for this capability and reserved for a separate future capability (a type's exposed surface needs its own specification). The system MUST NOT require TOML, YAML, Markdown, or any generated policy file to declare or run a semantic boundary.

#### Scenario: Semantic boundary declared in Rust

- **WHEN** a developer writes `SemanticBoundary::in_crate("app").module("crate::domain").must_not_expose("crate::infra").because("the domain API must not leak infrastructure types")`
- **THEN** a semantic boundary is held, anchored to `crate::domain` in crate `app`, forbidding exposure of `crate::infra`, with a non-empty reason and a default `enforce` severity, ready to be composed with the static boundaries at the gate

### Requirement: Anchor resolution

For each semantic boundary, the system SHALL resolve the named governed module anchor to a real module in the target crate's source before evaluating it. If the anchor cannot be resolved — an unknown module path, or a target crate absent from the workspace — the system SHALL treat this as a **constitution error** (exit 2), failing loud and distinct from a boundary violation (exit 1), so a mistyped anchor is not reported as architectural drift.

#### Scenario: Anchor resolves to a real item

- **WHEN** a boundary anchors to `crate::domain` and that module exists in the target crate's source
- **THEN** the system observes that module's public signatures for comparison

#### Scenario: Unresolvable anchor is a constitution error

- **WHEN** a boundary anchors to a module path that does not exist in the target crate's source
- **THEN** the system emits a constitution error naming the unresolved anchor and exits 2, never exit 0 (no silent pass) and never exit 1

### Requirement: Public-signature observation governs exposure

The system SHALL observe the **public** API surface of the governed module anchor and react to forbidden types that appear in *exposed* positions. The exposed surface SHALL comprise: public function parameter and return types; public struct, enum, and union field types; public type-alias targets; public trait method signatures and associated types; public const/static types; the generic bounds and `where`-clauses of public items where a bound names a trait by a literal, directly resolvable path; the public method signatures **and public associated `const`/`type` items** of **inherent `impl` blocks** for types defined in the module; and **named public re-exports** (specified in `semantic-reexport-exposure`). Within every observed **bound** position — a public item's generic-parameter bounds and `where`-clauses, a **trait's supertraits**, and a public **associated type's bounds and generic parameters** — a forbidden type appearing as a **generic argument** of the bound (e.g. the `crate::infra::Secret` in `AsRef<crate::infra::Secret>`) SHALL be observed with the same full-recursion coverage as any other type position, not only the bound's head trait path; comparing only the head would silently drop a resolvable forbidden type (the forbidden false negative). A public **associated type's default target** (`type Bar = crate::infra::Secret;`) is likewise an observed type position. Each exposed position SHALL be **seam-qualified injectively** so two distinct seams exposing the same forbidden type never collapse to one `(target, rule, finding_key)` baseline entry and mask a new leak — and this injectivity SHALL hold at **enum-variant field** granularity: each field of a tuple or struct variant carries a per-member seam (`variant {module}::{Enum}::{Variant}::{index|name}`, the same `::`-delimited member form struct/union fields use), mirroring struct/union fields. Trait `impl` blocks remain out of scope for a bare `must_not_expose` (governable via the opt-in `.including_trait_impls()` depth). A forbidden type used only in a non-public position SHALL NOT be a violation.

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

### Requirement: Forbidden-type matching by path and prefix

The forbidden-type set SHALL match an exposed type either by exact resolved path or by module prefix, where prefix containment is `::`-delimited (an exact match OR an `x::` prefix), so a sibling like `crate::infrastructure` is never matched by a `crate::infra` prefix. A boundary MAY forbid more than one path or prefix.

#### Scenario: A module prefix matches a type beneath it

- **WHEN** the boundary forbids the prefix `crate::infra` and a public signature exposes `crate::infra::db::DbPool`
- **THEN** the system emits a violation, because the exposed type is beneath the forbidden prefix

#### Scenario: A prefix-colliding sibling is not matched

- **WHEN** the boundary forbids the prefix `crate::infra` and a public signature exposes `crate::infrastructure::Helper`
- **THEN** the system reports no violation, because `::`-delimited containment does not treat the sibling as beneath the prefix

### Requirement: Name resolution scope and no false negative

The system SHALL resolve a type named in a signature using the **shared 渾儀 resolver** (`hunyi::resolve`), and within the resolved scope there SHALL be no false negative and no false positive: a forbidden type that *is* resolvable MUST react, and a name that resolves to a **local** item MUST NOT be mis-attributed to a same-named dependency. Resolution SHALL agree with rustc name resolution wherever the answer is observable from the local-crate AST:

- **A leading `::` is an unambiguous extern.** A path written `::serde::Value` resolves to the external crate named by its first segment, bypassing the `use`-map and any local shadow. It SHALL NOT be resolved as a relative path (which would both miss the extern exposure and, via the `use`-map, mis-attribute it to a local path).
- **A local type-namespace item shadows the extern prelude.** A bare head naming a local `struct`/`enum`/`union`/`trait`/`type`-alias/`mod` in the governed module denotes that local item, and the extern oracle SHALL NOT fire for it.
- **A bare local-alias chain resolves regardless of collection order.** When a type alias's target is itself a bare local alias whose name shadows a dependency (`type serde = crate::infra::Db; type X = serde;`), the alias-collection ladder SHALL resolve the local alias before the extern oracle (identical to the query ladder), closing the chain to the defining path.

A type whose resolution would require capabilities beyond the local AST — a glob import, a macro-generated type, a `#[path]`-remapped module, a complex-target or generic type alias, or full inference — remains OUT OF SCOPE, a stated coverage bound, never a claimed reaction.

#### Scenario: A leading-`::` extern path resolves and reacts through a local shadow

- **WHEN** the governed module declares a local `mod serde` (or `use crate::vendor::serde;`) and `pub fn f() -> ::serde::Value`, under `must_not_expose("serde")`
- **THEN** the system resolves `::serde::Value` to the external crate `serde` and emits a violation, and does NOT mis-attribute it to `crate::vendor` under a boundary forbidding `crate::vendor`

#### Scenario: A local type named like a dependency is not a false positive

- **WHEN** the governed module declares `pub struct serde; pub fn f() -> serde`, under `must_not_expose("serde")`
- **THEN** the system resolves `serde` to the local struct and does NOT react, while a real `use serde::Value; pub fn g() -> Value` under the same boundary still reacts

#### Scenario: A bare local-alias-of-an-alias shadowing a dependency resolves and reacts

- **WHEN** the governed module declares `type serde = crate::infra::Db; type X = serde; pub fn f() -> X`, under `must_not_expose("crate::infra")` (in either source order)
- **THEN** the system resolves the local alias `serde` before the extern oracle, closes the chain to `crate::infra::Db`, and emits a violation

### Requirement: CI reaction

The system SHALL fold semantic-boundary findings into the same exit-code contract as the static dimension: **exit 0** when no enforce-severity boundary is violated; **exit 1** when one or more enforce-severity boundaries are violated; **exit 2** for a constitution or scan error (e.g. an unresolvable anchor or an unreadable source file). A run that evaluates both static and semantic boundaries SHALL aggregate their findings into one report and one outcome, and a constitution error on any boundary SHALL supersede any violation in the same run.

#### Scenario: A clean semantic boundary passes

- **WHEN** the governed anchor exposes no forbidden type
- **THEN** the system reports the boundary satisfied and contributes exit 0

#### Scenario: A semantic violation fails CI

- **WHEN** an enforce-severity semantic boundary is violated
- **THEN** the system prints a report and exits 1

#### Scenario: An unresolvable anchor supersedes a violation

- **WHEN** one semantic boundary is violated and another names an unresolvable anchor
- **THEN** the system reports a constitution error and exits 2, not a violation (exit 1)

### Requirement: Severity and baseline parity

A semantic boundary SHALL carry a severity (`enforce` by default, or `warn`) with the same meaning as a static boundary: a `warn` violation is reported but does not by itself fail the reaction. Semantic violations SHALL be gated against the same `Baseline` mechanism as static violations, sharing the violation identity `(target, rule, finding_key)`, so a project may adopt a semantic boundary on a dirty codebase and gate only on new exposure.

The `finding` SHALL be **seam-qualified**: it names both the exposed type and the public **seam** (the owning item / sub-element — a free fn, an inherent method owner-qualified by self type, a trait method, a field, a variant, a type alias, a const/static, a supertrait or associated-item position) that exposes it, rendered as `{canonical type} exposed by {seam}`. Two distinct seams exposing the *same* forbidden type therefore SHALL produce distinct findings, so baselining one exposure MUST NOT mask a new exposure of the same type at another seam (the one forbidden bug — the same guarantee async-exposure secures with its owner-qualified identity).

#### Scenario: Two seams exposing the same forbidden type stay distinct findings

- **WHEN** two public functions in the governed module each expose the forbidden type `crate::infra::DbPool`, and one is recorded in the baseline as accepted
- **THEN** the second still reacts: its finding is qualified by its own seam, so the baseline identity `(target, rule, finding_key)` does not mask it

#### Scenario: A warn semantic boundary reports without failing

- **WHEN** a `warn`-severity semantic boundary is violated and no enforce-severity boundary is violated
- **THEN** the system reports the violation but the reaction does not fail (exit 0)

#### Scenario: A baselined semantic violation does not fail

- **WHEN** an enforce-severity semantic boundary's only violations are all present in the baseline
- **THEN** the system reports them as accepted and does not fail the reaction

#### Scenario: A new semantic violation beyond the baseline fails

- **WHEN** an enforce-severity semantic boundary has a violation not present in the baseline
- **THEN** the system fails the reaction (exit 1) for that new exposure

### Requirement: Human-readable violation report

A semantic violation report SHALL identify the governed anchor, the rule, the offending finding (the exposed type, seam-qualified as above), and the human-readable reason supplied with the boundary, and SHALL state that the reaction failed — the same report contract as a static violation.

#### Scenario: Report explains the exposure

- **WHEN** the public function `pool` in module `crate::domain` of crate `app` exposes `crate::infra::DbPool` under a boundary forbidding `crate::infra`
- **THEN** the report names the anchor `crate::domain`, the rule "must not expose", the finding `crate::infra::DbPool exposed by fn crate::domain::pool`, the boundary's reason, and indicates CI failed

### Requirement: The syn dependency is quarantined

The AST observation SHALL be implemented in the `hunyi` crate, which is the only crate permitted to depend on `syn`. The dependency-light static core (`guibiao`) MUST NOT acquire `syn`, and `hunyi` MUST NOT depend on the imperative shell `tianheng`. These invariants SHALL be enforced as `cargo test` self-governance gates.

#### Scenario: The core does not gain syn

- **WHEN** self-governance runs against the workspace
- **THEN** a boundary asserts `guibiao` does not depend on `syn`, and the test passes only while that holds

#### Scenario: The semantic dimension does not depend on the shell

- **WHEN** self-governance runs against the workspace
- **THEN** a boundary asserts `hunyi` does not depend on `tianheng`, and the test passes only while that holds

### Requirement: Inline dependency-rooted paths in signatures are resolved via the dependency-name oracle

The system SHALL resolve an **inline, fully-qualified external-crate path** named directly in
a public signature, field, or type position (for example a return type `-> worklane_core::spi::Foo`
or a public field of type `worklane_core::spi::Conn`) to its verbatim extern path, and react
when it is in/under the forbidden set. This closes the parity gap with the already-reacting
use-aliased form (`use worklane_core::spi::Foo; … -> Foo`), which resolves through the
`use`-map today: both spell the same public exposure of the same extern type; only the inline
spelling was silently dropped.

The external-crate determination SHALL use the external-crate name set (declared dependencies,
`-`→`_` normalized and `.rename`-aware, ∪ sysroot crates `std`/`core`/`alloc`/`proc_macro`/`test`)
**with the governed module's own child modules excluded** — a **per-module shadow**: a bare
type-position head that names a child module of the governed module (a `mod serde` making
`serde::X` denote `crate::…::serde::X`) is local, not the dependency `serde`, so it MUST NOT be
read as external. The shadow is scoped to the module being analyzed (a crate-root module never
shadows a *child* module's bare paths, and vice versa), computed from that module's own items —
not the whole crate. A bare head **in** the shadowed set resolves to its verbatim extern path; a
bare head **not** in it (a local module, a shadowed name, or a local single name) keeps its
existing non-resolving (`Ignore`) behavior — applied in the bare-fallback branch after `use`-map
and `crate`/`self`/`super` resolution — so it produces **no false positive**. (Re-export
positions use the *raw* set without this shadow, because a bare `pub use` head is external by
grammar; see `semantic-reexport-exposure`.) No DSL change; the forbidden operand is the extern
path as written in the governed source.

#### Scenario: An inline dependency-rooted return type reacts

- **WHEN** the governed module exposes `pub fn make() -> worklane_core::spi::Foo` where `worklane_core` is a declared dependency, under `must_not_expose("worklane_core::spi")`
- **THEN** the system resolves `worklane_core::spi::Foo` verbatim and emits a violation, matching the already-reacting use-aliased spelling

#### Scenario: An inline dependency-rooted field type reacts

- **WHEN** the governed module exposes `pub struct Handle { pub inner: worklane_core::spi::Conn }` under `must_not_expose("worklane_core::spi")`
- **THEN** the system emits a violation naming `worklane_core::spi::Conn`

#### Scenario: A bare local child-module path in a signature is not a false positive

- **WHEN** the governed module exposes `pub fn make() -> child::Local` where `child` is a local child module (not a declared dependency), under `must_not_expose("worklane_core::spi")`
- **THEN** the system does not resolve `child::Local` as an extern type (head is not in the set) and emits no violation — its existing non-resolving behavior is preserved

#### Scenario: A child module shadowing a dependency name is not a false positive

- **WHEN** the governed module declares its own `mod worklane_core { … }` AND the crate depends on `worklane_core`, and exposes `pub fn make() -> worklane_core::Foo` (the local child module), under `must_not_expose("worklane_core")`
- **THEN** the system does not react — the per-module shadow excludes the governed module's own `worklane_core` child from the type-position set, so the local type is not misread as the dependency (no false positive), even though a *re-export* of the dependency in the same module would still react

#### Scenario: An inline sysroot-crate type in a signature reacts

- **WHEN** the governed module exposes `pub fn lock() -> std::sync::Mutex<()>` under `must_not_expose("std::sync")`
- **THEN** the system reacts, because `std` is in the external-crate set

#### Scenario: An inline dependency-rooted path outside the forbidden set passes

- **WHEN** the governed module exposes `pub fn make() -> worklane_core::api::Handle` under `must_not_expose("worklane_core::spi")`
- **THEN** the system reports no violation (`worklane_core::api::Handle` is neither the forbidden path nor beneath `worklane_core::spi::`)

