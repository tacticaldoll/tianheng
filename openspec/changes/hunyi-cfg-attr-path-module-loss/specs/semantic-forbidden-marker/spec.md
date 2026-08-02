## MODIFIED Requirements

### Requirement: Anchor resolution and observation bounds

If the boundary's target crate is absent from the workspace, the system SHALL treat it as a constitution error (exit 2). An acquisition the syntactic scan cannot observe — a derive/impl produced by a macro, or a hand-impl whose self-type cannot be resolved to a subtree definition (a glob/external/complex-generic self-type) — is OUT OF SCOPE, a stated coverage bound, not a claimed reaction; `#[cfg]`-gated code is observed as written. A module reached only through a `cfg_attr`-wrapped `#[path]` remap IS followed: an inline body regardless of the attribute (which has no effect on an inline module's content), and a file module's conventional file and its `cfg_attr` target both read when they exist on disk, a cfg-blind union rather than a skip bound — matching the already-followed **unconditional** `#[path = "…"]` form. A `#[derive(...)]` whose arguments fail to parse SHALL be a scan error (exit 2), never a silent skip. Within the observed scope there SHALL be no false negative.

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

#### Scenario: A cfg_attr-wrapped-path module's derive or impl is followed, whichever candidate exists

- **WHEN** a governed module is declared only via `#[cfg_attr(any(), path = "never.rs")] pub mod domain;` with `domain.rs` (the conventional file, present) declaring `#[derive(serde::Serialize)] pub struct Order;` and `never.rs` (the target) absent, under a boundary forbidding `serde::Serialize`
- **THEN** the system reads `domain.rs` — the file every build actually compiles here — and reacts, never treating the `cfg_attr` attribute as a bound to skip the module outright
