# Backlog

Forward-looking work, deliberately deferred. Promote an item to an OpenSpec change when
you pick it up. Every future reaction obeys the drift law:

> **No drift type without an observation source. No target type or name without a
> reaction.**

Nothing here is "designed" yet — reaction *phases* with their observation sources named,
not APIs. A new observation dimension is **a crate, born when it is built** (never a
pre-created empty stub); the heavy dependency it needs is quarantined to that crate so the
`guibiao` core stays `serde_json`-only.

## Reaction phases — the 三儀 (observation dimensions)

Ordered by readiness. All three instruments ship in v0.1.0 (圭表 static, 渾儀 semantic, 漏刻
runtime); the entire admitted 三儀 layer is now built. What remains below is the rejected set
per dimension and the 三司 governance/observability layer.

### 圭表 (Guībiǎo) — the static dimension  · crate `guibiao`  · **BUILT — proven core (v0.1.0, from modou); growing by depth (v0.1.2 crate-source-boundary)**
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

Forward depths (born when built, same `cargo metadata` source):
- **Resolved dependency-source / build-provenance (capability B)** — *named, deferred.* "What my
  build **actually** pulls from, after `[patch]`/`[source] replace-with`", read from the
  **resolved** graph (`cargo metadata` **with** deps, the lockfile + patch applied). It catches a
  `[patch]`/`replace-with` redirect to a git source that the declared-layer crate-source-boundary
  is deliberately blind to, and in turn misses an *optional-off* git dep (not in the build) —
  the mirror blind spot. A genuinely distinct capability, not a completion of the declared one
  (see `PROJECT.md`); born when built, adding a resolved-layer read (heavier, lockfile-dependent),
  never retrofitted onto the hermetic declared rule.

### 渾儀 (Húnyí) — the semantic dimension  · crate `hunyi`  · **BUILT — originally-conceived layer (v0.1.0); growing by depth (v0.1.2 dyn-trait)**
Observation source: the **AST** (`syn`). Sees what the `圭表` `use`-scan cannot — semantics
in the syntax tree: `pub` signatures, `impl Trait for Type`, attributes/derives, visibility.
The observation-source fork is **resolved**: `syn` was chosen (stable; its syntactic partial
coverage — glob / cross-crate re-export / macro / inference blindness, while local `pub use`
chains, incl. multi-hop and aliased, *are* followed — is *stated*, never silently passed),
over `cargo rustdoc --output-format json` (nightly + unstable format).

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

Forward depths (born when built, same `syn` source):
- **`must_not_expose_existential` (unifier)** — a possible future capability folding impl-trait
  (written `impl Future`/RPIT) and async-exposure (implicit `impl Future`) under one "no
  existential at this seam" rule. Deferred: the two syntactic signals stay distinct rules until a
  unification earns its own admission (it must not blur the two findings' identities). Not built.

Explicitly **rejected** (essential gap — would be a false-negative engine, see `PROJECT.md`):
`Send`/`Sync` constraints (inferred auto-traits), external trait sealing (downstream crates),
transitive effect-purity ("no I/O anywhere reachable").

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

## Deferred — not a reaction phase (the 三司: governance & observability layer)

These are **not new drift types**; they wrap the reaction (how it is surfaced, recorded,
amended). Most are already built in v0.1.0 or are convention by design — listed so the map
survives across sessions.

- **垂象 (Chuíxiàng) — the reaction surface.** *Built:* text report (v0.1.1: leads with the
  `reason`, surfaces the offending file, groups violations by boundary), exit codes `0/1/2`,
  `--format json`, and **`--format sarif`** (v0.1.1: SARIF 2.1.0, the vendor-neutral CI surface
  GitHub code-scanning and other tools inline onto a PR diff). *Convention, not a tool format:* a
  GitHub-specific `::error::` output is deliberately **excluded** — it would couple the tool to one
  CI vendor; turning the neutral output into vendor annotations is a harness/CI-step recipe (a
  `jq` one-liner over `--format json`, or uploading the SARIF — see `README.md`). *Deferred (same
  observation, not new drift):* an **editor/LSP shift-left** so an illegal `use` is red-lined as
  typed (a large integration; the LSP server could be its own crate, born when built → 0.2.0+).
- **實錄 (Shílù) — baseline & history.** *Built:* the snapshot gate (record accepted
  violations, fail only on *new* drift). *Deferred:* a **debt-ratchet**
  (`--require-baseline-reduction`, only-fix-never-add) — **in tension** with "baseline is a
  snapshot, not policy" and "not a governance platform". A bounded opt-in flag may fit; a
  debt-scheduling system does not. Resolve the tension before building.
- **校讎 (Jiàochóu) — the amendment flow.** Deliberately **not a tool feature**: the tool
  cannot tell shape-drift from policy-drift (not an observable fact), and must not own PR /
  merge orchestration. Realized as **harness convention** — `.github/CODEOWNERS` + steward
  review + the OpenSpec lifecycle + `AGENTS.md`. Already in place; nothing to build.

## 潛移 (Qiányí) — the gravity axis (new in v0.1.1)

Not a 儀 (instrument) and not a 司 (office): a complementary mode of compliance for an
autoregressive agent — make the declared law **imitable and in its context**, so continuations
stay in-shape by default; the reaction stays the non-bypassable backstop (see `PROJECT.md`, 潛移).
*Built (v0.1.1):* the thesis and its drift-law bound (PROJECT.md); the **self-law projection**
(`AGENTS.self-law.md`, generated from `self_governance.rs`, staleness-gated) so an agent working on
this repo reads the *enforced* law, not the demo; **reason-foregrounding** in the law projection
(`list --format markdown` leads each boundary with its reason) and in the reaction's text report;
the **reason-writing convention** (AGENTS.md). *Forward (phase-2, → 0.2.0):* an **adopter-facing
潛移 face** — any project generates its own agent-context from its constitution. The library
primitive (`constitution_markdown`) and a README recipe shipped in v0.1.1; a full generator /
workflow is deferred (adopter-workflow product weight, and a `list-self`-style CLI would tangle the
demo-vs-self-law story). Held to the same bound: only what reacts or projects enters context; no
unobservable wish becomes law (prose prescription is the rejected open loop).

## Explicitly not on the roadmap

Active code-shaping / generation; a prescriptive framework you build inside; a **lint**
(built-in opinion rather than declared intent); a **universal graph API** (whole-graph
analysis rather than declared per-target boundaries); a **runtime policy engine**. Each
dimension keeps its own observation source; nothing is named before its reaction exists.
