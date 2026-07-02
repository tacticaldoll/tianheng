# semantic-signature-coupling Specification

## Purpose
The flagship semantic reaction: a module's public API must not **expose** a forbidden type.
Depending on a type internally is fine; naming it across the public surface — in a `pub`
signature, field, type alias, const/static, trait method, or a named public re-export — is the
leak. The complement of import-governance, and the case that provably earns the AST (`syn`):
a type named via a fully-qualified path with no `use` is invisible to a token scanner but caught
here. Trait-impl positions are out of scope for a bare boundary (see the opt-in
`semantic-trait-impl-exposure`); named public re-exports are in scope by default (see
`semantic-reexport-exposure`).

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

The system SHALL observe the **public** API surface of the governed module anchor and react to forbidden types that appear in *exposed* positions. The exposed surface SHALL comprise: public function parameter and return types; public struct, enum, and union field types; public type-alias targets; public trait method signatures and associated types; public const/static types; the generic bounds and `where`-clauses of public items where a bound names a trait by a literal, directly resolvable path; the public method signatures of **inherent `impl` blocks** for types defined in the module (those methods are public API the module itself authored); and **named public re-exports** — a bare `pub use` that republishes a forbidden type under the module's own path is itself an exposure (the most direct one: a consumer can name it through the module), specified in `semantic-reexport-exposure`. **Trait `impl` blocks SHALL remain out of scope for a bare `must_not_expose`** (the v1 default): a bare boundary does not govern any trait impl. A trait impl's *impl-site-authored* positions — the trait's generic arguments, the `Self` type, associated-type bindings, the impl's own generics / `where`-clause, and the method **return type as written at the impl site** (which return-position `impl Trait` in traits lets the impl author refine to a concrete type) — ARE observable via the opt-in `.including_trait_impls()` depth specified in `semantic-trait-impl-exposure`; without that opt-in they SHALL NOT be a violation. Trait impl method **parameter and receiver** types remain trait-dictated (invariant with the trait declaration, not refinable at the impl site) and are governed at the trait's own definition, not by this opt-in. A forbidden type that is imported or used only in a non-public (internal) position SHALL NOT be a violation — this rule governs exposure, the complement of the static import boundary.

#### Scenario: A forbidden type in a public return is a violation

- **WHEN** the governed module declares `pub fn pool() -> infra::DbPool` and the boundary forbids exposing `crate::infra`
- **THEN** the system emits a violation naming the exposed type `crate::infra::DbPool`

#### Scenario: A forbidden type used only internally is clean

- **WHEN** the governed module imports and uses `crate::infra::DbPool` only inside private function bodies and non-public items, exposing it in no public signature
- **THEN** the system reports no violation, even though a static import boundary would flag the import

#### Scenario: A forbidden type in a public field is a violation

- **WHEN** the governed module declares `pub struct Service { pub pool: crate::infra::DbPool }` and the boundary forbids exposing `crate::infra`
- **THEN** the system emits a violation naming the exposed type

#### Scenario: A forbidden type in an inherent impl public method is a violation

- **WHEN** the governed module declares `impl Service { pub fn pool(&self) -> crate::infra::DbPool { … } }` and the boundary forbids exposing `crate::infra`
- **THEN** the system emits a violation, because an inherent `impl` block's public method is part of the module's authored public API

#### Scenario: A forbidden trait named in a generic bound is a violation

- **WHEN** the governed module declares `pub fn run<T: crate::infra::Pooled>(t: T)` and the boundary forbids exposing `crate::infra`
- **THEN** the system emits a violation naming `crate::infra::Pooled`, because the bound writes the trait path literally and needs no inference to resolve

#### Scenario: A trait impl is out of scope for a bare boundary, governable via the opt-in depth

- **WHEN** the governed module declares `impl From<crate::infra::DbPool> for Service { … }` and the boundary forbids exposing `crate::infra` with a **bare** `must_not_expose` (no `.including_trait_impls()`)
- **THEN** the system does not claim to govern the trait impl (the v1 bound is preserved), rather than silently asserting the boundary is clean; adding `.including_trait_impls()` opts into the deeper surface governed by `semantic-trait-impl-exposure`

#### Scenario: A named public re-export of a forbidden type is a violation by default

- **WHEN** the governed module declares `pub use crate::infra::DbPool;` under a bare `must_not_expose("crate::infra")` (no opt-in)
- **THEN** the system emits a violation naming `crate::infra::DbPool` exposed by the re-export, because a public re-export republishes the forbidden type under the module's own public path — the exposure the boundary always meant to catch

### Requirement: Forbidden-type matching by path and prefix

The forbidden-type set SHALL match an exposed type either by exact resolved path or by module prefix, where prefix containment is `::`-delimited (an exact match OR an `x::` prefix), so a sibling like `crate::infrastructure` is never matched by a `crate::infra` prefix. A boundary MAY forbid more than one path or prefix.

#### Scenario: A module prefix matches a type beneath it

- **WHEN** the boundary forbids the prefix `crate::infra` and a public signature exposes `crate::infra::db::DbPool`
- **THEN** the system emits a violation, because the exposed type is beneath the forbidden prefix

#### Scenario: A prefix-colliding sibling is not matched

- **WHEN** the boundary forbids the prefix `crate::infra` and a public signature exposes `crate::infrastructure::Helper`
- **THEN** the system reports no violation, because `::`-delimited containment does not treat the sibling as beneath the prefix

### Requirement: Name resolution scope and no false negative

The system SHALL resolve a type named in a signature to a path using the **shared 渾儀 resolver** (`hunyi::resolve`): the in-scope `use` declarations of the file, including renamed imports (`use … as …`) and fully path-qualified mentions, `crate::`/`self`/`super`-relative paths (including a `use` target that is itself `self`/`super`-relative), and **local `pub use` re-export chains** so a forbidden type reached through a re-export (facade) path resolves to its defining path. A type whose resolution would require capabilities beyond this — a glob import (`use …::*`), a macro-generated type, a `#[path]`-remapped module, or full type inference (a return-position `impl Trait` that hides a concrete type, or an alias chain) — is OUT OF SCOPE, a stated coverage bound, not a claimed reaction. Within the resolved scope there SHALL be no false negative: a forbidden type that *is* resolvable MUST react. The system MUST NOT silently pass an exposed type it was able to resolve to a forbidden path. (Resolution previously stopped at the file's own `use` targets, so a forbidden type re-exported through a facade was an undocumented silent pass; following `pub use` chains closes that false negative — the one bug the core contract forbids.)

#### Scenario: A renamed import resolves and reacts

- **WHEN** the governed module declares `use crate::infra::DbPool as Pool;` and exposes `pub fn pool() -> Pool`
- **THEN** the system resolves `Pool` to `crate::infra::DbPool` and emits a violation

#### Scenario: A forbidden type exposed through a re-export facade resolves and reacts

- **WHEN** the governed module declares `use crate::facade::DbPool;` (where `crate::facade` declares `pub use crate::infra::DbPool;`) and exposes `pub fn pool() -> DbPool`, under a boundary forbidding `crate::infra`
- **THEN** the system follows the `pub use` chain, resolves `DbPool` to `crate::infra::DbPool`, and emits a violation, rather than silently passing it

#### Scenario: A glob-imported forbidden type is a documented coverage bound

- **WHEN** the governed module declares `use crate::infra::*;` and exposes `pub fn pool() -> DbPool`
- **THEN** the system does not claim to observe it (out of scope, consistent with the static scanner's glob bound), rather than silently asserting the boundary is clean

#### Scenario: An opaque return that hides a forbidden type is a documented inference bound

- **WHEN** the governed module exposes `pub fn pool() -> impl std::fmt::Display` whose concrete return is `crate::infra::DbPool`, or returns a type knowable only through an alias chain requiring inference — the forbidden type being hidden behind the opaque signature rather than named in it
- **THEN** the system treats the hidden type as an out-of-scope inference bound (the semantic dimension's own incidental gap), rather than silently asserting the boundary is clean

(A forbidden trait *named literally* in the opaque bound, e.g. `-> impl crate::infra::Pooled`, is by contrast directly resolvable and reacts — it is not this bound.)

#### Scenario: A resolvable forbidden type is never silently passed

- **WHEN** a forbidden type is exposed in a public signature and is resolvable from an in-scope `use`
- **THEN** the system emits a violation, never exit 0 for that boundary

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

A semantic boundary SHALL carry a severity (`enforce` by default, or `warn`) with the same meaning as a static boundary: a `warn` violation is reported but does not by itself fail the reaction. Semantic violations SHALL be gated against the same `Baseline` mechanism as static violations, sharing the violation identity `(target, rule, finding)`, so a project may adopt a semantic boundary on a dirty codebase and gate only on new exposure.

The `finding` SHALL be **seam-qualified**: it names both the exposed type and the public **seam** (the owning item / sub-element — a free fn, an inherent method owner-qualified by self type, a trait method, a field, a variant, a type alias, a const/static, a supertrait or associated-item position) that exposes it, rendered as `{canonical type} exposed by {seam}`. Two distinct seams exposing the *same* forbidden type therefore SHALL produce distinct findings, so baselining one exposure MUST NOT mask a new exposure of the same type at another seam (the one forbidden bug — the same guarantee async-exposure secures with its owner-qualified identity).

#### Scenario: Two seams exposing the same forbidden type stay distinct findings

- **WHEN** two public functions in the governed module each expose the forbidden type `crate::infra::DbPool`, and one is recorded in the baseline as accepted
- **THEN** the second still reacts: its finding is qualified by its own seam, so the baseline identity `(target, rule, finding)` does not mask it

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

