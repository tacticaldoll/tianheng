# semantic-impl-trait-boundary Specification

## Purpose
The 渾儀 (semantic) capability that governs **existential-type exposure**: a module's public API
must not **return** a written `impl Trait` (return-position `impl Trait`, RPIT). It is the
existential complement of `semantic-dyn-trait-boundary` — where that forbids the *dynamic-dispatch*
shape (`dyn`), this forbids the *existential* shape (an unnameable type the caller cannot name or
store, and to whose auto-traits the seam silently commits). It reuses the shape-only public-surface
walk and the `dyn` node's stable shape renderer, adding a return-position existential leaf; same
`syn` observation source, no new crate.

## Requirements

### Requirement: Impl-trait boundary declared in Rust

An impl-trait boundary SHALL be expressed as Rust code on an `ImplTraitBoundary`, part of the
single source of truth, declared on the 渾儀 dimension alongside signature-coupling and the
dyn-trait boundaries and composed with the other dimensions at the gate. It SHALL name a target
crate and a module anchor via `ImplTraitBoundary::in_crate("…").module("…").must_not_expose_impl_trait()`,
a human-readable reason, and a severity. The system MUST NOT require TOML, YAML, Markdown, or any
generated policy file to declare or run the boundary.

#### Scenario: Impl-trait boundary declared in Rust

- **WHEN** a developer writes `ImplTraitBoundary::in_crate("core").module("crate::core").must_not_expose_impl_trait().because("the core seam must return named types, not an existential")`
- **THEN** an impl-trait boundary is held, targeting `crate::core`, with a non-empty reason and a default `enforce` severity, ready to be composed with the semantic dimension at the gate

### Requirement: A returned impl Trait is a violation

The system SHALL emit a violation for each written `impl Trait` node appearing, at any depth, in
the **return type** of a governed module's public functions, public inherent methods, or public
trait method declarations, and SHALL report no violation when no such node is present. The finding
is the rendered `impl …` shape (e.g. `impl crate::Port`, `impl Iterator<Item = u8>`), rendered by
the same stable, injective form the dyn-trait rule uses for a `dyn` node, and **seam-qualified**
(`{rendered shape} exposed by {seam}`) exactly as `semantic-dyn-trait-boundary` so the same
returned shape at two seams stays two findings (the one forbidden bug). The reaction is
shape-only: any returned `impl Trait` reacts, no trait operand.

#### Scenario: A public function returning impl Trait is flagged

- **WHEN** the governed module declares `pub fn make() -> impl crate::Port { … }`
- **THEN** the system emits a violation whose finding is the rendered shape `impl crate::Port exposed by fn <module>::make`

#### Scenario: A returned impl Trait nested in the return type is flagged

- **WHEN** the governed module declares `pub fn maybe() -> Option<impl crate::Port> { … }`
- **THEN** the system emits a violation, because the `impl Trait` node appears at depth within the return type

#### Scenario: A trait method declaration's RPIT is flagged

- **WHEN** the governed module declares a public `trait T { fn make(&self) -> impl crate::Port; }`
- **THEN** the system emits a violation on the trait method's return position, because the trait declares the existential

#### Scenario: An argument-position impl Trait is not flagged

- **WHEN** the governed module declares `pub fn drive(p: impl crate::Port) { … }`
- **THEN** the system reports no violation, because argument-position `impl Trait` is universal (a generic parameter the caller chooses), not an existential exposure

#### Scenario: An async fn is not flagged

- **WHEN** the governed module declares `pub async fn connect() -> u8 { … }`
- **THEN** the system reports no violation from this rule, because the leaked `impl Future` is a compiler-inserted existential, not a written `impl Trait` — a distinct, out-of-scope form (never a silent claim of coverage)

#### Scenario: A trait-impl method return is not double-counted

- **WHEN** an `impl T for S` block's method returns `impl crate::Port` and the trait `T` is where that return shape is declared
- **THEN** the system does not additionally flag the trait-impl method, because a trait-impl method's return shape is dictated by the trait declaration (governed there), mirroring the dyn-trait rule's handling of trait impls

### Requirement: CI reaction, severity, baseline, and projection parity

The impl-trait boundary SHALL share the 渾儀 reaction contract with the dyn-trait boundary: findings
fold into the same aggregated report and exit-code outcome (**0** clean, **1** enforce violation,
**2** constitution/scan error such as an unresolvable crate or module); the boundary carries a
severity (`enforce` default, or `warn`) and is gated against the same `Baseline` under the shared
violation identity `(target, rule_key, fact)`, the finding being the seam-qualified rendered `impl …` shape; and
the rule projects through the `list` text/JSON/markdown projection with its own boundary section,
parallel to dyn-trait. The implementation SHALL keep the `syn` dependency quarantined in `hunyi`
(no new dependency) and SHALL NOT change the existing rules' behavior.

#### Scenario: An impl-trait violation fails CI

- **WHEN** an enforce-severity impl-trait boundary is violated
- **THEN** the system prints a report naming the target module, the rule, the offending `impl …` shape, and the reason, and exits 1

#### Scenario: An unresolvable target module is a constitution error

- **WHEN** an impl-trait boundary anchors to a crate or module not present in the workspace
- **THEN** the system emits a constitution/scan error and exits 2, never exit 0 and never exit 1

#### Scenario: Severity and baseline behave as for the dyn-trait rule

- **WHEN** a `warn`-severity impl-trait boundary is violated and no enforce boundary is, or an enforce boundary's only violations are all in the baseline
- **THEN** the reaction does not fail (exit 0); and an impl-trait violation not present in the baseline fails the reaction (exit 1)

#### Scenario: The rule projects in list output

- **WHEN** the constitution is projected via `list` (text/json/markdown)
- **THEN** the impl-trait boundary appears with its target, module, rule, severity, and reason — through its own projection section, parallel to the dyn-trait boundary

### Requirement: Impl-trait facts preserve shape and seam separately

Impl-trait violations SHALL encode the canonical forbidden shape/subject and public seam as
separate fact roles under a structured rule key. Rendered `impl ...` presentation and traversal
position SHALL NOT enter identity.

#### Scenario: The same shape at two seams stays distinct
- **WHEN** one impl-trait shape is exposed at two public seams
- **THEN** their structured seam roles produce distinct identities
