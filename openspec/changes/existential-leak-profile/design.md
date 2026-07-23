## Context

`ImplTraitBoundary` (RPIT — a *written* `-> impl Trait`) and `AsyncExposureBoundary` (an `async fn`'s
compiler-inserted `impl Future`) are `hunyi`'s two existential-leak signals — "shape" and
"implicit-existential" complements of each other, per their own doc comments. `BACKLOG.md` recorded a
"forward depth" wish to fold them under one `must_not_expose_existential` rule, deferred because
"the two syntactic signals stay distinct rules until a unification earns its own admission (it must
not blur the two findings' identities)."

An extended explore session re-examined this against the 0.3.0 semantic-identity model and found the
blocker dissolves once the unification target is corrected.

## Goals / Non-Goals

**Goals:**
- Give an adopter one declaration for "no existential type may leak from this seam," covering both
  signals.
- Preserve fully independent identity for each signal — two distinct causes remain two distinct,
  separately-baseline-able findings.
- Close the one real gap the composed profile's own honesty requires: `ImplTraitBoundary` gaining
  subtree scope.
- Encode both non-trivial facts (subtree symmetry; the operand-scoping non-goal) as tests, not
  prose — per this project's own Core Contract (a declared boundary reacts) and its "not a lint"
  scope statement (a purely behavioral, single-function consistency fact does not belong in the
  Tianheng constitution DSL; an ordinary `cargo test` is the correct, sufficient mechanism).

**Non-Goals:**
- A fused `Rule`/fact type merging impl-trait and async identity into one. Rejected — see Decision 1.
- Subtree scope for `DynTraitBoundary`. No demonstrated consumer needs it in this change; the drift
  law ("no capability before its need is demonstrated") applies even though the underlying
  `ShapeExposure` collector is already shared with impl-trait and the addition would likely be cheap.
- Operand-scoping for `AsyncExposureBoundary`. Its compiler-inserted `impl Future` is invariant —
  there is no written principal-trait text for the existing operand-resolution mechanism to
  target. See Decision 3.
- A new Tianheng self-governance boundary for anything in this change. See Decision 4.

## Decisions

### 1. A shell-composed profile (mirroring `sans_io_pure`), not a fused Rule

`crates/tianheng/src/sans_io.rs`'s `SansIoPure` already proves the target shape: one adopter-facing
declaration (`Constitution::sans_io_pure(...)`) expands into two fully independent boundaries — a
圭表 `must_not_call_inline` and a 渾儀 `must_not_expose_async_fn` — each keeping its own `rule_key`
and fact identity. Its own doc comment: "adds no new reaction... a convenience over declaring the
two boundaries by hand."

Applying the same shape here removes the identity-blur risk entirely, because nothing about identity
changes. This is also the more *correct* target, not merely the safer one: an RPIT leak and an
async-fn leak are, in this project's own repeatedly-demonstrated identity philosophy (injective
identity — never collapse distinct causes; see the `unauditable-probe-identity` change for the
freshest example), genuinely two different problems with two different fixes. A fused Rule would
have had to either invent a shared fact shape distinguishing them by an internal `shape` field
anyway (in which case nothing is actually "unified" except the DSL entry point — the same value a
composed profile delivers for far less risk) or genuinely collapse them (reintroducing the
forbidden-bug risk the original deferral was right to avoid). `must_not_expose_existential`'s
"fold... under one rule" framing conflated "one declaration" with "one identity"; only the former is
the real adopter need.

**Alternative considered — a fused `Rule` type with an internal shape discriminator:** rejected.
Even granting careful design, it buys nothing a composed profile doesn't already deliver, while
adding real identity-design surface (a new fact_type, new compatibility-catalog entries, a new
`RuleKey` shape) for a capability that both correctness (see above) and cost favor keeping split.

### 2. Structural home: `crates/tianheng`, not `crates/hunyi`

`Constitution` — the only type that assembles multiple boundaries together (`.impl_trait_boundary(...)`,
`.async_exposure_boundary(...)`) — is defined in `crates/tianheng/src/lib.rs`. `guibiao` has its own,
narrower, static-only `Constitution` (`crates/guibiao/src/model.rs`); `hunyi` has none. `hunyi`'s
self-law (`self_governance.rs`) restricts its dependencies to `xuanji, xingbiao, serde_json, syn` —
never `tianheng`. `hunyi` is therefore structurally unable to define any method on
`tianheng::Constitution` (dependency direction, not the orphan rule, since the type itself is
foreign either way). This is a dependency fact, not a style choice, and settles the question
definitively in favor of `crates/tianheng/src/existential.rs` — independent of whether 三儀 ⊥ 三儀
would otherwise have allowed a within-dimension convenience to live in `hunyi` (it is not violated
either way, since impl-trait and async-exposure are both `hunyi` capabilities, not sibling
dimensions — but the placement question is moot regardless, because `Constitution` itself is
unreachable from `hunyi`).

### 3. `ImplTraitBoundary` gains subtree scope; `DynTraitBoundary` and async operand-scoping deliberately do not

Neither `DynTraitBoundary` nor `ImplTraitBoundary` has subtree scope today; only
`AsyncExposureBoundary` does — it is the outlier, gained specifically because `sans_io_pure`'s own
use case ("the pure kernel throughout, not only at its own seam") demanded it. This composed profile
has the identical need on its impl-trait half: without subtree scope there, the profile would ship a
silently half-true "no existential leak in this subtree" claim (the impl-trait half checking only the
anchor module) — a real false-negative risk, not a cosmetic asymmetry. So `ImplTraitBoundary` gains
`including_submodules()`, mirroring `AsyncExposureBoundary`'s DSL field, `rule_key()` field, and
reaction-side subtree walk (reusing `walk_subtree_modules`) exactly. `DynTraitBoundary` does **not**
gain it in this change: no composed profile or other demonstrated consumer needs it yet, and adding
it speculatively — even though the underlying `ShapeExposure` collector is already shared with
impl-trait, making it likely cheap — is exactly what the drift law exists to prevent. It remains a
`BACKLOG.md` candidate, promoted only when a real second consumer demonstrates the need.

Symmetrically, `AsyncExposureBoundary` does **not** gain operand-scoping. `must_not_expose_impl_trait_of`/
`must_not_expose_dyn_of` resolve a *written* principal-trait path against a forbidden set; an async
fn's `impl Future` is compiler-inserted and invariant — there is no written text for that mechanism
to resolve against. Scoping by the `Future`'s associated `Output` type would be a fundamentally
different, deeper capability (semantic inspection of the return type's inner type), which neither
dyn nor impl-trait attempt today either. This is a stated non-goal, not a gap.

### 3a. The new subtree path must route through the same fail-loud gate the single-module path already uses — correction after adversarial review

Impl-trait's owner resolution (`canonical_self_owner`, `resolve/shape.rs:397`) is the
sentinel-producing kind: an unrenderable self type yields an internal `_#{ordinal}` label, caught
downstream by the shared `reject_positional_identity` gate (invoked inside `sort_faceted_facts` /
`sort_attributed_facts`, `finding.rs`). Impl-trait's *existing* single-module path
(`shape_module_findings`, `shape_scan.rs:72`) already calls `sort_faceted_facts` internally, so this
gate is already wired for it. The *new* subtree path is genuinely new code — `shape_scan.rs` has no
existing multi-module counterpart to `shape_module_findings` the way `async_exposure.rs` supplies
its own subtree function — and must explicitly call `sort_attributed_facts` on its collected
`Vec<(SemanticFact, String, PathBuf)>` before returning, mirroring
`async_exposure.rs:122`'s `sort_attributed_facts(&mut findings)?;` exactly. This is not automatic:
`push_multi_module_violations` itself does not call the reject-gate — the caller must. A
naive implementation that copies async's subtree *shape* without copying this specific call would
silently skip the gate for impl-trait's subtree path alone, reopening exactly the
never-publish-positional-identity guarantee this session's earlier exploration confirmed the
project treats as load-bearing.

The ordinal value passed into `collect_item_return_impl_traits` across the subtree walk must also
be threaded continuously across every module the walk visits (never hardcoded `0` for every call,
which async's own subtree function can get away with only because its collector ignores the
parameter entirely — impl-trait's does not). Getting the exact ordinal value right does not change
whether `reject_positional_identity` fires (it fires on any sentinel-bearing fact regardless), but a
non-unique ordinal risks two genuinely distinct unrenderable sites producing byte-identical sentinel
facts that a subsequent, unrelated dedup pass could not tell apart — thread it correctly rather than
relying on the gate alone to paper over it.

A new scenario belongs in the spec delta for this — see the updated
`specs/semantic-impl-trait-boundary/spec.md`. Its outcome is **not** the base async spec's own
"two distinct violations" wording (that scenario's prose does not match its own implementation
either — `tests.rs`'s `async_cfg_branches_never_share_an_unrenderable_owner_fallback` actually
asserts a scan error, a pre-existing wording drift in the *already-shipped* async spec, out of
scope for this change): impl-trait's actual mechanism aborts the whole check with a constitution
error (exit 2) the moment any unrenderable owner is observed, so the new scenario must describe
that outcome accurately, not copy the imprecise wording forward into a second spec.

### 4. No new self-governance boundary; ordinary tests carry both facts

`self_governance.rs` reacts to crate-dependency/import facts (e.g. "guibiao must not depend on
tianheng," the 三儀 ⊥ 三儀 mutual-independence law). This entire change sits within one
already-permitted dependency edge (`tianheng` → `hunyi`); nothing about it is a new crate-dependency
fact for self-governance to react to. The two facts this design must protect — the composed
profile's subtree symmetry, and async's operand-field absence — are both about one function's
internal behavioral consistency, not an architectural dependency fact. `PROJECT.md`: "it is not a
lint." Manufacturing a Tianheng constitution boundary for either would misapply the DSL to a fact it
was never meant to express. Both are instead carried as ordinary `cargo test` assertions (Decision 5).

### 5. Encode as reactions: a faithful-composition test and an exhaustive schema test

Mirroring `sans_io.rs`'s own `mod tests` (`sans_io_pure_composes_faithfully`,
`sans_io_pure_threads_severity_to_both`, `sans_io_pure_bakes_no_defaults`):

- `no_existential_leak_composes_faithfully` (and severity/no-baked-defaults siblings): a
  `hand_composed(...)` reference explicitly constructs both boundaries with `.including_submodules()`
  called on each; `via_profile(...)` uses the new composed API;
  `assert_eq!(constitution_markdown(&via_profile(...)), constitution_markdown(&hand_composed(...)))`.
  Also matches `sans_io_pure`'s own convention of hardcoding the subtree opt-in inside the profile's
  expansion rather than exposing it as an adopter choice — so there is only one place (the profile's
  own function body) where the symmetry could ever silently break, and this test is the reaction
  that catches it if it does.
- An exhaustive `rule_key`-shape test proving `AsyncExposureBoundary::rule_key()`'s field list is
  exactly `["including_submodules"]`. **Correction after adversarial review: this test already
  exists** — `crates/hunyi/src/tests.rs:3384`'s `every_hunyi_rule_family_has_exact_semantic_identity`
  already asserts exactly this (`tests.rs:3460-3469`), exhaustive by construction (`assert_eq!` on
  the full field vec — an added field breaks the equality). No new test is needed for async. What
  genuinely IS needed: that same test's existing `ImplTraitBoundary` block (`tests.rs:3415-3423`,
  currently asserting only `[("forbidden_operands", ...)]`) must be updated once
  `including_submodules` is added to `ImplTraitBoundary::rule_key()` (Decision 3) — otherwise this
  very test starts failing the moment task 1.2 lands, for the right reason (the schema genuinely
  changed) but for a task the original draft of this design failed to name.

### 6. Addressing two objections an adversarial review raised directly, rather than leaving them unargued

**Is manufacturing the consumer and the capability in one change the circularity the drift law
exists to prevent?** No, and there is a direct precedent for why not:
`AsyncExposureBoundary::including_submodules` itself — the exact feature this change mirrors — was
"gained specifically because `sans_io_pure`'s own use case... demanded it" (Decision 3). That
precedent's justification was *also* purely an in-repo composed profile's need, not independent
external adopter pressure. This change does not apply a looser standard than the one already used
to ship async's own subtree scope; it applies the identical one. The drift law's concern is
capability invented with **no** consumer (a name with nothing to react to); a profile that is
itself real, shipped code with its own tests is a real consumer, not a hypothetical one, even though
it is introduced in the same change as the depth it requires.

**Does this skip `BACKLOG.md`'s "Backlog governance — evidence before promotion" checklist (class,
observed pressure, observation source, trigger, etc.)?** That checklist governs promotion out of
the **Live decision index** (READY-PATCH / DESIGN-BREAKING / WATCH / ACCEPTED DEBT / DECLINED).
`must_not_expose_existential` was never in that index — it sits under 渾儀's "Forward depths"
ledger, the same informal category as `UnsafeBoundary`'s subtree confinement and the visibility
ceiling, both of which shipped as ordinary BUILT capabilities when a concrete design and consumer
materialized, without walking that newer checklist either. This change follows that same,
longer-standing "Forward depths" convention, not the newer evidence-gated one — consistent with
existing practice in this exact ledger section, not an exception carved out for this change.

## Risks / Trade-offs

- **[Risk]** New subtree-walk code in `crates/hunyi/src/impl_trait.rs` duplicates
  `async_exposure.rs`'s shape rather than sharing it, reintroducing the same class of twin-drift this
  session's other findings warned against.
  **Mitigation:** model it on `async_exposure.rs`'s existing branch for the two genuinely
  collector-agnostic, reusable helpers (`walk_subtree_modules`, `push_multi_module_violations`) — but
  note the two per-item collectors are **not** structurally parallel:
  `collect_item_return_impl_traits` returns `()` into a `&mut Vec<ShapeExposure>` and needs an
  additional `shape_finding(exposure, ExposureKind::ImplTrait)` render step (per
  `shape_scan.rs:43-44`'s own note that async's path skips this render step entirely), while
  `collect_item_async_exposures` returns `Result<(), String>` directly into `&mut Vec<SemanticFact>`.
  The new impl-trait subtree function is **not** a byte-for-byte copy of `async_exposure_subtree_findings`
  — its loop body differs — and must explicitly call `sort_attributed_facts` itself (Decision 3a). If
  the two subtree-walk bodies end up substantially identical after accounting for this, note it as a
  candidate for a shared helper in a follow-up — but do not block this change on that extraction
  (born-when-built; a third consumer, or clear duplication observed once both exist side by side, is
  the trigger, not speculation now).
- **[Risk]** `ImplTraitBoundary` gaining a new `rule_key()` field changes its projected identity
  shape (adds a field), which could look like a breaking baseline change.
  **Mitigation:** mirror `AsyncExposureBoundary`'s own precedent exactly — the field defaults to
  `false`/off, and the existing `semantic-async-exposure-boundary` spec's requirement states the
  default path stays byte-identical; the same guarantee applies here and must be proven by a
  scenario mirroring "A submodule async fn the seam scope misses reacts under the opt-in" /
  the byte-identical-default scenarios in that spec.
- **[Trade-off]** `DynTraitBoundary` remains without subtree scope after this change, so an adopter
  wanting "no dyn anywhere in this subtree" still cannot get it in one call. Accepted: no demonstrated
  need yet; recorded as a live `BACKLOG.md` candidate, not silently dropped.

## Open Questions

- Exact rustdoc example placement/wording for `NoExistentialLeak` (mirror `SansIoPure`'s doc-comment
  structure and its ````no_run``` example) — cosmetic, resolved during apply.
- (Noted, not fixed here) `semantic-async-exposure-boundary/spec.md`'s existing "Two cfg-split
  branches sharing an unrenderable owner fallback stay distinct findings" scenario says "THEN the
  system reports TWO distinct violations," but the actual test
  (`async_cfg_branches_never_share_an_unrenderable_owner_fallback`) asserts a scan error — a
  pre-existing wording drift in the already-shipped async spec, found while mirroring it for
  impl-trait. Out of scope for this change (a separate, narrow doc-fix); the new impl-trait scenario
  in this change's spec delta describes the correct (scan-error) outcome rather than propagating the
  drift.
