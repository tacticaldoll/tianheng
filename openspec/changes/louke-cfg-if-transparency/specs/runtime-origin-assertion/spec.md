## MODIFIED Requirements

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
recognizing all three macro delimiters (`()`, `{}`, `[]`). A probe lexically inside a **macro body**
— a `macro_rules!` definition body, or the body of any macro invocation `ident! (…)/{…}/[…]` other
than the `assert_boundary!` probe itself and the **transparent control-flow macro** carved out below
— is macro-generated or dead code and SHALL NOT count as coverage: the scanner skips such a body (the
same macro-body exclusion the static and semantic dimensions apply, reimplemented louke-locally
because 三儀 ⊥ 三儀 forbids importing 圭表's scanner).
Otherwise a probe in a never-invoked macro body would report its seam covered while the seam never
enforces at runtime — the audit's forbidden false negative. The scan is lexical and does not
evaluate `cfg`: a probe behind a non-production `#[cfg(...)]` is still counted, so a seam's
production probe must not live behind a non-production `cfg` — a stated bound, not a silent pass.
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
other name SHALL remain excluded — a stated bound, shared across all three dimensions. Observation
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
constitution error **unless** the declaration carries a `#[cfg(...)]` or `#[cfg_attr(...)]` gate, in
which case the module may legitimately have no file in the current configuration (an off feature or
another platform) and SHALL be skipped rather than errored — the same cfg-tolerance the semantic
dimension applies, reimplemented louke-locally (三儀 ⊥ 三儀). This does not evaluate `cfg`: a
*resolvable* cfg-gated module is still scanned and its probes still counted; only an *absent* file
for a cfg-gated declaration is tolerated. A resolution ambiguity (both `name.rs` and `name/mod.rs`
present) remains a constitution error regardless of any gate. A declaration written inside a
**transparent control-flow macro arm** (`cfg_if!`) SHALL be reached by this walker and treated as
**cfg-conditional**: the walker SHALL descend each arm of such an invocation as a transparent scope
— with the enclosing module bases unchanged, since an arm adds no directory component the way an
inline `mod` does — so an arm-declared `mod` enters the reachable corpus and its file's probes are
counted, and an absent conventional file (or absent unconditional `#[path]` target) for it SHALL be
tolerated exactly as under a bare `#[cfg]`, because the arm's predicate lives in the macro's
`if #[cfg(..)]` header rather than on the item and every arm is conditionally compiled by
construction. Arm membership SHALL NOT be inherited into an inline `mod { … }` body descended from
within an arm, matching the bare-`#[cfg]` case, and SHALL NOT make a resolution ambiguity tolerable.
Without this, every probe beneath an arm-declared module was invisible — the coverage false negative
this walker exists to prevent, reached through the module graph rather than the probe scan.

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
the worst outcome under FN-first). A `cfg_attr`-wrapped `#[path]` is cfg-conditional and SHALL NOT be
followed — following it cfg-blind could read a file rustc does not compile in this configuration — so
such a module is not counted (a stated bound); an absent target for it is tolerated like any cfg-gated
module, while an absent target for an unconditional `#[path]` is a fail-loud constitution error.

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

#### Scenario: A cfg_attr-wrapped #[path] relocation is not followed

- **WHEN** a target root declares `#[cfg_attr(unix, path = "unix_seam.rs")] mod plat;` with no conventional `plat.rs`, where the relocated file would hold the only probe for a declared seam
- **THEN** the cfg-conditional relocation is not followed (a stated bound, tolerated as a cfg-gated absence, not a constitution error) and the seam is reported unprobed — never a silent claim that a file rustc may not compile here is covered

#### Scenario: A semicolon inside an earlier attribute's own string does not hide a later #[path]

- **WHEN** a module declares `#[doc = "Handles A; falls back to B."]` immediately followed by an unconditional `#[path = "relocated.rs"] mod inner;`, and `relocated.rs` (not the conventional, absent `inner.rs`) holds a declared seam's probe
- **THEN** the `#[path]` attribute is still recognized and followed to `relocated.rs` — the bare `;` inside the preceding `#[doc]` attribute's own string value must never be mistaken for the preamble's own start, which would desync the attribute scan and silently lose the `#[path]` relocation (a false hard-failure on a module that genuinely compiles, or a misattribution to the wrong file)

#### Scenario: A brace-delimited attribute argument does not hide an earlier #[path]

- **WHEN** a module declares an unconditional `#[path = "relocated.rs"]` immediately followed by an unrelated `#[foo({ 1 })]` (a brace-delimited token-tree argument, not a string literal) before `mod inner;`, and `relocated.rs` holds a declared seam's probe
- **THEN** the `#[path]` attribute is still recognized and followed to `relocated.rs` — the brace-delimited attribute argument's own internal `{`/`}` bytes must never be mistaken for a top-level item boundary, which would desync the attribute scan and silently lose the `#[path]` relocation the same way an earlier attribute's own string content could

#### Scenario: Legacy directory callers remain compatible

- **WHEN** a direct caller passes a source directory instead of a target root file
- **THEN** the audit retains the recursive directory scan and the caller requires no source change
