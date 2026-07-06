# Backlog

Forward-looking work, deliberately deferred. Promote an item to an OpenSpec change when
you pick it up. Every future reaction obeys the drift law:

> **No drift type without an observation source. No target type or name without a
> reaction.**

Nothing here is "designed" yet — reaction *phases* with their observation sources named,
not APIs. A new observation dimension is **a crate, born when it is built** (never a
pre-created empty stub); the heavy dependency it needs is quarantined to that crate so the
`guibiao` core stays `serde_json`-only.

## Version horizons — what a 0.1.x patch carries vs what earns 0.2.0

The version follows SemVer honesty (`AGENTS.md`), not milestone size: **non-breaking →
patch, breaking → minor**, and never a vanity minor bump. The admitted 三儀 layer is
complete, so the near-term line is **0.1.x, a patch line**:

- **0.1.x (patch)** — additive depth on an existing observation source (a born-when-built
  capability that widens nothing already public), packaging / CI / license-bundling hygiene,
  and governance-doc corrections. Every *additive* forward item below lands here.
- **0.2.0 (minor)** — earned only by a **breaking** public-API change. The one on the table
  is the deliberate pre-1.0 refinement of `guibiao`'s widened public surface
  (`baseline` / `coverage` / `projection` / `check_and_cover`, made `pub` as the price of the
  crate split — see `PROJECT.md`, Decisions). Two further **breaking** candidates would ride the
  *same* minor rather than each forcing its own: the `Violation::new` newtype (retiring the
  4-positional-`String` footgun) and a **structured baseline** (findings as data, not strings — which
  is what unblocks the deferred 渾儀 `PublicSeam` / `ExposureSubject` typing). Bundle them into one
  honest 0.2.0, not three. Additive adopter-facing work (the LSP crate, the
  adopter-facing 潛移 generator) does **not** force a minor: it rides whatever release is
  current, a patch unless bundled with that breaking refinement. **Guardrail (reference-consumer
  steer):** the 0.2.0 break is quarantined to the *internal* surfaces — the `Baseline` data model,
  the `Violation::new` newtype, `guibiao`'s widened `pub` face — and **must never break the
  adopter-written builder** (`Constitution` / `CrateBoundary` / `run` / `prelude`). A real
  consumer's adoption story is a **drop-in swap**; breaking the builder breaks the zero-pain-upgrade
  promise and damages Tianheng's own adoption curve. A stable builder face is the *price* of the
  0.2.0 cleanup, not a casualty of it.

### 0.1.5 — known-depth consolidation (the current patch line)

0.1.5 has converged from scope map to shipped state. Its built items are recorded once in the
dimension / 三司 sections below; the remaining forward work stays there as forward depth. The 0.2.0
bundle above remains the only currently named breaking line.

### 0.1.6 — metadata SSOT extraction (dissolve the "don't share" prose into an enforced boundary)

The three dimensions reimplement parallel logic (三儀 ⊥ 三儀 forbids sharing code), and the
adversarial-review rounds surfaced this as a recurring **twin-drift** bug class: one dimension's
copy is hardened or corrected while its sibling's is not (symlink recursion, `file_module_path`'s
`lib`/`main` depth, cross-boundary dedup, keyword handling — and the **proc-macro `crate_root_file`
false negative**, a pure metadata-layer drift). `find_package` / `crate_root_file` /
`dependency_names` are written **twice** — `guibiao/src/cargo_metadata.rs` and
`hunyi/src/metadata.rs` — both `serde_json`-only and dimension-agnostic.

The current "intentionally not shared" stance (`PROJECT.md`, Name-resolution decision) is **prose,
not a reaction**, and its "a shared resolver would force `syn` into the core" reasoning is about the
*resolver*, not `serde_json`-only metadata parsing. Since Tianheng governs its own dependency graph,
the safe, non-proliferating, one-way dependency is itself **enforceable** — as 璇璣 already proves. So:

- **Extract** the dimension-agnostic cargo-metadata substrate (`find_package`, `crate_root_file`,
  `dependency_names`, the neutral package/target/dep-name lookup) into a **new low crate**, sibling
  to 璇璣, **below the 三儀**. Each dimension depends on it one-way (downward); its own self-law is
  `restrict_dependencies_to(["serde_json"])`. **This is the new dogfood**: Tianheng enforcing, on its
  own graph, the very (non-)proliferation it exists to catch — a single SSOT makes the metadata
  twin-drift class *structurally impossible*.
- **Scope — what moves vs not.** Only the dimension-agnostic substrate moves. `classify_source`
  (registry/git/path) and dep-kind (normal/dev/build) **stay in 圭表** — they are its *observation
  semantics* (crate-source-boundary), not neutral infrastructure. The `syn`-AST **resolver** and the
  byte-scan-vs-AST **module reachability** (`file_module_path`/`reachable_modules` vs
  `resolve_module_items`) do **not** move: sharing them genuinely forces `syn` into the light core, a
  real constraint — so that half of the prose is **kept but narrowed** to "the resolution *engines*
  differ", not a blanket "nothing is shared".
- **Not 璇璣.** Metadata parsing is *observation* and spawns `cargo` (IO), so it must not sit in the
  measure-only, verdict-free model. The prose's "dimension vs 璇璣" dichotomy missed the third home: a
  shared observation **substrate** crate, which is neither dimension nor measure.
- **Mechanics.** Behavior-preserving; the moved symbols are `pub(crate)` (not public API), so this is
  the **璇璣-extraction precedent** — an internal refactor, *not* an OpenSpec capability change: a
  steward-gated `self_governance.rs` amendment (the new crate's boundary + each dimension's allowlist
  gains the one downward path), a `PROJECT.md` Decisions update, and **retiring the metadata half of
  the Name-resolution prose** (prose → reaction; run `dissolve` to classify it).
- **Naming (a seed, not yet bound).** The substrate is neither a fourth 儀 nor 璇璣: it is the
  shared *declared-workspace-data* base (it reads `cargo metadata`), sibling to 璇璣's shared
  *reaction-model* base — so the name wants the 璿璣玉衡 / astronomical-artifact register. Two
  finalists: **星表** (`xingbiao`) — the tabulated declared catalog every instrument references;
  best semantic fit and in-scheme, but its `-biao` echoes 圭表 (`guibiao`); and **圖籍** (`tuji`) —
  the authoritative register of what a domain contains (most literal for the workspace inventory,
  distinct sound, governance-resonant), but off the celestial-instrument scheme (輿圖/版籍 register).
  Trade-off: scheme purity → **星表**; semantic precision / avoiding the `-biao` echo → **圖籍**.
  **Avoid 基準** (`jizhun`) — it collides with the model's existing `Baseline`. Runners-up carried
  real strikes: 星圖 (`xingtu`) risks a "graph/map" misread against the universal-graph-API non-goal;
  曆表 (`libiao`) connotes a time-series ephemeris where the metadata is a snapshot (and repeats the
  `-biao` echo); 經緯 (`jingwei`) is an evocative reference-frame but a loose fit for "reads the
  manifest"; 候簿 (`houbu`) uniquely encodes *observation* but 簿 drags it toward an event log.
  **Bind the name at build time**, after `dissolve` locks exactly which symbols move (name follows
  the settled scope), not now.
- **Version: 0.1.6 patch.** Non-breaking (zero adopter-facing API change; moved symbols are internal),
  so patch is the honest bump — a minor would be a forbidden vanity bump. It rides **its own** release,
  separate from 0.1.5's polish/bug-fixes, so the constitution amendment lands as one clean,
  steward-reviewable change, not a rider on unrelated diffs.
- **Considered alternative (kept in reserve):** a cross-dimension **conformance reaction** — a
  `cargo test` gate feeding identical inputs to each dimension's parallel logic and asserting they
  agree on the overlapping contracts. It catches drift *without* moving code, and remains the answer
  for the Tier-B logic that genuinely cannot share (the resolver); it complements, not replaces, the
  metadata SSOT.

## Reaction phases — the 三儀 (observation dimensions)

Ordered by readiness. All three instruments ship in v0.1.0 (圭表 static, 渾儀 semantic, 漏刻
runtime); the entire admitted 三儀 layer is now built. What remains below is the rejected set
per dimension and the 三司 governance/observability layer.

### 圭表 (Guībiǎo) — the static dimension  · crate `guibiao`  · **BUILT — proven core (v0.1.0, from modou); growing by depth (v0.1.2 crate-source-boundary); hardened in v0.1.4**
Observation source: `cargo metadata --no-deps` (the declared manifests) + a source `use` scan.
Like 渾儀, 圭表 grows by **depth** (finer reads of the same observation source), not by width.

- **Declared dependency-source boundary — crate-source-boundary**: **BUILT (v0.1.2)**
  (`restrict_dependency_sources_to([SourceKind…])`). Deepens the dependency reaction from
  *which crate* (by name / external-internal split) to *which declared source kind* — git vs.
  registry vs. path — reading the same `--no-deps` `source` field one notch finer. Hermetic; the
  publish-hygiene case (a manifest declaring no git source, optional git included). Two stated
  bounds: it observes the **declared** source (not the resolved one — `[patch]`/`replace-with`
  is not seen), and it is source-kind hygiene, not a `cargo publish` oracle (a `{ git, version }`
  dep is flagged though it would publish).
- **Module-source hardening**: **BUILT (v0.1.4)**. Module boundaries now use Cargo's observed
  lib/bin `src_path` as the compiled source root, and `#[path]`-remapped modules stay outside the
  reachable graph instead of being governed through a same-named conventional orphan file. This is
  a false-negative closure / stated-bound repair, not a new capability.
- **Inline-module orphan-shadow**: **BUILT (v0.1.4)**. The inline twin of the `#[path]` orphan
  hazard: an inline-only `mod name { … }`'s same-named conventional file (`name.rs`/`name/mod.rs`)
  is now recognized as an orphan and excluded from the scanned file list, so an inline target stays
  the self-describing exit-2 constitution error (never a silent pass over the orphan, never a
  phantom child mined from it) rather than governing a file rustc does not compile. Gated on
  inline-**only** so the `#[cfg]`-dual-declaration case stays within the existing cfg-blind bound.
  A propose- and apply-stage adversarial-review-driven false-negative closure; `crates/guibiao`
  only, no new capability. (渾儀 was immune — its AST descent is declaration-driven.)
- **Multibyte char-literal lexing — documented robustness bound (not a known FN).** *From the
  v0.1.5 hidden-bug sweep.* The `use`/`mod` lexer's simple-char branch assumes a one-byte char
  body, so a multibyte char literal (`'é'`) in an adjacent-literal pattern can misparse a few bytes.
  The sweep could construct **no valid-Rust input** where this drops or fabricates a `use`/`mod`
  (the misparse is bounded and emits lone quotes harmlessly), so it is a latent robustness weakness,
  not a confirmed FN/FP; a defensive fix (scan a char literal to its next unescaped `'`) is optional.
  Recorded, never silent.

**Built depth:**
- **Closed inbound allowlist — `must_only_be_imported_by` — BUILT (v0.1.5).** The **closed dual** of
  `must_not_be_imported_by`: "only `crate::facade` may import `crate::internal`" — every other
  importer reacts. Observed on the crate-wide `use` scan (no new source), declarative-not-lint.
  Surfaced by worklane's dogfooding as a live reference need. The open design question — new boundary
  type vs. a mode — resolved in favour of a **new `ModuleRule::MustOnlyBeImportedBy` variant** (the
  inbound dual of `RestrictImportsTo`, mirroring how it is distinct from `MustNotImport`); polarity
  `AllowlistGap`, projected under the surface-qualified `only_importers` key, crate-root protection a
  constitution error. Shipped as an OpenSpec change modifying `module-boundary` (ADDED requirement).

**Forward depth — potential, not active (adopter-surfaced):**
- **Module-scoped external-crate confinement — `restrict_dependency(C).to_module_subtree(S)`**:
  *not built.* The middle cell between crate-granularity (`restrict_dependencies_to`, whole-crate)
  and intra-crate module direction (`restrict_imports_to`, which by design **never flags an external
  import** — see its test): "crate Y *may* depend on C, but C may be imported only under subtree S" —
  the FFI/platform-vocabulary confinement pattern. Observable on the existing static `use` scan (it
  already sees external imports; the reaction just does not yet fire on them at module granularity),
  so it obeys the drift law; additive/patch. **Adopter-surfaced.** Two layers: (a) **import
  confinement** (`use C::…` only under S) reuses the scan directly — the in-scope core; (b)
  **inline-symbol-path confinement** (`C::foo()` / `C::CONST` written in bodies with no `use`) needs
  an all-body path scan deeper than any current pass — a later phase / stated bound, not the core.
  Not `cargo-deny`'s lane (declared/per-module, not resolved/whole-graph).

**Declined — externally covered (not a forward depth):**
- **Resolved dependency-source / build-provenance** — *declined.* Cargo-deny owns the resolved,
  whole-graph source-provenance layer; Tianheng keeps the declared, per-target manifest layer. See
  `PROJECT.md`'s 圭表 source decision for the full A/B rationale.

### 渾儀 (Húnyí) — the semantic dimension  · crate `hunyi`  · **BUILT — originally-conceived layer (v0.1.0); growing by depth (v0.1.2 dyn/impl-trait stair + async-exposure; v0.1.3 re-export + trait-impl exposure); hardened in v0.1.4**
Observation source: the **AST** (`syn`). Sees what the `圭表` `use`-scan cannot — semantics
in the syntax tree: `pub` signatures, `impl Trait for Type`, attributes/derives, visibility.
The observation-source fork is **resolved**: `syn` was chosen (stable; its syntactic partial
coverage — glob / cross-crate re-export / macro / inference blindness, while local `pub use`
chains, incl. multi-hop and aliased, *are* followed — is *stated*, never silently passed),
over `cargo rustdoc --output-format json` (nightly + unstable format).
Single-module resolution keeps `#[path]`-remapped modules outside scope instead of governing a
same-named conventional orphan file (v0.1.4 hardening; stated-bound repair, not a new capability).

- **Public-API type leakage — signature-coupling** (flagship): **BUILT.** "A module's public
  API must not *expose* a forbidden type" — depending on a type internally is fine; leaking
  it across the public surface is the violation. The semantic companion to the dependency
  boundary.

Admitted **and now built** (each born when built, each passed the capability-admission test
in `PROJECT.md` — declarative, no *essential* gap, anchorable):
- **Type-anchor / local trait-impl surface**: **BUILT** (`TraitImplBoundary`,
  `.only_implemented_in(...)`) — "only `crate::commands::*` may `impl Command`"; the impl-site
  is a `syn`-resolvable local element, the second 渾儀 anchor type.
- **Forbidden-marker / attribute / visibility boundaries**: **BUILT** (`ForbiddenMarkerBoundary`,
  `VisibilityBoundary` `.must_not_declare_pub()`) — "`internal` exposes no `pub`".
- **Dyn-trait exposure — type-shape exposure**: **BUILT (v0.1.2)** (`DynTraitBoundary`,
  `.must_not_expose_dyn()`) — "the core's public seam must not leak `dyn`". The first **depth**
  addition: it deepens signature-coupling's reaction from a forbidden *named type* to a
  forbidden *type shape* (a `dyn` node at any depth in the public surface), reusing its
  surface walk + resolver and adding only a trait-object leaf. Shape-only.

The originally-conceived 渾儀 layer is complete, but the dimension still grows by **depth**
(new capabilities on the same `syn` observation source, each a born-when-built patch — see
dyn-trait above), not by width (no new observation source). Named next depths and the rejected
set follow.

Built depths past the shape-only dyn (same `syn` source):
- **Operand-scoped dyn** (`must_not_expose_dyn_of([…])`) — **BUILT (v0.1.2).** Forbid only a
  *named* trait's `dyn` rather than any: a `dyn` whose **principal trait** (first trait bound)
  canonicalizes into the forbidden set reacts, resolved through the shared 渾儀 resolver (exact-
  or-module-prefix, re-export消歧) exactly as signature-coupling resolves a forbidden type. The
  next rung on the `name → shape → named-operand` stair. Empty operand set degenerates to
  shape-only (any `dyn`), never a no-op; auto-trait markers are never operands; an unresolvable
  principal is the stated resolver bound.
- **Impl-trait (existential) exposure** (`ImplTraitBoundary`, `.must_not_expose_impl_trait()`) —
  **BUILT (v0.1.2).** The **existential complement** of dyn-trait's dynamic-dispatch shape: a
  public seam must not *return* a written `impl Trait` (RPIT), which leaks an unnameable type and
  silently commits to its auto-traits. Shape-only; reuses the public-surface walk and the `dyn`
  bound renderer, governing **return positions only**. Argument-position `impl Trait` (APIT,
  universal) and `async fn`'s implicit `impl Future` are stated out-of-scope bounds.
- **Operand-scoped impl-trait** (`.must_not_expose_impl_trait_of([…])`) — **BUILT (v0.1.2).** The
  named-operand depth of the shape-only impl-trait, mirroring the dyn stair: a returned `impl
  Trait` whose **principal trait** canonicalizes into the forbidden set reacts (so a seam may allow
  `impl Iterator` while forbidding `impl crate::Port`), resolved through the shared 渾儀 resolver
  and generalized with dyn onto one `ShapeExposure` collector + `principal_trait_path`. Empty set ⇒
  any (never a no-op); return-position scoping and the APIT/async bounds are inherited.

- **Async-exposure** (`AsyncExposureBoundary`, `.must_not_expose_async_fn()`) — **BUILT (v0.1.2).**
  The **implicit-existential** complement of impl-trait: a public seam must not declare an `async
  fn` (its compiler-inserted `impl Future`), observed from `sig.asyncness` over public free fns /
  inherent methods / trait method declarations (trait-impl methods and private items excluded).
  The finding is an **owner-qualified item identity** (`async fn <Ty>::name(…)`) so same-named
  async fns across impls/traits never collide under the baseline (a false-negative guard). Its
  declarative gate is the dimension's weakest but holds (implicit existential at a declared seam,
  anchor-scoped). Complementary to impl-trait's *written* `-> impl Future`.

- **Trait-impl exposure** (`.must_not_expose(…).including_trait_impls()`) — **BUILT (v0.1.3).** An
  **opt-in surface depth** on signature-coupling (not a new boundary type): it closes the v1
  trait-`impl`-out-of-scope bound by observing a trait impl's **impl-site-authored** positions —
  the trait ref's generic args (`trait-arg`), the `Self` type bare+nested (`self`), associated-type
  bindings (`assoc {name}`), the impl's own generics/`where`-clause keyed by bounded type
  (`where {type}`), and the method **return as written** (`method {name} return`, catching an
  RPITIT-refined concrete return — the false negative that made "exclude all method sigs" untenable).
  Params/receiver stay trait-dictated non-goals; implementing a forbidden *trait* is
  `must_not_acquire`/locality's concern (stated non-goal). Position-qualified seams keep findings
  injective; reuses the resolver and `BareFallback::Ignore` verbatim, no new `syn` feature.
- **Re-export exposure** (`pub use`) — **BUILT (v0.1.3), default-on.** Closes a confirmed false
  negative in the flagship: `collect_item_exposures` had no `Item::Use` arm, so a bare
  `must_not_expose("crate::infra")` silently passed `pub use crate::infra::DbPool;` (which republishes
  the forbidden type under the module's path). Now a bare boundary observes named public re-exports
  (bare / aliased / grouped / facade-chain / whole-module / `self`-group), and a glob reacts when its
  root is in/under the forbidden set; `pub(crate)`/private/`as _` and sibling/ancestor
  globs are stated bounds (`pub extern crate` now reacts — see the external-crate exposure entry).
  Seam-keyed by the exported path for baseline injectivity. **Behavior-change
  (the first in 0.1.x):** API-compatible (patch), but a bare boundary now reacts to re-exports it
  previously missed — a downstream's green CI may go red on a real leak; adopt via `warn`/`Baseline`.
  This is the standing precedent that a false-negative closure is a patch (the contract's
  false-negative-first ordering over compatibility comfort).
- **External-crate exposure** (inline extern paths + facade chains) — **BUILT (v0.1.4), default-on.**
  Adopter-driven (a facade whose "must not re-export core's spi" invariant lived only in doc prose).
  Closes the flagship's inline-extern false negatives: a `pub use dep::spi::Foo;` re-export, a
  `-> dep::spi::Foo` signature, and a **local facade chain** ending at an extern type were silently
  dropped (only the *use-aliased* form reacted). Extern-determination is the crate's **external-crate
  name set** — declared deps (`.rename`-aware, `-`→`_` normalized) ∪ sysroot (`std`/`core`/`alloc`/
  `proc_macro`/`test`). A bare `pub use` head uses the raw set (extern by 2018+ grammar); a bare
  type-position head uses it minus the governed module's own child modules (per-module shadow) —
  a `PathExposure.is_reexport` bit selects which, so a local `mod serde` yields no FP in a signature
  yet a subtree's `pub use serde::X` still reacts (no FN). Bare-fallback branch after the `use`-map;
  only the exposure resolve + re-export closure (dyn/impl-trait operand and seam identity untouched).
  Patch, API-compatible (DSL unchanged), v0.1.3 precedent — though it also touches v0.1.0
  signature-coupling. Three adversarial review rounds: refuted an initial edition-grammar shortcut,
  drove the hardening (sysroot, hyphen, module-shadow, call-site scope), and caught a crate-level
  shadow that was both an FP and an FN (→ the per-position split). Residual stated bounds: extern glob
  leaves / foreign-module renames (foreign AST), a **module-scoped** source `extern crate as` rename
  (the crate-root form + `pub extern crate` are now observed — see the extern-crate exposure closure),
  distinct `[lib] name`, privately-`use`d-bare-name facade hops, 2015 relative local re-exports. Modifies
  `semantic-reexport-exposure` + `semantic-signature-coupling`.

**Residual false negatives / positives deferred from the v0.1.4 adversarial review (documented,
never silent — the FN-first contract requires a known gap be recorded, not hidden):**
- **Crate-root `extern crate X as Y;` scoping — `crate::<alias>` FN + submodule-`mod`-shadow FP —
  CLOSED (v0.1.5).** The crate-relative spelling `crate::Y::…` is now rewritten to the real crate (an
  unconditional `crate::<alias>` rewrite — only the segment immediately after `crate`), and the bare
  `Y::…` rewrite is suppressed under a governed submodule's own child `mod Y` (`renames −
  child_module_names`) while kept for every unshadowed module (the no-FN requirement the prior review
  flagged). Both rustc premises verified by compilation. Shipped as an OpenSpec change modifying
  `semantic-reexport-exposure`. The **module-scoped** rename stays a bound (only crate-root renames
  are collected).
- **Re-export head shadow FP — CLOSED (v0.1.5).** A `pub use serde::X` head in a module that also
  declares a local `mod serde` was misattributed to the dependency (rustc shadows it — E0432). The
  re-export head oracle now resolves against `externs − child_module_names` (only the governed
  module's own child modules), and the leading `::` is preserved so `pub use ::serde::X;` still
  reacts (no FN). Shipped as an OpenSpec change modifying `semantic-reexport-exposure`.
- **Facade-closure re-export head shadow FP — CLOSED (v0.1.5).** The narrower sibling of the above:
  a cross-module facade (`crate::b`'s `pub use crate::a::Foo;`) reaching a head shadowed in its
  *defining* module (`crate::a`'s `pub use dep::Foo;` under a child `mod dep`) still mis-canonicalized
  to the dependency, because the crate-wide re-export closure (`collect_reexports`) resolved every
  collected re-export against the raw extern set. Now `collect_reexports` takes the defining module's
  `child_module_names` and, for a **bare** head, resolves against `externs − child_mods` **and**
  `renames − child_mods` (mirroring the direct oracle's `externs_reexport`/`renames_bare` in full, so
  both the extern-set and crate-root-rename-alias variants close); a **leading-`::`** head keeps the
  raw sets (the closure now reads `use_item.leading_colon`, which its `collect_use_tree` walk
  discards, so `pub use ::dep::X;` through a facade still reacts — the propose-stage review caught that
  the naive extern-set-only fix would have introduced that FN). Fixed at the single collection site,
  so every consumer of the `reexports` map benefits. Shipped as an OpenSpec change modifying
  `semantic-reexport-exposure`.
- **Inherent-`impl` associated `const`/`type` exposure FN — CLOSED (v0.1.5).**
  `collect_item_exposures`'s inherent-`impl` arm now observes public `ImplItem::Const` (its type) and
  `ImplItem::Type` (its target), seam-qualified by `inherent_assoc_seam(kind, owner, name)` →
  `{const|type} <{owner}>::{name}`, so a forbidden type in a public inherent assoc const/type reacts
  (was skipped — only methods). Shipped as an OpenSpec change modifying `semantic-signature-coupling`.
- **`dyn`/`impl Trait` shape collectors share the inherent-impl assoc blind spot — residual, deferred
  (v0.1.5 sweep).** `collect_item_dyn_exposures` and the returned-`impl-Trait` collector also iterate
  `ImplItem::Fn` only, so `impl Foo { pub type T = Box<dyn crate::infra::Secret>; }` still silently
  passes the **shape** reaction (`must_not_expose_dyn` / `must_not_expose_impl_trait`) — a sibling FN
  in a different capability (shape exposure, not signature-coupling; the latter now catches the
  nested `crate::infra::Secret`). Fix: give those collectors the same public `ImplItem::Const`/`Type`
  arms. Additive FN closure; its own change, not mixed with the signature-coupling fix. Born when
  built.

Forward depths (born when built, same `syn` source):
- **`must_not_expose_existential` (unifier)** — a possible future capability folding impl-trait
  (written `impl Future`/RPIT) and async-exposure (implicit `impl Future`) under one "no
  existential at this seam" rule. Deferred: the two syntactic signals stay distinct rules until a
  unification earns its own admission (it must not blur the two findings' identities). Not built.
- **`UnsafeBoundary` — subtree-confined `unsafe`**: *not built.* "All `unsafe` (blocks, `unsafe
  impl`, `unsafe fn`) lives only under subtree S; elsewhere reacts" — the auditability boundary of a
  layered Rust crate. Same `syn` observation source (an `unsafe` token is directly visible); reaction
  is a whole-crate/subtree scan of the forbidden-marker family. Additive/patch, **adopter-surfaced.**
  **Value scoped precisely:** the pure "crate X is `unsafe`-free" case is already *stronger* as the
  compiler's `#![forbid(unsafe_code)]` (compile-time, unbypassable) — compile-time-first adopters
  should use that; `UnsafeBoundary`'s unique, non-compiler-expressible value is **intra-crate subtree
  confinement** ("unsafe only under `crate::ffi`") plus one unified cross-crate declaration. Spec it
  as subtree confinement, not the per-crate on/off the attribute already covers.
- **Visibility ceiling — `max_visibility(Crate|Super|Module)`**: *not built.* Generalizes the binary
  `must_not_declare_pub` (which becomes the `max_visibility(Crate)` special case, kept as sugar —
  additive, not breaking) to a parameterized ceiling: an item declared *above* the ceiling reacts,
  `pub(crate)`/below is allowed. Observation exists (visibility is already read). Non-compiler-
  expressible value: the compiler enforces a `pub(crate)` item can't be *used* externally but happily
  accepts *widening the declaration* to `pub`; the ceiling governs the declaration's evolution (never
  widen) — a governance fact the type system does not carry. **Adopter-surfaced.**

**Internal structure (refinement, not capability) — v0.1.4:** 渾儀's internals were structured
where a live pain existed — the finding-string formats centralized into one `SemanticFinding`
catalog, the ~8k-line `lib.rs` split into `lib` / `dsl` / `tests`, and the sibling-safe
`::`-containment rule converged into one `path_within` (retiring a drift-prone hand-copied
false-positive/false-negative rule). **Deferred — `PublicSeam` / `ExposureSubject`:** typing the seam
and subject is prep for a **structured baseline** (findings as data, not strings), not a live-risk fix
(collision is tested-closed by seam-qualification + the injectivity tests); it is breaking (0.2.0) and
raises a seam-type layering question (the seam is stored/stamped in the lower `resolve.rs`, but the
finding vocabulary lives in `lib.rs`). Born when the structured baseline is greenlit — see
`PROJECT.md`, "Structure semantic observation facts".

Explicitly **rejected** (essential gap — would be a false-negative engine, see `PROJECT.md`):
`Send`/`Sync` constraints (inferred auto-traits), external trait sealing (downstream crates),
transitive effect-purity ("no I/O anywhere reachable"). Also **rejected — trait-surface freeze**
(`freeze_methods([...])`, "trait T's method set is closed"): it is **API-contract stability, not
architectural shape** (a stated non-goal — behavioral/contract governance), and a frozen method
*list* in the constitution is a hand-copy of the trait definition that drifts — the exact
declaration-integrity anti-pattern the project fights. The real intent ("keep the facade small") is
a 潛移/review concern, not a brittle enumerated reaction. Adopter-surfaced, declined with reason.

### 漏刻 (Lòukè) — the runtime dimension  · crate `louke`  · **BUILT (v0.1.0) — admitted layer complete**
Observation source: **runtime `TypeId` / object origin** at architectural seams. Sees what
static analysis structurally cannot — the concrete type behind a `dyn Trait`. **Built:** the
**origin-assertion** capability — `RuntimeBoundary::at("seam").only_origins([...])` declared
and installed at startup; a type opts into an *observed* origin via `register_origin!(Type)`
(captures `module_path!()`); a probe `assert_boundary!("seam", obj)` reads the live object's
concrete origin (via a `louke::Tracked` supertrait) and reacts **fail-closed** (unknown
origin reacts). Default reaction emits a `Violation` event; `panic` is opt-in. Plus the **CI
face** `audit_probe_coverage` — a source scan that every declared seam has a probe (closing
the "declared but never enforced" essential gap). 漏刻 reuses 璇璣's `Violation` as the
*measure* (xuanji gained `BoundaryKind::Runtime`), projecting it as a runtime **event** (the
CI dimensions project the same measure as an exit code). Hot path std-only + fold-hasher,
write-once registry, no lock; `serde_json` cold-path only via 璇璣. Identity resolved in the
PROJECT.md decision "漏刻 is identity-coherent"; overhead cleared by a spike (~4 ns).

- **Composed into `tianheng check`** (done): the shell now runs `audit_probe_coverage`
  alongside the static/semantic gates against the unified `Constitution` — `run(&constitution,
  args)` projects all 三儀 into one exit code. `audit_probe_coverage` takes the **declared
  `RuntimeBoundary` objects** (authoritative) and scans each member's `cargo metadata` source
  root for probes; the shell now depends on `louke` (self-governance allowlist amended). The
  prod face stays a function the adopter wires into their binary
  (`louke::install(constitution().runtime_boundaries()…)`).

Deferred / forward:
- **Rejected** (an explicit non-goal): runtime capability/effect drift ("no I/O reachable")
  — a runtime policy engine. The registry holds static label allowlists only, never predicates.
- **Audit-scanner coverage-fidelity residuals (documented, not silent) — from the v0.1.4 review.**
  The CI probe scanner (`audit_probe_coverage` / `scan_source`) over-counts coverage in three cases,
  each a false negative of the audit (a "covered" seam that never enforces at runtime), deferred
  from `runtime-audit-always-run` (which shipped only the shell always-run fix):
  - **Probe inside a `macro_rules!` body counts as coverage — CLOSED (v0.1.5).** `scan_source` now
    skips a foreign macro body (a louke-local `foreign_macro_body_end`, keyed on the `!` after the
    probe marker is consumed; the name-skip gated to `macro_rules` so a keyword-glued `if!cond {…}`
    is not mistaken for a macro), so a probe in a never-invoked macro body no longer counts. Shipped
    as an OpenSpec change modifying `runtime-origin-assertion`; 三儀 ⊥ 三儀 kept (no `strip_macro_bodies`
    import).
  - **Probe in an unreachable/orphan `.rs` file counts.** louke scans the src subtree lexically with
    no module-graph reachability; a probe in a dead orphan file is counted. Closing it needs a
    reachability walk louke does not have and cannot borrow from 圭表 (三儀 ⊥ 三儀), and louke's prod
    weight forbids a syn/heavy walker — a design-bearing follow-up (a louke-local reachability walk,
    or the shell passing reachable files).
  - **`member_src_dirs` silently skips a lib/bin-less member.** `crate_root_file` returns `None` for
    a member with no lib/bin target (proc-macro/test-only), genuinely out of the audit corpus; a
    lib/bin target always carries a `src_path`, so the "resolvable-but-absent" case is unreachable in
    practice. A stated bound; if ever closed, distinguish no-target (skip) from target-without-src_path
    (constitution error) and narrow the `runtime-origin-assertion` spec's blanket "unresolvable =
    constitution error" wording to match.
- **Un-auditable-probe finding identity is file-granular (baseline re-mask hardening, not a coverage
  FN).** *From the v0.1.5 hidden-bug sweep.* The un-auditable-probe `Violation` is keyed by file
  (one reaction per file), so if an un-auditable probe is baselined and later removed, the stale
  baseline entry can re-mask a *new*, distinct un-auditable probe added to the same file until the
  baseline is pruned. Accepted debt today (while any un-auditable probe remains in the file the
  accepted fact stays true), and the general baseline-staleness surfacing (`Baseline::stale`) covers
  it; if ever tightened, qualify the finding by a per-probe locator (byte offset / occurrence
  index). Low; not the forbidden FN. Relates to the finding-identity-must-be-injective principle.

## Deferred — not a reaction phase (the 三司: governance & observability layer)

These are **not new drift types**; they wrap the reaction (how it is surfaced, recorded,
amended). Most are already built in v0.1.0 or are convention by design — listed so the map
survives across sessions.

- **垂象 (Chuíxiàng) — the reaction surface.** *Built:* text report (v0.1.1: leads with the
  `reason`, surfaces the offending file, groups violations by boundary), exit codes `0/1/2`,
  `--format json`, and **`--format sarif`** (v0.1.1: SARIF 2.1.0, the vendor-neutral CI surface
  GitHub code-scanning and other tools inline onto a PR diff). *Built (v0.1.4):* a **single-module
  semantic violation now names its governed module's source file** (signature-coupling exposure,
  dyn-trait, impl-trait, async-exposure, visibility), surfaced from 渾儀's existing module
  traversal via `resolve_module_file` at the reaction layer and projected in JSON + SARIF
  (`physicalLocation`, no `region`); `file` stays out of baseline identity. **Then completed to
  7/7 (v0.1.4):** the two whole-crate scans (trait-impl-locality, forbidden-marker) now name their
  file too — the offending element's module (the `impl` site's; the defining type's for a
  `#[derive]`), the hearts surfacing a per-finding module and the reaction layer resolving it with
  the same `resolve_module_file` (memoized, `.ok()`-degrades-to-null, dedup-by-finding to hold the
  count invariant). **Every semantic violation now names its source file.** *Convention,
  not a tool format:* a
  GitHub-specific `::error::` output is deliberately **excluded** — it would couple the tool to one
  CI vendor; turning the neutral output into vendor annotations is a harness/CI-step recipe (a
  `jq` one-liner over `--format json`, or uploading the SARIF — see `README.md`). *Deferred (same
  observation, not new drift):* an **editor/LSP shift-left** so an illegal `use` is red-lined as
  typed (a large integration; the LSP server could be its own crate, born when built — a far
  horizon, but *additive*: a new crate is a **patch** by SemVer honesty, not a minor by virtue of
  its size). *Refinements (declaration/reaction, additive):* (1)
  **structured `because` — a machine-stable `anchor` distinct from the prose sentence** — **BUILT
  (v0.1.5):** `.with_anchor("ADR-014")` on every boundary DSL; `Violation.anchor` surfaced in the JSON
  (always, like `file`), the SARIF property bag and text report (Some-only), and the `list` projection
  (Some-only, byte-stable). The `because` sentence stays for humans; the anchor is the durable pointer,
  closing the prose-drift this project kept hitting. (2) **a violation repair-direction polarity** —
  **BUILT (v0.1.5)** as `Polarity { DenyBreach, AllowlistGap }` on `Violation` (`Option`, **derived
  from the rule type**): an allowlist boundary (`restrict_*_to`/`only_*`, and `deny_external` by repair
  direction) whose fail-closed reaction on an undeclared-but-legitimate member has the *opposite*
  repair (declare the intent) is `AllowlistGap`; a deny-of-a-specific-target is `DenyBreach`. Machine-
  readable, distinct from `BoundaryKind` (the *dimension*). The runtime CI-audit coverage violations
  carry `None` — a declaration/probe-consistency axis, not the drift axis; a future `violation_class`
  is a separate field, never more `Polarity` variants. Both additive/patch.
  (*Already shipped, not forward:* the machine-readable **constitution** projection —
  `list --format json` / `constitution_json` — an adopter missed it; a docs pointer is the only gap.)
- **實錄 (Shílù) — baseline & history.** *Built:* the snapshot gate (record accepted
  violations, fail only on *new* drift). *Deferred:* a **debt-ratchet**
  (`--require-baseline-reduction`, only-fix-never-add) — **in tension** with "baseline is a
  snapshot, not policy" and "not a governance platform". A bounded opt-in flag may fit; a
  debt-scheduling system does not. Resolve the tension before building. *Metadata — BUILT (v0.1.5):*
  baseline entries carry **structured metadata** `owner` / `tracker` (external issue) via a
  `BaselineEntry`, so a grandfather list points debt at a tracker instead of accreting a silent,
  never-shrinking per-instance exemption table. Additive/patch — the match identity `(target, rule,
  finding)` and the required parse format are untouched (Some-only fields, `version` 1); `--write-baseline`
  is a metadata-preserving merge by identity (warns, never silently wipes). The once-listed `anchor`
  field was **dropped as redundant** with the boundary→violation anchor. *Rejected — time-based auto-decay /
  auto-escalation* (`expires("<date>")` producing a reaction; a `warn_until("<date>")` Warn→Enforce
  ramp): it makes the reaction depend on **wall-clock**, breaking the invariant that a reaction is a
  pure function of (declaration, observed code) — the determinism red line that keeps reactions
  reproducible. Gradual adoption is already served deterministically by `Baseline` (gate only new
  drift) + `warn` severity + a PR-gated `.warn()`→`.enforce()` flip when ready (auditable, unlike a
  silent date rollover). Reconsider only if the time-axis earns its own explicit design decision.
- **校讎 (Jiàochóu) — the amendment flow.** Deliberately **not a tool feature**: the tool
  cannot tell shape-drift from policy-drift (not an observable fact), and must not own PR /
  merge orchestration. Realized as **harness convention** — `.github/CODEOWNERS` + steward
  review + the OpenSpec lifecycle + `AGENTS.md`. Already in place; nothing to build.
- **Declaration integrity — self-observe the declaration, not only governed code.** PROJECT.md is
  the canonical decision record for this pattern: migrate only structural property-assertions
  about the declaration into reactions; leave rationale prose alone. **Built internally (v0.1.4):**
  the 三儀 ⊥ 三儀 clause is self-observed by
  `dimension_boundaries_declare_the_mutual_independence_law`; the old hand-maintained
  boundary-number index is gone. **Forward, born when built (no API before a second
  consumer):** (a) a small **constitution-assertion helper** so structural assertions are not
  re-hand-rolled per repo; (b) the adopter-facing **潛移 generator** (see the 潛移 section). Stated
  bound: a `because`-text `contains` predicate is weaker than a structural fact (a reworded clause
  slips it). Adopter-surfaced by worklane.

## 潛移 (Qiányí) — the gravity axis (new in v0.1.1)

Not a 儀 (instrument) and not a 司 (office): a complementary mode of compliance for an
autoregressive agent — make the declared law **imitable and in its context**, so continuations
stay in-shape by default; the reaction stays the non-bypassable backstop (see `PROJECT.md`, 潛移).
*Built (v0.1.1):* the thesis and its drift-law bound (PROJECT.md); the **self-law projection**
(`AGENTS.self-law.md`, generated from `self_governance.rs`, staleness-gated) so an agent working on
this repo reads the *enforced* law, not the demo; **reason-foregrounding** in the law projection
(`list --format markdown` leads each boundary with its reason) and in the reaction's text report;
the **reason-writing convention** (AGENTS.md). *Forward (phase-2):* an **adopter-facing
潛移 face** — any project generates its own agent-context from its constitution. The library
primitive (`constitution_markdown`) and a README recipe shipped in v0.1.1; a **byte-checked
staleness-gate recipe** shipped in v0.1.4 (a `cargo test` that regenerates the projection and
byte-compares it to the committed file — the adopter-facing form of Tianheng's own
`self_law_projection_is_fresh`, so an adopter's hand-maintained agent-context prose becomes a
non-bypassable projection; adopter-surfaced by worklane). A full generator / a `list-self`-style
CLI stays deferred (adopter-workflow product weight, and a `list-self` CLI would tangle the
demo-vs-self-law story) — the primitive plus the gate already close the drift. **Pilot now offered
(v0.1.5 dogfooding input):** worklane volunteers as the **first generator pilot** — it already has
the generation need, the freshness-gate discipline, and can feed back the CLI shape and staleness
semantics a real adopter wants; this is the live *second consumer* "born when built" was waiting
for, so the generator becomes a **0.1.5 Tier-2 candidate** (see Version horizons). Its north star —
the **adopter-facing adoption guide** the generator would produce — is carved out as its own future
item below (*潛移 applied to adoption*), not buried here. Held to the same
bound: only what reacts or projects enters context; no
unobservable wish becomes law (prose prescription is the rejected open loop). *Version by SemVer
honesty, not by phase:* an additive generator/CLI is a **patch**; a 0.2.0 is earned only by a
breaking change — e.g. the deliberate pre-1.0 refinement of `guibiao`'s widened public surface
(see PROJECT.md, Decisions) — never by bundling a milestone.

**潛移 applied to adoption — the adoption-gravity deliverable (future item, born when built).**
Tianheng exists, almost by definition, to **minimize drift-prone prose code-docs** — to turn a
structural claim written in a comment into a reaction. It follows that **non-adoption is not a
missed metric but mission failure**: an unadopted tool reduces no one's prose. So actively lowering
adoption friction is *on-charter*, not growth-hacking — **provided the method stays 潛移, never
instruction**: adoption is *pulled* by an imitable, in-context on-ramp, never *pushed* by a
"you-should-adopt-this" call to action (that would be the very instruction the project rejects). The
adoption funnel's weak seam is the *top* — whether it even occurs to an agent to govern architecture,
and the first-boundary decision — not the API (the on-ramp is already one line,
`forbid_all_workspace_dependencies()`). Two levers, different weight:

- **Cheap, patch-now: sharpen the README on-ramp.** Make the *first* boundary a one-line imitable
  Phase-0 pattern (lock one seam, Enforce, pipe `--format sarif` into CI) that an agent scanning the
  crate copies by reflex — 潛移 at the doc level, near-zero cost. Likely the highest adoption leverage
  per unit effort; can ride 0.1.5 on its own.
- **The full deliverable: a projectable two-track adoption guide** (produced by the 潛移 generator,
  worklane pilot). **Brownfield** (invariants already earned, prose exists → encode a mechanical
  subset, prose → code, straight to Enforce) vs. **greenfield** (assumptions, no prose →
  code-constitution → projection → prose grows *after*, Warn → soak → Enforce) — a playbook that
  **falls out of the capability set**, each capability carrying its own "when to reach for it /
  starting severity / truth-direction". **Self-consistency is mandatory, not optional:** because
  tianheng fights drift-prone prose, a *hand-written* prose adoption guide would *be* the thing it
  fights — so the guide must itself be a **projection (code → doc)**, eating its own dogfood; only
  the **spine** is hand-written, the irreducible minimum the projection cannot emit. The spine is
  three judgments *between* capabilities: *affordance ≠ reason* ("when not to adopt" is no
  capability's property), *ROI / lock one seam first* (which seam bears load is a human architectural
  read), and the *determinism red line* (a reaction is a pure function of `(declaration, observed
  code)` — no time-decay / effect-purity / trait-freeze). Those three are already recorded under
  "Explicitly not on the roadmap" and the rejected time-decay above — the guide *points at* them,
  never re-decides them.

*Version:* the README on-ramp and an additive generator / guide are **patches**; nothing in this
item earns a minor. The determinism red line and "affordance ≠ reason" are the standing bounds.

## Explicitly not on the roadmap

Active code-shaping / generation; a prescriptive framework you build inside; a **lint**
(built-in opinion rather than declared intent); a **universal graph API** (whole-graph
analysis rather than declared per-target boundaries); a **runtime policy engine**; a
**supply-chain policy engine** (resolved / whole-graph advisories, licenses, bans, source
allowlists — cargo-deny's lane; Tianheng governs the *declared, per-target* layer instead —
see the declined capability B above). Each dimension keeps its own observation source;
nothing is named before its reaction exists.

Also **not a cleanup target**: consolidating the declaration DSL's repetitive builders. The
per-capability `*Boundary` / `*Draft` chains read repetitively, but they are a designed-to-be-
**imitated** surface (潛移) *and* their anchoring genuinely diverges (crate / module / trait /
subtree, with different payloads); a macro would trade imitability and legibility for LOC and become
a per-capability mini-language. The repetition is intentional, not debt — leave it explicit.
