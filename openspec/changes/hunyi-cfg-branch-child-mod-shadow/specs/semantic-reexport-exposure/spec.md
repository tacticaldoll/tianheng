## MODIFIED Requirements

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

This exclusion — at both the direct head and inside the closure, and on **both** halves named above
— SHALL itself be **cfg-aware**: a same-named child `mod` declaration does NOT shadow a `pub use`
re-export when the two are **provably mutually exclusive** under `#[cfg]`/`cfg_if!` — i.e. they can
never both be present in any single compiled configuration, so the local module never actually wins
name resolution over the extern prelude (nor the extern prelude's rename alias) for that `pub use`'s
own build. "Provably mutually exclusive" covers exactly two syntactic shapes, proven without a
general `cfg`-predicate satisfiability engine: (1) the `mod` and the `pub use` are two different
arms of the IDENTICAL `cfg_if!` invocation (its arms are exclusive by construction — only one
predicate in the `if`/`else if`/`else` chain is ever true); (2) each carries exactly one bare
`#[cfg(...)]` attribute and the two predicates are syntactic negations of one another (`#[cfg(P)]`
on one, `#[cfg(not(P))]` on the other, compared structurally — immune to a whitespace/formatting
difference, not by source text). When either holds, the `mod`'s name is NOT subtracted from **either**
the external-crate-name set **or** the crate-root rename map for that specific `pub use`'s own
resolution, even though both declarations live in the same governed/defining module and the same
crate-wide-closure pass observes both. The rename-map half is not a cosmetic mirror of the
extern-name half: `extern_verbatim_renamed` (the resolver both the direct head and the closure use)
checks the rename map **before** falling back to the extern-name set, and a rename alias (e.g. `wc`
from `extern crate serde as wc;`) is never itself a member of the extern-name set — only the real
crate name (`serde`) is. So leaving the rename-map half cfg-blind while fixing only the extern-name
half would not merely under-shadow a rename-aliased re-export; it would drop its resolution outright,
since the shadowed alias falls through to an extern-name-set fallback holding no candidate for it at
all. Anything less syntactically direct than the two proven shapes above — unrelated predicates (e.g.
`cfg(windows)` beside `cfg(target_os = "macos")`), arms of two *different* `cfg_if!` invocations, or
more than one bare `#[cfg]` attribute stacked on either the `mod` or the `pub use` — SHALL remain the
pre-existing cfg-blind default (the `mod` still shadows, on both halves): a stated, conservative
residual bound, not a guess dressed as an observation. The **unconditional** case this requirement's
own rustc rationale was written for — a `mod` and a `pub use` with no `#[cfg]` gating at all,
genuinely compiled together in every build — is unaffected on either half: the shadow still applies
exactly as before. A bare **type-position** head's own use of the rename map (see the rename
requirement below) is a SEPARATE, narrower shadow (governed by `semantic-signature-coupling`'s own
whole-type-namespace exclusion, not this re-export-only cfg-aware carve-out) and stays cfg-blind — an
explicit, distinct non-goal, not an oversight this carve-out forgot.

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

#### Scenario: A mutually-exclusive bare-#[cfg] sibling module does not shadow the re-export

- **WHEN** the governed module declares `#[cfg(unix)] mod serde;` and `#[cfg(not(unix))] pub use serde::Value;`, where `serde` is a declared dependency (and `src/<module>/serde.rs` exists, backing the `unix`-gated `mod`), under `must_not_expose("serde")`
- **THEN** the system reacts, emitting `serde::Value exposed by pub use <module>::Value`: the `unix`-gated `mod serde` and the `not(unix)`-gated `pub use` are syntactic negations of one another and so provably never compile together, meaning the `mod` never actually shadows this `pub use`'s own build — unlike the unconditional case (no `#[cfg]` on either), where the identical pair genuinely coexists and the `mod` does shadow it

#### Scenario: A mutually-exclusive cfg_if arm sibling module does not shadow the re-export

- **WHEN** the governed module declares `cfg_if! { if #[cfg(unix)] { mod serde; } else { pub use serde::Value; } }`, `serde` a declared dependency, under `must_not_expose("serde")`
- **THEN** the system reacts identically to the bare-`#[cfg]` form: the `mod` and the `pub use` are two arms of one `cfg_if!` invocation, provably never compiled together, so the `mod` does not shadow the `pub use`

#### Scenario: A mutually-exclusive sibling module does not shadow a facade's extern re-export through the closure

- **WHEN** `crate::a` declares `#[cfg(unix)] mod serde;` and `#[cfg(not(unix))] pub use serde::Value;` (`serde` a declared dependency), and the governed module `crate::domain` declares `pub use crate::a::Value;`, under `must_not_expose("serde")`
- **THEN** the system follows the closure to `serde::Value` and reacts: `crate::a`'s own `mod serde` does not suppress `crate::a`'s genuine re-export inside the crate-wide closure either, matching the direct-head behavior for the identical mutually-exclusive pair — the facade does not silently canonicalize to nothing

#### Scenario: An unconditional child module still shadows the re-export

- **WHEN** the governed module declares `mod serde { … }` and `pub use serde::Value;` with **no** `#[cfg]` on either — the unconditional case this requirement's shadow rule was originally written for, both genuinely coexisting in every build — under `must_not_expose("serde")`
- **THEN** the system does not react: the `mod` and the `pub use` compile together in every build, so the shadow applies exactly as it did before the cfg-mutual-exclusion carve-out — that carve-out narrows the shadow only for a provably-exclusive pair, never the unconditional case

#### Scenario: A mutually-exclusive bare-#[cfg] sibling module does not shadow a rename-aliased re-export

- **WHEN** a crate-root `extern crate serde as wc;` is declared, and the governed module declares `#[cfg(unix)] mod wc;` and `#[cfg(not(unix))] pub use wc::Value;`, under `must_not_expose("serde")`
- **THEN** the system reacts, emitting `serde::Value exposed by pub use <module>::Value`: the `unix`-gated `mod wc` and the `not(unix)`-gated `pub use` are syntactic negations of one another and so provably never compile together, meaning the `mod` never shadows the rename alias `wc` for this `pub use`'s own build — the rename-map half of the shadow gets the identical carve-out the extern-name half does, not merely a weaker or absent one

#### Scenario: A mutually-exclusive cfg_if arm sibling module does not shadow a rename-aliased re-export

- **WHEN** a crate-root `extern crate serde as wc;` is declared, and the governed module declares `cfg_if! { if #[cfg(unix)] { mod wc; } else { pub use wc::Value; } }`, under `must_not_expose("serde")`
- **THEN** the system reacts identically to the bare-`#[cfg]` form: the `mod` and the `pub use` are two arms of one `cfg_if!` invocation, provably never compiled together, so the `mod` does not shadow the rename alias `wc`

#### Scenario: A mutually-exclusive sibling module does not shadow a facade's rename-aliased re-export through the closure

- **WHEN** a crate-root `extern crate serde as wc;` is declared, `crate::a` declares `#[cfg(unix)] mod wc;` and `#[cfg(not(unix))] pub use wc::Value;`, and the governed module `crate::domain` declares `pub use crate::a::Value;`, under `must_not_expose("serde")`
- **THEN** the system follows the closure to `serde::Value` and reacts: `crate::a`'s own `mod wc` does not suppress `crate::a`'s genuine rename-aliased re-export inside the crate-wide closure either, matching the direct-head behavior for the identical mutually-exclusive pair

#### Scenario: An unconditional child module still shadows a rename-aliased re-export

- **WHEN** a crate-root `extern crate serde as wc;` is declared, and the governed module declares `mod wc { … }` and `pub use wc::Value;` with **no** `#[cfg]` on either — both genuinely coexisting in every build — under `must_not_expose("serde")`
- **THEN** the system does not react: the `mod` and the `pub use` compile together in every build, so the rename-alias shadow applies exactly as it did before the cfg-mutual-exclusion carve-out
