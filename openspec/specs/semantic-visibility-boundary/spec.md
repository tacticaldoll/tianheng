# semantic-visibility-boundary Specification

## Purpose

The 渾儀 (semantic) dimension's visibility-hygiene capability: declare in Rust that a governed **module**'s direct items carry no more visibility than a declared **ceiling** — `Crate` (react on bare `pub`), `Super` (also on `pub(crate)`), or `Module` (any `pub`-family keyword). The rule is the item's **declared** visibility keyword on the module's own direct items, not crate-reachability. `must_not_declare_pub` is the `Crate`-ceiling sugar. Observed via the AST (`syn`), it is the cheapest case that earns `syn` — distinct from exposure (which types a `pub` API names) and impl locality.

## Requirements

### Requirement: Visibility boundary declared in Rust

A visibility boundary SHALL be expressed as Rust code and is part of the single source of truth. A `VisibilityBoundary` SHALL name a target crate, a governed **module** path, a human-readable reason, and a severity, and SHALL declare a **maximum-visibility ceiling** — one of `Crate`, `Super`, or `Module` — via `max_visibility(ceiling)`. `must_not_declare_pub()` SHALL be preserved as sugar for `max_visibility(Crate)`, byte-identical in behavior, rule string, and findings to its prior form (so existing baselines never churn). The system MUST NOT require TOML, YAML, Markdown, or any generated policy file to declare or run a visibility boundary.

#### Scenario: Boundary declared in Rust with a ceiling

- **WHEN** a developer writes `VisibilityBoundary::in_crate("app").module("crate::deep").max_visibility(VisibilityCeiling::Super).because("this submodule is sealed to its parent")`
- **THEN** a boundary is held, anchored to `crate::deep` in crate `app`, forbidding any direct item declared more visible than `pub(super)`, with a non-empty reason and a default `enforce` severity

#### Scenario: must_not_declare_pub is the Crate-ceiling sugar

- **WHEN** a developer writes `VisibilityBoundary::in_crate("app").module("crate::internal").must_not_declare_pub().because("…")`
- **THEN** the boundary behaves exactly as `max_visibility(VisibilityCeiling::Crate)` — reacting on bare `pub` and allowing `pub(crate)` and below — with the same rule string and findings as before this change

### Requirement: Module anchor resolution

For each boundary, the system SHALL resolve the named governed module to a real module in the target crate's source (descending file-based `mod x;` and inline `mod x { … }` alike, as the semantic dimension's existing module-descent does) before evaluating it. If the anchor cannot be resolved — an unknown module path, a target crate absent from the workspace, or a module reachable only through a `#[path]`-remapped or `#[cfg]`-gated-absent ancestor (the dimension's stated coverage bound) — the system SHALL treat this as a **constitution error** (exit 2), failing loud and distinct from a boundary violation (exit 1), so a mistyped or ungovernable anchor is never reported as a visibility violation and never silently passed.

#### Scenario: Anchor resolves to a real module

- **WHEN** a boundary anchors to `crate::internal` and that module exists in the target crate's source
- **THEN** the system observes that module's direct items for comparison

#### Scenario: Unresolvable anchor is a constitution error

- **WHEN** a boundary anchors to a module path that does not exist in the target crate's source
- **THEN** the system emits a constitution error naming the unresolved anchor and exits 2, never exit 0 (no silent pass) and never exit 1

### Requirement: Bare-pub item observation

The system SHALL observe the governed module's **direct** items and react to each whose **declared** visibility rank is **strictly above** the boundary's ceiling. Visibility ranks, most to least visible, are: `pub` (Public) > `pub(crate)` (Crate) > `pub(super)` (Super) > inherited-private / `pub(self)` (Module). A `pub(in P)` form SHALL rank by its path matched **whole and single-segment**: exactly `crate` → Crate, exactly `super` → Super, exactly `self` → Module. Any **multi-segment or otherwise-unrecognized** `pub(in P)` path SHALL rank as **Crate, a conservative upper bound** — notably `pub(in super::super)`, which is legal Rust reaching the grandparent's whole subtree (broader than `pub(super)`) and therefore MUST NOT be ranked `Super`. A `pub(in P)` path is always an ancestor module within the crate, so such an item is at most crate-visible; ranking every unrecognized restricted form Crate never under-reacts (no false negative). The observed item kinds SHALL be exactly those of the prior rule — `fn`, `struct`/`enum`/`union`, `type`, `const`/`static`, `trait` (incl. alias), `extern crate`, **`mod`** (a submodule declaration), and **`use`** re-exports incl. a `use …::*` glob observed as a raw `Item::Use` node. An item at or below the ceiling SHALL NOT react.

The unit of judgment is the **item's own declared visibility**, not its members' visibility nor its effective crate-reachability. This rule is therefore syntactic, with intentional consequences: an item reacts on its declared keyword even inside a non-`pub` module (the rule is "do not declare above the ceiling here", not "is it crate-reachable"); and the system governs the module's *direct* items only — descendants of a submodule are out of scope (a reacting `mod` submodule may carry its own boundary).

#### Scenario: An item above the ceiling is a violation

- **WHEN** the governed module has ceiling `Crate` and declares `pub fn connect() { … }`
- **THEN** the system emits a violation identifying the offending item and its declared visibility (e.g. `pub fn connect`)

#### Scenario: A Super ceiling reacts on pub(crate)

- **WHEN** the governed module has ceiling `Super` and declares `pub(crate) fn helper() { … }`
- **THEN** the system emits a violation, because `pub(crate)` is more visible than the `Super` ceiling

#### Scenario: A Module ceiling reacts on pub(super)

- **WHEN** the governed module has ceiling `Module` and declares `pub(super) fn helper() { … }`
- **THEN** the system emits a violation, because `pub(super)` is more visible than the `Module` (module-private) ceiling

#### Scenario: A multi-segment pub(in super::super) ranks Crate, not Super

- **WHEN** the governed module has ceiling `Super` and declares `pub(in super::super) fn helper() { … }`
- **THEN** the system reacts, because the multi-segment path ranks `Crate` (the conservative upper bound), which exceeds the `Super` ceiling — never silently passed as if it were `pub(super)`

#### Scenario: A pub use re-export above the ceiling is a violation

- **WHEN** the governed module has ceiling `Crate` and declares `pub use crate::db::Handle;`
- **THEN** the system emits a violation identifying the re-export as a public-surface contribution

#### Scenario: A pub use glob above the ceiling is a violation

- **WHEN** the governed module has ceiling `Crate` and declares `pub use crate::db::*;`
- **THEN** the system emits a violation for the bare-`pub` re-export declaration (observed as a raw `Item::Use`), rather than dropping it as the name-resolver would a glob

#### Scenario: A pub submodule above the ceiling is a violation

- **WHEN** the governed module has ceiling `Crate` and declares `pub mod sub;`
- **THEN** the system emits a violation identifying the public submodule declaration

#### Scenario: An item above the ceiling inside a non-pub module still reacts

- **WHEN** the governed module is itself `pub(crate)` (not crate-public), has ceiling `Crate`, yet declares `pub fn helper() { … }`
- **THEN** the system emits a violation, because the rule governs the declared visibility keyword on the item, not whether the item is crate-reachable

#### Scenario: A pub(in path) form ranks as a conservative crate upper bound

- **WHEN** the governed module has ceiling `Crate` and declares `pub(in crate::a::b) fn helper() { … }`
- **THEN** the system does not react (at most crate-visible, at or below the `Crate` ceiling), never a false negative

#### Scenario: An item at or below the ceiling is clean

- **WHEN** the governed module has ceiling `Crate` and declares `pub(crate) fn helper() { … }` and `fn private() { … }`, with no bare-`pub` item
- **THEN** the system reports no violation, because `pub(crate)` and private items are at or below the ceiling

### Requirement: Observation bounds and scope

The rule SHALL govern only the **declared** visibility keyword on the module's own direct items; the prior bounds hold verbatim, plus one added conservative bound:

- **Incidental observation bounds** (stated, never a silent claim): an item produced by a macro expansion, a module reached through a `#[path]` remap, or a `pub macro` (declarative macros 2.0, which parses as an opaque token item with no readable visibility) is not observed; `#[cfg]`-gated code is observed **as written** (cfg-agnostic).
- **Out of declared scope (not this capability):** public surface that carries no visibility keyword *in this module* — a `#[macro_export] macro_rules!` (crate-public via attribute), a `#[no_mangle]`/`pub extern` symbol, or an item re-exported *from another module*. Governing attribute-derived public surface is the deferred attribute capability's domain; the static import dimension governs cross-module reachability. This capability makes no claim about them.
- **Conservative `pub(in P)` upper bound** (stated, false-negative-safe): a `pub(in <non-canonical in-crate path>)` item ranks as `Crate`. Under a `Super` or `Module` ceiling this MAY over-react when the real path is narrow (effectively private), a loud over-reaction chosen over a silent pass. It never under-reacts.

Within the observed scope there SHALL be no false negative: an item whose declared-visibility rank *is* observed to exceed the ceiling MUST react.

#### Scenario: A macro-generated item is a documented bound

- **WHEN** an item in the governed module is produced by a macro expansion
- **THEN** the system does not claim to observe it (out of scope, the same nature as the dimension's existing macro bound), rather than silently asserting the module is clean

#### Scenario: A #[macro_export] macro is out of declared scope

- **WHEN** the governed module declares `#[macro_export] macro_rules! m { … }` (crate-public, but carrying no visibility keyword)
- **THEN** the system does not react (attribute-derived public surface is the deferred attribute capability's domain), and the capability's stated scope is the declared keyword, not crate-reachability

#### Scenario: A pub(in narrow-path) item may over-react under a tight ceiling

- **WHEN** the governed module has ceiling `Module` and declares `pub(in crate::a) fn helper()` where the item is itself directly in `crate::a` (effectively private)
- **THEN** the system MAY react (the conservative `Crate` rank exceeds the `Module` ceiling), a stated over-reaction bound, never a silent pass

#### Scenario: An observed above-ceiling item is never silently passed

- **WHEN** the governed module declares a direct item whose rank the scan observes to exceed the ceiling
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

A visibility violation report SHALL identify the governed module anchor, the rule (the ceiling the module must not exceed), the offending item and its declared visibility (the finding), and the human-readable reason supplied with the boundary, and SHALL state that the reaction failed — the same report contract as the other boundaries.

#### Scenario: Report explains the offending item

- **WHEN** the module `crate::internal` of crate `app` declares `pub struct Pool` under a `Crate`-ceiling visibility boundary
- **THEN** the report names the anchor `crate::internal`, the rule (that it must not declare `pub` items), the finding identifying `pub struct Pool`, the boundary's reason, and indicates CI failed
