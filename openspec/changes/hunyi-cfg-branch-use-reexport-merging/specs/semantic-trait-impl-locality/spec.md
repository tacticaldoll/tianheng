## MODIFIED Requirements

### Requirement: Trait-path resolution scope and no false negative

The system SHALL resolve the trait named at an impl site to a canonical path using the shared 渾儀 resolver: the file's in-scope `use` declarations (including renamed imports), `crate::`/`self`/`super`-relative paths (including a `use` target that is itself `self`/`super`-relative), a **bare or relative name resolved against the current module and crate root** (a same-module trait needs no `use`), and **local `pub use` re-export chains** (a trait reached through a facade path matches the anchor). A trait whose resolution would require capabilities beyond this — a glob import (`use …::*`), a macro-generated impl, a `cfg_attr`-wrapped `#[path]` module (an **unconditional** `#[path = "…"]` module is followed and observed), or `#[cfg]` feature evaluation — is OUT OF SCOPE, a stated coverage bound, not a claimed reaction. `#[cfg]`-gated code is observed **as written** (cfg-agnostic), and a `#[cfg]`-gated module whose source file is legitimately absent is skipped, not a scan error. Within the resolved scope there SHALL be no false negative: an impl of the anchored trait whose trait path *is* resolvable and whose location is disallowed MUST react. The system MUST NOT silently pass a disallowed impl it was able to resolve to the anchored trait. When a `use`-map name involved in resolution — on either the boundary's own declared anchor (reached through its re-export facade) or an impl site's written trait path — resolves to **more than one** candidate because of a mutually-exclusive `#[cfg]`-gated `use` alias for the identical local name, every candidate SHALL be checked, and the anchor match SHALL react if any impl-site candidate canonicalizes to any declared-anchor candidate, never silently keeping only the candidate from whichever declaration was written last (observation cannot know which `#[cfg]` branch is live).

#### Scenario: A use-imported trait path resolves and reacts

- **WHEN** a disallowed module declares `use crate::command::Command;` then `impl Command for Foo { … }`
- **THEN** the system resolves the trait to `crate::command::Command`, matches the anchor, and emits a violation

#### Scenario: A renamed trait import resolves and reacts

- **WHEN** a disallowed module declares `use crate::command::Command as Cmd;` then `impl Cmd for Foo { … }`
- **THEN** the system resolves `Cmd` to `crate::command::Command` and emits a violation

#### Scenario: A bare same-module trait name resolves and reacts

- **WHEN** the anchored `trait Command` is defined in the disallowed module `crate::domain`, which also declares `impl Command for Foo { … }` with a bare `Command` and no `use`
- **THEN** the system resolves the bare `Command` against the current module to `crate::domain::Command`, matches the anchor, and emits a violation (never a silent pass)

#### Scenario: A self/super-relative trait import resolves and reacts

- **WHEN** a disallowed module `crate::domain` declares `use super::command::Command;` then `impl Command for Foo { … }`
- **THEN** the system canonicalizes the relative `use` target against the module to `crate::command::Command`, matches the anchor, and emits a violation (never a silent pass)

#### Scenario: A re-exported trait path resolves and reacts

- **WHEN** a disallowed module declares `use crate::facade::Command;` (where `crate::facade` re-exports `crate::command::Command` via `pub use`) then `impl Command for Foo { … }`
- **THEN** the system follows the re-export chain, matches the anchor `crate::command::Command`, and emits a violation rather than silently passing

#### Scenario: A macro-generated impl is a documented coverage bound

- **WHEN** an `impl Command for Foo` is produced by a macro expansion in a disallowed module
- **THEN** the system does not claim to observe it (out of scope, the same nature as the existing macro bound), rather than silently asserting the boundary is clean

#### Scenario: An unconditional #[path]-remapped module is followed and its disallowed impl reacts

- **WHEN** a disallowed impl lives in a module declared `#[path = "…"] mod x;` (an unconditional remap) whose file is located off the conventional path
- **THEN** the system follows the remap to that file and reacts on the impl, attributed to the module's declared path `crate::…::x`, rather than silently asserting the boundary is clean — while a `cfg_attr`-wrapped `#[path]` remains a stated (unfollowed) coverage bound

#### Scenario: Two cfg-siblings declaring the identical name backed by one real file are one finding

- **WHEN** the crate declares the SAME module name under two mutually-exclusive `#[cfg]` arms with no `#[path]` (so both resolve to the identical real file), and that file contains a single disallowed impl whose self-type carries an unrenderable generic argument
- **THEN** the system reports exactly one violation, never two — the two `#[cfg]` arms name the same real, once-compiled file, and must not be treated as two independent scan sites merely because they were declared twice

#### Scenario: A blanket impl's own generic parameter never dedup-collapses with a genuine impl on the aliased type

- **WHEN** a disallowed module declares a blanket `impl<T> Trait for T {}` (`T` is the impl's own generic parameter) alongside an unrelated `use SomeType as T;` naming a real crate-defined type, AND a genuine direct `impl Trait for SomeType {}` in that same module
- **THEN** the system reports TWO distinct violations, not one — the blanket impl's own `T` is never resolved through the same-named alias to produce an owner identical to the direct impl's, which would otherwise let the two impl sites' findings collapse under exact-identity dedup and silently drop one genuine violation

#### Scenario: A cfg-gated module with an absent file is skipped, not a scan error

- **WHEN** the crate declares `#[cfg(feature = "x")] mod optional;` with no `optional.rs` (the feature is off)
- **THEN** the whole-crate walk skips the module (a stated coverage bound) rather than failing the gate with a scan error (exit 2)

#### Scenario: A resolvable disallowed impl is never silently passed

- **WHEN** an impl of the anchored trait is in a disallowed location and its trait path is resolvable by the shared resolver
- **THEN** the system emits a violation, never exit 0 for that boundary

#### Scenario: A mutually-exclusive cfg-gated use alias for the anchored trait's name reacts regardless of order

- **WHEN** a disallowed module declares `#[cfg(unix)] use crate::command::Command as T; #[cfg(not(unix))] use crate::other::Other as T;` then `impl T for Foo { … }`, under a boundary anchored to `crate::command::Command`, in either declaration order
- **THEN** the system emits a violation, regardless of which `use` line is written first — every candidate the impl site's `T` could resolve to is checked against the anchor, so the verdict never depends on source order
