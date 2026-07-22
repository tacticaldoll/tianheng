# Backlog

Forward-looking work, deliberately deferred. Promote an item to an OpenSpec change when
you pick it up. Every future reaction obeys the drift law:

> **No drift type without an observation source. No target type or name without a
> reaction.**

Nothing here is "designed" yet — reaction *phases* with their observation sources named,
not APIs. A new observation dimension is **a crate, born when it is built** (never a
pre-created empty stub); the heavy dependency it needs is quarantined to that crate so the
`guibiao` core stays `serde_json`-only.

## Backlog governance — evidence before promotion

The live backlog is a decision surface, not a promise that every recorded idea will ship. Before a
live item is promoted, it must name: **class**, **observed pressure**, **observation source**,
**current reaction or bound**, **risk**, **promotion trigger**, **version class**, and **authority**
(the spec, project decision, or code/test evidence that owns the claim). Classify it as:

- **READY-PATCH** — supported pressure with a concrete source, and the correction preserves the
  published API and current baseline/report identity wire. It may enter a `0.2.x` change or focused
  non-OpenSpec maintenance PR according to the authority it changes.
- **DESIGN-BREAKING** — a supported problem whose honest solution needs a public or wire migration.
  It waits for its named forcing trigger and an OpenSpec proposal; being listed does not itself
  promise `0.3.0`.
- **WATCH** — plausible pressure without enough adopter, second-consumer, or correctness evidence.
  Preserve the trigger, not a premature design.
- **ACCEPTED DEBT** — a known, bounded risk whose current reaction or documented coverage bound is
  intentionally sufficient. Reopen only when the recorded bound is defeated.
- **DECLINED** — a considered direction rejected for a recorded reason. Reopen only with evidence
  that invalidates that reason.
- **BUILT / HISTORY** — shipped context retained only where it explains a live contract or trigger;
  requirements live in `openspec/specs/*` and settled rationale in `PROJECT.md`.

Classification and promotion remain human-reviewed judgment. Add an automated reaction only after
an observable, repeated drift demonstrates what a machine can decide without pretending judgment is
structural enforcement.

## Live decision index — 0.2.x truth repair and the next breaking window

This index makes the current work discoverable without duplicating the detailed evidence below.

### DESIGN-BREAKING

- **Identity v3 migration bundle.** **Pressure/source:** version-2 identity still couples human
  `rule` presentation to `ViolationId` and SARIF `ruleId`; SARIF fingerprints remain presentation
  bearing; unsafe facts compress form/trait/owner/name into one label; async facts have not decided
  whether identity is the seam or exact signature. **Current reaction:** exhaustive v2 schema
  catalogs and v1/v2 baseline compatibility freeze the existing wire throughout 0.2.x. **Risk:** a
  piecemeal fix would churn baselines or create two competing identities. **Trigger:** a verified
  adopter migration need or correctness failure that cannot be solved additively. **Version:** one
  coordinated `0.3.0` OpenSpec migration: stable rule key, SARIF fingerprint v2, unsafe decomposition,
  async decision, and baseline v3 compatibility. **Authority:** the post-0.2 identity-pressure
  section below and the structured-identity decisions in `PROJECT.md`.

### WATCH / ACCEPTED / DECLINED / BUILT

- **WATCH:** judgment-neutral lexer/token extraction is now plausible, but the conformance work must
  first prove a cross-scanner false negative or a third scanner before the `PROJECT.md` trigger is
  treated as fired; `cfg_attr(path)` observe-both semantics, a reusable testing harness, qianyi
  generator, LSP/editor integration, and a debt ratchet remain gated by their detailed triggers below.
- **ACCEPTED DEBT:** multi-target conventional-path conflation, macro/configuration coverage bounds,
  and file-granular un-auditable-probe identity remain bounded as documented below.
- **DECLINED:** keep the existing explicitly rejected directions under their recorded rationale;
  this index does not reopen them.
- **BUILT / HISTORY:** shipped capability ledgers below are historical context, not live work. New
  work starts from the live classes above, then moves through OpenSpec where capability behavior
  changes.

## Version horizons — what 0.2.x carries vs what earns the next breaking window

The version follows SemVer honesty (`AGENTS.md`), not milestone size: **non-breaking →
patch, breaking → minor**, and never a vanity minor bump. Version `0.2.0` shipped the first
deliberate breaking window; the current line is **0.2.x**:

- **0.2.x (patch)** — additive depth on an existing observation source, false-negative closures
  that preserve the published API and version-2 identity wire, packaging / CI / license hygiene,
  compatibility reactions, and governance-doc corrections. Size alone never earns a minor.
- **Next breaking window (`0.3.0` only if earned)** — candidates that must reshape the published
  reaction-inspection or baseline wire wait here: unsafe-site fact decomposition, a decision on
  whether an async seam's rendered signature belongs to identity, and separation of stable rule
  identity from human rule presentation. These candidates do not promise a `0.3.0`; one is promoted
  only when a real adopter or correctness pressure justifies the break, and then the break earns the
  minor. The adopter-written builder (`Constitution` / boundary DSL / `run`) remains the guarded
  drop-in surface unless a separately demonstrated forcing function says otherwise.

**The `0.2.0` window — SHIPPED.** Structured violation identity moved live findings to
dimension-owned `FindingKey`s and version-2 baselines; the widened `guibiao` projection / baseline
surface was kept and shaped because modou consumes it standalone. The composed adopter surface was
compile-reacted against pacta's usage. Those decisions and their rationale now live in `PROJECT.md`;
the post-0.2 pressure points below are follow-ups, not unfinished 0.2.0 scope.

### The crate family as products — identity now, product weight on reaction

The six published crates carry deliberate roles, not just a workspace split:

- **三儀 = public products** — 圭表 (static import / dependency boundaries, syn-free), 渾儀 (public-API
  exposure), 漏刻 (runtime origin governance). Three **orthogonal** instruments (different observation
  sources, different audiences), not redundant crates — the strongest answer to any "why so many
  crates" read.
- **璇璣 / 星表 = the public substrate** the instruments stand on (public because the instruments
  depend on them, not products in their own right).
- **天衡 = the composer** — batteries-included, the funnel target: adopt one 儀 as an on-ramp,
  graduate to the composed constitution. Single 儀 → suite is the adoption funnel, not a dilution.

Productization is **demand-driven, in Tianheng's own form** — the drift law applied to go-to-market:
*no name without a reaction → no commitment without a reaction.* Identity can be declared now (it is
reversible narrative); irreversible / breaking / high-maintenance weight waits for a real reaction.

- **Establish now (reversible):** the product identities above; family positioning in each crate's
  README / docs.rs; that most adopters want `tianheng`. Always co-stated with the honest tier
  (**experimental / pre-1.0**) — a claimed-but-unsupported product identity is worse than none.
- **Defer to a reaction:** per-儀 standalone CLIs, docs / cookbooks, per-crate 1.0 / long-term
  stability promises, and the standalone 漏刻 product story (a legitimate category, but the
  least-proven — its standalone demand is the most speculative of the three).

**Stability posture: 0.1.x late-stage pre-stability.** Not immaturity — concept and function are
saturated (三儀 all born, a complete world-view); the 0.1.x line is the *honest pre-1.0 window* that
keeps API lock-in right until real adoption pressure says which public faces become long-term
contracts. A category-creating project cannot pull demand for a category nobody knows exists, so the
sequence is **push then pull**: push the honestly-labelled (experimental) narrative to bootstrap
awareness; let demand deepen it.

**Exit trigger from the 0.1.x hold → 0.2.0 deliberate definition** (any one):

- a first serious external adopter needing a compatibility promise;
- a 儀 actually adopted standalone, or an API that actually hurts in use;
- API convergence (no churn across several patch releases).

### Public-contract legibility & convergence (upstream-review-surfaced)

An external reviewer reading only the **published 0.1.6 crates** (no `PROJECT.md` / `self_governance.rs`)
proposed public-contract refinements. Triaged against the enforced architecture, **most are already
true and self-governed — the gap the reviewer hit is legibility, not architecture**: the contract
reads correctly from outside but is not stated in adopter-facing docs. So the payoff here is a
docs/contract pass (0.1.x, non-breaking), one surface audit, and one considered decline — not new
architecture. The primary vehicle is the **examples** (track 1): these invariants are *demonstrated*
there as runnable proof (the "Contract demonstrations" bullet above), with written docs as the
complement — show, then tell.

- **Already enforced; make legible (doc, do not build).**
  - *Three-layer split — declaration (`Constitution`) ⊥ reaction (`check`, pure) ⊥ shell (`run`).*
    Already a **self-law** (functional-core ⊥ imperative-shell: `guibiao` must not depend on
    `tianheng`). Actionable: state the layering in adopter docs and name the **presentation ⊥
    verdict** invariant — `--format json`/`sarif` and the ANSI render change presentation only,
    never the outcome (already CI-reacted: the `reaction` job asserts a SARIF projection still exits
    1). This is also track 2's render guardrail.
  - *`xuanji` = the sole cross-crate wire contract.* Already so (the shared reaction model,
    `serde_json`-only, below every dimension, self-law-enforced). Actionable: elevate its JSON /
    `Baseline` schema to an **explicitly versioned, migration-disciplined** contract in docs — ties
    to the 0.2.0 structured-baseline item (findings as data).
  - *Violation identity ⊥ metadata.* **BUILT (0.2.0 line):** the baseline match key is
    `ViolationId = { target, rule, finding, finding_key }` (v2 structured identity; a v1 baseline
    matches on `{ target, rule, finding }` for migration); `file` is explicitly *not* identity (set
    via `with_file`, non-breaking, never affects matching); `BaselineEntry.owner/tracker` are
    metadata-only; the baseline carries no `anchor` (it rides the live `Violation`). This is the
    injective-identity principle realized. Actionable: surface it in the **adopter-facing README** as
    a stability contract, not only in rustdoc.
- **Adoption ladder → README (track-1 pitch), enriched.** The reviewer's warn → enforce ramp is
  real but one-dimensional; the actual ladder is **two axes**: severity (`warn` first → `enforce`
  gate) *and* baseline (grandfather existing violations → enforce new). An existing codebase adopts
  via baseline, a greenfield one via warn-first — document both as the on-ramp.
- **Prelude / stable-surface audit — BUILT (0.2.0 line).** The real composed adopter uses the
  wildcard prelude for both declaration and `Outcome` inspection, so trimming it into a builder-only
  menu would break the very reaction that opened the 0.2 window. The surface is now classified by
  purpose, not by weaker stability: declaration/execution and reaction inspection carry the same
  0.2.x promise. An external-view integration crate names every promised export and composes all
  three instruments without dimension imports, making an accidental relocation a compile failure.
  That probe found one genuine asymmetry: `ModuleBoundary::rule()` was public but its `ModuleRule`
  type was absent from the recommended wildcard path, so the existing type is now re-exported beside
  crate-side `Rule`. Hidden drafts and granular semantic checks remain outside the contract;
  `check_semantic` is documented honestly as the focused signature-coupling check, never the full
  semantic bundle.
- **`Rule` / `ModuleRule` model-surface narrowing — BUILT (0.2.0 line).** The live reaction was
  `.strict_external()` having to ship in 0.1.9 as a payload-identical hidden variant: enum-level
  `#[non_exhaustive]` protects new variants but not fields added to an existing struct variant, so
  the patch line could not grow a modifier without downstream E0063/E0027 breaks. Every
  data-carrying rule variant is now itself `#[non_exhaustive]`: consumers construct through the
  unchanged boundary DSL and can still inspect known fields with `..`. The missing read side was
  closed deliberately with `ModuleBoundary::rule()`, symmetric with `CrateBoundary::rule()`, rather
  than retaining a public-but-unobtainable `ModuleRule`. The strict twin folds back into one
  `ConfineInlineSymbolPath { strict_external, … }`; reaction, projection, and violation identity stay
  pinned by the existing tests. The break remains quarantined to direct variant construction and
  closed-field matches; pacta's builder and modou's widened guibiao surface compile unchanged.
- **`inline_symbol_findings` positional-arg growth — collapse into an `InlineScanRequest` param
  struct (internal, born-when-needed).** `.strict_external()` pushed the scanner entry to 8
  positional args (now under `#[allow(clippy::too_many_arguments)]`, having added `external` +
  `dependency_names`); the *next* dimension input should tip it into a named `InlineScanRequest`
  rather than a ninth positional. Behavior-preserving, internal-only (no model / adopter surface, so
  distinct from the variant-refactor debt above) — lands whenever the next input does, not a
  standalone task. Until then the 8 args are cohesive single-caller scan inputs (Gate-5-passed).
- **`governed_files` positional-arg growth — same class, same threshold (internal, born-when-needed).**
  0.2.2's `#[path]`-following work added `remapped` then `remap_shadowed` (the orphan-shadow fix
  below), pushing the reachability walk's own selector to 8 positional args (now under
  `#[allow(clippy::too_many_arguments)]`). Its *next* dimension input should tip it into a named
  param struct (mirroring the `InlineScanRequest` direction above) rather than a ninth positional —
  the two `reachable_modules`/`governed_files` outputs-as-inputs pairs (`reachable`+`inline_only`,
  `remapped`+`remap_shadowed`) are exactly the kind of cohesive group such a struct would carry
  together. Behavior-preserving, internal-only. Lands whenever the next input does, not standalone.
- **Considered decline — a mechanical "policy adapter" importing an existing rule source into a
  `Constitution`.** The *goal* (low-friction adoption, do not reinvent governance syntax) is
  legitimate and is served by the **cookbook / examples** (track 1) that translate common governance
  patterns into the `Constitution`.

- **Reserve (still future): a cross-dimension conformance reaction** for the logic that genuinely
  cannot share — the `syn`-vs-token-scan **resolvers** and byte-scan-vs-AST **module reachability**,
  which stay per-dimension because sharing them forces `syn` into the light core. A `cargo test` gate
  feeding identical inputs to each dimension's parallel logic and asserting agreement would catch
  drift *without* moving code. Deferred until a resolver twin-drift actually bites; 星表 does not
  address it.

### 渾儀 (Húnyí) — the semantic dimension  · crate `hunyi`  · **BUILT — originally-conceived layer, growing by depth**
Observation source: the **AST** (`syn`). Sees what the `圭表` `use`-scan cannot — semantics
in the syntax tree: `pub` signatures, `impl Trait for Type`, attributes/derives, visibility.
The observation-source fork is **resolved**: `syn` was chosen (stable; its syntactic partial
coverage — glob / cross-crate re-export / macro / inference blindness, while local `pub use`
chains, incl. multi-hop and aliased, *are* followed — is *stated*, never silently passed),
over `cargo rustdoc --output-format json` (nightly + unstable format).
Single-module and whole-crate resolution now **follow** an unconditional `#[path = "…"] mod x;` to
its author-chosen file (0.2.1; base = the containing file's own directory with each enclosing
inline-`mod` name accumulated onto it, matching rustc for mod-rs and non-mod-rs files), closing the
coverage false negative where a relocated module's items were dropped. A `cfg_attr`-wrapped `#[path]`
stays a cfg-blind bound; an absent unconditional target fails loud (exit 2).

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
  *named* trait's `dyn` rather than any: a `dyn` whose **principal trait** (its sole non-auto trait,
  whatever its bound position) canonicalizes into the forbidden set reacts, resolved through the shared 渾儀 resolver (exact-
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
- **`dyn` shape collector's inherent-impl assoc blind spot — CLOSED (v0.1.5).**
  `collect_item_dyn_exposures` now observes public `ImplItem::Const`/`Type` (its arms at
  `collect.rs:705`/`709`), so `impl Foo { pub type T = Box<dyn crate::infra::Secret>; }` reacts to
  `must_not_expose_dyn`. The `impl Trait` shape collector (`collect_item_return_impl_traits`) stays
  `ImplItem::Fn`-only **by correctness, not omission**: return-position `impl Trait` is the only
  stable-Rust existential leak, and an associated `const`/`type` has no return type (`const: impl
  Trait` is invalid, `type = impl Trait` is unstable TAIT), so there is nothing for it to observe
  there. No residual FN remains in this pair.

Forward depths (born when built, same `syn` source):
- **`must_not_expose_existential` (unifier)** — a possible future capability folding impl-trait
  (written `impl Future`/RPIT) and async-exposure (implicit `impl Future`) under one "no
  existential at this seam" rule. Deferred: the two syntactic signals stay distinct rules until a
  unification earns its own admission (it must not blur the two findings' identities). Not built.
- **`UnsafeBoundary` — subtree-confined `unsafe`**: **BUILT (v0.1.8).** `UnsafeBoundary::in_crate(p)
  .only_under(["crate::ffi"])` — `unsafe` (blocks, `unsafe fn`/`impl`/`trait`, `unsafe extern`) may
  appear only under the declared subtree(s); a site elsewhere reacts. Observed via an
  `UnsafeSiteCollector` (`syn::visit`) run per-module by a dedicated whole-crate walk inheriting
  `scan_crate`'s guards. **Confinement-only** (the admission-critical scope): the pure "crate is
  `unsafe`-free" case is deliberately excluded — `#![forbid(unsafe_code)]` is stronger (compile-time,
  unbypassable) — so an **empty or crate-root allowed set is a constitution error** pointing at
  `#![forbid]`; this keeps it declarative-not-lint (governs *where* `unsafe` lives, not *whether* it
  exists). Findings are per-module, per-kind (anonymous blocks dedup per module; the trait is in an
  `unsafe impl` finding for injectivity). Stated bounds: `#[unsafe(...)]` attributes, bare `unsafe fn`
  pointer types, plain `extern "C" {}` blocks (call sites still react), and the inherited macro
  whole-crate-scan bound (an unconditional `#[path]` module is now followed as of 0.2.1; a
  `cfg_attr`-wrapped `#[path]` stays the bound). Two adversarial-review rounds hardened it (the propose review
  caught a body-nested-`mod` false negative → `visit_item_mod` left at default + only top-level `mod`
  filtered). Shipped as the OpenSpec change `semantic-unsafe-confinement`.
- **Visibility ceiling — `max_visibility(Crate|Super|Module)`**: **BUILT (v0.1.8).** Generalizes the
  binary `must_not_declare_pub` (now the `max_visibility(Crate)` sugar, byte-stable in findings, rule
  string, and baselines) to a parameterized ceiling: a direct item reacts iff its declared-visibility
  rank (`pub`=3 > `pub(crate)`=2 > `pub(super)`=1 > private=0) is strictly above the ceiling. Same
  `syn` source and item set as before — only the per-item predicate and finding change. Non-compiler-
  expressible (the compiler accepts *widening* a `pub(crate)` declaration to `pub`; the ceiling governs
  the declaration's evolution). Key decision: `pub(in P)` is matched **whole and single-segment**
  (`crate`/`super`/`self`); every other restricted form (multi-segment like `pub(in super::super)`,
  leading-colon) ranks **Crate, a conservative upper bound** — a `pub(in P)` path is an in-crate
  ancestor, so at most crate-visible, so this never under-reacts (no false negative), only ever
  over-reacts under a tight ceiling (a stated bound). Shipped as an OpenSpec change modifying
  `semantic-visibility-boundary`. Adopter-surfaced.

**Internal structure (refinement, not capability) — v0.1.4 → 0.2.0 line:** 渾儀's internals were structured
where a live pain existed — the finding-string formats centralized into one `SemanticFinding`
catalog, the ~8k-line `lib.rs` split into `lib` / `dsl` / `tests`, and the sibling-safe
`::`-containment rule converged into one `path_within` (retiring a drift-prone hand-copied
false-positive/false-negative rule). **Built on the 0.2.0 line — structured semantic facts:** the
structured baseline supplied the previously absent forcing function. A private `PublicSeam` now
carries item/owner/module/member/trait-impl-position data through the lower resolver and collectors;
the one `SemanticFact` catalog derives fact-specific named key fields and byte-identical text. The
canonical path/shape remains the observed `subject` value rather than growing a speculative subject
AST. This closes the live gap where 渾儀's nominally structured key was still one rendered
`descriptor`, so presentation polish would re-identify a baseline entry. See `PROJECT.md`,
"Structure semantic observation facts".

- **`resolve_self_type`'s fixpoint lacks the hop cap its sibling resolver carries — BUILT (0.2.3).**
  **Pressure/source:** a follow-up sweep of the 0.2.2 lesson (divergent-rewrite protection applied at
  one call site, missed at a structurally-identical sibling) found a third instance, this time a real
  safety gap rather than a style duplication. `containment.rs::resolve_self_type` (108-119) hand-rolls
  its own alias+re-export fixpoint loop — `alias_targets.get(&landing)` (an exact full-path lookup)
  interleaved with a full `canonicalize_through_reexports` pass per outer iteration — guarded only by
  `let mut seen = HashSet::new(); while seen.insert(landing.clone())`. The crate's actual shared
  fixpoint, `resolve/mod.rs::canonicalize_through_aliases` (449-476), tries alias-then-reexport at
  *each single step* (not a full reexport pass per iteration) and additionally bounds the loop with
  `let cap = aliases.len() + reexports.len() + 1; if seen.len() > cap { break; }` — its own doc
  comment (437-438) states the two canonicalizers "share one fixpoint / hop-cap implementation and
  cannot drift," a guarantee that does not extend to this third, independent reimplementation. Per
  that same doc comment, the cap exists because "the exact-repeat `seen` set cannot catch" **a
  divergent rewrite** (a chain of landings that never exactly repeats). **Current reaction:** none —
  `resolve_self_type` had no test exercising a divergent (non-cycling) alias chain, so a construction
  that recreates the shape `canonicalize_through_aliases`'s cap was hardened against would loop until
  the alias map is exhausted of *new* strings to produce, which is not itself bounded the way a
  cycle is. **Fix:** `resolve_self_type` now routes through `canonicalize_through_aliases`, retiring
  the hand-rolled loop, so the two resolvers share the one hop-capped implementation `resolve/mod.rs`
  already claimed they do; a regression test (`resolve_self_type_does_not_diverge_on_a_reexport_
  whose_key_prefixes_its_value`) constructs a divergent (non-cycling) alias chain to prove the cap
  actually bounds `resolve_self_type`'s termination, not just `canonicalize_through_aliases`'s own.
  **Risk (before the fix):** low likelihood in practice (needs a specific alias-map shape to
  construct a genuinely divergent, non-cycling rewrite chain) but high severity if hit (an unbounded
  loop in a CI scan, not a wrong answer) — matched the 0.2.2-lesson risk class exactly. **Version:**
  shipped in 0.2.3 — no public API, wire format, or baseline identity change. **Authority:** this
  session's static review + direct code verification; `PROJECT.md` Decisions (0.2.2 lesson, in eight
  rounds).

Explicitly **rejected** (essential gap — would be a false-negative engine, see `PROJECT.md`):
`Send`/`Sync` constraints (inferred auto-traits), external trait sealing (downstream crates),
transitive effect-purity ("no I/O anywhere reachable"). Also **rejected — trait-surface freeze**
(`freeze_methods([...])`, "trait T's method set is closed"): it is **API-contract stability, not
architectural shape** (a stated non-goal — behavioral/contract governance), and a frozen method
*list* in the constitution is a hand-copy of the trait definition that drifts — the exact
declaration-integrity anti-pattern the project fights. The real intent ("keep the facade small") is
a 潛移/review concern, not a brittle enumerated reaction. Adopter-surfaced, declined with reason.

### 漏刻 (Lòukè) — the runtime dimension  · crate `louke`  · **BUILT — admitted layer complete**
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
  - **Probe in an unreachable/orphan `.rs` file counts — CLOSED for root-aware/composed audit
    (0.2.x).** 天衡 now preserves exact Cargo target roots through 星表 and passes those files to a
    louke-local, audit-only module walk. Only the root, inline bodies in reachable files, and
    conventionally resolved `mod name;` descendants count; undeclared or inline-shadow sibling
    files cannot cover a seam. 漏刻 still imports neither 圭表 nor `syn`, and its production face
    remains unchanged. Existing direct callers that pass directories retain the recursive corpus
    for source compatibility; passing root files opts into reachability. An unconditional
    `#[path]`-remapped module is now **followed** to its target (0.2.1, see the `#[path]` sub-bullets
    below) rather than excluded; a `cfg_attr`-wrapped one stays an explicit bound. A shared
    reachability substrate still waits for a second dimension proving genuinely shared semantics.
    - **`#[path]` detection tightened — 0.2.1 adversarial review (CLOSED).** Detection was a raw
      `path` substring scan of the module preamble, so a `// fast path` comment or a
      `#[cfg(feature = "fastpath")]` misclassified a *reachable* module (mis-resolving its file) — a
      **silent coverage FN** risk. Now detected structurally (an outer attribute whose meta name is
      exactly `path`, comments and unrelated attributes skipped); `#[cfg_attr(.., path = ..)]` stays a
      bound. Two pins guard it.
    - **Unconditional `#[path]` followed with rustc fidelity — 0.2.1 re-review (CLOSED).** Beyond
      detecting the attribute, 漏刻 (with 渾儀) now **follows** an unconditional `#[path]` to its file
      so a relocated module's probes count (closing the drop-the-relocated-module FN). Three
      rustc-fidelity corrections landed under adversarial review, each with a real-`rustc`-1.96
      ground-truth test: (1) the base is the **containing file's own directory**, not the
      conventional-child dir — mod-rs-blind; (2) with each enclosing **inline-`mod`** name accumulated
      onto it, so a `#[path]` inside `mod inline { … }` reads `inline/p.rs`, never a same-named orphan
      (the inline-nested base bug was a silent exit-0 drop — the forbidden FN); (3) the byte scanner
      **decodes the path literal's escapes** (`\x`/`\u{}`, raw strings) as syn does, so 漏刻 and 渾儀
      resolve the same file (twin-drift parity). 渾儀's whole-crate walk also stopped misreporting two
      declarations sharing one `#[path]` target as a false module cycle (ancestor-path guard, not a
      monotonic visited set); 漏刻 already accepted such input. `runtime-origin-assertion` and
      `semantic-unsafe-confinement` carry the scenarios; louke stays `syn`-free (三儀 ⊥ 三儀).
    - **`cfg`-gated module whose file is absent is now tolerated — 0.2.1 review, CLOSED.**
      louke's walker errored on *any* unresolvable reachable module, so a `#[cfg(windows)] mod win;`
      with no `win.rs` on a non-Windows checkout hard-failed the audit, breaking cross-platform
      adopters. This was **not** a deliberate bound (an earlier triage wrongly kept it as one): 渾儀
      already tolerates exactly this case (cfg-gated absent → skip; non-cfg absent → exit 2), so
      louke was merely inconsistent with its sibling dimension. Fixed to match — a
      `#[cfg(...)]`/`#[cfg_attr(...)]`-gated module with no file is skipped (it compiles no probes in
      this configuration, so skipping cannot silently cover a seam: no FN weakening), while a non-cfg
      missing module and a resolution ambiguity stay fail-loud. Not `cfg` evaluation: a resolvable
      cfg-gated module is still scanned. `runtime-origin-assertion` updated with a scenario; louke
      stays `syn`-free (byte-level detection, 三儀 ⊥ 三儀).
    - **Forward candidate — `cfg_attr(pred, path=…)` cfg-blindness, both directions (0.3.x depth).**
      A `cfg_attr`-wrapped `#[path]` is a stated bound today (not followed, because following it
      cfg-blind could read a file rustc does not compile in this configuration). The 0.2.1 re-review
      confirmed the bound masks a genuine two-directional divergence when the predicate is *active*
      (e.g. `unix` on a unix host): rustc compiles the relocated file and ignores the conventional
      one, but (a) 渾儀's whole-crate walk **drops the whole module** (an in-domain FN — a real
      `unsafe`/marker in the compiled relocated file goes unobserved), and (b) 漏刻 **scans the
      conventional file** rustc never compiles (an FP on dead code, and an FN on the compiled file's
      seam). Neither dimension evaluates `cfg` (by design, 三儀 ⊥ 三儀), so no single cfg-blind file is
      universally correct. The **FN-safe design is observe-both**: union the relocated *and*
      conventional files (a probe/`unsafe` in *either* configuration reacts), which neither dimension
      does today. A depth, promoted only if a real adopter's `cfg_attr` relocation hides a site — the
      current stated bound is honest, not silent. (`cfg` evaluation itself stays a permanent non-goal.)
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
  re-hand-rolled per repo; (b) the adopter-facing **潛移 generator** (see the 潛移 section); (c) an
  adopter-facing **`tianheng::testing` boundary-test harness** (`assert_violates!` / `assert_clean!`
  over a fixture) — every adopter currently re-hand-rolls a temp-workspace + `check`/`check_all`
  assertion. **Built prerequisite (0.2.0 line):**
  `check_constitution(&Constitution, &Path) -> Outcome` exposes the runner's one shared
  static→semantic→runtime evaluation path without CLI presentation; the composed example no longer
  splits its law back into per-dimension checks merely to inspect findings. **Docs-first shipped
  (v0.1.9):** the COOKBOOK "Test that a boundary reacts" recipe over the public entry points; the
  higher-level assertion/fixture *API* remains deferred until its shape settles under a real second
  consumer — shell-hosted, std-only, feature-gated, additive/patch when it lands (the Spike-A
  verdict). Note: the entry points read a manifest on disk, so an inline-fixture
  ergonomics would still materialize a temp crate. Stated
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

- **Cheap, patch-now: sharpen the README on-ramp. · SHIPPED (v0.1.8)** Make the *first* boundary a
  one-line imitable Phase-0 pattern (lock one seam, Enforce, pipe `--format sarif` into CI) that an
  agent scanning the crate copies by reflex — 潛移 at the doc level, near-zero cost. Likely the
  highest adoption leverage per unit effort.
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
