## ADDED Requirements

### Requirement: A dual-backed module anchor is a resolution ambiguity, not a first-form pick

A governed module anchor whose plain `mod name;` is backed by BOTH conventional source forms at once — `name.rs` AND `name/mod.rs` present together — SHALL be a **constitution error** (exit 2) naming both resolved paths and the exactly-one-file rule, and SHALL NOT resolve to either form as the anchor's source. A live declaration of this shape is a genuine rustc compile error (E0761); a declaration gated off by `#[cfg(...)]` or `#[cfg_attr(...)]` is stripped by rustc before module resolution and therefore compiles, yet SHALL still be a constitution error, because observation is cfg-blind and cannot know which arm is live — the same policy the static and runtime dimensions already apply to this shape, each having placed the ambiguity check ahead of its own absent-file cfg-tolerance (三儀 ⊥ 三儀: the same rule, not the same function). cfg-tolerance covers an *absent* conventional file and never two present ones. Silently selecting the first form probed SHALL NOT be permitted: it makes whether the anchor is governed at all depend on which of the two files an author happened to write a given item in, so an item in the unselected form escapes observation entirely — a false negative, the one outcome the core contract forbids. This resolution property SHALL hold for every single-module-anchored semantic capability (signature-coupling, visibility, dyn/impl-trait, async-exposure) rather than signature-coupling alone, SHALL hold equally for a further segment resolved beneath a dual-backed ancestor, and SHALL hold for the crate-wide and subtree walks that back nearly every semantic capability — signature-coupling's own crate-wide alias and extern scan included, alongside trait-impl locality, forbidden marker, unsafe confinement, and the subtree walks behind impl-trait and async exposure — so the anchored descent and the crate-wide walk do not disagree on the identical shape, as their absent-file policies once silently had.

#### Scenario: A dual-backed anchor is a constitution error

- **WHEN** a crate declares `pub mod child;`, both `src/child.rs` and `src/child/mod.rs` exist, and a boundary anchors at `crate::child`
- **THEN** the system emits a constitution error naming both resolved paths and exits 2 — never exit 0 and never exit 1 — rather than resolving the anchor to `src/child.rs`

#### Scenario: An exposure in the unselected form does not escape

- **WHEN** a crate declares `pub mod child;`, `src/child.rs` is clean, `src/child/mod.rs` declares `pub fn leak() -> crate::forbidden::Thing`, and a boundary anchors at `crate::child` forbidding `crate::forbidden::Thing`
- **THEN** the system exits 2 on the ambiguity and never exits 0 — the exposure written in the form that would otherwise be skipped is never silently unobserved

#### Scenario: A cfg-gated dual-backed declaration is still an ambiguity, though the crate compiles

- **WHEN** the dual-backed `mod child;` declaration carries a `#[cfg(...)]` gate whose predicate is off, so rustc strips the declaration before module resolution and the crate compiles cleanly
- **THEN** the system still emits a constitution error and exits 2 — cfg-blind observation cannot know which arm is live, and the static and runtime dimensions already refuse to judge this shape; cfg-tolerance covers an *absent* conventional file and never two present ones

#### Scenario: A dual-backed module elsewhere in the crate reacts on the crate-wide walk

- **WHEN** a dual-backed module exists anywhere in the crate and a capability whose evaluation walks the crate or a subtree is evaluated — including signature-coupling's own crate-wide alias and extern scan — while the boundary names some other module
- **THEN** that walk emits the same constitution error and exits 2, matching the static and runtime dimensions' own walkers rather than judging a crate it cannot resolve

#### Scenario: A single-form anchor is unaffected

- **WHEN** exactly one of `src/child.rs` or `src/child/mod.rs` exists and a boundary anchors at `crate::child`
- **THEN** the system resolves the anchor to that form and evaluates the boundary as before, reacting on a forbidden exposure with exit 1 and permitting a clean module with exit 0
