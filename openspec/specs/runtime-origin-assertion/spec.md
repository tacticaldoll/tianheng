# runtime-origin-assertion Specification

## Purpose

The 漏刻 (runtime) dimension's first capability: declare which concrete-type **origins** may cross a named runtime **seam**, and probe live `dyn` objects in production to catch a forbidden-origin type slipping through a `dyn Trait` into a layer it must not reach — what static and semantic analysis structurally cannot see. It has two faces: a **prod face** (the probe reacts fail-closed, emitting a `Violation` event by default, panic opt-in) and a **CI face** (`audit_probe_coverage` verifies every declared seam is probed and every probe references a declared seam). Origin is **observed**: it is **derived from the type** — the module the concrete type is defined in — and never taken from the registering call, so no code in the process can present a type under an origin it does not have (the origin requirement below states this in full and is authoritative over this summary). The hot path is std-only and lock-free; the crate depends on 璇璣 (`xuanji`) only — 星表 (`xingbiao`) is an additive, `audit`-feature-gated exception for the CI face's own cycle guard that never reaches the production hot path.
## Subject

- `crates/louke/src/**/*.rs`
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

The shipped default sink writes to stderr and MUST NOT itself panic if that write fails (a closed
or broken stderr — EPIPE on a closed pipe after its reader exits, EBADF on a closed fd) — the same
no-panic invariant the sink protects on its happy path. A failed default-sink write SHALL instead
increment a process-global, lock-free counter exposed as `dropped_sink_events() -> u64`, rather
than silently discarding all trace of the loss, so an adopter who relies on the default sink (has
never called `set_sink`) can detect from outside the process that an event went unobserved. The
counter increment itself SHALL be infallible — a single atomic add, never a lock, never able to
itself fail or panic — so closing this observability gap cannot reopen the panic risk it exists to
avoid. A custom sink's own success or failure is opaque to the system (`set_sink` takes a
`Fn(&Violation)` returning nothing) and is NOT counted — the counter is scoped to the shipped
default sink only.

#### Scenario: Default reaction emits an event, does not panic

- **WHEN** a boundary with default posture reacts
- **THEN** the system emits the `Violation` (kind `Runtime`) as json to the installed sink and the program continues (no panic)

#### Scenario: Panic is opt-in

- **WHEN** a boundary configured to panic on violation reacts
- **THEN** the system panics — only because panic was explicitly opted in

#### Scenario: A user-installed sink receives the event

- **WHEN** the user installs a custom sink and a boundary reacts
- **THEN** the custom sink receives the `Violation`, not the default sink

#### Scenario: A broken default-sink write is counted, not silently lost

- **WHEN** no custom sink is installed and the default sink's write to stderr fails (a closed or broken stderr)
- **THEN** the system does not panic, and `dropped_sink_events()` increases by exactly one for that violation

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
is out of scope — the same stated bound (bound: runtime-origin-assertion/source-outside-a-member-s-library-or-binary-target-subtree-is-out-of-scope-a-stated-bound) as the semantic dimension.

The probe scan SHALL be build/CI-time only (std-only source scan, never on the runtime hot path),
comment- and string-literal-aware (including raw and byte strings), tracking **nested** block
comments (a probe inside a nested comment is commented out and SHALL NOT count as coverage) and
recognizing all three macro delimiters (`()`, `{}`, `[]`). A probe lexically inside a **macro body**
— a `macro_rules!` definition body, or the body of any macro invocation `ident! (…)/{…}/[…]` other
than the `assert_boundary!` probe itself and the **transparent control-flow macro** carved out below
— is macro-generated or dead code and SHALL NOT count as coverage: the scanner skips such a body (the
same macro-body exclusion the static and semantic dimensions apply, reimplemented louke-locally
because 三儀 ⊥ 三儀 forbids importing 圭表's scanner).
Otherwise a probe in a never-invoked macro body would report its seam covered while the seam never
enforces at runtime — the audit's forbidden false negative. The scan is lexical and does not
evaluate `cfg`: a probe behind a non-production `#[cfg(...)]` is still counted, so a seam's
production probe must not live behind a non-production `cfg` — a stated bound (bound: runtime-origin-assertion/a-production-probe-behind-a-non-production-cfg-is-still-counted-a-stated-bound), not a silent pass.
A probe whose seam argument is a **string literal** (plain or raw) is auditable, and a **plain**-string
seam SHALL be compared to the declared seams by its **decoded** value — the exact `&str` the Rust
compiler produces from that literal, resolving the standard string escapes (`\n`, `\r`, `\t`, `\\`,
`\0`, `\'`, `\"`, `\xHH` byte escapes with value `<= 0x7F`, and `\u{…}` unicode escapes with
underscores permitted as digit separators) — so it matches the compiler-decoded declared seam
(`RuntimeBoundary::seam()`), NOT the raw source bytes between the quotes. Comparing the un-decoded
bytes would let an escape-bearing seam diverge between the two faces (reporting a probed seam as
unprobed and its probe as undeclared) and, when a declaration and a probe decode to the same bytes
by different spellings, silently count a seam as covered whose runtime probe would panic on an
undeclared seam — the forbidden false negative. A raw-string seam (`r"…"` / `r#"…"#`) keeps its
verbatim value (raw strings have no escapes, so their bytes already equal the compiler value). A
backslash-newline **line continuation** in a plain-string seam SHALL be decoded exactly as
rustc/`syn` decode it (stripping the backslash, the newline, and the continued line's leading
whitespace), so a seam literal written across a line continuation still matches its
compiler-decoded declared seam. A plain-string seam whose escape the std-only scanner cannot
decode — a malformed or unrecognized escape, an out-of-range `\x`, or an invalid `\u{…}` — SHALL
react as an un-auditable probe rather than a silently mismatched literal. A probe whose seam
argument is **not** a string literal (a constant or other expression) cannot be traced to a
declared seam, and
the system SHALL react to it (an enforce `Violation` naming the un-auditable probe site) rather than
silently skip it — a silent skip would be a false negative, and erring toward a loud reaction is the
project's forbidden-bug trade.

One macro is **transparent** and SHALL be carved out of the macro-body exclusion above: `cfg_if!`,
whose arms wrap human-authored items without transforming their identities, so code written inside an
arm is real, compiled code rather than macro-generated. The scanner SHALL scan **into** a transparent
invocation's body rather than skipping it, so a probe written in an arm counts as coverage, a probe in
an arm naming an undeclared seam reacts as a typo, and an un-auditable probe in an arm reacts as
un-auditable — each exactly as at top level. Skipping such a body produced errors in both directions
on ordinary, compilable source: a seam whose only production probe lived in an arm was reported
unprobed (a false alarm against real coverage), while a typo'd seam and an un-auditable probe inside
an arm escaped entirely (the audit's forbidden false negative, contradicting the un-auditable rule
stated above). Transparency SHALL be gated on the macro **name**, matching the static and semantic
dimensions' identical gate rather than deriving a third rule: a byte scanner cannot distinguish an
arbitrary macro's nested blocks from a transparent macro's arms, so a body-wrapping macro under any
other name SHALL remain excluded — a stated bound (bound: semantic-signature-coupling/a-macro-under-another-name-is-not-treated-as-transparent-a-stated-bound), shared across all three dimensions. Observation
SHALL stay **cfg-blind** here as everywhere in this scan: every arm is read, so a probe in an arm the
current configuration does not compile still counts, consistent with the already-stated `#[cfg]`
bound above.

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

#### Scenario: A probe inside a macro body is not counted as coverage

- **WHEN** a declared seam's only `assert_boundary!("s", o)` probe appears inside a `macro_rules!` body (or another macro invocation's body, e.g. `some_macro! { assert_boundary!("s", o) }`), and a real probe for a different declared seam `t` follows the macro body
- **THEN** `audit_probe_coverage` reports seam `s` unprobed (the macro body is skipped, so its probe does not count) while still capturing the real probe for `t` after the body — the forbidden false negative (a "covered" seam that never enforces) is avoided

#### Scenario: A probe inside a cfg_if arm counts as coverage

- **WHEN** a declared seam's only `assert_boundary!("s", o)` probe sits inside `cfg_if! { if #[cfg(unix)] { … } }`
- **THEN** `audit_probe_coverage` reports the seam covered, rather than reporting it unprobed against real coverage — the arm is transparent, not a macro-generated body

#### Scenario: A typo'd seam inside a cfg_if arm is caught

- **WHEN** a probe inside a `cfg_if!` arm names seam `seaam` while the declared set holds only `seam`
- **THEN** the audit reacts to the probe as referencing an undeclared seam, exactly as it would at top level

#### Scenario: An un-auditable probe inside a cfg_if arm reacts

- **WHEN** a probe inside a `cfg_if!` arm passes a non-literal seam argument (a `const`)
- **THEN** the audit reacts to it as un-auditable rather than silently skipping it, honoring the never-a-silent-skip rule inside arms too

#### Scenario: An escaped plain-string probe matches its escaped declared seam

- **WHEN** a `RuntimeBoundary::at("a\n")` seam is declared and its only probe in the workspace is `assert_boundary!("a\n", obj)`
- **THEN** `audit_probe_coverage` counts the seam covered and reports clean (the probe's decoded seam `a`+newline equals the declared seam), never the false pair of "declared seam unprobed" and "probe references undeclared seam" the raw-byte comparison produced

#### Scenario: A declaration and a probe that decode differently are caught, not counted as covered

- **WHEN** a seam is declared `RuntimeBoundary::at("a\\n")` (decoded: `a`, backslash, `n`) but the only probe is `assert_boundary!("a\n", obj)` (decoded: `a`, newline)
- **THEN** `audit_probe_coverage` reacts (the declared seam is reported unprobed and the probe references an undeclared seam), so the runtime mismatch is caught at CI rather than silently counted as coverage — the forbidden false negative is avoided

#### Scenario: An un-decodable escape reacts as un-auditable

- **WHEN** a probe's plain-string seam literal carries an escape the std-only scanner cannot decode (a malformed or unrecognized escape, an out-of-range `\x`, or an invalid `\u{…}`)
- **THEN** `audit_probe_coverage` emits an enforce un-auditable violation naming the probe site, never recording a silently mismatched literal (erring toward a loud reaction, the project's forbidden-bug trade)

#### Scenario: A backslash-newline line continuation decodes like rustc

- **WHEN** a probe's plain-string seam literal contains a backslash-newline line continuation (e.g. `"a\` + newline + `b"`)
- **THEN** `audit_probe_coverage` decodes it to the joined value (matching rustc/`syn`), rather than reacting as un-auditable

#### Scenario: An escape-free or raw-string seam is unaffected

- **WHEN** a seam and its probe are escape-free (e.g. `"domain-entry"`) or the seam is written as a raw string (`r"…"`)
- **THEN** `audit_probe_coverage` behaves exactly as before (the decoded value equals the raw value), so no existing adopter's coverage result or baseline identity changes

### Requirement: Root-aware audit excludes unreachable source files

When `audit_probe_coverage` receives a Rust target root file, it SHALL count probes only from that
file and source files reachable through its lexical module declarations. An undeclared `.rs` file,
or a conventional file shadowed by an inline-only module, SHALL NOT count as coverage. Every
reachable selected source file SHALL be read fail-loud; an unreadable file SHALL produce a
constitution error. The walker SHALL remain louke-local, std-only, and audit-feature-only. Directory
inputs SHALL remain accepted as the legacy recursive corpus for source compatibility.

A `mod name;` whose conventional file (`name.rs` or `name/mod.rs`) cannot be resolved SHALL be a
constitution error **unless** the declaration carries a `#[cfg(...)]` gate (below the `mod` keyword
and its name, comments permitted anywhere between them, never a reason the declaration goes
unrecognized), in which case the module may legitimately have no file in the current configuration
(an off feature or another platform) and SHALL be skipped rather than errored — the same
cfg-tolerance the semantic dimension applies, reimplemented louke-locally (三儀 ⊥ 三儀). This does not
evaluate `cfg`: a *resolvable* cfg-gated module is still scanned and its probes still counted; only
an *absent* file for a cfg-gated declaration is tolerated. A resolution ambiguity (both `name.rs` and
`name/mod.rs` present) remains a constitution error regardless of any gate. A declaration written
inside a **transparent control-flow macro arm** (`cfg_if!`) SHALL be reached by this walker and
treated as **cfg-conditional**: the walker SHALL descend each arm of such an invocation as a
transparent scope — with the enclosing module bases unchanged, since an arm adds no directory
component the way an inline `mod` does — so an arm-declared `mod` enters the reachable corpus and
its file's probes are counted, and an absent conventional file (or absent unconditional `#[path]`
target) for it SHALL be tolerated exactly as under a bare `#[cfg]`, because the arm's predicate lives
in the macro's `if #[cfg(..)]` header rather than on the item and every arm is conditionally compiled
by construction. Arm membership SHALL NOT be inherited into an inline `mod { … }` body descended from
within an arm, matching the bare-`#[cfg]` case, and SHALL NOT make a resolution ambiguity tolerable.
Without this, every probe beneath an arm-declared module was invisible — the coverage false negative
this walker exists to prevent, reached through the module graph rather than the probe scan.

The **only** legal non-inline `mod` form written inside a function or block body is one carrying
`#[path]` (a bare `mod name;` there has no established file-path convention and does not compile) —
the walker SHALL descend into **every** block scope (a fn/const/static body, a bare block, a match
arm, or any other brace-delimited scope), not only the scopes it specifically recognizes, so this
form is reached wherever Rust permits it. A `mod` found this way adds no directory component of its
own (unlike a NAMED inline `mod x { … }`), so the enclosing file's own bases carry through unchanged;
arm membership is inherited into a nested block the same way it already is into a directly
arm-declared `mod`.

A `mod name;` carrying an **unconditional** `#[path = "..."]` relocation SHALL be recognized
**structurally** — an outer attribute whose meta name is exactly `path` followed by `=` and a string
literal, with comments and string literals skipped, and the literal's escapes decoded as rustc/syn
decode them — and SHALL be **followed** to that author-chosen file, resolved relative to the
containing file's own directory with each enclosing inline-`mod` name accumulated onto it (rustc's
mod-rs-blind rule); for a non-mod-rs `name.rs` this differs from the conventional-child directory its
own `mod y;` would use, and a `#[path]` nested inside an inline `mod { … }` block adds that block's
name as a directory component. A `#[path]`-loaded file is itself mod-rs-like, so its own children
resolve from its directory. Probes inside a relocated module are therefore counted, and an
undeclared-seam probe there is caught. A declaration whose preamble merely
*contains* the text `path` — a `// fast path` comment, a `#[cfg(feature = "fastpath")]` gate — SHALL
NOT be read as a relocation and SHALL resolve conventionally, so no reachable module is dropped by a
false substring match (which would silently drop every probe beneath it — a coverage false negative,
the worst outcome under FN-first). A `cfg_attr`-wrapped `#[path]` is cfg-conditional on which file a
given build compiles, but `cfg_attr` never removes the `mod` item the way a bare `#[cfg]` does — so
its own target SHALL be followed too, resolved the identical way an unconditional `#[path]` is (from
the containing file's own directory): EVERY such target that exists on disk SHALL be read, unioned
with the conventional file if it too exists — cfg-blind observation cannot know which one a given
build actually compiles, so neither is silently preferred over the other. A declaration MAY carry
more than one SEPARATE `cfg_attr`-wrapped `#[path]` attribute (one per platform predicate); every one
SHALL be extracted and unioned the same way. Absence SHALL be tolerated only when NEITHER any
`cfg_attr` target NOR the conventional file resolves anywhere, and the declaration carries no other
cfg-conditional gate (a bare `#[cfg]` or transparent-arm membership) — that combination is a
genuinely broken reference on every configuration, so it SHALL still fail loud. A doubly-**nested**
`#[cfg_attr(a, cfg_attr(b, path = "…"))]` SHALL be resolved the same way as a single-level one: the
scanner locates the `path` value anywhere within the outer attribute's argument span rather than parsing
nesting structure, so nesting depth does not change whether it is found.

The identical union SHALL apply to a `cfg_attr`-wrapped `#[path]` on an **inline** `mod name { ... }`
(a body, not a `;`-terminated declaration), where it governs the **base directory** `name`'s own
nested items resolve from rather than a file to read (the body itself is already present in source
regardless of which base applies). Each candidate base -- every `cfg_attr` target and the conventional
directory -- SHALL be descended only when it exists as a directory; recursing into one that does not
exist would spuriously fail loud on `name`'s other, unrelated nested items solely because one
platform's directory happens to be absent, even when another candidate already backs them. If no
candidate directory exists at all, the conventional base SHALL be descended anyway, so a nested
reference genuinely broken on every platform still fails loud exactly as it did before this tolerance
existed.

#### Scenario: Orphan probe cannot cover a seam

- **WHEN** a target root declares no module for `orphan.rs` and that orphan file contains the only probe for a declared seam
- **THEN** the audit reports the seam unprobed because the compiler-unreachable file is absent from the root-aware corpus

#### Scenario: Reachable external module covers a seam

- **WHEN** a target root declares `mod adapter;` and the resolved `adapter.rs` or `adapter/mod.rs` contains the seam's probe
- **THEN** the audit counts the probe as coverage

#### Scenario: A module declared inside a cfg_if arm covers a seam

- **WHEN** a target root declares `cfg_if! { if #[cfg(unix)] { mod adapter; } }` and the resolved `adapter.rs` contains the seam's only probe
- **THEN** the audit descends the arm-declared module and counts the probe, rather than reporting the seam unprobed because the declaration sat inside a macro body

#### Scenario: An arm-declared module with no file is tolerated

- **WHEN** a target root declares `cfg_if! { if #[cfg(unix)] { mod unix_impl; } else { mod windows_impl; } }` and only `unix_impl.rs` exists
- **THEN** the audit skips the fileless arm declaration rather than reporting a constitution error — the arm's predicate is the gate, so rustc strips the arm and the crate compiles — while the same declaration written outside any arm with no file remains a constitution error

#### Scenario: An arm-declared dual-backed module is still an ambiguity

- **WHEN** a `mod child;` declared inside a `cfg_if!` arm resolves to both `child.rs` and `child/mod.rs`
- **THEN** the audit reports the resolution ambiguity as a constitution error — arm membership makes an absence tolerable, never two present files resolvable

#### Scenario: Inline module shadow does not activate a sibling file

- **WHEN** a root declares only `mod adapter { ... }` and a sibling `adapter.rs` contains a probe
- **THEN** the sibling file does not count because the inline body, not the conventional file, is the compiled module

#### Scenario: Custom target root remains auditable

- **WHEN** Cargo reports a custom library root filename and its reachable modules contain probes
- **THEN** the audit starts from that exact file rather than guessing `src/lib.rs`

#### Scenario: A cfg-gated module with no file is tolerated

- **WHEN** a target root declares `#[cfg(feature = "x")] mod optional;` with no `optional.rs` (the feature is off) alongside a non-cfg `mod present;` that resolves normally
- **THEN** the audit skips the absent cfg-gated module without a constitution error, while a non-cfg `mod missing;` with no file remains a fail-loud constitution error

#### Scenario: An unconditional #[path] module is followed; an incidental "path" substring is not

- **WHEN** a target root declares `#[path = "elsewhere.rs"] mod relocated;` (no conventional `relocated.rs`; `elsewhere.rs` holds a declared seam's probe) alongside a `// fast path` comment and `#[cfg(feature = "fastpath")] mod present;` whose conventional `present.rs` resolves normally
- **THEN** the `#[path]`-relocated module is recognized structurally and **followed** to `elsewhere.rs`, so its probe counts as coverage (never dropped as off the conventional path), while `present` — matched by no `path` attribute despite the incidental substring — still resolves conventionally

#### Scenario: A #[path] nested inside an inline module resolves from the accumulated directory

- **WHEN** a target root declares `mod inline { #[path = "other.rs"] mod inner; }`, `inline/other.rs` holds an undeclared-seam probe, and a same-named `other.rs` decoy sits beside the root
- **THEN** the audit resolves `inner` to `inline/other.rs` (the enclosing inline-`mod` name accumulated onto the base, as rustc compiles it) and reports the undeclared-seam probe, never reading the `other.rs` decoy and returning Clean at exit 0

#### Scenario: A semicolon inside an earlier attribute's own string does not hide a later #[path]

- **WHEN** a module declares `#[doc = "Handles A; falls back to B."]` immediately followed by an unconditional `#[path = "relocated.rs"] mod inner;`, and `relocated.rs` (not the conventional, absent `inner.rs`) holds a declared seam's probe
- **THEN** the `#[path]` attribute is still recognized and followed to `relocated.rs` — the bare `;` inside the preceding `#[doc]` attribute's own string value must never be mistaken for the preamble's own start, which would desync the attribute scan and silently lose the `#[path]` relocation (a false hard-failure on a module that genuinely compiles, or a misattribution to the wrong file)

#### Scenario: A brace-delimited attribute argument does not hide an earlier #[path]

- **WHEN** a module declares an unconditional `#[path = "relocated.rs"]` immediately followed by an unrelated `#[foo({ 1 })]` (a brace-delimited token-tree argument, not a string literal) before `mod inner;`, and `relocated.rs` holds a declared seam's probe
- **THEN** the `#[path]` attribute is still recognized and followed to `relocated.rs` — the brace-delimited attribute argument's own internal `{`/`}` bytes must never be mistaken for a top-level item boundary, which would desync the attribute scan and silently lose the `#[path]` relocation the same way an earlier attribute's own string content could

#### Scenario: Legacy directory callers remain compatible

- **WHEN** a direct caller passes a source directory instead of a target root file
- **THEN** the audit retains the recursive directory scan and the caller requires no source change

#### Scenario: A comment between mod and its name does not drop the module

- **WHEN** a target root declares `pub mod /* relocated */ child;` (or `pub mod child /* relocated */;`) and `child.rs` holds a declared seam's probe
- **THEN** the audit recognizes the declaration and counts the probe, rather than silently dropping the module and its whole subtree because the comment fell between the `mod` keyword and its terminator

#### Scenario: A #[path] mod inside a function body reacts

- **WHEN** a target root declares `pub fn f() { #[path = "inner.rs"] mod inner; }` and `inner.rs` holds an undeclared-seam probe
- **THEN** the audit descends into the function body, recognizes the block-scoped `#[path] mod`, and reports the undeclared-seam probe — the only legal non-inline module form inside a block, previously invisible because every unrecognized brace was treated as one opaque, unwalked unit

#### Scenario: Two cfg_attr-wrapped #[path] declarations covering every platform are scanned, not erred

- **WHEN** a target root declares `#[cfg_attr(unix, path = "u.rs")]` and `#[cfg_attr(not(unix), path = "w.rs")]` on the same `pub mod plat;`, both `u.rs` and `w.rs` present, each holding a probe
- **THEN** the audit reads both targets and counts their probes as coverage, rather than reporting a constitution error on source that compiles cleanly on every configuration

#### Scenario: A missing cfg_attr target is tolerated when the conventional file backs the module

- **WHEN** a target root declares `#[cfg_attr(windows, path = "win.rs")] pub mod plat;` with `win.rs` absent but the conventional `plat.rs` present, holding a declared seam's probe
- **THEN** the audit reads the conventional file and counts its probe, tolerating the absent `cfg_attr` target rather than treating its absence alone as a constitution error

#### Scenario: A cfg_attr-wrapped #[path] on an inline module redirects its own nested items

- **WHEN** a target root declares `#[cfg_attr(unix, path = "unix_dir")] pub mod x { pub mod y; }` with `unix_dir/y.rs` present and holding a declared seam's probe, but the conventional `x/y.rs` absent
- **THEN** the audit descends `y` from the `cfg_attr` target's directory and counts its probe, rather than reporting a constitution error against the absent conventional `x/y.rs`

#### Scenario: A doubly-nested cfg_attr-wrapped #[path] is followed the same as a single-level one

- **WHEN** a target root declares `#[cfg_attr(unix, cfg_attr(feature = "x", path = "nested.rs"))] pub mod plat;`, `nested.rs` present and holding a declared seam's probe, and the conventional `plat.rs` present but holding none
- **THEN** the audit reads `nested.rs`'s probe and counts it as coverage, rather than reading the conventional file alone and reporting the seam unprobed

### Requirement: An un-auditable probe's identity distinguishes distinct offending expressions

An un-auditable-probe fact SHALL identify the offending non-literal seam expression's own source
text (its first macro-argument span, trimmed) and its **owner-qualified enclosing item**, alongside
its file, so two non-literal expressions differing by file, enclosing item, or expression text
remain distinct findings, and baselining one SHALL NOT mask another. The owner-qualified enclosing
item SHALL NOT be a bare innermost name: for a free `fn` it is the module path plus the fn name; for
a method inside `impl Type { … }` it is the `Self` type plus the method name; for a method inside
`impl Trait for Type { … }` it is the trait path, the `Self` type, and the method name; for a
trait's own default-body method it is the trait name plus the method name — mirroring the owner/
trait_ref qualification `semantic-unsafe-confinement` already uses for the identical same-named-item
collision. A bare method name alone SHALL NOT be used, since two distinct owners may share one.

Byte-identical expression text within the same file and the same owner-qualified enclosing item
collapses to one finding — a stated bound (bound: runtime-origin-assertion/identical-expression-repeated-in-the-same-function-collapses-to-one-finding-a-stated-bound), not a silent gap: at that granularity no further source
content distinguishes the two occurrences, so they represent the same restated fact (mirroring
`module-boundary`'s "the same import on multiple lines is one violation" precedent), not two masked
problems. Neither the enclosing-item qualification nor the expression text SHALL be derived from
byte offset, line number, or occurrence count.

The `file` component of this identity SHALL be labeled relative to a **caller-supplied anchor
directory** — a required parameter of `audit_probe_coverage` — never the raw absolute filesystem path
the scanner happened to read it from, and never a path the audit derives from the source roots it was
given. An absolute path varies by checkout location (a different clone path, a different CI runner)
even for byte-identical source, so baking it unconditionally into the identity would make a recorded
baseline match nothing in any other checkout — the accepted violation re-fires as new while the
recorded entry is simultaneously reported stale.

The anchor SHALL NOT be derived from the source-root set, because such a derivation buys
checkout-independence at the cost of **member-set independence** and thereby reopens the same loss
through a second door: the longest common prefix of every member under `crates/` is `<root>/crates`,
labeling a file `a/src/lib.rs`, and adding one member outside that prefix drops the anchor to
`<root>` and relabels the identical file `crates/a/src/lib.rs`. Every entry recorded against the old
label then goes stale and re-fires as new at once, on a change that touched none of the observed
files. An identity SHALL therefore be a function of the observed source and the caller's stable
anchor only — never of which other roots happened to be scanned alongside it. A caller composing this
audit over a Cargo workspace SHALL pass the workspace root Cargo itself resolves
(`xingbiao::workspace_root`), which moves with neither the clone location nor the member set; the
`tianheng` shell SHALL do so, falling back to the target manifest's own directory only when metadata
carries no such field.

That `file` label SHALL be an **injective** encoding of the observed path, never a lossy rendering of
it. A platform path is a byte string, not necessarily valid UTF-8, and a lossy conversion replaces
each undecodable byte with one replacement character — so two distinct source files differing only in
such bytes would carry one label, hence one identity, and a baseline accepting the first would suppress
the second's never-accepted violation (the forbidden false negative, reached through an identity
component rather than through the scan). The encoding SHALL therefore preserve distinctness in both
directions: an undecodable byte SHALL be escaped, and the escape-introducing character SHALL itself be
escaped, so no escaped label can be spelled by a path that did not need escaping. A path that is valid
UTF-8 and contains no escape-introducing character SHALL be labeled unchanged, so no existing baseline
entry re-keys.

The anchor SHALL be an **absolute** path, and a relative or empty one SHALL be a constitution error
(exit 2) naming the failed precondition and the value to pass instead — never accepted with a
silently degraded label. Stripping a relative prefix from an absolute source path cannot succeed, and
stripping an empty one succeeds while removing nothing, so either anchor leaves every label in its raw
absolute form: the checkout-dependent identity this rule exists to prevent, reached through an
argument that looked accepted. An empty anchor is therefore refused by the same rule rather than
treated as a "no anchor" opt-out, because its effect *is* the defect and no caller has a correct use
for it.

Absoluteness is what the audit itself can verify; being a true ANCESTOR of the observed files is the
caller's responsibility, and is the reason the anchor is the caller's to supply. An observed file that
does not lie under the anchor SHALL keep the path as observed, so an absolute anchor unrelated to the
source roots degrades per file rather than erroring. A file reached through an ABSOLUTE
`#[path = "/…"]` literal is governed by its own rule — it keeps the path the literal wrote and is never
relativized — because relativizing what does not move with the checkout is what made that one
construct's identity checkout-dependent. The violation SHALL react in every case; no labeling rule may
drop one.

#### Scenario: Two member sets over one checkout label a shared file identically

- **WHEN** the same checkout is audited twice with the same anchor but two different source-root sets
  — first the members under one shared prefix, then those members plus one outside that prefix (a
  tool, example, or fixture crate)
- **THEN** the file both runs observe carries the identical `file` label and therefore the identical
  identity, so a baseline recorded before the member was added still matches afterwards

#### Scenario: A relative or empty anchor is a constitution error

- **WHEN** `audit_probe_coverage` is called with absolute source roots and an anchor that is relative
  (`.`, `crates`, `../sibling`) or empty
- **THEN** it reports a constitution error naming the failed precondition and the value to pass
  instead, and exits 2 — rather than returning findings whose `file` labels silently kept their raw
  absolute form

#### Scenario: Same expression in two different free functions stays distinct

- **WHEN** `fn a() { assert_boundary!(SEAM_A, obj); }` and `fn b() { assert_boundary!(SEAM_A, obj); }` appear in the same file
- **THEN** `audit_probe_coverage` emits two distinct un-auditable-probe violations, distinguished by their enclosing function, and baselining one does not suppress the other

#### Scenario: Same-named method in two different impls stays distinct

- **WHEN** a file contains `impl A { fn probe(&self) { assert_boundary!(SEAM_A, obj); } }` and `impl B { fn probe(&self) { assert_boundary!(SEAM_A, obj); } }`
- **THEN** `audit_probe_coverage` emits two distinct un-auditable-probe violations, distinguished by their owner (`A` vs `B`), even though the method name and expression text are identical, and baselining one does not suppress the other

#### Scenario: Same-named method in two different trait impls of the same type stays distinct

- **WHEN** a file contains `impl Foo for T { fn probe(&self) { assert_boundary!(SEAM_A, obj); } }` and `impl Bar for T { fn probe(&self) { assert_boundary!(SEAM_A, obj); } }`
- **THEN** `audit_probe_coverage` emits two distinct un-auditable-probe violations, distinguished by their trait (`Foo` vs `Bar`) even though the `Self` type, method name, and expression text are identical

#### Scenario: Two distinct expressions in the same function stay distinct

- **WHEN** a single `fn` contains both `assert_boundary!(SEAM_A, obj)` and `assert_boundary!(compute_seam(), obj)`
- **THEN** `audit_probe_coverage` emits two distinct un-auditable-probe violations, distinguished by their expression text

#### Scenario: Source outside a member's library or binary target subtree is out of scope — a stated bound

- **WHEN** a probe or a seam mention sits in `tests/`, `examples/`, or `build.rs`
- **THEN** the audit does not read it, its corpus being the member's library and binary targets — a stated bound shared with the semantic dimension, never a silent claim of coverage
- **PINNED-BY** `source_outside_lib_or_bin_target_subtree_is_out_of_scope_corpus_bound`

#### Scenario: A production probe behind a non-production cfg is still counted — a stated bound

- **WHEN** a seam's only probe sits behind a `#[cfg(test)]` or another non-production predicate
- **THEN** the audit counts it as coverage, being cfg-blind, so a seam whose production probe lives there is reported as probed — a stated bound, never a silent pass
- **PINNED-BY** `production_probe_behind_non_production_cfg_is_counted_as_coverage`

#### Scenario: Identical expression repeated in the same function collapses to one finding — a stated bound

- **WHEN** a single `fn` contains `assert_boundary!(SEAM_A, obj)` written twice, verbatim
- **THEN** `audit_probe_coverage` emits one un-auditable-probe violation for that site — a stated bound, since no further source content distinguishes the two occurrences
- **PINNED-BY** `identical_expression_repeated_in_the_same_function_collapses_to_one_violation`

#### Scenario: Two paths differing only in an undecodable byte stay distinct identities

- **WHEN** two observed source files' paths differ only in a byte that is not valid UTF-8, and each
  contains a non-literal probe
- **THEN** `audit_probe_coverage` emits two distinct un-auditable-probe identities, so baselining one
  cannot suppress the other — rather than one label, one identity, and one silently accepted violation

#### Scenario: A literal escape character in a path cannot spell an escaped byte

- **WHEN** one observed path genuinely contains the escape-introducing character followed by hex
  digits, and another contains the undecodable byte those digits would encode
- **THEN** the two carry distinct labels, so the encoding is injective in both directions rather than
  only for undecodable input

#### Scenario: Two files with the identical expression stay distinct by file

- **WHEN** `assert_boundary!(SEAM_A, obj)` appears in `src/a.rs` and, separately, in `src/b.rs`
- **THEN** `audit_probe_coverage` emits two distinct un-auditable-probe violations, distinguished by file

#### Scenario: The same source scanned from two different checkouts yields the identical identity

- **WHEN** a byte-identical file containing a non-literal probe is scanned once from one absolute
  checkout location and again from a different absolute checkout location (the same relocation a
  different clone path or CI runner produces)
- **THEN** `audit_probe_coverage` emits identical un-auditable-probe violation identities in both
  runs, so a baseline recorded in one checkout remains valid in the other, rather than differing
  only in the `file` field's absolute prefix

#### Scenario: An absolute #[path] literal's target outside the anchor keeps its absolute label — a stated bound

- **WHEN** a module is reached only through an absolute `#[path = "/…"]` literal whose target does not lie under the scanning checkout's own anchor, and its body contains a non-literal probe
- **THEN** `audit_probe_coverage` still emits the un-auditable-probe violation, naming the site with the raw absolute path — a stated bound, since the literal has no textual relationship to the anchor
- **PINNED-BY** `an_absolute_path_literal_outside_the_anchor_keeps_the_path_the_literal_wrote`

### Requirement: Un-auditable probe identity includes lexical ownership

An un-auditable runtime probe fact SHALL identify its complete enclosing lexical item context within
the source file. Equal nested function names, methods on equal local type names, or local impl
contexts in distinct enclosing functions SHALL remain distinct without using absolute byte
offsets, global traversal ordinals, or collection positions. Anonymous equal-header siblings MAY
use a parent-local discriminator that is stable under differently-shaped unrelated insertion.

#### Scenario: Equal nested functions in distinct outer functions remain distinct

- **WHEN** two outer functions in one file each define the same-named nested function containing byte-identical non-literal probes
- **THEN** the audit emits two distinct structured fact identities so baselining either cannot suppress the other

#### Scenario: Unrelated insertion preserves lexical identity

- **WHEN** an unrelated item is inserted before a nested un-auditable probe
- **THEN** the probe retains the same structured fact identity

### Requirement: Anonymous lexical scopes distinguish un-auditable probes

An un-auditable probe's complete lexical owner SHALL include anonymous block scopes that enclose a
named item, including closure bodies. Equal nested function names and expression text under
distinct closures in the same named owner SHALL remain distinct facts. The discriminator MUST NOT
use an absolute byte offset; equal structural siblings MAY use a parent-local discriminator that
is stable when a differently-shaped unrelated item is inserted.

#### Scenario: Equal nested functions under distinct closures stay distinct

- **WHEN** one function contains two closure bodies that each declare `fn inner()` with the same
  non-literal `assert_boundary!` expression
- **THEN** the audit emits two distinct un-auditable-probe identities

#### Scenario: Unrelated insertion preserves anonymous ownership

- **WHEN** a differently-shaped unrelated statement or item is inserted before one of those
  closures
- **THEN** the pre-existing closure probe retains its structured fact identity

### Requirement: CI face accepts configurable custom probe macro markers

The `audit_probe_coverage` scanner SHALL support custom probe macro marker names (`&[&str]`), defaulting to `["assert_boundary"]`. Each custom marker identifier SHALL be a valid ASCII Rust identifier (`[A-Za-z_][A-Za-z0-9_]*`, strictly excluding keywords, raw identifiers, and non-ASCII characters). Custom markers SHALL be matched at a valid word boundary followed by optional whitespace and `!`, applying the same lexical scanning, seam argument decoding, and macro-body exclusion rules as `assert_boundary!`. A custom marker probe referencing a declared seam SHALL count toward probe coverage for that seam.

#### Scenario: A custom probe macro wrapper is recognized in CI coverage

- **WHEN** a project wraps `assert_boundary!` in a custom macro `company_seam!` and runs `audit_probe_coverage_with_markers` configured with `["assert_boundary", "company_seam"]`
- **THEN** calls to `company_seam!("seam-name", obj)` are scanned as valid auditable probes for seam `"seam-name"`

#### Scenario: Non-ASCII or invalid identifier markers cause a constitution error

- **WHEN** `audit_probe_coverage_with_markers` is called with a marker containing non-ASCII characters, keywords, or invalid identifier characters
- **THEN** the action fails loud with `Outcome::ConstitutionError`

#### Scenario: Unregistered custom macro markers are ignored

- **WHEN** a file contains `other_macro!("seam-name", obj)` where `"other_macro"` is not in the configured marker list
- **THEN** the scanner ignores it and does not record a probe

#### Scenario: Empty or blank marker list is a constitution error

- **WHEN** `audit_probe_coverage_with_markers` is called with an empty marker list or a marker string that is empty or blank
- **THEN** the action fails loud with `Outcome::ConstitutionError`, never silently flooding violations or matching empty strings

### Requirement: Deeply nested source structure is a scan error, never a stack overflow

The system SHALL react 0/1/2 when discovering a file's own child module declarations for the
file-input reachable-module walk, even when the source's lexical structure — nested blocks, a
transparent macro's (`cfg_if!`) arms, or inline `mod` bodies — nests past a measured
stack-safety depth cap, rather than overflowing the native stack (an uncontrolled process abort).
Nesting comfortably under the cap SHALL be observed exactly as a shallower structure would be.

#### Scenario: Pathologically nested blocks are a scan error, not a crash

- **WHEN** a governed source file's own lexical structure (nested blocks, a transparent macro's
  arms, or inline `mod` bodies) nests past the depth cap this scanner supports
- **THEN** the system reports a constitution error (exit 2) naming the depth bound it could not
  judge past, rather than crashing the process

#### Scenario: Moderately nested blocks still observe a real violation

- **WHEN** a governed source file nests a genuinely unresolvable `mod` declaration inside blocks
  well under the depth cap
- **THEN** the system still reports the missing-module constitution error, proving the walk
  reaches that depth rather than being narrowed by the cap

### Requirement: Origin is derived from the registered type, not supplied by the registering code

A concrete type SHALL opt into an origin via a `macro_rules!` (no proc-macro, no `syn`) whose expansion
target is **generic over that type and takes no origin argument**. Every component of the resulting
registration — the type's identity, its origin, and the type name carried in findings — SHALL be
derived from the type parameter alone. The registering code SHALL have no way to supply, override, or
influence the origin it registers, so an origin naming a module the type does not belong to is
**unrepresentable** rather than detected.

An origin SHALL be the module the type is **defined** in. The system MUST NOT derive an origin from the
registration call's own location, because that location is the caller's choice — a label the
registering code selects for itself rather than a property of the type; deriving it from the type is
what makes the origin an observation. For a
registration written inside the type's own module — the documented idiom — the derived origin equals
the registration site's module path, so that idiom's declarations are unaffected.

Because std has no pre-`main` hook, registration SHALL be performed by an explicit startup call (the
macro yields an entry the startup installs); a type that is never registered has no known origin.
Observing the concrete type behind a `dyn Trait` requires the governed trait to carry a `louke::Tracked`
supertrait (rust-1.85-compatible; no trait upcasting), and the concrete type to be `'static`.

The derivation SHALL happen where the type is still a type parameter — inside the macro's expansion
target — because no reverse lookup from a type's identity back to its path exists; a design that
validated a supplied origin at install time instead would be reacting to a disagreement this
requirement makes impossible to express. The prod hot path SHALL be unchanged by this derivation: the
registry still holds `&'static str` origins resolved once at startup, with no lock, no allocation per
crossing, and no dependency beyond std.

#### Scenario: A type's origin is its defining module

- **WHEN** `register_origin!(PostgresRepo)` is written for a `PostgresRepo` defined in module
  `app::infra`, and installed at startup
- **THEN** the origin registry maps that type to the origin `app::infra`, derived from the type itself
  rather than from any label the registering code supplies

#### Scenario: A registration away from the type's module still names the type's module

- **WHEN** `register_origin!(PostgresRepo)` for a `PostgresRepo` defined in `app::infra` is written
  instead inside a startup module `app::startup`
- **THEN** the registered origin is `app::infra`, not `app::startup` — the registration's location does
  not enter the origin at all

#### Scenario: A registration cannot present an origin the type does not have

- **WHEN** code bypasses `register_origin!` and calls its expansion target directly for a type of its
  own, intending to register that type under an allowlisted origin it does not belong to
- **THEN** no such call can be written: the expansion target accepts only the type, so the registered
  origin is that type's own defining module, and a seam crossing by that type reacts fail-closed

#### Scenario: Naming another type's identity registers that type honestly

- **WHEN** code calls the expansion target with a type it does not own, hoping to inject a false
  mapping for it
- **THEN** the registration produced is the correct one for that type, and a second registration of an
  already-registered type fails loud as a duplicate, exactly as two `register_origin!` sites for one
  type already do

### Requirement: The derived origin's shape bounds are stated, not implied

The system SHALL state the following bounds on the derived origin rather than imply a uniform module
path. None of them is a fail-loud class today, because the existing fail-closed allowlist match already
reacts to each one loudly and in the safe direction — a bound is stated where a reaction already covers
it, never used to forbid a future reaction by prose. The origin is derived from the type's own reported
path, whose shape is not uniform across all types:

- A type defined in **another crate** reports that crate's own defining path, which may be a private
  internal module rather than the public path it is re-exported at. Registering a foreign type
  therefore does not attribute it to the registering layer; a type that should carry a layer's origin
  is a type defined in that layer (a newtype), which is also what actually crosses the seam.
- A type defined inside a **function body** reports a path qualified by the enclosing function, which
  is not a module path.
- A **generic** type's arguments are not part of its origin: the origin is taken from the path with its
  argument list removed, so two instantiations of one generic type share one origin. Argument text may
  itself contain path separators and nested argument lists, so the removal SHALL be delimiter-aware
  rather than a search for the last separator. The consequence SHALL be stated in the direction that
  matters, not only as deduplication: a generic **defined in an allowed module** carries that origin
  whatever its arguments are, so an instantiation whose argument type comes from a forbidden module
  crosses as an allowed origin. This follows from an origin being a **module** and not a type, and it is
  the bound of what a runtime origin observes — governing which *instantiations* may cross is a
  different capability, not a defect of this one. It is stated because the alternative readings are
  worse: an origin cannot name two modules at once, and reacting whenever an argument's module differs
  from the outer type's would fire on every ordinary instantiation over a primitive.
- A **type alias** reports the aliased type's defining path, not the alias's location, so an alias
  cannot relabel an origin.
- A **composite** shape — a reference, tuple, array, pointer, or function pointer — yields a
  **truncated** rendering rather than a module path, and SHALL be stated as truncated rather than as
  unchanged. The argument cut above is delimiter-aware for `<…>` and for nothing else, so for any other
  composite the search for the final separator lands inside the wrapped type's own path: `&m::Foo`
  derives `&m`. This SHALL NOT be resolved by widening the cut to the other delimiters, because there is
  no answer to widen it to — an origin is a **module**, and a composite has no single defining module,
  the tuple `(a::T, b::U)` requiring two for the same reason a generic's argument cannot contribute one.
  The reaction is unaffected and SHALL stay fail-closed: such a rendering equals no module name, so it
  matches no allowlist entry, and in particular it never equals the **wrapped** type's own defining
  module, so an allowlist permitting that module cannot admit the composite. That last property is what
  SHALL be pinned by test, rather than the exact rendering, which the paragraph below declares unstable.

The reported path's exact rendering is not guaranteed stable across compiler versions. The system
SHALL keep that instability confined to loud reactions: a rendering change makes an origin stop
matching its allowlist entry, which reacts fail-closed, and SHALL NOT be able to produce a silent pass.
An observed origin SHALL NOT enter a rule key or any recorded baseline identity, so no accepted
violation re-keys on a toolchain change.

#### Scenario: A composite shape yields a truncated origin — a stated bound

- **WHEN** a registered type is a composite — a reference, tuple, array, pointer, or function pointer — whose wrapped type is defined in a module the seam allows
- **THEN** the derived origin is a truncated rendering that equals no module name, so it matches no allowlist entry and in particular never the wrapped type's own defining module; the crossing reacts fail-closed rather than being admitted through the wrapper
- **PINNED-BY** `the_derived_origin_honors_its_stated_shape_bounds`

#### Scenario: A foreign type's origin is its own defining path

- **WHEN** a type defined in another crate is registered
- **THEN** its origin is that crate's own defining module path for it, so it does not match an
  allowlist entry naming the registering layer, and the crossing reacts fail-closed with a finding
  naming the observed origin

#### Scenario: A generic defined in an allowed module carries that origin whatever it wraps

- **WHEN** a generic type defined in an allowed module is instantiated with an argument type defined in
  a module the seam forbids, and that instantiation crosses the seam
- **THEN** the crossing carries the allowed origin — the outermost type's defining module — rather than
  reacting, which is the stated bound of observing an origin as a module: the type that crosses is the
  one defined in the allowed module

#### Scenario: Two instantiations of one generic type share one origin

- **WHEN** a generic type is registered at two different argument instantiations
- **THEN** both register the same origin — the type's defining module, with the argument list removed —
  including when an argument's own text contains path separators or a nested argument list

#### Scenario: An alias cannot relabel an origin

- **WHEN** a type alias declared in one module names a type defined in another, and the alias is
  registered
- **THEN** the registered origin is the aliased type's defining module, not the alias's

### Requirement: An observed origin matches an allowed entry by equality

The allowlist match SHALL compare an observed origin to each allowed entry by **equality**, never by
module-prefix or subtree containment. A containment match would let a type defined in a module beneath
an allowed entry newly pass a seam that reacts today, converting a live reaction into a silent pass —
the forbidden false negative, reached by loosening the matcher rather than by missing an observation.
Governing a subtree SHALL therefore be expressed by declaring each module that may cross, not by
widening the comparison.

#### Scenario: A type beneath an allowed module does not pass

- **WHEN** a seam allows origin `app::infra` and a crossing object's type is defined in the descendant
  module `app::infra::pg`
- **THEN** the system reacts — the observed origin is not equal to any allowed entry — rather than
  treating the descendant as covered by its ancestor

### Requirement: The legacy directory corpus does not descend a symlinked subdirectory

A **directory** input to the probe-coverage audit is the legacy recursive corpus, and it SHALL NOT
descend a symlinked subdirectory. The directory walk classifies entries without following symlinks, so a
symlinked directory is not recognized as one and is skipped — a deliberate choice, because following it
admits an unbounded walk on a cyclic symlink, which the audit must never turn into a process abort.

The consequence SHALL be stated rather than left as a silent claim of completeness over that corpus: a
probe living behind a symlinked subdirectory is invisible to a directory-input audit, so a declared seam
whose only probe sits there reads as **unprobed**, and an undeclared-seam or un-auditable probe there is
not reacted to at all. A caller that needs such a file observed SHALL pass the **target root file**
instead, which reaches it through the module graph — reading a file follows symlinks, so no bound
applies there. The 天衡 shell passes target root files, so an adopter's `check` is unaffected; the bound
belongs to the directory input that exists for source compatibility.

#### Scenario: A probe behind a symlinked subdirectory is seen from the root and not from the directory — a stated bound

- **WHEN** a module reached through a `#[path]` into a symlinked directory holds the only probe for a
  declared seam, and the audit is run twice over that package — once with the target root file, once
  with the source directory
- **THEN** the root-file run reports the seam covered, while the directory run reports it unprobed — the
  stated bound of the legacy corpus, recorded rather than presented as equivalent coverage
- **PINNED-BY** `a_symlinked_subdirectory_is_descended_from_a_root_file_and_not_from_a_directory`

### Requirement: A file reached through an absolute path literal keeps the path the literal wrote

A file reached only through an **absolute** `#[path = "/…"]` literal SHALL be labeled with the path as
that literal wrote it, and SHALL NOT be relativized against the caller's anchor — whether or not its
target happens to lie under that anchor. Relativizing it is what made this construct's identity
checkout-DEPENDENT: prefix-stripping succeeds by pure text match wherever the target coincidentally
shares the anchor's prefix and fails everywhere else, so one identical committed literal produced a
relative-looking label in one checkout and the raw absolute path in another. An absolute literal does
not move with the checkout, so its label SHALL NOT either, and the coincidence of whether a given
checkout's anchor happens to contain the target SHALL NOT reach the identity.

Being reached through such a literal SHALL be **inherited** by the files that literal's target reaches
in turn: they resolve from its own directory, so the identical coincidence applies to them.

Inheritance SHALL hold along **both** paths by which such a target is reached, and they are distinct: a
file reached from another file, and a file reached from a *base directory* that an **inline** `mod`'s own
absolute `#[path]` established within one file. The second is not covered by the first — the base is
introduced inside a single file, so provenance threaded only from file to file never reaches it — and it
SHALL be stated separately because implementing one and not the other reads as complete. Provenance
SHALL also accumulate through nesting, so an inline base derived conventionally from an absolute one is
itself absolute-reached.

A file reached by any other means — a conventional child, a relative `#[path]`, or the legacy directory
walk — SHALL continue to be labeled relative to the anchor, which is what keeps the realistic
sibling-share idiom checkout-independent. The violation SHALL react in every case; no labeling rule
here may drop one.

#### Scenario: One committed absolute literal yields one identity across checkouts

- **WHEN** the identical absolute `#[path = "/…"]` literal is committed into two different checkouts,
  its target happens to lie under one checkout's anchor and not the other's, and its body contains a
  non-literal probe
- **THEN** `audit_probe_coverage` emits the **same** un-auditable-probe identity in both runs, so a
  baseline recorded in one checkout remains valid in the other

#### Scenario: An absolute literal's target under the anchor is not relativized

- **WHEN** a module is reached only through an absolute `#[path = "/…"]` literal whose target lies
  under the caller's anchor, and its body contains a non-literal probe
- **THEN** the violation's `file` is the path the literal wrote, not a label relative to the anchor —
  the anchor's containment of that target being the coincidence this rule exists to keep out of the
  identity

### Requirement: An audit finding SHALL carry no repair polarity, and that SHALL be stated

An audit finding SHALL carry **no** polarity, and the reason SHALL be recorded rather than left as an absence a
reader has to interpret. `Polarity` distinguishes a **deny breach** — repair by removing the offending code — from
an **allowlist gap** — repair by removing the code *or* by widening the declared set. The audit's findings are
neither: a declared seam with no probe is repaired by probing it or by dropping the declaration, and a probe
naming an undeclared seam by declaring it or by deleting the probe. Assigning either value would name a repair
direction that does not exist.

This is the only production emission path in the family that carries none, and the difference is by construction
elsewhere: 圭表's crate and module rules answer through **exhaustive matches returning `Polarity`**, so a new rule
variant cannot compile without declaring one, and 渾儀 carries a non-optional `Polarity` on the context every
finding is emitted through. `Violation::polarity` is therefore an `Option` for exactly this dimension's audit, and
saying so is what keeps a reader from reading the `Option` as a gap — measured, a review did read it that way.

No reaction is required for the by-construction half: an exhaustive match is a stronger guard than a test, and
adding one would be a second copy of a fact the compiler already holds.

#### Scenario: A declared seam has no probe

- **WHEN** the audit reports a declared-but-unprobed seam
- **THEN** the violation carries no polarity, because probing it and dropping the declaration are both repairs
  and neither is the direction `Polarity` names

#### Scenario: A probe names an undeclared seam

- **WHEN** the audit reports a probe against a seam the constitution does not declare
- **THEN** the violation carries no polarity, for the same reason in the mirror direction

#### Scenario: A static or semantic rule is added

- **WHEN** a new rule variant is added to a dimension whose findings carry a repair direction
- **THEN** it does not compile until it declares one, so that half of the contract needs no reaction of its own
