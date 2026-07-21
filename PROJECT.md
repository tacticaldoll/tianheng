# Project Contract — 天衡 (Tianheng)

Tianheng's orientation layer for humans and AI agents. Keep it short and concrete.

## Purpose

Tianheng is a Rust-native **reactive architectural-governance** framework. It does not
run your app and it does not instruct your agent; developers and agents propose change,
and Tianheng uses compiler/CI and runtime **reactions** to keep
architectural shape from drifting. The source of truth is Rust code; TOML, Markdown, and
reports are projections of it.

It grows from **modou** (墨斗): modou proved the static dimension as a single focused crate,
and 圭表 is derived from it — but modou lives on as an independently-developed sibling project,
not a line Tianheng supersedes. Tianheng keeps that proven core and grows it into a **crate
family** of observation dimensions — without becoming a god crate.

## What Tianheng is — and is not

- It **is** reactive governance across **observation dimensions** (static, semantic, and
  runtime — all three 儀 built), each a separate crate the user selects by depending on it.
- It is **not active shaping**: it observes and reacts; it does not generate or prescribe
  structure. (Active shaping is a different axis, deliberately deferred — adopting it would
  be a conscious amendment, not drift.)
- It is **not a framework** in the prescriptive sense: you do not build *inside* it. It is
  a CLI + library.
- It is **not a lint**: every dimension must be real drift — declared intent vs. observed
  reality — never an opinionated style check.
- It is **not a supply-chain policy engine**: resolved, whole-graph dependency policy —
  advisories, dependency licenses, bans / duplicates, resolved source allowlists — is
  cargo-deny's lane (run in this repo's `supply-chain` CI job). Tianheng governs the
  *declared, per-target, architectural* layer instead (deps / imports by name, declared
  dependency-source kind for manifest hygiene, type exposure, impl locality, visibility,
  runtime seams). The two are complementary, not overlapping — the reason resolved
  build-provenance is cargo-deny's, not a Tianheng capability (see the 圭表 depth decision).

## Core Contract

A **declared boundary reacts.** A boundary declared in Rust must produce a real,
non-bypassable reaction when violated — for the CI dimensions, a CI failure with a non-zero exit and
an explanatory report. The reaction MUST never silently pass, and MUST distinguish a
boundary violation (exit 1) from a constitution error / misconfiguration (exit 2). The
one forbidden bug is a **false negative** (a real violation that silently passes).

## 潛移 (Qiányí) — govern by gravity, too: the idiom is imitated, the reaction is the backstop

The reaction binds, but an autoregressive agent is first an **imitation engine** — it
continues whatever idiom sits in its context. So compliance has a second source,
complementary to the non-bypassable reaction: **潛移 (qiányí) — gravity, the quiet pull of an
idiom** (潛移默化: it is assimilated without being told). The more the declared law and the
governed code read *as one strong, distinctive idiom in the agent's context*, the more its
continuations stay in-shape by default — and invocation stops being an act the agent must
remember and becomes an **emergent property of imitation**. This is neither instruction
(dictating what to write) nor bare reaction (catching it after); the agent is *pulled*, not
pushed or told — still consistent with "we do not instruct your agent."

Hence a standing design principle: **every Tianheng-facing semantic surface — the
declaration DSL, the `because`/`reason` prose, and the law's projections (`list --format
markdown` foremost) — is designed to be *imitated*, not merely *read* or *parsed*.**
Legibility serves a human; imitability serves the continuation engine, and the two optimize
differently — density, distinctiveness, and reason-as-first-class-prose over exhaustive
enumeration. But imitability is **bounded by the drift law**: an imitable surface states the
forward shape its boundary *observes and reacts to*, never structural guidance beyond that
perimeter. A reason that pulls the agent toward a shape Tianheng cannot react to is prose
prescription — the open loop this project exists to close, smuggled back in as text.

Gravity does **not** replace the reaction; it relocates it. Imitation transports *surface
form*, never *invariants* — a strong idiom still admits a locally-plausible violation. So
gravity lowers the base rate of drift (the frictionless primary) and the non-bypassable
reaction forecloses what gravity misses (the backstop). The one forbidden bug is still a
false negative, and only the reaction can foreclose it.

## Inherited laws (from modou — non-negotiable)

- **Drift law** — *No drift type without an observation source. No target type or name
  without a reaction.* Names are not claimed for reactions that do not yet exist; this
  holds at module, crate, and **dimension** granularity (we do not pre-create empty
  `semantic`/`runtime` crates).
- **Minimalism bound** — fail loud only on *observable misconfiguration*; no defensive
  over-foolproofing of impossible states.
- **SemVer honesty** — pre-1.0, a release that breaks no public API is a **patch**, never
  a vanity minor bump. (modou's hard-won lesson.)

## Architecture — a crate family, not a god crate

- **`xuanji` (璇璣) — the 底 (bedrock).** The dimension-agnostic **reaction model** the
  whole stack turns on: `Severity`, `BoundaryKind`, `Violation`, `Report`, `Baseline`,
  `Outcome`, with the JSON serialization intrinsic to those types. `serde_json`-only; carries no observation
  engine, and depends on no workspace member — every dimension sits above it.
- **`xingbiao` (星表) — the workspace-data substrate.** The star-table: the shared,
  `serde_json`-only reader of `cargo metadata` (`cargo_metadata` / `find_package` /
  `crate_root_file`), sitting below every dimension like 璇璣 and depending on no workspace
  member. It is *not* 璇璣 — it does IO (it spawns cargo) and observes — but a substrate beneath
  the dimensions, so the static and semantic dimensions read the workspace through **one** source
  of truth, not two hand-copied twins that drift apart (the v0.1.6 SSOT extraction — see Decisions).
- **`guibiao` (圭表) — the static observation core.** The gnomon: it reads the cast
  shadow — imports, dependencies, and inline symbol-path calls (the clock-free
  `must_not_call_inline` confinement). The dependency-light static engine, derived from
  modou: declare crate- and module-import boundaries, observe from `cargo metadata` (read
  through 星表) and source `use` / symbol scans, compare, react. Pure functional core — no shell. Depends on `xuanji`
  (the reaction model), `xingbiao` (the metadata substrate), and `serde_json` only; the
  report/constitution *assembly* (which folds in the static `Coverage`) lives here, not in the model.
- **`tianheng` (天衡) — the shell.** The celestial balance that weighs declared against
  observed: the imperative shell + facade — CLI (arg parsing, filesystem, stdout/stderr),
  the `run` reaction that composes every dimension into one, and the re-exported public
  surface. Depends on every dimension it composes (`guibiao` + `hunyi` + `louke`). It is also where
  cross-cutting **composed profiles** live (e.g. `sans_io_pure`, folding a 圭表 clock-free and a 渾儀
  synchronous-API boundary into one declaration) — a dimension never composes its sibling, only the
  shell does (三儀 ⊥ 三儀).

**Functional core ⊥ imperative shell, at crate granularity.** `guibiao` must not depend
on `tianheng`. This is the crate-level upgrade of modou's module-level `engine ⊥ runner`,
and Tianheng enforces it on itself (`crates/tianheng/tests/self_governance.rs`) — eating
its own dog food, now across crate boundaries.

**A dimension is a crate born when built** (drift law at crate granularity), and the user
selects governance by depending on the dimensions they want:
- **`hunyi` (渾儀)** — AST/semantic observation (`syn`). **Built (v0.1.0):**
  signature-coupling (a module's public API must not *expose* a forbidden type), plus
  trait-impl locality, visibility, and forbidden-marker boundaries; **(v0.1.2):** a
  type-shape/existential **depth stair** on the same `syn` source — dyn-trait and impl-trait
  exposure (each shape-only *and* named-operand-scoped) and async-fn exposure — the type-shape
  and existential complements of signature-coupling; **(v0.1.3):** two further same-source depth
  additions to that flagship exposure surface — **re-export exposure** (a named public `pub use`
  of a forbidden type is itself an exposure, default-on — an API-compatible but behavior-changing
  false-negative closure) and **trait-impl exposure** (the opt-in `.including_trait_impls()`
  depth, surfacing a trait impl's impl-site-authored positions); **(v0.1.8):** a **visibility
  ceiling** (`max_visibility(Crate|Super|Module)` — the binary `must_not_declare_pub` generalized to
  a rank), **`unsafe`-confinement** (`UnsafeBoundary::only_under([…])` — `unsafe` confined to a
  declared subtree, the non-compiler-expressible complement of `#![forbid(unsafe_code)]`), and an
  opt-in **whole-subtree scope** for async-exposure (`including_submodules`) — all detailed in the
  Decisions section. The heavy `syn` dependency is quarantined here, never in the core.
- **`louke` (漏刻)** — runtime observation. **Built (v0.1.0):** origin-assertion (a
  declared seam's `only_origins` allowlist), in two faces — the prod probe
  (`assert_boundary!`, fail-closed, a structured event by default, panic opt-in) and the
  `audit_probe_coverage` CI face, composed into `tianheng check`. Ships into the production
  binary; hot path is std-only, depends on 璇璣 only. (Design gate resolved — see Decisions.)

**The observatory vocabulary (manifested in governance).** The three observation
dimensions — 圭表 (static), 渾儀 (semantic), 漏刻 (runtime) — are the **三儀** (the three
instruments): *what* Tianheng measures; each is a crate born when built, each adds a new
drift type. The governance & observability layer — 垂象 (the reaction surface), 實錄 (the
baseline), 校讎 (the amendment flow) — are the **三司** (the three offices): *how* a reaction
is surfaced, recorded, and amended (see `BACKLOG.md`). 儀 measures, 司 administers — the
三儀 add what is observed, the 三司 wrap the reaction. Both are crate-or-convention as their
nature dictates, never named before their reaction exists.

## Naming — narrative, with meaning in the SSOT

Crate and concept names are **coined / narrative** (圭表, 渾儀, 漏刻), in the celery/kombu
tradition: a name is a stable handle, not a self-description. Meaning lives in the
authoritative **metadata SSOT** (each crate's `description` + docs) — fitting for a tool
whose own thesis is "the source is the SSOT; names are projections." The brand `tianheng`
(天衡) and the bedrock `xuanji` (璇璣) split the one master instrument, 璿璣玉衡: 璣 → 璇璣,
the jade pivot every measurement aligns to; 衡 → 天衡, the balance that weighs declared
against observed. The brand is a star (玉衡, in the Dipper's handle), not an instrument — so
it sits cleanly above the 三儀 it wields, sharing no name with any of them.

潛移 (the gravity thesis above) deliberately breaks the celestial pattern: it names neither
an instrument (儀) nor an office (司) but a **mode of governance** — compliance by imitation —
so it is drawn from the idiom 潛移默化 (change that assimilates without the subject's
awareness), not from 璿璣玉衡. It is a handle for *how* the declared law spreads, parallel to
govern-by-reaction, never a thing the tool wields.

## Decisions

Record significant decisions here (the *why*; specs and code carry the *what*).

- **Reborn from modou as a crate family.** modou was taken as frozen/complete at its own
  `0.1.1` when Tianheng was reborn; Tianheng started fresh (clean git history, clean SemVer
  from `0.1.0`) rather than expanding modou's single crate into a god crate. The runtime
  dimension *must* be a separate crate (it ships into production and must stay light), so a
  family is the destiny — but members are born only when built. *(Amended 2026-07: modou is
  unfrozen and now develops independently in its own repo — a living sibling, not a superseded
  ancestor. Tianheng retains all three dimensions including the static 圭表, does not reroute
  static-only users to modou, and 圭表 stays derived-from-modou by lineage. How modou evolves —
  including whether it depends on `guibiao`/`tianheng` — is out of Tianheng's scope; the two do
  not share a workspace, so no shared-shell / born-when-built commitment is pulled forward.)*
- **The static core is `guibiao`, not `tianheng-core`.** Named by its stable identity
  (the gnomon, the static instrument, modou's derivative), not by a temporary role ("the
  whole core back when it was the only dimension").
- **Cross-crate visibility is the price of the split.** Items modou kept `pub(crate)`
  (baseline, coverage, projection, `check_and_cover`) are `pub` in `guibiao` because the
  shell consumes them across the crate boundary. This widens the engine's public API
  beyond modou's minimal `check` — acceptable, and refinable pre-1.0.
- **Baseline is a generated snapshot, not policy.** A baseline records accepted
  violations so a dirty project can adopt a boundary and gate only on *new* drift; it is
  a projection of the report, never the constitution.
- **Module imports are observed by scanning source `use` declarations**, not by parsing
  a full AST. A hand-rolled scanner keeps the 圭表 core dependency-light and macro-free;
  its partial coverage — bare path expressions and macro-generated imports are out of scope —
  is acceptable because the drift law only enforces what is observed. An unconditional, direct
  `#[path = "…"]` remap is **followed** to its target (0.2.2), matching 渾儀/漏刻, so all three
  observation dimensions agree on what rustc compiles; a `cfg_attr`-wrapped `path = "…"` stays a
  cfg-conditional exclusion from the conventional module graph (following it cfg-blind could read
  a file rustc does not compile in the active configuration), so it fails loud rather than
  governing a same-named orphan. Comments and
  string literals (normal, byte, and raw) are stripped so their text is never mistaken
  for a `use`. A module's identity is derived in three places — its file path, its `mod`
  declaration, and a `use` path that names it — and these MUST stay in lockstep, since a
  divergence both fails to govern a real module and silently hides its imports (a false
  negative, the one thing the core contract forbids). Two consequences stay token-level,
  not parser-level, to keep the hand-rolled scanner: a raw identifier is canonicalized
  (`mod r#type;` compiles to `type.rs`, so `r#type` and `type` are one module), and a
  `use` is attributed to the inline `mod { … }` that encloses it (so `self`/`super`
  resolve correctly); macro bodies are stripped before scanning for `mod` declarations
  too, not just `use`s, so the out-of-scope rule for macro-generated items is symmetric.
  Adopting a real parser (`syn`) would resolve all of this for free but would break the
  dependency-light core (the `serde_json`-only self-law); that is an amendment, not a
  silent trade. A boundary's governed *target* is file-based: an inline `mod name { … }`
  is reachable for import attribution but owns no file, so it cannot be a target — a
  boundary on one fails loud with a self-describing constitution error (exit 2), distinct
  from an unknown-module typo, never a silent pass. Governing inline modules as targets is
  a deliberate non-goal here; if ever wanted it is a separate amendment.
- **A cfg-blind union that makes several physically distinct files share one logical identity
  must never let per-module bookkeeping (an ancestor set, a structural lookup) act on that shared
  identity as if it named one file.** **(0.2.2 lesson, in eight rounds — confirmed across
  two independent crates.)** Landing the
  unconditional-`#[path]`-follow work (above) first tracked cycle-guard "already-open source
  files" in one `HashSet` keyed by the *logical* module path string. Two `#[cfg]` arms of the same
  name with different unconditional targets — the standard, already-supported per-platform shim —
  share one logical path, so their targets' canonical paths were unioned into that single set. A
  further `#[path]` inside one arm's target legitimately referencing the *other* arm's target (the
  two are never simultaneously open in any real single build) was then misreported as a cycle. A
  first-round fix moved ancestor tracking onto each `ScanSource` individually — but a **second**
  adversarial-review round on the shipped result found the *same* mistake still live one hop away,
  in the sibling plain-child code path added alongside it: `plain_child_base_ancestors` still
  unioned a module's *entire* source list rather than only the source that actually declared the
  child, reproducing the identical false-positive cycle through a plain (non-`#[path]`) child of an
  inline `#[cfg]` arm; a stray, wholly uncompiled file coincidentally sitting at a remapped
  module's naive structural location could also be phantom-governed alongside the real one, since
  the (then still `by_module`-based) plain-child lookup had no way to know its parent was
  remapped; and a plain child resolved through the `#[path]`-remap live probe had no working
  formula for *its own* further plain children at all (a false negative), since a directory-shape
  concept the probe needed (where do THIS file's own plain children live — different from where
  a `#[path]` written inside it resolves, whenever the file is an ordinary flat `name.rs` rather
  than `mod.rs`-shaped) was never modeled as its own field, so it silently fell back to whichever
  adjacent value happened to be at hand. All three were root-caused together: plain-child
  resolution was redesigned to carry, on every `ScanSource`, both `path_base` (where a `#[path]`
  written in it resolves from) and a *separate* `child_base` (where its own plain/inline children
  live — these coincide only for the crate root, an inline body, and a `#[path]` target, never for
  an ordinary flat file), and the global `by_module` structural index was dropped from child
  resolution entirely in favor of this per-source probe — closing all three at once rather than
  patching each symptom where it surfaced. The durable lesson, confirmed twice at increasing
  depth: keyed identity for *governance* (what to report a violation under) and keyed identity for
  *safety* bookkeeping (what counts as "the same thing already open", or "where do this file's own
  children live") are not automatically the same key, and a fix that narrows to the one reported
  instance rather than the shared underlying model tends to resurface one hop further away on the
  very next adversarial pass — which is exactly what a second review round is for.
  A **third** round, re-auditing 渾儀's own single-module resolver (`descend`) redesigned alongside
  the fixes above, found the identical shape of mistake in a second, independent crate: a
  `#[path]`-loaded module's own *conventional* children were resolved from a name-derived directory
  unrelated to where the file actually lives (the exact inverse of the `path_base`/`child_base`
  conflation above), and two non-inline (file-form) `#[cfg]` siblings — one plain, one
  `#[path]`-remapped — were still not unioned, first-match-wins, unlike the crate-wide walk's own
  already-established "never stop at one match" policy for the identical shape of declaration. The
  lesson holds a third time, now confirmed independently in two separate crates built by two
  separate reaction models (a stateful worklist walk in 圭表, a recursive branch-vec descent in
  渾儀): a redesign that closes a reported instance is not evidence the *shared model* is closed —
  only a fresh adversarial pass against the *new* code, not the old bug, earns that confidence.
  A **fourth** round found the mistake yet again, this time at the boundary between 渾儀's
  single-module resolver and its OWN subtree walker: `resolve_module_root` correctly unions every
  surviving branch's items for a single-module report, but the subtree walker fed that union back
  through only the *first* branch's own directories for further descent — a child declared only in
  a non-first branch's own file resolved against the wrong directory, silently dropping it. Fixed
  by exposing every branch on its own (not the collapsed, first-branch-only shape) to the one
  caller that must keep descending beneath a possibly-split anchor. The same round also found a
  duplicate-items case in the opposite, lesser-severity direction: two `#[cfg]` arms plainly
  declaring the identical name resolve to one real file, but nothing deduped the resulting
  branches, inflating one real violation into two apparently-distinct findings. Four rounds, one
  recurring root cause: a data shape (a single merged tuple) built for one caller's needs (report
  "the" file) got reused by a different caller (keep descending) whose needs it does not actually
  meet — the same "governance key vs. safety key" conflation the lesson opened with, now seen at
  a third layer of the same two crates.
  A **fifth** round, scoped to the newest surface area above plus a fresh cross-crate consistency
  pass, found two further instances in 圭表 — this time not a repeat of the governance/safety-key
  conflation, but the SAME "a fix closes the reported instance, not the shared model" pattern
  applied to the plain-child live probe's own completeness. The probe's `structurally_matches`
  check only compared the candidate's naive on-disk path against its logical module path — it
  never checked whether that candidate was actually a member of the crate-wide walk's own file
  list (`fs_walk::rust_files`, which deliberately never recurses into a symlinked directory, its
  own cycle guard). A plain child reached only through a symlinked directory component agreed on
  path but was never in that list, so the probe wrongly assumed `governed_files`' structural
  iterator would find it on its own — silently governing nothing. Separately, the inline-module
  scan path never read a `#[path]` attribute preceding an inline header at all (a comment claimed
  it was "a no-op for rustc", true only for the header's own content, not for where its file-form
  children resolve from — verified false against a real build). Both are the same shape as the
  lesson's opening case: a check or a scan that is locally reasonable for the ordinary case turns
  out to silently assume away a real, rustc-compiling shape it was never actually tested against.
  Fixed by intersecting the live probe's structural-match test against the walk's own canonical
  file set, and by reading and following an inline header's own unconditional `#[path]` exactly
  like a file-form one. The same round found 渾儀's OWN single-module resolver (`descend`)
  mishandling the identical inline-`#[path]` shape in the mirror-image direction: its
  inline-collection loop excluded ANY `#[path]`-bearing item before ever checking whether it had
  inline content, and its file-form loop then also skipped it (a stale comment assumed it was
  "already collected above"), so the item vanished from both loops — a hard, spurious "module not
  found" (exit 2) on a module that demonstrably exists and compiles, for every single-module-
  anchored capability, while 渾儀's own crate-wide walker (`resolve_child_modules`, maintained as
  an independent function) resolved the identical layout without trouble. Two crates, opposite
  failure directions (圭表 too permissive — silently governing nothing; 渾儀 too strict — hard-
  erroring on a real module), the SAME root shape: a scan/check written for the ordinary case
  silently assumed away a real one it was never tested against. Fixed by unioning `descend`'s
  inline items regardless of `#[path]`, while still following an unconditional one to relocate
  the base its own file-form children resolve from — matching `resolve_child_modules`'s existing,
  correct handling of the identical syntax. Five rounds now, the same two-crate pattern each time:
  closing a reported instance earns confidence only from the NEXT fresh adversarial pass, never
  from the fix itself.
  The same round's fifth finding was different in KIND, not degree: `resolve_module_root`'s
  single-file shape (branches[0]'s file, alongside every branch's unioned items — the deliberate,
  documented choice from round 4) turned out to disagree with the CLI report's own published
  promise (`openspec/specs/cli-check-runner/spec.md`: `file` names "where the offending seam is
  written") once a `#[cfg]`-split module's finding genuinely came from a non-first branch — a real
  violation could be reported at an innocent sibling's file. `push_multi_module_violations`'
  `per_finding_file` cache had the identical hazard one hop further: two findings sharing one
  module string (a legitimate cfg-split) resolved through a cache keyed only by that string, so
  the second finding silently inherited the first's file. Unlike the four one-hop directory/dedup
  fixes above, closing this one properly meant `Violation.file` could no longer be resolved from a
  single module string AFTER the fact at all — it needed to be collected AT THE SITE, while the
  real file is still open, and carried alongside every item/finding from there to the violation.
  `resolve_module_items_with_files` now pairs every item with its own branch's file; `ImplSite`,
  `TypeDef`, and `UnsafeSite` (the crate-wide scan's own site records) and the subtree walker's
  per-module output now each carry their own real file directly. `seam_file` and `per_finding_file`
  — the module-string-keyed re-resolution layer this whole hazard lived in — are gone; there is no
  longer a caller that resolves a finding's file from anything other than the exact branch that
  produced it. A genuine schema change, not a same-shape patch — confirmed as necessary rather than
  deferred, since the disagreement was with a published, adopter-facing report guarantee, not an
  internal implementation comment.
  A **sixth** round, adversarially reviewing round 5's own changes, found the symlinked-directory
  fix (round 5, above) had reintroduced the identical false-negative SHAPE through the fix's own
  new check: `structurally_matches` compared the live-probed candidate's CANONICAL (symlink-
  resolved) identity against a set of the crate-wide walk's own files' canonical identities. Two
  on-disk paths that resolve to the same physical file are not the same module — `#[path]` remaps
  sharing one target is the existing, correctly-handled case for exactly this reason (rustc
  compiles the same bytes twice as two distinct modules) — but a canonical-identity set cannot
  distinguish "this exact candidate was walked" from "some OTHER file that happens to alias the
  same physical target was walked": a module reached only through a symlinked directory that
  happened to alias an unrelated, genuinely-walked module's file was wrongly treated as "already
  found," silently un-governing it. The fix within the fix: compare LITERAL (never canonicalized)
  path identity instead of canonical identity — two distinct on-disk paths are never literally
  equal merely because they resolve to the same target, so the aliasing hazard cannot arise, while
  the original round-5 false negative (a candidate whose own literal path was never in the walked
  file list at all) is still caught exactly as before.
  The same round found a distinct, previously-unnoticed instance of the founding lesson in 渾儀:
  `module_findings` (exposure.rs) and `shape_module_findings`/`operand_module_findings`
  (shape_scan.rs) each called `collect_uses` ONCE over the flattened cross-branch union
  `resolve_module_items_with_files` returns — the identical items/file conflation the round-5
  redesign closed for FILE attribution, left unfixed for the `use`-map. Two mutually-exclusive
  `#[cfg]` branches each declaring `use <different path> as Handle;` (a realistic per-platform
  shim) collided in the one shared map; the branch unioned last silently overwrote the earlier
  branch's alias, misresolving the FIRST branch's own bare reference through the SECOND branch's
  `use` and hiding a real forbidden-exposure violation — a genuine false negative, not a
  hypothetical. Fixed the same way round 5 fixed file attribution: a `use`-map PER FILE, looked up
  by each item's own branch file rather than one map merged across the whole module.
  The same round found a THIRD instance, this time in `scan.rs`'s own `resolve_child_modules` (the
  shared descent skeleton backing the whole-crate scan and the async-exposure subtree's further
  descent): `module_resolve.rs::descend` gained a `seen_files` dedup in 0.2.2 for two
  mutually-exclusive `#[cfg]` arms plainly declaring the identical name (resolving to the one real
  file) — but `resolve_child_modules`, a separately-maintained sibling walker for the identical
  shape, never received it. Because a self-type's generic argument `canonical_self_owner` cannot
  render falls back to a positional `_#{ordinal}` marker computed from the scan `Vec`'s own
  position, the two duplicate scan entries got DIFFERENT ordinals and escaped the eventual
  fact-identity dedup — inflating one real trait-impl or forbidden-marker acquisition into two
  apparently-distinct findings. Fixed with the identical `seen_files`-style guard, keyed on
  (declared name, canonical file) rather than file alone, since two DIFFERENT names `#[path]`-
  remapped to the same file are two real, separately-compiled modules — an existing, already-
  tested case that must never collide with this new dedup entry (a live regression caught by the
  existing test suite the moment the naive file-only key was tried). Six rounds now: even a fix
  produced BY an adversarial-review round is not exempt from the next round's scrutiny — the
  lesson applies recursively to its own remedies, not just the original bug, and a redesign that
  closes one data shape's cross-branch conflation (files) does not by itself close a SIBLING data
  shape's identical conflation (use-maps, or a sibling walker's own missing dedup) computed from
  the same underlying union.
  A **seventh** round confirmed exactly the sibling gap round 6's own writeup had flagged but left
  unconfirmed: `exposure.rs::module_findings` fixed its `use`-map per file in round 6 but left
  `child_mods`/`externs_type`/`externs_reexport`/`renames_bare` computed once over the SAME
  flattened cross-`#[cfg]`-branch union — a branch with no local `mod net;` had its own genuine
  `pub use net::Something;` (the real extern crate) silently suppressed merely because a
  MUTUALLY-EXCLUSIVE sibling branch happened to declare its own local `mod net`. The identical
  shape existed one layer down in `crate_scope.rs::extern_resolution` (feeding
  `shape_scan.rs::operand_module_findings`, backing the dyn-trait/impl-trait operand-scoped
  boundaries) — `externs_type`/`renames_bare` there were *also* computed once over the flattened
  union, never split by file. Both fixed the same way as the `use`-map: a per-file `FileScope`
  (exposure.rs) / `FileExternScope` (crate_scope.rs) computed from each branch's own items alone,
  looked up by each exposure's own file — never a value shared across mutually-exclusive branches.
  This is the clearest demonstration yet of the recurring lesson: round 6 fixed ONE instance
  (`use`-maps) of a conflation that had FOUR siblings in the same function (and a fifth in a
  parallel function one file away) — flagging the risk in the fix's own commit message was
  necessary but not sufficient; only the next round's adversarial pass, armed with a real
  constructed repro, actually confirmed and closed it.
  An **eighth** round found that rounds 6 and 7's own "per file" fix rests on an assumption that
  is true for a file-form `#[cfg]` sibling (each gets a genuinely distinct file, guaranteed by
  round 4's per-branch design plus round 6's same-file dedup) but FALSE for an **inline** one:
  `descend()` still merged every same-named inline `#[cfg]` occurrence into ONE shared `Branch`
  before round 6's per-file grouping ever ran, so two inline siblings — the standard shape,
  `#[cfg(a)] mod x { .. } #[cfg(b)] mod x { .. }` — were never separated in the first place; the
  round-6/7 fix was structurally a no-op for this shape, and the original cross-branch conflation
  reappeared one hop further in, unnoticed because every prior round's repro used at least one
  file-form sibling. Fixed in two parts: `descend()` now gives every inline occurrence its OWN
  branch (mirroring the file-form loop's existing "every declaration produces its own branch"
  policy), and — since two inline siblings still share one identical ENCLOSING file even once
  split into separate branches — `resolve_module_items_with_files` now pairs each item with a
  **branch index**, not just a file, and every consumer (`exposure.rs`'s `FileScope`,
  `shape_scan.rs`'s `uses_by_branch`/`operand_module_findings`'s `file_scopes`) groups by that
  index instead of by file alone. `async_exposure.rs`'s subtree path
  (`scan.rs::walk_subtree_modules`/`collect_subtree`) needed no matching consumer-side change:
  its own child walker (`resolve_child_modules`) already gave each inline occurrence its own
  entry (verified — it never merged same-named inline siblings the way `descend()` did), and
  each subtree entry already builds its `use`-map from its own items directly, never grouped
  across sibling entries by file. Eight rounds now, the sharpest form of the lesson yet: "grouped
  by file" was itself an unexamined stand-in for "grouped by branch" that happened to coincide for
  every shape tested through round 7 — the fix that finally closes it replaces the file key with
  the actual identity the whole model has been about since round 4.
- **A round-9 adversarial pass, scoped to fresh surfaces the eight rounds above never touched
  (forbidden-marker's own self-type gate, and 漏刻's probe scanner), found two unrelated bugs —
  a different bug class each, not the cfg-branch conflation above.** First: `forbidden_marker.rs`'s
  marker-acquisition gate (`resolve_self_type` in `containment.rs`) resolved a bare `impl` self type
  exactly like any other path reference, with no notion that the identifier might be the impl
  block's OWN declared generic type parameter rather than a nominal type — so a blanket
  `impl<T> Marker for T {}` in a module that also happened to declare an unrelated `use … as T`
  alias resolved `T` through that alias, fabricating a marker-acquisition finding on a type the
  source never actually impls the marker for. The sibling exposure collectors already shadow an
  impl's own generic-param names for every OTHER position (`collect.rs::type_param_names`,
  guarding the existing `a_trait_impl_generic_param_shadowing_an_alias_is_not_exposed` regression),
  but that shadowing was never carried into the crate-wide scan's `ImplSite` (backing
  forbidden-marker, trait-impl-locality, and unsafe-confinement) or into the self-type gate
  specifically. Fixed by giving `ImplSite` its own `type_params: HashSet<String>` (the impl's
  declared generic names, via the now-`pub(crate)` `type_param_names`) and threading it into
  `resolve_self_type`, which now drops a bare self type matching one of them before any resolution
  is attempted — the same "stated bound, never a silent claim" treatment already given to any other
  non-placeable self-type shape. Second: 漏刻's `mod_preamble_attrs` (the probe scanner's own
  attribute-preamble reader) found where a `mod name;`/`mod name {`'s preamble begins by scanning
  **backward** from the `mod` keyword for the nearest raw byte equal to `;`/`{`/`}` — the only
  traversal in the whole file that was NOT literal/comment-aware, unlike every other walk there,
  which routes through `skip_literal_or_comment` for exactly this reason. An EARLIER attribute's own
  string value containing a bare `;` in ordinary prose (`#[doc = "Handles A; falls back to B."]`)
  stopped the backward scan mid-literal, desyncing the forward attribute walk that followed: it read
  the string's own closing quote as the opener of a bogus new string, swallowing a later
  `#[path = "…"]` attribute's own `#` inside it — so the scanner never saw the real relocation and
  either hard-failed on a module that genuinely compiles, or silently substituted the wrong
  (conventional) file. Fixed by replacing the backward raw-byte scan with a **forward**,
  literal-aware scan from the enclosing scope's own start (a real, always-outside-any-literal
  boundary) up to the `mod` keyword, tracking the last `;`/`{`/`}` seen outside any literal/comment —
  matching every other traversal in the file. Both confirmed with a real, executed, then-reverted
  repro; both closed root-cause, not deferred, since both are genuine false positives / false
  hard-failures on valid, `cargo build`-clean code, not a narrow or hypothetical edge.
  A **round-10** pass, adversarially re-reviewing round 9's own two fixes one hop further (the same
  discipline the cfg-branch lesson above established, now shown to apply beyond that one bug class
  too), found both were incomplete. `resolve_self_type`'s new shadow check recognized ONLY a bare
  single-segment self type (`Path::get_ident()`, which returns `None` for anything but exactly one
  segment) — so `impl<T> Marker for T::Assoc {}` (a projection off the impl's own parameter, never a
  nominal type) was still never shadowed and still resolved through a same-named alias, the
  identical false positive one segment deeper. The sibling exposure collector
  (`resolve/shape.rs::PathCollector::is_shadowed_param`) already had the RIGHT shape — shadow the
  leading segment regardless of how many further segments follow — so the fix extracted that check
  into a shared `is_shadowed_param_path` function and pointed both call sites at it, closing the
  drift at its root (two independent copies of "what counts as a param use" is exactly the shape
  this whole lesson warns against) rather than patching `resolve_self_type`'s own copy again. The
  same round found `canonical_self_owner` (the self-type **label** renderer used to build
  `trait_impl.rs`'s `MisplacedImpl` owner field, among others) had received NO shadow treatment at
  all — not a narrower copy, none — despite `ImplSite` carrying `type_params` since round 9
  specifically for this purpose. This was not merely a cosmetic mislabel: `owner` is part of
  `MisplacedImpl`'s finding IDENTITY, deduplicated by exact equality, so a blanket impl's own
  parameter resolving through an alias to the SAME canonical string a genuine direct impl on that
  aliased type also produces silently collapsed two textually and semantically distinct
  trait-impl-locality violations into one reported finding — a real false negative (one violation
  vanishing), not just a wrong display string. Fixed by giving `canonical_self_owner` the identical
  `impl_type_params` shadow via the same shared `is_shadowed_param_path`, threaded through its seven
  call sites (`collect.rs` ×5, `forbidden_marker.rs`, `trait_impl.rs`). And 漏刻's `mod_preamble_attrs`
  forward scan (round 9) was literal-aware but not attribute-group-aware: it tracked no nesting, so
  a brace-delimited attribute ARGUMENT (`#[foo({ 1 })]`, a valid token tree, not a string literal)
  between an earlier real `#[path = "…"]` and the `mod` keyword had its own internal `{`/`}` bytes
  mistaken for item-boundary terminators, resetting the scan past the real attribute — the identical
  failure mode round 9 closed, reached through a different vector. Fixed by skipping a whole
  `#[…]`/`#![…]` group as one atomic unit (via `attr_group_end`, the same primitive the existing
  attribute-matching pass already used) when scanning for the preamble's own start, and likewise
  skipping a non-attribute `{…}` (a preceding sibling item's own body) via `balanced_brace_end` to
  its own matching `}` rather than examining its interior bytes. Ten rounds now: a fix that closes
  the CONFIRMED instance is not evidence the shared model — or even that SAME fix's own new check —
  is complete; only the next round's fresh adversarial pass, aimed specifically at the prior round's
  own remedy, earns that confidence, and this now holds across at least two structurally distinct
  bug classes in this codebase, not only the cfg-branch one.
- **圭表's source concern is the declared layer; the resolved layer is cargo-deny's, not ours.**
  **(v0.1.2)** crate-source-boundary (`restrict_dependency_sources_to`) is the static
  dimension's first **depth** addition — like 渾儀's dyn-trait, it deepens a proven reaction
  (dependency governance) on the *same* observation source (`cargo metadata --no-deps`, the
  declared manifests), reading the `source` field one notch finer (git vs. registry vs. path)
  rather than widening to a new source. It reads the **declared** layer, and that is the right
  SSOT for its intent — manifest hygiene / publishability: a published manifest is rejected for
  naming a git source (an *optional* git dep included), while `[patch]`/`[source] replace-with`
  is workspace-local, never part of the published manifest, and never blocks `cargo publish`, so
  a patch-redirected registry dep correctly reads `Registry` and does not violate. The mirror
  concern — **resolved build-provenance** ("what my build *actually* pulls from", the *resolved*
  graph after lockfile + `[patch]` applied) — catches the patch-redirect the declared layer is
  blind to, and in turn misses an optional-off git dep; neither layer dominates (A governs
  optional-git and is patch-blind; the resolved layer is the inverse). But that resolved,
  **whole-graph** concern is **cargo-deny's lane**, not Tianheng's: `deny.toml [sources]` (run in
  the `supply-chain` CI job) already denies unknown git/registry sources on the resolved graph — so
  a `[patch]`→git redirect surfaces there — and a whole-graph view fits build-provenance better than
  Tianheng's per-target model. So Tianheng **declines** resolved build-provenance (a would-be
  *capability B*) rather than deferring it: A is not an incomplete B, it is the whole of Tianheng's
  source concern — the hermetic, declared, per-target layer (no lockfile, no network), the
  complement cargo-deny does not cover. A stated
  second bound: A is source-kind *hygiene*, not a `cargo publish` oracle — a `{ git, version }`
  dep declares a git source and is flagged though it would publish (the rule does not parse
  `version`), deliberately conservative.
- **`xuanji` is an internal refactor, not a spec'd capability.** When the second
  dimension (渾儀) is built it needs the shared reaction DSL — `Severity`, `Baseline`,
  `Violation`, `Report`, `Outcome` — without `guibiao`'s static engine, so those leaf
  types extract into a `xuanji` crate, re-exported from `guibiao` to preserve its
  public API. The crate is **`serde_json`-only**: `Baseline` *is* a JSON snapshot and the
  per-type renderings (a `Violation`/`Outcome` → JSON value, baseline (de)serialization) are
  intrinsic to those types, so they move with them; `serde_json` is the family's one
  sanctioned dependency. But the **report/constitution assembly stays in `guibiao`/the
  shell**: `report_json` folds in `Coverage` (`workspace_crates`/`uncovered`) and stale
  baseline entries, and `constitution_json` walks `Boundary`/`Rule`/`DependencyKind` — all
  static-dimension concepts that must not leak into the dimension-agnostic model. So the
  split is *per-type serialization in the model, document assembly out of it*. This is a
  behavior-preserving prep step that changes no requirement, so — following modou's line
  that adopter-facing reactions live in specs while Tianheng's own architecture lives here
  and in `self_governance.rs` — it is **not** an OpenSpec capability change. Its invariants
  (model `serde_json`-only and below every dimension; dependency points model ← engine ←
  shell, never the reverse) are enforced as `cargo test` self-governance boundaries, the
  same way the `engine ⊥ shell` law already is. `guibiao`'s *external*-dependency bound
  stays `{serde_json}`; its self-law uses the stricter `restrict_dependencies_to` (which
  governs internal paths too), so it was amended — by deliberate, human-reviewed change to
  `self_governance.rs` — to `["serde_json", "xuanji"]` (later `["serde_json", "xuanji", "xingbiao"]`;
  see the 星表 decision below), naming the internal paths the family split requires. 璇璣's own
  boundary `restrict_dependencies_to(["serde_json"])`
  keeps it beneath every dimension (no workspace member below it).
- **The semantic capability-admission test (the gate against lints).** `syn` makes
  opinionated checks trivial to write ("no `unwrap`", "fns ≤ 50 lines"), every one forbidden
  by the not-a-lint contract. So a semantic capability is admissible as a 渾儀 reaction
  **iff all three hold**: (1) **declarative, not lint** — the constraint has *no universal
  right answer* (two sane projects could declare the opposite and both be correct), so it is
  the developer's intent, not the tool's opinion; (2) **no *essential* gap** — its full
  observation surface is reachable from the local-crate AST, tolerating only *incidental*
  gaps defined by their nature (everything resolvable always reacts; the unresolved
  remainder is a stated bound, never silently passed) and never an *essential* one
  (downstream crates, inferred auto-traits, the transitive call closure), which would make
  it a false-negative engine; (3) **anchorable** — the target is a `syn`-resolvable element,
  and an unresolvable anchor is a constitution error (exit 2), never a silent pass. The
  first admitted capability is **signature-coupling** (a module's public API must not
  *expose* a forbidden type — the complement of import-governance, the case that provably
  earns the AST). Also admitted and now built (v0.1.0):
  local trait-impl surface (`only_implemented_in`), visibility (`must_not_declare_pub`), and
  forbidden-marker boundaries — each born when built. **(v0.1.2)** **dyn-trait-boundary**
  (`must_not_expose_dyn`) — the public API must not expose `dyn` trait-object *syntax*, the
  type-shape complement of signature-coupling and the first **depth** addition (it deepens a
  proven reaction's predicate from a named type to a type shape on the same `syn` source,
  rather than widening to a new dimension). It passes all three gates: declarative-not-lint
  (static dispatch at a *declared* seam is intent — by anchor scoping, not an operand),
  no *essential* gap (a `dyn` node syntactically present in the local-crate public surface is
  always observable; the residual is the inherited macro/alias bound), and anchorable (a
  `syn`-resolvable module). **(v0.1.2, same release)** its **named-operand depth**
  (`must_not_expose_dyn_of([…])`) — the next rung on the `name → shape → named-operand` stair:
  it refines the shape-only predicate ("any `dyn`") to "a `dyn` of a *named* trait", resolving
  each `dyn`'s **principal trait** (its sole non-auto trait, whatever its bound position)
  through the same 渾儀 resolver signature-coupling uses (exact-or-module-prefix, re-export
  canonicalization). It reuses the shape-only surface walk and the resolver, adding only the
  operand match — no new source, no new struct. An **empty** operand set degenerates to
  shape-only ("any `dyn`") — a loud over-reaction chosen deliberately over a silent no-op
  (`Of([])`), so a mis-declared operand set never becomes a false negative. Auto-trait markers
  are never operands (only the principal, non-auto, trait — matched regardless of bound position),
  and an unresolvable principal (a bare
  std trait, a macro/glob re-export) is the inherited resolver bound, never a silent pass of a
  *resolvable* operand. **(v0.1.2, same release)** its **existential sibling**
  `ImplTraitBoundary` (`must_not_expose_impl_trait`) — where dyn-trait forbids the *dynamic-
  dispatch* shape (`dyn`), this forbids the *existential* shape: a public seam must not **return**
  a written `impl Trait` (RPIT), an unnameable type that commits the seam to the hidden type's
  auto-traits. It passes the same gates: declarative (an existential at a *declared* seam is
  intent — and **argument-position `impl Trait`/APIT is deliberately not governed**, since it is
  *universal*, a caller-chosen generic, not a leak, which is what keeps this a boundary and not an
  `impl Trait`-style lint); no *essential* gap (a written `impl Trait` in a return position is
  always syntactically observable — `async fn`'s *implicit* `impl Future` and nightly TAIT are
  **distinct, stated-out-of-scope** forms, not silent misses of the written-RPIT domain); and
  anchorable (module). It reuses the public-surface walk and the `dyn` bound renderer, governing
  return positions only. Its **named-operand depth** (`must_not_expose_impl_trait_of([…])`, same
  release) climbs the same `shape → named-operand` stair as operand-scoped dyn — a returned
  `impl Trait` whose principal trait resolves into a forbidden set reacts (so a seam may allow
  `impl Iterator` yet forbid `impl crate::Port`); dyn and impl-trait were generalized onto one
  `ShapeExposure` collector and a shared `principal_trait_path`, so the two shapes share the
  operand machinery exactly. **(v0.1.2, same release)** its **implicit-existential complement**
  `AsyncExposureBoundary` (`must_not_expose_async_fn`) — an `async fn` leaks a compiler-inserted
  `impl Future`, so where impl-trait forbids the *written* existential this forbids the `async fn`
  sugar (observed from the pure AST flag `sig.asyncness`, over the same public-surface item kinds,
  trait-impl methods excluded). Its admission is the dimension's **weakest declarative** gate but
  holds: the intent is *implicit existential exposure at a declared seam* (a sync-core/async-edges
  layering), by anchor scoping, not a blanket "no async" lint; observability (a local AST flag) and
  anchoring are strong. Its finding is an **owner-qualified item identity**
  (`async fn <SelfTy>::name(…)`, `async fn trait <Trait>::name(…)`), NOT a bare name or a
  future-shape, because same-named
  public async fns across impls/traits in one module would otherwise collide under the
  `(target, rule, finding_key)` baseline and let a new leak be masked (the one forbidden bug).
  **(v0.1.2 hardening)** the sibling exposure findings now carry the same guarantee:
  signature-coupling, dyn-trait, and impl-trait findings are **seam-qualified**
  (`{type|shape} exposed by {seam}`, the seam being the owning item / sub-element — free fn,
  owner-qualified inherent method, trait method, field, variant, alias, const/static, or
  supertrait/associated position), so two distinct seams exposing the same type or shape no longer
  collapse to one finding — closing the same masking bug across every exposure rule, not only async.
  A future
  `must_not_expose_existential` could unify written + implicit, deferred until it earns admission
  without blurring those identities. **Rejected**, as explicit non-goals with their reason:
  `Send`/`Sync` constraints (auto-traits are inferred, never written), external trait
  sealing (downstream crates are outside the scan), and transitive effect-purity ("no I/O
  anywhere reachable") — each has an *essential* gap. This test is the standing gate: a new
  semantic capability passes all three in writing, or it is a lint and does not belong.
- **(v0.1.3) Two same-source depths on signature-coupling — one default-on, one opt-in — and the
  patch line they hold.** The flagship exposure surface gained two capabilities on the *same*
  `syn` source, both passing the admission test. (a) **Re-export exposure**
  (`semantic-reexport-exposure`) — a named public `pub use crate::infra::X;` republishes a
  forbidden type on the module's surface, so `must_not_expose` reacts to it (and its aliased /
  grouped / facade-chained forms; a glob re-export reacts when its root resolves in/under the
  forbidden set). It is **default-on**, because a missed public-surface item is not a new optional
  depth but a **false negative of the flagship** — the one forbidden bug — so per the core contract
  the closure cannot stay silent to keep CI green. That makes it the first 0.1.x change to alter an
  *existing* boundary's reaction (prior depths were additive/opt-in): an API-compatible
  **behavior-changing bugfix** — the DSL is unchanged and downstream still compiles, but a
  previously-green adopter may newly react to a real leak, adopted via the baseline/warn rung.
  Findings are seam-qualified by the *exported* path (`{type} exposed by pub use {path}`) so two
  aliases of one type never collapse under the baseline. (b) **Trait-impl exposure**
  (`semantic-trait-impl-exposure`) — the opt-in `.including_trait_impls()` depth extends exposure
  from a module's items to a trait `impl` block's **impl-site-authored** positions (the trait
  reference's generic args, the `Self` type, associated type/const bindings, the impl's own
  generics / `where`-clause including const-param types, and each method's *return* type);
  trait-*dictated* params and the receiver stay out of scope — they belong to the trait definition,
  already governed. It is **opt-in**, not default-on, precisely because a v1 spec declared trait
  impls out of scope and the impl-authored/trait-dictated split is a real narrowing choice, not a
  missed item — additive depth, not a bugfix. Both keep 0.1.3 on the **patch** line (SemVer
  honesty): the opt-in is purely additive, the default-on re-export is API-compatible. Before
  release, an adversarial review closed two further false negatives — a forbidden type in a
  trait-impl **const-generic parameter's type** (the generics walk matched only
  `GenericParam::Type`), and a **`{self}` facade chain** whose whole-module republish did not
  canonicalize back to the forbidden module — both restoring parity the code's own invariants
  already required, locked with regression tests. The two capabilities ship as OpenSpec changes
  (they carry specs); the two review-found fixes are conformance bugfixes within those specs,
  recorded in git, not new requirements.
- **(v0.1.4) External-crate exposure via the external-crate name set — adopter-driven, and the
  oracle that replaced an edition shortcut.** Signature-coupling reacted to an external type only
  when it arrived *use-aliased* (`use dep::Foo; … -> Foo` resolves through the `use`-map); an
  *inline, fully-qualified* extern path — a `pub use dep::spi::Foo;` re-export, a `-> dep::spi::Foo`
  signature, or a **local facade chain** ending at such a type — was silently dropped
  (`resolve_path` returns `None` for a bare extern head, and the re-export closure discarded an
  extern-headed target). All three are false negatives of the flagship, all observable from the
  local-crate AST — so per the false-negative-first contract the closure is **default-on** and,
  the DSL unchanged, on the **patch** line (the v0.1.3 re-export precedent), even though it now
  also touches the v0.1.0 flagship's inline-extern behavior. Driven by a real adopter (a facade
  whose "facade must not re-export core's spi" invariant lived only in doc prose). The
  extern-determination **oracle is the crate's external-crate name set** — declared dependencies
  (from the `cargo metadata --no-deps` already read, `.rename`-aware and `-`→`_` normalized to the
  path spelling) ∪ the sysroot crates (`std`/`core`/`alloc`/`proc_macro`/`test`, never in
  `dependencies` yet valid heads). A bare **`pub use`** head resolves against this raw set (extern
  by edition-2018+ grammar); a bare **type-position** head resolves against it **minus the
  governed module's own child modules** — a per-module shadow, so a local `mod serde` makes
  `serde::X` local (no false positive) without suppressing a subtree's real `pub use serde::X`
  (no false negative). A `PathExposure.is_reexport` bit selects which. The oracle sits in the
  bare-fallback branch **after** the `use`-map (a local `use … as dep` alias still wins) and is
  threaded **only** into the exposure resolve and the re-export closure — never `resolve_path`'s
  other callers, so dyn/impl-trait operand resolution and seam identity are untouched. **Three
  adversarial review rounds** shaped this: round 1 **refuted an initial edition-2018 grammar
  shortcut** (the tool reads no edition, so it mis-classifies edition-2015 heads); round 3, on the
  implementation, **caught a crate-level shadow that was both a false positive (nested local
  module) and a false negative (a subtree's extern re-export dropped)**, driving the per-position
  split above; round 2 drove the hardening — sysroot inclusion, hyphen normalization,
  module-shadow, and the call-site-scoped application. The residual stays a stated bound, never a
  silent pass: extern glob leaves and foreign-module renames (need the foreign AST), a source-level
  `extern crate dep as alias;` rename (invisible to `cargo metadata`, unlike the handled manifest
  `package =` rename), a dependency with a distinct `[lib] name`, a facade hop written as re-export
  of a privately-`use`d bare name (an inherited v0.1.3 closure bound), the edition-2015 relative
  local re-export, and `pub extern crate`. Ships as an OpenSpec change modifying
  `semantic-reexport-exposure` and `semantic-signature-coupling`. *(Retracted in part by the
  extern-crate-exposure Decision below: a **crate-root** `extern crate … as` rename and
  `pub extern crate` are now observed from the local AST and react — the residual narrows to a
  **module-scoped** source rename.)*
- **(v0.1.4) Declaration integrity — govern the integrity of the declaration itself, not only
  the governed code.** The 三儀 observe *governed code*; they do not observe the **consistency of
  the declaration and its artifacts** — the `Constitution` object, its projections
  (`constitution_markdown`), and hand-written narration that *indexes or asserts a structural
  property of* the constitution. Three reactions of exactly this shape already existed, each
  hand-rolled in a different place: the self-law byte-check (`self_law_projection_is_fresh`,
  projection ↔ constitution), `audit_probe_coverage` (declared seams ↔ probes), and an adopter's
  reason-drift test (`because` ↔ `AGENTS.md` prose). They are **one pattern**: a reaction whose
  *observation source is the declaration and its artifacts, not user code*. It is **not a new 儀**
  (it observes no governed code) and adds no drift in the governed graph; it sits beside 潛移
  (self-law) and 校讎, and is drift-law-compliant (a real observation source + a real reaction).
  Its bright line against a lint (the same discipline as the semantic admission test): migrate to a
  reaction only prose that **asserts a property of the declaration** (an index like "boundaries 2,
  3, 6", a coverage claim); never migrate prose that **explains a human choice** (rejected
  alternatives, rationale, perf/impl notes) — that has no observation source and stays prose. The
  first internal instance, built here: the cross-cutting 三儀 ⊥ 三儀 clause was cited by a
  hand-maintained index in `self_governance.rs`'s doc comment ("(boundaries 2, 3, 6)"), which had
  already drifted once (an off-by-one that swapped the `hunyi` dimension for the core⊥shell
  boundary — the class of bug that survives *because nothing observes it*). The fix is not to
  correct the numbers but to **delete the index and assert the property**: a test
  (`dimension_boundaries_declare_the_mutual_independence_law`) walks `constitution.boundaries()`,
  selects the three dimension allowlists (`restrict_dependencies_to` on `guibiao`/`hunyi`/`louke`),
  and asserts each `because` carries the clause — tianheng's own `semantic-forbidden-marker`
  philosophy (replace a prose reference with a reaction) turned on its own constitution narration.
  A stated bound: the predicate observes the `because` **text** (`contains`), weaker than a
  structural fact — a reworded clause would slip it — but it still reacts to the real drift a
  hand-maintained pointer could not. The **adopter-facing** forms are deferred, born when built (no
  API before a second consumer): (a) a small constitution-assertion helper so a structural-property
  reaction is not re-hand-rolled per repo, and (b) the 潛移 generator (§5) — though the v0.1.4
  byte-checked staleness-gate **recipe** already lets an adopter retire a hand-maintained agent-
  context face with the shipped `constitution_markdown` primitive, so the generator is ergonomic
  polish, not the enabler. Adopter-surfaced by worklane; see `BACKLOG.md`, 校讎/潛移.
- **Name resolution is a 渾儀-internal shared layer (`hunyi::resolve`), not 璇璣's and not
  cross-dimension.** Resolution maps a path as written to a canonical item — it is
  *observation*, so it belongs to a dimension, never to 璇璣 (the `serde_json`-only,
  dimension-agnostic reaction model that renders no verdict — the measure, not an observing
  dimension). It is also not a
  cross-dimension facility: 圭表 resolves with a `syn`-free token scanner (to keep the
  dependency-light core) while 渾儀 resolves with the AST, so a shared resolver would force
  `syn` into the core — the very law that quarantines it. The two resolvers are therefore
  intentionally separate; within 渾儀 the resolver is shared by every semantic capability
  (signature-coupling and trait-impl-locality), resolving via in-scope `use`s,
  `crate`/`self`/`super`, an opt-in bare-name-against-current-module fallback (impl-locality
  needs it; exposure declines it), and local `pub use` re-export chains. This sharing closed
  a re-export false negative in signature-coupling (the contract's one forbidden bug) when
  the second capability was built. Findings are rendered by hand-rolled path/type
  stringification, never `quote`/`syn`'s `printing` feature, so the syn quarantine is untouched.
  **What is intentionally separate is the resolution *engine*, not reading the workspace.** The
  `serde_json`-only cargo-metadata reads (`cargo_metadata` / `find_package` / `crate_root_file`)
  carry no syn-forcing constraint — they are dimension-agnostic manifest observation — so, unlike
  the resolver, they are *not* reimplemented per dimension: they are shared through 星表 (`xingbiao`),
  the substrate below the 三儀 (see the 星表 extraction decision below). The two syn-vs-token-scan
  resolvers stay separate; the metadata reader is one.
- **漏刻 (runtime) is identity-coherent; its design gate is resolved on identity, pending
  only an overhead spike.** A design-gate exploration found that the runtime dimension does
  *not* require amending the project's principles — two apparent amendments dissolve:
  (1) **Instrumentation is entailed, not a departure.** The runtime dimension ships into the
  user's binary (the very reason for the modou→tianheng rebirth into a crate family), and the
  concrete type behind a `dyn Trait` is *physically* unobservable from outside — so runtime
  governance needs in-binary probes + `TypeId` origin opt-in. The "source is SSOT, observe
  don't instrument" stance is properly read as *don't instrument **gratuitously***: the
  semantic dimension's rejection of `#[sealed]` was right because outside-observation was
  available there; for runtime it is not, so minimal instrumentation is the consistent
  application, not a reversal.
  (2) **璇璣 is the shared *measure* (`Violation`/`Severity`), and `Outcome`/exit-code is one
  *projection* of it — not the measure itself.** The CI dimensions (圭表, 渾儀) project the
  measure as a process exit code; 漏刻 projects the *same* `Violation` as a structured runtime
  event (`Violation::to_json` — an audit-log record *is* JSON, so 漏刻's dependence on the
  `serde_json`-bearing 璇璣 is justified on the **cold** path, while the **hot** path —
  `TypeId`→origin lookup — stays dependency-light). So 璇璣 remains "the pivot every dimension
  aligns to"; the name's narrative holds, and 漏刻 (clepsydra — the flow/time instrument)
  completes the 三儀 rather than straining it.
  Consequently 漏刻 has **two faces**: a *prod face* (probes emit `Violation` events; default
  emit/audit, `panic` opt-in — never crash prod on a false positive) and a *CI face* (a static
  scan that every declared `RuntimeBoundary` has at least one probe, projected as an exit code
  through the 天衡 shell — closing the "declared but never enforced" coverage gap). The
  **runtime admission test** mirrors the semantic one, with a sharpened criterion (2): a
  runtime capability is admissible iff (1) declarative-not-lint (an origin allowlist is
  intent), (2) **fail-closed — an unknown/unregistered origin reacts, never silently passes**
  (else it is the rejected false-negative engine), (3) anchorable to a declared, named seam
  (an undeclared seam name is a constitution error). First capability when built:
  **origin-assertion** (a declared seam's `only_origins` allowlist). Still **rejected**:
  runtime capability/effect drift ("no I/O reachable") — a runtime policy engine, an explicit
  non-goal. What remains before a propose is **engineering, not identity**: a hot-path
  **overhead spike** (`TypeId`/registry/`dyn` downcast cost). `louke` is still born only when
  built — no stub, no names before their reaction. **(Built in v0.1.0** as origin-assertion,
  both faces — prod probe + the `audit_probe_coverage` CI scan that closes the otherwise-
  *essential* unprobed-seam gap.**)** Two honesties this dimension forces, recorded here:
  (a) runtime observation is **minimally prescriptive** — the user instruments trait/type/seam
  surfaces (`: louke::Tracked`, `register_origin!`, `assert_boundary!`), which the CI dimensions
  never do; "not a framework you build inside" holds for 圭表/渾儀 but relaxes to its irreducible
  minimum for 漏刻 (the cost of observing what is invisible from outside), and origin is **observed**
  (`register_origin!` captures `module_path!()`), not a self-asserted label. (b) The bright line
  against the rejected runtime-policy-engine: louke's registries hold **static label allowlists
  only, never predicates** — no dynamic conditions, effects, or I/O-reachability.
- **The shell takes the 渾儀 dimension as one `SemanticBoundaries` unit, not one parameter per
  capability.** As the semantic dimension grew (signature-coupling, trait-impl-locality,
  visibility), the shell `run`/`dispatch` accumulated one `&[…]` slice per capability — by the
  fourth, an unwieldy positional list where empty slices are easy to misorder. The semantic
  boundaries are gathered into a `hunyi::SemanticBoundaries` container (one field per
  capability) held by the unified `Constitution`, and the shell composes them via
  `hunyi::check_all(constitution.semantic_boundaries(), …)` with a single `cargo metadata`
  read. `run(&Constitution, args)` is then stable as further semantic capabilities are added
  (they extend the container behind a new typed adder, not the `run` signature). This is a
  **behavior-preserving facade refactor** — the same dimensions compose into the same reaction,
  no capability's requirements change — so, like the 璇璣 extraction, it is **not** an OpenSpec
  capability change; it is recorded here and kept honest by the existing tests and
  `self_governance` (the container lives in 渾儀, owns only its own sub-capabilities, and does
  not couple across the 三儀). The per-capability `check_*` entries remain for direct use.
- **漏刻's CI face is behind the non-default `audit` feature, not a 5th crate.** The runtime
  dimension ships into production and must stay light, but its CI face (`audit_probe_coverage` +
  the hand-rolled source scanner) is build-time-only code. Rather than a separate `louke-ci` crate
  — which would be a crate *born without a new reaction* (against the drift law) and would couple a
  detached scanner to `louke`'s own `assert_boundary!` spelling — the CI face is gated behind a
  non-default `audit` Cargo feature. A production binary depending on `louke` for the hot path then
  compiles **zero** scanner code (guaranteed by `cfg`, not left to dead-code elimination), while the
  天衡 shell enables `louke/audit` to run the audit inside `check`. Two honesties: (a) the gated
  scanner's tests compile only with the feature on, so CI runs `test`/`clippy`/`doc` with
  `--all-features` (a plain `--workspace` would silently `cfg` them out — strictly worse than no
  gate, since they guard the *one forbidden bug*), and keeps a default-feature `build`+`clippy` to
  guard the prod-light config has no unused items; (b) `tianheng` is a CI/dev-only tool —
  co-depending on it from a production binary unifies the `audit` feature back ON, voiding the
  zero-scanner-in-prod property. This is a packaging/architecture refactor (no capability's
  requirements change beyond the CI face being feature-scoped) — like the 璇璣 extraction it is
  recorded here, not an OpenSpec change, and kept honest by the tests and `self_governance`. Two
  scanner false negatives were fixed in the same pass: nested block comments (Rust comments nest;
  a probe inside one is commented out and must not count) and non-`()` macro delimiters
  (`{}`/`[]`); both are now reacted to, with the lexical-scan `cfg`-blindness recorded as a stated
  bound in `audit_probe_coverage`'s doc and the spec.
- **(v0.1.4) A semantic violation names its governed module's source file — a 垂象 fidelity
  closure powered by 渾儀's existing traversal.** A semantic violation reported `file: null` (a
  spec-stated bound), yet the reaction already descends to the governed module's file to observe
  its items — the file is a **faithful byproduct**, the same admission the static module-import
  `file` meets. The gap was only that the file lived inside the observation heart and never
  reached the reaction layer. Closed by refactoring the one module traversal (`descend`) to return
  `(items, file)`, keeping `resolve_module_items`' signature (so the pure `*_findings` hearts —
  the *what* — are untouched, no test churn) and adding `resolve_module_file` (the *where*), which
  the reaction layer (`check_*_boundary`) attaches via `Violation::with_file`, resolved once per
  boundary and only when reporting. The `finding` still names the canonicalized forbidden
  type/shape (which may be *defined* elsewhere); the `file` names the **seam's** location, the
  actionable one. `file` stays out of the `(target, rule, finding_key)` baseline identity, so a
  previously-null violation baselined and then populated never re-baselines nor changes the count;
  SARIF gains a `physicalLocation` (still no `region` — a file, not a line). **Scope is the five
  single-module capabilities** whose anchor resolves to one module (signature-coupling exposure
  incl. its re-export/trait-impl depths, dyn-trait, impl-trait, async-exposure, visibility) — each
  knows its file at the reaction layer from `boundary.module` alone. The **two whole-crate/subtree
  scans stay `file: null`**: trait-impl-locality and forbidden-marker name sites (a trait `impl`, a
  `#[derive]`) scattered across the crate, not one governed-module file; surfacing their per-site
  file needs the heart to carry the per-finding `site.module` (then the same `resolve_module_file`)
  — a heart signature change deferred as a born-when-built follow-up, with the `cli-check-runner`
  null bound **narrowed** to name these two explicitly rather than left blanket (never a silent
  claim). An adversarial review at propose caught the mis-scoping of forbidden-marker (it *looks*
  module-anchored — its `boundary.module` is a subtree prefix — but is mechanically a whole-crate
  scan). Additive / API-compatible (a previously-`null` field is now sometimes populated), so
  **patch** (0.1.4) per SemVer honesty. Ships as an OpenSpec change modifying `cli-check-runner`.
- **(v0.1.4) …and then 7/7: the two whole-crate scans name their file too.** The follow-up above
  was closed immediately: trait-impl-locality and forbidden-marker now carry a `file` as well, so
  **every semantic violation names its source file** — the single-module ones by their governed
  module, the whole-crate-scan ones by the *offending element's* module (the `impl` site's module;
  the defining type's module for a `#[derive]`, carried on `TypeDef`). The two hearts surface a
  **per-finding module** (already embedded in every finding) and the reaction layer resolves it
  with the same `resolve_module_file`, memoized per module. Two adversarial-review points shaped
  it: dedup **by finding** (not the `(finding, module)` pair) so `file` never changes the violation
  count (the count invariant), and resolve with **`.ok()`** (degrade to `null`) so a resolution
  failure — the module comes from the whole-crate scan, the file from the single-path resolver —
  never turns a real violation into an exit-2 error or drops it. Additive/API-compatible (patch);
  stacked on the change above (its `cli-check-runner` delta is the final 7/7 form, retiring the
  narrowed null carve-out). Ships as an OpenSpec change modifying `cli-check-runner`.
- **(v0.1.4) A resolvable type alias in a public seam reacts — narrowing the over-broad "alias =
  inference" bound.** `semantic-signature-coupling` parked *all* alias chains under "full type
  inference" as out of scope, but a `type H = crate::infra::Db;` names its target **literally** —
  resolving `H` needs only substitution. So the representative shape was a genuine **false
  negative**: a *private* alias used in a public seam (`pub fn f() -> H`) silently passed, as did a
  cross-module alias reached via `use` (a **public** same-module alias already reacted — its target
  is a walked exposed position). Closed by mirroring the existing re-export closure: the crate-wide
  scan collects an **alias map** (`{module}::X → canonical target`, target resolved through the
  defining module's `use`-scope / `crate`-relative / extern oracle with the same per-module child
  shadow as type positions), and the signature-coupling exposure pipeline gains a bare-local-alias
  fallback plus a **combined alias+re-export fixpoint** (`canonicalize_through_aliases`) so an
  alias→alias / alias→re-export chain resolves to the defining path. **Scoped to the exposure
  pipeline only** (the extern-oracle precedent) — `resolve_path`'s other callers (dyn/impl-trait/
  async operands, visibility, anchor) are untouched (their operands are traits; a `dyn`/`impl` of a
  *type* alias is not stable Rust); the trait-impl-exposure depth shares `module_findings` and so
  inherits the closure, exactly as its spec's "same resolver" deferral intends. **Bounds kept**
  (never a silent claim): complex-target aliases (`type H = Vec<Db>`, `&Db`, tuple/`dyn`/`impl` —
  the *directly*-written form still reacts), generic aliases (`type H<T> = …`), and genuine
  inference. Expansion can only **add** findings (close FNs), never remove one or change an existing
  finding's canonical — the finding names the resolved target (`crate::infra::Db`), never the alias
  spelling (`H`), so identity is spelling-independent and no baseline churns. Adversarial review at
  propose caught the **alias-before-extern** ordering (a local `type serde = …` shadows a same-named
  dependency, per Rust's own resolution — `extern_verbatim` is meaningful only for a multi-segment
  `dep::Foo`, which a type alias cannot prefix). Additive/API-compatible (a previously silently-
  passing exposure now reacts), so **patch** (0.1.4) per SemVer honesty. Ships as an OpenSpec change
  modifying `semantic-signature-coupling`.
- **(v0.1.4) `extern crate` exposures react — two local-observable residuals of external-crate
  exposure, closed.** `semantic-reexport-exposure` left two stated bounds that were genuine false
  negatives of an *existing* capability (not speculative new ones), so the "I don't use it" /
  frequency argument did not excuse them — a downstream adopter governs its own (possibly
  edition-2015 or crate-renaming) code. **FN-A**: a source-level `extern crate worklane_core as wc;`
  rename made `wc::spi::Foo` silently pass, because `wc` is absent from `cargo metadata`; but the
  `extern crate … as …` item is in the **local AST**, so a **crate-root** rename is now collected
  into an `ExternRenameMap` (`Y → X`) and applied by `extern_verbatim_renamed` — the head is mapped
  to the real crate before the extern check, in the exposure pipeline (type position + the governed
  module's own `pub use`). **FN-B**: `pub extern crate worklane_core;` republishes the crate root
  like `pub use ::worklane_core;`, but `collect_item_exposures` had no `Item::ExternCrate` arm, so
  an exposure boundary missed it (the visibility dimension caught it only if separately declared —
  an exposure boundary must not depend on another dimension). Now an `ExternCrate` exposure arm
  names the **real** crate (`item.ident`, not the `as`-rename). **Bounds kept**: a **module-scoped**
  rename (binds locally — collecting it crate-wide would false-positive, so crate-root only), a
  rename reached through a **type alias** or a **multi-hop facade closure** (the map is applied in
  the exposure pipeline, not threaded into alias-target resolution or the re-export closure), a
  distinct **`[lib] name`**, and the edition-2015 relative re-export. The finding names the real
  crate, never the source alias, so identity is spelling-independent and no baseline churns; the
  existing `..._is_a_stated_bound` test flips to a reacting test. Additive/API-compatible, so
  **patch** (0.1.4). This decision also records a corrected judgment: when ordering the work I called
  this "cheaper than the alias closure" — recon showed the opposite (two capabilities, a rename map
  threaded through two positions), yet the *false-negative-first* law (not frequency) is what
  governs an existing capability, so it was still right to close. Ships as an OpenSpec change
  modifying `semantic-reexport-exposure`.
- **(v0.1.4) Resolver collection↔query parity — a comprehensive pre-release adversarial review
  caught a genuine false negative and closed the whole alias × extern-rename × re-export family.**
  The alias-target *collection* ladder (`walk_module`) resolved with a weaker ladder than the
  query-time exposure pipeline (`module_findings`), so a forbidden type reachable only by the
  stronger steps was silently dropped from the alias map — never recorded, never followed. **FN1**
  (the blocker): a **bare** alias-of-an-alias (`type Inner = crate::infra::Db; type Public = Inner;`)
  silently passed, a real false negative against signature-coupling's own "alias→alias chain
  resolves / no false negative in the resolved scope" claim (the existing test used the *qualified*
  intermediate, so the bare form — ordinary Rust — was untested). **FN2** (alias target through a
  crate-root `extern crate … as` rename) and its **facade-closure sibling**, plus **FN3** (the
  per-module child-module shadow suppressing a *renamed* head), were stated/contrived bounds closed
  in the same pass. Fix: pre-collect crate-root renames before the walk (source-order independence);
  a renamed head resolves to its real crate verbatim (a rename alias is never a local child module);
  the collection ladder gains `extern_verbatim_renamed` + a **name-gated** bare-local-alias fallback
  (records a bare target only when it names one of the module's own aliases, so the query fixpoint
  closes the chain order-independently); renames threaded into the re-export closure. The
  **apply-stage adversarial review caught a false positive in the first-cut fix** — a blanket
  `CurrentModule` fallback mis-recorded a bare std-prelude target (`type H = String`) as
  `crate::domain::String`, which false-positives under a self-forbidding boundary; name-gating
  removed it (permanent regression guard). Bounds kept (genuinely non-local): foreign-module
  routing, glob leaves, distinct `[lib] name`, macro/inference/complex/generic alias, module-scoped
  rename. `crates/hunyi` only; additive false-negative closure, so **patch** (0.1.4). Ships as an
  OpenSpec change (no spec-requirement change beyond narrowing the reexport residual bound;
  signature-coupling was already correct — FN1 was a code bug against its claim).
- **(v0.1.4) 璇璣 understands, never reacts — the charter line is *no verdict*, not *no
  observation engine*.** The charter said 璇璣 "holds no observation engine"; the load-bearing
  invariant is narrower and truer. 璇璣 may hold the **measure** and **judgment-neutral
  mechanism**, but never a **verdict** — the *react* itself: comparing a **declared** boundary
  against **observed** reality to emit a `Violation`. The measure is the react's **output**
  (`Violation`/`Report`/`Baseline`/`Outcome`); the declaration is its **input**
  (`Boundary`/`Rule`/`Constitution`, which stay in the dimensions and the shell). 璇璣 carries the
  output, never the input — and that asymmetry is **already enforced by the existing
  `restrict_dependencies_to(["serde_json"])` boundary on 璇璣** in `self_governance.rs`: the
  declaration types live *above* 璇璣, and 璇璣 depends on no workspace member, so it *structurally
  cannot name a declared boundary to compare against*. No **new** reaction is needed; the
  typed-verdict ban falls out of the dependency-direction law already declared. The charter word
  "observation engine" over-restricted — it banned *mechanism*, when the real invariant bans only
  *verdict*, and the verdict is already foreclosed.
  Consequence: a **judgment-neutral parsing primitive** (a std lexer / token scanner) is admissible
  in 璇璣 — it would be, to the syn-free dimensions (圭表, 漏刻), exactly what `syn` is to 渾儀: a
  mechanism that *understands Rust source* and produces structure but renders no architectural
  verdict. It is dimension-agnostic (it knows neither the static `use`/`mod` nor 漏刻's
  `assert_boundary!` — each dimension matches its own pattern over the shared token stream), so it
  belongs below the dimensions, in 璇璣's slot; admitting it does not let 璇璣 react. **Born when
  built, not now:** two hand-rolled scanners (圭表's `module_scan`, 漏刻's audit `scan_source`) work
  and are green, and the drift law forbids laying shared infrastructure ahead of a forcing function.
  This entry declares the **direction** (璇璣 = judgment-neutral base), not the build: 圭表/漏刻 keep
  their local scanners — written *toward* this shape — until a third forcing event (a third syn-free
  scanner, or a cross-scanner false negative) earns the extraction. When it lands, *where the two
  scanners already agree* it is behavior-preserving (identical scan results) → **patch** by SemVer
  honesty; any lexical divergence the unification surfaces (one scanner handling a case the other
  missed, e.g. 漏刻's nested-comment/non-`()`-delimiter fixes that 圭表 never took) is a separate
  false-negative closure, still **patch** by the v0.1.4 precedent but never a silent behavior change;
  漏刻's production weight
  is held the same way its CI face already is — the primitive sits behind a `cfg` feature that a
  standalone 漏刻 prod build leaves off (linking zero scanner code), subject to the same
  feature-unification honesty recorded for the `audit` feature (co-depending on `tianheng`, or on
  圭表 which needs the lexer for its default scan, unifies the feature back on). **Stated bound**
  (never a silent claim): the dependency law forecloses the *typed* react (璇璣 cannot name a
  `Boundary`), not a contrived *stringly-typed* comparison over primitives — that residual stays a
  charter / human-review invariant, legitimately prose per the "Declaration integrity" line above
  (verdict-vs-mechanism is a *judgment*, not a structural property, so it is not migrated to a
  reaction). Architecture/charter, not a capability change — like the 璇璣 extraction and the
  `SemanticBoundaries` facade, recorded here and kept honest by `self_governance` + the tests,
  **not** an OpenSpec change.
- **(v0.1.4 → 0.2.0 line) Structure semantic observation facts — containment first, then typed
  identity when the structured baseline supplied the reaction.** A refinement pass structured 渾儀's internals
  *where a live pain existed and stopped where only prep remained*. **Shipped:** the sibling-safe
  `::`-path containment rule — hand-copied in `under_subtree` / `matches_forbidden` / `matches_allowed`
  (`x == entry || x.starts_with("{entry}::")`, the `::` that keeps `crate::commands` from matching the
  sibling `crate::commandeer`) — converges into one `path_within(path, prefix)`. This is the
  born-when-built case: the pain is real (one false-positive/false-negative-critical rule in three
  drift-prone copies), the reaction surface already exists, and the abstraction is a byte-identical
  convergence of *existing* observation. (Alongside two behavior-preserving tidies not worth their own
  decisions — the `SemanticFinding` catalog centralizing the finding-string formats, and the split of
  the ~8k-line `lib.rs` into `lib` / `dsl` / `tests` — recorded in git, not here.) **The forcing event
  arrived on the 0.2.0 line:** version-2 structured baseline identity made the remaining string seam
  a live contract defect, because 渾儀 was still keying the complete rendered descriptor while 圭表
  and 漏刻 keyed named observed values. The OpenSpec change `structure-semantic-fact-identity`
  resolves the earlier layering question with a private `PublicSeam` owned by the finding vocabulary
  and carried by `resolve.rs`'s `ShapeExposure`; the lower resolver stores the typed observation but
  does not own its presentation. One `SemanticFact` enum now derives both byte-identical text and
  fact-specific named key fields. The canonical type path / dyn or impl shape remains the observed
  `subject` string — no speculative recursive `ExposureSubject` AST. This is deliberately breaking
  for unreleased descriptor-shaped version-2 semantic keys, while published version-1 baseline text
  migration remains intact. The earlier deferral therefore did its job: no structure before a
  reaction, then the narrowest data model once baseline identity required it.
- **(v0.1.4) Module-source / module-resolution hardening — observe the compiled source root, and
  keep `#[path]` remaps out of scope instead of governing the wrong file.** An adversarial review
  found edge cases against the scanner/resolver decisions above. First, 圭表 module boundaries used
  the old `manifest_dir/src` shortcut even though Cargo already observes the real lib/bin
  `src_path`; a custom `[lib] path = "lib.rs"` could therefore make a real module boundary scan the
  wrong root. Second, a `#[path = "..."] mod foo;` declaration was still admitted into 圭表's
  reachable module graph; if a same-named conventional `foo.rs` orphan also existed, 圭表 could
  govern that uncompiled orphan while the real remapped file stayed outside observation. The same
  wrong-file shape existed in 渾儀's single-module resolver (`resolve_module`), which could resolve
  a semantic boundary to the conventional orphan. The fix reuses Cargo's target `src_path` for
  static module boundaries and skips direct `#[path]` module declarations in both the token
  reachability walk and the semantic single-module descent. It does **not** add `#[path]` support;
  it preserves the bound honestly. `crates/guibiao` + `crates/hunyi`; false-negative closure, so
  **patch** (0.1.4), no new capability.
- **(v0.1.4) The inline twin of the `#[path]` orphan-shadow — an inline-only module's same-named
  conventional file is an orphan, not its backing file.** A comprehensive pre-release adversarial
  review found the inline sibling of the hardening above, left open when the `#[path]` half was
  closed. 圭表's static target is file-based (an inline `mod name { … }` owns no file, so it is a
  self-describing exit-2 constitution error, a deliberate non-goal), and that error fires when
  `governed_files(target)` is empty — but `governed_files` selected files by their path-derived
  identity, so a same-named conventional orphan (`name.rs` / `name/mod.rs`) beside the inline body
  made the set non-empty, bypassing the guard: 圭表 scanned the orphan (a file rustc never compiles
  as that module) and missed the inline body's imports — a **silent pass, the one forbidden bug**.
  The root cause is that 圭表 infers module identity from **filenames** and must correct for each
  declaration form that decouples a file from its module (`#[path]`, now inline; `#[cfg]`-duplicate
  is the third, the stated cfg-blind bound); 渾儀 is immune because its AST descent is
  **declaration-driven** (follows `mod` content), consulting no orphan — the reason the static and
  semantic dimensions legitimately differ on inline targets (圭表 exit-2, 渾儀 governs), a difference
  declared per-dimension in the specs, not narrated. The fix classifies each `mod` declaration
  inline vs file and excludes, at the file-list source, a conventional file whose path is declared
  **inline-only** (inline AND not also file-form) — so the reachability walk does not read it (no
  phantom child modules) and `governed_files` does not scan it, restoring the intended exit-2. An
  **apply-stage** adversarial review sharpened the condition to inline-**only**: a naïve "any
  inline-declared path" would also fire on a `#[cfg]`-gated dual declaration, which a cfg-blind
  scanner cannot distinguish — silently changing a case the fix must not touch; gating on inline-only
  (a dual declaration arises only under `#[cfg]` or already-invalid code) leaves the cfg-blind bound
  exactly as it was. `crates/guibiao` only; false-negative closure, **patch** (0.1.4), no new
  capability. Ships as an OpenSpec change modifying `module-boundary`.
- **(v0.1.4) The published crate must self-test — a packaged-crate CI reaction, and the fixture
  tests skip when packaged.** `cargo publish` packages only files inside a crate's own directory, so
  a data file *outside* it — a test fixture (`crates/tianheng/tests/fixtures/*`, each its own
  `[workspace]`, which `cargo package` excludes as a nested package even under an explicit
  `include`) or the workspace root — is absent from the tarball. The in-repo `cargo test` never
  notices (those files exist in the checkout); the failure is **package-only** (three fixture-driven
  `tianheng` tests failed only from the published crate). Two moves, same family as the
  license-bundling reaction (release hygiene, a CI reaction — never a Tianheng boundary): (a) the
  fixture-driven dispatch tests reuse the existing `workspace_manifest` sentinel — present in the
  repo (they run as a real end-to-end gate), absent in a packaged `.crate` (they **skip**, never
  fail) — the same repo-vs-packaged discipline `self_governance` already uses, with
  `TIANHENG_WORKSPACE_TESTS=1` turning a *missing* repo layout into a loud failure so CI never
  silently skips; (b) a `packaged-selftest` CI job packages every publishable crate, extracts the
  tarball, patches its workspace-sibling deps back to local source (they are not on crates.io at the
  in-development version), and runs its tests **from the tarball** — proving the skip is real and
  catching any future crate whose packaged tests reference an unpackaged file. Docs/tests/CI only;
  no capability or reaction-behavior change.
- **(v0.1.5) A durable governance anchor on boundaries and their violations.** `because(...)` stays
  the human repair hint; `.with_anchor("ADR-014")` gives tools and agents a stable governance
  coordinate. The anchor is metadata, not a reaction input or baseline key, so this is additive and
  keeps anchor-less projections byte-identical.
- **(v0.1.5) A repair-direction polarity on violations.** `Polarity` is derived from the rule type:
  deny rules point toward removing the offending code, allowlist rules toward removing or declaring
  intent. Runtime CI-audit consistency findings stay off this axis (`None`); this is violation
  metadata, not constitution data.
- **(v0.1.5) `owner`/`tracker` metadata on baseline entries.** Baselines can point accepted debt at
  people or trackers without changing the match identity (the legacy text triple in v1; structured
  `(target, rule, finding_key)` in v2). This is the
  additive 實錄 step, deliberately not the future 0.2.0 structured-baseline break.
- **(v0.1.5) 漏刻 decodes escaped seam literals in the CI probe-coverage face.** The CI audit now
  compares probe literals to the compiler-decoded declaration value, so escaped seam names cannot be
  falsely treated as covered or uncovered. Unreproducible escapes stay loud as un-auditable probes.
- **(v0.1.5) 圭表 skips precise-capturing `use<...>` bounds in the import scanner.** A Rust type
  bound spelled `use<...>` is not an import and must not consume the next real `use` declaration.
  This closes a scanner false negative without changing the observed source class.
- **(v0.1.5) 渾儀 signature-coupling walks trait-bound generic arguments recursively.** Supertraits,
  associated-type bounds, and GAT/default positions now use the same nested-path collection as the
  rest of the public surface, closing a same-source false negative.
- **(v0.1.5) 漏刻 skips foreign macro bodies in probe-coverage scanning.** A probe inside an
  unexpanded macro body no longer counts as runtime coverage. The scanner remains louke-local to
  preserve 三儀 independence.
- **(v0.1.5) 渾儀 re-export head resolution honors child-module shadowing.** A bare `pub use dep::X`
  is not attributed to an external crate when the re-exporting module's own child `mod dep` shadows
  that head; leading-`::` remains the explicit extern escape hatch.
- **(v0.1.5) 渾儀 crate-root extern renames resolve rustc-correctly.** `crate::<alias>::...` rewrites
  to the real crate, while bare alias heads are suppressed only under a same-module child-module
  shadow. This closes the paired extern-rename FN/FP without broadening the observation source.
- **(v0.1.5) 圭表 gains `must_only_be_imported_by`.** The inbound closed allowlist is the dual of
  `restrict_imports_to`: it expresses thin-facade ownership that forbid-one rules cannot. It reuses
  the existing crate-wide `use` scan and is an additive module-rule variant.
- **(v0.1.5) `projection_gate` makes 潛移 projection freshness reusable.** Adopters can byte-check a
  checked-in Markdown law projection against the live constitution with a pure helper; full
  generator / `list-self` product work stays deferred.
- **(v0.1.5) 渾儀 observes public inherent associated `const`/`type` items.** Inherent impls already
  exposed public method signatures; their public associated items are the same public surface and now
  react when they leak forbidden types.
- **(v0.1.5) 渾儀 re-export closure applies child-module shadowing per defining module.** Facade
  chains now inherit the same rustc-correct head-shadow rule as direct re-exports, including the
  leading-`::` escape hatch and crate-root rename aliases.
- **(v0.1.6) 星表 (`xingbiao`) — the shared declared-workspace-data substrate.** The
  `serde_json`-only cargo-metadata reads (`cargo_metadata` / `find_package` / `crate_root_file`)
  were written twice — once in 圭表, once in 渾儀 — and drifted: 圭表's `crate_root_file` never
  learned the `proc-macro` arm its 渾儀 twin gained, a live cross-dimension false negative (a
  proc-macro crate silently dropped from 圭表's module-boundary resolution and, via
  `member_src_dirs`, from 漏刻's CI-audit corpus — which `runtime-origin-assertion` already
  requires be "the same source root the semantic dimension uses"). This is the **twin-drift bug
  class** the 0.1.5 review rounds surfaced repeatedly. Following the **`xuanji` precedent** (an
  internal refactor, not a spec'd capability), the neutral substrate extracts into a new crate
  below the 三儀, sibling to 璇璣: `serde_json`-only, spawns `cargo`, observes but renders no
  verdict (so it is **not** 璇璣, the measure-only model). Each dependent dimension names it in its
  `restrict_dependencies_to` allowlist (圭表, 渾儀; 漏刻 reads no metadata, so its allowlist is
  unchanged), a one-way downward edge that **is not** a cross-dimension dependency — 三儀 ⊥ 三儀
  forbids only dimension-to-dimension. A single reader makes the metadata twin-drift class
  structurally impossible; unifying `crate_root_file` on the `proc-macro`-aware body is a
  false-negative closure that brings 圭表 into conformance with the existing runtime-audit spec, so
  it is **patch** (0.1.6) per SemVer honesty. What stays per-dimension: 圭表's dependency
  source/kind semantics (`classify_source`, `dependencies*`) — its own observation, not neutral
  infrastructure — and the syn-vs-token-scan **resolvers** (sharing them would force `syn` into the
  light core; see the Name-resolution decision above). The remaining `xuanji`-slot judgment-neutral
  *scanner* extraction stays deferred, awaiting its own forcing function.
- **The crate family carries product identities; productization is demand-driven.** The six
  published crates are not merely a workspace split. The **三儀 are public products** — 圭表 (static
  import / dependency boundaries, syn-free), 渾儀 (public-API exposure), 漏刻 (runtime origin) — three
  *orthogonal* instruments with distinct observation sources and audiences, not redundant crates.
  **璇璣 / 星表 are the public substrate** they stand on (public because the instruments depend on
  them, not products in their own right). **天衡 is the composer** and the funnel target: adopt one
  儀 as an on-ramp, graduate to the composed constitution. Which face becomes a long-term contract is
  decided by *reaction*, not ambition — the drift law applied to go-to-market: **no name without a
  reaction, so no commitment without a reaction.** The **product identity** is declared now (a
  reversible narrative); the **product weight** — per-儀 standalone CLIs, cookbooks, per-crate 1.0 /
  stability promises, and the standalone 漏刻 story (a legitimate category, but the least-proven as a
  product) — waits for an adoption signal. The historical posture was **0.1.x late-stage
  pre-stability**: concept and function were saturated (三儀 all born, a complete world-view), while
  the deliberate hold withheld API lock-in until real use identified which public surfaces to
  freeze. That trigger fired before 0.2.0; the current **0.2.x compatibility line** preserves its
  published API and version-2 identity wire across patches, while a break must earn a new minor. A
  category-creating project cannot pull demand for a category nobody knows exists, so the
  order is **push then pull**: push the honestly-labelled (experimental) narrative to bootstrap
  awareness; let demand deepen it. The exit trigger to the deliberate breaking 0.2.0 was a real
  reaction — composed adopter pacta plus standalone 圭表 adopter modou — never the calendar. See
  BACKLOG "Version horizons" for the current operational split.
- **(v0.1.8) Severity is centrally gated and boundary-inherited — and this coverage invariant is
  prose, not a reaction (a bright-line instance).** Every boundary reaction inherits its declared
  severity, and the gating decision lives in exactly **one** place — `xuanji::Outcome::exit_code()`:
  warn-only or fully-baselined → exit 0, any non-baselined `Enforce` → exit 1. No dimension
  re-implements the exit rule; each only *produces* `Outcome::Violations(report)` and the shell folds
  them, so `Severity`'s meaning cannot drift per-dimension. Propagation is structural, not
  per-capability: the 圭表 module reaction stamps `boundary.severity` at a **single** `Violation::new`
  site, so all six `ModuleRule` variants inherit it (`must_not_call_inline` included), and 圭表/渾儀
  both dedup violations by `id()` keeping the **more severe** on a collision (`Enforce` dominates
  `Warn` — spelled out because `Severity` is `#[non_exhaustive]` with no `Ord`), the guard that stops
  a `warn` duplicate from masking an `enforce` one, which would silently drop exit-1 to exit-0, the
  one forbidden bug. **漏刻's CI face carries the one deliberate asymmetry, recorded so it does not
  read as an oversight:** only the *declared-but-unprobed* coverage reaction inherits
  `boundary.severity()` (a `warn` runtime boundary is advisory, not a CI failure); the three
  **wiring-integrity** reactions — a seam declared twice, a probe naming an undeclared seam, an
  un-auditable non-literal probe — are always `Enforce` **by design**, because they are
  misconfigurations (always wrong, not gradable architectural drift you would warn-adopt), mirroring
  the prod face's fail-loud `install`. This whole invariant **stays prose and is not migrated to a
  reaction** — itself an application of the "Declaration integrity" bright line above: "gating lives
  in one place" is a code-shape property no boundary observes (reacting to it would name a capability
  with no proven drift, against the drift law), and "why 漏刻's three are `Enforce`" is *rationale
  explaining a human choice*, which the bright line says never migrates. The real backstop against a
  future dimension re-implementing the exit rule is **潛移** (the idiom — every dimension returns
  `Outcome`, only the shell projects it through `exit_code()`), not a manufactured gate. Architecture
  recorded here, kept honest by the tests; **not** an OpenSpec change.
- **(v0.1.8) `sans_io_pure` — the first shell-composed profile, gated honest.** A pure kernel's two
  *source-observable* axes — it reads no ambient clock, and its public API exposes no `async fn` —
  were already two shipped reactions (圭表 `must_not_call_inline` + 渾儀 `must_not_expose_async_fn`).
  `Constitution::sans_io_pure(SansIoPure::in_crate(c).module(m).reading_clock_via(prefix, verbs).because(r))`
  folds them into one declaration. It lives in the **shell** (三儀 ⊥ 三儀: a dimension never imports
  its sibling; only 天衡 composes) and adds **no new requirement** — it emits two already-spec'd
  reactions verbatim — so, like the `SemanticBoundaries` facade and the 璇璣 extraction, it is a
  Decision kept honest by tests, **not** an OpenSpec change. Three calls, each surfaced or hardened by
  adversarial review: (1) **Honest scope, nothing baked.** The name is the recognized domain term, but
  the profile governs the *clock* and *async* axes only — `fs`/`net`/`env` stay the adopter's to add
  explicitly. The clock marker set (prefix + read verbs) is **not** baked: the adopter supplies it via
  `reading_clock_via`, so a second sans-I/O consumer's needs cannot silently diverge from a frozen
  default. A *batteries-included* variant (a default marker set) is deliberately **deferred to a second
  consumer** — the drift-law promotion gate applied to a convenience: the composite *pairing* is proven
  (the surfacing adopter hand-rolls both halves), but the default *content* is what a second domain
  reveals; the over-claim risk is bounded because the name appears in no projection (only the adopter's
  source) — the two emitted boundaries and the reason do. (2) **Fluent, not flat — a false-negative-safe
  surface.** A flat `(crate, module, prefix, …, reason)` method would put four swappable `&str`
  adjacent; a caller transposing the confinement prefix and the reason compiles, resolves nothing, and
  never reacts — the one forbidden bug, and precisely what the whole fluent DSL exists to foreclose. So
  the profile is a builder where each argument is anchored by its own method; severity is the
  boundaries' own `.warn()` (one idiom, not a second), applied to both. (3) **Faithful composition is
  the test, not the reaction.** The two atoms already carry reacting tests in their crates, so
  `sans_io_pure` is tested by *projection-equality* against the hand-composed pair — every
  reaction-affecting field (prefix, verbs, severity, reason) is in the projection, so equal projection
  ⇒ equal boundaries — with a non-canonical-marker case guarding the no-bake rule. `crates/tianheng`
  only; additive / API-compatible → **patch** (0.1.8).
- **(v0.1.8) `max_visibility` — a visibility *ceiling* depth, resolved false-negative-safe on the
  `pub(in P)` corner.** 渾儀's binary `must_not_declare_pub` generalizes to a parameterized ceiling
  `max_visibility(Crate|Super|Module)`: a direct item reacts iff its declared-visibility **rank**
  (`pub`=3 > `pub(crate)`=2 > `pub(super)`=1 > private/`pub(self)`=0) is strictly above the ceiling.
  Depth, not width — same `syn` source, same module descent, same observed item set; only the per-item
  predicate and the finding string change. `must_not_declare_pub` becomes the `Crate`-ceiling sugar,
  **byte-stable** in findings, rule string, and baselines (the non-breaking constraint: `item_finding`
  at rank 2 reproduces the old `pub_item_description` exactly). Admission holds: declarative-not-lint
  (Crate/Super/Module are genuine per-project intent), non-compiler-expressible (rustc accepts
  *widening* a `pub(crate)` declaration to `pub`; the ceiling governs the declaration's evolution),
  anchorable (a module). The one non-obvious call, recorded because it is the false-negative-safety
  crux: `pub(in P)` is matched **whole and single-segment** (`crate`/`super`/`self`); **every other
  restricted form** — a multi-segment path such as `pub(in super::super)` (which reaches the
  grandparent's whole subtree, *broader* than `pub(super)`), a leading-`::` path, an unrecognized
  single segment — falls to a `match` catch-all and ranks **Crate (2), a conservative upper bound**. A
  `pub(in P)` path is always an in-crate ancestor, so the item is at most crate-visible; ranking every
  unrecognized restricted form Crate therefore **never under-reacts** (the one forbidden bug is
  foreclosed), and at worst **over-reacts** under a Super/Module ceiling when the real path is narrow —
  a stated bound, loud-over-reaction-over-silent-pass, the same posture as dyn's empty-operand and the
  glob fail-closed. A first-segment match (`super`→Super) would have silently passed `pub(in
  super::super)` under a Super ceiling — the exact defect an apply-stage adversarial review is aimed at,
  guarded here by a rank unit test. The rule string encodes the ceiling (`Crate` keeps the legacy
  string; `Super`/`Module` are distinct, injective across the whole semantic family), and the finding
  carries the item's declared visibility so items differing only in visibility never collapse. Ships as
  an OpenSpec change modifying `semantic-visibility-boundary`; additive → **patch** (0.1.8).
- **(v0.1.8) `UnsafeBoundary` — confinement-only by admission, and the body-nested-`mod` false
  negative the review caught.** A new 渾儀 capability confines `unsafe` (blocks, `unsafe fn`/`impl`/
  `trait`, `unsafe extern`) to a declared subtree: `UnsafeBoundary::in_crate(p).only_under([subtrees])`
  reacts on a site outside every allowed subtree. **The admission-critical decision is that it is
  confinement-only, never a crate-wide on/off.** The pure "crate is `unsafe`-free" case is deliberately
  excluded because `#![forbid(unsafe_code)]` does it *stronger* (compile-time, unbypassable); an empty
  or crate-root `only_under` is a **constitution error** (exit 2) pointing at `#![forbid]`. This is what
  keeps the capability declarative-not-lint under the semantic admission test: it governs *where*
  `unsafe` lives (architectural intent — two crates can confine it differently and both be right),
  never *whether* it may exist (the compiler's opinion-free job). It reuses the whole-crate-scan family
  shape (a `syn::visit` `UnsafeSiteCollector` per module over a dedicated walk that inherits
  `scan_crate`'s symlink-cycle / missing-file / `#[path]` guards; per-module findings via
  `per_finding_file`, `AllowlistGap` polarity, a fixed rule string with the allowed set as projected
  configuration — all mirroring trait-impl-locality). Two false-negative-safety calls, both surfaced by
  adversarial review: (1) the **propose** review caught that making `visit_item_mod` a no-op *and*
  excluding `Item::Mod` would silently drop `unsafe` in a `mod` declared inside a fn body — fixed by
  leaving `visit_item_mod` at its default (recursing) and filtering only **top-level** `mod`s (walk-
  owned), so a body-nested `mod`'s `unsafe` is caught while a top-level inline `mod` is not double-
  counted; (2) the same review demoted an over-broad "no essential gap" claim to **explicit stated
  bounds** — `#[unsafe(...)]` attributes, bare `unsafe fn` pointer *types*, and plain `extern "C" {}`
  blocks (no keyword; their call sites still react) are lexical `unsafe` tokens that are not executable-
  `unsafe` code sites, declared out of scope rather than silently missed. Findings are per-module,
  per-kind (anonymous blocks dedup per module — the confinement unit is "this module has `unsafe`
  outside the subtree", so a fragile per-block ordinal is deliberately avoided; the trait is in an
  `unsafe impl` finding for injectivity). Ships as the OpenSpec change `semantic-unsafe-confinement`;
  additive → **patch** (0.1.8).
- **(v0.1.8) Async-exposure gains an opt-in *subtree* scope — the false negative dogfooding
  surfaced, fixed at the source.** The 渾儀 async-exposure boundary governs a module's **own** public
  seam by design ("this declared seam is synchronous", anchor scoping). Dogfooding the shell's
  `sans_io_pure` profile on 璇璣 exposed a latent false negative in the *profile's* whole-crate use:
  it pairs a 圭表 `must_not_call_inline` clock boundary (subtree-wide, filesystem-based) with the
  seam-only async boundary, so anchored at `crate` the two scopes coincide **only for a single-file
  crate** — the moment such a crate grows a submodule with a public `async fn`, the clock half still
  reacts while the async half silently does not. Rather than paper over it with honest-prose or a
  structure-pin on 璇璣, the fix is **at the source**: an **opt-in** `including_submodules()` on
  `AsyncExposureBoundary` (default OFF, so an existing boundary is byte-identical in reaction and
  projection — the same shape as `SemanticBoundary::including_trait_impls`). When set, the reaction
  descends the anchored module's whole subtree, emitting a per-module finding for every public
  `async fn` at or below the anchor; `sans_io_pure`'s async half opts in, so a pure kernel is
  sans-I/O *throughout*. The subtree walk (`walk_subtree_modules`) is a **rooted** analogue of the
  crate-scan family's `walk_module`, inheriting its guards verbatim (`#[path]` skip as a stated
  bound, `#[cfg]`-fileless tolerance, non-cfg missing → exit 2, symlink cycle → exit 2). Two
  false-negative-safety calls the adversarial review confirmed: (1) the violation `target` stays the
  boundary **anchor** (not the deeper enclosing module), so identity `(target, rule, finding_key)` is
  stable — enabling the opt-in adds only new, deeper findings and never re-identifies a seam finding
  (baseline stability); a seam finding is byte-identical to the single-module path (same collector,
  module string, `ordinal`, and `collect_uses`). (2) `push_multi_module_violations` flattens findings
  to `(anchor, rule, finding)`, so cross-module distinctness rests on the finding string carrying the
  **module-qualified owner** — always true for a valid inherent impl (`canonical_self_owner` prefixes
  the module), pinned by a two-submodules-same-method test. A body-nested `mod` is a stated bound (not
  public API, unlike unsafe-confinement's body-code observation — differing rule semantics, not a
  gap); `#[path]` parity between the clock (fs) and async (mod-tree) halves is the semantic
  dimension's stated bound, disclosed in the `sans_io_pure` doc rather than over-claimed. Ships as the
  OpenSpec change `async-exposure-subtree-scope`; additive / API-compatible → **patch** (0.1.8).
- **(v0.1.8) Self-governance grows a *semantic* dimension — dogfood `sans_io_pure` on 璇璣.** Until
  now the self-constitution (`tianheng_constitution()` in `self_governance.rs`) dogfooded only 圭表
  (crate dependency allowlists); the 渾儀 dimension governed adopters but never Tianheng itself. The
  self-constitution is migrated from `GnomonConstitution` to the full `Constitution` and now carries
  its first cross-cutting semantic self-boundary: `sans_io_pure` on 璇璣 — the crate that most owes the
  property, being the measure-only reaction model that reads no ambient clock and exposes no `async`.
  It reacts across both axes (圭表 must-not-call-inline `std::time::…::now`, 渾儀 must-not-expose an
  async public fn), asserted as **two independent Clean gates** (not merged) so a drift names the
  dimension it broke — 三儀 ⊥ 三儀 exercised on self. **No new public API, no OpenSpec change:** the
  test composes the existing public `check` / `check_all`, and the self-law projection is
  dimension-agnostic (it grows to include the two new boundaries, byte-checked by
  `self_law_projection_is_fresh`). This is the *second consumer* that retroactively validates the
  `sans_io_pure` pairing on real code — the surfacing adopter was `pacta`, the self-law is the second.
  (This dogfood is what surfaced the `async-exposure-subtree-scope` false negative above; with that
  fix landed, `sans_io_pure`'s async half is subtree-scoped, so 璇璣's "exposes no async" self-claim
  holds whole-crate, not merely because 璇璣 is single-file today.)
- **(v0.1.8) The family declares `#![forbid(unsafe_code)]`, and that is why `UnsafeBoundary` is *not*
  self-dogfooded.** Every workspace crate is `unsafe`-free; each now says so with
  `#![forbid(unsafe_code)]` at its crate root — the strongest, compile-time, unbypassable statement. By
  the `UnsafeBoundary` admission decision above (confinement-only; the crate-wide ban is `#![forbid]`'s
  *stronger* job), applying `UnsafeBoundary` to a family crate would be a vacuous boundary that can
  never react. So `UnsafeBoundary`'s demonstration belongs in an **example** (a crate with genuinely
  confined `unsafe`), never in self-governance — the honest scope kept literal. Likewise
  `max_visibility` is not self-dogfooded absent a real "this module must not leak `pub` past the crate"
  intent; its demonstration too is an example's job. Source hardening + a test's law, no reaction of its
  own → **patch** (0.1.8).
- **(v0.1.8) `#[cfg_attr(<pred>, path = "…")]` is recognized as a remap in both dimensions —
  implementation caught up to the spec.** A module written with the combined spelling
  `#[cfg_attr(windows, path = "os/windows.rs")]` (== the separate `#[cfg(windows)] #[path = "…"]`,
  a legal rustc cross-platform form) was NOT recognized as a `#[path]` remap by either dimension's
  module-graph probes, though the separate spelling was. The consequence was **dimension-dependent
  and included a silent false negative**: 圭表 treated `crate::os` as an ordinary file module and,
  when no conventional file existed, scanned nothing yet reported clean (exit 0 — the cardinal sin);
  渾儀 hit a spurious `missing_module_file_error` (exit 2, ungovernable). The
  `inline-symbol-path-confinement` spec **already** stated a "#[path]-remapped module (including a
  cfg_attr-wrapped #[path])" is a bound — so this is an *implementation-vs-spec bug*, fixed by making
  the implementation honor what the spec declared, **not an OpenSpec change**. Now both dimensions
  treat a `cfg_attr(path)` as the documented `#[path]` bound (cfg-blind: a conditional remap is a
  remap either way), so a boundary on such a module fails loud (exit 2) and no conventional file is
  silently governed as the wrong module. The false-negative-critical care is on the **inverse**: a
  `cfg_attr` that carries no `path` meta must stay a normal governed module (over-detection would
  drop it = a new false negative). Guibiao is `serde_json`-only, so its detector is a byte scanner
  that (a) skips the cfg **predicate** (the first meta) before matching, mirroring hunyi's syn
  `metas.skip(1)`, so a `path` cfg *key* is not mistaken for a remap, and (b) matches only a genuine
  depth-1 `path =` name-value — surfaced by adversarial review, which caught the missing
  predicate-skip (an over-detection + cross-dimension inconsistency). Found by the 打磨 bug-hunt.
  Bug fix + tests → **patch** (0.1.8). **Follow-up (second bug-hunt round):** the single-level fix
  still missed a *nested* `#[cfg_attr(a, cfg_attr(b, path = "…"))]` — both detectors went one level,
  so guibiao again silently passed (exit 0) while hunyi failed loud. Both detectors now **recurse**
  into a nested applied `cfg_attr` (skipping its predicate at each level), and both require a
  `path = "…"` **name-value** (a bare `path` / `path(…)` is not a remap) — closing the residual
  silent false negative and aligning the twins on every spelling. The recursion terminates (each
  byte-scanner call takes a strictly shorter slice) and cannot over-detect (only a genuine applied
  `path` name-value matches). Adversarially reviewed (25 inputs executed through both dimensions).
- **(v0.1.8) The 圭表 token scanner's `self`/`super` resolver is shared, not triplicated —
  within-crate, so no dimension line is crossed.** The inline-symbol-path scanner (`symbol_scan.rs`,
  v0.1.8) had added two more copies of the `self`/`super` relative-path resolution skeleton already
  in `use_scan.rs::normalize_module_path` — three byte-identical copies of the subtle
  `super`-pop loop + the **over-pop guard** (a `super` chain past the crate root returns `None`, so a
  non-crate-rooted path is never mistaken for an outward edge). Extracted into
  `path_vocab::resolve_self_super` (the documented shared foundation that already owns
  `canonical_module_path` / `path_within`), plus the one-line crate-root-shadow predicate as
  `is_crate_root_shadow`. **Not** the cross-dimension resolver consolidation BACKLOG defers (that is
  guibiao's *token* resolver vs 渾儀's *syn* resolver — a 三儀 ⊥ 三儀 / syn-quarantine concern); this
  is entirely within guibiao's syn-free scanner, one crate. Pure refactor, adversarially reviewed as
  strictly behavior-preserving (every `self` / single-/multi-/over-popping-`super` case at all three
  sites traced to byte-identical output); 193 guibiao tests unchanged. Surfaced by the 打磨
  hardcoding audit; no spec change → **patch** (0.1.8).
- **(v0.1.8) 渾儀's three whole-crate/subtree module walks share one module-descent helper —
  within-crate, so 三儀 ⊥ 三儀 does not bind.** `hunyi/scan.rs` grew a third whole-crate/subtree walk
  when `walk_subtree_modules`/`collect_subtree` landed (async-exposure subtree scope), joining
  `walk_module` (the `scan_crate` closure) and `walk_unsafe` (`scan_unsafe_sites`). All three carried
  a **byte-identical** module-descent loop with the four false-negative-critical guards (`#[path]`
  remap skip, inline-vs-file dispatch, symlink-cycle → exit 2, `#[cfg]`-tolerance / non-cfg-missing →
  exit 2). Three copies is the twin-drift bug class — a future fix to one guard silently diverging the
  others — the same class the 星表 metadata extraction (v0.1.6) and the louke `skip_block_comment`
  sharing were written to kill. Extracted the descent into one helper `resolve_child_modules`, which
  each walk calls before recursing (doing its own per-module work). **Crucially this does NOT violate
  三儀 ⊥ 三儀:** that law forbids a *dimension importing a sibling dimension*; all three walks live in
  one crate (渾儀), one file — the cross-crate consolidations the duplication audit surfaced (guibiao's
  token walk, louke's probe scanner) stay **rejected-by-design** (sharing them would force `syn` into
  the light core or couple the dimensions). The extraction is a **pure refactor**: adversarial review
  proved it strictly behavior-preserving — `visited` is monotonic over the same module tree, so the
  eager child-resolution is outcome-equivalent to the former lazy interleaving (OK ⟺ OK, Err ⟺ Err;
  only which module a cycle/missing-file error *names* can differ, asserted by no test). The one cost
  is an inline module's body is now cloned (the callers borrow their items). No spec change → **patch**
  (0.1.8).
- **(v0.1.8) Projection consistency: one runtime rule-line formatter, and anchor parity across all
  three `list` formats.** Two drift/parity fixes surfaced by the 打磨 consistency audit. (1) The
  runtime seam rule *label* was already the shared `RUNTIME_SEAM_RULE` const, but the folded
  `… (only origins: A, B)` **wrapper** was hand-copied in two places — the prod reaction
  (`check_crossing`'s violation `rule`) and the `list --format text` projection — the twin-drift bug
  class. Extracted `louke::runtime_seam_rule_line(&[&str])`, called by both, so the human-readable
  line is written once. The JSON projection deliberately keeps the label bare with origins as a
  field, so it is unaffected (the folded style matches the dyn/impl text projection). (2) A boundary's
  durable governance `anchor` surfaced in the JSON and Markdown `list` projections and in the `check`
  report, but **not** in `list --format text` — a real three-format parity gap. Added an emit-when-set
  `anchor:` line to every text boundary block (the same discipline the operand / `including_*` opt-ins
  use), so a boundary without an anchor stays byte-identical. Both locked by tests (the rule-line is
  shared by reaction and projection; the anchor appears in text iff set). Deferred as lower-value
  minor tidies (noted, not done): annotating the deliberate-vs-drifted hunyi/guibiao twin error
  strings, naming the runner's non-`Outcome` exit-code literals, and a `STRUCTURAL`-keys guard test.
  No spec change → **patch** (0.1.8).
- **(v0.1.8) Two new standalone examples for the capabilities the family cannot self-demo.** The
  0.1.8 capabilities were dogfooded and unit-tested, but the `examples/` set (a BACKLOG track-1
  role-split: a violating subject as an excluded sub-workspace + a green driver asserting the
  reaction) covered only the older `must_not_import` / `must_not_expose` / `only_origins`. Added two
  more, following the same pattern (own `[workspace]`, committed `= "0.1"` dep patched to local by
  `scripts/test_examples.sh`, a `tests/reaction.rs` that asserts the exit code AND a discriminating
  does-not-react case). (1) **`examples/unsafe-confinement`** — the flagship, the one capability that
  *structurally* cannot be shown inside the family (every family crate is `#![forbid(unsafe_code)]`,
  so a self-applied `UnsafeBoundary` would be vacuous): a crate with real `unsafe` confined to
  `crate::ffi` and a stray `unsafe` in `crate::net` that reacts, plus the confinement-only exit-2
  guard (an empty `only_under` points at `#![forbid]`). (2) **`examples/sans-io-pure`** — the shell's
  `sans_io_pure` profile on `tianheng`, a `crate::kernel` breaking both axes (an inline
  `std::time::…::now()` clock read + a `pub async fn` in a **submodule**), demonstrating in one shot
  the profile, the clock half, and the async **subtree** scope — with the load-bearing discriminator
  that a seam-only async boundary misses the submodule. `max_visibility` gained a demonstration as
  two `tests/reaction.rs` cases on a new `crate::internal` in `examples/hunyi-standalone` (a `pub`
  item breaches the `Crate` ceiling; the neighbouring `pub(crate)` passes it but reacts under the
  stricter `Super` ceiling — the rank-not-binary depth). All wired into `test_examples.sh` (the
  sans-io-pure entry also asserts the JSON carries BOTH a `module` and a `semantic` violation, so a
  silently-dropped axis fails CI). Examples only; no crate change → **patch** (0.1.8).
- **(v0.1.8) The `unsafe impl` finding is owner-qualified — the self type joins the identity, not
  just the trait.** A post-diff adversarial review found an `unsafe impl`'s finding rendered only its
  trait (`unsafe impl Send in crate::net`), so `Send for Foo` and `Send for Bar` in one out-of-subtree
  module collapsed to one finding: baseline the first and the second `unsafe` site passes unobserved —
  a false negative, the one forbidden class. The label now renders `unsafe impl {trait} for {self-type}`,
  mirroring the semantic dimension's own trait-impl-locality finding (already owner+trait-qualified for
  exactly this "a baseline cannot mask a new one" reason) and the spec's standing no-false-negative
  clause; the distinct-findings scenario is extended to name the self-type axis. The self type is
  rendered by the shared lexical `type_to_string` (the never-collapse-to-`None` renderer the exposure
  hearts use) with a positional `_#n` fallback for an unrenderable type — the light `unsafe` walk stays
  resolution-free. Not an OpenSpec capability change (the requirement is unchanged; the code is brought
  into fuller compliance with it) → **patch** (0.1.8).
- **(v0.1.8) The 圭表 type-alias resolver skips a defaulted generic parameter's `=`.** The same review
  found `type_aliases` stopped at the first `=`, so `type Clock<Tz = LocalTz> = std::time::SystemTime;`
  resolved `Clock` to the default `LocalTz` instead of its real target — a confined clock reached through
  the alias would pass unobserved (a false negative). The scan to the aliasing `=` now skips a `<…>`
  parameter list whole (via the shared `skip_angles`), and the target runs to the top-level `;` honoring
  `[]`/`()`/`{}` nesting (an inner `[T; N]` `;` no longer truncates it). Behavior-preserving for an
  ordinary `type X = a::b::C;` → **patch** (0.1.8).
- **(v0.1.9) `must_not_call_inline` grows an opt-in `.strict_external()` — a new `ModuleRule`
  variant (patch-safe), not a field.** By default the inline confinement resolves an un-`use`d bare
  head to a fake-local `{module}::…` path (the load-bearing fallback the `type`-alias / re-export
  closure depends on), so a fully-qualified, un-`use`d external call (`chrono::Utc::now()` with no
  `use chrono`) resolved local and was **silently missed** while a sysroot `std::time::…::now()` was
  caught — an under-coverage false negative reading as full coverage. `.strict_external()` closes it:
  a bare head matching a **declared dependency** (rename-aware, `-`→`_`-folded to its import
  identifier) resolves as external. It rides a **new variant** `ConfineInlineSymbolPathExternal`
  (payload-identical to `ConfineInlineSymbolPath`), **not a field** on the existing variant — a field
  on a public enum variant is a downstream `E0063`/`E0027` compile break, while a variant on a
  `#[non_exhaustive]` enum is patch-safe (the same growth pattern as `RestrictDependencySourcesTo`).
  The twin-variant is `#[doc(hidden)]` and recorded as **0.2.0 model-surface debt** (fold back into
  one variant + field once the surface is narrowed — resolved by the later 0.2.0-line decision
  below). **Identity parity is
  baseline-critical:** both variants route through one shared path, so `label()` returns the
  *identical* `"inline symbol path confined to module"`, `polarity()` is `DenyBreach`, and
  `target`/`finding` are byte-identical — adding the flag to an already-baselined boundary never
  re-keys a finding (`strict_external` is emitted only in `text()`/`json_params()`, never in
  identity). The reclassification honors a **full local-precedence ladder** (first match wins,
  gated entirely on the flag so the default path is byte-identical): (i) the per-file use-map, (ii)
  a crate-root module shadow, (iii) **any** local module `{module}::head` at any depth (from the
  complete crate module-path set — not `root_modules`, which holds only crate-root children), (iv)
  **any** local item definition across all namespaces (mod/struct/enum/union/trait/type/fn/const/
  static — a byte-scanned set, not `ctx.defs`, which holds only type-alias + `pub use`), then (v) a
  declared-dependency match ⇒ external. Only if (i)–(iv) do not claim the head does dep-match fire,
  so a local item named like a dependency **at any depth** stays local (no false positive — the
  first-cut flaw the review caught was threading only `root_modules` + `ctx.defs`). **三儀 ⊥ 三儀
  kept:** 圭表 grows its **own** syn-free, `serde_json`-only rename-aware deps reader
  (`cargo_metadata::dependency_import_names`) from the `package["dependencies"]` value it already
  reads via 星表 — a small parallel of `hunyi::crate_scope::dependency_names`, **no** `guibiao → hunyi`
  edge and no new crate dependency (the closed guibiao allowlist `["serde_json","xuanji","xingbiao"]`
  still holds — self-governance green). An `extern crate dep as alias;` rename remains a **stated
  bound** (the use-map reads `use` only; rustdoc must not overclaim "all external calls caught"), and
  the glob-hazard reaction **extends** to external-crate globs under the flag (an external glob is an
  ancestor of the prefix → reacts fail-closed). Twice adversarially reviewed; guard test proves the
  reaction (and disappears when only the resolver branch is reverted), FP-safety tests cover a deep
  local module / local fn / local alias, and a baseline-churn guard locks identity parity → **minor**
  (0.1.9, additive opt-in capability).
- **(0.2.0 line) Violation identity is a structured observed fact, not its presentation.** Each
  observation dimension owns typed fact schemas and their human rendering; 璇璣 carries the
  vocabulary-neutral `FindingKey` envelope and compares current identities by
  `(target, rule, finding_key)`. The envelope is deliberately the narrowest shared measure that
  fits the built facts — a namespace and fact code plus flat, canonically name-ordered string
  fields — rather than a recursive generic value model: field meaning, typed fact variants, and
  rendering stay in the dimension that observed them, so adding vocabulary never makes 璇璣 model a
  儀. A human finding sentence is presentation; a named field's canonical observed value is wire
  identity, so the dimension owns that canonicalization and its compatibility cost without exposing
  resolver internals as facts. The human `finding`, `file`, `reason`, severity, polarity, anchor,
  and debt metadata remain diagnostic context rather than identity. Newly generated baselines use
  version 2 and retain both the structured key and human finding. Version-1 baselines remain
  readable and match current violations only through their exact legacy text triple; writing a new
  baseline upgrades the durable contract without guessing a key that was never recorded.
- **(0.2.0 line) Rule construction is builder-owned; rule inspection stays open-ended.** The 0.1
  patch line exposed a concrete model-growth failure: adding the `.strict_external()` modifier as a
  field would break downstream struct expressions and closed-field matches, so it required a hidden
  duplicate `ModuleRule` variant. Every data-carrying `Rule` / `ModuleRule` variant is now itself
  `#[non_exhaustive]`: the existing DSL is the sole construction surface, while consumers retain
  read access through `CrateBoundary::rule()` and the new symmetric `ModuleBoundary::rule()`, using
  `Variant { known, .. }`. The strict-external twin consequently collapses into one inline variant
  with a modifier field. This spends the breaking window on accidental model construction only;
  `Constitution`, the boundary DSL, projections, reactions, and violation identity stay unchanged,
  and the pacta/modou reference probes remain green.
- **(0.2.0 line) The composed adopter surface is compile-reacted and classified by purpose.** A
  serious adopter uses `tianheng::prelude::*` for both declaration and `Outcome` inspection, so the
  prelude is not honestly reducible to builders alone. Its declaration/execution and reaction-
  inspection tiers now carry the same 0.2.x compatibility status and are named by an external-view
  integration test; an accidental re-export deletion therefore fails compilation instead of
  silently contradicting documentation. The test exposed and closed one asymmetry by adding the
  existing `ModuleRule` beside `Rule`, making both boundary `rule()` accessors usable through the
  recommended entrypoint. Hidden drafts and granular semantic checks remain outside this contract;
  the explicit `check_semantic` root alias is signature-coupling only, while composed evaluation
  stays `Constitution` plus `run`.
- **(0.2.0 line) A unified Constitution has one inspectable evaluation path.** The composed example
  exposed a real mismatch: declaration was unified, but a library test had to split back into static
  and semantic checks because only CLI `run` composed all three dimensions. Public
  `check_constitution(&Constitution, &Path) -> Outcome` now sits above the same private evaluator as
  `run`, preserving static-first error precedence, merged findings, and the always-run orphan-probe
  audit without duplicating composition. It is presentation-free, not observation-free: the
  explicit manifest is read through Cargo metadata and source scans. Baselines, coverage advisories,
  formats, manifest discovery, and process output remain shell concerns. This closes the primitive
  testing seam but deliberately does not pre-build the deferred fixture/assertion harness.
