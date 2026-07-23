## ADDED Requirements

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
