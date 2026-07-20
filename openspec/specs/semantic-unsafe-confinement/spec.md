# semantic-unsafe-confinement Specification

## Purpose

The 渾儀 (semantic) dimension's `unsafe`-confinement capability: declare in Rust that a crate's `unsafe` (blocks, `unsafe fn`/`impl`/`trait`, `unsafe extern`) may appear **only under** a declared subtree — the auditability boundary of a layered crate ("all `unsafe` lives behind `crate::ffi`"). It governs *where* `unsafe` lives (architectural intent), not *whether* it may exist: the crate-wide "no `unsafe`" case is `#![forbid(unsafe_code)]`'s (compile-time, stronger), so an empty or crate-root allowed set is a constitution error. Observed via the AST (`syn`), a whole-crate scan of the forbidden-marker family. This is confinement, the non-compiler-expressible complement of the attribute.

## Requirements

### Requirement: Unsafe confinement declared in Rust

An unsafe-confinement boundary SHALL be expressed as Rust code and is part of the single source of truth. An `UnsafeBoundary` SHALL name a target crate, one or more **allowed subtree** module paths via `only_under([...])`, a human-readable reason, and a severity. The rule confines `unsafe` to the allowed subtree(s): a `unsafe` site outside all of them reacts. It governs *where* `unsafe` may live, never *whether* it may exist. The system MUST NOT require TOML, YAML, Markdown, or any generated policy file.

The confinement-only scope SHALL be enforced as a constitution error (exit 2), never a silent degeneracy:

- An **empty** allowed set (`only_under([])`) SHALL be a constitution error whose message directs the adopter to `#![forbid(unsafe_code)]` — a crate-wide "no `unsafe`" is the compiler's stronger job, not this rule's.
- An allowed set naming the **crate root** (`crate`) SHALL be a constitution error — `unsafe` would be permitted everywhere (the rule could never react).

#### Scenario: Boundary declared in Rust

- **WHEN** a developer writes `UnsafeBoundary::in_crate("app").only_under(["crate::ffi"]).because("unsafe lives only behind the ffi module")`
- **THEN** a boundary is held, confining `unsafe` in crate `app` to the subtree `crate::ffi`, with a non-empty reason and a default `enforce` severity

#### Scenario: Empty allowed set is a constitution error

- **WHEN** a boundary declares `only_under([])`
- **THEN** the system emits a constitution error (exit 2) pointing at `#![forbid(unsafe_code)]`, never treating it as a silent no-op or a crate-wide reaction

#### Scenario: Crate-root allowed set is a constitution error

- **WHEN** a boundary declares `only_under(["crate"])`
- **THEN** the system emits a constitution error (exit 2), because `unsafe` would be permitted in the whole crate and the rule could never react

### Requirement: Unsafe-site observation

The system SHALL walk the whole target crate (descending file-based `mod x;` and inline `mod x { … }` alike) and observe every `unsafe` **site**, attributing each to its enclosing module. The observed sites SHALL be: an `unsafe fn` (free function, inherent method, trait method declaration, or trait-impl method), an `unsafe impl`, an `unsafe trait`, an `unsafe extern` block (the `unsafe` keyword form), and an `unsafe {}` expression block (observed within item bodies, including bodies of `const`/`static` initializers, closures, and nested functions). A `mod` declared **inside a function or block body** (which the top-level module walk does not descend) SHALL still be observed — its `unsafe` attributed to the enclosing file module — so no body-nested `unsafe` is silently dropped. A site SHALL react iff its enclosing module is **not under** any allowed subtree (a module equal to or beneath an allowed subtree passes). Within the observed source there SHALL be no false negative: an observed `unsafe` site outside every allowed subtree MUST react.

#### Scenario: An unsafe block outside the subtree is a violation

- **WHEN** the crate has `only_under(["crate::ffi"])` and a function in `crate::net` contains an `unsafe { … }` block
- **THEN** the system emits a violation naming the module `crate::net` and the `unsafe block`

#### Scenario: An unsafe fn / impl / trait outside the subtree is a violation

- **WHEN** the crate has `only_under(["crate::ffi"])` and `crate::net` declares `unsafe fn decode()`, an `unsafe impl` block, or an `unsafe trait`
- **THEN** the system emits a violation for each, named by kind and (where present) name, qualified by `crate::net`

#### Scenario: Two unsafe impls that differ in trait or self type stay distinct

- **WHEN** `crate::net` (outside the subtree) declares `unsafe impl Send for Foo {}` alongside either `unsafe impl Sync for Foo {}` (a different trait) or `unsafe impl Send for Bar {}` (the same trait, a different self type)
- **THEN** the system emits two distinct findings (both the trait **and** the self type are part of the finding), so neither masks the other under the baseline

#### Scenario: Two same-named unsafe fns on different owners stay distinct

- **WHEN** `crate::net` (outside the subtree) declares `impl Foo { unsafe fn m(&self) {} }` alongside `impl Bar { unsafe fn m(&self) {} }` (the same method name, different owners), or two traits each declaring `unsafe fn m`
- **THEN** the system emits two distinct findings — each `unsafe fn` finding is qualified by its enclosing owner (`unsafe fn Foo::m`, the inherent-impl self type, or `unsafe fn A::m`, the declaring trait) — so neither masks the other under the baseline, the `unsafe fn` counterpart of the `unsafe impl` distinctness above

#### Scenario: Unsafe in a body-nested module is attributed to the enclosing module

- **WHEN** the crate has `only_under(["crate::ffi"])` and a function in `crate::net` declares `mod raw { pub unsafe fn poke() {} }` (a `mod` inside a fn body)
- **THEN** the system emits a violation for the `unsafe fn`, attributed to `crate::net`, never silently dropping it because the top-level walk did not descend a body-nested `mod`

#### Scenario: Unsafe under the allowed subtree is clean

- **WHEN** the crate has `only_under(["crate::ffi"])` and all `unsafe` (blocks, `fn`, `impl`, `trait`, `extern`) sits in `crate::ffi` or a submodule beneath it
- **THEN** the system reports no violation

#### Scenario: An observed unsafe site is never silently passed

- **WHEN** an `unsafe` site the scan observes lies outside every allowed subtree
- **THEN** the system emits a violation, never exit 0 for that boundary

### Requirement: Crate and subtree resolution

For each boundary the system SHALL resolve the target crate to a workspace member and its source root before evaluating it. A target crate absent from the workspace, an unreadable/unparseable source file, or a non-`#[cfg]` missing module file encountered during the walk SHALL be a **constitution error** (exit 2), failing loud and distinct from a violation (exit 1), so an ungovernable target is never reported as an unsafe violation and never silently passed. A symlink module cycle SHALL be a constitution error, never a crash.

#### Scenario: Unknown crate is a constitution error

- **WHEN** a boundary targets a crate that is not a member of the workspace
- **THEN** the system emits a constitution error (exit 2), never exit 0 or exit 1

### Requirement: Observation bounds and scope

The rule SHALL observe the executable-`unsafe` **code sites** (blocks, `fn`, `impl`, `trait`, `unsafe extern`); other lexical `unsafe` tokens and non-source `unsafe` SHALL be **stated bounds, never a silent claim of safety**:

- **Peripheral `unsafe` keywords, out of scope by design:** an `unsafe(...)` **attribute** (`#[unsafe(no_mangle)]`, Rust 2024 — a linkage assertion, not a code region), a bare **`unsafe fn` pointer type** (`type H = unsafe fn(...)` — a type signature, not an execution), and a **plain `extern "C" { … }` block** carrying no `unsafe` keyword (only the `unsafe extern {}` form is a site; the plain block's foreign-fn *call sites* are `unsafe {}` and DO react). The rule confines executable-`unsafe` code sites, not every lexical `unsafe` token.
- **Incidental bounds** (the dimension's inherited whole-crate-scan bounds): `unsafe` produced by a macro expansion or inside an unexpanded macro body is not observed; a module reached through a `#[path]` remap is not observed; a `#[cfg]`-gated module absent when its feature is off is tolerated, while cfg-present code is observed **as written** (cfg-blind); a distinct `[lib] name` is a bound.

The system makes no claim about `unsafe` outside these observed sites.

#### Scenario: Macro-generated unsafe is a documented bound

- **WHEN** an `unsafe` block is produced by a macro expansion in a module outside the allowed subtree
- **THEN** the system does not claim to observe it (out of scope, the dimension's macro bound), rather than silently asserting the module is unsafe-free

### Requirement: CI reaction

The system SHALL fold unsafe-confinement findings into the same exit-code contract as the other dimensions: **exit 0** when no enforce-severity boundary is violated; **exit 1** when one or more enforce-severity boundaries are violated; **exit 2** for a constitution or scan error. A run aggregating static and semantic boundaries SHALL produce one report and one outcome, and a constitution error on any boundary SHALL supersede any violation in the same run.

#### Scenario: A clean boundary passes

- **WHEN** all `unsafe` in the crate sits under the allowed subtree(s)
- **THEN** the system reports the boundary satisfied and contributes exit 0

#### Scenario: An unsafe-confinement violation fails CI

- **WHEN** an enforce-severity unsafe-confinement boundary is violated
- **THEN** the system prints a report and exits 1

### Requirement: Severity and baseline parity

An unsafe-confinement boundary SHALL carry a severity (`enforce` by default, or `warn`) with the same meaning as other boundaries, and its violations SHALL gate against the same `Baseline` mechanism, sharing the violation identity `(target, rule, finding_key)` — the `target` being the confined crate, the `rule` a fixed string, and the `finding` naming the offending site qualified by its module — so a project may adopt the boundary on a crate with existing `unsafe` and gate only on new sites. An anonymous `unsafe {}` block SHALL be identified per-module (not by a fragile per-block ordinal), so multiple blocks in one module dedup to one stable finding.

#### Scenario: A warn boundary reports without failing

- **WHEN** a `warn`-severity unsafe-confinement boundary is violated and no enforce-severity boundary is violated
- **THEN** the system reports the violation but the reaction does not fail (exit 0)

#### Scenario: A baselined unsafe site does not fail

- **WHEN** an enforce-severity boundary's only violations are all present in the baseline
- **THEN** the system reports them as accepted and does not fail the reaction

### Requirement: Human-readable violation report

An unsafe-confinement violation report SHALL identify the confined crate, the rule (that `unsafe` is confined to the declared subtree(s), naming them), the offending site and its module (the finding), the boundary's reason, and SHALL indicate the reaction failed — the same report contract as the other boundaries.

#### Scenario: Report explains the offending site

- **WHEN** crate `app` confines `unsafe` to `crate::ffi` and `crate::net` contains an `unsafe` block
- **THEN** the report names the crate `app`, the rule (confined to `crate::ffi`), the finding (`unsafe block in crate::net`), the reason, and indicates CI failed
