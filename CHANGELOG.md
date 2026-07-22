# Changelog

All notable changes to the 天衡 (Tianheng) crate family. This is the **adopter-facing**
projection of the release history; the per-change *why* lives in the squashed change commits and
their pull requests. The format follows [Keep a Changelog](https://keepachangelog.com/en/1.1.0/).

Versioning is **SemVer honesty** for a pre-1.0 line (see `AGENTS.md`): the family is
**experimental / pre-1.0**. It held at `0.1.x` deliberately until real adopters arrived; `0.2.0` is
the first deliberate minor past that hold. Pre-1.0, additive depth on an existing observation source
and packaging/hygiene are patch releases, a breaking change earns a minor, and no release
intentionally breaks the adopter-written builder (`Constitution` / boundary DSL / `run`).

## [Unreleased]

### Added
- Semantic `RuleKey` and `StructuredFactIdentity` inspection across 圭表, 渾儀, 漏刻, 璇璣, and
  `tianheng::prelude::*`; all three instruments remain directly adoptable and return the same
  structured reaction model.
- Explicit machine-contract formats: `tianheng.baseline/structured-facts`,
  `tianheng.reaction/structured-facts`, and `tianheng.constitution/declared-boundaries`.

### Changed
- **Breaking:** violation and baseline identity is now exactly governed target + semantic rule key
  + structured fact identity. Rule/finding wording and all diagnostics remain available but cannot
  affect matching, ordering, or SARIF fingerprints.
- **Breaking:** SARIF partial fingerprints now use `tianheng/structured-fact-identity`, derived
  solely from canonical semantic identity; `tianhengViolationId/v1` is no longer emitted.

### Removed
- **Breaking:** `FindingKey`, presentation-derived `ViolationId` construction, numeric baseline
  generations, legacy text matching, and automatic baseline upgrade behavior.

### Migration
- Preserve desired `owner` / `tracker` annotations externally, move or delete the old baseline,
  run `tianheng check --write-baseline <file>`, then restore annotations onto the newly observed
  facts. Unsupported existing files are never overwritten. There is no automatic adapter.
- Architecture tests should call an existing standalone `check*` function or
  `check_constitution`, then assert on `Violation::rule_key()` and `Violation::fact()`. This release
  adds no plugin protocol or `tianheng::testing` DSL.

### Compatibility evidence
- Pacta `d3e24df`'s unpublished `pacta-governance` consumer compiled against this checkout's local
  `tianheng` and `guibiao` crates (`cargo check -p pacta-governance`) from a temporary copy; no
  Pacta source migration was required. This is recorded external evidence, not a sibling-repository
  dependency of Tianheng's required CI.

### Documentation
- Refined core project documentation density (`PROJECT.md`, `BACKLOG.md`) to archive verbose historical post-mortems and prune redundant release ledgers, reducing context token overhead.

## [0.2.3] - 2026-07-22

### Fixed
- 渾儀's forbidden-marker self-type resolver (`resolve_self_type`) now routes through the crate's
  own hop-capped alias/re-export fixpoint instead of a second, hand-rolled loop guarded only by an
  exact-repeat check — closing a real unbounded-loop gap (a divergent, non-cycling alias rewrite
  chain the exact-repeat guard alone cannot catch) and, as a side effect, an alias-resolution false
  negative (a member reached through an aliased *prefix*, not just an exact alias key, now lands).
- 圭表 now reacts (a constitution error) when a plain `mod x;` resolves to BOTH `x.rs` and
  `x/mod.rs` at once — a genuine `rustc` compile error (E0761) it previously accepted silently as
  two separate sources, dual-governing one module path. Matches 漏刻's own probe scanner, which
  already reacted on this exact shape.
- 渾儀's single-module-anchored resolver (`descend`) now tolerates a `#[cfg]`-gated `mod x;` with
  no backing file, matching its own crate-wide walker's (`resolve_child_modules`) existing policy —
  the two previously disagreed, so a boundary anchored directly at a `#[cfg]`-gated module hard-
  failed even when a mutually-exclusive per-platform sibling (e.g. an inline arm) legitimately
  resolved it.
- 漏刻's CI probe-coverage scanner now canonicalizes its module-cycle dedup guard (via a new,
  additive `xingbiao` dependency gated behind the non-default `audit` feature — never reaches the
  production hot path), matching 圭表/渾儀's own guards. Previously deduped on the literal path
  only, so a symlinked directory or circular `#[path]` chain reached via two distinct literal paths
  to the same real file could make the scan misbehave instead of terminating cleanly.
- 漏刻's CI probe-coverage scanner no longer tolerates a missing conventional module file merely
  because the item carries ANY `#[cfg]`/`#[cfg_attr]` attribute. Verified against a real `rustc`
  build: unlike a bare `#[cfg(pred)]` (which genuinely removes the item when `pred` is false),
  `#[cfg_attr(pred, …)]` never removes the item — only conditionally applies its wrapped
  attribute — so a `#[cfg_attr(unix, allow(dead_code))] mod x;` with no backing file is a real,
  unconditional compile error (E0583) that was previously silently skipped by the audit.
- 圭表 and 渾儀 now tolerate a missing unconditional `#[path]` target when the item also carries a
  co-occurring bare `#[cfg(pred)]` — a standard per-platform shim (`#[cfg(windows)] #[path =
  "windows_impl.rs"] mod imp;`) that previously hard-failed on any platform whose target file
  wasn't committed, even though rustc itself strips the whole item, `#[path]` included, before
  ever resolving it when `pred` is false (verified against a real build).
- 圭表 now reacts (a constitution error), rather than silently dropping the module from
  `reachable`, when a plain `mod x;` with no backing file carries no `#[cfg]` at all — closing a
  longstanding cross-dimension coverage gap (渾儀 already hard-erred on the identical shape). A
  `#[cfg]`-gated missing file is still tolerated, matching 渾儀. A boundary anchored directly at a
  module whose sole declaration was `#[cfg]`-tolerated away now reacts as an unknown module
  (never a vacuous clean pass), matching 渾儀's own resolver's identical precedent — unless an
  inline sibling arm of the same name exists, in which case the self-describing inline-target
  error still applies (never misreported as a generic "unknown module, check the path" error).
- 圭表's and 漏刻's independent `#[path]`-string decoders now handle backslash-newline line
  continuation (`"a\` + newline + `b"` decoding to `"ab"`), matching `syn` (used by 渾儀) and real
  `rustc` behavior. Previously 圭表 silently dropped such a remapped module from `reachable` with
  no error, and 漏刻 fell back to (or hard-errored on) the conventional location instead of
  following the real target.

### Changed
- Internal refactor: modularized crate internals across `xuanji`, `xingbiao`, `guibiao`, `hunyi`, `louke`, and the `tianheng` runner's projection layer (deduplicated JSON/text boundary-projection rendering) — no public API, JSON wire format, or self-governance boundary changed.

## [0.2.2] - 2026-07-22

### Fixed
- 圭表 module reachability now walks into an inline `mod parent { … }` body to find its own
  file-backed declarations, so a child reached only through an inline parent (`mod parent { mod
  child; }`, compiling `parent/child.rs`) is observed and its imports are checked.
- 圭表 now follows an unconditional, direct `#[path = "…"]` module declaration to its real target
  (matching 渾儀 and 漏刻), so a relocated module's imports are observed by all three observation
  dimensions. A `cfg_attr`-wrapped `#[path]` remains excluded (cfg-conditional, never followed
  cfg-blind).
- Every declared source for a module name is now observed, cfg-blind: an inline module body's own
  nested declarations, a plain conventional file, and an unconditional `#[path]` remap of the same
  name under mutually-exclusive `#[cfg]` arms (the standard per-platform shim) are all governed,
  regardless of attribute order or which source is scanned first. A plain (`#[path]`-free) `mod
  child;` declared inside a file reached through an unconditional `#[path]` remap is now governed
  under its logical path.
- A `#[path]` inside one mutually-exclusive `#[cfg]` arm's target — or inside a plain child of that
  arm — that legitimately references a sibling arm's own target (the two are never simultaneously
  open in any real build) is no longer misreported as a module cycle. Plain-child resolution now
  tracks each source's own directory context (where a `#[path]` written in it resolves, and
  separately, where its own plain/inline children live) instead of resolving through a shared
  structural index.
- A plain child reached only through a **symlinked directory** component, and an inline module
  preceded by an unconditional `#[path]` header (which relocates the base its own file-form
  children resolve from), are both now followed and governed correctly.
- 渾儀's single-module resolver (backing signature-coupling, visibility, dyn/impl-trait, and
  async-exposure anchors) now unions every mutually-exclusive `#[cfg]` variant of a module — inline
  and file-form alike — instead of stopping at the first match, and resolves a segment nested
  beneath a split point, or a `#[path]`-loaded module's own conventional children, from that
  variant's own directory rather than a name-derived or shared one. Two `#[cfg]` arms plainly
  declaring the identical name (resolving to one real file) are deduped by canonical path so they
  never inflate one violation into two.
- A `use`-map, and the child-module/re-export/rename tables it depends on, are now computed **per
  branch** of a `#[cfg]`-split module rather than once over the flattened cross-branch union —
  closing false negatives where one branch's own `use` alias or genuine re-export was silently
  shadowed or overwritten by an unrelated, mutually-exclusive sibling branch. Two purely-inline
  `#[cfg]` siblings sharing one enclosing file are split into their own branches for this purpose,
  not just file-form ones.
- A finding's reported `file` is now attributed **at collection time**, carried from the exact
  `#[cfg]` branch that produced it, rather than re-resolved afterward from a module-path string —
  so a violation written in a non-first branch is reported at its own file, never an innocent
  sibling's.
- The subtree walker backing `.including_submodules()` now descends every surviving `#[cfg]` branch
  independently, each from its own resolved `#[path]` base, instead of collapsing several branches
  to one shared directory pair for further descent.
- A self type that resolves to the enclosing `impl`'s own declared generic type parameter —
  written as a bare identifier, a projection (`T::Assoc`), or a qualified path (`<T>::Item`) — is
  no longer resolved through a same-named `use` alias, in both the forbidden-marker acquisition
  gate and the trait-impl-locality owner label. This closes a false-positive marker finding and a
  dedup-collapse false negative where two distinct `MisplacedImpl` violations were silently
  reported as one.
- `async_exposure`'s subtree scan now assigns a continuously-incrementing ordinal across the whole
  walk, never reset per module — closing a dedup-collapse false negative where two
  mutually-exclusive `#[cfg]` branches of one async fn, each carrying an unrenderable const-generic
  self type, collided on the same fallback identity and were reported as a single finding.
- 漏刻's probe-coverage scanner now locates a `mod` declaration's own attribute preamble with a
  forward, literal- and attribute-group-aware scan, replacing a backward raw-byte scan that could
  desync on a bare `;`/`{`/`}` inside an earlier attribute's string value or a brace-delimited
  attribute argument — closing false hard-fails and wrong-file substitutions on valid, compiling
  code.
- 圭表's crate-boundary rules (`forbid_dependency_on`, `restrict_dependencies_to`,
  `restrict_workspace_dependencies_to`, `restrict_dependency_sources_to`, and the
  feature-granularity rules) no longer observe a crate's own self-referential dependency on
  itself — a real, Cargo-legal pattern (e.g. a `[dev-dependencies]` path dependency on `.`, used
  for doctest/dogfooding) that names no other crate at all, so it can never be the cross-crate
  concern any of these rules exist to govern. The exclusion lives in the shared dependency
  observation itself, so every crate rule is covered at once.

## [0.2.1] - 2026-07-21

### Changed
- Published finding schemas and their dimension-local canonicalizers are now exhaustively pinned as
  compatibility reactions. Human finding wording remains presentation and is deliberately not
  snapshot-frozen.
- The baseline guide now documents the existing `--write-baseline` operation as the bounded,
  explicit V1-to-V2 upgrade path, including metadata carry-forward and stale-entry removal.
- 圭表 `must_not_import` now documents a stated partial-coverage bound: a `use`-glob of an
  *ancestor* of the forbidden module (`use crate::*;` while forbidding `crate::secret`) is observed
  at the glob's base, not as the forbidden descendant edge, so it does not react — forbid or confine
  the parent. The narrow `use crate::secret;` / `use crate::secret::*;` forms are caught as before.

### Fixed
- 渾儀 unsafe-confinement now qualifies a **trait-impl** `unsafe fn` by `<trait for self>`
  (`unsafe fn <A for Foo>::m`), not its self type alone: on one self type, an inherent `unsafe fn m`,
  `impl A for Foo { unsafe fn m }`, and `impl B for Foo { unsafe fn m }` are three distinct sites and
  now stay three findings. Previously all collapsed to `unsafe fn Foo::m`, so a baseline of one
  silently accepted a later-added trait-impl `unsafe fn` on a safe trait — a false negative, the
  trait-impl case 0.2.0's notes already claimed owner-qualified. *Baseline note:* this changes the
  `finding_key` of a trait-impl `unsafe fn`, so a 0.2.0 baseline entry for one resurfaces on upgrade
  and must be re-accepted (`--write-baseline`); unsafe-confinement is one release old, so the
  affected surface is minimal.
- Baseline `owner` / `tracker` metadata now rejects non-string JSON values instead of silently
  erasing malformed governance data; the CLI gate fails as a constitution error and explicit
  rewrite retains its warning-before-recovery behavior.
- Runtime probe coverage now starts from every exact Cargo library and binary target root and walks
  only module-reachable source, so an orphan `.rs` file can no longer satisfy a seam it never
  enforces. Direct callers that pass a directory retain the legacy recursive corpus.
- 渾儀 and 漏刻 now **follow** an unconditional `#[path = "…"] mod x;` to its author-chosen file,
  closing a coverage false negative: a relocated module's `unsafe` sites, trait impls, and
  `assert_boundary!` probes were previously dropped, so a disallowed impl or an undeclared-seam probe
  in a relocated module passed unobserved (semantic single-module boundaries on such a module errored
  loudly rather than governing it). The target is resolved with rustc fidelity — relative to the
  containing file's own directory, accumulating each enclosing inline-`mod` name as a directory
  component (so `mod inline { #[path="p.rs"] mod inner; }` reads `inline/p.rs`), with the path
  literal's escapes decoded as rustc and syn do; the two independent dimensions resolve the same
  file, and two declarations sharing one target (or a conventional `mod` plus a `#[path]` alias to
  it) are governed under each path rather than misread as a module cycle. A `#[path]`-loaded file is
  mod-rs-like, so its own children resolve from its directory. A `cfg_attr`-wrapped `#[path]` stays a
  stated bound — not followed cfg-blind, since it could observe a file rustc does not compile in this
  configuration — and an absent unconditional target is a fail-loud constitution error. Both
  dimensions detect the attribute structurally, so an incidental `path` substring in a comment or a
  `#[cfg(feature = "fastpath")]` gate is never mistaken for a relocation. As with any false-negative
  closure, a downstream carrying a real violation inside a relocated module may see green CI turn red
  on upgrade — adopt via `warn` / `Baseline` (the same patch-level precedent as the v0.1.3 re-export
  closure).
- The probe-coverage walker now tolerates a `#[cfg(...)]`/`#[cfg_attr(...)]`-gated module whose file
  is absent in the current configuration (an off feature or another platform), skipping it instead
  of failing the audit — matching the semantic dimension, so a cross-platform workspace no longer
  hard-errors on a platform-specific module. A non-cfg missing module and a resolution ambiguity
  remain fail-loud.

## [0.2.0] - 2026-07-20

The first **breaking** window since `0.1.0` — a deliberate `0.2.0` minor (the `0.1.x` hold ended
when real adopters arrived). The break is quarantined to internal identity/model surfaces; the
adopter-written builder is a drop-in swap (see **Compatibility**).

### Added
- **`tianheng::check_constitution`** — one inspectable composed reaction over the static (圭表),
  semantic (渾儀), and runtime (漏刻) dimensions in a single call, sharing the runner's evaluator
  (static-first error precedence, runtime orphan-probe auditing) without going through the CLI.
- **Adopter surface contract.** The composed wildcard `prelude` is now an explicit,
  compile-checked external compatibility promise, with a symmetric `ModuleRule` inspection path;
  hidden granular checks stay outside the promise.

### Changed
- **BREAKING — structured violation identity.** Violation matching moved from rendered finding
  *text* to dimension-owned **structured keys**: `Violation::new` now takes a typed `ViolationId`,
  and newly-written baselines use version-2 `finding_key`s (fact-specific named fields) instead of a
  rendered descriptor. 渾儀's semantic findings derive both their diagnostic text and their key from
  one typed fact model. Reports stay byte-identical.
- **BREAKING — 圭表 rule model surface narrowed.** `Rule` / `ModuleRule` are now
  builder-constructed only — downstream can no longer construct or exhaustively destructure their
  data-carrying variants (open-ended *inspection* stays available through the boundary accessors);
  `InlineExternalStrict` is folded into `Inline`. Reaction, projection, polarity, and violation
  identity are unchanged.

### Fixed
- 渾儀 unsafe-confinement: `unsafe fn` findings are now **owner-qualified** (`unsafe fn {owner}::{m}`)
  for inherent, trait-declaration, and trait-impl methods, so two same-named `unsafe fn`s on
  different owners in one out-of-subtree module no longer collapse to one finding — closing a
  baseline-masking false negative (the `unsafe fn` sibling of 0.1.8's `unsafe impl` closure).
- 圭表 inline-symbol-path confinement (`must_not_call_inline`): a `use`-group member whose name
  merely *starts with* the substring `self` (e.g. `use chrono::{self_utc as clk}`) is now resolved
  rather than dropped, so a confined inline call through such an alias reacts — closing a false
  negative.
- 渾儀 single-module resolution: a module split across `#[cfg(…)] mod x { … }` **inline variants** now
  has every variant governed (matching the crate-wide scan's observe-all), so a forbidden exposure
  in a non-source-first variant reacts — closing a `mod`-resolution false negative.

### Compatibility
- The **adopter-written builder** (`Constitution`, `CrateBoundary`, `ModuleBoundary`, the boundary
  DSL, `run`, `prelude`) is a **drop-in swap** — the break is quarantined to the internal
  `Violation` / `ViolationId` / baseline wire and 圭表's rule-model surface.
- **Baseline migration.** Version-1 baselines are still read (exact-text match), so existing
  baselines keep grandfathering; a baseline rewritten under this release upgrades to the version-2
  structured form.

## [0.1.10] - 2026-07-15

### Added
- 圭表 **feature-granularity crate-dependency boundary** — `CrateBoundary::crate_(…)`'s
  `restrict_features_of(C, […])` / `forbid_features_of(C, […])` / `forbid_feature(C, f)` govern
  which features a crate *declares* on a dependency `C`: its explicit `features` list plus the
  `default` pseudo-feature (so `forbid_feature(C, "default")` ≡ requiring `default-features =
  false`), matched by package name and unioned across the target's dependency edges. It observes
  the **declared** request only — never expanding `C`'s own `[features]` graph and never reading
  `cargo metadata`'s resolved `resolve.nodes[].features` — so it is stable under Cargo feature
  unification and builds under the existing `--no-deps` metadata read with no new dependency.
  Findings are `C/feature` (kind-qualified when the dependency kind is not `Normal`), injective
  across the two polarities; severity, baseline, dependency-kind selection, and the text/JSON
  projection reuse the existing crate-rule machinery. Transitive/unification-enabled features are
  an explicit non-goal (declared-not-resolved, at the altitude of the existing dependency rules).
  Additive and non-breaking; existing constitutions and baselines are unaffected. See
  `COOKBOOK.md`.

### Changed
- Contributor-facing docs only: `AGENTS.md` makes the project's practised conventions explicit
  (document authority, OpenSpec lifecycle, adversarial review, single-source Definition of Done,
  branch prefixes, subject-only release commits); `BACKLOG.md` records the `0.1.x → 0.2.0` trigger
  and the install-vs-constitution decision; the `README.md` license section links to its files.

## [0.1.9] - 2026-07-11

### Added
- 圭表 `must_not_call_inline(…).strict_external()` — **opt-in**: also catch a *fully-qualified
  external-crate* call (e.g. a bare `chrono::Utc::now()` with no `use chrono`), closing the
  asymmetry where a sysroot head (`std::time::…`) was caught but a fully-qualified external head was
  silently resolved as local. A bare head matching a declared dependency is resolved as that crate,
  after a local-precedence ladder so a genuinely-local item of the same name stays local at any
  nesting depth. Composes with `.ending_with` / `.strict_prefix_only`; with the flag off the default
  is **byte-identical**, so existing constitutions and baselines are unaffected. Carried as a new
  `#[non_exhaustive]` rule variant (patch-safe; identity-parity, no baseline churn), and 圭表 grows
  its own rename-aware dependency-name reader — no dependency on 渾儀 (三儀 ⊥ 三儀), still `syn`-free.
  Stated bounds (an `extern crate … as` rename; and, under a single-segment prefix, a local binding
  or a definition site that reads as a call) are declared, never a silent pass.
- Adopter cookbook recipes (`COOKBOOK.md`): test that a boundary reacts, gate workspace coverage in
  CI, why exposure rules are deny-shaped (not a "may only expose" allowlist), and the
  `strict_external` recipe. `README.md` gains a "what the instruments do **not** see" note, so a
  reader does not over-infer a dimension's reach (渾儀 reads a signature's types/traits, never a
  call site).

### Changed
- Internal refinement, behavior-preserving and no public-API change: 渾儀's whole-crate-scan
  capabilities share one violation-emission helper; the text projection shares a module-block
  helper; idiom/consistency cleanups; and `xingbiao` now carries `#![deny(missing_docs)]` like its
  five sibling crates.

## [0.1.8] - 2026-07-11

### Added
- 圭表 inline-symbol-path confinement — forbid a crate from *calling* a fully-qualified path inline
  (e.g. `std::time::SystemTime::now()`), resolving `use` renames / aliases / re-exports and the
  glob-danger shapes. The syn-free static complement to observing a `use`-import.
- 渾儀 `UnsafeBoundary` — declare that a crate's `unsafe` (blocks, `unsafe fn`/`impl`/`trait`,
  `unsafe extern`) may appear **only under** a declared subtree
  (`UnsafeBoundary::in_crate("app").only_under(["crate::ffi"])`): the auditability boundary of a
  layered crate, the confinement complement of `#![forbid(unsafe_code)]`.
- 渾儀 visibility ceiling — `max_visibility(Crate | Super | Module)`, generalizing the binary
  `must_not_declare_pub` into a rank ceiling (an item declared above the ceiling reacts; the prior
  rule is now the `max_visibility(Crate)` sugar, byte-stable in findings).
- 渾儀 async-exposure opt-in **subtree** scope — `.including_submodules()` descends the anchored
  module's whole subtree, so a "this seam is synchronous" boundary governs a pure kernel throughout,
  not only at its own seam.
- Every crate declares `#![forbid(unsafe_code)]` — the family is `unsafe`-free and says so at
  compile time.
- `examples/` gained `unsafe-confinement` and `sans-io-pure`, plus a `max_visibility` demo in
  `hunyi-standalone`.

### Fixed
- 渾儀 unsafe-confinement: the finding is owner-qualified (`unsafe impl {trait} for {self type}`), so
  two `unsafe impl`s of one trait for different self types in a module no longer collapse to one
  finding — closing a baseline-masking false negative.
- 渾儀 / 圭表: a nested `#[cfg_attr(pred, path = "…")]` module remap is recognized in both dimensions,
  closing a silent false negative in the static scanner and the semantic subtree walk.
- 圭表 type-alias resolution skips a defaulted generic parameter's `=`
  (`type Clock<Tz = LocalTz> = std::time::SystemTime;` now resolves to its real target), closing a
  false negative where a confined type reached through the alias passed unobserved.

### Changed
- modou is no longer framed as superseded. It is a living, independently-developed sibling project;
  Tianheng's static core (圭表) is *derived from* it, and Tianheng keeps all three dimensions
  (README / PROJECT).
- README gained a Phase-0 one-line on-ramp (lock one seam, enforce, pipe SARIF into CI) above the
  full multi-dimension example.

## [0.1.7] - 2026-07-08

### Added
- 圭表 `confine_external_crate` — confine an **external** crate's `use` imports to one declared
  module subtree (FFI / platform-vocabulary confinement): `ModuleBoundary::in_crate("app")
  .module("crate::ffi").confine_external_crate("libc")` reacts when any module outside
  `crate::ffi`'s subtree imports `libc`. The first static rule to *observe* external-crate imports
  (every other rule ignores them), source-observed — not a `cargo metadata` dependency-table rule.
  The confined crate is the violation target, so confinements of different crates on one module stay
  distinct in the baseline. A package name written with a `-` (e.g. `windows-sys`) matches its
  underscore import identifier (`windows_sys`).
- `COOKBOOK.md` — a cookbook of common governance intents expressed as declared boundaries (圭表 /
  渾儀 / 漏刻 recipes), the imitable surface an adopter or agent copies rather than translating a
  foreign policy format.
- Coloured, reason-first terminal output for the human `check` report — a severity-coloured header
  (red for an enforced violation, yellow for an advisory) over the emphasised reason. Presentation
  only: gated to an interactive terminal (honours `NO_COLOR`), so a pipe, a redirect, or a CI log
  stays byte-identical, and `--format json` / `sarif` are never coloured.
- `examples/` — three runnable, self-checking examples: `guibiao-standalone` (the syn-free static
  import linter), `hunyi-standalone` (the semantic public-API exposure linter), and `composed`
  (the `tianheng` shell governing one app with all three instruments, in a CI-time `check` mode and
  a runtime `run` mode).
- Per-instrument GitHub issue templates (圭表 / 渾儀 / 漏刻).

## [0.1.6] - 2026-07-07

### Changed
- Extracted the `cargo metadata` substrate into a new `xingbiao` crate — a `serde_json`-only base
  beneath the dimensions — so the static and semantic dimensions read the workspace through one
  source of truth instead of two hand-copied twins.

### Fixed
- 渾儀 forbidden-marker: closed two false negatives — a hand `impl` whose self-type is spelled
  through a `pub use` re-export, and a locally-renamed (`use … as`) trait/derive leaf.

## [0.1.5] - 2026-07-07

### Added
- 圭表 `must_only_be_imported_by` — the closed inbound dual of `must_not_be_imported_by`
  ("only `crate::facade` may import `crate::internal`").

### Fixed
- 漏刻 probe-coverage audit: a probe inside a `macro_rules!` body no longer counts as coverage.
- Recorded a documented robustness bound in the `use`/`mod` lexer around multibyte char literals
  (no confirmed false negative).

## [0.1.4] - 2026-07-05

### Fixed
- 圭表 module-source hardening: module boundaries use Cargo's observed `src_path`, and
  `#[path]`-remapped and inline-only orphan modules are excluded rather than governed through a
  same-named conventional file.
- Packaging: every publishable crate now physically bundles its `LICENSE-MIT` / `LICENSE-APACHE`
  texts (`cargo publish` ships only crate-local files; 0.1.0–0.1.1 shipped without them). Guarded
  by a CI reaction.

## [0.1.3] - 2026-07-02

### Added
- 渾儀 semantic depth: public re-export exposure and trait-impl exposure.

## [0.1.2] - 2026-07-02

### Added
- 圭表 `restrict_dependency_sources_to` — govern the declared dependency source kind
  (git / registry / path).
- 渾儀 `dyn`-trait and `impl Trait` boundaries, and async-exposure.

## [0.1.1] - 2026-06-30

### Fixed
- Early packaging and metadata hygiene.

## [0.1.0] - 2026-06-29

### Added
- Initial release of the crate family: the `xuanji` reaction model, the three observation
  instruments — 圭表 (`guibiao`, static), 渾儀 (`hunyi`, semantic), 漏刻 (`louke`, runtime) — and
  the 天衡 (`tianheng`) shell that composes them into one `check` with a `0` / `1` / `2` exit
  contract and `--format json` / `sarif` projections.

[Unreleased]: https://github.com/tacticaldoll/tianheng/compare/v0.2.3...HEAD
[0.2.3]: https://github.com/tacticaldoll/tianheng/compare/v0.2.2...v0.2.3
[0.2.2]: https://github.com/tacticaldoll/tianheng/compare/v0.2.1...v0.2.2
[0.2.1]: https://github.com/tacticaldoll/tianheng/compare/v0.2.0...v0.2.1
[0.2.0]: https://github.com/tacticaldoll/tianheng/compare/v0.1.10...v0.2.0
[0.1.10]: https://github.com/tacticaldoll/tianheng/compare/v0.1.9...v0.1.10
[0.1.9]: https://github.com/tacticaldoll/tianheng/compare/v0.1.8...v0.1.9
[0.1.8]: https://github.com/tacticaldoll/tianheng/compare/v0.1.7...v0.1.8
[0.1.7]: https://github.com/tacticaldoll/tianheng/compare/v0.1.6...v0.1.7
[0.1.6]: https://github.com/tacticaldoll/tianheng/compare/v0.1.5...v0.1.6
[0.1.5]: https://github.com/tacticaldoll/tianheng/compare/v0.1.4...v0.1.5
[0.1.4]: https://github.com/tacticaldoll/tianheng/compare/v0.1.3...v0.1.4
[0.1.3]: https://github.com/tacticaldoll/tianheng/compare/v0.1.2...v0.1.3
[0.1.2]: https://github.com/tacticaldoll/tianheng/compare/v0.1.1...v0.1.2
[0.1.1]: https://github.com/tacticaldoll/tianheng/compare/v0.1.0...v0.1.1
[0.1.0]: https://github.com/tacticaldoll/tianheng/releases/tag/v0.1.0
