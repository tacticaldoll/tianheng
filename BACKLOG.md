# Backlog

Forward-looking work, deliberately deferred. Promote an item to an OpenSpec change when
you pick it up. Every future reaction obeys the drift law:

> **No drift type without an observation source. No target type or name without a
> reaction.**

Nothing here is "designed" yet — reaction *phases* with their observation sources named,
not APIs. A new observation dimension is **a crate, born when it is built** (never a
pre-created empty stub); the heavy dependency it needs is quarantined to that crate so the
`guibiao` core stays `serde_json`-only.

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

Until the trigger fired, staying 0.1.x was a **deliberate hold** (waiting for reaction), not drift.

**Trigger fired (2026-07) — the 0.2.0 window opened and has now shipped.** Two adopters, two shapes
(verified against their source) earned that window:

- **`../pacta` (composed)** depends on `tianheng` + `guibiao`, drives one `Constitution` through
  `tianheng::run` / `check_all` (+ `guibiao::check_and_cover` for coverage). This is the
  suite/funnel adoption → fires *"a first serious external adopter needing a compatibility promise"*.
- **`../modou` (standalone 圭表)** depends on `guibiao` **only** (no `tianheng`), re-exporting
  guibiao's DSL + widened surface and adding a thin CLI shell → fires *"a 儀 actually adopted
  standalone"*.

What each unblocks — the two gates are **not** one:

- **`guibiao` widened surface (`check_and_cover`/`baseline`/`coverage`/`projection`) → resolved to
  KEEP + shape.** `modou` depends on it directly, so narrowing/deprecating it breaks a real consumer.
  This **supersedes** the guardrail line above that listed "`guibiao`'s widened `pub` face" among the
  *safe-to-break internal* surfaces — it is now a kept surface. The 0.2.0 break must avoid breaking
  **both** pacta's builder (`Constitution`/`CrateBoundary`/`run`/`prelude`) **and** modou's dependence
  on guibiao's widened projection/baseline (`report_json`/`constitution_json`/`check_and_cover`/
  `Baseline`/`Coverage`/`apply_baseline`/`Violation`/`ViolationId`).
- **xuanji-sink (run/projection → `xuanji`, generalizing `BoundaryKind`/`Polarity`) → STILL gated.**
  The standalone product reaction *did* arrive (modou), but it **validates keeping the dimension's own
  widened surface, not sinking it**: modou re-exports guibiao's projection, so the sink would *break*
  modou; and the sink's dedup payoff needs *multiple* standalone 儀 (a hunyi/louke standalone product),
  which no adopter shows. Revisit only when that arrives — not before.
- **Structured-baseline / typed-identity bundle → BUILT (0.2.0).** The per-dimension union resolved
  as dimension-owned typed facts projected into 璇璣's vocabulary-neutral, flat `FindingKey`
  envelope; no dimension vocabulary or resolver internals moved into the shared model. `reason`
  stays prose (潛移), while the human finding is rendered from the same fact but never becomes the
  sole version-2 identity. `pacta` + `modou` remain the reference consumers bounding future changes
  to `Violation` / `ViolationId` / baseline projection; the settled rationale lives in
  `PROJECT.md`, and the residual pressure is recorded below.
- **P3 (un-auditable-probe file-granular re-mask, below) — re-evaluated: no single-point miss.** Two
  co-existing non-literal probes in one file react as one file-level violation (neither masked); the
  only masking is the *temporal* stale-baseline re-mask already recorded below as low-risk (surfaced by
  `Baseline::stale`). Not a coverage false negative; do not re-key it in 0.2.x absent a stronger
  reaction that justifies a stable per-probe locator.

### Post-0.2 structured-identity pressure — preserve the wire, promote only on reaction

Version 2 closed the live presentation-as-identity failure, but publishing the key also made each
dimension's namespace, fact code, field names, canonical values, and the outer `target` / `rule`
roles compatibility-bearing. Keep the current line honest without treating every pressure point as
an invitation to redesign the model:

- **Published identity-schema compatibility reaction — BUILT (0.2.x).** Each dimension owns an
  exhaustive test catalog for every shipped fact family and identity-bearing discriminator. The
  catalogs pin the 0.2.0 namespace, code, named fields, representative canonical values, and a real
  production-emitted target/rule/key wiring case per dimension — never an entire presentation JSON
  snapshot. Adding a fact or nested discriminator now requires an explicit catalog decision, while
  finding wording remains free. Key-producing canonicalizers (`DependencyKind::key_label`, semantic
  path/type/signature rendering, and the runtime unregistered `TypeId` discriminant) are marked as
  published wire; the runtime discriminant retains its honest build-local stability bound. This is
  a compatibility reaction only: no production behavior, key, public API, manifest, or package
  version changed. The unsafe-site decomposition, async-seam identity, and rule-key separation below
  remain breaking-window questions rather than being smuggled into the catalog work.
- **Unsafe-site fact decomposition — next breaking-window candidate (strengthened by 0.2.1).**
  `SemanticFact::UnsafeSite` still stores a pre-rendered `label` as one opaque key field beside
  `module`, despite the collector observing those roles separately. 0.2.1 **widened that opacity**:
  closing the trait-impl identity-collapse FN made a trait-impl unsafe fn's label carry the trait
  role too — `unsafe fn <Trait for Owner>::method` (an inherent one stays `unsafe fn Owner::method`;
  an impl stays `unsafe impl Trait for Owner`) — so the trait is now **identity-bearing**, not
  presentation, and the single string crams form / trait / owner / name. The existing flat envelope
  can already represent form / trait / owner / name / module as named scalar fields; no richer shared
  value model is needed. The concrete cost of keeping the key opaque is now visible: that
  trait-qualification re-keyed a trait-impl unsafe fn, so a 0.2.0 baseline entry for one resurfaces on
  upgrade and must be re-accepted — a named-field decomposition would have localized the change to a
  new field rather than churning the whole key. Do not re-key published version-2 baselines in a
  patch. Anonymous `unsafe {}` remains deliberately module-granular, never ordinal-keyed.
- **Async seam identity — decide in the next breaking window, do not assume the answer.** The
  owner-qualified module/trait/type plus item name already identifies an async public seam, while
  the current key additionally stores the rendered signature tail. Decide whether the observed
  architectural fact is the seam or its exact declaration. If it is the seam, parameter/return
  refactors should change presentation but not identity; removing `signature` is nevertheless a
  version-2 wire break and cannot ride 0.2.x.
- **Rule identity is still presentation-bearing — next reaction-model window.** `rule` remains both
  human text and a `ViolationId` component (and SARIF `ruleId`); the runtime prod face additionally
  folds the complete allowed-origin set into that string, so an unrelated allowlist edit changes
  event identity. Treat existing labels as wire identifiers throughout 0.2.x. Revisit a stable
  `rule_key` separate from human rule presentation only with a deliberate model/baseline migration,
  not wording polish.
- **Baseline metadata parse strictness — BUILT (0.2.x).** Omitted or explicit-null `owner` /
  `tracker` parse as absence, strings are preserved, and every other JSON type now invalidates the
  baseline instead of silently erasing governance metadata. The shared parser reacts for standalone
  and composed consumers alike; the gate exits 2, while the explicit `--write-baseline` recovery
  path retains its warning-before-fresh-snapshot behavior. Identity, canonical output, and the
  version-1/version-2 matching contracts are unchanged.
- **Legacy migration communication — BUILT (documentation only).** Version 1 remains supported by
  exact `(target, rule, finding)` matching. The adopter workflow now names the existing
  `--write-baseline` action as the bounded opt-in upgrade, tells wording-sensitive V1 users to
  rewrite before presentation changes when suppression or metadata preservation matters, and states
  that exact live matches carry metadata while stale entries drop with the fresh snapshot. No
  migration command, wall-clock window, automatic read rewrite, or perpetual warning was added.
- **Flat-envelope pressure trigger — watch, do not pre-build.** `FindingKey` is intentionally the
  public, vocabulary-neutral shared chokepoint for identity instances, while schemas stay in their
  dimensions. Revisit its flat string-field representation only when a real observed fact cannot
  retain role separation, injectivity, and presentation-independent stability through named scalar
  fields plus dimension-local canonicalization. Its place in the 0.2.x reaction-inspection contract
  makes in-place reshaping costly; prefer a dimension-local or additive path unless the forcing fact
  proves those insufficient.

### Product maturity from the 0.1.x hold — shipped history and remaining DX/trust work

Reading as a **mature product** during the deliberate hold was not new capability — it was
**lower friction (DX) and higher trust**. This is the drift law applied to go-to-market: build no
shell without a reaction, but polish the packaging and on-ramp of the observation mechanisms that
already react. Everything here is **convention / CI-reaction hygiene — zero constitution boundaries,
zero pre-built empty shells** (the class of the branching ritual and license-bundling in
`AGENTS.md`). Three tracks, each with the guardrail that keeps it inside Tianheng's own law.

**1 — Onboarding: examples that dogfood, mirroring the funnel.**

- **Role split (the invariant).** A live-red `tianheng check` target and a green `cargo test`
  citizen are different roles; compiling both into one workspace member is the trap. So the
  **violating subject** is a check *target* — its own excluded sub-workspace (self-`[workspace]`),
  like `crates/tianheng/tests/fixtures/{clean,violating}`. Intentionally-"wrong" code lives there as
  **data**: invisible to `cargo build/clippy/fmt/doc --workspace` (the real enemy is
  `clippy -D warnings` on deliberately-ugly demo code, not self-governance) and to self-governance,
  which declares no boundary over it — self-governance governs only the family's *crate-dependency*
  edges (confirmed in `self_governance.rs`: all `CrateBoundary`, no module/semantic rule), so an
  example's internal import-direction / API-leak fault is never scanned by it. The **driver** — the
  harness that runs the reaction and asserts the outcome — is a `publish = false` member under a
  top-level `examples/` (added to `[workspace] members`, kept **out of `crates/`** so the `crates/*`
  CI globs never see it), clean code that passes every gate, green *because* the reaction fired. It
  need not `deny(missing_docs)` (a demo, not an API), so it does not fight `cargo doc -D warnings`.
- **The composed example demos 天衡 as funnel target, in two modes** — because the 三儀 react in two
  places:
  - *check-mode* (CI-time, against source): one target carrying a static fault (a `domain` →
    `infra` import) and a semantic fault (an `api` `pub` signature exposing `infra::DbPool`), with
    the declared runtime seam **probed** so 漏刻's CI face (`audit_probe_coverage`) *passes* —
    coverage is satisfied, because the honest steady state at CI time is a well-covered seam, and
    漏刻's actual reaction is a runtime event shown in run-mode (below), not a check-time verdict.
    Checked against **incrementally-scoped constitutions** — static-only = the 圭表 dimension's
    view, +semantic = +渾儀, full = 天衡 all-open. Single 儀 → suite made literal from one body of
    code. This is the funnel, **not** standalone-crate adoption (the README's job, below). Assert
    `exit 1` + the expected `reason`/`rule` via `--format json` — never the ANSI render (track 2
    makes it a moving target). A tiny `missing-probe` variant (the same seam with its
    `assert_boundary!` removed) demonstrates 漏刻's CI-face *reaction* — the "declared but never
    enforced" gap — as a side branch, so the main target need not choose between a passing coverage
    check and a probed seam that run-mode requires.
  - *run-mode* (runtime, in a binary): 漏刻's prod face **cannot** be shown by `check` — `check`
    runs only louke's *CI face* (the `audit_probe_coverage` source scan). The runtime reaction fires
    against live objects in a running binary, so a small runnable binary `install`s a
    `RuntimeBoundary`, crosses a seam with a disallowed-origin object, and the CI script asserts the
    **emitted `Violation` event** (default reaction; `panic` stays opt-in). This is intrinsically
    top-down — you wire louke into your binary — reconfirming that 漏刻 belongs in the *composed*
    example, never a standalone on-ramp.
- **Standalone dependency-footprint pitch → shown by the standalone examples; 漏刻's stays a README
  snippet.** A footprint *is* the product claim (圭表 syn-free and light; 渾儀 the one that *carries*
  the quarantined `syn` — honest, not "light"; 漏刻 `xuanji`-only), and a composed member's
  `Cargo.toml` (whole family + `syn`) cannot show any of them. The `guibiao`-standalone example
  commits `[dependencies] guibiao = "0.2"` alone (syn-free, light); the `hunyi`-standalone commits
  `hunyi = "0.2"` (which honestly pulls `syn` — the point is that the semantic instrument is *where
  syn lives*, not that it is light) — each footprint *demonstrated, not asserted*. 漏刻 has no
  standalone example (a top-down depth), so its `xuanji`-only footprint stays a copy-paste
  README/docs.rs snippet. Every crate's README still carries a ~10-line Constitution + a copy-paste
  GitHub Actions snippet (`tianheng check` on PRs) as prose — a snippet, not a published composite
  action (more weight; defer to a reaction).
- **Committed-honest, CI-local (one resolution, not two forms in tension).** Every example commits
  the adopter's real form — currently `guibiao = "0.2"` — so its `Cargo.toml` is copy-paste-honest and, for the
  standalone examples, *is* the footprint demo. To also track HEAD in CI (catch a local regression
  before it publishes), the CI script injects the `--config patch.crates-io.<crate>.path=` resolution
  the `packaged-selftest` job already uses — committed file honest, resolution local. A raw `path =`
  dep is **not** used for the standalone examples: it would falsify the footprint demo (an adopter
  writes a crates.io requirement, never `path =`). The composed example, whose footprint is not the pitch, may use
  `path =` freely.
- **Decided — three examples: `composed` + `guibiao`-standalone + `hunyi`-standalone.** The two
  CI-time instruments each get their own runnable standalone demo (check-against-source, no runtime),
  so 圭表 (the strongest standalone product) and 渾儀 each show a real light `Cargo.toml` and an
  on-ramp, not just a README snippet. 漏刻 has **no** standalone example — it is a top-down *depth*,
  so it appears only inside `composed` (run-mode). The accepted cost is the largest example-set to
  maintain; the "dogfood does not rot" CI candidate below is what keeps that cost bounded.
- **0.2.x dogfood refresh — BUILT.** The three examples above remain the adoption funnel;
  `unsafe-confinement` and `sans-io-pure` are focused capability demonstrations added only where the
  family itself cannot honestly carry the law. One explicitly non-tutorial `capability-catalog`
  owns the remaining published-family breadth (dependency-source metadata, external-crate
  confinement, trait-impl locality, forbidden markers, dyn exposure, and impl-trait exposure),
  asserting stable rule/FindingKey identities through the real composed evaluator and shell.
  Tianheng's genuine self-Constitution now also runs through `check_constitution`, so self-governance
  dogfoods static → semantic → always-run runtime audit without inventing fake self-law. This is a
  frozen 0.2.x coverage ledger, not permission to add an example per modifier.
- **Published-family completeness reaction — BUILT (0.2.x).** The examples gate now compares one
  repository-only inventory with fulfilled reaction owners, counting a family only after its real
  evaluator and structured identity assertions succeed. Unknown claims and missing owners both
  fail loud and name the family. The inventory remains test governance: no public
  `PublishedBoundaryFamily`, production metadata, violation field, baseline identity component, or
  serialized-report field was added. OpenSpec/API review still decides whether a future insertion
  path is a family, depth, modifier, or shorthand; once declared as a family, the executable ledger
  prevents it from landing ownerless.
- **Worked shape (for imitation — the DSL is real).** `composed` grows its constitution by one
  `.boundary()` per stage: `ModuleBoundary::in_crate("app").module("crate::domain")
  .must_not_import("crate::infra")` (圭表) → `SemanticBoundary::in_crate("app").module("crate::api")
  .must_not_expose("crate::infra::DbPool")` (渾儀) → `RuntimeBoundary::at("adapter-seam")
  .only_origins(["crate::adapters::blessed"])` (漏刻). run-mode is a hexagonal port/adapter seam: a
  `trait Adapter: louke::Tracked`, a blessed adapter that `register_origin!`s **inside** its own
  module (the origin is the registration site's `module_path!()`) and a rogue one whose origin is
  not in `only_origins`; `install` the boundary, cross the seam with the rogue via
  `assert_boundary!("adapter-seam", &*obj)`, and the fail-closed probe emits the `Violation` event
  the CI script asserts (an unregistered type fails closed too). This is the same port/adapter shape
  whose *static* layering 圭表 governs — the audience that enforces "domain depends inward only" is
  the one that wants "only the blessed adapter crosses the seam," which is why they compose.
- **Contract demonstrations (absorbs the upstream-review points — show, don't tell).** These need
  only one boundary + one violation, so they ride the simplest example — the `guibiao`-standalone one
  — **not** the already-loaded `composed` target (which stays focused on funnel + runtime; piling the
  severity/baseline axis onto its enforce-scoped funnel would collide expected exit codes). The
  examples *demonstrate* the public-contract invariants a reviewer otherwise has to infer from
  source, turning them into runnable proof (dogfood / 潛移):
  - *Presentation ⊥ verdict* — run the same check in default / `--format json` / `--format sarif`
    and assert an **identical exit code** across all three; only the rendering differs, so
    formatting never moves the verdict.
  - *Adoption ladder, lived* — the same target run through the two-axis ramp: `warn` severity → the
    violation is reported but `check` exits 0 (signal without gating); a generated `Baseline`
    grandfathers the existing violations → exit 0, while a newly-added one → exit 1; then `enforce`
    with no baseline → exit 1. The real `Baseline` JSON is `xuanji`'s versioned wire contract in
    action.
  - *Identity ⊥ metadata* — baseline a violation, **move the offending code to another file**
    (changing `Violation.file`), re-check: the baseline still matches and the violation stays
    grandfathered, because `ViolationId` excludes `file` from its structured identity. Refactoring
    file layout does not churn your baseline — the stability contract made tangible.

**2 — Output: reaction-voice render polish, never fix-instruction.**

- The actionability goal is right and half-built: `AGENTS.md`'s read order is **reason → file →
  finding**. Polish the terminal render to make that visual (reason foregrounded, file secondary,
  finding concrete) so the first glance lands on *why* — pulling out the linter's scolding tone.
- **Hard guard — keep the reaction voice, give no fix command.** `Fix: remove the import at
  db.rs:12` is a lint's prescriptive-remediation voice, and Tianheng is explicitly *not a lint*
  (see the non-goals); worse, the "fix" is often wrong (move the type / invert the dep / add a port,
  not "remove"). Emit *what · where · why*; *how* is the adopter's, repaired toward the `reason`.
- **Placement: `tianheng` (shell) only, hand-rolled ANSI.** 璇璣 renders no verdict and is
  `serde_json`-only, so color/layout cannot live there; and any color crate would trip `tianheng`'s
  own `restrict_dependencies_to(guibiao, hunyi, louke, serde_json)` self-law. So it is a small
  hand-rolled ANSI module in `tianheng` — zero new dependency, no self-law amendment.
- **Three things that make hand-rolled color read as mature (all still zero-dep):** (a) TTY-gate via
  `std::io::IsTerminal` (in std since 1.70; MSRV 1.85, so free) and honor `NO_COLOR`, so a
  redirected / CI log gets no escape bytes; (b) color only the human/default format — `--format
  json` / `sarif` stay uncolored and TTY-agnostic (CI greps the SARIF `"version": "2.1.0"`; no
  machine consumer may eat color bytes); (c) distinguish the voices — exit 1 (violation) vs exit 2
  (constitution/usage error) — by prefix/color. **Width-agnostic:** no wrap-to-terminal-width (width
  detection needs a dep or ioctl); a fixed prefix + indented lines reads at any width and stays
  zero-dep.

**3 — Repository hygiene: the demand-signal funnel.**

- **Per-儀 issue templates, routed to the spectrum** (not flat), doubling as the 0.2.0-trigger
  collection funnel: 圭表 "report a static import/dependency false-negative / request a layer rule"
  (bottom-up); 渾儀 "report a missed API leak / request structured exposure typing" (bottom-up); 漏刻
  "**discuss** a runtime origin pattern" (top-down — "discuss," not "report," because 漏刻's signal is
  a 天衡 adopter leaning on the runtime dimension as a primary reason, not standalone adoption).
  Every template carries one shared field that *is* the funnel instrument — **"using `<crate>`
  standalone, or via `tianheng`?"** — so each bug report becomes a demand-signal datapoint feeding
  the 0.2.0 "a 儀 adopted standalone" trigger; plus version, a minimal repro (Constitution snippet +
  code), observed-vs-expected reaction, and the `--format json` output.
- **User-facing `CHANGELOG.md`** — an adopter-facing projection (a *different reader* than the git
  history, which is why it earns its keep despite the self-describing-commit rule): record every
  false-negative closure / depth extension — trust for conservative adopters even though 0.1.x
  promises no breaking. **Drift guard** (the declaration-integrity / prose-drift class): a
  hand-maintained CHANGELOG mirrors releases and will drift from the `release: X.Y.Z` spine, so
  anchor its maintenance into the release SOP — the entry written **on the release branch before its
  squash to `main`**, never independently.

**0.2.x hygiene reactions — promote independently.** The examples gate now owns both halves of
"dogfood does not rot": its existing behavior checks run every isolated example and assert the
declared Tianheng reaction, while the **BUILT (0.2.x)** quality matrix first runs format, all-target
Clippy, and warning-denied rustdoc in each isolated workspace using the same execution-time local
patches. A real warning fixture proves quality failure stops before reaction acceptance; the first
live matrix also corrected two safe public raw-pointer wrappers while leaving their deliberate
unsafe-confinement reaction intact. The gate's **BUILT (0.2.x)** example-set reaction now also
derives every immediate `examples/*/Cargo.toml` workspace and requires a fulfilled owner after its
quality and declared reaction assertions; a forgotten directory or nonexistent claim fails loudly,
independently of the published-family ledger. All machine projections and generated baselines live
under one invocation-local, failure-cleaned temporary root, so parallel runs cannot share evidence.
Separately, the release-readiness review found an empty
`[Unreleased]`, stale 0.1.x compatibility prose, and repeated lockfile/version friction. The
repository-state release-coherence gate is now **BUILT (0.2.x)**: it derives development,
release-ready, and snapshot state from the exact release-commit spine and relates manifests,
internal pins, CHANGELOG, and release-time lock entries without a wall clock, warning window, or
Tianheng constitution boundary. Development requires adopter-facing notes but does not manufacture
lock churn; release-ready and snapshot states require every version surface to agree. Its dedicated
full-history CI job preserves the distinct observation source and repair direction rather than
folding release integrity into generic polish machinery.

**Rests on the spectrum + triggers** (the product-identity note above): 圭表 genuinely standalone ·
渾儀 semi-product (a distinct, only partly-overlapping library-author audience) · 漏刻 a *depth* of
the composed product, not an on-ramp (its adoption path is top-down via 天衡). The demand-signal
triggers differ in **direction** — bottom-up ×2, top-down ×1 — which is why the issue-template
routing and the standalone-pitch placement differ per 儀.

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
- **Considered decline — a mechanical "policy adapter" importing an existing rule source into a
  `Constitution`.** The *goal* (low-friction adoption, do not reinvent governance syntax) is
  legitimate and is served by the **cookbook / examples** (track 1) that translate common governance
  intents into boundaries. The *mechanism* — an importer that generates a Constitution from an
  external / prose rule source — is **declined for now**: it bypasses the 潛移 authoring surface (the
  human/agent *writing* the boundary and its forward-voice `because` is the point, not a generated
  artifact), risks a `because` that asserts structure the law does not react to, and has no concrete
  machine-readable source format or adopter demanding it (drift law: no capability without a
  reaction). Tianheng keeps the *declared, per-target* layer; it is not a policy-translation engine.
  Reconsider only if a concrete machine-readable source **and** a real adopter appear together.

### 0.1.5 — known-depth consolidation · **SHIPPED**

0.1.5 has converged from scope map to shipped state (0.1.6 through 0.1.10 and 0.2.0 have since shipped
on top of it; 0.2.x is the current line). Its built items are recorded once in the dimension / 三司 sections
below; the remaining forward work stays there as forward depth. The 0.2.0 bundle above remains the
only currently named breaking line.

### 0.1.6 — metadata SSOT extraction + forbidden-marker re-export/rename hardening · **SHIPPED**

The `serde_json`-only cargo-metadata substrate (`cargo_metadata` / `find_package` /
`crate_root_file`), written twice and drifted (the proc-macro `crate_root_file` false negative),
extracted into **星表 (`xingbiao`)** below the 三儀 — sibling to 璇璣, enforced by
`restrict_dependencies_to(["serde_json"])` and the 圭表/渾儀 downward allowlist edges. The metadata
twin-drift class is now structurally impossible. Recorded in `PROJECT.md` Decisions (the 星表 entry);
the internal refactor followed the `xuanji` precedent (no OpenSpec capability change).

The **渾儀 forbidden-marker** capability closed two false negatives inside its observed scope — the
same class as the v0.1.3 re-export-exposure closure below, in the sibling capability. (1) A hand impl
whose self-type is spelled through a `pub use` re-export (`impl Marker for crate::facade::Order`) was
not followed to the definition: the impl form chased only the type-alias map, never the re-export
closure its siblings (trait-impl, exposure) already close. (2) Leaf-matching compared the *written*
trait/derive leaf, so a local `use … as` rename (`impl Ser for …` / `#[derive(Ser)]`) evaded it.
Both are closed by folding the self-type canonicalization into `resolve_self_type` — canonical **by
construction**, the re-export and type-alias closures interleaved to a fixpoint, so no caller can
resolve a self-type without closing them (the structural-convergence固化 discipline); it keeps the
`CurrentModule`-fallback alias-target map, not the `Ignore`-built exposure alias map, so an alias to
a bare local struct (`type Bar = Real`) still lands — and by resolving the trait/derive leaf through
the site's `use`-map before matching, falling back to the written leaf so leaf-matching stays
cross-crate-blind. Spec-conforming bugfix (semantic-forbidden-marker already forbade a false negative
in the observed scope); its scenarios were aligned to the fixed behavior. Additive/patch, no OpenSpec
capability change.

- **Reserve (still future): a cross-dimension conformance reaction** for the logic that genuinely
  cannot share — the `syn`-vs-token-scan **resolvers** and byte-scan-vs-AST **module reachability**,
  which stay per-dimension because sharing them forces `syn` into the light core. A `cargo test` gate
  feeding identical inputs to each dimension's parallel logic and asserting agreement would catch
  drift *without* moving code. Deferred until a resolver twin-drift actually bites; 星表 does not
  address it.

## Reaction phases — the 三儀 (observation dimensions)

Ordered by readiness. All three instruments ship in v0.1.0 (圭表 static, 渾儀 semantic, 漏刻
runtime); the entire admitted 三儀 layer is now built. What remains below is the rejected set
per dimension and the 三司 governance/observability layer.

### 圭表 (Guībiǎo) — the static dimension  · crate `guibiao`  · **BUILT — proven core (from modou), growing by depth**
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
  - **Forward candidate — follow unconditional `#[path]` in 圭表 (0.3.x depth, non-breaking).** As of
    0.2.1, 渾儀 and 漏刻 **follow** an unconditional `#[path = "…"] mod x;` to its target (base = the
    containing file's own dir with each enclosing inline-`mod` name accumulated onto it, rustc's
    mod-rs-blind rule; `cfg_attr` stays a bound). 圭表 still holds the v0.1.4 posture of keeping such
    modules outside its reachable `use`-graph — so it is now the **one dimension that diverges**: a
    `use` edge inside a `#[path]`-relocated module is not governed by an import/confinement rule. The
    observation source already exists (圭表's own `use`/`mod` byte scan); following the relocation
    there — reusing the rustc base-directory rule the other two now share, staying `syn`-free — would
    close the divergence. A depth (additive, false-negative closure), promoted when a real adopter
    relocates a governed module; the three-dimension agreement is the correctness pressure, not size.
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
- **Module-scoped external-crate confinement — `confine_external_crate(C)` — BUILT (v0.1.7).** The
  middle cell between crate-granularity (`restrict_dependencies_to`, whole-crate) and intra-crate
  module direction (`restrict_imports_to`, which by design **never flags an external import**): "crate
  Y *may* depend on C, but C may be imported only under subtree S" — the FFI/platform-vocabulary
  confinement pattern. Declared as
  `ModuleBoundary::in_crate(p).module(S).confine_external_crate(C).because(…)`; the confined crate is
  the violation `target` and the offending importer the structured fact, so
  `(target, rule, finding_key)` is
  injective by structure. This is the **first 圭表 rule that observes external-crate imports** — it
  inverts, for this one rule only, the module scanner's long-standing "external imports are out of
  scope" stance; every other rule keeps ignoring them. This is **layer (a), import confinement**
  (`use C::…` only under S), reusing the existing `use` scan directly — the in-scope core. No new
  builder-intermediate type (one method on the existing `ModuleTargetDraft`); additive/patch,
  semver-safe within 0.1.x. Not `cargo-deny`'s lane (declared/per-module, not resolved/whole-graph).
  Shipped as the OpenSpec change `external-crate-confinement`, whose synced spec is now the SSOT
  under `openspec/specs/external-crate-confinement/`.

**Inline-symbol-path confinement — layer (b): BUILT (v0.1.8).** The sibling of
`confine_external_crate` (layer (a)), realized as the `must_not_call_inline` rule
(`inline-symbol-path-confinement`): within a governed subtree, forbid inline symbol-path **calls**
resolving under a module prefix (the "core reads no ambient clock; time is injected" pattern).
Surfaced by that adopter demand. Key decisions, hardened through a feasibility spike and three
adversarial-review rounds:
- **Feasibility spike** (tokio/hyper/reqwest/chrono, prefix `std::time`): macro-body reads are rare
  and resolve cleanly under a resolve-only posture; the FP concern was a *target-granularity*
  choice, not a scanner limit.
- **Call-vs-mention default** (no read-verb heuristic in 圭表): a bare `.must_not_call_inline` reacts
  on calls; type annotations and constants pass. `.ending_with([verbs])` narrows (adopter owns the
  FN); `.strict_prefix_only()` escalates to mentions; the two are mutually exclusive (exit 2).
- **Close the idioms, fail-closed on globs**: resolution follows the alias-carrying use-map, local
  `type` aliases, and the local `pub use` re-export closure to a fixpoint; a glob that can bring a
  prefix-resolving name into scope reacts fail-closed, stated by *hazard not shape* (recursing into
  local re-export closures) so the rule does not itself drift.
- Stayed 圭表-internal on the hand-rolled token scanner (serde_json-only, no `syn`, 三儀 ⊥ 三儀).
- **`.strict_external()` opt-in — depth (v0.1.9):** reclassify a fully-qualified un-`use`d external
  call (`chrono::Utc::now()` with no `use chrono`) as external when its head matches a declared
  dependency, closing the sysroot-vs-external asymmetry (a false negative). A new patch-safe
  `#[non_exhaustive]` variant (see the model-surface debt above), default byte-identical, gated
  behind a local-precedence ladder (no false positive on a local item named like a dependency, at
  any nesting depth) with stated bounds (`extern crate … as` rename; single-segment over-reaction).
  圭表 grows its **own** rename-aware declared-dependency reader — no `渾儀` edge (三儀 ⊥ 三儀).

**Declined — externally covered (not a forward depth):**
- **Resolved dependency-source / build-provenance** — *declined.* Cargo-deny owns the resolved,
  whole-graph source-provenance layer; Tianheng keeps the declared, per-target manifest layer. See
  `PROJECT.md`'s 圭表 source decision for the full A/B rationale.

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
stays a cfg-blind bound; an absent unconditional target fails loud (exit 2). (The v0.1.4 posture of
keeping `#[path]` modules outside scope — governing neither the target nor a same-named orphan — is
**superseded**; 圭表 still holds that older bound, see its section.)

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
- **CI-declared vs runtime-installed law can diverge — evaluated, kept as documented convention.**
  *From an external review.* `install` takes an arbitrary `IntoIterator<Item = RuntimeBoundary>`, so
  the runtime-installed set can diverge from the `Constitution` the CI face verified: a
  declared-but-uninstalled boundary never enforces (or, if crossed, panics as an *undeclared* seam —
  the wrong reaction), and an installed-but-undeclared one enforces a law CI never saw. A fix was
  designed — a prod-face startup reaction `verify_installed(&declared)` comparing the live registry
  to the declared set, framed as an instance of the "Declaration integrity" decision — and **rejected
  on adversarial review**: (1) strict set-equality contradicts the deliberately-open `install`
  iterator (a legitimate compositional/conditional install would false-positive); (2) comparing seam
  *names* misses a diverged `allowed` allowlist — the load-bearing law — so it would green-light
  exactly louke's one forbidden bug (a declared boundary silently not enforcing as declared); (3) the
  only honest structural closure (make `install` consume a `Constitution` projection) **breaks the
  `install` API and shifts louke from measure to prevent**, a real identity change. Given the project
  already makes install-vs-constitution the user's responsibility with a documented idiom
  (`install(constitution().runtime_boundaries()…)`, and the prod face fails loud on a crossed
  undeclared seam), and there is **no live consumer**, the edge stays **documented convention**. The
  structural closure (install-from-projection) is the option to revisit *if* a consumer ever
  justifies the identity cost — not before. Do not re-open as a coverage FN: it is a deliberate
  deployment-edge convention, not a scanner gap.

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
  assertion (the same rebuild pain as (a)). **Built prerequisite (0.2.0 line):**
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
