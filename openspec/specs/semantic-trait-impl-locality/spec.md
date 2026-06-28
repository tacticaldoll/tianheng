# semantic-trait-impl-locality Specification

## Purpose

The 渾儀 (semantic) dimension's impl-locality capability: declare in Rust that a trait may be implemented only within an allowed module location **inside the local crate** — "only `crate::commands::*` may `impl Command`". Observed via the AST (`syn`), it governs *impl locality* — the complement of exposure (`semantic-signature-coupling`) and of import (the static dimension). It governs only the crate's own impl sites; it makes no claim about downstream crates (external trait sealing is a rejected, essential-gap non-goal).

## Requirements

### Requirement: Trait-impl-locality boundary declared in Rust

A trait-impl-locality boundary SHALL be expressed as Rust code and is part of the single source of truth. Mirroring the semantic dimension's other declarations, each dimension owns its own declaration DSL and the boundaries are **composed at the gate**. A `TraitImplBoundary` SHALL name: a target crate, a governed **trait** path, an **allowed-location** set (one or more module paths/prefixes within the crate where the trait MAY be implemented), a human-readable reason, and a severity. The system MUST NOT require TOML, YAML, Markdown, or any generated policy file to declare or run a trait-impl-locality boundary.

#### Scenario: Boundary declared in Rust

- **WHEN** a developer writes `TraitImplBoundary::in_crate("app").trait_("crate::command::Command").only_implemented_in("crate::commands").because("Command impls live with the registry")`
- **THEN** a boundary is held, anchored to the trait `crate::command::Command` in crate `app`, allowing impls only under `crate::commands`, with a non-empty reason and a default `enforce` severity, ready to be composed at the gate

#### Scenario: A boundary may allow more than one location

- **WHEN** a developer writes `…trait_("crate::command::Command").only_implemented_in("crate::commands").and_in("crate::builtins").because(…)`
- **THEN** the boundary allows impls of the trait under either `crate::commands` or `crate::builtins`

### Requirement: Local trait anchor resolution

For each boundary, the system SHALL resolve the named governed trait to a real `trait` item defined in the target crate's source before evaluating it, **following local `pub use` re-export hops** so the anchor may be named at a re-export (facade) path the project actually uses, not only at the trait's defining path. If the anchor cannot be resolved to a real local `trait` — an unknown module path, a target crate absent from the workspace, or no `trait` reachable at the named path — the system SHALL treat this as a **constitution error** (exit 2), failing loud and distinct from a boundary violation (exit 1), so a mistyped trait anchor is never scanned, matched against nothing, and silently passed as clean.

#### Scenario: Anchor resolves to a real local trait

- **WHEN** a boundary anchors to `crate::command::Command` and a `trait Command` is defined in that module of the target crate
- **THEN** the system proceeds to scan the crate's impl sites for that trait

#### Scenario: Anchor named at a re-export path resolves

- **WHEN** a boundary anchors to `crate::facade::Command` where `crate::facade` declares `pub use crate::command::Command;` and `trait Command` is defined in `crate::command`
- **THEN** the system resolves the anchor through the re-export to the real local trait and proceeds to scan, rather than emitting a false constitution error

#### Scenario: An unresolvable trait anchor is a constitution error

- **WHEN** a boundary anchors to a trait path with no `trait` item reachable (directly or via local `pub use`) in the target crate's source
- **THEN** the system emits a constitution error naming the unresolved anchor and exits 2, never exit 0 (no silent pass) and never exit 1

### Requirement: Impl-site observation governs locality within the crate

The system SHALL observe **every** `impl <Trait> for <Type>` block in the target crate's own source, descending file-based (`mod x;`) and inline (`mod x { … }`) modules from the crate root while tracking each impl block's module location. For each trait `impl` whose written trait path resolves to the anchored trait, the system SHALL react when the impl block's module location is **not** contained within any allowed-location prefix. The system governs only the target crate's own impl sites and SHALL make no claim about impls in other (downstream) crates — that is an explicit out-of-scope question (external trait sealing), never silently asserted clean.

#### Scenario: An in-scope impl outside the allowed location is a violation

- **WHEN** the boundary allows `impl Command` only under `crate::commands`, and the crate defines `impl Command for Foo` in module `crate::domain`
- **THEN** the system emits a violation identifying the offending impl by its location `crate::domain` and the implemented-for type `Foo`

#### Scenario: An impl inside the allowed location is clean

- **WHEN** the boundary allows `impl Command` only under `crate::commands`, and every `impl Command for _` in the crate appears under `crate::commands` (including `crate::commands::greet`)
- **THEN** the system reports no violation for that boundary

#### Scenario: An impl in an inline module is located correctly

- **WHEN** a crate declares `mod domain { impl Command for Foo { … } }` inline and the boundary allows `impl Command` only under `crate::commands`
- **THEN** the system locates the impl at `crate::domain` and emits a violation

#### Scenario: A non-anchored trait's impl is ignored

- **WHEN** the boundary anchors the trait `Command` and the crate defines `impl Display for Foo` outside the allowed location
- **THEN** the system reports no violation, because the impl is not of the anchored trait

### Requirement: Allowed-location matching by path and prefix

An allowed location SHALL match an impl's module location either by exact path or by module prefix, where prefix containment is `::`-delimited (an exact match OR an `x::` prefix), so a sibling like `crate::commandeer` is never treated as beneath the allowed `crate::command`. A boundary MAY declare more than one allowed location, and an impl SHALL be clean if its location is contained within any one of them.

#### Scenario: A nested module beneath an allowed prefix is permitted

- **WHEN** the boundary allows `crate::commands` and an `impl Command for Foo` appears in `crate::commands::greet`
- **THEN** the system reports no violation, because the location is beneath the allowed prefix

#### Scenario: A prefix-colliding sibling is not treated as allowed

- **WHEN** the boundary allows `crate::command` and an `impl Command for Foo` appears in `crate::commandeer`
- **THEN** the system emits a violation, because `::`-delimited containment does not treat the sibling as beneath the allowed prefix

### Requirement: Trait-path resolution scope and no false negative

The system SHALL resolve the trait named at an impl site to a canonical path using the shared 渾儀 resolver: the file's in-scope `use` declarations (including renamed imports), `crate::`/`self`/`super`-relative paths (including a `use` target that is itself `self`/`super`-relative), a **bare or relative name resolved against the current module and crate root** (a same-module trait needs no `use`), and **local `pub use` re-export chains** (a trait reached through a facade path matches the anchor). A trait whose resolution would require capabilities beyond this — a glob import (`use …::*`), a macro-generated impl, a `#[path]`-remapped module, or `#[cfg]` feature evaluation — is OUT OF SCOPE, a stated coverage bound, not a claimed reaction. `#[cfg]`-gated code is observed **as written** (cfg-agnostic), and a `#[cfg]`-gated module whose source file is legitimately absent is skipped, not a scan error. Within the resolved scope there SHALL be no false negative: an impl of the anchored trait whose trait path *is* resolvable and whose location is disallowed MUST react. The system MUST NOT silently pass a disallowed impl it was able to resolve to the anchored trait.

#### Scenario: A use-imported trait path resolves and reacts

- **WHEN** a disallowed module declares `use crate::command::Command;` then `impl Command for Foo { … }`
- **THEN** the system resolves the trait to `crate::command::Command`, matches the anchor, and emits a violation

#### Scenario: A renamed trait import resolves and reacts

- **WHEN** a disallowed module declares `use crate::command::Command as Cmd;` then `impl Cmd for Foo { … }`
- **THEN** the system resolves `Cmd` to `crate::command::Command` and emits a violation

#### Scenario: A bare same-module trait name resolves and reacts

- **WHEN** the anchored `trait Command` is defined in the disallowed module `crate::domain`, which also declares `impl Command for Foo { … }` with a bare `Command` and no `use`
- **THEN** the system resolves the bare `Command` against the current module to `crate::domain::Command`, matches the anchor, and emits a violation (never a silent pass)

#### Scenario: A self/super-relative trait import resolves and reacts

- **WHEN** a disallowed module `crate::domain` declares `use super::command::Command;` then `impl Command for Foo { … }`
- **THEN** the system canonicalizes the relative `use` target against the module to `crate::command::Command`, matches the anchor, and emits a violation (never a silent pass)

#### Scenario: A re-exported trait path resolves and reacts

- **WHEN** a disallowed module declares `use crate::facade::Command;` (where `crate::facade` re-exports `crate::command::Command` via `pub use`) then `impl Command for Foo { … }`
- **THEN** the system follows the re-export chain, matches the anchor `crate::command::Command`, and emits a violation rather than silently passing

#### Scenario: A macro-generated impl is a documented coverage bound

- **WHEN** an `impl Command for Foo` is produced by a macro expansion in a disallowed module
- **THEN** the system does not claim to observe it (out of scope, the same nature as the existing macro bound), rather than silently asserting the boundary is clean

#### Scenario: A #[path]-remapped module is a documented coverage bound

- **WHEN** a disallowed impl lives in a module declared `#[path = "…"] mod x;` whose file is located off the conventional path
- **THEN** the system does not observe it (a stated coverage bound), rather than silently asserting the boundary is clean

#### Scenario: A cfg-gated module with an absent file is skipped, not a scan error

- **WHEN** the crate declares `#[cfg(feature = "x")] mod optional;` with no `optional.rs` (the feature is off)
- **THEN** the whole-crate walk skips the module (a stated coverage bound) rather than failing the gate with a scan error (exit 2)

#### Scenario: A resolvable disallowed impl is never silently passed

- **WHEN** an impl of the anchored trait is in a disallowed location and its trait path is resolvable by the shared resolver
- **THEN** the system emits a violation, never exit 0 for that boundary

### Requirement: CI reaction

The system SHALL fold trait-impl-locality findings into the same exit-code contract as the other dimensions: **exit 0** when no enforce-severity boundary is violated; **exit 1** when one or more enforce-severity boundaries are violated; **exit 2** for a constitution or scan error (an unresolvable trait anchor, or an unreadable/unparseable source file). A run that evaluates static, signature-coupling, and trait-impl-locality boundaries SHALL aggregate their findings into one report and one outcome, and a constitution error on any boundary SHALL supersede any violation in the same run.

#### Scenario: A clean boundary passes

- **WHEN** every impl of the anchored trait is within an allowed location
- **THEN** the system reports the boundary satisfied and contributes exit 0

#### Scenario: A locality violation fails CI

- **WHEN** an enforce-severity trait-impl-locality boundary is violated
- **THEN** the system prints a report and exits 1

#### Scenario: An unresolvable anchor supersedes a violation

- **WHEN** one trait-impl-locality boundary is violated and another names an unresolvable trait anchor
- **THEN** the system reports a constitution error and exits 2, not a violation (exit 1)

### Requirement: Severity and baseline parity

A trait-impl-locality boundary SHALL carry a severity (`enforce` by default, or `warn`) with the same meaning as other boundaries: a `warn` violation is reported but does not by itself fail the reaction. Its violations SHALL be gated against the same `Baseline` mechanism, sharing the violation identity `(target, rule, finding)` — where the finding identifies the offending impl by its module location and implemented-for type. The rule is a fixed string; the allowed-location set is policy configuration, not part of the violation identity, so editing the allowed set does not turn a still-misplaced impl into a new violation. A project may thus adopt the boundary on a dirty codebase and gate only on new misplaced impls.

#### Scenario: A warn boundary reports without failing

- **WHEN** a `warn`-severity trait-impl-locality boundary is violated and no enforce-severity boundary is violated
- **THEN** the system reports the violation but the reaction does not fail (exit 0)

#### Scenario: A baselined locality violation does not fail

- **WHEN** an enforce-severity boundary's only violations are all present in the baseline
- **THEN** the system reports them as accepted and does not fail the reaction

#### Scenario: A new locality violation beyond the baseline fails

- **WHEN** an enforce-severity boundary has a misplaced impl not present in the baseline
- **THEN** the system fails the reaction (exit 1) for that new violation

### Requirement: Human-readable violation report

A trait-impl-locality violation report SHALL identify the governed trait anchor, the rule (that the trait may only be implemented in the declared location(s)), the offending impl (its module location and implemented-for type — the finding), and the human-readable reason supplied with the boundary, and SHALL state that the reaction failed — the same report contract as the other boundaries.

#### Scenario: Report explains the misplaced impl

- **WHEN** the crate `app` defines `impl Command for Foo` in `crate::domain` under a boundary allowing only `crate::commands`
- **THEN** the report names the trait anchor `crate::command::Command`, the rule (that it may only be implemented in the declared location(s)), the finding identifying `crate::domain` / `Foo`, the boundary's reason, and indicates CI failed (the allowed locations themselves are surfaced in the `list` projection and the reason, not embedded in the rule identity)
