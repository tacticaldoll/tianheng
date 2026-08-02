## MODIFIED Requirements

### Requirement: Bare-pub item observation

The system SHALL observe the governed module's **direct** items and react to each whose **declared** visibility rank is **strictly above** the boundary's ceiling. Visibility ranks, most to least visible, are: `pub` (Public) > `pub(crate)` (Crate) > `pub(super)` (Super) > inherited-private / `pub(self)` (Module). A `pub(in P)` form SHALL rank by its path matched **whole and single-segment**: exactly `crate` → Crate, exactly `super` → Super, exactly `self` → Module. Any **multi-segment or otherwise-unrecognized** `pub(in P)` path SHALL rank as **Crate, a conservative upper bound** — notably `pub(in super::super)`, which is legal Rust reaching the grandparent's whole subtree (broader than `pub(super)`) and therefore MUST NOT be ranked `Super`. A `pub(in P)` path is always an ancestor module within the crate, so such an item is at most crate-visible; ranking every unrecognized restricted form Crate never under-reacts (no false negative). The observed item kinds SHALL be exactly those of the prior rule — `fn`, `struct`/`enum`/`union`, `type`, `const`/`static`, `trait` (incl. alias), `extern crate`, **`mod`** (a submodule declaration), and **`use`** re-exports incl. a `use …::*` glob observed as a raw `Item::Use` node — **plus a `pub fn`, `pub static`, or `pub type` declared inside an `extern` block**: the FFI declaration is a real item in the enclosing module's own namespace, exactly as visible as a same-shaped ordinary `fn`/`static`/`type` item, and Rust cannot declare both an ordinary item and a foreign one under the same name in one module, so there is no identity collision in observing it identically (reusing the `fn`/`static`/`type` kinds verbatim rather than a distinct label). A foreign `macro` invocation and any unparsed foreign-item token stream carry no readable visibility keyword and stay out of scope, the same nature as this requirement's existing attribute-derived/opaque-token bounds below. An item at or below the ceiling SHALL NOT react.

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

#### Scenario: A pub fn/pub static/pub type inside an extern block is a violation

- **WHEN** the governed module has ceiling `Crate` and declares `unsafe extern "C" { pub fn open(h: *mut u8) -> u8; pub static K: u8; pub type Opaque; }` (the plain edition-2021 `extern "C" { … }` form behaves identically)
- **THEN** the system emits a violation for each of `pub fn open`, `pub static K`, and `pub type Opaque` — exactly as it would for the same-shaped ordinary items — rather than silently passing the whole block

#### Scenario: A non-pub extern-block item is not observed

- **WHEN** the governed module has ceiling `Crate` and declares `unsafe extern "C" { fn hidden() -> u8; static S: u8; type T; }` (no `pub` on any foreign item)
- **THEN** the system reports no violation, because none of the foreign items is declared above the ceiling

### Requirement: Observation bounds and scope

The rule SHALL govern only the **declared** visibility keyword on the module's own direct items; the prior bounds hold verbatim, plus one added conservative bound:

- **Incidental observation bounds** (stated, never a silent claim): an item produced by a macro expansion, or a `pub macro` (declarative macros 2.0, which parses as an opaque token item with no readable visibility) is not observed; `#[cfg]`-gated code is observed **as written** (cfg-agnostic). A module reached through a `cfg_attr`-wrapped `#[path]` remap IS observed (its conventional file and its `cfg_attr` target both read when they exist on disk), exactly like an **unconditional** `#[path = "…"]` module already is.
- **Out of declared scope (not this capability):** public surface that carries no visibility keyword *in this module* — a `#[macro_export] macro_rules!` (crate-public via attribute), a `#[no_mangle]`/`pub extern` symbol, an item re-exported *from another module*, or a foreign-item shape with no visibility syntax to observe (a macro invocation inside an `extern` block, or an unparsed foreign-item token stream). Governing attribute-derived public surface is the deferred attribute capability's domain; the static import dimension governs cross-module reachability. This capability makes no claim about them.
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

#### Scenario: A macro invocation inside an extern block is out of declared scope

- **WHEN** the governed module declares `unsafe extern "C" { m!(); }`, a macro invocation as a foreign item
- **THEN** the system does not react — a foreign macro invocation carries no visibility keyword, the same nature as an ordinary `#[macro_export]` macro's own out-of-scope bound
