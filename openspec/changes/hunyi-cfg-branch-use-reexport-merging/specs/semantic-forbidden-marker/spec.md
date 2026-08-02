## MODIFIED Requirements

### Requirement: Trait matching by leaf identifier

A forbidden entry SHALL match a derive/trait path by **leaf identifier** — so a forbidden `Serialize` or `serde::Serialize` matches `#[derive(Serialize)]`, `#[derive(serde::Serialize)]`, `#[derive(serde_derive::Serialize)]`, and `impl serde::Serialize for …` alike (the derive-macro re-export path and the trait path share a leaf, and the resolver is cross-crate-blind, so leaf is what reliably catches acquisition). The compared leaf is taken from the path **resolved through the acquisition site's `use`-map**, so a locally renamed trait or derive — `use serde::Serialize as Ser; impl Ser for …` or `#[derive(Ser)]` — resolves to its true leaf `Serialize` and reacts (a local rename is observable, so a missed one would be a false negative); a path that does not resolve locally — a bare/prelude name or a cross-crate path — falls back to its **written** leaf, keeping the match cross-crate-blind (the derive-macro-crate path `serde_derive::Serialize` still matches by the leaf `Serialize`). A path-qualified forbidden entry is accepted for the author's clarity but does **not** narrow the match — narrowing by resolved path would silently miss the derive-macro-crate path (`serde_derive::Serialize`), the exact false negative the contract forbids. The cost is a documented false **positive** when two traits share a leaf — reportable, and the safe direction, since a false negative is the one forbidden bug. When the acquisition site's `use`-map resolves the derive/trait name to **more than one** candidate — a mutually-exclusive `#[cfg]`-gated `use` alias for the identical local name — every candidate's leaf SHALL be checked and the match SHALL react if any candidate's leaf matches, never silently keeping only the leaf of whichever declaration was written last (observation cannot know which `#[cfg]` branch is live).

#### Scenario: A derive-macro-crate path still reacts

- **WHEN** a governed type declares `#[derive(serde_derive::Serialize)] pub struct Order;` under a boundary forbidding `serde::Serialize`
- **THEN** the system emits a violation, matched by leaf identifier (the derive-macro path `serde_derive::Serialize` would not resolve to the trait path, but the leaf `Serialize` matches), rather than a false negative

#### Scenario: A same-leaf different trait is a documented false positive

- **WHEN** a governed type derives `rkyv::Serialize` under a boundary forbidding the bare `Serialize`
- **THEN** the system reacts (a leaf match); the user may path-qualify the forbidden entry to tighten — a reportable false positive is accepted, never a silent false negative

#### Scenario: A locally renamed trait or derive reacts by its true leaf

- **WHEN** `crate::domain::order` declares `use serde::Serialize as Ser; #[derive(Ser)] pub struct Order;` (or a hand impl `impl Ser for crate::domain::Order`) under a boundary forbidding `serde::Serialize` on `crate::domain`
- **THEN** the system resolves `Ser` through the module's `use`-map to `serde::Serialize` and reacts by the leaf `Serialize` (the finding renders the written spelling, `derive Ser on crate::domain::order::Order`), rather than silently passing the rename

#### Scenario: Two mutually-exclusive cfg-gated use aliases for a derive or trait name both react

- **WHEN** a governed type's module declares `#[cfg(unix)] use bad::Marker as M; #[cfg(not(unix))] use good::NotBad as M;` and derives `#[derive(M)]` (or an impl site declares the identical alias collision and writes `impl M for <the type>`), under a boundary forbidding `bad::Marker`, in either declaration order
- **THEN** the system emits a violation, regardless of which `use` line is written first — the verdict never depends on source order

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

#### Scenario: A type-alias landing reached through a mutually-exclusive cfg-gated use alias reacts

- **WHEN** an impl site declares `#[cfg(unix)] use crate::domain::Order as Y; #[cfg(not(unix))] use crate::domain::NotOrder as Y; type X = Y; impl serde::Serialize for X {}`, where `Order` is a subtree-defined type and `NotOrder` is not defined anywhere, under a boundary forbidding `serde::Serialize` on the subtree, in either declaration order
- **THEN** the system emits a violation, regardless of which `use` line is written first — every landing candidate for `X` is checked against the defined/under-subtree gate, so the genuinely governed candidate (`Order`) is never dropped in favor of the undefined one (`NotOrder`) merely because it was declared first
