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

#### Scenario: A cfg-duplicated inline anchor governs every variant

- **WHEN** the anchored module is declared as two `#[cfg(…)] mod x { … }` inline variants (which `syn` parses as two separate modules, evaluating no `cfg`), and only the source-*later* variant exposes a forbidden type
- **THEN** the system observes the union of all same-named inline variants and reacts on the exposure, never resolving only the source-first variant (a `mod`-resolution divergence from the crate-wide scan is the false-negative class this resolver forbids). This anchor-resolution property is shared by every single-module-anchored semantic capability (visibility, dyn/impl-trait, async-exposure), not only signature-coupling; an **unconditional** `#[path = "…"]` file module is followed to its target, and a `cfg_attr`-wrapped `#[path]` module is followed too — its conventional file and its `cfg_attr` target both read when they exist on disk, cfg-blind union rather than a skip bound, exactly like the crate-wide walk. Only when NEITHER a conventional file NOR an existing `cfg_attr` target backs a declaration, and it carries no other cfg-conditional gate, is resolution a genuine constitution error.

#### Scenario: A cfg-mixed inline and file-form anchor governs both variants

- **WHEN** the anchored module is declared as one `#[cfg(feature = "a")] mod x { … }` inline variant and one `#[cfg(feature = "b")] mod x;` file-form variant (the standard per-platform shim pairing an inline body with a file-form sibling), and only the file-form variant exposes a forbidden type
- **THEN** the system observes both variants' items and reacts on the file-form exposure, never stopping at the inline variant merely because it was found first — the same additive, cfg-blind union as two inline variants of one name, shared by every single-module-anchored semantic capability

#### Scenario: A segment nested beneath a flat cfg-mixed sibling resolves from its own directory

- **WHEN** `x` is cfg-mixed (an inline variant on one arm, a flat, non-`mod.rs` file-form sibling on another), and the anchor is a further segment `x::y` reached through an unconditional `#[path]` written inside the flat file-form sibling itself
- **THEN** the system resolves `y` from the file-form sibling's own containing directory — the same directory a `#[path]` written in an ordinary flat file always resolves from — rather than from the inline variant's accumulated directory, which coincides with it only when the file-form sibling is `mod.rs`-shaped

#### Scenario: A plain child of a #[path]-remapped anchor resolves from the remap's own directory

- **WHEN** the anchored module is `crate::net::inner`, `crate::net` is declared `#[path = "moved/thing.rs"] pub mod net;`, and `moved/thing.rs` declares a plain `pub mod inner;`
- **THEN** the system resolves `inner` to `moved/inner.rs` — the `#[path]`-loaded file's own directory, since it is mod-rs-like regardless of its own filename — never a name-derived `net/inner.rs` that has no relationship to where the file actually lives

#### Scenario: Two non-inline cfg-sibling variants, one plain and one path-remapped, both govern

- **WHEN** the anchored module is declared as one `#[cfg(feature = "a")] mod x;` plain variant (backed by `x.rs`) and one `#[cfg(feature = "b")] #[path = "moved.rs"] mod x;` remapped variant, and only the remapped variant exposes a forbidden type
- **THEN** the system observes both variants' items and reacts on the remapped exposure, matching the crate-wide walk's own policy of never stopping at the first non-inline declaration for a name — two non-inline siblings need not name the same file once an unconditional `#[path]` can relocate one of them

#### Scenario: An inline module carrying an unconditional path is resolved, not reported unknown

- **WHEN** the anchored module is `crate::thread` (or a further segment beneath it, e.g. `crate::thread::local_data`), declared `#[path = "thread_files"] pub mod thread { pub mod local_data; }` — an inline header with an unconditional `#[path]` — and `thread_files/local_data.rs` exposes a forbidden type
- **THEN** the system resolves `crate::thread` (finding its inline items) and follows the `#[path]` to relocate the base `local_data` resolves from to `thread_files/`, reacting on the exposure — rather than reporting `crate::thread` as an unknown module merely because an unconditional `#[path]` precedes its inline header

#### Scenario: A cfg-split module's own use-map does not merge across mutually-exclusive branches

- **WHEN** the anchored module is declared as two mutually-exclusive `#[cfg]` branches, each declaring `use <different real path> as Handle;` under the same local alias name, and only the FIRST branch's own bare `Handle` reference genuinely resolves to a forbidden type
- **THEN** the system reacts on the first branch's own exposure, resolving its bare `Handle` reference through THAT branch's own `use` declaration — never through the second, mutually-exclusive branch's `use Handle` alias merely because both branches' items were observed in one pass

#### Scenario: A cfg-split branch's own child module does not shadow a sibling branch's extern re-export

- **WHEN** the anchored module is declared as two mutually-exclusive `#[cfg]` branches, one declaring a local child module with the same name as a real extern crate dependency, and the OTHER branch (with no such local child module) contains a genuine `pub use <dep>::Something;` naming the real extern crate
- **THEN** the system reacts on the second branch's own re-export, resolving `<dep>` as the real extern crate — never treating it as shadowed by a local child module that only the FIRST, mutually-exclusive branch declares

#### Scenario: Two INLINE cfg siblings sharing one enclosing file do not merge their use-maps or child-module shadows

- **WHEN** the anchored module is declared as two mutually-exclusive `#[cfg]` branches, BOTH inline (`#[cfg(a)] mod x { .. }` / `#[cfg(b)] mod x { .. }`, sharing the identical enclosing file), each declaring its own `use <different real path> as Handle;` under the same local alias name, and only the FIRST arm's own bare `Handle` reference genuinely resolves to a forbidden type
- **THEN** the system reacts on the first arm's own exposure, resolving its bare `Handle` reference through THAT arm's own `use` declaration — never through the second, mutually-exclusive arm's `use Handle` alias merely because both arms are inline and share one file; the same isolation holds for a local child module in one inline arm shadowing the other inline arm's own genuine extern re-export

#### Scenario: A mutually-exclusive SIBLING ITEM's child module does not shadow the item's own extern re-export

- **WHEN** the anchored module resolves to a SINGLE branch/file (no module-path split at all) that declares two mutually-exclusive sibling items directly — a `#[cfg(unix)] mod dep;` beside a `#[cfg(not(unix))] pub use dep::Something;` (real extern crate `dep`), or the identical pair as the two arms of one `cfg_if!` invocation
- **THEN** the system reacts on the `not(unix)`/else arm's own re-export, resolving `dep` as the real extern crate: the branch-level fix above (two DIFFERENT branches/files never merging their child-module shadows) is a no-op here, since both sibling items share the identical branch and file — the exclusion must instead be computed per re-export ITEM against its own provably-mutually-exclusive siblings, not once over the branch's whole child-module set (`semantic-reexport-exposure` owns the detailed cfg-mutual-exclusion rule this scenario exercises, on both the extern-name and the crate-root rename-alias halves)

#### Scenario: A cfg_attr-wrapped-path anchor resolves through its own target with no resolving sibling at all

- **WHEN** the anchored module `crate::foo` is declared only as `#[cfg_attr(windows, path = "win.rs")] mod foo;` with no conventional `foo.rs` present, and `win.rs` (the `cfg_attr` target) exists and exposes a forbidden type
- **THEN** the system reads `win.rs` and reacts on the exposure, rather than reporting a constitution error — a `cfg_attr`-wrapped `#[path]` module's own target is now followed even with no sibling declaration to keep the branch count non-empty

#### Scenario: A cfg_attr-wrapped-path sibling reacts through its own file, not absorbed by another sibling's success

- **WHEN** the anchored module `crate::foo` is declared as two mutually-exclusive `#[cfg]` branches — one `#[cfg_attr(<pred>, path = "weird.rs")] mod foo;` and the other a plain `mod foo;` — and only the `cfg_attr` branch's target file exposes a forbidden type
- **THEN** the system reacts on that exposure — the `cfg_attr` branch's own resolution is never silently dropped merely because the OTHER, mutually-exclusive branch's plain declaration also resolved successfully (found on adversarial review: the prior fail-loud-only-when-completely-unresolvable check never fired once any sibling succeeded, so the `cfg_attr` branch's file vanished with no error and no reaction at all)

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

### Requirement: Forbidden-type matching by path and prefix

The forbidden-type set SHALL match an exposed type either by exact resolved path or by module prefix, where prefix containment is `::`-delimited (an exact match OR an `x::` prefix), so a sibling like `crate::infrastructure` is never matched by a `crate::infra` prefix. A boundary MAY forbid more than one path or prefix.

#### Scenario: A module prefix matches a type beneath it

- **WHEN** the boundary forbids the prefix `crate::infra` and a public signature exposes `crate::infra::db::DbPool`
- **THEN** the system emits a violation, because the exposed type is beneath the forbidden prefix

#### Scenario: A prefix-colliding sibling is not matched

- **WHEN** the boundary forbids the prefix `crate::infra` and a public signature exposes `crate::infrastructure::Helper`
- **THEN** the system reports no violation, because `::`-delimited containment does not treat the sibling as beneath the prefix

### Requirement: A malformed `::`-path forbidden operand is a constitution error

A forbidden operand given to `must_not_expose`/`and_not_expose` SHALL be rejected as a **constitution
error** (exit 2) when its `::`-delimited spelling has any empty segment — a leading `::`, a trailing
`::`, a doubled `::`, or the empty string itself. This is a restriction on the **DSL operand string**
the developer writes, distinct from the "Requirement: Name resolution scope and no false negative"
section's leading-`::` guidance for the **source path being scanned**: that guidance is about how to
write `-> ::serde::Value` in the governed module's own code so it resolves as an unambiguous extern
rather than a local shadow; this requirement is about the separate `must_not_expose("...")` string,
which the resolver never produces with a leading `::` regardless of how the source is spelled (the
resolved canonical path of any extern exposure is always the bare form, e.g. `serde::Value`, never
`::serde::Value`). A forbidden operand shaped with an empty segment can therefore never equal or
prefix-contain any canonical path this system resolves, so without this requirement the boundary
would silently and permanently never react to it — the one class of bug this capability's core
contract forbids everywhere else it can occur. There is no legitimate reason to write a leading `::`
in this operand: no canonical path this crate ever produces carries one, so the spelling is always
either inert or broken, never meaningfully different from the bare form.

#### Scenario: A leading-`::` forbidden operand is a constitution error

- **WHEN** a boundary declares `must_not_expose("::serde")` against a module exposing `-> ::serde::Value`, with `serde` a real dependency
- **THEN** the system reports a constitution error (exit 2) naming the malformed operand, rather than silently reporting the boundary satisfied

#### Scenario: A trailing-`::` forbidden operand is a constitution error

- **WHEN** a boundary declares `must_not_expose("serde::")` against the same module
- **THEN** the system reports a constitution error (exit 2), for the identical reason

#### Scenario: A doubled-`::` forbidden operand is a constitution error

- **WHEN** a boundary declares `must_not_expose("::serde::")` against the same module
- **THEN** the system reports a constitution error (exit 2), for the identical reason

#### Scenario: The bare-string spelling is unaffected

- **WHEN** a boundary declares `must_not_expose("serde")` against the same module
- **THEN** the system emits a violation naming `serde::Value`, exactly as before this requirement existed

### Requirement: Name resolution scope and no false negative

The system SHALL resolve a type named in a signature using the **shared 渾儀 resolver** (`hunyi::resolve`), and within the resolved scope there SHALL be no false negative and no false positive: a forbidden type that *is* resolvable MUST react, and a name that resolves to a **local** item MUST NOT be mis-attributed to a same-named dependency. Resolution SHALL agree with rustc name resolution wherever the answer is observable from the local-crate AST:

- **A leading `::` is an unambiguous extern.** A path written `::serde::Value` resolves to the external crate named by its first segment, bypassing the `use`-map and any local shadow. It SHALL NOT be resolved as a relative path (which would both miss the extern exposure and, via the `use`-map, mis-attribute it to a local path).
- **A local type-namespace item shadows the extern prelude.** A bare head naming a local `struct`/`enum`/`union`/`trait`/`type`-alias/`mod` in the governed module denotes that local item, and the extern oracle SHALL NOT fire for it.
- **A bare local-alias chain resolves regardless of collection order.** When a type alias's target is itself a bare local alias whose name shadows a dependency (`type serde = crate::infra::Db; type X = serde;`), the alias-collection ladder SHALL resolve the local alias before the extern oracle (identical to the query ladder), closing the chain to the defining path.
- **A mutually-exclusive `#[cfg]` collision on a `use`-map name or a `pub use` re-export target does not suppress either candidate.** When two mutually-exclusive `#[cfg]` branches (bare `#[cfg]` or `cfg_if!` arms alike) each declare `use ... as Name;` (or `pub use ... as Name;`) for the identical local name with different targets, the system SHALL treat both targets as candidates and react if resolving through EITHER one exposes a forbidden type — never silently keeping only the declaration that happens to be written last.

A type whose resolution would require capabilities beyond the local AST — a glob import, a macro-generated type, a generic type alias, nominal paths nested only inside alias-target forms outside the explicitly supported non-generic compound constructors below, or full inference — remains OUT OF SCOPE, a stated coverage bound, never a claimed reaction. A type defined only in a module reached through a `cfg_attr`-wrapped `#[path]` remap is NOT out of scope: like the already-followed **unconditional** `#[path = "…"]` form, its types, aliases, and re-exports ARE collected into the crate-wide closure and resolvable — an inline body regardless of the attribute (which has no effect on it), and a file module's conventional file and its `cfg_attr` target both read when they exist on disk, cfg-blind union rather than a skip bound.

#### Scenario: A leading-`::` extern path resolves and reacts through a local shadow

- **WHEN** the governed module declares a local `mod serde` (or `use crate::vendor::serde;`) and `pub fn f() -> ::serde::Value`, under `must_not_expose("serde")`
- **THEN** the system resolves `::serde::Value` to the external crate `serde` and emits a violation, and does NOT mis-attribute it to `crate::vendor` under a boundary forbidding `crate::vendor`

#### Scenario: A local type named like a dependency is not a false positive

- **WHEN** the governed module declares `pub struct serde; pub fn f() -> serde`, under `must_not_expose("serde")`
- **THEN** the system resolves `serde` to the local struct and does NOT react, while a real `use serde::Value; pub fn g() -> Value` under the same boundary still reacts

#### Scenario: A bare local-alias-of-an-alias shadowing a dependency resolves and reacts

- **WHEN** the governed module declares `type serde = crate::infra::Db; type X = serde; pub fn f() -> X`, under `must_not_expose("crate::infra")` (in either source order)
- **THEN** the system resolves the local alias `serde` before the extern oracle, closes the chain to `crate::infra::Db`, and emits a violation

#### Scenario: Two mutually-exclusive cfg-gated use aliases for the same name both react

- **WHEN** the governed module declares `#[cfg(unix)] use crate::infra::Secret as Handle; #[cfg(not(unix))] use crate::safe::Handle; pub fn leak() -> Handle`, under `must_not_expose("crate::infra")`, in either declaration order
- **THEN** the system emits a violation naming `crate::infra::Secret`, regardless of which `use` line is written first — the verdict never depends on source order

#### Scenario: Two mutually-exclusive cfg-gated re-export targets for the same name both canonicalize correctly

- **WHEN** a facade module declares `#[cfg(unix)] pub use crate::infra::Secret as Handle; #[cfg(not(unix))] pub use crate::safe::Thing as Handle;`, another module exposes `crate::facade::Handle`, and the boundary forbids `crate::infra`, in either declaration order
- **THEN** the system emits a violation naming `crate::infra::Secret`, regardless of which `pub use` line is written first

#### Scenario: A re-export declared only in a cfg_attr-wrapped-path module is resolved and reacts

- **WHEN** a facade module is reached only via `#[cfg_attr(windows, path = "weird.rs")] pub mod facade;` with no conventional `facade.rs` present, `weird.rs` declares `pub use crate::infra::Secret;`, another module exposes `crate::facade::Secret`, and the boundary forbids `crate::infra`
- **THEN** the system reads `weird.rs` into the crate-wide re-export closure and emits a violation naming `crate::infra::Secret`, rather than treating the facade module as out of scope and passing the exposure through unresolved

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

A semantic boundary SHALL carry a severity (`enforce` by default, or `warn`) with the same meaning as a static boundary: a `warn` violation is reported but does not by itself fail the reaction. Semantic violations SHALL be gated against the same `Baseline` mechanism as static violations, sharing the violation identity `(target, rule_key, fact)`, so a project may adopt a semantic boundary on a dirty codebase and gate only on new exposure.

The `finding` SHALL be **seam-qualified**: it names both the exposed type and the public **seam** (the owning item / sub-element — a free fn, an inherent method owner-qualified by self type, a trait method, a field, a variant, a type alias, a const/static, a supertrait or associated-item position) that exposes it, rendered as `{canonical type} exposed by {seam}`. Two distinct seams exposing the *same* forbidden type therefore SHALL produce distinct findings, so baselining one exposure MUST NOT mask a new exposure of the same type at another seam (the one forbidden bug — the same guarantee async-exposure secures with its owner-qualified identity).

#### Scenario: Two seams exposing the same forbidden type stay distinct findings

- **WHEN** two public functions in the governed module each expose the forbidden type `crate::infra::DbPool`, and one is recorded in the baseline as accepted
- **THEN** the second still reacts: its finding is qualified by its own seam, so the baseline identity `(target, rule_key, fact)` does not mask it

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

### Requirement: Signature exposure facts use structural seam roles

Every signature-coupling fact SHALL separately encode the forbidden subject and the public seam
roles that make the exposure distinct. Semantic member positions such as tuple-field indices MAY be
identity-bearing observations; scan order, item ordinal, and renderer fallback position SHALL NOT.
An inherent method or associated `const`/`type` seam SHALL additionally encode the **module the impl
block itself is written in**, distinct from the self type's own canonical owner path: Rust's
coherence rules let an inherent `impl` for one type be written in any module of the same crate, so
two impl blocks in different modules for the identical self type — each declaring a same-named
public method or associated item — resolve to the identical owner and therefore MUST NOT collapse to
one seam. This module role is on behalf of every capability that builds an inherent-method/associated
seam through the shared `PublicSeam` vocabulary (dyn-trait, impl-trait), not signature-coupling
alone, matching how this spec already states shared anchor-resolution properties on their behalf.

#### Scenario: Two exposed seams stay distinct
- **WHEN** the same forbidden subject appears at two public seams
- **THEN** their structured seam roles differ and accepting one does not mask the other

#### Scenario: Reordering does not alter a seam
- **WHEN** unrelated items are inserted or declarations reordered
- **THEN** pre-existing exposure identities remain unchanged

#### Scenario: Two impl blocks in different modules for the same owner stay distinct inherent-method seams

- **WHEN** a type is declared in one module and inherent-`impl`'d with a same-named public method in
  two OTHER, sibling modules (a platform-conditional split, e.g. `plat_unix`/`plat_win` both writing
  `impl Conn { pub fn open(&self) -> impl crate::Port { … } }` for a `Conn` declared in `common`),
  observed by a capability that walks both modules in one evaluation
- **THEN** the two impl sites produce two distinct seams, qualified by each impl block's own
  declaring module in addition to the self type's owner, so accepting one does not mask the other —
  the same guarantee an inherent method already held across two DIFFERENT self types, extended to
  hold across two impl blocks of the SAME self type written in different modules

#### Scenario: The same guarantee holds for an inherent associated const/type seam

- **WHEN** two impl blocks in different modules for the same owner type each declare a same-named
  public associated `const` or `type`
- **THEN** the two associated-item seams stay distinct by the same module-qualified rule

### Requirement: Non-generic compound type aliases inspect nested nominal targets

The type alias extraction scanner SHALL inspect non-generic type alias declarations (`type Alias = Target;`) and walk nested compound type constructors—including references (`&T`), tuples (`(A, B)`), slices (`[T]`), arrays (`[T; N]`), groups, and parens—extracting all nested nominal target paths (`syn::Path`). Each nested nominal target SHALL be registered into the alias map so signature coupling boundaries react when a forbidden type is exposed through a compound type alias.

#### Scenario: Non-generic tuple type alias target is inspected

- **WHEN** a governed module declares `pub type Pair = (crate::infra::DbConn, String);` and exposes `pub fn get_pair() -> Pair` under `must_not_expose("crate::infra")`
- **THEN** the semantic boundary reacts on `crate::infra::DbConn` exposed via `Pair`

#### Scenario: Non-generic reference type alias target is inspected

- **WHEN** a governed module declares `pub type ConnRef = &'a crate::infra::DbConn;` and exposes `pub fn get_ref() -> ConnRef` under `must_not_expose("crate::infra")`
- **THEN** the semantic boundary reacts on `crate::infra::DbConn` exposed via `ConnRef`

#### Scenario: Non-generic slice type alias target is inspected

- **WHEN** a governed module declares `pub type ConnSlice = [crate::infra::DbConn];` and exposes `pub fn get_slice() -> ConnSlice` under `must_not_expose("crate::infra")`
- **THEN** the semantic boundary reacts on `crate::infra::DbConn` exposed via `ConnSlice`

### Requirement: A dual-backed module anchor is a resolution ambiguity, not a first-form pick

A governed module anchor whose plain `mod name;` is backed by BOTH conventional source forms at once — `name.rs` AND `name/mod.rs` present together — SHALL be a **constitution error** (exit 2) naming both resolved paths and the exactly-one-file rule, and SHALL NOT resolve to either form as the anchor's source. A live declaration of this shape is a genuine rustc compile error (E0761); a declaration gated off by `#[cfg(...)]` or `#[cfg_attr(...)]` is stripped by rustc before module resolution and therefore compiles, yet SHALL still be a constitution error, because observation is cfg-blind and cannot know which arm is live — the same policy the static and runtime dimensions already apply to this shape, each having placed the ambiguity check ahead of its own absent-file cfg-tolerance (三儀 ⊥ 三儀: the same rule, not the same function). cfg-tolerance covers an *absent* conventional file and never two present ones. Silently selecting the first form probed SHALL NOT be permitted: it makes whether the anchor is governed at all depend on which of the two files an author happened to write a given item in, so an item in the unselected form escapes observation entirely — a false negative, the one outcome the core contract forbids. This resolution property SHALL hold for every single-module-anchored semantic capability (signature-coupling, visibility, dyn/impl-trait, async-exposure) rather than signature-coupling alone, SHALL hold equally for a further segment resolved beneath a dual-backed ancestor, and SHALL hold for the crate-wide and subtree walks that back nearly every semantic capability — signature-coupling's own crate-wide alias and extern scan included, alongside trait-impl locality, forbidden marker, unsafe confinement, and the subtree walks behind impl-trait and async exposure — so the anchored descent and the crate-wide walk do not disagree on the identical shape, as their absent-file policies once silently had.

#### Scenario: A dual-backed anchor is a constitution error

- **WHEN** a crate declares `pub mod child;`, both `src/child.rs` and `src/child/mod.rs` exist, and a boundary anchors at `crate::child`
- **THEN** the system emits a constitution error naming both resolved paths and exits 2 — never exit 0 and never exit 1 — rather than resolving the anchor to `src/child.rs`

#### Scenario: An exposure in the unselected form does not escape

- **WHEN** a crate declares `pub mod child;`, `src/child.rs` is clean, `src/child/mod.rs` declares `pub fn leak() -> crate::forbidden::Thing`, and a boundary anchors at `crate::child` forbidding `crate::forbidden::Thing`
- **THEN** the system exits 2 on the ambiguity and never exits 0 — the exposure written in the form that would otherwise be skipped is never silently unobserved

#### Scenario: A cfg-gated dual-backed declaration is still an ambiguity, though the crate compiles

- **WHEN** the dual-backed `mod child;` declaration carries a `#[cfg(...)]` gate whose predicate is off, so rustc strips the declaration before module resolution and the crate compiles cleanly
- **THEN** the system still emits a constitution error and exits 2 — cfg-blind observation cannot know which arm is live, and the static and runtime dimensions already refuse to judge this shape; cfg-tolerance covers an *absent* conventional file and never two present ones

#### Scenario: A dual-backed module elsewhere in the crate reacts on the crate-wide walk

- **WHEN** a dual-backed module exists anywhere in the crate and a capability whose evaluation walks the crate or a subtree is evaluated — including signature-coupling's own crate-wide alias and extern scan — while the boundary names some other module
- **THEN** that walk emits the same constitution error and exits 2, matching the static and runtime dimensions' own walkers rather than judging a crate it cannot resolve

#### Scenario: A single-form anchor is unaffected

- **WHEN** exactly one of `src/child.rs` or `src/child/mod.rs` exists and a boundary anchors at `crate::child`
- **THEN** the system resolves the anchor to that form and evaluates the boundary as before, reacting on a forbidden exposure with exit 1 and permitting a clean module with exit 0

### Requirement: Transparent control-flow macro arm contents are observed

The system SHALL observe the contents of a **transparent control-flow macro** arm as real code: for a `cfg_if!` invocation in item position, each arm's items SHALL be collected exactly as if written at the invocation's own position, and a `mod` declaration inside an arm SHALL enter the module graph so its source file is scanned. A transparent macro wraps human-authored items without transforming identities, so treating its arms as macro-generated would leave a real, compiled item unobserved — an exposure written inside an arm would escape every single-module-anchored capability (signature-coupling, visibility, dyn/impl-trait, async-exposure) while the static dimension already reacts to the identical block, and a module declared only inside an arm would additionally hide its file's `unsafe` sites, forbidden markers, and trait impls from the crate-wide walk. Nested invocations SHALL recurse. An arm-declared module SHALL be treated as **cfg-conditional** for absent-file purposes — the arm's predicate gates it, every arm being conditionally compiled by construction — matching the static dimension's own rule rather than a re-derived one, while the ambiguity reaction (both conventional forms present) SHALL still fire regardless of arm membership. Transparency SHALL be gated on the macro **name** (`cfg_if`), and that gate is load-bearing rather than conservative: applied to an arbitrary macro invocation, arm extraction reads a nested `impl Foo { … }` body's braces as an arm and reports items the macro may never emit verbatim, a false positive. Three bounds SHALL therefore be stated rather than left silent — a body-wrapping macro under any **other** name is NOT covered and its contents remain unobserved; arms are unioned **cfg-blind**, so a violation written in an arm that the current configuration does not compile still reacts, since knowing which arm is live would require evaluating the whole feature and target resolution; and transparency applies to **item position** only, so an invocation written inside an `impl` or `trait` body (whose arms hold impl items rather than items, reached through a different set of walkers) is NOT flattened and its contents remain unobserved, a measured residual gap owned by its own change rather than silently absorbed into this one.

#### Scenario: An exposure inside a cfg_if arm reacts

- **WHEN** a governed module's body is `cfg_if! { if #[cfg(unix)] { pub fn leak() -> crate::forbidden::Thing { … } } else { … } }` and a boundary forbids `crate::forbidden::Thing`
- **THEN** the system reports the exposure (exit 1), rather than returning zero findings because the item sits inside a macro invocation

#### Scenario: The same exposure at top level also reacts

- **WHEN** the identical function is written directly in the governed module rather than inside `cfg_if!`
- **THEN** the system reports the exposure (exit 1) — the control establishing that the boundary reacts at all, so a clean result for the wrapped form would be a false negative rather than a misconfigured fixture

#### Scenario: Every arm of an else-if chain is observed

- **WHEN** a `cfg_if!` invocation has three arms (`if` / `else if` / `else`) and only the last exposes a forbidden type
- **THEN** the system reports that exposure — arms are unioned cfg-blind, never stopping at the first

#### Scenario: A nested cfg_if invocation is observed

- **WHEN** a forbidden exposure sits inside a `cfg_if!` invocation written inside another `cfg_if!` arm
- **THEN** the system reports the exposure, recursing into the inner invocation

#### Scenario: A module declared inside a cfg_if arm is scanned

- **WHEN** a crate declares `cfg_if! { if #[cfg(unix)] { pub mod unix_impl; } else { pub mod windows_impl; } }`, `src/unix_impl.rs` exists and contains a forbidden exposure, and a boundary governs `crate::unix_impl`
- **THEN** the system resolves the arm-declared module and reports the exposure, rather than treating `crate::unix_impl` as unknown

#### Scenario: An arm-declared module with no file is tolerated

- **WHEN** the same declaration pair exists but `src/windows_impl.rs` does not
- **THEN** the system skips the fileless arm declaration rather than reporting a missing-file constitution error, matching the static dimension's rule for the identical shape — the arm's predicate is the gate, so rustc strips the whole arm and the crate compiles

#### Scenario: An arm-declared dual-backed module is still an ambiguity

- **WHEN** a `mod child;` declared inside a `cfg_if!` arm resolves to both `src/child.rs` and `src/child/mod.rs`
- **THEN** the system reports the ambiguity constitution error (exit 2) — arm membership makes an absence tolerable, never two present files resolvable

#### Scenario: An invocation inside an impl body is a stated bound

- **WHEN** a governed module writes `impl Api { cfg_if! { if #[cfg(unix)] { pub fn leak() -> crate::forbidden::Thing { … } } } }` and a boundary forbids `crate::forbidden::Thing`
- **THEN** the system reports no exposure — transparency covers item position, where an arm's contents are items; an `impl`-body invocation's arms are impl items, observed through different walkers, and that remains a declared gap rather than a claimed reaction

#### Scenario: A macro under another name is not treated as transparent

- **WHEN** a governed module's body is `generate_wrapper! { impl Foo { pub fn hidden() -> crate::forbidden::Thing { … } } }` and a boundary forbids `crate::forbidden::Thing`
- **THEN** the system reports no exposure — the invocation is not transparent, so its body stays a stated coverage bound; extracting from it would read the `impl` body's braces as an arm and report an item the macro may never emit

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

