# runtime-origin-assertion Specification

## Purpose

The 漏刻 (runtime) dimension's first capability: declare which concrete-type **origins** may cross a named runtime **seam**, and probe live `dyn` objects in production to catch a forbidden-origin type slipping through a `dyn Trait` into a layer it must not reach — what static and semantic analysis structurally cannot see. It has two faces: a **prod face** (the probe reacts fail-closed, emitting a `Violation` event by default, panic opt-in) and a **CI face** (`audit_probe_coverage` verifies every declared seam is probed and every probe references a declared seam). Origin is **observed** (`module_path!()` at the registration site), not self-asserted; the hot path is std-only and lock-free; the crate depends on 璇璣 (`xuanji`) only.

## Requirements

### Requirement: Runtime boundary declared in Rust and installed write-once

A runtime boundary SHALL be expressed as Rust code and is part of the single source of truth. A `RuntimeBoundary` SHALL name a runtime **seam** (a string), an **allowlist of origins**, and a reaction posture. Boundaries SHALL be installed once at startup into a process-global **write-once** registry; a second install SHALL be a constitution error (the registry is read-only after startup so the hot path needs no lock). A probe references a seam **by name**, so policy lives in the declaration, not at the call site. The system MUST NOT require TOML, YAML, Markdown, or any generated policy file. Within a single install, a **duplicate seam declaration** or a **duplicate origin registration** (the same type registered twice) SHALL fail loud, never silently overwrite — a silent overwrite would let the last declaration shadow an earlier law (a declared boundary that never enforces, the forbidden false negative).

#### Scenario: Boundary declared and installed

- **WHEN** a developer writes `louke::install([RuntimeBoundary::at("domain-entry").only_origins(["app::domain"])])` at startup
- **THEN** the seam `domain-entry` is registered allowing origin `app::domain`, ready for probes to reference by name

#### Scenario: Re-install is a constitution error

- **WHEN** `install` is called a second time after startup
- **THEN** the system fails loud (a constitution error), never silently replacing or merging the write-once registry

#### Scenario: A duplicate seam or origin in one install fails loud

- **WHEN** `install` is given two `RuntimeBoundary` objects naming the same seam, or two origin registrations for the same type
- **THEN** the system fails loud (it panics with a constitution-error-style message), never silently keeping only the last — a silent overwrite would shadow the earlier law

### Requirement: Origin is observed, not self-asserted

A concrete type SHALL opt into an origin via a `macro_rules!` (no proc-macro, no `syn`) that captures `module_path!()` at the registration site as the origin — so the origin is **where the type is registered**, an observed location, not a free self-asserted label. Because std has no pre-`main` hook, registration SHALL be performed by an explicit startup call (the macro yields an entry the startup installs); a type that is never registered has no known origin. Observing the concrete type behind a `dyn Trait` requires the governed trait to carry a `louke::Tracked` supertrait (rust-1.85-compatible; no trait upcasting), and the concrete type to be `'static`.

#### Scenario: A type's origin is its registration location

- **WHEN** `register_origin!(PostgresRepo)` is written in module `app::infra` and installed at startup
- **THEN** the origin registry maps `TypeId::of::<PostgresRepo>()` to the observed origin `app::infra`, which the type cannot claim falsely without physically registering elsewhere

### Requirement: Seam probe observes the live object's concrete origin

A probe at a seam SHALL read the **concrete** origin of a live `dyn` object crossing it — obtaining the concrete `TypeId` via the `louke::Tracked` supertrait's `as_any()` (no trait upcasting), resolving it through the origin registry — and compare it to the named seam's allowlist. It observes the concrete type behind a `dyn Trait`, which the static and semantic dimensions structurally cannot.

#### Scenario: A probe reads a crossing object's origin

- **WHEN** `assert_boundary!("domain-entry", obj)` runs where `obj: &dyn DomainPort` (`DomainPort: louke::Tracked`) whose concrete type registered origin `app::infra`
- **THEN** the system resolves the object's origin to `app::infra` and compares it against the `domain-entry` allowlist

### Requirement: Fail-closed allowlist matching

The allowlist SHALL be matched fail-closed: an origin **in** the allowlist passes; an origin **not in** it reacts; and an **unknown** origin (a type that never registered) is treated as not-allowed and **reacts**. The system MUST NOT silently pass an object whose origin it could not resolve — origin opt-in is incomplete by nature, and fail-closed is what keeps that incompleteness from becoming a false negative.

#### Scenario: An allowed origin passes

- **WHEN** a crossing object's observed origin is `app::domain` and the seam allows `["app::domain"]`
- **THEN** the system does not react

#### Scenario: A disallowed origin reacts

- **WHEN** a crossing object's observed origin is `app::infra` and the seam allows `["app::domain"]`
- **THEN** the system reacts (origin `app::infra` is not in the allowlist)

#### Scenario: An unknown origin reacts (fail-closed)

- **WHEN** a crossing object's concrete type never registered an origin, and the seam allows `["app::domain"]`
- **THEN** the system reacts, treating the unresolved origin as not-allowed, never silently passing it

### Requirement: Anchorability — undeclared seam is a constitution error

A probe SHALL reference a seam that was declared and installed; referencing an **undeclared seam name** SHALL be a constitution error (fail loud), the runtime analogue of an unresolvable static anchor — never silently treated as satisfied.

#### Scenario: A probe on an undeclared seam is a constitution error

- **WHEN** `assert_boundary!("ghost-seam", obj)` runs but no `RuntimeBoundary::at("ghost-seam")` was installed
- **THEN** the system fails loud (a constitution error), never silently passing the crossing

### Requirement: Default-safe reaction — a Violation event, panic opt-in

A reaction SHALL build a `xuanji::Violation` of kind **`Runtime`** (the shared measure: `target` = seam, `rule` = the allowlist rule, `finding` = offending origin + concrete type, with a severity) and by default **project it as a structured runtime event** (`Violation::to_json`) to a process-global **sink** the user can install (the system ships a default sink). A hard `panic` SHALL be **opt-in** (per boundary), never the default — a governance tool MUST NOT crash production on a false positive. A `warn`-severity boundary SHALL always be event-only.

#### Scenario: Default reaction emits an event, does not panic

- **WHEN** a boundary with default posture reacts
- **THEN** the system emits the `Violation` (kind `Runtime`) as json to the installed sink and the program continues (no panic)

#### Scenario: Panic is opt-in

- **WHEN** a boundary configured to panic on violation reacts
- **THEN** the system panics — only because panic was explicitly opted in

#### Scenario: A user-installed sink receives the event

- **WHEN** the user installs a custom sink and a boundary reacts
- **THEN** the custom sink receives the `Violation`, not the default sink

### Requirement: Production-light, lock-free hot path

The runtime dimension ships into the user's production binary, so the probe hot path SHALL be std-only and near-zero overhead: a write-once registry read with **no lock** (no `Mutex`/`RwLock` on the hot path) and a `TypeId` map that does NOT use the default SipHash hasher (a fold-hasher — a `TypeId` is already a hash). The crate MUST NOT depend on `syn` or any static-analysis engine; `serde_json` (via the shared measure) SHALL be used only on the cold path (emitting an event), never the hot path. The core check SHALL be a pure function over explicit registries, so it is testable without process-global state.

#### Scenario: The hot path adds no heavy dependency or lock

- **WHEN** self-governance and dependency checks run against the runtime crate
- **THEN** the crate depends only on `xuanji`, pulls no `syn`, and its origin lookup is a lock-free read of a non-SipHash `TypeId` registry

### Requirement: CI face — every declared seam is probed

The system SHALL provide a build/CI-time check (`audit_probe_coverage`), compiled behind the
non-default `audit` Cargo feature so a production binary that depends on the runtime dimension only
for its hot path carries none of the scanner (the shell enables the feature to run the audit inside
`check`). The check takes the **declared
`RuntimeBoundary` objects** as the authoritative set of seams and scans the workspace's source for
`assert_boundary!` probes, reacting (a `Violation` per offending seam, with the same exit-code
contract as the static dimensions) in **both directions**: a **declared seam with no probe** (the
boundary is never enforced — the otherwise-essential "declared but never enforced" gap) and a
**probe referencing an undeclared seam** (a typo against the declared set, caught at CI). The check
SHALL derive declared seams from the passed boundary objects, NOT by scanning source for
`RuntimeBoundary::at(...)` literals, so an unconventionally spelled or macro-built declaration
cannot silently escape the audit. A declared-but-unprobed seam SHALL react at the declaring
boundary's **declared severity** (a warn-severity boundary yields an advisory, not a failure); a
probe referencing an undeclared seam SHALL react at **enforce** severity.

Probe coverage SHALL be evaluated across the **whole workspace as one corpus**, scanning each
member crate's source root resolved from `cargo metadata` (the same source root the semantic
dimension scans), so a seam declared once and probed in any member counts as covered. A member
whose source root cannot be resolved SHALL be a constitution error (never a silent skip). Source
outside a member's library/binary target subtree (for example `tests/`, `examples/`, `build.rs`)
is out of scope — the same stated bound as the semantic dimension.

The probe scan SHALL be build/CI-time only (std-only source scan, never on the runtime hot path),
comment- and string-literal-aware (including raw and byte strings), tracking **nested** block
comments (a probe inside a nested comment is commented out and SHALL NOT count as coverage) and
recognizing all three macro delimiters (`()`, `{}`, `[]`). The scan is lexical and does not
evaluate `cfg`: a probe behind a non-production `#[cfg(...)]` is still counted, so a seam's
production probe must not live behind a non-production `cfg` — a stated bound, not a silent pass.
A probe whose seam argument is
a **string literal** (plain or raw) is auditable; a probe whose seam argument is **not** a string
literal (a constant or other expression) cannot be traced to a declared seam, and the system SHALL
react to it (an enforce `Violation` naming the un-auditable probe site) rather than silently skip
it — a silent skip would be a false negative, and erring toward a loud reaction is the project's
forbidden-bug trade.

The CI face verifies coverage against the **declared** seams and the **source**; it does NOT
observe the live, process-global install registry (which exists only in the adopter's running
binary). Consistency between the declared boundaries and what is actually installed is the single
source of truth's responsibility (the constitution is the one declared source both faces project
from) and is reacted to by the **prod face** at runtime: a probe on a seam absent from the
installed registry fails loud (the runtime analogue of a constitution error). The CI face SHALL
NOT claim to verify installation.

#### Scenario: A declared-but-unprobed seam reacts at CI time

- **WHEN** a `RuntimeBoundary` for seam `orphan` is passed to the audit but no `assert_boundary!("orphan", …)` probe exists anywhere in the workspace
- **THEN** `audit_probe_coverage` emits a violation naming the unprobed seam `orphan` and contributes a non-zero exit (when the boundary's severity is enforce), so the gap is caught at CI rather than silently unenforced at runtime

#### Scenario: A warn-severity declared seam without a probe is advisory

- **WHEN** a warn-severity `RuntimeBoundary` for seam `soft` is passed to the audit but has no probe
- **THEN** `audit_probe_coverage` reports the unprobed seam as an advisory that does not by itself cause a non-zero exit

#### Scenario: A probe referencing an undeclared seam reacts at CI time

- **WHEN** `assert_boundary!("ghost", …)` exists in source but no `RuntimeBoundary` for `ghost` is in the passed set
- **THEN** `audit_probe_coverage` emits an enforce violation naming the undeclared seam `ghost`, so the typo is caught at CI rather than panicking at runtime

#### Scenario: A declaration is recognized from the object, not its source spelling

- **WHEN** a seam's `RuntimeBoundary` is passed to the audit but the constructing call in source is spelled unconventionally (e.g. via a helper or constant) such that a textual scan would not find a `RuntimeBoundary::at(...)` literal
- **THEN** the audit still treats that seam as declared (it reads the passed objects), so coverage is judged against the seam that actually governs

#### Scenario: An un-auditable probe reacts rather than being silently skipped

- **WHEN** a probe is written as `assert_boundary!(SEAM_CONST, obj)` whose first argument is a constant or expression, not a string literal
- **THEN** `audit_probe_coverage` emits an enforce violation naming the un-auditable probe site, converting a silent coverage hole into a loud reaction

#### Scenario: A probe inside a nested block comment is not counted as coverage

- **WHEN** a declared seam's only `assert_boundary!` probe appears inside a nested block comment (e.g. `/* outer /* inner */ assert_boundary!("s", o); */`), so the compiler never sees it
- **THEN** `audit_probe_coverage` reports the seam unprobed (the scan tracks block-comment nesting), never counting the commented-out probe as coverage — the forbidden false negative is avoided

#### Scenario: A brace- or bracket-delimited probe is audited

- **WHEN** a probe is written with `{}` or `[]` delimiters (`assert_boundary!{"s", o}` or `assert_boundary!["s", o]`), which Rust accepts identically to `()`
- **THEN** `audit_probe_coverage` captures it as a probe (it is not silently dropped), so a typo'd seam written with non-`()` delimiters cannot escape the undeclared-seam check

#### Scenario: Coverage spans the workspace

- **WHEN** a seam is declared once and its only `assert_boundary!` probe lives in a different member crate of the workspace than the declaration site
- **THEN** `audit_probe_coverage` counts the seam as probed (the workspace is scanned as one corpus), not as an unprobed declaration

#### Scenario: A member's source root is resolved like the semantic dimension's

- **WHEN** a workspace member uses a non-default source layout (e.g. a custom library target path) so its source is not under `<manifest-dir>/src`
- **THEN** the audit still scans that member's real source root (resolved from `cargo metadata` as the semantic dimension resolves it), so a probe there is not invisible to the corpus

#### Scenario: A duplicate declared seam reacts at CI time

- **WHEN** two `RuntimeBoundary` objects naming the same seam are passed to the audit (the misconfiguration the prod `install` panics on)
- **THEN** `audit_probe_coverage` emits an enforce violation naming the duplicated seam, so the constitution error is caught at CI before it reaches a running binary

#### Scenario: A fully probed and declared set is clean

- **WHEN** every declared `RuntimeBoundary` seam has at least one string-literal `assert_boundary!` probe in the workspace, and every probe references a declared seam
- **THEN** `audit_probe_coverage` reports clean (exit 0)
