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
  current, a patch unless bundled with that breaking refinement.

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

**Forward depth — potential, not active (adopter-surfaced):**
- **Closed inbound allowlist — `must_only_be_imported_by`**: *not built.* The **closed dual** of
  the existing `must_not_be_imported_by` (forbid-one): "only `crate::facade` may import
  `crate::internal`" — every other importer reacts. Observable on the same source (the `use` scan
  across the workspace), declarative-not-lint (two projects may sanely allow different importers),
  so it obeys the drift law. **Surfaced by an adopter (worklane)**, whose own analysis judged the
  threshold *above* feasibility — a facade-protection rule it does not yet find worth making
  prescriptive — so it is recorded here as a **potential, non-active** consumer, not promoted. An
  open design question rides with it: a new boundary type vs. a mode of the existing import
  boundary; resolve it when a *live* adopter need appears (born when built, never a name before
  its reaction).
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
- **Resolved dependency-source / build-provenance (would-be capability B)** — *declined.* "What my
  build **actually** pulls from, after `[patch]`/`[source] replace-with`", read from the
  **resolved** graph (`cargo metadata` **with** deps, the lockfile + patch applied). It would catch
  a `[patch]`/`replace-with` redirect to a git source that the declared-layer crate-source-boundary
  is deliberately blind to (and in turn miss an *optional-off* git dep — the mirror blind spot). But
  that resolved, **whole-graph** source-provenance concern is **cargo-deny's lane**, not Tianheng's:
  `deny.toml [sources]` (run in the `supply-chain` CI job) already denies unknown git/registry
  sources on the resolved graph — so a `[patch]`→git redirect surfaces there — and a whole-graph
  view fits build-provenance better than Tianheng's per-target model. So B is **declined, not
  deferred**: Tianheng keeps only the *declared*, per-target crate-source-boundary (A), the
  hermetic manifest-hygiene reaction cargo-deny does not provide. See the declared-vs-resolved
  division of labor in `PROJECT.md`.

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
- **Crate-root `extern crate X as Y;` scoping — `crate::<alias>` FN + submodule-`mod`-shadow FP.**
  A crate-root rename `Y` binds crate-wide, so bare `Y::…` resolves everywhere (handled). But
  `crate::Y::…` (the crate-relative spelling) is **not** rewritten to the real crate — a false
  negative; and the bare-`Y` rewrite is not suppressed when a submodule declares its own `mod Y`,
  which rustc lets shadow it — a false positive. The proposal-stage review of `resolver-rustc-fidelity`
  proved the originally-sketched "crate-root-only rewrite" model would have *introduced* a new FN
  (bare `Y` in a submodule with no local shadow currently reacts and must keep reacting), so the
  rustc-correct model (keep the crate-wide bare rewrite; suppress under a local type-namespace
  shadow; add a `crate::<alias>` rewrite) is a **design-bearing follow-up**, not rushed onto the
  patch line. Local-observable; born when built.
- **Re-export head shadow FP.** A `pub use serde::X` head in a module that also declares a local
  `mod serde` is misattributed to the dependency (rustc shadows it — E0432). The re-export branch
  uses the raw extern set with a comment that over-claims "a bare `pub use` head is always extern";
  narrowing it to honour the child-module shadow (`externs − child_module_names`) removes the FP
  without a new FN. Narrow, FP-not-FN; deferred with the rename family above.

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
  - **Probe inside a `macro_rules!` body counts as coverage.** `scan_source` strips comments and
    strings but not macro bodies, so a probe in a never-invoked macro body (dead code the compiler
    never emits) is counted. The static scanner's `strip_macro_bodies` is the model — an easy lexical
    addition, but it extends the CI-face requirement's stated lexical scope, so it earns its own
    spec-modifying change (modifies `runtime-origin-assertion`). Additive/patch.
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
  its size). *Forward (declaration/reaction refinements, additive, adopter-surfaced):* (1)
  **structured `because` — a machine-stable `anchor` distinct from the prose sentence**
  (`.anchor("ADR-014")`, surfaced as a field in the JSON/SARIF projection): the free-text `because`
  accretes ephemeral refs (PR numbers, handles, "recently") that rot faster than the invariant they
  justify — the exact prose-drift this project keeps hitting; the anchor is the durable pointer, the
  sentence stays for humans. (2) **a violation `kind` distinguishing `DenyBreach` from
  `AllowlistGap`** — for an allowlist boundary (`restrict_*_to`/`only_in`/`allow_origins`) a
  fail-closed reaction on an undeclared-but-legitimate member has the *opposite* repair (declare the
  intent) from a deny breach (fix the code); the polarity is known at the boundary constructor, so
  tagging `Violation` (additive, like `file`) makes the repair direction machine-readable — and
  distinct from `BoundaryKind`, which names the *dimension*, not the polarity. Both additive/patch.
  (*Already shipped, not forward:* the machine-readable **constitution** projection —
  `list --format json` / `constitution_json` — an adopter missed it; a docs pointer is the only gap.)
- **實錄 (Shílù) — baseline & history.** *Built:* the snapshot gate (record accepted
  violations, fail only on *new* drift). *Deferred:* a **debt-ratchet**
  (`--require-baseline-reduction`, only-fix-never-add) — **in tension** with "baseline is a
  snapshot, not policy" and "not a governance platform". A bounded opt-in flag may fit; a
  debt-scheduling system does not. Resolve the tension before building. *Forward (additive,
  adopter-surfaced):* baseline entries carrying **structured metadata** — `owner` / `tracker`
  (external issue) / `anchor` — so a grandfather list points debt at a tracker instead of accreting
  a silent, never-shrinking per-instance exemption table. *Rejected — time-based auto-decay /
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
- **Declaration integrity — self-observe the declaration, not only governed code.** A reaction
  whose observation source is the **declaration and its artifacts** (the `Constitution`, its
  `constitution_markdown` projections, and prose that indexes/asserts a structural property of the
  law), not user code — the shape unifying `self_law_projection_is_fresh`, `audit_probe_coverage`,
  and an adopter's reason-drift test (see PROJECT.md, "Declaration integrity"). **Built internally
  (v0.1.4):** the 三儀 ⊥ 三儀 clause is now self-observed by
  `dimension_boundaries_declare_the_mutual_independence_law`, and the hand-maintained "(boundaries
  2, 3, 6)" index it retired is deleted (prose index → reaction). *Not a new 儀* (observes no
  governed code); drift-law-compliant; bounded against a lint by migrating only property-assertions,
  never decision-rationale. **Forward, born when built (no API before a second consumer):** (a) a
  small **constitution-assertion helper** (an iterator over `boundaries()` with kind/reason, usable
  in a `#[test]`) so form-(a) structural assertions are not re-hand-rolled per repo — abstract it
  once tianheng-internal use meets a second (adopter) call site; (b) the adopter-facing **潛移
  generator** (see the 潛移 section) — the v0.1.4 byte-check *recipe* already unlocks retiring a
  hand-maintained agent-context face, so the generator is ergonomic, not the enabler. Stated bound:
  a `because`-text `contains` predicate is weaker than a structural fact (a reworded clause slips
  it). Adopter-surfaced by worklane.

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
demo-vs-self-law story) — the primitive plus the gate already close the drift. Held to the same
bound: only what reacts or projects enters context; no
unobservable wish becomes law (prose prescription is the rejected open loop). *Version by SemVer
honesty, not by phase:* an additive generator/CLI is a **patch**; a 0.2.0 is earned only by a
breaking change — e.g. the deliberate pre-1.0 refinement of `guibiao`'s widened public surface
(see PROJECT.md, Decisions) — never by bundling a milestone.

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
