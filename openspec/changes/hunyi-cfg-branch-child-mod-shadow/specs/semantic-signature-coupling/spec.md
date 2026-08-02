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

#### Scenario: A mutually-exclusive SIBLING ITEM's child module does not shadow the item's own extern re-export

- **WHEN** the anchored module resolves to a SINGLE branch/file (no module-path split at all) that declares two mutually-exclusive sibling items directly — a `#[cfg(unix)] mod dep;` beside a `#[cfg(not(unix))] pub use dep::Something;` (real extern crate `dep`), or the identical pair as the two arms of one `cfg_if!` invocation
- **THEN** the system reacts on the `not(unix)`/else arm's own re-export, resolving `dep` as the real extern crate: the branch-level fix above (two DIFFERENT branches/files never merging their child-module shadows) is a no-op here, since both sibling items share the identical branch and file — the exclusion must instead be computed per re-export ITEM against its own provably-mutually-exclusive siblings, not once over the branch's whole child-module set (`semantic-reexport-exposure` owns the detailed cfg-mutual-exclusion rule this scenario exercises)

#### Scenario: A cfg_attr-wrapped-path anchor resolves through its own target with no resolving sibling at all

- **WHEN** the anchored module `crate::foo` is declared only as `#[cfg_attr(windows, path = "win.rs")] mod foo;` with no conventional `foo.rs` present, and `win.rs` (the `cfg_attr` target) exists and exposes a forbidden type
- **THEN** the system reads `win.rs` and reacts on the exposure, rather than reporting a constitution error — a `cfg_attr`-wrapped `#[path]` module's own target is now followed even with no sibling declaration to keep the branch count non-empty

#### Scenario: A cfg_attr-wrapped-path sibling reacts through its own file, not absorbed by another sibling's success

- **WHEN** the anchored module `crate::foo` is declared as two mutually-exclusive `#[cfg]` branches — one `#[cfg_attr(<pred>, path = "weird.rs")] mod foo;` and the other a plain `mod foo;` — and only the `cfg_attr` branch's target file exposes a forbidden type
- **THEN** the system reacts on that exposure — the `cfg_attr` branch's own resolution is never silently dropped merely because the OTHER, mutually-exclusive branch's plain declaration also resolved successfully (found on adversarial review: the prior fail-loud-only-when-completely-unresolvable check never fired once any sibling succeeded, so the `cfg_attr` branch's file vanished with no error and no reaction at all)
