# semantic-reexport-exposure Specification

## Purpose
Close a false negative in signature-coupling: a named public re-export
(`pub use crate::infra::X;`) republishes a forbidden type on a module's public surface, so a bare
`must_not_expose` must react to it by default — like any other public-API exposure. Covers
aliased, grouped, whole-module (`{self}`), and facade-chained re-exports, and a glob whose root
resolves in/under the forbidden set. It also covers re-exports rooted at an **external crate**
(v0.1.4) — determined by the crate's external-crate name set (declared dependencies, `.rename`-aware
and `-`→`_` normalized, ∪ the sysroot crates), including single-segment crate-root re-exports and a
local facade chain terminating at an extern type. Default-on and API-compatible (the DSL is
unchanged), so it ships on the patch line as a behavior-changing bugfix; the residual — a
non-forbidden-root glob, a re-export routed through a *foreign* module (needs the foreign AST), a
**module-scoped** source `extern crate … as` rename (the **crate-root** form reacts, including
through a type alias or a facade closure) or a distinct `[lib] name` (absent from `cargo metadata
--no-deps`), a facade hop re-exporting a privately-`use`d bare name, an edition-2015 relative local
re-export, macro-generated — is a stated bound, never a silent pass. (`pub extern crate` is now an
observed exposure, not a bound.)
## Requirements
### Requirement: Named public re-exports are observed by default

A bare `must_not_expose(forbidden)` boundary SHALL observe the governed module's **named public
re-exports** (`pub use`) and react to a forbidden type republished through them. This is **on by
default** (no opt-in): a public re-export is the most direct public-API exposure — a missed
public-surface item, not an optional depth — so it is part of signature-coupling's default surface.
Reaction reuses signature-coupling's forbidden-type matching (exact resolved path OR `::`-delimited
module prefix), the shared `hunyi::resolve` resolver with the same `BareFallback::Ignore` policy, and
`canonicalize_through_reexports`, and folds into the same exit-code (0/1/2), `Baseline`, and severity
(`enforce` default / `warn`) contract. The used path SHALL be resolved and canonicalized before
matching, so a re-export reached through a local `pub use` **facade chain** resolves to its defining
path.

#### Scenario: A named public re-export reacts

- **WHEN** the governed module declares `pub use crate::infra::DbPool;` under `must_not_expose("crate::infra")`
- **THEN** the system emits a violation naming `crate::infra::DbPool`, exposed by the re-export

#### Scenario: A re-export through a facade chain reacts

- **WHEN** the governed module declares `pub use crate::facade::DbPool;` where `crate::facade` declares `pub use crate::infra::DbPool;`, under `must_not_expose("crate::infra")`
- **THEN** the system follows the `pub use` chain, canonicalizes to `crate::infra::DbPool`, and reacts, rather than silently passing it

#### Scenario: A clean module re-exporting no forbidden type passes

- **WHEN** the governed module declares only `pub use crate::api::Handle;` under `must_not_expose("crate::infra")`
- **THEN** the system reports no violation

### Requirement: Whole-module re-exports are observed

A re-export whose resolved path is a forbidden **module** (not only a leaf type) SHALL react — it is
the most blatant leak, republishing the whole module under the governed module's path. This SHALL
hold for a **named module re-export** (`pub use crate::infra as fs;`) and for a **`self` group
member** (`pub use crate::infra::{self, DbPool};`, whose `self` re-exports `crate::infra` itself). The
`self` group member SHALL resolve to the prefix module (`crate::infra`, collapsing the trailing
`self`) and be keyed in the seam by the **prefix's final segment** (the name the consumer binds,
`infra`), never the literal `self` — so two distinct `self`-group module re-exports stay distinct
findings. The system SHALL NOT distinguish a re-exported module from a re-exported type: whatever the
resolved path, if it is in/under the forbidden set the re-export reacts.

#### Scenario: A named module re-export reacts

- **WHEN** the governed module declares `pub use crate::infra as fs;` under `must_not_expose("crate::infra")`
- **THEN** the system emits a violation `crate::infra exposed by pub use crate::domain::fs`, because the whole forbidden module is republished under the name `fs`

#### Scenario: A self-group module re-export reacts, keyed by the module name

- **WHEN** the governed module declares `pub use crate::infra::{self, DbPool};` under `must_not_expose("crate::infra")`
- **THEN** the system emits `crate::infra exposed by pub use crate::domain::infra` (the `self` member, keyed by the prefix's final segment `infra`) and `crate::infra::DbPool exposed by pub use crate::domain::DbPool` (the leaf), never a seam keyed by the literal `self`

### Requirement: Re-export findings are seam-qualified by the exported path

A re-export finding SHALL be seam-qualified as `{canonical forbidden type} exposed by pub use
{exported-path}`, where `{exported-path}` is the module-qualified name the re-export publishes (the
alias when `as` is used, otherwise the re-exported leaf name). Two re-exports of the **same** forbidden
type under **different** exported names SHALL therefore produce **distinct** findings, so baselining
one MUST NOT mask the other under the `(target, rule, finding)` baseline identity (the one forbidden
false negative). Re-export findings SHALL share the `(target, rule)` of the signature-coupling
boundary (`rule` = "must not expose").

#### Scenario: An aliased re-export is keyed by its exported alias

- **WHEN** the governed module declares `pub use crate::infra::DbPool as Pool;` under `must_not_expose("crate::infra")`
- **THEN** the finding is `crate::infra::DbPool exposed by pub use crate::domain::Pool`, keyed by the exported alias `Pool`

#### Scenario: Two aliases of the same forbidden type stay distinct findings

- **WHEN** the governed module declares both `pub use crate::infra::DbPool;` and `pub use crate::infra::DbPool as Pool;`, and the first is recorded in the baseline as accepted
- **THEN** the aliased re-export still reacts: its seam names its own exported path, so the baseline identity does not mask it

#### Scenario: A grouped re-export reacts per leaf

- **WHEN** the governed module declares `pub use crate::infra::{DbPool, Config};` under `must_not_expose("crate::infra")`
- **THEN** the system emits one finding per re-exported leaf (`… pub use crate::domain::DbPool` and `… pub use crate::domain::Config`)

### Requirement: Only bare public re-exports are exposure

The system SHALL treat a **bare `pub use`** and a **`pub extern crate`** as public re-export
exposure. A `pub(crate) use`, a `pub(in path) use`, or a private `use` (or a private
`extern crate`) SHALL NOT be a violation, because it does not republish the type on the module's
public surface (it is the re-export analogue of the internal-use exemption). A **`pub extern
crate X [as Y];`** in the governed module republishes the external crate root `X` on the module's
public surface — like `pub use ::X;` — and SHALL react when `X` is in/under the forbidden set; the
exposure names the **real crate `X`** (not the `as`-rename `Y`), seam-qualified `X exposed by pub
extern crate X` (`extern crate self as …` is not an external exposure and is skipped). One form
SHALL remain a **documented non-observed bound**, never a silent claim of cleanliness: an
**underscore rename** (`pub use crate::infra::DbPool as _;`) imports a trait's methods without
binding a nameable path, so it exposes no name a consumer can reach through the module.

#### Scenario: A restricted-visibility re-export is not a violation

- **WHEN** the governed module declares `pub(crate) use crate::infra::DbPool;` under `must_not_expose("crate::infra")`
- **THEN** the system reports no violation, because a `pub(crate)` re-export is not public exposure

#### Scenario: A private use is not a violation

- **WHEN** the governed module declares `use crate::infra::DbPool;` (private, for internal use) under `must_not_expose("crate::infra")`
- **THEN** the system reports no violation

#### Scenario: A public extern crate re-export reacts

- **WHEN** the governed module declares `pub extern crate worklane_core;` (or `pub extern crate worklane_core as wc;`) where `worklane_core` is a declared dependency, under `must_not_expose("worklane_core")`
- **THEN** the system emits `worklane_core exposed by pub extern crate worklane_core`, naming the real crate, because a public extern crate republishes the crate root on the module's public surface

#### Scenario: A private extern crate is not a violation

- **WHEN** the governed module declares `extern crate worklane_core;` (no `pub`) under `must_not_expose("worklane_core")`
- **THEN** the system reports no violation, because a private extern crate does not republish the crate on the public surface

#### Scenario: An underscore rename is a documented non-observed bound

- **WHEN** the governed module declares `pub use crate::infra::DbPool as _;` under `must_not_expose("crate::infra")`
- **THEN** the system does not react — `as _` binds no nameable path a consumer can reach — and this is a stated bound, not a silent claim of cleanliness

### Requirement: Glob re-export reacts on a forbidden root, else a stated bound

For a glob re-export `pub use <root>::*;`, the system SHALL resolve the glob's **root** path (the
path up to the `*`) and match it against the forbidden set: if the root **is in/under** the forbidden
set (`root == entry` or `root` beneath the `entry::` prefix) the system SHALL react — re-exporting an
entire forbidden module or subtree is the most blatant leak and MUST NOT be silent. The finding SHALL
be rendered `{matched root} exposed by pub use {exported-root}::*` (pinned so its baseline identity is
stable). A glob whose root is **not** in/under the forbidden set SHALL be a **documented out-of-scope
bound** (its leaves are not enumerable without resolving the target module's contents — the inherited
glob bound), never a silent claim of cleanliness. This bound has **two sub-cases**, both stated: an
**unrelated/sibling root** (`pub use crate::elsewhere::*`), and — the sharper, foreseeable one — an
**ancestor root** whose glob spans a *deeper* forbidden prefix (`pub use crate::infra::*` under
`must_not_expose("crate::infra::db")`): the glob may surface the forbidden subtree as a child, but the
system cannot tell without enumerating the root's public children, so it stays a stated bound.
Reacting on an ancestor root is deliberately NOT done — it would be a false positive on genuinely
unobservable state (the forbidden child may not be a public re-export at all); closing it requires a
future cross-module-resolution capability, not a guess.

#### Scenario: A glob whose root is forbidden reacts, with a pinned finding

- **WHEN** the governed module `crate::domain` declares `pub use crate::infra::*;` under `must_not_expose("crate::infra")`
- **THEN** the system emits the violation `crate::infra exposed by pub use crate::domain::*`, because the glob's root `crate::infra` is in the forbidden set — the whole forbidden module is re-exported

#### Scenario: A glob whose root is deeper than the forbidden prefix reacts

- **WHEN** the governed module declares `pub use crate::infra::db::*;` under `must_not_expose("crate::infra")`
- **THEN** the system reacts, because the glob's root `crate::infra::db` is beneath the forbidden `crate::infra` prefix

#### Scenario: A sibling-root glob is a documented bound

- **WHEN** the governed module declares `pub use crate::elsewhere::*;` where `crate::elsewhere` transitively re-exports a `crate::infra` type, under `must_not_expose("crate::infra")`
- **THEN** the system does not claim to observe the transitively re-exported leaf (the inherited glob bound — the glob's leaves are not enumerable here), rather than silently asserting the boundary is clean

#### Scenario: An ancestor-root glob spanning a deeper forbidden prefix is a documented bound

- **WHEN** the governed module declares `pub use crate::infra::*;` under `must_not_expose("crate::infra::db")` (the forbidden prefix is *deeper* than the glob root)
- **THEN** the system treats it as a stated bound (it cannot enumerate whether `crate::infra` publicly re-exports the forbidden `db` subtree), documented as the sharper ancestor-root sub-case rather than lumped with the innocent sibling glob or silently claimed clean

### Requirement: External-crate re-exports are observed by default

A bare `must_not_expose(forbidden)` boundary SHALL observe a named public re-export
(`pub use`) whose first written segment is an **external crate**, and react when the
re-exported path is in/under the forbidden set. This is **on by default**: an extern-rooted
`pub use` republishes the named external type on the module's public surface exactly as a
local re-export republishes a local type — a missed public-surface item, so leaving it silent
is a false negative of the flagship signature-coupling boundary (the one forbidden bug).

The system SHALL determine external-crate-ness from the governed crate's **external-crate name
set**, composed from local-crate AST and declared-manifest data only:

- the crate's **declared dependencies**, read from the `cargo metadata --no-deps` the pipeline
  already consumes (each `dependencies[].name`, substituting `.rename` when present as the name
  written in source), each **normalized `-`→`_`** to match the Rust path spelling (a Cargo name
  `async-trait` is written `async_trait` in a path);
- **plus the sysroot crates** `std`, `core`, `alloc`, `proc_macro`, `test`, which never appear
  in `dependencies` yet are valid extern path heads.

A bare **re-export** (`pub use`) head is resolved against this set **with the governed module's own
child modules excluded** (`externs − child_module_names`): a `pub use dep::X;` head that names a
child `mod dep` **of the re-exporting module** is shadowed by that local module (rustc resolves it to
the local module — E0432 if the path is absent there — not the dependency), so attributing it to the
dependency would be a false positive. The shadow is **per-module**: a same-named module at another
level (e.g. a crate-root `mod dep` while the `pub use dep::X;` lives in a *child* module, where bare
`dep` reaches only the extern prelude) does NOT suppress the re-export — it still reacts (no false
negative). Only child **modules** are excluded, not the whole local type namespace: a same-named
child `mod` is the only shadow that arises in **compiling** code. A local `struct`/`enum`/`trait`/type
named `HEAD` also shadows the `use` head, but makes the re-export itself fail to compile (`HEAD::X` is
then "not a module" — E0433/E0432), so it never occurs in a buildable crate; on compiling code,
subtracting child modules and subtracting the whole type namespace therefore agree, and module-only
is the minimal, most-conservative choice (it also degrades most safely on mid-edit source). A bare
**type-position** head, by contrast, excludes the governed module's whole child-item type namespace
(`semantic-signature-coupling`), since a type-position head may denote any local type-namespace item.
A leading `::` (`pub use ::dep::X;`) bypasses the shadow entirely and resolves to the crate.

This same per-module child-module exclusion SHALL be applied **both** to the direct re-export head
resolution **and** inside the crate-wide re-export **closure** (`collect_reexports`, whose map
`canonicalize_through_reexports` / `canonicalize_through_aliases` follow), keyed by each collected
re-export's own **defining** module, and applied to **both** the external-crate set
(`externs − child_module_names`) **and** the crate-root rename map (`renames − child_module_names`),
exactly as the direct head does. A `pub use dep::X;` (extern-set variant) or `pub use wc::X;`
(crate-root-rename-alias variant) collected in a module `crate::a` that declares a child `mod dep` /
`mod wc` is not recorded as the dependency / renamed crate in the closure, so a **cross-module
facade** that re-exports it onward (`crate::b`'s `pub use crate::a::X;`) does not mis-canonicalize to
the dependency through the closure. A **leading-`::`** re-export (`pub use ::dep::X;`) SHALL bypass
the shadow inside the closure too: the closure honors the `use` item's leading colon and resolves
such a head against the **raw** sets — so the extern escape hatch still reacts through a facade even
under a same-named child `mod dep` (suppressing it would be a false negative). A genuine extern
facade chain — whose defining module declares no same-named child module — still records the extern
hop and reacts. The subtraction is scoped to each module's own declared children during the crate
walk, so the crate-root-vs-child distinction holds inside the closure exactly as it does for the
direct head.

The system SHALL additionally apply a **source-level crate-root `extern crate X as Y;` rename**:
a crate-root `extern crate` item with an `as`-rename binds `Y` crate-wide (the extern prelude),
so a head `Y` SHALL be mapped to the real crate `X` **before** the external-crate check, resolving
`Y::…` to the verbatim `X::…` path. This is read from the local AST (unlike `cargo metadata`, which
does not parse source `extern crate` renames), and is applied in the signature-coupling exposure
pipeline, covering a renamed head in a **type position** and in the **governed module's own
`pub use`**. Only a **crate-root** rename is collected — a module-scoped `extern crate … as …`
binds only within its module, so collecting it crate-wide would be a false positive (a stated bound
below).

The rename SHALL be resolved rustc-correctly in three positions of the head:

- **Bare head `Y::…`** — rewritten to `X::…`, **unless** the governed module declares its own child
  `mod Y`, which rustc lets shadow the extern alias within that module (bare `Y::…` is then the local
  module, not the crate). The rewrite is therefore applied with the governed module's own
  child-module names removed from the rename map. A bare `Y::…` in a module with **no** local `mod Y`
  still rewrites and reacts (suppressing it there would be a false negative). Only child **modules**
  shadow a `Y::…` path head — the sole shadow that arises in compiling code (a non-module local `Y`
  makes `Y::…` uncompilable).
- **Crate-relative spelling `crate::Y::…`** — rewritten to `X::…`. `crate::Y` unambiguously names the
  crate-root extern rename (a crate-root `mod Y` cannot coexist with `extern crate … as Y`), so no
  shadow applies and the rewrite is unconditional; only the segment **immediately** after `crate` is
  treated as the alias (a deeper `crate::m::Y` is a submodule item, not the rename). The rewrite is
  applied to the **final** resolved path (after the alias/re-export closure), so a `crate::Y::…`
  reached directly, through a `type` alias, or through a `pub use` target reacts alike.
- **Leading-`::` `::Y::…`** — an unambiguous extern, rewritten to `X::…` regardless of any local `mod Y`.

A bare head in this set resolves to its **verbatim** path; a bare head not in it keeps its
existing non-resolving behavior. The determination SHALL be applied in the bare-fallback branch
**after** `use`-map and `crate`/`self`/`super` resolution, so a local `use … as <depname>`
alias still wins. Matching reuses the exact-or-`::`-prefix comparison,
`canonicalize_through_reexports`, and the same exit-code / `Baseline` / severity /
seam-qualification contract. The forbidden operand is the extern path **as written in the
governed source** (for a renamed dependency, the in-source name); **no DSL change**.

#### Scenario: A bare dependency-rooted re-export reacts

- **WHEN** the governed module declares `pub use worklane_core::spi::Foo;` where `worklane_core` is a declared dependency, under `must_not_expose("worklane_core::spi")`
- **THEN** the system emits `worklane_core::spi::Foo exposed by pub use <module>::Foo`

#### Scenario: A hyphenated dependency is matched under its underscore path spelling

- **WHEN** the crate depends on `async-trait` and the governed module declares `pub use async_trait::Thing;`, under `must_not_expose("async_trait")`
- **THEN** the system reacts, because the dependency name is normalized `-`→`_` to the path spelling

#### Scenario: A sysroot-crate re-export reacts

- **WHEN** the governed module declares `pub use std::sync::Mutex;` under `must_not_expose("std::sync")`
- **THEN** the system reacts, because `std` is in the external-crate set though it is not a declared dependency

#### Scenario: An aliased dependency-rooted re-export is keyed by its exported alias

- **WHEN** the governed module declares `pub use worklane_core::spi::Foo as Bar;` under `must_not_expose("worklane_core::spi")`
- **THEN** the finding is `worklane_core::spi::Foo exposed by pub use <module>::Bar`, keyed by the alias so two aliases of the same extern type stay distinct under the baseline

#### Scenario: A grouped dependency-rooted re-export reacts per leaf

- **WHEN** the governed module declares `pub use worklane_core::spi::{Foo, Bar};` under `must_not_expose("worklane_core::spi")`
- **THEN** the system emits one finding per re-exported leaf

#### Scenario: A single-segment crate-root re-export reacts when the crate is forbidden

- **WHEN** the governed module declares `pub use worklane_core;` (or `pub use worklane_core as wc;`) where `worklane_core` is a declared dependency, under `must_not_expose("worklane_core")`
- **THEN** the system reacts — the whole forbidden dependency crate is republished

#### Scenario: A same-named local module does not suppress a subtree's extern re-export

- **WHEN** the governed crate declares a crate-root `mod worklane_core { … }` AND also depends on a crate `worklane_core`, and a **child** module `crate::domain` declares `pub use worklane_core::Foo;`
- **THEN** the system reacts, because the shadow is per-module: `crate::domain` (the re-exporting module) declares no child `mod worklane_core`, so `worklane_core` is not excluded from its re-export extern set — the crate-root module shadows only in the root module itself, not in a child, and suppressing here would be a false negative

#### Scenario: A cross-module facade reaching a child-shadowed head does not react

- **WHEN** `crate::a` declares both `pub use worklane_core::spi::Foo;` and a child `mod worklane_core { … }` (rustc resolves the bare head to the local module — E0432 if the path is absent there), and the governed facade module `crate::b` re-exports it onward with `pub use crate::a::Foo;`, under `must_not_expose("worklane_core::spi")` (the dependency)
- **THEN** the system does not misattribute the facade to the dependency: `crate::a`'s own child module `worklane_core` is excluded from its re-export extern set when the crate-wide closure collects `crate::a`'s re-exports, so the closure does not record `crate::a::Foo → worklane_core::spi::Foo`, and canonicalizing `crate::b::Foo` through the closure does not reach the dependency — no violation is emitted

#### Scenario: A cross-module facade reaching a rename-alias child-shadowed head does not react

- **WHEN** a crate-root `extern crate worklane_core as wc;` is declared, `crate::a` declares both `pub use wc::spi::Foo;` and a child `mod wc { … }` (which rustc lets shadow the bare alias head within `crate::a`; a submodule `mod wc` does not conflict with the crate-root rename), and the governed facade module `crate::b` re-exports it onward with `pub use crate::a::Foo;`, under `must_not_expose("worklane_core::spi")`
- **THEN** the system does not misattribute the facade to the renamed crate: `crate::a`'s own child module `wc` is removed from the rename map for `crate::a`'s bare re-export heads when the crate-wide closure collects `crate::a`'s re-exports, so the closure does not record `crate::a::Foo → worklane_core::spi::Foo`, and canonicalizing `crate::b::Foo` through the closure does not reach the renamed crate — no violation is emitted

#### Scenario: A leading-colon facade hop reacts through the closure despite a child module

- **WHEN** `crate::a` declares both `pub use ::worklane_core::spi::Foo;` (a leading-`::` extern head, which a same-named child module does not shadow) and a child `mod worklane_core { … }`, and the governed module `crate::b` re-exports it onward with `pub use crate::a::Foo;`, under `must_not_expose("worklane_core::spi")`
- **THEN** the system reacts: the closure honors the `use` item's leading colon and resolves the `::worklane_core` head against the raw external-crate set (unshadowed by the child `mod worklane_core`), so it records `crate::a::Foo → worklane_core::spi::Foo` and canonicalizes `crate::b::Foo` through the closure to the dependency

#### Scenario: A genuine extern facade chain still reacts through the closure

- **WHEN** `crate::facade` declares `pub use worklane_core::spi::Foo;` and declares **no** child `mod worklane_core`, and the governed module `crate::domain` declares `pub use crate::facade::Foo;`, under `must_not_expose("worklane_core::spi")`
- **THEN** the system reacts: `crate::facade`'s re-export extern set retains `worklane_core` (no same-named child module), so the closure records the extern hop and canonicalizes `crate::domain::Foo` to `worklane_core::spi::Foo` — the child-module exclusion is per defining module and does not suppress a genuine extern facade (no false negative)

#### Scenario: A source-level crate-root extern-crate rename resolves and reacts

- **WHEN** the governed crate declares a crate-root `extern crate worklane_core as wc;` and a module declares `pub use wc::spi::Foo;`, under `must_not_expose("worklane_core::spi")`
- **THEN** the system maps `wc` to `worklane_core` (read from the local AST) and emits `worklane_core::spi::Foo exposed by pub use <module>::Foo`, rather than silently passing it

#### Scenario: A source-level extern-crate rename in a type position resolves and reacts

- **WHEN** the governed crate declares a crate-root `extern crate worklane_core as wc;` and the governed module declares `pub fn make() -> wc::spi::Foo`, under `must_not_expose("worklane_core::spi")`
- **THEN** the system resolves `wc::spi::Foo` to `worklane_core::spi::Foo` and reacts, matching the re-export spelling

#### Scenario: A dependency-rooted re-export outside the forbidden set passes

- **WHEN** the governed module declares `pub use worklane_core::api::Handle;` under `must_not_expose("worklane_core::spi")`
- **THEN** the system reports no violation (neither the forbidden path nor beneath `worklane_core::spi::`)

#### Scenario: A renamed dependency is observed under its in-source name

- **WHEN** the crate declares `wc = { package = "worklane_core" }` and a module declares `pub use wc::spi::Foo;`, under `must_not_expose("wc::spi")`
- **THEN** the system reacts, matching the path as written (`wc`, from `.rename`); declaring the operand under the real crate name `worklane_core::spi` would not match — the stated as-written semantics

### Requirement: A local facade chain of inline re-exports terminating at an extern type reacts

The system SHALL follow a local public re-export facade chain **composed of inline `pub use`
path hops** that terminates at an external-crate type, and react to it — restoring for extern
endpoints the facade-chain guarantee already given for local endpoints. The re-export closure
SHALL retain an extern-headed target (head ∈ the external-crate set) so the chain canonicalizes
to the forbidden extern type rather than being dropped. A hop written as a re-export of a
privately-`use`d bare name (`use dep::spi::Foo; pub use Foo;`) is NOT captured (the closure
follows inline `pub use` paths only) — an inherited stated bound, not a silent claim.

#### Scenario: A facade chain of inline re-exports to an extern type is followed

- **WHEN** the governed module `crate::domain` declares `pub use crate::facade::Foo;`, and `crate::facade` declares `pub use worklane_core::spi::Foo;`, under `must_not_expose("worklane_core::spi")`
- **THEN** the system follows the local chain, canonicalizes to `worklane_core::spi::Foo`, and reacts

#### Scenario: A facade hop re-exporting a privately-used bare name is a stated bound

- **WHEN** `crate::facade` declares `use worklane_core::spi::Foo; pub use Foo;` (private import then bare re-export), re-exported onward by `crate::domain`, under `must_not_expose("worklane_core::spi")`
- **THEN** the system does not follow that hop (the closure captures only inline `pub use` paths) and this is a documented inherited bound, not silently claimed clean

### Requirement: External resolution has stated residual bounds

The system SHALL treat the following as **documented out-of-scope bounds**, never a silent
claim of cleanliness, because closing them needs observation not available from the local-crate
AST + declared manifest:

- A **glob over an external module whose root is not in/under the forbidden set**
  (`pub use worklane_core::spi::*;` under a disjoint operand): its leaves are not enumerable
  without the foreign module's contents. A glob whose **root** is in/under the forbidden set
  still reacts.
- A re-export **routed/renamed through a foreign module** (`pub use worklane_core::prelude::Foo;`
  where the foreign `prelude` re-exports `worklane_core::spi::Foo`): the written path is matched
  as-is; following it into the foreign crate needs its AST.
- A **module-scoped `extern crate worklane_core as wc;` rename** (declared inside `mod m { … }`,
  not at the crate root): its alias binds only within `m`, so it is not collected into the
  crate-wide rename map (collecting it would false-positive on a same-named head elsewhere). A
  **crate-root** `extern crate … as …` rename, by contrast, binds crate-wide via the extern
  prelude and is resolved wherever it is reached **as a bare head** (`wc::…`) — directly in a
  signature or re-export, **through a type alias** (`type H = wc::Foo;`), and **through a `pub use`
  facade closure** — because the (pre-collected) rename map is threaded into the exposure query, the
  alias-target resolution, and the re-export closure alike. Its bare head, crate-relative spelling
  (`crate::wc::…`), and submodule-shadow suppression are all resolved rustc-correctly (see the
  crate-root-rename clause of "External-crate re-exports are observed by default"); only the
  *module-scoped* rename remains a bound.
- A dependency that renames its **`[lib] name`** to a spelling not derivable from its package
  name (e.g. package `foo-thing` with `[lib] name = "foobar"`, imported as `foobar`): the
  foreign crate's target name lives in *its* manifest, absent from this crate's
  `cargo metadata --no-deps`, so `foobar` is not in the external set. (A `-`→`_` normalization is
  applied, so the common `foo-thing` → `foo_thing` case *is* covered; only a genuinely distinct
  `[lib] name` is the bound.)
- An **edition-2015 bare crate-root-relative re-export naming a local module** not shadowing a
  dependency (`pub use foo::Bar;`, `foo` local): the pre-2018 relative-path form — an inherited
  bound.

#### Scenario: A non-forbidden-root external glob is a documented bound

- **WHEN** the governed module declares `pub use worklane_core::spi::*;` under `must_not_expose("worklane_core::other")`
- **THEN** the system does not claim to observe the glob's individual leaves, rather than silently asserting the boundary is clean

#### Scenario: A forbidden-root external glob reacts on the root

- **WHEN** the governed module declares `pub use worklane_core::spi::*;` under `must_not_expose("worklane_core::spi")`
- **THEN** the system reacts on the glob root `worklane_core::spi`, consistent with the local forbidden-root glob rule

#### Scenario: A re-export renamed through a foreign module is a documented bound

- **WHEN** the governed module declares `pub use worklane_core::prelude::Foo;` where the foreign `worklane_core::prelude` re-exports `worklane_core::spi::Foo`, under `must_not_expose("worklane_core::spi")`
- **THEN** the system matches only the written path (`worklane_core::prelude::Foo`, not in/under the forbidden set) and does not silently claim to have followed the foreign chain

#### Scenario: A crate-root extern-crate rename reached through a type alias resolves and reacts

- **WHEN** the governed crate declares a crate-root `extern crate worklane_core as wc;` and the governed module declares `type H = wc::spi::Foo;` exposed by `pub fn make() -> H`, under `must_not_expose("worklane_core::spi")`
- **THEN** the system resolves the alias target through the rename to `worklane_core::spi::Foo` and emits a violation — the rename is honored in alias-target resolution, not only in a directly-written signature

#### Scenario: A crate-root extern-crate rename reached through a facade closure resolves and reacts

- **WHEN** a crate-root `extern crate worklane_core as wc;` is declared, `crate::facade` declares `pub use wc::spi::Foo;`, and the governed module `crate::domain` declares `pub use crate::facade::Foo;`, under `must_not_expose("worklane_core::spi")`
- **THEN** the system follows the re-export closure through the rename to `worklane_core::spi::Foo` and emits a violation

#### Scenario: A module-scoped extern-crate rename is a documented bound

- **WHEN** the governed crate declares `extern crate worklane_core as wc;` **inside** a module `mod m { … }` (not at the crate root) and `m` declares `pub fn make() -> wc::spi::Foo`, under `must_not_expose("worklane_core::spi")`
- **THEN** the system does not resolve `wc` to `worklane_core` (only crate-root renames are collected, since a module-scoped alias binds only locally) and this is a documented bound, distinct from the handled crate-root rename

#### Scenario: A crate-root rename reached by its crate-relative spelling reacts

- **WHEN** a crate-root `extern crate worklane_core as wc;` is declared and the governed module exposes `worklane_core::spi::Foo` written as `crate::wc::spi::Foo` (the crate-relative spelling of the alias), under `must_not_expose("worklane_core::spi")`
- **THEN** the system rewrites `crate::wc::…` to `worklane_core::…` (the segment immediately after `crate` is the crate-root rename alias) and emits a violation

#### Scenario: A bare alias head shadowed by a submodule's own child module does not react

- **WHEN** a crate-root `extern crate worklane_core as wc;` is declared and a governed submodule that also declares a child `mod wc { … }` exposes `wc::spi::Foo` (which rustc resolves to the local `mod wc`, not the crate), under `must_not_expose("worklane_core::spi")`
- **THEN** the system does not rewrite the bare `wc` head to `worklane_core` (the alias is removed from the rename map for that module's bare heads under its child-module shadow), so it does not misattribute the local path to the dependency

#### Scenario: A bare alias head with no local shadow still reacts

- **WHEN** a crate-root `extern crate worklane_core as wc;` is declared and a governed submodule with **no** local `mod wc` exposes `wc::spi::Foo`, under `must_not_expose("worklane_core::spi")`
- **THEN** the system rewrites the bare `wc` head to `worklane_core::spi::Foo` and emits a violation — the crate-wide bare rewrite is preserved (suppressing it here would be a false negative)

#### Scenario: A re-export head shadowed by a same-named local module does not react

- **WHEN** the governed module declares both `pub use worklane_core::spi::Foo;` and a local child `mod worklane_core { … }` (which rustc lets shadow the extern head — E0432 if the path is absent there), under `must_not_expose("worklane_core::spi")`
- **THEN** the system does not misattribute the `worklane_core` head to the dependency: the re-exporting module's own child module `worklane_core` is excluded from its re-export extern set, so the head is not resolved as the dependency and no violation is emitted (a genuine extern re-export coexisting with the local module is still reachable via `pub use ::worklane_core::spi::Foo;`)
