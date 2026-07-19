# semantic-async-exposure-boundary Specification

## Purpose
The 渾儀 (semantic) capability that governs **implicit-existential exposure**: a module's public API
must not declare an `async fn`. It is the implicit-future complement of `semantic-impl-trait-boundary`
— an `async fn` leaks a compiler-inserted `impl Future` (and commits the seam's contract to async),
so where impl-trait forbids a *written* `-> impl Future`, this forbids the `async fn` sugar,
observed from the pure local AST signal `syn::Signature.asyncness`. Shape-only, over the same public
surface impl-trait governs (free fns, inherent methods, trait method declarations; trait-impl
methods and private items excluded). Its finding is an owner-qualified item identity so distinct
async fns never collide under the baseline.

## Requirements

### Requirement: Async-exposure boundary declared in Rust

An async-exposure boundary SHALL be expressed as Rust code on an `AsyncExposureBoundary`, part of
the single source of truth, declared on the 渾儀 dimension and composed with the other dimensions
at the gate. It SHALL name a target crate and a module anchor via
`AsyncExposureBoundary::in_crate("…").module("…").must_not_expose_async_fn()`, a human-readable
reason, and a severity. The system MUST NOT require TOML, YAML, Markdown, or any generated policy
file to declare or run the boundary.

#### Scenario: Async-exposure boundary declared in Rust

- **WHEN** a developer writes `AsyncExposureBoundary::in_crate("core").module("crate::core").must_not_expose_async_fn().because("the core seam is synchronous; async lives at the adapter edges")`
- **THEN** an async-exposure boundary is held, targeting `crate::core`, with a non-empty reason and a default `enforce` severity, ready to be composed with the semantic dimension at the gate

### Requirement: A public async fn is a violation

The system SHALL emit a violation for each `async fn` declared in the governed **scope**'s public
surface — a public free function, a public inherent method, or a public trait method declaration —
observed from `syn::Signature.asyncness`. It SHALL exclude trait-*impl* methods (their `asyncness`
is dictated by the trait declaration, governed there) and private functions/methods. The reaction
is shape-only: any public `async fn` in scope reacts.

The governed **scope** SHALL be, by default, the anchored module's **own** items only (the declared
seam — "this declared seam is synchronous"). When the boundary opts into subtree scope
(`including_submodules`, the requirement below), the scope SHALL instead be the anchored module's
whole subtree. The default (seam-only) reaction SHALL be byte-identical to before the opt-in existed —
a boundary that does not opt in observes exactly the anchored module's own items.

#### Scenario: A public async free function is flagged

- **WHEN** the governed module declares `pub async fn connect() -> u8 { … }`
- **THEN** the system emits a violation identifying that async fn

#### Scenario: A public inherent async method is flagged

- **WHEN** the governed module declares `impl Service { pub async fn run(&self) { … } }`
- **THEN** the system emits a violation identifying `Service`'s async method `run`

#### Scenario: A public trait async method declaration is flagged

- **WHEN** the governed module declares `pub trait Port { async fn fetch(&self) -> u8; }`
- **THEN** the system emits a violation identifying the trait `Port`'s async method `fetch`, because the trait declares the async contract

#### Scenario: A trait-impl async method is not double-counted

- **WHEN** an `impl Port for Service` block declares `async fn fetch(&self) -> u8 { … }` and the trait `Port` is where that async contract is declared
- **THEN** the system does not additionally flag the trait-impl method, mirroring impl-trait's handling of trait impls

#### Scenario: A private async fn and a non-async fn are not flagged

- **WHEN** the governed module declares `async fn helper() {}` (private) and `pub fn ready() -> u8 { 0 }` (non-async)
- **THEN** the system reports no violation for either — one is not public API, the other is not async

#### Scenario: The default scope is the anchored module's own items only

- **WHEN** a boundary anchored at `crate` does NOT opt into subtree scope, and a public `async fn` is declared in a submodule `crate::net` (not in the crate root)
- **THEN** the system reports no violation — the default scope is the anchored module's own seam, so a submodule's async fn is out of scope (governing it requires the subtree opt-in)

### Requirement: Subtree scope opt-in

An async-exposure boundary SHALL support an opt-in **subtree scope** via `including_submodules()` on
the rule draft, defaulting OFF (a boundary without it governs the anchored module's own seam, per the
requirement above, byte-identically in reaction and projection). When set, the reaction SHALL descend
the anchored module's **whole subtree** — every descendant module, file-based `mod x;` and inline
`mod x { … }` alike — and SHALL emit a violation for every public `async fn` at or below the anchor,
each attributed to its enclosing module. Anchoring at `crate` with the opt-in SHALL govern the whole
crate. Within the observed subtree there SHALL be no false negative: a public `async fn` in any
descendant module MUST react.

The violation `target` SHALL remain the boundary's anchored module (not the deeper enclosing
module), so a finding's identity `(target, rule, finding_key)` is stable whether or not the opt-in is
set — enabling it adds only new, deeper findings and never re-identifies a seam finding (baseline
stability). A seam finding (one in the anchored module itself) under the opt-in SHALL be
byte-identical to the same finding under the default scope.

The subtree walk SHALL inherit the crate-scan family's guards so it never silently under-reacts: a
`#[path]`-remapped module SHALL be a stated coverage bound (not observed); a `#[cfg]`-gated module
absent when its feature is off SHALL be tolerated; a non-`#[cfg]` missing module file SHALL be a scan
error (exit 2); a symlink module cycle SHALL be a scan error (exit 2), never a stack overflow. A
`mod` declared inside a **function body** SHALL be a stated bound (not observed) — it is not part of
the public module tree, so this rule, which governs the *public* seam, makes no claim about it,
rather than silently asserting cleanliness.

The subtree opt-in SHALL project through the `list` text/JSON/markdown output only when set, so a
bare boundary's projection stays byte-identical.

#### Scenario: A submodule async fn the seam scope misses reacts under the opt-in

- **WHEN** a boundary anchored at `crate` opts into subtree scope, and a public `async fn` is declared in a submodule `crate::net`
- **THEN** the system emits a violation identifying that async fn, attributed to `crate::net` — the same case the default scope (seam-only) does not observe

#### Scenario: The anchor's own seam finding is byte-identical under the opt-in

- **WHEN** the anchored module itself declares a public `async fn` and a submodule declares another, and the boundary opts into subtree scope
- **THEN** the system emits a finding for the anchor's own async fn byte-identical to the default-scope finding, plus a distinct finding for the submodule's — so enabling the opt-in adds the deeper finding without re-identifying the seam one

#### Scenario: The subtree is bounded by the anchor, not the whole crate

- **WHEN** a boundary anchored at `crate::a` opts into subtree scope, `crate::a::b` declares a public `async fn`, and a sibling `crate::c` also declares one
- **THEN** the system reacts to the one under `crate::a` (including `crate::a::b`) and not to `crate::c` — the subtree is rooted at the anchor

#### Scenario: A cfg-gated fileless submodule is tolerated; a non-cfg missing file is a scan error

- **WHEN** a subtree-scoped boundary descends a `#[cfg]`-gated `mod` with no source file (feature off) alongside a present module, versus a non-`#[cfg]` `mod x;` with no file
- **THEN** the cfg-gated one is tolerated (the present module still reacts) and the non-cfg missing file is a scan error (exit 2), never a silent pass

#### Scenario: A body-nested module is a stated bound

- **WHEN** a subtree-scoped boundary descends a module containing `pub fn outer() { mod inner { pub async fn hidden() {} } }`
- **THEN** the system does not observe `hidden` — a `mod` inside a fn body is not public API (not reachable as `crate::…`), a stated bound, never a silent claim about it

#### Scenario: The subtree opt-in projects in list output

- **WHEN** a subtree-scoped async-exposure boundary is projected via `list` (text/json/markdown)
- **THEN** the projection surfaces the subtree scope (a `(including submodules)` marker / an `including_submodules: true` field), and a boundary without the opt-in projects byte-identically to before it existed

### Requirement: The finding is an owner-qualified item identity

The finding SHALL be an owner-qualified item identity — the owner kind, the owner path or type, the
function name, and a stable render of the parameters and return type — NOT a bare function name and
NOT a future type-shape. Two distinct public async fns SHALL yield two distinct findings, so that
baselining one never masks the other under the `(target, rule, finding_key)` identity. The owner render
SHALL preserve generic arguments (`Foo<u8>` and `Foo<u16>` stay distinct). The rendered return type
serves readability and collision-avoidance, not to represent the implicit future.

The system MAY render an unrenderable owner or parameter type as `_` — a **stated
render-granularity bound**, the same class the `dyn` / `impl Trait` shape renderers carry: two
DISTINCT but unrenderable-typed items with an identical name and signature would then share a
finding. This is unreachable for a valid inherent `impl` (its self type is always a concrete nominal
path, which renders), so it is stated rather than guarded — never a silent claim that such a case is
distinguished.

#### Scenario: Two same-named async methods across two impls yield distinct findings

- **WHEN** the governed module declares `impl A { pub async fn run(&self) {} }` and `impl B { pub async fn run(&self) {} }`
- **THEN** the system emits two distinct violations whose findings are owner-qualified (naming `A` and `B` respectively), never a single finding that would let a baselined `run` mask the other

#### Scenario: Two same-named async methods across two traits yield distinct findings

- **WHEN** the governed module declares `pub trait T { async fn run(&self); }` and `pub trait U { async fn run(&self); }`
- **THEN** the system emits two distinct violations whose findings name the trait owners `T` and `U` respectively

### Requirement: CI reaction, severity, baseline, and projection parity

The async-exposure boundary SHALL share the 渾儀 reaction contract with the sibling boundaries:
findings fold into the same aggregated report and exit-code outcome (**0** clean, **1** enforce
violation, **2** constitution/scan error such as an unresolvable crate or module); the boundary
carries a severity (`enforce` default, or `warn`) and is gated against the same `Baseline` under
the shared violation identity `(target, rule, finding_key)`; and the rule projects through the `list`
text/JSON/markdown projection with its own boundary section. The implementation SHALL keep the
`syn` dependency quarantined in `hunyi` (no new dependency) and SHALL NOT change existing rules.

#### Scenario: An async-exposure violation fails CI

- **WHEN** an enforce-severity async-exposure boundary is violated
- **THEN** the system prints a report naming the target module, the rule, the offending owner-qualified async fn, and the reason, and exits 1

#### Scenario: An unresolvable target module is a constitution error

- **WHEN** an async-exposure boundary anchors to a crate or module not present in the workspace
- **THEN** the system emits a constitution/scan error and exits 2, never exit 0 and never exit 1

#### Scenario: Severity and baseline behave as for the sibling rules

- **WHEN** a `warn`-severity async-exposure boundary is violated and no enforce boundary is, or an enforce boundary's only violations are all in the baseline
- **THEN** the reaction does not fail (exit 0); and an async-exposure violation not present in the baseline fails the reaction (exit 1)

#### Scenario: The rule projects in list output

- **WHEN** the constitution is projected via `list` (text/json/markdown)
- **THEN** the async-exposure boundary appears with its target, module, rule, severity, and reason — through its own projection section, parallel to the sibling boundaries
