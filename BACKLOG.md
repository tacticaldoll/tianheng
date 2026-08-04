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

No live queue currently. The `v0.2.3..release/0.3.1` adversarial sweep that populated this section
is fully closed: every finding reached a terminal state — 11 fixed (each its own `change/*` + PR,
cited in the affected code's history and `CHANGELOG.md`), 2 verified moot, 2 refuted, 6 promoted to
live decisions below, and its own prior two-round sweep's 6 refuted / 6 upheld-by-only-one-lens
findings absorbed into `DECLINED`/`WATCH` below (condensed deliberately rather than kept verbatim,
to avoid a future reader anchoring on a raw agent-verdict's specific phrasing instead of the settled
conclusion). The sweep's own working queue file is not retained once fully drained — its substance
now lives here and in the closing PRs, not in a file kept only because it once existed. A future
sweep gets its own dated `docs/audit/*.md` queue file and its own pointer here.

## Live decision index

### DESIGN-BREAKING

- ~~**Owner-label identity collapses across a cfg-collided self-type alias.**~~ **CLOSED** in the
  0.4.0 window, after an independent review re-derived it. Closed by refusing to name the ambiguity
  rather than by inventing an encoding for it: an owner role whose head admits more than one distinct
  candidate now reaches the shared fail-loud identity gate (a constitution error, exit 2, naming the
  `#[cfg]` collision), because neither candidate may be preferred and the candidate SET is identical
  for both colliding sites, so nothing available separates them. "Cannot judge" over a silent collapse
  is the Core Contract's own ordering. The single-candidate `resolve_path` that fed every such label
  was **deleted**, not merely bypassed — `resolve_path_all` is now the only resolver, so a caller
  needing one value must decide what to do about `len() > 1` instead of receiving an arbitrary pick,
  and the defect class is unrepresentable rather than fixed at three sites. `semantic-signature-coupling`
  gains the requirement and two scenarios. The original entry is kept below for its reproduction record.

- **Owner-label identity collapses across a cfg-collided self-type alias.** Class:
  DESIGN-BREAKING. Observed pressure: reproduced by the maintainer during round-3
  adversarial review of `change/hunyi-cfg-branch-use-reexport-merging` (PR #149) —
  two genuinely independent violations sharing one cfg-collided self-type alias
  (`#[cfg(unix)] use crate::a::Foo as X; #[cfg(not(unix))] use crate::b::Bar as X;`,
  each arm implementing the same governed trait) render the identical single-candidate
  owner label and collapse to one finding under exact-identity dedup, across
  trait-impl-locality, forbidden-marker, unsafe-confinement, and signature-coupling at
  once. Observation source: that review's reproduction, described above. Current bound:
  `canonical_self_owner`/`canonical_self_owner_without_fallback`
  (`crates/hunyi/src/resolve/shape.rs`) and `canonical_unsafe_owner`
  (`crates/hunyi/src/scan/unsafe_sites.rs`) each render a label from a single resolved candidate,
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
  violation-identity decisions, PR #149's commit body (which names this exact gap and
  scopes it out as a follow-up).

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
  body; the stated bound is also pinned in `crates/louke/src/finding.rs`/`audit.rs`'s own
  doc comments and `openspec/specs/runtime-origin-assertion/spec.md`.

- ~~**`InherentGenerics` seam identity has no per-block distinguisher WITHIN one module.**~~ **CLOSED**
  in the 0.4.0 window. The seam now carries the **bounded thing** each exposure sits on — a parameter's
  own name, or a where-predicate's rendered bounded type — so two blocks in one module bounding
  different parameters to the same forbidden type are two distinct facts. The distinguisher this entry
  called a design decision is resolved by taking the one the codebase already had: it is keyed exactly
  like a trait `impl`'s own `where` position, and both now come from one shared walk over an impl's
  generics positions, so the vocabularies cannot drift. An impl-block **ordinal** was ruled out rather
  than chosen against, since `semantic-signature-coupling` forbids identity resting on scan order or
  item ordinal.
  What remains is a limit, not a gap, and is stated in the seam's own doc: two blocks whose bounds are
  *textually identical* still resolve to one seam. Nothing structural distinguishes them — their
  contents carry their own seams and position is forbidden — so two blocks bounding the same parameter
  to the same forbidden type state one architectural fact twice, exactly as one import on two lines is
  one violation. Completeness of the position walk rests on a language rule verified against a real
  `rustc` rather than assumed: an `impl`'s generic parameters cannot carry defaults, so a parameter
  contributes only its bounds (or, for a const parameter, its type). The original entry is kept below
  for its reproduction record.

- **`InherentGenerics` seam identity has no per-block distinguisher WITHIN one module.**
  Class: DESIGN-BREAKING. Observed pressure: verified real during
  0.3.1 sweep cleanup (2026-08-02/03) — two separate inherent impl blocks on the same
  type, each exposing the same forbidden subject through a different where-clause
  bound, collapse to one violation. Observation source: direct reproduction against
  `hunyi::check` (`crates/hunyi/src/collect/exposure.rs`'s inherent-generics collector) —
  described above. Current bound: `PublicSeam::InherentGenerics` is keyed on
  `{module, owner}`. The **cross-module** half of this entry is CLOSED — the module role was
  added in the 0.4.0 window, after an independent review re-derived it, and the adjacent doc
  comment that wrongly claimed owner-qualification alone kept the seam "distinct... from
  another block's generics" was corrected in the same change. What remains is strictly
  narrower: two impl blocks for the same owner **in the same module** still resolve to one
  seam, since owner-plus-module distinguishes where an impl is written, not which block it is.
  Risk: a false negative on a narrow idiom (two inherent impl blocks on one type in one
  module, each with its own where-clause bound exposing a forbidden type) — not yet observed
  as adopter pressure, and one step narrower than when this entry was written. Promotion
  trigger: a real per-block distinguisher added to `PublicSeam::InherentGenerics` — real
  design work, and constrained rather than open: `semantic-signature-coupling` forbids
  identity from resting on "scan order, item ordinal, and renderer fallback position", so an
  impl-block ordinal is NOT available as the distinguisher and a stable structural key (a
  rendering of the block's own generics, say) has to be designed and its collision behavior
  argued. Version class: DESIGN-BREAKING. Authority: this entry's own reproduction record
  (above); `openspec/specs/semantic-signature-coupling/spec.md`'s ordinal prohibition for the
  constraint on the trigger.

- ~~**Trait-impl-locality's violation target/rule-key reads the constitution's declared trait
  spelling instead of the already-resolved canonical anchor.**~~ **CLOSED** in the 0.4.0 window, after
  an independent review re-derived it. Both roles are now keyed on the resolved anchor, threaded out of
  the function that already computed it rather than recomputed (a second resolution site could
  disagree with the one that decided the matches). The multi-candidate question this entry named as the
  reason it was design work is answered by refusing rather than picking: a declared anchor whose
  re-export closure reaches more than one distinct local trait DEFINITION is a constitution error
  naming the candidates, since the ambiguity is in the declaration and choosing would make the
  governed target arbitrary. A declaration that already names the defining path — the ordinary case —
  is unchanged, so the fix churns only the baselines it must. `allowed_locations` deliberately stays in
  the rule key (it keeps two boundaries on one trait distinct) and the in-code comment that wrongly
  claimed it was NOT identity-bearing is corrected: `ViolationId` compares `rule_key` in full.
  `semantic-trait-impl-locality` gains all three rules and two scenarios. The original entry is kept
  below for its reproduction record.

- **Trait-impl-locality's violation target/rule-key reads the constitution's declared
  trait spelling instead of the already-resolved canonical anchor.** Class:
  DESIGN-BREAKING. Observed pressure: verified real during 0.3.1 sweep cleanup
  (2026-08-02/03) — declaring the identical boundary via two different (but
  re-export-equivalent) spellings of the same trait produces two `ViolationId`s for the
  same real-world fact. Observation source: direct reproduction against
  `hunyi::check_trait_impl_locality` — described above.
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
  DESIGN-BREAKING. Authority: this entry's own reproduction record (above).

- **`OriginEntry`'s public constructor lets any code in the process assert an arbitrary runtime
  origin, defeating origin-based fail-closed confinement.** (Spelled `OriginEntry::new` when this
  entry was written; renamed `__from_register_origin` in the 0.4.0 window — see the update below.)
  Class: DESIGN-BREAKING.
  Observed pressure: verified real during 0.3.1 sweep cleanup (2026-08-02/03) — a
  hand-built `OriginEntry::new(TypeId::of::<RogueAdapter>(), "loukehot::good", "RogueAdapter")`
  passed to `install` alongside genuine `register_origin!` entries produces zero
  reaction for a seam declared `.only_origins(["loukehot::good"])`, even though
  `RogueAdapter` never legitimately registered that origin. Observation source: direct
  reproduction against the real `louke::install`/`assert_boundary!` public API —
  described above. Current bound: `OriginEntry` is a `pub
  struct` and its constructor a `pub fn` taking a caller-supplied `origin: &'static str`
  with no field-level or capability-level constraint — which, when this entry was written,
  directly contradicted `openspec/specs/runtime-origin-assertion/spec.md`'s own stated requirement
  that origin is "observed, not self-asserted... which the type cannot claim falsely without
  physically registering elsewhere" (that absolute claim is what the 0.4.0 window retired; the
  capability gap it described is what stays open). Risk: HIGH — this defeats the crate's core stated
  guarantee outright for any code sharing the process, not merely a narrow idiom;
  unlike the other five entries here, this is a capability gap in the trust boundary
  itself, not an identity-collision edge case. Verified that the obvious mechanical fix
  (`pub` → `pub(crate)` on that constructor) breaks the legitimate `register_origin!`
  macro path too, since `macro_rules!` visibility is checked at the macro's expansion
  site, not its definition site — a real Rust limitation, not an oversight. Promotion
  trigger: a `#[track_caller]`/`std::panic::Location`-based redesign of `OriginEntry::new`
  so the recorded origin is always the true call-site location rather than a
  caller-supplied string (achievable in pure std, consistent with 漏刻's `serde_json`-light
  constraint) — real design work, not mechanical, and touches the public DSL surface.
  Version class: DESIGN-BREAKING. Authority: `openspec/specs/runtime-origin-assertion/spec.md`'s
  origin-observation requirement, whose absolute form this gap directly contradicted; this
  entry's own reproduction record (above) for the rest.
  **Updated in the 0.4.0 window** (the gap itself stays OPEN): the promotion trigger recorded above —
  a `#[track_caller]`/`std::panic::Location` redesign — **does not work as written**, verified rather
  than reasoned: `Location` yields a *file path*, while an origin's whole vocabulary is a module path
  (`register_origin!` captures `module_path!()`, and `only_origins(["app::infra"])` is declared in the
  same terms). Adopting it would redefine what an origin is and invalidate every existing declaration,
  which is a different change from the one this entry describes.
  **A second recorded path also does not work**, and it was recorded by this same window before being
  tested: a **proc-macro** does NOT let the constructor become private. A proc-macro is expanded into
  its caller's crate and resolved there, exactly as a `macro_rules!` is, so every item its expansion
  names must be reachable from the call site. Verified with a three-crate probe (proc-macro crate + a
  lib holding a `pub(crate)` constructor + a consumer): the failure is `error[E0603]: function
  `hidden_constructor` is private`, reported at the **consumer's** call, not at the macro. A macro form
  has no privilege its caller lacks, which is the same structural reason the `Location` trigger fails.
  Both dead ends share one shape — looking for a *macro* that can pass something hand-written code
  cannot — so no third macro variant is worth recording either.
  What CAN close it is a mechanism that never takes the origin from the caller, since
  `std::any::type_name` reports the type's **own** defining path. Two forms, differing in which cost is
  paid: (a) **redefine** an origin as "where defined" — unforgeable by construction, at the cost of
  invalidating every `only_origins` declaration and resting *identity* on a format std documents as
  unspecified (measured: a type in `rogue` reports `crate::rogue::Repo` while the registration site's
  `module_path!()` reported only `crate`); (b) **cross-check** the asserted origin against the type's own
  defining path at `install` and react to a disagreement, keeping the origin's meaning and every
  declaration intact — mechanically available, since `module_path!()` at a type's defining module equals
  `type_name` minus its trailing type name (measured: `tn_probe::internal` vs
  `tn_probe::internal::Repo`, and they differ exactly when the registration claims a module the type
  does not live in) — at the cost of resting a fail-loud gate on that unspecified format, so a toolchain
  rendering it differently would react against correct adopter code, plus a narrowing of today's
  behaviour (registering a type from anywhere but its own module becomes an error). What DID land: the constructor is `#[doc(hidden)]` and renamed
  `__from_register_origin` so a hand-written call reads as the bypass it is (it cannot be made private —
  `macro_rules!` visibility is checked at the expansion site, as this entry already recorded); the
  spec's claim that a type "cannot claim falsely" is corrected to state the process trust boundary —
  on every surface, not only in the requirement's body (the Purpose summary, the requirement's own
  name, the crate README, and the `register_origin!` doc each carried the absolute form for one more
  round), with `the_origin_guarantee_is_never_summarized_as_absolute` now reacting in both directions
  so the agreement is checked rather than hand-maintained; and the residual itself is pinned by
  `a_hand_built_origin_entry_is_accepted_a_known_trust_bound`, so it cannot change state in either
  direction unnoticed.

### READY-PATCH

- **An inbound rule's target match is namespace-blind, so a name declared in two namespaces at
  once is observed only as its module.** Class: READY-PATCH (the correction only adds reactions;
  no public API and no identity shape changes). Observed pressure: raised by an independent review
  of the 0.4.0 window against `crates/guibiao/src/module_check.rs`'s `resolve_import_module`, then
  verified against **rustc** rather than reasoned — a crate declaring `pub mod foo;` and
  `pub fn foo()` in one module compiles, and a single `use crate::internal::foo;` in another module
  binds both bindings (the importer can call `foo()` *and* reach `foo::INSIDE` from that one
  `use`). Observation source: that build, plus the pinned test
  `shallow_inbound_target_match_is_namespace_blind_a_stated_bound`
  (`crates/guibiao/src/tests/symbol_confinement.rs`). Current bound: the resolver sees only the
  path, so it returns the longest reachable module — the module reading — and the value reading is
  unobserved. The two disagree only under `ScanDepth::Shallow` anchored at that module's own
  parent: the value reading reaches the anchored module and should react, the module reading
  resolves to the descendant and does not. Under `Subtree` both readings lie within the anchored
  module, so nothing is lost. Risk: LOW-but-real — a false negative, the one class the Core
  Contract forbids reacting silently in, confined to one exotic shape (a module and a value
  sharing a name in the protected module) in one depth cell. Deliberately NOT closed by reacting
  on both readings: that makes every ordinary `use m::child;` react under `Shallow`, contradicting
  `rule-model-surface`'s own exact-seam scenario — a narrow false negative must not be traded for
  a broad false positive. Promotion trigger: a value-namespace item observation for the anchored
  module (`fn`/`const`/`static` declared directly in it, inline `mod` bodies included), so both
  readings are unioned exactly when the ambiguity is real; note it carries its own bounds a
  macro-generated or `pub use`-re-exported `fn` would still escape, which must be stated with it.
  Version class: PATCH. Authority: `openspec/specs/rule-model-surface/spec.md`'s stated
  namespace-blind bound and its scenario; the rustc verification and pinned test above.

### WATCH / ACCEPTED / DECLINED / BUILT

- **WATCH:**
  - Token/Lexer extraction (requires cross-scanner false negative or 3rd scanner).
  - `qianyi` generator & LSP/editor integration.
  - A `#[cfg_attr(pred, path=…)]` remap on an **inline** `mod name { … }` (not a file-backed
    module): the relocated child base is collected in `declarations.rs` but then discarded by
    `walk.rs`, so the arm's real children may never be scanned. Observed once during the 0.3.1
    adversarial sweep, upheld by exactly one verification lens and never independently
    re-tested or refuted — treat as a hypothesis, not a confirmed defect.
    (`crates/guibiao/src/module_scan/reachability/walk.rs`.) Promotion trigger: reproduce
    directly against the real entry point before acting; if confirmed, likely mechanical
    (the collected base already exists, just needs to survive into `walk.rs`'s consumption).
  - A symlinked source subdirectory may silently disappear from 漏刻's directory-mode audit
    while 圭表/渾儀 continue to govern the same subtree — `try_visit`'s cycle guard possibly
    bypassed by a second, weaker guard specific to the audit walker. Observed once during the
    0.3.1 adversarial sweep, single lens, never independently re-tested.
    (`crates/louke/src/audit/scan/probes.rs`.) Promotion trigger: reproduce directly (a real symlinked
    subdirectory containing a probed seam) before acting.
  - `xingbiao::crate_root_file` may collapse a multi-root package (a manifest declaring more
    than one crate root, e.g. via `[[bin]]`/`[lib]` combinations) to a single resolved root, so
    every non-first root of that package would be silently ungoverned by 圭表 and 渾儀. Observed
    once during the 0.3.1 adversarial sweep, single lens, never independently re-tested — the
    trigger shape (a real multi-root package in this workspace or an adopter's) was not
    confirmed to actually arise from `cargo metadata`'s output. (`crates/xingbiao/src/lib.rs`.)
    Promotion trigger: confirm `cargo metadata` actually emits multiple root files for one
    package before treating this as more than speculative.
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
  - Macro/configuration coverage bounds. **One** named residual now that all three dimensions read
    `cfg_if!` arms: transparency covers **item position** only — an invocation inside an
    `impl`/`trait` body holds impl items, needing a parallel flattening across ~10 body walkers
    (measured, pinned by `a_cfg_if_inside_an_impl_body_is_a_stated_bound`). 漏刻's own lack of
    transparency, listed here as the second residual, is CLOSED: its byte scanner reads arm contents
    as real code in both passes, and `cfg_if_transparency_conformance.rs` now pins all three
    dimensions on the one fixture rather than two of three — its module doc already says so, and this
    entry was the site left behind.
  - File-granular un-auditable-probe identity.
  - **The baseline directory flush has no reacting test, by construction.** `sync_parent_dir` is
    infallible and best-effort on purpose — a platform or filesystem that cannot flush a directory
    must not turn an already-landed write into a reported failure — which leaves it with no
    externally observable behavior for a test to bind. Measured rather than assumed: `cargo mutants`
    over `runner.rs` reports both of its mutants (`replace sync_parent_dir with ()`, `delete !`) as
    MISSED, while the mutants for the flag and zero-length rules beside it are caught. Its evidence
    is the syscall sequence recorded in the PR that added it (`fsync(temp)` → `rename` →
    `fsync(dir)`), not a gate. Accepted because the alternative — making it fallible, or counting
    failures the way 漏刻's `dropped_sink_events` does — either reintroduces the regression it exists
    to avoid or adds an adopter-visible counter for a CLI-side hygiene step nobody can read. Revisit
    only if a directory-flush regression is ever actually observed. The *strict* half of the
    guarantee (the file flush) is not in this bound: it is covered by the 18-test `baseline_cli`
    suite.
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
  - "`.github/CODEOWNERS`'s amendment reaction is unenforced" (0.3.1 sweep). Self-refuted by its
    own citation: the file's own disclaimer at the cited line already states plainly that
    designation alone only auto-requests review, and that branch protection must be separately
    enabled to make it binding — not a hidden gap.
  - "`ci.yml`'s Release-coherence job is the one Definition-of-Done gate CI runs but branch
    protection doesn't enforce" (0.3.1 sweep). Observations reproduced accurately, but none
    constitutes a defect under this project's own contract taxonomy — CI running a check and
    branch protection gating a merge are different, both-documented mechanisms, not a claimed
    single guarantee.
  - "漏刻 never diagnoses a module cycle (a circular `#[path]`/symlinked module directory
    silently collapses instead of exit 2)" (0.3.1 sweep). Refuted: a test already pins the exact
    shape, asserting the opposite of the claim — a documented, deliberately-pinned bound, not an
    undetected divergence.
  - "The self-law giving `xingbiao` its canonicalization monopoly only reacts to the
    free-function form; `path.canonicalize()` (the method call) escapes" (0.3.1 sweep). The
    mechanism reproduces, but it is an explicitly declared, spec'd, and test-pinned observation
    bound of the observation source itself — not a silent false negative, and not a reason
    outside the projected perimeter.
  - "A duplicated field name in a baseline entry's fact is silently last-wins, so the entry
    suppresses a different violation than the one it records" (0.3.1 sweep,
    `crates/xuanji/src/identity.rs`). Mechanics reproduced (`Baseline::from_json` on a
    duplicate-keyed `fact.fields` object does resolve last-wins), but the claim does not survive
    the contract lens — the shape cannot arise from any real fact construction path in this
    codebase, only from hand-authored malformed JSON.
  - "The composed baseline dogfood (`scripts/test_examples.sh`) exercises only the suppression
    direction, and its in-script justification misstates what the standalone test proves"
    (0.3.1 sweep). Refuted: a test-coverage complaint dressed as an unhonored claim — every
    documented claim it cites is in fact honored by the referenced test and README.
  - "Feature rules never read the target's own `[features]` table, so `serde/derive` declared
    there passes a rule that forbids every feature of `serde`" (0.3.1 sweep,
    `crates/guibiao/src/cargo_metadata.rs`). Mechanics accurate, but doubly refuted: the behavior
    is a stated bound (圭表 governs the *declared* per-target layer; resolved whole-graph
    feature unification is `cargo-deny`'s lane, per this file's own architecture decision), and
    separately the specific reproduction's classification didn't hold either.
  - "A `cfg_attr`-wrapped `#[path]` target is never enqueued, so the relocated file's typo'd /
    un-auditable probes are silently skipped" (0.3.1 sweep, `crates/louke/src/audit/scan/probes.rs`).
    Refuted: a stated bound spelled out in three places (spec, public API doc, `CHANGELOG.md`)
    and pinned by a deliberate test, not introduced by the diff under audit.
  - "`first_macro_arg_end` truncates an `as`-cast generic, merging two textually distinct
    un-auditable probes into one finding" (0.3.1 sweep, `crates/louke/src/audit/scan/lexer.rs`).
    Mechanics reproduce at the byte-scanner level, but the trigger is not reachable from
    compilable adopter input — refuted on the reproduction lens.
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
  - `PublicSeam::InherentMethod`/`InherentAssoc` now carry the impl block's own declaring module,
    closing the two-different-modules false negative verified real during the 0.3.1 sweep
    (`hunyi-public-seam-module-injection`).
  - `unsafe_confinement`'s and `trait_impl`'s `allowed_locations` (`only_implemented_in`/`and_in`,
    `only_under`) now reject a malformed `::`-path entry (an empty segment) as a constitution error,
    sharing the `must_not_expose`/`must_not_acquire` forbidden-operand family's guard
    (`resolve::validate_path_operands`, extracted from four verbatim call sites at the same time).
    Closes the entry previously recorded here as ACCEPTED DEBT: reproduced directly against
    `trait_impl_findings`/`unsafe_findings` before fixing, confirming the failure direction the
    debt entry named (a malformed allowed entry made every real site look disallowed, a spurious
    violation rather than a silent pass) — both named call sites are now fixed, not merely one
    (`hunyi-shared-path-operand-validation`).
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
