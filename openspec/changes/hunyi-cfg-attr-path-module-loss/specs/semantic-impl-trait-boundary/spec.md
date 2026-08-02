## MODIFIED Requirements

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
**unconditional** `#[path = "…"]` module SHALL be followed and observed; a module reached only
through a `cfg_attr`-wrapped `#[path]` remap SHALL be observed too — an inline body regardless of the
attribute (which has no effect on an inline module's content), and a file module's conventional file
and its `cfg_attr` target both read when they exist on disk, cfg-blind union rather than a skip
bound; a `#[cfg]`-gated module absent when its feature is off SHALL be tolerated; a non-`#[cfg]`
missing module file SHALL be a scan error (exit 2); a symlink module cycle SHALL be a scan error
(exit 2), never a stack overflow. A `mod` declared inside a **function body** SHALL be a stated
bound (not observed) — it is not part of the public module tree, so this rule, which governs the
*public* seam, makes no claim about it, rather than silently asserting cleanliness.

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

#### Scenario: A cfg_attr-wrapped-path submodule's returned impl Trait reacts, whichever candidate file exists

- **WHEN** a subtree-scoped boundary anchored at `crate` descends `#[cfg_attr(any(), path = "never.rs")] pub mod net;` with `net.rs` (the conventional file, present) declaring `pub fn make() -> impl crate::Port` and `never.rs` (the target) absent
- **THEN** the system reads `net.rs` — the file every build actually compiles here — and reacts, attributed to `crate::net`, rather than treating the `cfg_attr` attribute as a bound to skip the submodule outright
