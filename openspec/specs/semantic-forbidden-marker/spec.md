# semantic-forbidden-marker Specification

## Purpose

The 渾儀 (semantic) dimension's forbidden-marker capability: types **defined in a governed module subtree** must not acquire a forbidden trait — observed as a `#[derive(T)]` on the type or a hand `impl T for X` (anywhere in the crate) whose self-type resolves to a definition under the subtree. It delivers the "this layer is not `T`-able" intent (both idiomatic acquisition forms), the forbidden-marker complement to exposure, impl-locality, and visibility. Matching is by leaf identifier (no false negative across the derive-macro/trait path split); the forbidden-attribute slice stays deferred.

## Requirements

### Requirement: Forbidden-marker boundary declared in Rust

A forbidden-marker boundary SHALL be expressed as Rust code and is part of the single source of truth. A `ForbiddenMarkerBoundary` SHALL name a target crate, a governed **module subtree** (a module-path prefix), a forbidden-trait set (one or more trait names/paths), a human-readable reason, and a severity. The system MUST NOT require TOML, YAML, Markdown, or any generated policy file.

#### Scenario: Boundary declared in Rust

- **WHEN** a developer writes `ForbiddenMarkerBoundary::in_crate("app").module("crate::domain").must_not_acquire("serde::Serialize").because("domain types are not serializable")`
- **THEN** a boundary is held, governing the subtree `crate::domain`, forbidding acquisition of `serde::Serialize`, with a non-empty reason and a default `enforce` severity

### Requirement: Subtree scope by type definition

A type SHALL be governed by the boundary iff its **definition** is under the anchor's module-subtree prefix (`::`-delimited containment, sibling-safe — `crate::domain` covers `crate::domain::order` but not `crate::domainx`). The governed set therefore spans the whole subtree, not only the anchor module's direct items.

#### Scenario: A type in a submodule of the subtree is governed

- **WHEN** the boundary governs `crate::domain` and `crate::domain::order` defines `struct Order`
- **THEN** `Order` is governed (a type defined anywhere under the subtree counts)

#### Scenario: A prefix-colliding sibling is not governed

- **WHEN** the boundary governs `crate::domain` and a type is defined in `crate::domainx`
- **THEN** that type is not governed (`::`-delimited containment does not treat the sibling as under the subtree)

### Requirement: Both acquisition forms react

The system SHALL react when a governed type acquires a forbidden trait by **either** form: a `#[derive(T)]` on the type's declaration, **or** an `impl T for X` block anywhere in the crate whose self-type `X` resolves to a definition under the subtree. Covering both is required — a derive-only or impl-only rule would silently pass the other idiomatic form. A `#[cfg_attr(<pred>, derive(T))]` SHALL be read (the nested derive, cfg-agnostic), including a **nested** `#[cfg_attr(a, cfg_attr(b, derive(T)))]`.

#### Scenario: A forbidden derive on a subtree type reacts

- **WHEN** `crate::domain::order` declares `#[derive(serde::Serialize)] pub struct Order;` under a boundary forbidding `serde::Serialize` on `crate::domain`
- **THEN** the system emits a violation identifying `derive serde::Serialize on crate::domain::order::Order` (the finding uses the type's canonical path, so two same-named types stay distinct)

#### Scenario: A forbidden hand-impl for a subtree type reacts

- **WHEN** `crate::wire` declares `impl serde::Serialize for crate::domain::Order { … }` (a hand impl, no derive) under a boundary forbidding `serde::Serialize` on `crate::domain`
- **THEN** the system emits a violation identifying `impl serde::Serialize for crate::domain::Order in crate::wire` (the impl form names the impl-site module), because `Order`'s definition is under the subtree — even though the impl is written outside it

#### Scenario: A hand-impl through a re-export or type-alias spelling reacts

- **WHEN** `crate::wire` re-exports the governed type (`pub use crate::domain::Order;`) and declares `impl serde::Serialize for crate::wire::Order { … }`, or a `type Bar = crate::domain::Order;` alias is written `impl serde::Serialize for Bar`, under a boundary forbidding `serde::Serialize` on `crate::domain`
- **THEN** the system follows the re-export and type-alias closures to the definition `crate::domain::Order` (a re-export/alias denotes the same type — to coherence the marker lands on the definition) and reacts, identifying the impl by its written self-type spelling and impl-site module (`impl serde::Serialize for crate::wire::Order in crate::wire`), rather than silently passing the facade/alias spelling

#### Scenario: A cfg_attr-wrapped derive reacts

- **WHEN** a governed type declares `#[cfg_attr(feature = "serde", derive(serde::Serialize))] pub struct Order;` under a boundary forbidding `serde::Serialize`
- **THEN** the system emits a violation (the nested derive is read, cfg-agnostic), rather than silently passing the optional-serde shape

#### Scenario: A nested cfg_attr derive reacts

- **WHEN** a governed type declares `#[cfg_attr(all(), cfg_attr(all(), derive(serde::Serialize)))] pub struct Order;`
- **THEN** the system recurses into the nested `cfg_attr` and emits a violation, rather than silently dropping the derive

#### Scenario: A non-forbidden trait is clean

- **WHEN** a governed type derives or impls only traits not in the forbidden set
- **THEN** the system reports no violation

### Requirement: Trait matching by leaf identifier

A forbidden entry SHALL match a derive/trait path by **leaf identifier** — so a forbidden `Serialize` or `serde::Serialize` matches `#[derive(Serialize)]`, `#[derive(serde::Serialize)]`, `#[derive(serde_derive::Serialize)]`, and `impl serde::Serialize for …` alike (the derive-macro re-export path and the trait path share a leaf, and the resolver is cross-crate-blind, so leaf is what reliably catches acquisition). The compared leaf is taken from the path **resolved through the acquisition site's `use`-map**, so a locally renamed trait or derive — `use serde::Serialize as Ser; impl Ser for …` or `#[derive(Ser)]` — resolves to its true leaf `Serialize` and reacts (a local rename is observable, so a missed one would be a false negative); a path that does not resolve locally — a bare/prelude name or a cross-crate path — falls back to its **written** leaf, keeping the match cross-crate-blind (the derive-macro-crate path `serde_derive::Serialize` still matches by the leaf `Serialize`). A path-qualified forbidden entry is accepted for the author's clarity but does **not** narrow the match — narrowing by resolved path would silently miss the derive-macro-crate path (`serde_derive::Serialize`), the exact false negative the contract forbids. The cost is a documented false **positive** when two traits share a leaf — reportable, and the safe direction, since a false negative is the one forbidden bug.

#### Scenario: A derive-macro-crate path still reacts

- **WHEN** a governed type declares `#[derive(serde_derive::Serialize)] pub struct Order;` under a boundary forbidding `serde::Serialize`
- **THEN** the system emits a violation, matched by leaf identifier (the derive-macro path `serde_derive::Serialize` would not resolve to the trait path, but the leaf `Serialize` matches), rather than a false negative

#### Scenario: A same-leaf different trait is a documented false positive

- **WHEN** a governed type derives `rkyv::Serialize` under a boundary forbidding the bare `Serialize`
- **THEN** the system reacts (a leaf match); the user may path-qualify the forbidden entry to tighten — a reportable false positive is accepted, never a silent false negative

#### Scenario: A locally renamed trait or derive reacts by its true leaf

- **WHEN** `crate::domain::order` declares `use serde::Serialize as Ser; #[derive(Ser)] pub struct Order;` (or a hand impl `impl Ser for crate::domain::Order`) under a boundary forbidding `serde::Serialize` on `crate::domain`
- **THEN** the system resolves `Ser` through the module's `use`-map to `serde::Serialize` and reacts by the leaf `Serialize` (the finding renders the written spelling, `derive Ser on crate::domain::order::Order`), rather than silently passing the rename

### Requirement: Anchor resolution and observation bounds

If the boundary's target crate is absent from the workspace, the system SHALL treat it as a constitution error (exit 2). An acquisition the syntactic scan cannot observe — a derive/impl produced by a macro, a module reached only through a `cfg_attr`-wrapped `#[path]` remap (an **unconditional** `#[path = "…"]` module IS followed and observed), or a hand-impl whose self-type cannot be resolved to a subtree definition (a glob/external/complex-generic self-type) — is OUT OF SCOPE, a stated coverage bound, not a claimed reaction; `#[cfg]`-gated code is observed as written. A `#[derive(...)]` whose arguments fail to parse SHALL be a scan error (exit 2), never a silent skip. Within the observed scope there SHALL be no false negative.

#### Scenario: An unresolvable hand-impl self-type is a documented bound

- **WHEN** a hand-impl's self-type is brought in by a glob import (`use crate::domain::*; impl serde::Serialize for Order`) so the scan cannot resolve `Order` to its definition
- **THEN** the system does not claim to observe it (a stated coverage bound), rather than silently asserting cleanliness — the co-located, `use`-imported, re-export-spelled, and type-alias cases (the common ones) do resolve and react

#### Scenario: A blanket impl's own generic parameter is never resolved through a same-named alias

- **WHEN** a module declares a blanket `impl<T> Marker for T {}` and ALSO declares an unrelated `use <some path> as T;` naming a real subtree-defined type
- **THEN** the system does not react — `T` in the impl header is the impl's own declared generic type parameter, not a nominal self-type, so it is never resolved through the module's same-named `use ... as T` alias merely because both share the identifier `T`; the source never writes an impl for the aliased type at all

#### Scenario: The shadow holds through a projection off the impl's own generic parameter

- **WHEN** a module declares a blanket `impl<T> Marker for T::Assoc {}` (a projection off the impl's own parameter, never a nominal type) and ALSO declares an unrelated `use <some path> as T;` naming a real subtree-defined type
- **THEN** the system does not react — the shadow applies to the self type's LEADING segment regardless of how many further segments follow (`T::Assoc`, not only the bare `T` form), so it is never resolved through the alias merely because the projection's head shares the identifier `T`

#### Scenario: The shadow holds through a qualified-path projection dependent on the impl's own generic parameter

- **WHEN** a module declares an impl whose self type is a QUALIFIED path dependent on the impl's own generic parameter (`impl<T: HasItem> Marker<T> for <T>::Item {}`) and ALSO declares an unrelated `use <some path> as Item;` naming a real subtree-defined type
- **THEN** the system does not react — a qualified-path self type is never a placeable nominal path (its own dependent type lives outside the path's segments entirely, so no bare-segment shadow check alone can recognize it), so it is dropped before any resolution is attempted, never resolved through the alias merely because the projection's trailing segment shares the identifier `Item`

### Requirement: CI reaction, severity, and baseline parity

The system SHALL fold forbidden-marker findings into the same exit-code contract as the other dimensions (0 clean / 1 enforce violation / 2 constitution or scan error) and aggregate them with the other boundaries. A boundary SHALL carry a severity (`enforce` default, or `warn`, which reports without failing), and its violations SHALL be gated against the same `Baseline` (identity `(target, rule, finding_key)`, the rule a fixed string), so a project may adopt the boundary on a dirty codebase and gate only on new acquisitions.

#### Scenario: A warn boundary reports without failing

- **WHEN** a `warn`-severity forbidden-marker boundary is violated and no enforce-severity boundary is violated
- **THEN** the system reports the violation but the reaction does not fail (exit 0)

#### Scenario: A new acquisition beyond the baseline fails

- **WHEN** an enforce-severity boundary has an acquisition not present in the baseline
- **THEN** the system fails the reaction (exit 1) for that new acquisition

### Requirement: Human-readable violation report

A forbidden-marker violation report SHALL identify the governed subtree anchor, the rule (that the subtree's types must not acquire the forbidden trait), the offending acquisition and the type (the finding — `derive <T> on <Type>` or `impl <T> for <Type> in <module>`, the impl form naming the impl-site module), and the human-readable reason, and SHALL state that the reaction failed — the same report contract as the other boundaries.

#### Scenario: Report explains the acquisition

- **WHEN** `crate::domain::order` declares `#[derive(serde::Serialize)] pub struct Order;` under a boundary forbidding `serde::Serialize` on `crate::domain`
- **THEN** the report names the anchor `crate::domain`, the rule "must not acquire trait", the finding `derive serde::Serialize on crate::domain::order::Order` (canonical-path-keyed), the boundary's reason, and indicates CI failed
