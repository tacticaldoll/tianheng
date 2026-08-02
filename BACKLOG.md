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
  published API and current baseline/report identity wire.
- **DESIGN-BREAKING** — a supported problem whose honest solution needs a public or wire migration.
- **WATCH** — plausible pressure without enough adopter, second-consumer, or correctness evidence.
- **ACCEPTED DEBT** — a known, bounded risk whose current reaction or documented coverage bound is
  intentionally sufficient.
- **DECLINED** — a considered direction rejected for a recorded reason.
- **BUILT / HISTORY** — shipped context retained only where it explains a live contract or trigger;
  requirements live in [`openspec/specs/*`](openspec/specs) and settled rationale in [`PROJECT.md`](PROJECT.md).
  Detailed historical ledgers for 0.1.x – 0.3.0 are archived in [`docs/history/0.1.0-0.3.0-built-ledger.md`](docs/history/0.1.0-0.3.0-built-ledger.md).

## Open defect queue

No live queue currently — the `v0.2.3..release/0.3.1` adversarial sweep's queue is fully drained:
every entry reached a terminal state (fixed, verified moot, refuted, or promoted to a decision
below), and its record is closed and retired to
[`docs/history/0.3.1-adversarial-sweep.md`](docs/history/0.3.1-adversarial-sweep.md). A future sweep
gets its own dated `docs/audit/*.md` queue file and its own pointer here; this section stays empty
between sweeps rather than pointing at a stale one.

## Live decision index

### DESIGN-BREAKING

- **Owner-label identity collapses across a cfg-collided self-type alias.** Class:
  DESIGN-BREAKING. Observed pressure: reproduced by the maintainer during round-3
  adversarial review of `change/hunyi-cfg-branch-use-reexport-merging` (PR #149) —
  two genuinely independent violations sharing one cfg-collided self-type alias
  (`#[cfg(unix)] use crate::a::Foo as X; #[cfg(not(unix))] use crate::b::Bar as X;`,
  each arm implementing the same governed trait) render the identical single-candidate
  owner label and collapse to one finding under exact-identity dedup, across
  trait-impl-locality, forbidden-marker, unsafe-confinement, and signature-coupling at
  once. Observation source: that review's reproduction, recorded in the historical
  0.3.1 adversarial sweep (`docs/history/0.3.1-adversarial-sweep.md`). Current bound:
  `canonical_self_owner`/`canonical_self_owner_without_fallback`
  (`crates/hunyi/src/resolve/shape.rs`) and `canonical_unsafe_owner`
  (`crates/hunyi/src/scan.rs`) each render a label from a single resolved candidate,
  cfg-blind, feeding straight into a dedup key. Risk: a real, enforce-severity
  violation is silently lost whenever this exact cfg-collision shape occurs — a false
  negative, the one bug class PROJECT.md's Core Contract forbids outright — but the
  trigger (a cfg-gated self-type alias reused across cfg-exclusive impls of the same
  governed trait/type) is narrow and has not yet been observed outside adversarial
  probing. Promotion trigger: either a redesigned owner identity that stays injective
  across cfg-ambiguous candidates, or a different anti-collapse key not fed by a
  single-candidate renderer — real design work, not a mechanical multi-candidate swap
  (explicitly scoped out of PR #149 as its own follow-up). Version class:
  DESIGN-BREAKING (an injective owner identity almost certainly changes baseline
  `finding_key` shape for every affected capability). Authority: `PROJECT.md`'s
  violation-identity decisions, PR #149's commit body, `docs/history/0.3.1-adversarial-sweep.md`.

- **An absolute `#[path]` literal's identity still disagrees across checkouts when its
  target coincidentally lies under one checkout's own anchor.** Class: DESIGN-BREAKING.
  Observed pressure: found during round-2 adversarial review of
  `change/louke-unauditable-probe-relative-identity` (PR #157), which closed the
  general relative-path case but left this one already-non-portable construct
  unresolved. Observation source: pinned regression test
  `a_nested_absolute_path_literal_still_disagrees_across_checkouts_a_known_residual_gap`
  (`crates/louke/src/audit/tests.rs`). Current bound: `strip_prefix` succeeds by pure
  textual match wherever an absolute `#[path = "/…"]` literal's target happens to share
  the anchor's prefix, producing a relative-looking label in that checkout and falling
  back to the raw absolute path in another — the two checkouts' `unauditable-probe`
  identities differ. Risk: narrow — an absolute `#[path]` literal is already
  non-portable and machine-specific by construction; this only affects that one already
  fragile idiom, not the realistic relative sibling-share case PR #157 fixed. Promotion
  trigger: threading "was this file reached via an absolute `#[path]` literal" through
  the whole `resolve_path_module`/`external_module_files`/`collect_scope_modules`/
  `collect_reachable_probes` pipeline so such a file's label is never relativized at
  all — a separate, scoped refactor. Version class: DESIGN-BREAKING (changes
  un-auditable-probe identity shape for this construct). Authority: PR #157's commit
  body, `docs/history/0.3.1-adversarial-sweep.md`.

- **`InherentMethod` seam identity omits its declaring module, so two impl sites in
  different modules collapse to one violation.** Class: DESIGN-BREAKING. Observed
  pressure: verified real during 0.3.1 sweep cleanup (2026-08-02/03) — a type with
  inherent-method impl-trait-returning methods declared in two different modules
  (e.g. platform-split `plat_unix`/`plat_win` inherent impls on one shared type)
  produces exactly one violation instead of two; the second module's real violation is
  silently lost, not merely deduped. Observation source: direct reproduction against
  `hunyi::check_impl_trait` (and structurally the same gap in dyn-trait/signature-coupling's
  inherent-method seams, sharing the identical shape logic), recorded in
  `docs/history/0.3.1-adversarial-sweep.md`. Current bound: `PublicSeam::InherentMethod`
  is keyed on `{owner, name}` only — correct when distinguishing different self types on
  the same governed surface, but blind to the same self type's impl blocks written in
  separate modules (unlike `FreeFn`/`TraitMethod`, which already carry a module field).
  Risk: a false negative — PROJECT.md's Core Contract forbids this outright — but
  requires the specific idiom of splitting one type's inherent methods across modules
  (common in platform-conditional code, not yet observed as an adopter complaint).
  Promotion trigger: add a declaring-module/location field to `PublicSeam::InherentMethod`
  (and sibling `InherentAssoc`) and thread it through `inherent_method_seam`/
  `inherent_assoc_seam` — real design work (deciding whether the module belongs in the
  seam's identity only or also its rendered label). Version class: DESIGN-BREAKING
  (changes affected seams' baseline `finding_key` shape). Authority:
  `docs/history/0.3.1-adversarial-sweep.md`.

- **`InherentGenerics` seam identity has no per-block distinguisher, contradicting its
  own doc comment.** Class: DESIGN-BREAKING. Observed pressure: verified real during
  0.3.1 sweep cleanup (2026-08-02/03) — two separate inherent impl blocks on the same
  type, each exposing the same forbidden subject through a different where-clause
  bound, collapse to one violation. Observation source: direct reproduction against
  `hunyi::check` (`crates/hunyi/src/collect.rs`'s inherent-generics collector), recorded
  in `docs/history/0.3.1-adversarial-sweep.md`. Current bound: `PublicSeam::InherentGenerics`
  is keyed on `{owner}` only, despite its own adjacent doc comment claiming it stays
  "distinct... from another block's generics" — that claim does not hold; owner-qualification
  distinguishes different types, not different impl blocks of the same type. Risk: a
  false negative on a narrow idiom (multiple inherent impl blocks on one type, each with
  its own where-clause bound exposing a forbidden type) — not yet observed as adopter
  pressure. Promotion trigger: a real per-block distinguisher (an impl-block ordinal or
  a stable rendering of the block's own where-clause) added to `PublicSeam::InherentGenerics`
  — real design work, since a *stable* (not source-position-fragile) distinguisher is
  itself a design decision. Version class: DESIGN-BREAKING. Authority:
  `docs/history/0.3.1-adversarial-sweep.md`.

- **Trait-impl-locality's violation target/rule-key reads the constitution's declared
  trait spelling instead of the already-resolved canonical anchor.** Class:
  DESIGN-BREAKING. Observed pressure: verified real during 0.3.1 sweep cleanup
  (2026-08-02/03) — declaring the identical boundary via two different (but
  re-export-equivalent) spellings of the same trait produces two `ViolationId`s for the
  same real-world fact. Observation source: direct reproduction against
  `hunyi::check_trait_impl_locality`, recorded in `docs/history/0.3.1-adversarial-sweep.md`.
  Current bound: `target`/`rule_key` in `crates/hunyi/src/trait_impl.rs` are built from
  `canonical_path_str(&boundary.trait_path)` — raw-identifier stripping only, never
  re-export resolution — even though the same function already computes a
  re-export-resolved `true_anchors`/`canonical` value for match-decision purposes,
  unused for identity. Risk: real baseline-stability consequence — renaming a
  constitution declaration from a re-export spelling to the canonical spelling (a pure
  refactor, no code behavior change) silently flips every affected violation's identity
  and defeats the baseline. Promotion trigger: thread the already-computed resolved
  anchor into `target`/`rule_key` instead of the raw declared string — real design work,
  since `true_anchors` is a cfg-blind multi-candidate set and picking one deterministic
  canonical member for identity purposes is itself a design decision. Version class:
  DESIGN-BREAKING. Authority: `docs/history/0.3.1-adversarial-sweep.md`.

- **`OriginEntry::new` lets any code in the process self-assert an arbitrary runtime
  origin, defeating origin-based fail-closed confinement.** Class: DESIGN-BREAKING.
  Observed pressure: verified real during 0.3.1 sweep cleanup (2026-08-02/03) — a
  hand-built `OriginEntry::new(TypeId::of::<RogueAdapter>(), "loukehot::good", "RogueAdapter")`
  passed to `install` alongside genuine `register_origin!` entries produces zero
  reaction for a seam declared `.only_origins(["loukehot::good"])`, even though
  `RogueAdapter` never legitimately registered that origin. Observation source: direct
  reproduction against the real `louke::install`/`assert_boundary!` public API, recorded
  in `docs/history/0.3.1-adversarial-sweep.md`. Current bound: `OriginEntry` is a `pub
  struct` and `new` a fully `pub fn` taking a caller-supplied `origin: &'static str`
  with no field-level or capability-level constraint — directly contradicting
  `openspec/specs/runtime-origin-assertion/spec.md`'s own stated requirement that origin
  is "observed, not self-asserted... which the type cannot claim falsely without
  physically registering elsewhere." Risk: HIGH — this defeats the crate's core stated
  guarantee outright for any code sharing the process, not merely a narrow idiom;
  unlike the other five entries here, this is a capability gap in the trust boundary
  itself, not an identity-collision edge case. Verified that the obvious mechanical fix
  (`pub` → `pub(crate)` on `OriginEntry::new`) breaks the legitimate `register_origin!`
  macro path too, since `macro_rules!` visibility is checked at the macro's expansion
  site, not its definition site — a real Rust limitation, not an oversight. Promotion
  trigger: a `#[track_caller]`/`std::panic::Location`-based redesign of `OriginEntry::new`
  so the recorded origin is always the true call-site location rather than a
  caller-supplied string (achievable in pure std, consistent with 漏刻's `serde_json`-light
  constraint) — real design work, not mechanical, and touches the public DSL surface.
  Version class: DESIGN-BREAKING. Authority: `openspec/specs/runtime-origin-assertion/spec.md`,
  `docs/history/0.3.1-adversarial-sweep.md`.

### WATCH / ACCEPTED / DECLINED / BUILT

- **WATCH:**
  - Token/Lexer extraction (requires cross-scanner false negative or 3rd scanner).
  - `qianyi` generator & LSP/editor integration.
  - Baseline debt ratchet (`--require-baseline-reduction`, only-fix-never-add). This remains in
    tension with “baseline is a generated snapshot, not policy” and “not a governance platform”:
    a bounded opt-in gate may fit, while debt scheduling does not. Promote only after that tension
    is resolved with adopter pressure and an explicit observation/reaction design.
  - **`xuanji` sink for shared run/projection vocabulary.** Class: WATCH. Observed pressure:
    `pacta` consumes the composed facade while `modou` consumes and re-exports Guibiao's widened
    standalone check/baseline/projection surface. Observation source: those reference-consumer
    builds and the current crate dependency graph. Current bound: keep dimension-owned
    projection/check surfaces above vocabulary-neutral `xuanji`; sinking them now would break the
    proven standalone Guibiao consumer without demonstrated cross-dimension deduplication. Risk:
    duplicated shell vocabulary may grow if multiple instruments acquire standalone products.
    Promotion trigger: a second standalone instrument consumer (Hunyi or Louke) demonstrates the
    same projection/run need and makes a shared sink earn its migration cost. Version class:
    DESIGN-BREAKING. Authority: `PROJECT.md`'s crate-family and dimension-independence decisions,
    `openspec/specs/rule-model-surface/spec.md` reference-consumer contract, and the Pacta/Modou
    compatibility evidence recorded in `CHANGELOG.md`.
- **ACCEPTED DEBT:**
  - Multi-target conventional-path conflation.
  - `unsafe_confinement`'s and `trait_impl`'s `allowed_locations` tolerate a malformed `::`-path
    entry (an empty segment) rather than rejecting it. Observed alongside the
    `hunyi-forbidden-operand-colon-validation` fix for `must_not_expose`/`must_not_acquire`'s
    forbidden-operand family: `matches_allowed`/`path_within` has the identical shape of defect, but
    the failure DIRECTION is the safe one — a malformed allowed entry makes every real site look
    disallowed, producing spurious violations (fails loud, if noisily) rather than the silent,
    permanent non-reaction the forbidden-operand fix closes. Current bound: intentionally
    unvalidated for now, since the existing behavior already errs loud rather than silent. Risk: an
    adopter with a genuinely malformed `allowed_locations` entry gets confusing false-positive
    violations instead of a clear constitution error naming the typo. Promotion trigger: a reported
    or measured case of this confusion, or a bundled pass over every allowed-operand-shaped DSL
    method once one is warranted on its own terms. Version class: READY-PATCH (same
    check-time-`Result` mechanism, no signature change). Authority:
    `crates/hunyi/src/containment.rs`'s `path_within`, and
    `openspec/changes/hunyi-forbidden-operand-colon-validation/design.md`'s Non-Goals (pruned from
    `openspec/changes/` on sync; the reasoning is carried here instead).
  - Macro/configuration coverage bounds. Two named residuals after 渾儀 gained `cfg_if!`
    transparency: it covers **item position** only (an invocation inside an `impl`/`trait` body holds
    impl items, needing a parallel flattening across ~10 body walkers — measured, pinned by
    `a_cfg_if_inside_an_impl_body_is_a_stated_bound`), and 漏刻 has no transparency at all yet (its
    byte scanner reads macro bodies in two independent passes and would need 圭表's brace-kind model
    in both). Each is its own change with its own spike; `cfg_if_transparency_conformance.rs` states
    in its module doc that it pins two of three dimensions until 漏刻's lands.
  - File-granular un-auditable-probe identity.
- **DECLINED:**
  - Wall-clock auto-decay / auto-expiration (breaks determinism).
  - Trait method set freezing (API contract, not architectural shape).
  - Pre-creating empty crates/modules.
  - `GovernanceTest` standard-preamble generator API (`standard_preamble(test_name: &str)`
    or equivalent). Drift law: no API surface without adoption pressure. The preamble discipline
    (universals only, no architectural claims) is a documentation concern, not an API contract;
    `assert_projection_fresh_with_preamble` already accepts a caller-supplied `&str`. If three
    adopters independently write incorrect preambles containing architectural claims that then
    rot, revisit.
- **BUILT / HISTORY:**
  - Opt-in gate flag `--disallow-stale` enforcing zero stale baseline entries in CI gate mode.
  - Non-generic compound type alias target traversal in `hunyi` (tuples, arrays, slices, references, raw pointers).
  - Mid-path `super` and `self` module path normalization in `guibiao`.
  - `cfg_if!` macro expansion unstripping and ancestor-glob fail-closed observation in `guibiao`.
  - `cfg_if!` arm transparency in `hunyi` (item position: arm items collected, arm-declared modules
    walked, arm membership treated as cfg-conditional), pinned against `guibiao` on one shared
    fixture.
  - Configurable custom probe macro marker attributes in `louke`.
  - Three-layer agent-law artifact naming (Preamble + Projection + Law Source) and `COOKBOOK.md` recipe with `GovernanceTest` preamble discipline.
  - `cfg_attr(path)` observe-both semantics (union-scan over default and physically existing `cfg_attr(path)` target paths).
  - `guibiao`'s own module-boundary walk now tolerates a module backed only by one or more resolved `cfg_attr(path)` remaps (no plain conventional file, no direct `#[path]`), matching `hunyi`/`louke`'s identical rule for the same shape.
  - Reusable testing harness (`tianheng::testing::GovernanceTest` fluent builder in facade for reaction, coverage, projection freshness with `BLESS=1`, and fixture testing).
  - Self-governance observation depth upgrade (explicit ScanDepth declarations across self_governance.rs boundaries).
  - Detailed shipped capability ledgers for 0.1.x through 0.3.0 are archived in [`docs/history/0.1.0-0.3.0-built-ledger.md`](docs/history/0.1.0-0.3.0-built-ledger.md).

## Version horizons

The version follows SemVer honesty (`AGENTS.md`), not milestone size: **non-breaking →
patch, breaking → minor**, and never a vanity minor bump.

- **0.2.x (patch)** — additive depth on an existing observation source, false-negative closures.
- **0.3.0 (shipped)** — stable rule identity (`RuleKey`), `StructuredFactIdentity`, unsafe-site decomposition, async seam identity.
- **Next breaking window (if earned)** — requires real adopter or correctness pressure.

## Explicitly not on the roadmap

- Active code-shaping / generation.
- Prescriptive framework you build inside.
- Lints (opinionated style checks rather than declared intent).
- Universal graph API (whole-graph analysis rather than declared per-target boundaries).
- Supply-chain policy engine (cargo-deny's lane).
- DSL macro consolidation (repetitive builders are designed-to-be-imitated for 潛移 gravity; leave explicit).
