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

Measured defects awaiting repair live in [`docs/audit/0.3.1-adversarial-sweep.md`](docs/audit/0.3.1-adversarial-sweep.md)
— 22 verified plus 14 unverified findings from a two-round adversarial sweep of
`v0.2.3..release/0.3.1`, each sized to become one OpenSpec change, with the provenance and the
trust level of each entry stated there. That file is a **queue**, not a decision index: an entry
leaves it by being fixed (or by failing to reproduce, struck with a note). This section is only its
pointer, so the index below stays about *capability* decisions.

## Live decision index

### DESIGN-BREAKING

None currently live — the `0.3.0` identity migration closed the prior candidate.

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
