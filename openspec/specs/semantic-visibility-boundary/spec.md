# semantic-visibility-boundary Specification

## Purpose

The 渾儀 (semantic) dimension's visibility-hygiene capability: declare in Rust that a governed **module** must not declare any bare-`pub` direct items — a syntactic rule for an internal / implementation-detail layer. The rule is the declared `pub` keyword (`syn::Visibility::Public`) on the module's own direct items, not crate-reachability; `pub(crate)`/`pub(super)`/`pub(in …)`/private are allowed. Observed via the AST (`syn`), it is the cheapest case that earns `syn` — distinct from exposure (which types a `pub` API names) and impl locality.

## Requirements

### Requirement: Visibility boundary declared in Rust

A visibility boundary SHALL be expressed as Rust code and is part of the single source of truth. Mirroring the semantic dimension's other declarations, each dimension owns its own declaration DSL and the boundaries are **composed at the gate**. A `VisibilityBoundary` SHALL name a target crate, a governed **module** path, a human-readable reason, and a severity. The system MUST NOT require TOML, YAML, Markdown, or any generated policy file to declare or run a visibility boundary.

#### Scenario: Boundary declared in Rust

- **WHEN** a developer writes `VisibilityBoundary::in_crate("app").module("crate::internal").must_not_declare_pub().because("internal is an impl detail; nothing here is public API")`
- **THEN** a boundary is held, anchored to `crate::internal` in crate `app`, forbidding bare-`pub` items, with a non-empty reason and a default `enforce` severity, ready to be composed at the gate

### Requirement: Module anchor resolution

For each boundary, the system SHALL resolve the named governed module to a real module in the target crate's source (descending file-based `mod x;` and inline `mod x { … }` alike, as the semantic dimension's existing module-descent does) before evaluating it. If the anchor cannot be resolved — an unknown module path, a target crate absent from the workspace, or a module reachable only through a `#[path]`-remapped or `#[cfg]`-gated-absent ancestor (the dimension's stated coverage bound) — the system SHALL treat this as a **constitution error** (exit 2), failing loud and distinct from a boundary violation (exit 1), so a mistyped or ungovernable anchor is never reported as a visibility violation and never silently passed.

#### Scenario: Anchor resolves to a real module

- **WHEN** a boundary anchors to `crate::internal` and that module exists in the target crate's source
- **THEN** the system observes that module's direct items for comparison

#### Scenario: Unresolvable anchor is a constitution error

- **WHEN** a boundary anchors to a module path that does not exist in the target crate's source
- **THEN** the system emits a constitution error naming the unresolved anchor and exits 2, never exit 0 (no silent pass) and never exit 1

### Requirement: Bare-pub item observation

The system SHALL observe the governed module's **direct** items and react to each whose **declared** visibility is bare `pub` (`syn::Visibility::Public`). The observed items SHALL include `pub fn`, `pub struct`/`enum`/`union`, `pub type`, `pub const`/`static`, `pub trait` (including a `pub trait` alias), `pub extern crate`, **`pub mod`** (a public submodule declaration), and **`pub use`** re-exports — including a `pub use …::*` glob, observed as a raw `Item::Use` node (the re-export *declaration* is bare-`pub`), not via the name-resolver. Items with `pub(crate)`, `pub(super)`, `pub(in …)`, or inherited (private) visibility SHALL NOT react.

The unit of judgment is the **item's own declared visibility**, not its members' visibility nor its effective crate-reachability. This rule is therefore syntactic, with intentional consequences: a bare-`pub` item reacts even when it sits inside a non-`pub` module (the rule is "do not write `pub` here", not "is it crate-reachable"); and the system governs the module's *direct* items only — descendants of a submodule are out of scope (a `pub mod` submodule is itself a reacting item, and may carry its own boundary).

#### Scenario: A pub item is a violation

- **WHEN** the governed module declares `pub fn connect() { … }`
- **THEN** the system emits a violation identifying the offending item (e.g. `pub fn connect`)

#### Scenario: A pub use re-export is a violation

- **WHEN** the governed module declares `pub use crate::db::Handle;`
- **THEN** the system emits a violation identifying the re-export as a public-surface contribution

#### Scenario: A pub use glob is a violation

- **WHEN** the governed module declares `pub use crate::db::*;`
- **THEN** the system emits a violation for the bare-`pub` re-export declaration (observed as a raw `Item::Use`), rather than dropping it as the name-resolver would a glob

#### Scenario: A bare-pub item inside a non-pub module still reacts

- **WHEN** the governed module is itself `pub(crate)` (not crate-public) yet declares `pub fn helper() { … }`
- **THEN** the system emits a violation, because the rule governs the declared `pub` keyword on the item, not whether the item is crate-reachable

#### Scenario: A pub submodule is a violation

- **WHEN** the governed module declares `pub mod sub;`
- **THEN** the system emits a violation identifying the public submodule declaration

#### Scenario: A crate-visible item is clean

- **WHEN** the governed module declares `pub(crate) fn helper() { … }` and `fn private() { … }`, with no bare-`pub` item
- **THEN** the system reports no violation, because `pub(crate)` and private items are not part of the public API surface

### Requirement: Observation bounds and scope

The rule SHALL govern only the declared `pub` keyword on the module's own items; two classes lie outside it:

- **Incidental observation bounds** (stated, never a silent claim): a `pub` item produced by a macro expansion, a module reached through a `#[path]` remap, or a `pub macro` (declarative macros 2.0, which parses as an opaque token item with no readable visibility) is not observed; `#[cfg]`-gated code is observed **as written** (cfg-agnostic).
- **Out of declared scope (not this capability):** public surface that carries no `pub` keyword *in this module* — a `#[macro_export] macro_rules!` (crate-public via attribute), a `#[no_mangle]`/`pub extern` symbol, or a `pub(crate)` item re-exported `pub` *from another module*. Governing attribute-derived public surface is the deferred attribute capability's domain; the static import dimension governs cross-module reachability. This capability makes no claim about them.

Within the observed scope there SHALL be no false negative: a bare-`pub` direct item that *is* observed MUST react.

#### Scenario: A macro-generated pub item is a documented bound

- **WHEN** a `pub` item in the governed module is produced by a macro expansion
- **THEN** the system does not claim to observe it (out of scope, the same nature as the dimension's existing macro bound), rather than silently asserting the module is clean

#### Scenario: A #[macro_export] macro is out of declared scope

- **WHEN** the governed module declares `#[macro_export] macro_rules! m { … }` (crate-public, but carrying no `pub` keyword)
- **THEN** the system does not react (attribute-derived public surface is the deferred attribute capability's domain), and the capability's stated scope is the `pub` keyword, not crate-reachability

#### Scenario: An observed pub item is never silently passed

- **WHEN** the governed module declares a bare-`pub` direct item that the scan observes
- **THEN** the system emits a violation, never exit 0 for that boundary

### Requirement: CI reaction

The system SHALL fold visibility findings into the same exit-code contract as the other dimensions: **exit 0** when no enforce-severity boundary is violated; **exit 1** when one or more enforce-severity boundaries are violated; **exit 2** for a constitution or scan error (an unresolvable anchor, or an unreadable/unparseable source file). A run that evaluates static and any semantic boundaries SHALL aggregate their findings into one report and one outcome, and a constitution error on any boundary SHALL supersede any violation in the same run.

#### Scenario: A clean boundary passes

- **WHEN** the governed module declares no bare-`pub` item
- **THEN** the system reports the boundary satisfied and contributes exit 0

#### Scenario: A visibility violation fails CI

- **WHEN** an enforce-severity visibility boundary is violated
- **THEN** the system prints a report and exits 1

### Requirement: Severity and baseline parity

A visibility boundary SHALL carry a severity (`enforce` by default, or `warn`) with the same meaning as other boundaries: a `warn` violation is reported but does not by itself fail the reaction. Its violations SHALL be gated against the same `Baseline` mechanism, sharing the violation identity `(target, rule, finding)` — where the rule is a fixed string and the finding identifies the offending item — so a project may adopt the boundary on a dirty codebase and gate only on new `pub` items.

#### Scenario: A warn boundary reports without failing

- **WHEN** a `warn`-severity visibility boundary is violated and no enforce-severity boundary is violated
- **THEN** the system reports the violation but the reaction does not fail (exit 0)

#### Scenario: A baselined visibility violation does not fail

- **WHEN** an enforce-severity boundary's only violations are all present in the baseline
- **THEN** the system reports them as accepted and does not fail the reaction

#### Scenario: A new pub item beyond the baseline fails

- **WHEN** an enforce-severity boundary has a bare-`pub` item not present in the baseline
- **THEN** the system fails the reaction (exit 1) for that new item

### Requirement: Human-readable violation report

A visibility violation report SHALL identify the governed module anchor, the rule (that the module must not declare `pub` items), the offending item (the finding), and the human-readable reason supplied with the boundary, and SHALL state that the reaction failed — the same report contract as the other boundaries.

#### Scenario: Report explains the public item

- **WHEN** the module `crate::internal` of crate `app` declares `pub struct Pool` under a visibility boundary
- **THEN** the report names the anchor `crate::internal`, the rule (that it must not declare `pub` items), the finding identifying `pub struct Pool`, the boundary's reason, and indicates CI failed
