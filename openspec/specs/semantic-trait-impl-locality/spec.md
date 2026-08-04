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

The **resolved** anchor, not the declared spelling, SHALL be the violation's governed `target` and the
trait role of its rule key. Matching already resolves both sides — the declared anchor through this
crate's re-export closure, and each impl site's own trait path — so keeping the raw declaration in
identity alone made a pure declaration refactor identity-changing: renaming a boundary from a facade
spelling to the trait's defining path, with no code change and the same impls still misplaced, gave
every affected violation a new `ViolationId`, so each accepted violation re-fired as new while its
recorded baseline entry reported stale. Two equivalent spellings of one trait SHALL therefore produce
one identity.

Where the declared anchor's own re-export closure reaches **more than one distinct local trait
definition** — two mutually-exclusive `#[cfg]` branches re-exporting different traits under one facade
name — the system SHALL emit a constitution error naming the candidates, and SHALL NOT choose one:
the ambiguity is in the declaration, and choosing would make the governed target arbitrary. The error
SHALL say that the defining path can be declared instead of the facade.

The rule key SHALL retain the declared allowed-location set. That is a deliberate trade, not an
oversight: it is what keeps two boundaries governing the same trait with different allowed sets from
producing one identity for one misplaced impl, which would let a baseline accepting the first suppress
the second's never-accepted violation. Its cost — editing the allowed set re-fires still-misplaced
impls as new and reports the previous entries stale — SHALL be stated rather than denied, and is the
loud direction of the two.

#### Scenario: Two equivalent trait spellings produce one violation identity

- **WHEN** one boundary anchors at a facade `pub use` path and another anchors at the same trait's defining path, over the same crate with the same misplaced impl
- **THEN** both produce the identical `ViolationId` — target and rule key both keyed on the resolved defining path — so a baseline recorded under either declaration matches the other

#### Scenario: A trait facade reaching two definitions is a constitution error

- **WHEN** a boundary anchors at a facade name that two mutually-exclusive `#[cfg]` branches re-export from different traits, both defined locally
- **THEN** the system emits a constitution error naming both candidates and exits 2, rather than picking one as the governed target

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

### Requirement: A malformed `::`-path allowed-location entry is a constitution error

An allowed-location entry given to `only_implemented_in`/`and_in` SHALL be rejected as a
**constitution error** (exit 2) when its `::`-delimited spelling has any empty segment — a leading
`::`, a trailing `::`, a doubled `::`, or the empty string itself — checked before any crate
scanning. This is the identical restriction `semantic-signature-coupling`'s "A malformed `::`-path
forbidden operand is a constitution error" requirement already places on `must_not_expose`'s
operand, read at the allowed-location polarity: `matches_allowed`'s `::`-delimited containment
(equality or a `prefix::`-boundary match) can never equal or prefix-contain a real module location
against an operand shaped this way, so without this requirement a malformed entry would not
silently pass the boundary — the containment check already fails loud, since a location outside
every (non-matching) allowed entry is reported as a violation — but it would silently misreport
every genuinely-in-place impl as a spurious violation, naming no cause, rather than a clear
constitution error identifying the actual typo. There is no legitimate reason to write this shape:
no canonical module path this system ever resolves carries an empty segment, so the spelling is
always either inert or broken, never meaningfully different from the bare form.

#### Scenario: A leading-`::` allowed-location entry is a constitution error

- **WHEN** a boundary declares `only_implemented_in("::crate::commands")` and the crate defines `impl Command for Foo` genuinely inside `crate::commands`
- **THEN** the system reports a constitution error (exit 2) naming the malformed entry, rather than reporting the genuinely-in-place impl as a spurious violation

#### Scenario: A trailing-`::` allowed-location entry is a constitution error

- **WHEN** a boundary declares `only_implemented_in("crate::commands::")` against the same crate
- **THEN** the system reports a constitution error (exit 2), for the identical reason

#### Scenario: A doubled-`::` allowed-location entry is a constitution error

- **WHEN** a boundary declares `only_implemented_in("crate::commands::::sub")` against the same crate
- **THEN** the system reports a constitution error (exit 2), for the identical reason

#### Scenario: The bare-string spelling is unaffected

- **WHEN** a boundary declares `only_implemented_in("crate::commands")` against the same crate
- **THEN** the system reports no violation for the genuinely-in-place impl, exactly as before this requirement existed

### Requirement: Trait-path resolution scope and no false negative

The system SHALL resolve the trait named at an impl site to a canonical path using the shared 渾儀 resolver: the file's in-scope `use` declarations (including renamed imports), `crate::`/`self`/`super`-relative paths (including a `use` target that is itself `self`/`super`-relative), a **bare or relative name resolved against the current module and crate root** (a same-module trait needs no `use`), and **local `pub use` re-export chains** (a trait reached through a facade path matches the anchor). A trait whose resolution would require capabilities beyond this — a glob import (`use …::*`), a macro-generated impl, or `#[cfg]` feature evaluation — is OUT OF SCOPE, a stated coverage bound, not a claimed reaction. `#[cfg]`-gated code is observed **as written** (cfg-agnostic), and a `#[cfg]`-gated module whose source file is legitimately absent is skipped, not a scan error. A module reached only through a `cfg_attr`-wrapped `#[path]` remap is followed too: an inline body regardless of the attribute (which has no effect on an inline module's content), and a file module's conventional file and its `cfg_attr` target both read when they exist on disk, cfg-blind union rather than a skip bound. Within the resolved scope there SHALL be no false negative: an impl of the anchored trait whose trait path *is* resolvable and whose location is disallowed MUST react. The system MUST NOT silently pass a disallowed impl it was able to resolve to the anchored trait. When a `use`-map name involved in resolution — on either the boundary's own declared anchor (reached through its re-export facade) or an impl site's written trait path — resolves to **more than one** candidate because of a mutually-exclusive `#[cfg]`-gated `use` alias for the identical local name, every candidate SHALL be checked, and the anchor match SHALL react if any impl-site candidate canonicalizes to any declared-anchor candidate, never silently keeping only the candidate from whichever declaration was written last (observation cannot know which `#[cfg]` branch is live).

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

#### Scenario: An unconditional #[path]-remapped module is followed and its disallowed impl reacts

- **WHEN** a disallowed impl lives in a module declared `#[path = "…"] mod x;` (an unconditional remap) whose file is located off the conventional path
- **THEN** the system follows the remap to that file and reacts on the impl, attributed to the module's declared path `crate::…::x`, rather than silently asserting the boundary is clean

#### Scenario: A cfg_attr-remapped module's target is followed when the conventional file is absent

- **WHEN** a disallowed impl lives in a module declared `#[cfg_attr(<pred>, path = "weird.rs")] mod domain;` with no conventional `domain.rs` present, and `weird.rs` (the `cfg_attr` target) exists and contains the impl
- **THEN** the system reads `weird.rs` — the file every build actually compiles here — and reacts on the impl, attributed to the module's declared path `crate::domain`, rather than treating the `cfg_attr` attribute as an out-of-scope bound; a nested `#[cfg_attr(a, cfg_attr(b, path = "…"))]` is followed the identical way

#### Scenario: Two cfg-siblings declaring the identical name backed by one real file are one finding

- **WHEN** the crate declares the SAME module name under two mutually-exclusive `#[cfg]` arms with no `#[path]` (so both resolve to the identical real file), and that file contains a single disallowed impl whose self-type carries an unrenderable generic argument
- **THEN** the system reports exactly one violation, never two — the two `#[cfg]` arms name the same real, once-compiled file, and must not be treated as two independent scan sites merely because they were declared twice

#### Scenario: A blanket impl's own generic parameter never dedup-collapses with a genuine impl on the aliased type

- **WHEN** a disallowed module declares a blanket `impl<T> Trait for T {}` (`T` is the impl's own generic parameter) alongside an unrelated `use SomeType as T;` naming a real crate-defined type, AND a genuine direct `impl Trait for SomeType {}` in that same module
- **THEN** the system reports TWO distinct violations, not one — the blanket impl's own `T` is never resolved through the same-named alias to produce an owner identical to the direct impl's, which would otherwise let the two impl sites' findings collapse under exact-identity dedup and silently drop one genuine violation

#### Scenario: A cfg-gated module with an absent file is skipped, not a scan error

- **WHEN** the crate declares `#[cfg(feature = "x")] mod optional;` with no `optional.rs` (the feature is off)
- **THEN** the whole-crate walk skips the module (a stated coverage bound) rather than failing the gate with a scan error (exit 2)

#### Scenario: A resolvable disallowed impl is never silently passed

- **WHEN** an impl of the anchored trait is in a disallowed location and its trait path is resolvable by the shared resolver
- **THEN** the system emits a violation, never exit 0 for that boundary

#### Scenario: A mutually-exclusive cfg-gated use alias for the anchored trait's name reacts regardless of order

- **WHEN** a disallowed module declares `#[cfg(unix)] use crate::command::Command as T; #[cfg(not(unix))] use crate::other::Other as T;` then `impl T for Foo { … }`, under a boundary anchored to `crate::command::Command`, in either declaration order
- **THEN** the system emits a violation, regardless of which `use` line is written first — every candidate the impl site's `T` could resolve to is checked against the anchor, so the verdict never depends on source order

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

A trait-impl-locality boundary SHALL carry a severity (`enforce` by default, or `warn`) with the same meaning as other boundaries: a `warn` violation is reported but does not by itself fail the reaction. Its violations SHALL be gated against the same `Baseline` mechanism, sharing the violation identity `(target, rule_key, fact)` — where the finding identifies the offending impl by its module location and implemented-for type. The rule is a fixed string; the allowed-location set is policy configuration, not part of the violation identity, so editing the allowed set does not turn a still-misplaced impl into a new violation. A project may thus adopt the boundary on a dirty codebase and gate only on new misplaced impls.

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

A trait-impl-locality violation report SHALL identify the governed trait anchor, the rule ("must only be implemented in the declared location(s)"), the offending impl (its module location and implemented-for type — the finding), and the human-readable reason supplied with the boundary, and SHALL state that the reaction failed — the same report contract as the other boundaries.

#### Scenario: Report explains the misplaced impl

- **WHEN** the crate `app` defines `impl Command for Foo` in `crate::domain` under a boundary allowing only `crate::commands`
- **THEN** the report names the trait anchor `crate::command::Command`, the rule "must only be implemented in the declared location(s)", the finding identifying `crate::domain` / `Foo`, the boundary's reason, and indicates CI failed (the allowed locations themselves are surfaced in the `list` projection and the reason, not embedded in the rule identity)

### Requirement: Trait-impl locality uses structured law and fact roles

Locality violations SHALL encode the governed target, a structured locality rule key, and a fact
containing impl module, trait, and canonical self type. Rule configuration SHALL be canonically
classified as identity-bearing or presentation-only; rendered impl text SHALL NOT define identity.

#### Scenario: Two impls stay distinct
- **WHEN** two misplaced impls differ by module, trait, or self type
- **THEN** their structured facts differ in the corresponding role

### Requirement: An impl nested in a const or fn body is observed

The system SHALL observe a trait `impl` block that is written as a direct statement of the outermost body of a `const` initializer (a bare `{ … }` block expression) or of a `fn`'s own body — the "const-eval trick" idiom (`const _: () = { impl Trait for Type { … } };`, commonly used for a compile-time trait assertion or a doctest/dogfooding scratch impl) and its fn-body-nested sibling (`fn _also() { impl Trait for Type { … } }`) — exactly as if it were written at the enclosing module's own top level. Rust binds a trait `impl` to its self type's coherence set regardless of where the `impl` is lexically written, so wrapping it in a body does not change what it makes real; a walker that stops at a module's own top-level items therefore has a genuine observation gap here, distinct from the correct treatment of a body-nested `mod` (whose contents genuinely are unreachable as `crate::…`, an existing bound this requirement does not disturb). Recovery is bounded to exactly this shape: only an `impl` that is a DIRECT statement of the `const`/`fn`'s own outermost block is recovered — an `impl` nested one level FURTHER inside that body (inside an `if`/`loop`/closure/nested `fn`) is NOT recovered, and a `static` initializer is NOT inspected (the const-eval trick is specifically about `const`, which forces compile-time evaluation even when the binding is never read; no audited idiom uses `static` for it). Both bounds are stated rather than left silent.

#### Scenario: A const-wrapped disallowed trait impl reacts

- **WHEN** the boundary allows `impl Command` only under `crate::commands`, and `crate::rogue` declares `pub struct Rogue; const _: () = { impl Command for Rogue { fn run(&self) {} } };`
- **THEN** the system emits a violation identifying the offending impl by its location `crate::rogue` and the implemented-for type `Rogue`, rather than reporting zero findings because the impl sits inside a const initializer

#### Scenario: A fn-body-wrapped disallowed trait impl reacts

- **WHEN** the boundary allows `impl Command` only under `crate::commands`, and `crate::rogue` declares `pub struct Rogue2; fn _also() { impl Command for Rogue2 { fn run(&self) {} } }`
- **THEN** the system emits a violation identifying the offending impl by its location `crate::rogue` and the implemented-for type `Rogue2`, rather than reporting zero findings because the impl sits inside a fn body

#### Scenario: An impl nested one level further inside the body is a stated bound

- **WHEN** a disallowed module declares `fn _also() { if true { impl Command for Foo { fn run(&self) {} } } }`
- **THEN** the system does not claim to observe it — recovery covers only a direct statement of the const/fn's own outermost block, and this impl is one level further in, a stated coverage bound rather than a silent claim of cleanliness

#### Scenario: A static-wrapped impl is a stated bound

- **WHEN** a disallowed module declares `static S: () = { impl Command for Foo { fn run(&self) {} } };`
- **THEN** the system does not claim to observe it — only a `const` initializer or a `fn` body is inspected, never a `static` initializer, a stated coverage bound rather than a silent claim of cleanliness
