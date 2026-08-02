## MODIFIED Requirements

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
`#[cfg_attr(a, cfg_attr(b, path = "…"))]` is a stated, undetected bound of this hand-rolled scanner
(unlike `hunyi`'s `syn`-based recursive resolution of the identical shape), never a silent claim of
coverage.

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
