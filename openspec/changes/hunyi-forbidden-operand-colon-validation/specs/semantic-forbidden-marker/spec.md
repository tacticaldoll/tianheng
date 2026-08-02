## MODIFIED Requirements

### Requirement: Trait matching by leaf identifier

A forbidden entry SHALL match a derive/trait path by **leaf identifier** — so a forbidden `Serialize` or `serde::Serialize` matches `#[derive(Serialize)]`, `#[derive(serde::Serialize)]`, `#[derive(serde_derive::Serialize)]`, and `impl serde::Serialize for …` alike (the derive-macro re-export path and the trait path share a leaf, and the resolver is cross-crate-blind, so leaf is what reliably catches acquisition). The compared leaf is taken from the path **resolved through the acquisition site's `use`-map**, so a locally renamed trait or derive — `use serde::Serialize as Ser; impl Ser for …` or `#[derive(Ser)]` — resolves to its true leaf `Serialize` and reacts (a local rename is observable, so a missed one would be a false negative); a path that does not resolve locally — a bare/prelude name or a cross-crate path — falls back to its **written** leaf, keeping the match cross-crate-blind (the derive-macro-crate path `serde_derive::Serialize` still matches by the leaf `Serialize`). A path-qualified forbidden entry is accepted for the author's clarity but does **not** narrow the match — narrowing by resolved path would silently miss the derive-macro-crate path (`serde_derive::Serialize`), the exact false negative the contract forbids. The cost is a documented false **positive** when two traits share a leaf — reportable, and the safe direction, since a false negative is the one forbidden bug. When the acquisition site's `use`-map resolves the derive/trait name to **more than one** candidate — a mutually-exclusive `#[cfg]`-gated `use` alias for the identical local name — every candidate's leaf SHALL be checked and the match SHALL react if any candidate's leaf matches, never silently keeping only the leaf of whichever declaration was written last (observation cannot know which `#[cfg]` branch is live). A forbidden entry whose **leaf itself would be empty** — a trailing `::` (`"serde::"`), a doubled `::`, or the empty string — SHALL be rejected as a constitution error rather than silently compared: leaf-identifier matching is immune to a *leading* `::` (`leaf_of("::serde::Serialize")` is still the real leaf `Serialize`), but not to a *trailing* one, since no real identifier is ever empty and such an entry could therefore never match anything, in the same silent-pass class signature-coupling's own forbidden-operand validation closes for its own (full-path) matching mechanism.

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

#### Scenario: A trailing-`::` forbidden entry is a constitution error

- **WHEN** a boundary declares `must_not_acquire("serde::")` (trailing `::`)
- **THEN** the system reports a constitution error (exit 2), rather than silently reporting the boundary satisfied — the computed leaf would be empty and could never match a real identifier

#### Scenario: A leading-`::` forbidden entry is rejected for DSL-wide consistency, not because it would mismatch

- **WHEN** a boundary declares `must_not_acquire("::serde::Serialize")` (leading `::`) against a type deriving `#[derive(serde::Serialize)]`
- **THEN** the system reports a constitution error (exit 2) — leaf-identifier matching alone would tolerate a leading `::` (`leaf_of` still yields `Serialize`), but the operand is rejected anyway, for consistency with every other forbidden/allowed-operand-shaped DSL method in this family, none of which assigns the leading-`::` spelling a meaning distinct from the bare form
