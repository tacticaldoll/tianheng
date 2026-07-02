# semantic-reexport-exposure Specification

## Purpose
Close a false negative in signature-coupling: a named public re-export
(`pub use crate::infra::X;`) republishes a forbidden type on a module's public surface, so a bare
`must_not_expose` must react to it by default — like any other public-API exposure. Covers
aliased, grouped, whole-module (`{self}`), and facade-chained re-exports, and a glob whose root
resolves in/under the forbidden set. Default-on and API-compatible (the DSL is unchanged), so it
ships on the patch line as a behavior-changing bugfix; the residual (a non-forbidden-root glob,
cross-crate, macro-generated) is a stated bound, never a silent pass.

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

The system SHALL treat only a **bare `pub use`** as a public re-export exposure. A `pub(crate) use`,
a `pub(in path) use`, or a private `use` SHALL NOT be a violation, because it does not republish the
type on the module's public surface (it is the re-export analogue of the internal-use exemption).
Two further forms SHALL be **documented non-observed bounds**, never a silent claim of cleanliness:
an **underscore rename** (`pub use crate::infra::DbPool as _;`) imports a trait's methods without
binding a nameable path, so it exposes no name a consumer can reach through the module; and a
**`pub extern crate` re-export** (`pub extern crate infra as fs;`) republishes an *external* crate
root (a `syn::Item::ExternCrate`, outside the local-crate exposure scan), not a `crate::`-relative
forbidden path.

#### Scenario: A restricted-visibility re-export is not a violation

- **WHEN** the governed module declares `pub(crate) use crate::infra::DbPool;` under `must_not_expose("crate::infra")`
- **THEN** the system reports no violation, because a `pub(crate)` re-export is not public exposure

#### Scenario: A private use is not a violation

- **WHEN** the governed module declares `use crate::infra::DbPool;` (private, for internal use) under `must_not_expose("crate::infra")`
- **THEN** the system reports no violation

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

