## MODIFIED Requirements

### Requirement: A returned impl Trait of a forbidden operand is a violation

The system SHALL emit a violation for each returned `impl Trait` in the governed module's public
surface **any** of whose non-auto (principal) traits canonicalizes to a member of the forbidden
operand set — a returned `impl Foo + Bar` may name several, and forbidding any one flags it — and
SHALL report no violation for a returned `impl Trait` none of whose non-auto traits is in the set. The principal trait path SHALL be canonicalized and
matched **exactly as signature-coupling matches a forbidden type** — through the *same* resolver
ladder: the module's `use` map, `crate`/`self`/`super`-relative paths, the **external-crate
name-set oracle** (declared dependencies ∪ sysroot, `.rename`- and `-`→`_`-aware, with a crate-root
`extern crate … as` rename applied and a leading-`::` head resolved against the raw set), and the
`pub use` re-export closure, then compared exact-or-module-prefix. So a re-exported or aliased trait
facade matches its defining path, **and an inline fully-qualified extern or sysroot trait operand
reacts** (`impl std::error::Error` under `must_not_expose_impl_trait_of(["std::error::Error"])`),
closing the false negative where only the `use`-aliased spelling reacted. A principal trait that is
**genuinely unresolvable** — a bare single-segment name with no `use` (neither a local item nor an
extern head), a macro-generated trait, or a glob/foreign-module re-export — is dropped, the stated
resolver-coverage bound, never a silent pass of a *resolvable* operand. Auto-trait and lifetime
bounds are never operands. The finding is the **seam-qualified** rendered `impl …` shape (`{shape}
exposed by {seam}`), and the return-position scoping is inherited unchanged (argument-position `impl
Trait` and `async fn` are not governed). A mutually-exclusive `#[cfg]` collision on the `use`-map name a principal trait resolves through — the identical discipline signature-coupling's own resolver ladder states — SHALL treat every candidate target as a possible principal and react if any is forbidden, never silently keeping only the declaration written last. The crate-wide re-export closure this resolver walks includes a `pub use` declared in a module reached only through a `cfg_attr`-wrapped `#[path]` remap — the identical crate-wide collection signature-coupling's own closure gets, never a silent gap specific to this operand-scoped resolver. A forbidden operand shaped with an empty `::`-segment (leading, trailing, or doubled `::`, or the empty string) is rejected as a constitution error, inheriting signature-coupling's own requirement for the identical reason: this resolver ladder never produces a canonicalized principal with an empty segment, so such an operand could never react. This holds identically for the subtree-scoped (`including_submodules()`) path, which canonicalizes its own copy of the forbidden set through the same rejection.

#### Scenario: A returned impl Trait of a named forbidden trait is flagged

- **WHEN** the governed module declares `pub fn make() -> impl crate::ports::Port` and the boundary forbids `["crate::ports::Port"]`
- **THEN** the system emits a violation whose finding is the seam-qualified rendered shape (`impl crate::ports::Port exposed by {seam}`)

#### Scenario: An inline fully-qualified extern trait operand reacts

- **WHEN** the governed module declares `pub fn make() -> impl std::error::Error` *inline* (no `use std::error::Error;`) and the boundary forbids `["std::error::Error"]`
- **THEN** the system resolves the principal trait through the external-crate oracle to `std::error::Error` and emits a violation — the same reaction the `use`-aliased spelling already produced

#### Scenario: A returned impl Trait of an unlisted trait passes

- **WHEN** the governed module declares `pub fn it() -> impl Iterator<Item = u8>` and the boundary forbids only `["crate::ports::Port"]`
- **THEN** the system reports no violation, because the principal trait is outside the forbidden operand set (and a bare `Iterator` does not resolve to the forbidden path)

#### Scenario: A module-prefix operand forbids a subtree of returned traits

- **WHEN** the boundary forbids `["crate::ports"]` (a module prefix) and the module declares `pub fn make() -> impl crate::ports::Port`
- **THEN** the system emits a violation, because the principal trait canonicalizes under the forbidden prefix

#### Scenario: A re-exported trait operand matches its defining path

- **WHEN** the module returns `impl crate::Port`, a `pub use crate::ports::Port` facade of the trait defined at `crate::ports::Port`, and the boundary forbids the defining path `["crate::ports::Port"]`
- **THEN** the system emits a violation, because the returned principal canonicalizes through the re-export closure to the same defining path

#### Scenario: A genuinely unresolvable bare principal is a documented bound

- **WHEN** the module returns `impl Frobnicate` where `Frobnicate` has no `use`, is not a declared dependency or sysroot crate, and is not a local trait resolvable in scope, under any operand set
- **THEN** the system does not resolve the principal and reports no violation — a stated resolver-coverage bound, never a silent claim over a resolvable operand

#### Scenario: Two mutually-exclusive cfg-gated use aliases for the principal trait's name both react

- **WHEN** the governed module declares `#[cfg(unix)] use crate::infra::Port as P; #[cfg(not(unix))] use crate::safe::SafePort as P;` and declares `pub fn make() -> impl P`, under an operand boundary forbidding `["crate::infra::Port"]`, in either declaration order
- **THEN** the system emits a violation, regardless of which `use` line is written first — the verdict never depends on source order

#### Scenario: A re-exported trait operand declared only in a cfg_attr-wrapped-path module still matches

- **WHEN** a facade module is reached only via `#[cfg_attr(windows, path = "weird.rs")] pub mod facade;` with no conventional `facade.rs` present, `weird.rs` declares `pub use crate::infra::Port;`, the governed module declares `pub fn make() -> impl crate::facade::Port`, and the boundary forbids `["crate::infra::Port"]`
- **THEN** the system reads `weird.rs` into the crate-wide re-export closure and emits a violation, rather than treating the facade module as unobserved and passing the returned principal through unresolved

#### Scenario: A malformed forbidden operand is a constitution error

- **WHEN** a boundary declares `must_not_expose_impl_trait_of(["::serde::Serialize"])` (or a trailing/doubled-`::` spelling), with or without `including_submodules()`
- **THEN** the system reports a constitution error (exit 2), rather than silently reporting the boundary satisfied
