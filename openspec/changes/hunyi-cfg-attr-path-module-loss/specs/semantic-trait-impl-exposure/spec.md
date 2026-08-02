## MODIFIED Requirements

### Requirement: Resolution, matching, and reaction reuse signature-coupling

Trait-impl exposure SHALL reuse signature-coupling's forbidden-type matching (exact resolved path
OR `::`-delimited module prefix) and the shared `hunyi::resolve` resolver with the **same bare-name
fallback policy as signature-coupling** — a bare, unqualified local name SHALL NOT be resolved
against the current module (`BareFallback::Ignore`), so an impl position naming a bare local name is
not turned into a same-module false positive. Resolution SHALL follow in-scope `use`s (incl.
renames), `crate`/`self`/`super`-relative paths, and local `pub use` re-export chains. A type whose
resolution requires a glob import, a macro-generated type, or full inference SHALL be an inherited
OUT-OF-SCOPE bound, never a silent pass, and no new hole SHALL be introduced. A type defined only in
a module reached through a `cfg_attr`-wrapped `#[path]` remap is NOT out of scope: like the
already-followed **unconditional** `#[path = "…"]` form, it is collected into the crate-wide closure
this capability shares with signature-coupling — a file module's conventional file and its
`cfg_attr` target both read when they exist on disk, cfg-blind union rather than a skip bound. Within
the resolved scope there SHALL be no false negative. Trait-impl exposure findings SHALL fold into the
same exit-code contract (**0** clean, **1** enforced violation, **2** constitution/scan error), the
same `Baseline` gating, and the same severity semantics (`enforce` default, `warn`) as
signature-coupling.

#### Scenario: A bare local name in an impl position is not a false positive

- **WHEN** the governed module declares `impl From<DbPool> for Service` where `DbPool` is a bare, unqualified name resolvable only against the current module, under a boundary forbidding `crate::infra` with `.including_trait_impls()`
- **THEN** the system does not resolve the bare name against the current module (parity with signature-coupling's `BareFallback::Ignore`) and does not emit a same-module false positive

#### Scenario: A re-exported forbidden type in an impl position resolves and reacts

- **WHEN** the governed module declares `use crate::facade::DbPool;` (where `crate::facade` declares `pub use crate::infra::DbPool;`) and declares `impl From<DbPool> for Service` under a boundary forbidding `crate::infra` with `.including_trait_impls()`
- **THEN** the system follows the `pub use` chain, resolves `DbPool` to `crate::infra::DbPool`, and emits a `trait-arg` violation rather than silently passing it

#### Scenario: A glob-imported type in an impl position is a documented coverage bound

- **WHEN** the governed module declares `use crate::infra::*;` and `impl From<DbPool> for Service` with `.including_trait_impls()`
- **THEN** the system does not claim to observe it (inherited glob bound), rather than silently asserting the boundary is clean

#### Scenario: A baselined trait-impl exposure does not fail; a new one does

- **WHEN** an `enforce`-severity boundary's only trait-impl exposures are all present in the baseline
- **THEN** the system reports them accepted and does not fail; and WHEN a new exposure not in the baseline appears at any position, the system fails the reaction (exit 1)

#### Scenario: A forbidden type re-exported only from a cfg_attr-wrapped-path module resolves and reacts

- **WHEN** a facade module is reached only via `#[cfg_attr(windows, path = "weird.rs")] pub mod facade;` with no conventional `facade.rs` present, `weird.rs` declares `pub use crate::infra::DbPool;`, the governed module declares `use crate::facade::DbPool; impl From<DbPool> for Service` under a boundary forbidding `crate::infra` with `.including_trait_impls()`
- **THEN** the system reads `weird.rs` into the crate-wide re-export closure, resolves `DbPool` to `crate::infra::DbPool`, and emits a `trait-arg` violation rather than treating the facade module as out of scope
