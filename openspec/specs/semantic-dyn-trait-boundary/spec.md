# semantic-dyn-trait-boundary Specification

## Purpose
The 渾儀 (semantic) capability that governs **type-shape exposure**: a module's public API must
not expose trait-object (`dyn`) syntax. It is the type-shape complement of
`semantic-signature-coupling` — where that forbids an exposed *named type*, this forbids an
exposed *type shape* (a `dyn` node at any depth in the public surface). Internal `dyn` is never a
violation; leaking dynamic dispatch across the *declared* seam is, so the rule is declarative
intent (by anchor scoping), not a lint. Shape-only: any exposed `dyn` reacts.

## Requirements

### Requirement: Dyn-trait boundary declared in Rust

A dyn-trait boundary SHALL be expressed as Rust code and is part of the single source of
truth, declared on the 渾儀 dimension alongside `semantic-signature-coupling` and composed
at the gate (never held in one unified `Constitution` object below both engines). The
boundary SHALL name a governed anchor — **a module path within a target crate** — a
human-readable reason, and a severity. The boundary SHALL be **shape-only**: it takes **no
trait operand**, so *any* exposed `dyn` reacts; a boundary that forbids only a *named*
trait's `dyn` is an explicit non-goal of this capability (a separate future capability,
born when built). The system MUST NOT require TOML, YAML, Markdown, or any generated policy
file to declare or run the boundary.

#### Scenario: Dyn-trait boundary declared in Rust

- **WHEN** a developer writes `DynTraitBoundary::in_crate("app").module("crate::core").must_not_expose_dyn().because("the core public seam must be statically dispatched")`
- **THEN** a dyn-trait boundary is held, anchored to `crate::core` in crate `app`, forbidding exposure of any trait-object syntax, with a non-empty reason and a default `enforce` severity, ready to be composed with the other dimensions at the gate

#### Scenario: The boundary carries no trait operand

- **WHEN** the boundary is declared with `must_not_expose_dyn()`
- **THEN** it reacts to every exposed `dyn` regardless of the trait named, never requiring or accepting a specific trait path (operand-scoped `dyn` is out of scope for this capability)

### Requirement: Public-surface dyn exposure reacts at any depth

The system SHALL observe the **public** API surface of the governed module anchor — the
same exposed surface as `semantic-signature-coupling` (public function parameter and return
types; public struct, enum, and union field types; public type-alias targets; public trait
method signatures and associated types; public const/static types; the generic bounds and
`where`-clauses of public items; and the public method signatures of inherent `impl`
blocks) — and SHALL react when a trait-object (`dyn`) type node appears at **any nesting
depth** within an exposed type position. The reaction is on the *presence of a `dyn` node*,
not on a top-level shape: a `dyn` nested inside `Box<…>`, `&…`, `Vec<…>`, `Option<…>`, or an
`impl Trait`'s type arguments is still exposed and reacts. A `dyn` appearing only in a
non-public (internal) position SHALL NOT be a violation — this rule governs **exposure**
across the declared seam, not internal use of dynamic dispatch.

#### Scenario: A dyn in a public return is a violation

- **WHEN** the governed module declares `pub fn connect() -> Box<dyn crate::Port>`
- **THEN** the system emits a violation naming the exposed dynamic dispatch, e.g. "`pub fn connect` exposes `dyn crate::Port`"

#### Scenario: A dyn in a public parameter is a violation

- **WHEN** the governed module declares `pub fn drive(x: &dyn crate::Port)`
- **THEN** the system emits a violation, because a parameter type is part of the exposed public signature

#### Scenario: A dyn in a public field is a violation

- **WHEN** the governed module declares `pub struct S { pub p: Box<dyn crate::Port> }`
- **THEN** the system emits a violation naming the exposed `dyn crate::Port`

#### Scenario: A nested dyn at any depth is a violation

- **WHEN** the governed module declares `pub fn all() -> Vec<Box<dyn crate::Port>>` or `pub fn maybe(x: Option<&dyn crate::Port>)`
- **THEN** the system emits a violation, because the `dyn` node is present in the exposed type at a nested depth

#### Scenario: A dyn nested inside an impl-Trait return is a violation

- **WHEN** the governed module declares `pub fn ports() -> impl Iterator<Item = Box<dyn crate::Port>>`
- **THEN** the system emits a violation, because the `dyn` node is exposed to the caller through the iterator's item type, even though the outer return is `impl Trait`

#### Scenario: An impl-Trait return with no dyn node is clean

- **WHEN** the governed module declares `pub fn port() -> impl crate::Port`
- **THEN** the system reports no violation — not because `impl Trait` is whitelisted, but because the exposed type contains **no `dyn` node**

#### Scenario: A dyn in a public const or static type is a violation

- **WHEN** the governed module declares `pub const C: &dyn crate::Port = …;` or `pub static S: &dyn crate::Port = …;`
- **THEN** the system emits a violation, because a `pub` const/static type is an exposed public position

#### Scenario: A dyn in a public trait method signature is a violation

- **WHEN** the governed module declares `pub trait Service { fn port(&self) -> Box<dyn crate::Port>; }`
- **THEN** the system emits a violation, because a public trait method signature is part of the exposed surface

#### Scenario: A dyn in a public trait associated-type default is a violation

- **WHEN** the governed module declares `pub trait Service { type Out = Box<dyn crate::Port>; }`
- **THEN** the system emits a violation, because the associated-type default is an exposed type position

#### Scenario: A dyn in a public item's where-clause is a violation

- **WHEN** the governed module declares `pub fn run<T>() where Box<dyn crate::Port>: Into<T>`
- **THEN** the system emits a violation, because the `dyn` node is syntactically present in the public item's `where`-clause and is observable; the uniform any-depth rule reacts rather than carving out an unpinned silent-pass class

#### Scenario: A dyn used only internally is clean

- **WHEN** the governed module uses `Box<dyn crate::Port>` only inside private function bodies and non-public items, exposing it in no public signature
- **THEN** the system reports no violation, because this rule governs exposure across the public seam, not internal dynamic dispatch (the surface walk descends only public items and public fields, so an internal `dyn` is never visited)

### Requirement: Public type-alias targets are governed; named aliases are not expanded

A **public** type alias whose own target writes `dyn` SHALL react at the alias item itself,
because a public type-alias target is part of the governed exposed surface. A public item
that *names* such an alias SHALL NOT receive an additional reaction by expanding the alias:
the shared `hunyi::resolve` resolver follows local `pub use` re-export chains but does
**not** expand `type` alias definitions, so a `dyn` reached only by expanding a named alias
is a **stated coverage bound**, not a claimed reaction. This is the same alias-resolution
bound `semantic-signature-coupling` carries — the dyn is still caught, at the public alias
site rather than the use site; only a *private* alias used in a public position escapes
both reactions, and that escape is the stated bound, never silently asserted clean.

#### Scenario: A public alias whose target writes dyn is a violation

- **WHEN** the governed module declares `pub type Handler = Box<dyn crate::Port>;`
- **THEN** the system emits a violation at the alias item, because the public type-alias target exposes `dyn crate::Port`

#### Scenario: A public item naming such an alias is not expanded

- **WHEN** the governed module declares `pub type Handler = Box<dyn crate::Port>;` and `pub fn make() -> Handler`
- **THEN** the system reacts at the alias declaration but emits **no additional** reaction for `make` via alias expansion — the `dyn` is already caught at the alias, and `type` aliases are not expanded (a stated bound)

#### Scenario: A private alias hiding a dyn in a public position is a stated bound

- **WHEN** the governed module declares a non-public `type Handler = Box<dyn crate::Port>;` and exposes `pub fn make() -> Handler`
- **THEN** the system does not claim to observe the hidden `dyn` (a stated coverage bound — the resolver does not expand `type` aliases), rather than silently asserting the boundary is clean

### Requirement: Stated coverage bounds with no false negative

Within the resolvable public surface there SHALL be no false negative: a `dyn` node that
*is* syntactically present in an exposed position MUST react, and the system MUST NOT
silently pass an exposed `dyn` it was able to observe. The capability inherits 渾儀's
**incidental, already-stated** coverage bounds unchanged and SHALL NOT silently assert a
boundary clean when one applies: a `dyn` introduced by **macro expansion** (the call site
writes no `dyn` token), a `dyn` reached only through a **glob import** or a
**`cfg_attr`-wrapped `#[path]` module** (an **unconditional** `#[path = "…"]` module is followed
and observed), and a `dyn` reached only by expanding a **named `type`
alias** are out of scope. No *new* essential gap is introduced by this capability.

#### Scenario: A macro-generated dyn is a documented coverage bound

- **WHEN** a macro invoked in the governed module expands to a public signature containing `dyn`, while the call site writes no `dyn` token
- **THEN** the system does not claim to observe it (the universal 渾儀 macro-expansion bound), rather than silently asserting the boundary is clean

#### Scenario: A resolvable exposed dyn is never silently passed

- **WHEN** a `dyn` node is syntactically present in a public signature of the governed anchor
- **THEN** the system emits a violation, never exit 0 for that boundary

#### Scenario: Distinct exposed dyn shapes produce distinct findings

- **WHEN** the governed anchor exposes two structurally different trait objects whose differing payload is observable — the boxed-closure family (`Box<dyn Fn(i32) -> i32>` vs `Box<dyn FnMut(String) -> bool>`), associated-type bindings (`dyn Iterator<Item = u8>` vs `<Item = u16>`), nested trait objects, lifetimes, simple const generics, macro-named or fn-pointer generic arguments — and one is recorded in the baseline as accepted
- **THEN** the second still reacts: its canonical `subject` key field differs from the first. The dimension MUST preserve every observable distinguishing payload in that field (never collapse the realistic shapes above to one key — which would silently pass a new exposure under a baselined one, the one forbidden bug); its human rendering remains diagnostic rather than identity

#### Scenario: The same dyn shape at two seams stays distinct findings

- **WHEN** the governed anchor exposes the *same* `dyn` shape (e.g. `Box<dyn crate::infra::Port>`) at two distinct public seams — two functions, or a function and a field — and one is recorded in the baseline as accepted
- **THEN** the second still reacts: its structured seam kind and item/module/owner/member fields differ, so two seams sharing a `subject` do not collapse to one `(target, rule_key, fact)` and baselining one MUST NOT mask the other (the one forbidden bug); the human finding remains seam-qualified as `{rendered shape} exposed by {seam}`

#### Scenario: An unrenderable sub-node is a stated rendering bound

- **WHEN** two trait objects differ only inside a sub-node that cannot be rendered without macro expansion, token printing, or edit-unstable spans — a complex const-generic *expression* (`dyn Foo<{ N + 1 }>`), a same-named macro with different arguments (`dyn Foo<m!(1)>` vs `dyn Foo<m!(2)>`), a `verbatim` type, or a distinction carried only by a **lifetime** (a reference lifetime or an HRTB `for<'a>` binder, which carry no architectural intent and are not rendered)
- **THEN** the system does not claim to distinguish them: they share a canonical `subject` field and key at the same seam (each still *reacts* on first occurrence; only baseline-dedup granularity is bounded). This is a **stated subject-rendering bound** — the same `(target, rule_key, fact)` granularity bound `semantic-trait-impl-locality`'s `(impl for <self_ty>)` fact carries — declared here, never a silent claim of cleanliness

### Requirement: Anchor resolution, CI reaction, severity, baseline, and report parity

The dyn-trait boundary SHALL share the 渾儀 reaction contract with
`semantic-signature-coupling`: an unresolvable module anchor is a **constitution error**
(exit 2), distinct from a violation (exit 1) and never a silent pass; findings fold into the
same aggregated report and exit-code outcome (0 clean, 1 enforce violation, 2 constitution
error, a constitution error superseding any violation in the same run); the boundary carries
a severity (`enforce` default, or `warn`) and is gated against the same `Baseline` under the
shared violation identity `(target, rule_key, fact)` (the finding **seam-qualified** as
`{rendered shape} exposed by {seam}`, per the scenarios above); and the violation report
identifies the governed anchor, the rule (`must not expose dyn`), the offending finding (the
exposed trait-object, named where resolvable), and the boundary's reason. The AST observation SHALL
remain in `hunyi` — the only crate permitted to depend on `syn` — and findings SHALL render
via the existing hand-rolled path/type stringification, never `quote`/`syn`'s `printing`
feature, so the `hunyi` dependency allowlist (`{serde_json, syn, xuanji}`) is untouched.

#### Scenario: An unresolvable anchor is a constitution error

- **WHEN** a dyn-trait boundary anchors to a module path that does not exist in the target crate's source
- **THEN** the system emits a constitution error naming the unresolved anchor and exits 2, never exit 0 and never exit 1

#### Scenario: A clean boundary passes and a violated one fails

- **WHEN** the governed anchor exposes no `dyn` in its public surface
- **THEN** the boundary is satisfied and contributes exit 0; and **WHEN** an enforce-severity dyn-trait boundary is violated, the system prints a report naming the anchor, the rule, the finding, and the reason, and exits 1

#### Scenario: Severity and baseline behave as for signature-coupling

- **WHEN** a `warn`-severity dyn-trait boundary is violated and no enforce boundary is, or an enforce boundary's only violations are all in the baseline
- **THEN** the reaction does not fail (exit 0); and a violation not present in the baseline fails the reaction (exit 1)

#### Scenario: The syn dependency stays quarantined

- **WHEN** self-governance runs against the workspace
- **THEN** the boundaries asserting `guibiao` does not depend on `syn` and `hunyi` does not depend on `tianheng` continue to hold, and the dyn-trait capability adds no dependency to `hunyi`'s allowlist

### Requirement: Dyn-trait facts preserve shape and seam separately

Dyn-trait violations SHALL encode the canonical forbidden shape/subject and public seam as separate
fact roles under a structured rule key. Stated renderer-granularity bounds MAY coalesce the same
subject at the same seam, but traversal position SHALL NOT be used to claim injectivity.

#### Scenario: The same shape at two seams stays distinct
- **WHEN** one dyn-trait shape is exposed at structurally different public seams
- **THEN** the seam fields produce distinct identities
