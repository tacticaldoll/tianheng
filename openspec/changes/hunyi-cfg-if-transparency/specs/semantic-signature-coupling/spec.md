## ADDED Requirements

### Requirement: Transparent control-flow macro arm contents are observed

The system SHALL observe the contents of a **transparent control-flow macro** arm as real code: for a `cfg_if!` invocation in item position, each arm's items SHALL be collected exactly as if written at the invocation's own position, and a `mod` declaration inside an arm SHALL enter the module graph so its source file is scanned. A transparent macro wraps human-authored items without transforming identities, so treating its arms as macro-generated would leave a real, compiled item unobserved — an exposure written inside an arm would escape every single-module-anchored capability (signature-coupling, visibility, dyn/impl-trait, async-exposure) while the static dimension already reacts to the identical block, and a module declared only inside an arm would additionally hide its file's `unsafe` sites, forbidden markers, and trait impls from the crate-wide walk. Nested invocations SHALL recurse. An arm-declared module SHALL be treated as **cfg-conditional** for absent-file purposes — the arm's predicate gates it, every arm being conditionally compiled by construction — matching the static dimension's own rule rather than a re-derived one, while the ambiguity reaction (both conventional forms present) SHALL still fire regardless of arm membership. Transparency SHALL be gated on the macro **name** (`cfg_if`), and that gate is load-bearing rather than conservative: applied to an arbitrary macro invocation, arm extraction reads a nested `impl Foo { … }` body's braces as an arm and reports items the macro may never emit verbatim, a false positive. Three bounds SHALL therefore be stated rather than left silent — a body-wrapping macro under any **other** name is NOT covered and its contents remain unobserved; arms are unioned **cfg-blind**, so a violation written in an arm that the current configuration does not compile still reacts, since knowing which arm is live would require evaluating the whole feature and target resolution; and transparency applies to **item position** only, so an invocation written inside an `impl` or `trait` body (whose arms hold impl items rather than items, reached through a different set of walkers) is NOT flattened and its contents remain unobserved, a measured residual gap owned by its own change rather than silently absorbed into this one.

#### Scenario: An exposure inside a cfg_if arm reacts

- **WHEN** a governed module's body is `cfg_if! { if #[cfg(unix)] { pub fn leak() -> crate::forbidden::Thing { … } } else { … } }` and a boundary forbids `crate::forbidden::Thing`
- **THEN** the system reports the exposure (exit 1), rather than returning zero findings because the item sits inside a macro invocation

#### Scenario: The same exposure at top level also reacts

- **WHEN** the identical function is written directly in the governed module rather than inside `cfg_if!`
- **THEN** the system reports the exposure (exit 1) — the control establishing that the boundary reacts at all, so a clean result for the wrapped form would be a false negative rather than a misconfigured fixture

#### Scenario: Every arm of an else-if chain is observed

- **WHEN** a `cfg_if!` invocation has three arms (`if` / `else if` / `else`) and only the last exposes a forbidden type
- **THEN** the system reports that exposure — arms are unioned cfg-blind, never stopping at the first

#### Scenario: A nested cfg_if invocation is observed

- **WHEN** a forbidden exposure sits inside a `cfg_if!` invocation written inside another `cfg_if!` arm
- **THEN** the system reports the exposure, recursing into the inner invocation

#### Scenario: A module declared inside a cfg_if arm is scanned

- **WHEN** a crate declares `cfg_if! { if #[cfg(unix)] { pub mod unix_impl; } else { pub mod windows_impl; } }`, `src/unix_impl.rs` exists and contains a forbidden exposure, and a boundary governs `crate::unix_impl`
- **THEN** the system resolves the arm-declared module and reports the exposure, rather than treating `crate::unix_impl` as unknown

#### Scenario: An arm-declared module with no file is tolerated

- **WHEN** the same declaration pair exists but `src/windows_impl.rs` does not
- **THEN** the system skips the fileless arm declaration rather than reporting a missing-file constitution error, matching the static dimension's rule for the identical shape — the arm's predicate is the gate, so rustc strips the whole arm and the crate compiles

#### Scenario: An arm-declared dual-backed module is still an ambiguity

- **WHEN** a `mod child;` declared inside a `cfg_if!` arm resolves to both `src/child.rs` and `src/child/mod.rs`
- **THEN** the system reports the ambiguity constitution error (exit 2) — arm membership makes an absence tolerable, never two present files resolvable

#### Scenario: An invocation inside an impl body is a stated bound

- **WHEN** a governed module writes `impl Api { cfg_if! { if #[cfg(unix)] { pub fn leak() -> crate::forbidden::Thing { … } } } }` and a boundary forbids `crate::forbidden::Thing`
- **THEN** the system reports no exposure — transparency covers item position, where an arm's contents are items; an `impl`-body invocation's arms are impl items, observed through different walkers, and that remains a declared gap rather than a claimed reaction

#### Scenario: A macro under another name is not treated as transparent

- **WHEN** a governed module's body is `generate_wrapper! { impl Foo { pub fn hidden() -> crate::forbidden::Thing { … } } }` and a boundary forbids `crate::forbidden::Thing`
- **THEN** the system reports no exposure — the invocation is not transparent, so its body stays a stated coverage bound; extracting from it would read the `impl` body's braces as an arm and report an item the macro may never emit
