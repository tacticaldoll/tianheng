## MODIFIED Requirements

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
