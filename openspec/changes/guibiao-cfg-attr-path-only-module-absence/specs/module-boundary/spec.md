## MODIFIED Requirements

### Requirement: A plain module declaration resolves to exactly one conventional file

A plain `mod name;` declaration SHALL resolve to exactly one conventional source file — `name.rs` or `name/mod.rs` — and the system SHALL react rather than guess in every other outcome, never silently dropping the module from the reachable set (which would hide every import beneath it, the false negative the core contract forbids). When **both** forms are present the system SHALL report a constitution error (exit 2) naming both resolved paths and the exactly-one-file rule, regardless of any `cfg_attr(path)` candidate also present on the same declaration — the ambiguity test SHALL precede the absent-file tolerance below and SHALL NOT be overridden by it, so a declaration whose predicate is off — which rustc strips before module resolution, leaving a crate that compiles cleanly and raises no E0761 — is still a constitution error: the scanner is cfg-blind and cannot know which arm is live, and treating one arm's ambiguity as resolvable would require evaluating `cfg`. When **neither** form is present the system SHALL report a constitution error (exit 2) naming both expected paths, EXCEPT when the declaration is **cfg-conditional**, in which case the module may legitimately have no file in the current configuration and SHALL be skipped rather than errored. A declaration SHALL be cfg-conditional from any of three sources, which the system SHALL treat identically because they express one intent — "this declaration may legitimately have no conventional file in the active configuration": a **bare** `#[cfg(...)]` attribute preceding the item; membership in a transparent control-flow macro arm (a `mod` written directly inside a `cfg_if!` arm, whose predicate lives in the macro's `if #[cfg(..)]` header rather than on the item — every such arm is conditionally compiled by construction, the trailing `else` on its predicate's negation); or the declaration carrying one or more `cfg_attr(..., path = "…")` remap attributes of which **at least one candidate physically resolves to a real file on disk**. A `#[cfg_attr(...)]` wrapper that carries no `path` meta at all, or whose every `path` remap candidate is absent from disk, SHALL NOT make a declaration cfg-conditional on that basis alone: `cfg_attr` never removes the item, it only conditionally applies its wrapped attribute, so with no resolved candidate to back it a missing conventional file beneath it is a genuine compile error (E0583) in every configuration — the same blind, existence-only test the plain-file check itself already uses, extended to a resolved conditional remap target, never a predicate-exhaustiveness proof the scanner cannot perform. The same cfg-conditional test SHALL govern an absent `#[path]` remap target, so the two absence outcomes cannot drift apart. Either constitution error SHALL abort the whole reachability walk rather than excluding one module, since a crate whose module graph cannot be resolved cannot be judged. This is the static dimension's own independently-implemented policy for these outcomes; the runtime dimension states the same rules for its own probe-coverage walker (三儀 ⊥ 三儀: the same rule, not the same function).

#### Scenario: A module backed by both conventional forms is a constitution error

- **WHEN** a crate declares a plain `mod child;` and both `src/child.rs` and `src/child/mod.rs` exist
- **THEN** the system reports a constitution error (exit 2) naming both resolved paths and the exactly-one-file rule, rather than accepting either form as the module's source or treating the two as separate sources of one module path

#### Scenario: A cfg-gated dual-backed declaration is still an ambiguity, though the crate compiles

- **WHEN** the dual-backed `mod child;` declaration carries a bare `#[cfg(...)]` gate whose predicate is off, so rustc strips the declaration before module resolution and the crate compiles
- **THEN** the system still reports the ambiguity constitution error (exit 2) — cfg-conditionality covers an *absent* conventional file and never two present ones

#### Scenario: A dual-backed declaration inside a cfg_if arm is still an ambiguity

- **WHEN** a `mod child;` declared inside a `cfg_if!` arm resolves to both `src/child.rs` and `src/child/mod.rs`
- **THEN** the system still reports the ambiguity constitution error (exit 2) — arm membership makes an absence tolerable, never two present files resolvable

#### Scenario: A dual-backed declaration alongside a resolved cfg_attr(path) candidate is still an ambiguity

- **WHEN** a crate declares `#[cfg_attr(unix, path = "weird.rs")] mod child;`, `weird.rs` exists on disk, and BOTH `src/child.rs` and `src/child/mod.rs` also exist
- **THEN** the system still reports the ambiguity constitution error (exit 2) — a resolved conditional remap candidate makes an *absent* conventional file tolerable, never an ambiguity between two present ones resolvable

#### Scenario: An unconditionally missing conventional file is a constitution error

- **WHEN** a crate declares a plain `mod child;` with no `#[cfg]` gate and neither `src/child.rs` nor `src/child/mod.rs` exists
- **THEN** the system reports a constitution error (exit 2) naming both expected paths, rather than silently dropping `crate::child` from the reachable set

#### Scenario: A bare cfg-gated missing file is tolerated

- **WHEN** a crate declares `#[cfg(feature = "extra")] mod child;` and neither conventional file exists
- **THEN** the system skips the declaration rather than erroring, since an off predicate legitimately leaves the module with no file in this configuration

#### Scenario: A missing file for a module declared inside a cfg_if arm is tolerated

- **WHEN** a crate declares `cfg_if! { if #[cfg(unix)] { pub mod unix_impl; } else { pub mod windows_impl; } }`, `src/unix_impl.rs` exists, and `src/windows_impl.rs` does not
- **THEN** the system skips the fileless arm declaration rather than erroring, and judges the crate — matching the identical shape written as two bare-`#[cfg]`-gated declarations, which it already tolerates; refusing one spelling and accepting the other would make a working build's verdict depend on which form its author chose

#### Scenario: An arm module whose file exists is still reached and governed

- **WHEN** a crate declares `cfg_if! { if #[cfg(unix)] { pub mod unix_impl; } else { pub mod windows_impl; } }`, only `src/unix_impl.rs` exists, it contains a forbidden import, and a boundary governs `crate::unix_impl`
- **THEN** the system reports the forbidden import violation (exit 1) — tolerating the sibling arm's absent file does not stop the present arm's module from being observed

#### Scenario: A cfg_attr-wrapped missing file with no path meta is not tolerated

- **WHEN** a crate declares `#[cfg_attr(unix, allow(dead_code))] mod child;` and neither conventional file exists
- **THEN** the system reports the missing-file constitution error (exit 2), because this `cfg_attr` carries no `path` remap at all — `cfg_attr` never removes the item, so with no candidate to back it the absent file is a genuine compile error in every configuration, and tolerating it would be a silent pass over source that cannot build

#### Scenario: A single resolved cfg_attr(path) candidate tolerates a missing conventional file

- **WHEN** a crate declares `#[cfg_attr(unix, path = "weird.rs")] mod child;`, `weird.rs` exists on disk, and neither `src/child.rs` nor `src/child/mod.rs` exists
- **THEN** the system skips the plain-file requirement rather than erroring — the resolved candidate is treated exactly like a bare `#[cfg]`-tolerated absence — and `weird.rs` is still governed under `crate::child` through the union-scan the sibling requirement below already performs

#### Scenario: Stacked resolved cfg_attr(path) candidates tolerate a missing conventional file

- **WHEN** a crate declares `#[cfg_attr(unix, path = "unix_child.rs")] #[cfg_attr(not(unix), path = "other_child.rs")] mod child;`, BOTH `unix_child.rs` and `other_child.rs` exist on disk, and neither `src/child.rs` nor `src/child/mod.rs` exists
- **THEN** the system skips the plain-file requirement rather than erroring, and both `unix_child.rs` and `other_child.rs` are governed under `crate::child` — the shape every real rustc build compiles through exactly one of the two mutually-exhaustive targets, never through a conventional file this declaration never needs

#### Scenario: An unresolved cfg_attr(path) candidate alone does not tolerate a missing conventional file

- **WHEN** a crate declares `#[cfg_attr(windows, path = "windows_only.rs")] mod child;`, `windows_only.rs` does NOT exist on disk, and neither `src/child.rs` nor `src/child/mod.rs` exists either
- **THEN** the system reports the missing-file constitution error (exit 2) — every candidate this declaration could compile through is absent, and no bare `#[cfg]`/`cfg_if!` arm applies, so the module is genuinely unbacked on every configuration, matching the runtime dimension's identical boundary for this shape (三儀 ⊥ 三儀)

#### Scenario: An absent path remap target inside a cfg_if arm is tolerated

- **WHEN** a `#[path = "windows_impl.rs"] mod imp;` is declared inside a `cfg_if!` arm and that target file does not exist
- **THEN** the system skips the declaration rather than reporting the remap-target-missing constitution error, the same cfg-conditional test the plain-absence outcome uses
