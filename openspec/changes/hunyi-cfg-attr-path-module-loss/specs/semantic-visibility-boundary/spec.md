## MODIFIED Requirements

### Requirement: Module anchor resolution

For each boundary, the system SHALL resolve the named governed module to a real module in the target crate's source (descending file-based `mod x;` and inline `mod x { … }` alike, as the semantic dimension's existing module-descent does) before evaluating it. If the anchor cannot be resolved — an unknown module path, a target crate absent from the workspace, or a `#[cfg]`-gated-absent ancestor with no `cfg_attr`-wrapped `#[path]` target backing it either — the system SHALL treat this as a **constitution error** (exit 2), failing loud and distinct from a boundary violation (exit 1), so a mistyped or ungovernable anchor is never reported as a visibility violation and never silently passed. A module reached only through a `cfg_attr`-wrapped `#[path]` remap IS followed, exactly like an **unconditional** `#[path = "…"]` ancestor already is: its conventional file and its `cfg_attr` target are both read when they exist on disk, cfg-blind union rather than a skip bound — even when no sibling declaration for the same name would otherwise keep the descent alive.

#### Scenario: Anchor resolves to a real module

- **WHEN** a boundary anchors to `crate::internal` and that module exists in the target crate's source
- **THEN** the system observes that module's direct items for comparison

#### Scenario: Unresolvable anchor is a constitution error

- **WHEN** a boundary anchors to a module path that does not exist in the target crate's source
- **THEN** the system emits a constitution error naming the unresolved anchor and exits 2, never exit 0 (no silent pass) and never exit 1

#### Scenario: A cfg_attr-wrapped-path anchor resolves through its own target with no resolving sibling at all

- **WHEN** a boundary anchors to `crate::foo`, declared only as `#[cfg_attr(windows, path = "win.rs")] mod foo;` with no conventional `foo.rs` present, and `win.rs` exists and declares a bare-`pub` item above the boundary's ceiling
- **THEN** the system reads `win.rs` and reacts on the item, rather than reporting a constitution error — the `cfg_attr` target is now followed even with no sibling declaration to keep the branch count non-empty

### Requirement: Observation bounds and scope

The rule SHALL govern only the **declared** visibility keyword on the module's own direct items; the prior bounds hold verbatim, plus one added conservative bound:

- **Incidental observation bounds** (stated, never a silent claim): an item produced by a macro expansion, or a `pub macro` (declarative macros 2.0, which parses as an opaque token item with no readable visibility) is not observed; `#[cfg]`-gated code is observed **as written** (cfg-agnostic). A module reached through a `cfg_attr`-wrapped `#[path]` remap IS observed (its conventional file and its `cfg_attr` target both read when they exist on disk), exactly like an **unconditional** `#[path = "…"]` module already is.
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
