## MODIFIED Requirements

### Requirement: A dyn of a forbidden trait operand is a violation

The system SHALL emit a violation for each `dyn` node in the governed module's public surface whose
principal trait canonicalizes to a member of the forbidden operand set, and SHALL report no
violation for a `dyn` whose principal trait is outside the set. The **principal trait** is the trait
object's sole non-auto trait — matched regardless of its position among the bounds, so an auto-trait
(`Send`, `Sync`) or lifetime bound (which may be written before or after it, e.g. `dyn Send + Port`)
is never the matched operand. The principal trait path SHALL be canonicalized and matched **exactly as
signature-coupling matches a forbidden type** — through the *same* resolver ladder: the module's
`use` map, `crate`/`self`/`super`-relative paths, the **external-crate name-set oracle** (declared
dependencies ∪ sysroot, `.rename`- and `-`→`_`-aware, with a crate-root `extern crate … as` rename
applied and a leading-`::` head resolved against the raw set), and the `pub use` re-export closure,
then compared exact-or-module-prefix. So a re-exported or aliased trait facade matches its defining
path, **and an inline fully-qualified extern or sysroot trait operand reacts** (`dyn
std::error::Error` under `must_not_expose_dyn_of(["std::error::Error"])`), closing the false
negative where only the `use`-aliased spelling reacted. A principal trait that is **genuinely
unresolvable** — a bare single-segment name with no `use` (neither a local item nor an extern head),
a macro-generated trait, or a glob/foreign-module re-export — is dropped, the same stated
resolver-coverage bound signature-coupling carries, never a silent pass of a *resolvable* operand.
The finding is the **seam-qualified** rendered `dyn …` shape (`{shape} exposed by {seam}`), matching the shape-only rule. A mutually-exclusive `#[cfg]` collision on the `use`-map name the principal trait resolves through — the identical discipline signature-coupling's own resolver ladder states — SHALL treat every candidate target as a possible principal and react if any is forbidden, never silently keeping only the declaration written last. The crate-wide re-export closure this resolver walks includes a `pub use` declared in a module reached only through a `cfg_attr`-wrapped `#[path]` remap — the identical crate-wide collection signature-coupling's own closure gets, never a silent gap specific to this operand-scoped resolver. A forbidden operand shaped with an empty `::`-segment (leading, trailing, or doubled `::`, or the empty string) is rejected as a constitution error, inheriting signature-coupling's own requirement for the identical reason: this resolver ladder never produces a canonicalized principal with an empty segment, so such an operand could never react.

#### Scenario: A dyn of a named forbidden trait is flagged

- **WHEN** the governed module's public API exposes `Box<dyn crate::ports::Port>` and the boundary forbids `["crate::ports::Port"]`
- **THEN** the system emits a violation whose finding is the seam-qualified rendered shape (`dyn crate::ports::Port exposed by {seam}`), because the principal trait is in the forbidden operand set

#### Scenario: An inline fully-qualified extern trait operand reacts

- **WHEN** the governed module's public API exposes `Box<dyn std::error::Error>` *inline* (no `use std::error::Error;`) and the boundary forbids `["std::error::Error"]`
- **THEN** the system resolves the principal trait through the external-crate oracle to `std::error::Error` and emits a violation — the same reaction the `use`-aliased spelling already produced, closing the false negative

#### Scenario: A cfg-split branch's own child module does not shadow a sibling branch's extern principal

- **WHEN** the anchored module is declared as two mutually-exclusive `#[cfg]` branches, one declaring a local child module with the same name as a real extern crate dependency, and the OTHER branch (with no such local child module) exposes a `dyn` whose principal trait is written with that extern crate's bare name
- **THEN** the system resolves the second branch's own principal trait through the external-crate oracle to the real extern crate — never treating it as shadowed by a local child module that only the FIRST, mutually-exclusive branch declares

#### Scenario: Two INLINE cfg siblings sharing one enclosing file do not let one arm's child module shadow the other's extern principal

- **WHEN** the anchored module is declared as two mutually-exclusive `#[cfg]` branches, BOTH inline (sharing the identical enclosing file), one declaring a local child module with the same name as a real extern crate dependency, and the OTHER inline arm (with no such local child module) exposes a `dyn` whose principal trait is written with that extern crate's bare name
- **THEN** the system resolves the second arm's own principal trait through the external-crate oracle to the real extern crate — never treating it as shadowed by a local child module that only the FIRST inline arm declares, merely because both arms share one file

#### Scenario: A dyn of an unlisted trait passes

- **WHEN** the governed module's public API exposes `Box<dyn std::error::Error>` and the boundary forbids only `["crate::ports::Port"]`
- **THEN** the system reports no violation, because the principal trait `std::error::Error` is outside the forbidden operand set

#### Scenario: A module-prefix operand forbids a subtree of traits

- **WHEN** the boundary forbids `["crate::ports"]` (a module prefix) and the module exposes `dyn crate::ports::Port`
- **THEN** the system emits a violation, because the principal trait canonicalizes under the forbidden prefix — the same exact-or-prefix match the sibling forbidden-type rule uses

#### Scenario: A re-exported trait operand matches its defining path

- **WHEN** the module exposes `dyn crate::Port`, a `pub use crate::ports::Port` facade of the trait defined at `crate::ports::Port`, and the boundary forbids the defining path `["crate::ports::Port"]`
- **THEN** the system emits a violation, because the exposed facade canonicalizes through the re-export closure to the same defining path — closing the re-export false negative

#### Scenario: A genuinely unresolvable bare principal is a documented bound

- **WHEN** the module exposes `dyn Frobnicate` where `Frobnicate` has no `use`, is not a declared dependency or sysroot crate, and is not a local trait resolvable in scope, under any operand set
- **THEN** the system does not resolve the principal and reports no violation — a stated resolver-coverage bound (the oracle does not over-reach a single bare segment), never a silent claim of cleanliness over a resolvable operand

#### Scenario: Auto-trait markers are not operands

- **WHEN** the module exposes `dyn crate::ports::Port + Send` and the boundary forbids `["crate::ports::Port"]`
- **THEN** the system emits a violation on the principal trait `crate::ports::Port` (the sole non-auto trait); the trailing `Send` marker is not the operand, so a boundary forbidding only `["Send"]` flags nothing here — and against a bare `dyn Send`, `Send` does not resolve under `BareFallback::Ignore` and is likewise dropped

#### Scenario: Two mutually-exclusive cfg-gated use aliases for the principal trait's name both react

- **WHEN** the governed module declares `#[cfg(unix)] use crate::infra::Port as P; #[cfg(not(unix))] use crate::safe::SafePort as P;` and exposes `Box<dyn P>`, under an operand boundary forbidding `["crate::infra::Port"]`, in either declaration order
- **THEN** the system emits a violation, regardless of which `use` line is written first — the verdict never depends on source order

#### Scenario: A re-exported trait operand declared only in a cfg_attr-wrapped-path module still matches

- **WHEN** a facade module is reached only via `#[cfg_attr(windows, path = "weird.rs")] pub mod facade;` with no conventional `facade.rs` present, `weird.rs` declares `pub use crate::infra::Port;`, the governed module exposes `Box<dyn crate::facade::Port>`, and the boundary forbids `["crate::infra::Port"]`
- **THEN** the system reads `weird.rs` into the crate-wide re-export closure and emits a violation, rather than treating the facade module as unobserved and passing the exposure through unresolved

#### Scenario: A malformed forbidden operand is a constitution error

- **WHEN** a boundary declares `must_not_expose_dyn_of(["::serde::Serialize"])` (or a trailing/doubled-`::` spelling)
- **THEN** the system reports a constitution error (exit 2), rather than silently reporting the boundary satisfied
