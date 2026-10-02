# semantic-impl-trait-operand-boundary Specification

## Purpose
The 渾儀 (semantic) capability that governs **operand-scoped existential exposure**: a public seam
must not **return** a `impl Trait` of a **named trait**. It is the named-operand depth of the
shape-only `semantic-impl-trait-boundary` — where that forbids *any* returned `impl Trait`, this
forbids only one whose **principal trait** resolves into a declared forbidden set (so a seam may
return an ergonomic `impl Iterator` yet never leak an existential `impl crate::ports::Port`). It
reuses the return-position walk and signature-coupling's resolver (resolve + re-export
canonicalization + exact-or-module-prefix match), the same machinery operand-scoped dyn uses; same
`syn` observation source, no new crate.

## Subject

- `crates/hunyi/src/*.rs`
- `crates/hunyi/src/tests/*.rs`

## Requirements
### Requirement: Operand-scoped impl-trait boundary declared in Rust

An operand-scoped impl-trait boundary SHALL be expressed as Rust code on an `ImplTraitBoundary`,
part of the single source of truth, declared on the 渾儀 dimension and composed with the other
dimensions at the gate. It SHALL name a target crate and a module anchor and a closed set of
**forbidden trait operands** via `must_not_expose_impl_trait_of([...])`, a human-readable reason,
and a severity. A returned `impl Trait` in the governed module's public surface whose **principal
trait** canonicalizes to a member of the set is a violation. The shape-only
`must_not_expose_impl_trait()` is unchanged and reacts to any returned `impl Trait`; the operand
variant is a distinct, narrower rule on the same boundary type. The system MUST NOT require TOML,
YAML, Markdown, or any generated policy file to declare or run the boundary.

The operand filter SHALL compose with the boundary's `including_submodules()` scan depth. In that
composition, the system SHALL apply the same operand-resolution ladder independently in every
descendant module branch yielded by the subtree walk; module-local `use` aliases and extern-name
shadowing MUST NOT leak between mutually exclusive cfg branches. The subtree walk's existing
missing-module, `#[path]`, and cycle guards remain authoritative, so the composed reaction fails
loud on an unobservable descendant rather than silently skipping it.

#### Scenario: Operand-scoped boundary declared in Rust

- **WHEN** a developer writes `ImplTraitBoundary::in_crate("core").module("crate::core").must_not_expose_impl_trait_of(["crate::ports::Port"]).because("the core seam may return impl Iterator but never an existential Port")`
- **THEN** an impl-trait boundary is held, targeting `crate::core`, forbidding a returned `impl Trait` whose principal trait is `crate::ports::Port`, with a non-empty reason and a default `enforce` severity, ready to be composed with the semantic dimension at the gate

#### Scenario: Operand scope composes with subtree scope

- **WHEN** an operand-scoped boundary anchored at `crate::core` calls `including_submodules()`, and
  `crate::core::adapter` returns `impl ApiPort` through a module-local
  `use crate::ports::Port as ApiPort`
- **THEN** the system resolves that descendant branch's alias, emits the forbidden-operand
  violation attributed to the descendant's source file, and keeps the violation target anchored at
  `crate::core`

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
closing the false negative where only the `use`-aliased spelling reacted. A **bare single-segment**
principal needs no `use` when the governed module's own `#[cfg]` branch **declares that name**: the
branch's own type-namespace names are the observation source, and the name is canonicalized before it
is used, so `r#type` and `type` are one name here exactly as at every other resolution site. A
principal trait that is **genuinely unresolvable** — a bare single-segment name the branch does not
declare and that is no extern head (a prelude trait, a glob-imported trait, or a name the file never
mentions), a macro-generated trait, or a glob/foreign-module re-export — is dropped, the stated
resolver-coverage bound, never a silent pass of a *resolvable* operand; the drop holds against
**every** operand spelling, including the module-qualified one a bare name would produce if it were
declared. Auto-trait and lifetime bounds are never principal operands. A forbidden operand
whose final path segment is recognized as an auto trait by the same std leaf-name test that
removes auto-trait bounds from the principal list (e.g. `Send`, including qualified and
raw-identifier spellings) SHALL be rejected as a constitution error before resolution, even in a
mixed set: the observer removes those bounds before principal resolution, so the entry can never
react. This applies equally with `including_submodules()`.
The finding is the **seam-qualified** rendered `impl …` shape (`{shape}
exposed by {seam}`), and the return-position scoping is inherited unchanged (argument-position `impl
Trait` and `async fn` are not governed). A mutually-exclusive `#[cfg]` collision on the `use`-map name a principal trait resolves through — the identical discipline signature-coupling's own resolver ladder states — SHALL treat every candidate target as a possible principal and react if any is forbidden, never silently keeping only the declaration written last. The crate-wide re-export closure this resolver walks includes a `pub use` declared in a module reached only through a `cfg_attr`-wrapped `#[path]` remap — the identical crate-wide collection signature-coupling's own closure gets, never a silent gap specific to this operand-scoped resolver. A forbidden operand shaped with an empty `::`-segment (leading, trailing, or doubled `::`, or the empty string) is rejected as a constitution error, inheriting signature-coupling's own requirement for the identical reason: this resolver ladder never produces a canonicalized principal with an empty segment, so such an operand could never react. This holds identically for the subtree-scoped (`including_submodules()`) path, which canonicalizes its own copy of the forbidden set through the same rejection.

#### Scenario: Auto-trait markers are not principal operands

- **WHEN** a module returns `impl crate::ports::Port + Send` and the boundary forbids `["crate::ports::Port"]`
- **THEN** the system emits a violation on the named principal trait; the trailing `Send` marker is removed before principal resolution
- **PINNED-BY** `impl_trait_operand_filters_auto_trait_markers_and_refuses_them_as_operands`
- **PINNED-BY** `named_principal_and_forbidden_marker_send_remain_observable`

#### Scenario: An auto-trait operand is a constitution error

- **WHEN** an impl-trait operand boundary forbids `["Send"]`, `["std::marker::Sync"]`, or a mixed set containing an auto-trait leaf, with or without `including_submodules()`
- **THEN** the system exits 2 before principal resolution, names the offending operand, and directs the author to use `must_not_expose_impl_trait_bounded_by`, or remove it
- **PINNED-BY** `impl_auto_trait_operand_is_a_constitution_error`
- **PINNED-BY** `impl_subtree_auto_trait_operand_is_a_constitution_error`
- **PINNED-BY** `mixed_auto_trait_operand_is_a_constitution_error`
- **PINNED-BY** `qualified_std_auto_trait_operand_is_a_constitution_error`
- **PINNED-BY** `qualified_local_auto_trait_leaf_is_a_constitution_error`
- **PINNED-BY** `raw_auto_trait_leaf_is_a_constitution_error`

#### Scenario: A returned impl Trait of a named forbidden trait is flagged

- **WHEN** the governed module declares `pub fn make() -> impl crate::ports::Port` and the boundary forbids `["crate::ports::Port"]`
- **THEN** the system emits a violation whose finding is the seam-qualified rendered shape (`impl crate::ports::Port exposed by {seam}`)

#### Scenario: An inline fully-qualified extern trait operand reacts

- **WHEN** the governed module declares `pub fn make() -> impl std::error::Error` *inline* (no `use std::error::Error;`) and the boundary forbids `["std::error::Error"]`
- **THEN** the system resolves the principal trait through the external-crate oracle to `std::error::Error` and emits a violation — the same reaction the `use`-aliased spelling already produced

#### Scenario: A returned impl Trait of an unlisted trait passes

- **WHEN** the governed module declares `pub fn it() -> impl Iterator<Item = u8>` and the boundary forbids only `["crate::ports::Port"]`
- **THEN** the system reports no violation, because the principal trait is outside the forbidden operand set (and a bare `Iterator` the module does not declare does not resolve at all, so it cannot match any path)

#### Scenario: A module-prefix operand forbids a subtree of returned traits

- **WHEN** the boundary forbids `["crate::ports"]` (a module prefix) and the module declares `pub fn make() -> impl crate::ports::Port`
- **THEN** the system emits a violation, because the principal trait canonicalizes under the forbidden prefix

#### Scenario: A re-exported trait operand matches its defining path

- **WHEN** the module returns `impl crate::Port`, a `pub use crate::ports::Port` facade of the trait defined at `crate::ports::Port`, and the boundary forbids the defining path `["crate::ports::Port"]`
- **THEN** the system emits a violation, because the returned principal canonicalizes through the re-export closure to the same defining path

#### Scenario: A bare principal the governed module declares resolves without a use

- **WHEN** the governed module `crate::m` declares `pub trait Frobnicate {}` and `pub fn f() -> impl Frobnicate` with no `use`, and the boundary forbids `["crate::m::Frobnicate"]`
- **THEN** the system resolves the principal against the declaring module and emits a violation, because a name its own module declares needs no import — the branch's own type-namespace names being what admits it

#### Scenario: A bare raw-identifier principal canonicalizes before it is matched

- **WHEN** the governed module `crate::m` declares `pub trait r#type {}` and `pub fn f() -> impl r#type`, and the boundary forbids the canonical `["crate::m::type"]`
- **THEN** the system emits a violation, because a raw identifier canonicalizes to the same name here as at every other resolution site — never leaving `crate::m::r#type` unmatched against the canonical forbidden spelling, which would be a silent pass of a declared operand

#### Scenario: A genuinely unresolvable bare principal is a documented bound

- **WHEN** the module returns `impl Frobnicate` where `Frobnicate` has no `use`, is not a declared dependency or sysroot crate, and is **not declared by the governed module's own branch** (a prelude trait, a glob-imported trait, or a name the file never mentions), under any operand set — including one forbidding the module-qualified spelling `crate::m::Frobnicate`
- **THEN** the system does not resolve the principal and reports no violation — a stated resolver-coverage bound, never a silent claim over a resolvable operand
- **PINNED-BY** `impl_trait_operand_genuinely_unresolvable_bare_principal_is_a_bound`

#### Scenario: Two mutually-exclusive cfg-gated use aliases for the principal trait's name both react

- **WHEN** the governed module declares `#[cfg(unix)] use crate::infra::Port as P; #[cfg(not(unix))] use crate::safe::SafePort as P;` and declares `pub fn make() -> impl P`, under an operand boundary forbidding `["crate::infra::Port"]`, in either declaration order
- **THEN** the system emits a violation, regardless of which `use` line is written first — the verdict never depends on source order

#### Scenario: A re-exported trait operand declared only in a cfg_attr-wrapped-path module still matches

- **WHEN** a facade module is reached only via `#[cfg_attr(windows, path = "weird.rs")] pub mod facade;` with no conventional `facade.rs` present, `weird.rs` declares `pub use crate::infra::Port;`, the governed module declares `pub fn make() -> impl crate::facade::Port`, and the boundary forbids `["crate::infra::Port"]`
- **THEN** the system reads `weird.rs` into the crate-wide re-export closure and emits a violation, rather than treating the facade module as unobserved and passing the returned principal through unresolved

#### Scenario: A malformed forbidden operand is a constitution error

- **WHEN** a boundary declares `must_not_expose_impl_trait_of(["::serde::Serialize"])` (or a trailing/doubled-`::` spelling), with or without `including_submodules()`
- **THEN** the system reports a constitution error (exit 2), rather than silently reporting the boundary satisfied

### Requirement: Empty operand set degenerates to shape-only, never a silent no-op

The system SHALL treat an **empty** forbidden operand set as "no operand filter — any returned
`impl Trait` is a violation" (the shape-only behavior). `must_not_expose_impl_trait()` constructs
the empty set; `must_not_expose_impl_trait_of([])` therefore reacts to any returned `impl Trait` as
well — a loud over-reaction, never a boundary that reacts to nothing. The system MUST NOT model an
operand-scoped boundary that silently passes every returned `impl Trait`.

#### Scenario: An empty operand list forbids any returned impl Trait

- **WHEN** a boundary is declared with `must_not_expose_impl_trait_of([])` and the module returns any `impl Trait`
- **THEN** the system emits a violation for that returned `impl Trait`, identical to the shape-only `must_not_expose_impl_trait()` reaction — the empty set is unfiltered, not an inert no-op

### Requirement: Reaction, severity, baseline, and projection parity with the shape-only rule

The operand-scoped impl-trait boundary SHALL share the 渾儀 impl-trait reaction contract: findings
fold into the same aggregated report and exit-code outcome (**0** clean, **1** enforce violation,
**2** constitution/scan error such as an unresolvable crate or module); the boundary carries a
severity (`enforce` default, or `warn`) and is gated against the same `Baseline` under the shared
violation identity `(target, rule_key, fact)`, the finding being the seam-qualified rendered `impl …` shape (as in the shape-only rule); and
the rule projects through the existing impl-trait `list` text/JSON/markdown projection, adding a
`forbidden` parameter listing the operand set when non-empty (a shape-only, empty-set boundary
projects unchanged). The implementation SHALL keep the `syn` dependency quarantined in `hunyi`
(no new dependency) and SHALL NOT change the return-position walk.

#### Scenario: An operand violation fails CI

- **WHEN** an enforce-severity operand-scoped impl-trait boundary is violated
- **THEN** the system prints a report naming the target module, the rule, the offending `impl …` shape, and the reason, and exits 1

#### Scenario: An unresolvable target module is a constitution error

- **WHEN** an operand-scoped impl-trait boundary anchors to a crate or module not present in the workspace
- **THEN** the system emits a constitution/scan error and exits 2, never exit 0 and never exit 1

#### Scenario: Severity and baseline behave as for the shape-only impl-trait rule

- **WHEN** a `warn`-severity operand boundary is violated and no enforce boundary is, or an enforce operand boundary's only violations are all in the baseline
- **THEN** the reaction does not fail (exit 0); and an operand violation not present in the baseline fails the reaction (exit 1)

#### Scenario: The operand set projects in list output

- **WHEN** the constitution is projected via `list` (text/json/markdown)
- **THEN** an operand-scoped boundary appears with its target, module, rule, the forbidden operand set, severity, and reason; a shape-only boundary appears exactly as before, with no operand parameter

### Requirement: Impl-trait operand facts share structured shape semantics

Operand-specific impl-trait reactions SHALL use the same structured subject/seam roles as the
shape-only rule and a distinct semantic rule key for operand policy. Presentation SHALL NOT define
their relationship or identity.

#### Scenario: Shape and operand rules do not collide
- **WHEN** the same seam violates both shape-only and operand-specific laws
- **THEN** their semantic rule keys keep the violation identities distinct

### Requirement: Auto-trait bound governance on returned impl Trait

An auto-trait bound impl-trait boundary SHALL be expressed as Rust code via
`must_not_expose_impl_trait_bounded_by([...])` on `ImplTraitModuleDraft`, targeting a module anchor
with a non-empty set of auto-trait bound names. The boundary SHALL accept only the standard auto traits:
`Send`, `Sync`, and `Unpin` under `core::marker` or `std::marker`, and `UnwindSafe` and `RefUnwindSafe`
under `core::panic` or `std::panic` (or any of the five as bare or raw identifiers). An empty bound set SHALL be rejected as a constitution error (exit 2)
directing the author to use `must_not_expose_impl_trait()`. An unrecognized bound name SHALL be rejected
as a constitution error (exit 2) directing the author to use `must_not_expose_impl_trait_of(...)`.
A malformed path (e.g. `::Send` or containing an empty segment) SHALL be rejected as a constitution error.

The system SHALL emit a violation for each returned `impl Trait` in the governed module's public surface
whose own bound list contains any of the forbidden auto-trait bounds. The boundary governs written
return-position `impl Trait` (RPIT) on free functions, inherent methods, and trait methods, and composes
with `including_submodules()`. It does NOT govern generic type parameters or `where` clauses (`<T: Send>`
or `where T: Send`), argument-position `impl Trait` (APIT), or `async fn`'s compiler-synthesized existential.
An auto-trait bound on an inner nested trait object within a returned `impl Trait` (e.g.
`impl Future<Output = Box<dyn Trait + Send>>`) belongs to the nested trait object and SHALL NOT react
under this boundary.

The rule key SHALL be `tianheng.rule/hunyi/impl-trait-auto-bound` with parameter `forbidden_auto_bounds`,
reusing fact `tianheng.fact/hunyi/impl-trait-exposure` with polarity `DenyBreach`, and projecting through
the existing `list` projections (document/text/markdown) with parameter `forbidden_auto_bounds`.
The parameter `forbidden_auto_bounds` SHALL be determined by the normalized leaf set: different syntactic
spellings expressing the same forbidden auto-trait bounds (e.g. `["Send"]`, `["std::marker::Send"]`,
`["core::marker::Send"]`, `["r#Send"]`, and redundant sets such as `["Send", "std::marker::Send"]`) produce
the identical rule identity, with `forbidden_auto_bounds` carrying the sorted, deduplicated leaf set. Path
qualifiers other than the defining module under `std` or `core` (e.g. `foo::Send`) SHALL be rejected as a constitution
error (exit 2).

#### Scenario: A returned impl Trait carrying a forbidden auto-trait bound is flagged

- **WHEN** the governed module declares a public function returning `impl Future<Output = ()> + Send` and the boundary forbids `["Send"]`
- **THEN** the system emits a violation whose finding is the seam-qualified rendered shape (`impl Future<Output = ()> + Send exposed by {seam}`)

#### Scenario: An empty auto-trait bound set is a constitution error

- **WHEN** a boundary is declared with `must_not_expose_impl_trait_bounded_by([])`
- **THEN** the system exits 2, reporting that the auto-trait bound set cannot be empty and directing the author to use `must_not_expose_impl_trait()`

#### Scenario: An unrecognized auto-trait bound name is a constitution error

- **WHEN** a boundary declares `must_not_expose_impl_trait_bounded_by(["Clone"])`
- **THEN** the system exits 2, reporting that `Clone` is not a recognized auto trait and directing the author to use `must_not_expose_impl_trait_of`

#### Scenario: A qualified impl-trait auto-trait bound names its defining module

- **WHEN** the forbidden path and public impl Trait bound spell each of `Send`, `Sync` and `Unpin` under `marker`
  and each of `UnwindSafe` and `RefUnwindSafe` under `panic`, with `std` and with `core` as the root, or spell
  any of those five under the other module with either root
- **THEN** a defining-module path produces an enforced finding, while a path under the wrong module
  is a constitution error (exit 2)
- **PINNED-BY** `auto_bound_qualified_path_matrix`
- **PINNED-BY** `declared_auto_traits_and_the_producer_are_the_same_set`

#### Scenario: An impl-trait auto-trait path outside the defining modules is refused

- **WHEN** the boundary's forbidden auto-trait path is `foo::Send`
- **THEN** the declaration is a constitution error (exit 2)
- **PINNED-BY** `impl_auto_bound_invalid_qualifier_exits_2_shallow_and_subtree`

#### Scenario: A local trait sharing an auto-trait leaf name over-reacts as an impl auto bound - a stated bound

- **WHEN** a module defines a local trait named `Send` and returns `impl Send`, under `must_not_expose_impl_trait_bounded_by(["Send"])`
- **THEN** the system over-reacts and emits a violation, because auto-trait bounds are identified by leaf name without symbol resolution
- **PINNED-BY** `impl_trait_local_auto_trait_leaf_over_reacts_is_a_bound`

#### Scenario: A macro-generated impl trait auto bound is a documented bound

- **WHEN** a module defines an item whose return-position impl trait auto bound is generated only by macro expansion with no written `impl` token in the source
- **THEN** the system does not observe the return-position impl trait and reports no violation — a documented coverage bound
- **PINNED-BY** `impl_trait_macro_generated_auto_bound_is_a_bound`

#### Scenario: Auto-trait bound rule key identity is normalized across equivalent spellings

- **WHEN** an impl-trait auto-bound boundary is declared with any of `["Send"]`, `["std::marker::Send"]`, `["core::marker::Send"]`, `["r#Send"]`, or `["Send", "std::marker::Send"]`
- **THEN** the system produces the identical rule key (`tianheng.rule/hunyi/impl-trait-auto-bound` with parameter `forbidden_auto_bounds` as `["Send"]`)
- **PINNED-BY** `impl_auto_bound_rule_key_normalized_identity`

#### Scenario: The impl-trait projection carries sorted, distinct auto-trait leaves

- **WHEN** an impl-trait auto-bound boundary is declared with `std::marker::Send`, `r#Sync`, and bare `Send`
- **THEN** JSON carries `forbidden_auto_bounds` as `["Send", "Sync"]`, and text carries `Send, Sync`
- **PINNED-BY** `auto_bound_projection_uses_normalized_leaf_set`
