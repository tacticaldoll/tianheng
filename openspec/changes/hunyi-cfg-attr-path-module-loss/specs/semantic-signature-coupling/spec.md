## MODIFIED Requirements

### Requirement: Anchor resolution

For each semantic boundary, the system SHALL resolve the named governed module anchor to a real module in the target crate's source before evaluating it. If the anchor cannot be resolved — an unknown module path, or a target crate absent from the workspace — the system SHALL treat this as a **constitution error** (exit 2), failing loud and distinct from a boundary violation (exit 1), so a mistyped anchor is not reported as architectural drift.

#### Scenario: Anchor resolves to a real item

- **WHEN** a boundary anchors to `crate::domain` and that module exists in the target crate's source
- **THEN** the system observes that module's public signatures for comparison

#### Scenario: Unresolvable anchor is a constitution error

- **WHEN** a boundary anchors to a module path that does not exist in the target crate's source
- **THEN** the system emits a constitution error naming the unresolved anchor and exits 2, never exit 0 (no silent pass) and never exit 1

#### Scenario: A cfg-duplicated inline anchor governs every variant

- **WHEN** the anchored module is declared as two `#[cfg(…)] mod x { … }` inline variants (which `syn` parses as two separate modules, evaluating no `cfg`), and only the source-*later* variant exposes a forbidden type
- **THEN** the system observes the union of all same-named inline variants and reacts on the exposure, never resolving only the source-first variant (a `mod`-resolution divergence from the crate-wide scan is the false-negative class this resolver forbids). This anchor-resolution property is shared by every single-module-anchored semantic capability (visibility, dyn/impl-trait, async-exposure), not only signature-coupling; an **unconditional** `#[path = "…"]` file module is followed to its target, and a `cfg_attr`-wrapped `#[path]` module is followed too — its conventional file and its `cfg_attr` target both read when they exist on disk, cfg-blind union rather than a skip bound, exactly like the crate-wide walk. Only when NEITHER a conventional file NOR an existing `cfg_attr` target backs a declaration, and it carries no other cfg-conditional gate, is resolution a genuine constitution error.

#### Scenario: A cfg-mixed inline and file-form anchor governs both variants

- **WHEN** the anchored module is declared as one `#[cfg(feature = "a")] mod x { … }` inline variant and one `#[cfg(feature = "b")] mod x;` file-form variant (the standard per-platform shim pairing an inline body with a file-form sibling), and only the file-form variant exposes a forbidden type
- **THEN** the system observes both variants' items and reacts on the file-form exposure, never stopping at the inline variant merely because it was found first — the same additive, cfg-blind union as two inline variants of one name, shared by every single-module-anchored semantic capability

#### Scenario: A segment nested beneath a flat cfg-mixed sibling resolves from its own directory

- **WHEN** `x` is cfg-mixed (an inline variant on one arm, a flat, non-`mod.rs` file-form sibling on another), and the anchor is a further segment `x::y` reached through an unconditional `#[path]` written inside the flat file-form sibling itself
- **THEN** the system resolves `y` from the file-form sibling's own containing directory — the same directory a `#[path]` written in an ordinary flat file always resolves from — rather than from the inline variant's accumulated directory, which coincides with it only when the file-form sibling is `mod.rs`-shaped

#### Scenario: A plain child of a #[path]-remapped anchor resolves from the remap's own directory

- **WHEN** the anchored module is `crate::net::inner`, `crate::net` is declared `#[path = "moved/thing.rs"] pub mod net;`, and `moved/thing.rs` declares a plain `pub mod inner;`
- **THEN** the system resolves `inner` to `moved/inner.rs` — the `#[path]`-loaded file's own directory, since it is mod-rs-like regardless of its own filename — never a name-derived `net/inner.rs` that has no relationship to where the file actually lives

#### Scenario: Two non-inline cfg-sibling variants, one plain and one path-remapped, both govern

- **WHEN** the anchored module is declared as one `#[cfg(feature = "a")] mod x;` plain variant (backed by `x.rs`) and one `#[cfg(feature = "b")] #[path = "moved.rs"] mod x;` remapped variant, and only the remapped variant exposes a forbidden type
- **THEN** the system observes both variants' items and reacts on the remapped exposure, matching the crate-wide walk's own policy of never stopping at the first non-inline declaration for a name — two non-inline siblings need not name the same file once an unconditional `#[path]` can relocate one of them

#### Scenario: An inline module carrying an unconditional path is resolved, not reported unknown

- **WHEN** the anchored module is `crate::thread` (or a further segment beneath it, e.g. `crate::thread::local_data`), declared `#[path = "thread_files"] pub mod thread { pub mod local_data; }` — an inline header with an unconditional `#[path]` — and `thread_files/local_data.rs` exposes a forbidden type
- **THEN** the system resolves `crate::thread` (finding its inline items) and follows the `#[path]` to relocate the base `local_data` resolves from to `thread_files/`, reacting on the exposure — rather than reporting `crate::thread` as an unknown module merely because an unconditional `#[path]` precedes its inline header

#### Scenario: A cfg-split module's own use-map does not merge across mutually-exclusive branches

- **WHEN** the anchored module is declared as two mutually-exclusive `#[cfg]` branches, each declaring `use <different real path> as Handle;` under the same local alias name, and only the FIRST branch's own bare `Handle` reference genuinely resolves to a forbidden type
- **THEN** the system reacts on the first branch's own exposure, resolving its bare `Handle` reference through THAT branch's own `use` declaration — never through the second, mutually-exclusive branch's `use Handle` alias merely because both branches' items were observed in one pass

#### Scenario: A cfg-split branch's own child module does not shadow a sibling branch's extern re-export

- **WHEN** the anchored module is declared as two mutually-exclusive `#[cfg]` branches, one declaring a local child module with the same name as a real extern crate dependency, and the OTHER branch (with no such local child module) contains a genuine `pub use <dep>::Something;` naming the real extern crate
- **THEN** the system reacts on the second branch's own re-export, resolving `<dep>` as the real extern crate — never treating it as shadowed by a local child module that only the FIRST, mutually-exclusive branch declares

#### Scenario: Two INLINE cfg siblings sharing one enclosing file do not merge their use-maps or child-module shadows

- **WHEN** the anchored module is declared as two mutually-exclusive `#[cfg]` branches, BOTH inline (`#[cfg(a)] mod x { .. }` / `#[cfg(b)] mod x { .. }`, sharing the identical enclosing file), each declaring its own `use <different real path> as Handle;` under the same local alias name, and only the FIRST arm's own bare `Handle` reference genuinely resolves to a forbidden type
- **THEN** the system reacts on the first arm's own exposure, resolving its bare `Handle` reference through THAT arm's own `use` declaration — never through the second, mutually-exclusive arm's `use Handle` alias merely because both arms are inline and share one file; the same isolation holds for a local child module in one inline arm shadowing the other inline arm's own genuine extern re-export

#### Scenario: A cfg_attr-wrapped-path anchor resolves through its own target with no resolving sibling at all

- **WHEN** the anchored module `crate::foo` is declared only as `#[cfg_attr(windows, path = "win.rs")] mod foo;` with no conventional `foo.rs` present, and `win.rs` (the `cfg_attr` target) exists and exposes a forbidden type
- **THEN** the system reads `win.rs` and reacts on the exposure, rather than reporting a constitution error — a `cfg_attr`-wrapped `#[path]` module's own target is now followed even with no sibling declaration to keep the branch count non-empty

#### Scenario: A cfg_attr-wrapped-path sibling reacts through its own file, not absorbed by another sibling's success

- **WHEN** the anchored module `crate::foo` is declared as two mutually-exclusive `#[cfg]` branches — one `#[cfg_attr(<pred>, path = "weird.rs")] mod foo;` and the other a plain `mod foo;` — and only the `cfg_attr` branch's target file exposes a forbidden type
- **THEN** the system reacts on that exposure — the `cfg_attr` branch's own resolution is never silently dropped merely because the OTHER, mutually-exclusive branch's plain declaration also resolved successfully (found on adversarial review: the prior fail-loud-only-when-completely-unresolvable check never fired once any sibling succeeded, so the `cfg_attr` branch's file vanished with no error and no reaction at all)

### Requirement: Name resolution scope and no false negative

The system SHALL resolve a type named in a signature using the **shared 渾儀 resolver** (`hunyi::resolve`), and within the resolved scope there SHALL be no false negative and no false positive: a forbidden type that *is* resolvable MUST react, and a name that resolves to a **local** item MUST NOT be mis-attributed to a same-named dependency. Resolution SHALL agree with rustc name resolution wherever the answer is observable from the local-crate AST:

- **A leading `::` is an unambiguous extern.** A path written `::serde::Value` resolves to the external crate named by its first segment, bypassing the `use`-map and any local shadow. It SHALL NOT be resolved as a relative path (which would both miss the extern exposure and, via the `use`-map, mis-attribute it to a local path).
- **A local type-namespace item shadows the extern prelude.** A bare head naming a local `struct`/`enum`/`union`/`trait`/`type`-alias/`mod` in the governed module denotes that local item, and the extern oracle SHALL NOT fire for it.
- **A bare local-alias chain resolves regardless of collection order.** When a type alias's target is itself a bare local alias whose name shadows a dependency (`type serde = crate::infra::Db; type X = serde;`), the alias-collection ladder SHALL resolve the local alias before the extern oracle (identical to the query ladder), closing the chain to the defining path.
- **A mutually-exclusive `#[cfg]` collision on a `use`-map name or a `pub use` re-export target does not suppress either candidate.** When two mutually-exclusive `#[cfg]` branches (bare `#[cfg]` or `cfg_if!` arms alike) each declare `use ... as Name;` (or `pub use ... as Name;`) for the identical local name with different targets, the system SHALL treat both targets as candidates and react if resolving through EITHER one exposes a forbidden type — never silently keeping only the declaration that happens to be written last.

A type whose resolution would require capabilities beyond the local AST — a glob import, a macro-generated type, a generic type alias, nominal paths nested only inside alias-target forms outside the explicitly supported non-generic compound constructors below, or full inference — remains OUT OF SCOPE, a stated coverage bound, never a claimed reaction. A type defined only in a module reached through a `cfg_attr`-wrapped `#[path]` remap is NOT out of scope: like the already-followed **unconditional** `#[path = "…"]` form, its types, aliases, and re-exports ARE collected into the crate-wide closure and resolvable — an inline body regardless of the attribute (which has no effect on it), and a file module's conventional file and its `cfg_attr` target both read when they exist on disk, cfg-blind union rather than a skip bound.

#### Scenario: A leading-`::` extern path resolves and reacts through a local shadow

- **WHEN** the governed module declares a local `mod serde` (or `use crate::vendor::serde;`) and `pub fn f() -> ::serde::Value`, under `must_not_expose("serde")`
- **THEN** the system resolves `::serde::Value` to the external crate `serde` and emits a violation, and does NOT mis-attribute it to `crate::vendor` under a boundary forbidding `crate::vendor`

#### Scenario: A local type named like a dependency is not a false positive

- **WHEN** the governed module declares `pub struct serde; pub fn f() -> serde`, under `must_not_expose("serde")`
- **THEN** the system resolves `serde` to the local struct and does NOT react, while a real `use serde::Value; pub fn g() -> Value` under the same boundary still reacts

#### Scenario: A bare local-alias-of-an-alias shadowing a dependency resolves and reacts

- **WHEN** the governed module declares `type serde = crate::infra::Db; type X = serde; pub fn f() -> X`, under `must_not_expose("crate::infra")` (in either source order)
- **THEN** the system resolves the local alias `serde` before the extern oracle, closes the chain to `crate::infra::Db`, and emits a violation

#### Scenario: Two mutually-exclusive cfg-gated use aliases for the same name both react

- **WHEN** the governed module declares `#[cfg(unix)] use crate::infra::Secret as Handle; #[cfg(not(unix))] use crate::safe::Handle; pub fn leak() -> Handle`, under `must_not_expose("crate::infra")`, in either declaration order
- **THEN** the system emits a violation naming `crate::infra::Secret`, regardless of which `use` line is written first — the verdict never depends on source order

#### Scenario: Two mutually-exclusive cfg-gated re-export targets for the same name both canonicalize correctly

- **WHEN** a facade module declares `#[cfg(unix)] pub use crate::infra::Secret as Handle; #[cfg(not(unix))] pub use crate::safe::Thing as Handle;`, another module exposes `crate::facade::Handle`, and the boundary forbids `crate::infra`, in either declaration order
- **THEN** the system emits a violation naming `crate::infra::Secret`, regardless of which `pub use` line is written first

#### Scenario: A re-export declared only in a cfg_attr-wrapped-path module is resolved and reacts

- **WHEN** a facade module is reached only via `#[cfg_attr(windows, path = "weird.rs")] pub mod facade;` with no conventional `facade.rs` present, `weird.rs` declares `pub use crate::infra::Secret;`, another module exposes `crate::facade::Secret`, and the boundary forbids `crate::infra`
- **THEN** the system reads `weird.rs` into the crate-wide re-export closure and emits a violation naming `crate::infra::Secret`, rather than treating the facade module as out of scope and passing the exposure through unresolved
