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

### Requirement: Subtree scope opt-in

An impl-trait boundary SHALL support an opt-in **subtree scope** via `including_submodules()` on the
rule draft, defaulting OFF (a boundary without it governs the anchored module's own seam, per the
existing requirement above, byte-identically in reaction and projection). When set, the reaction
SHALL descend the anchored module's **whole subtree** — every descendant module, file-based `mod x;`
and inline `mod x { … }` alike — and SHALL emit a violation for every returned `impl Trait` node at
or below the anchor, each attributed to its enclosing module. Anchoring at `crate` with the opt-in
SHALL govern the whole crate. Within the observed subtree there SHALL be no false negative: a
returned `impl Trait` in any descendant module MUST react.

The violation `target` SHALL remain the boundary's anchored module (not the deeper enclosing
module), so a finding's identity `(target, rule_key, fact)` is stable whether or not the opt-in is
set — enabling it adds only new, deeper findings and never re-identifies a seam finding (baseline
stability). A seam finding (one in the anchored module itself) under the opt-in SHALL be
byte-identical to the same finding under the default scope.

The subtree walk SHALL inherit the crate-scan family's guards so it never silently under-reacts: an
**unconditional** `#[path = "…"]` module SHALL be followed and observed, while a `cfg_attr`-wrapped
`#[path]` SHALL remain a stated coverage bound (not followed cfg-blind); a `#[cfg]`-gated module
absent when its feature is off SHALL be tolerated; a non-`#[cfg]` missing module file SHALL be a scan
error (exit 2); a symlink module cycle SHALL be a scan error (exit 2), never a stack overflow. A
`mod` declared inside a **function body** SHALL be a stated bound (not observed) — it is not part of
the public module tree, so this rule, which governs the *public* seam, makes no claim about it,
rather than silently asserting cleanliness.

The subtree opt-in SHALL project through the `list` text/JSON/markdown output only when set, so a
bare boundary's projection stays byte-identical.

A returned `impl Trait` whose enclosing `impl` block's `Self` type cannot be rendered to a stable
structural label (e.g. a complex const-generic argument) SHALL NOT publish a positional fallback as
identity: the system SHALL fail loud with a constitution error (exit 2) rather than risk two
distinct unrenderable sites silently sharing one label. This holds under the subtree opt-in exactly
as it already does for the default (seam-only) scope.

#### Scenario: A submodule's returned impl Trait the seam scope misses reacts under the opt-in

- **WHEN** a boundary anchored at `crate` opts into subtree scope, and a submodule `crate::net` declares `pub fn make() -> impl crate::Port`
- **THEN** the system emits a violation identifying that returned shape, attributed to `crate::net` — the same case the default scope (seam-only) does not observe

#### Scenario: The anchor's own seam finding is byte-identical under the opt-in

- **WHEN** the anchored module itself declares a returned `impl Trait` and a submodule declares another, and the boundary opts into subtree scope
- **THEN** the system emits a finding for the anchor's own returned shape byte-identical to the default-scope finding, plus a distinct finding for the submodule's — so enabling the opt-in adds the deeper finding without re-identifying the seam one

#### Scenario: The subtree is bounded by the anchor, not the whole crate

- **WHEN** a boundary anchored at `crate::a` opts into subtree scope, `crate::a::b` returns an `impl Trait`, and a sibling `crate::c` also does
- **THEN** the system reacts to the one under `crate::a` (including `crate::a::b`) and not to `crate::c` — the subtree is rooted at the anchor

#### Scenario: A cfg-gated fileless submodule is tolerated; a non-cfg missing file is a scan error

- **WHEN** a subtree-scoped boundary descends a `#[cfg]`-gated `mod` with no source file (feature off) alongside a present module, versus a non-`#[cfg]` `mod x;` with no file
- **THEN** the cfg-gated one is tolerated (the present module still reacts) and the non-cfg missing file is a scan error (exit 2), never a silent pass

#### Scenario: A body-nested module is a stated bound

- **WHEN** a subtree-scoped boundary descends a module containing `pub fn outer() { mod inner { pub fn hidden() -> impl crate::Port { .. } } }`
- **THEN** the system does not observe `hidden` — a `mod` inside a fn body is not public API (not reachable as `crate::…`), a stated bound, never a silent claim about it

#### Scenario: The subtree opt-in projects in list output

- **WHEN** a subtree-scoped impl-trait boundary is projected via `list` (text/json/markdown)
- **THEN** the projection surfaces the subtree scope (a `(including submodules)` marker / an `including_submodules: true` field), and a boundary without the opt-in projects byte-identically to before it existed

#### Scenario: An unrenderable self type under subtree scope fails loud rather than publishing a positional label

- **WHEN** a subtree-scoped boundary descends two mutually-exclusive `#[cfg]` branches that each independently declare a same-named type with an unrenderable const-generic self-type argument (e.g. `Arr<{ N + 1 }>` vs `Arr<{ N + 2 }>`), and that type's `impl` block returns an `impl Trait`
- **THEN** the system reports a constitution error (exit 2) rather than publishing an internal positional label as identity — never silently collapsing the two genuinely distinct sites into one reported finding, and never partially succeeding

