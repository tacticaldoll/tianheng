# semantic-dyn-trait-operand-boundary Specification

## Purpose
The 渾儀 (semantic) capability that governs **operand-scoped `dyn` exposure**: a module's public
API must not expose a trait object (`dyn`) of a **named trait**. It is the named-operand depth of
the shape-only `semantic-dyn-trait-boundary` — where that forbids *any* exposed `dyn`, this forbids
only a `dyn` whose **principal trait** resolves into a declared forbidden set (a core seam may
expose `dyn std::error::Error` freely while never leaking `dyn crate::Port`). It reuses the
shape-only public-surface `dyn` walk and signature-coupling's resolver (resolve + re-export
canonicalization + exact-or-module-prefix match), adding only the operand match; same `syn`
observation source, no new crate.

## Requirements

### Requirement: Operand-scoped dyn boundary declared in Rust

An operand-scoped dyn boundary SHALL be expressed as Rust code on a `DynTraitBoundary`, part of
the single source of truth, declared on the 渾儀 dimension alongside the shape-only dyn-trait and
signature-coupling boundaries and composed with the other dimensions at the gate. It SHALL name a
target crate and a module anchor and a closed set of **forbidden trait operands** via
`must_not_expose_dyn_of([...])`, a human-readable reason, and a severity. A `dyn` node in the
governed module's public surface whose **principal trait** canonicalizes to a member of the set is
a violation. The shape-only `must_not_expose_dyn()` is unchanged and reacts to any `dyn`; the
operand variant is a distinct, narrower rule on the same boundary type. The system MUST NOT require
TOML, YAML, Markdown, or any generated policy file to declare or run the boundary.

#### Scenario: Operand-scoped boundary declared in Rust

- **WHEN** a developer writes `DynTraitBoundary::in_crate("core").module("crate::core").must_not_expose_dyn_of(["crate::ports::Port"]).because("the core seam must not leak a dyn Port")`
- **THEN** a dyn-trait boundary is held, targeting `crate::core`, forbidding the exposure of a `dyn` whose principal trait is `crate::ports::Port`, with a non-empty reason and a default `enforce` severity, ready to be composed with the semantic dimension at the gate

### Requirement: A dyn of a forbidden trait operand is a violation

The system SHALL emit a violation for each `dyn` node in the governed module's public surface whose
principal trait canonicalizes to a member of the forbidden operand set, and SHALL report no
violation for a `dyn` whose principal trait is outside the set. The **principal trait** is the
first trait bound of the trait object — Rust's grammar guarantees the base trait is syntactically
first, so any auto-trait (`Send`, `Sync`) or lifetime bound can only follow it and is never the
matched operand. The principal trait path SHALL be canonicalized and matched exactly as
signature-coupling matches a forbidden type (resolved against the module's `use` map and re-export
closure via `BareFallback::Ignore`, then compared exact-or-module-prefix), so a re-exported or
aliased trait facade matches its defining path; a principal trait that does not resolve (a bare
name with no `use`, a macro-generated or glob/cross-crate re-exported trait) is dropped — the same
stated resolver-coverage bound signature-coupling carries, never a silent pass of a *resolvable*
operand. The finding is the rendered `dyn …` shape, matching the shape-only rule.

#### Scenario: A dyn of a named forbidden trait is flagged

- **WHEN** the governed module's public API exposes `Box<dyn crate::ports::Port>` and the boundary forbids `["crate::ports::Port"]`
- **THEN** the system emits a violation whose finding is the rendered shape `dyn crate::ports::Port`, because the principal trait is in the forbidden operand set

#### Scenario: A dyn of an unlisted trait passes

- **WHEN** the governed module's public API exposes `Box<dyn std::error::Error>` and the boundary forbids only `["crate::ports::Port"]`
- **THEN** the system reports no violation, because the principal trait `std::error::Error` is outside the forbidden operand set

#### Scenario: A module-prefix operand forbids a subtree of traits

- **WHEN** the boundary forbids `["crate::ports"]` (a module prefix) and the module exposes `dyn crate::ports::Port`
- **THEN** the system emits a violation, because the principal trait canonicalizes under the forbidden prefix — the same exact-or-prefix match the sibling forbidden-type rule uses

#### Scenario: A re-exported trait operand matches its defining path

- **WHEN** the module exposes `dyn crate::Port`, a `pub use crate::ports::Port` facade of the trait defined at `crate::ports::Port`, and the boundary forbids the defining path `["crate::ports::Port"]`
- **THEN** the system emits a violation, because the exposed facade canonicalizes through the re-export closure to the same defining path — closing the re-export false negative

#### Scenario: Auto-trait markers are not operands

- **WHEN** the module exposes `dyn crate::ports::Port + Send` and the boundary forbids `["crate::ports::Port"]`
- **THEN** the system emits a violation on the principal trait `crate::ports::Port` (the first trait bound); the trailing `Send` marker is not the operand, so a boundary forbidding only `["Send"]` flags nothing here — and against a bare `dyn Send`, `Send` does not resolve under `BareFallback::Ignore` and is likewise dropped

### Requirement: Empty operand set degenerates to shape-only, never a silent no-op

The system SHALL treat an **empty** forbidden operand set as "no operand filter — any `dyn` in the
governed surface is a violation" (the shape-only behavior). `must_not_expose_dyn()` constructs the
empty set; `must_not_expose_dyn_of([])` therefore reacts to any `dyn` as well — a loud
over-reaction, never a boundary that reacts to nothing. The system MUST NOT model an operand-scoped
boundary that silently passes every `dyn`.

#### Scenario: An empty operand list forbids any dyn

- **WHEN** a boundary is declared with `must_not_expose_dyn_of([])` and the module exposes any `dyn`
- **THEN** the system emits a violation for that `dyn`, identical to the shape-only `must_not_expose_dyn()` reaction — the empty set is unfiltered, not an inert no-op

### Requirement: Reaction, severity, baseline, and projection parity with the shape-only rule

The operand-scoped dyn boundary SHALL share the 渾儀 dyn-trait reaction contract: findings fold
into the same aggregated report and exit-code outcome (**0** clean, **1** enforce violation, **2**
constitution/scan error such as an unresolvable crate or module); the boundary carries a severity
(`enforce` default, or `warn`) and is gated against the same `Baseline` under the shared violation
identity `(target, rule, finding)`, the finding being the rendered `dyn …` shape; and the rule
projects through the existing dyn-trait `list` text/JSON/markdown projection, adding a `forbidden`
parameter listing the operand set when non-empty (a shape-only, empty-set boundary projects
unchanged). The implementation SHALL keep the `syn` dependency quarantined in `hunyi` (no new
dependency) and SHALL NOT change the public-surface walk.

#### Scenario: An operand violation fails CI

- **WHEN** an enforce-severity operand-scoped boundary is violated
- **THEN** the system prints a report naming the target module, the rule, the offending `dyn` shape, and the reason, and exits 1

#### Scenario: An unresolvable target module is a constitution error

- **WHEN** an operand-scoped boundary anchors to a crate or module not present in the workspace
- **THEN** the system emits a constitution/scan error and exits 2, never exit 0 and never exit 1

#### Scenario: Severity and baseline behave as for the shape-only dyn rule

- **WHEN** a `warn`-severity operand boundary is violated and no enforce boundary is, or an enforce operand boundary's only violations are all in the baseline
- **THEN** the reaction does not fail (exit 0); and an operand violation not present in the baseline fails the reaction (exit 1)

#### Scenario: The operand set projects in list output

- **WHEN** the constitution is projected via `list` (text/json/markdown)
- **THEN** an operand-scoped boundary appears with its target, module, rule, the forbidden operand set, severity, and reason — through the existing dyn-trait projection, no separate projector; a shape-only boundary appears exactly as before, with no operand parameter
