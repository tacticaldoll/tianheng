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

A **closed** item leaves the live class it was filed under; it does not stay there struck through. Its
reproduction record moves to *Closed in the 0.4.0 window* below, because a class heading is read as a
queue and an entry that carries a question and its answer at once is a reader trap.

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

**Empty.** Every item that stood here when the 0.4.0 window opened is closed; each one's reproduction
record is kept under *Closed in the 0.4.0 window* below, out of this queue so the index cannot read as open
migrations. Nothing currently requires a public or wire migration.

The `xuanji` sink for shared run/projection vocabulary remains *classed* DESIGN-BREAKING while sitting
under WATCH — its promotion trigger (a second standalone instrument consumer demonstrating the same
projection/run need) has not fired, and acting on it speculatively would break the proven standalone 圭表
consumer for an undemonstrated deduplication.

### READY-PATCH

- **`check_reference_integrity.sh` has no companion failure matrix.** Class: READY-PATCH (a test; nothing
  else moves). Observed pressure: noticed while narrowing that gate for the propose phase and recorded only
  in that pull request's body until now — a 306-line gate whose siblings
  (`check_release_coherence.sh`, the publish-source gate, the bound register) each carry a
  `test_*.sh` proving every refusal, while this one has none. Observation source: `scripts/` itself; the
  asymmetry is visible from a directory listing. Current bound: its refusals are unproven, so a change that
  narrowed it too far would pass CI — and one change already narrowed it, exempting `openspec/changes/`.
  Risk: MEDIUM — this gate is the reason stale in-repository references get caught at all, and it judges
  tracked content across 250 files. Promotion trigger: none needed; the fixture shape is the one
  `test_publish_source.sh` established (a throwaway git repo, since the gate judges tracked content).
  Version class: PATCH. Authority: the gate's own header, which states its rules in detail and proves none
  of them.

- **A bare trait name may not resolve against a same-module trait, contrary to the bound's own wording.**
  Class: WATCH (one observation, mechanism unconfirmed). Observed pressure: probing
  `semantic-impl-trait-operand-boundary`'s unresolvable-bare-principal bound, a bare `impl Frobnicate`
  beside a locally declared `pub trait Frobnicate` in the **same** module, with `crate::m::Frobnicate`
  forbidden, did **not** react. The bound's wording — "not a local trait resolvable in scope" — implies a
  local trait resolves. Observation source: that probe, recorded in the pull request that pinned the twin
  bound. Current bound: unknown whether the forbidden-operand spelling for a same-module trait differs from
  what was tried, or same-module bare resolution has a gap. **Asserted in neither direction**: it is filed
  as a lead precisely because this window produced four confident wrong claims from partial views. Risk:
  LOW-to-MEDIUM depending on which it is — a resolution gap here would be a false negative. Promotion
  trigger: one probe that distinguishes the two explanations, which is a fixture and a spelling change.
  Version class: PATCH if a spelling, minor if a false-negative closure. Authority: the probe, and
  `semantic-trait-impl-locality`'s resolution requirement, which states that a same-module trait needs no
  `use`.

- **The 天衡 shell's baseline-writing and CLI surface has never been swept.** Class: READY-PATCH (a sweep;
  its corrections are classified when they exist). Observed pressure: measured on the window's own history —
  of the 116 commits landing after the last range sweep closed, **20 carry a `tianheng` scope** and none of
  the four slices touched them. They are one coherent surface: atomic baseline writing (temp-then-rename,
  descriptor-vs-path `chmod`, symlink resolution, a temp-plant loop, data durability, directory flush) and
  CLI flag handling (inapplicable flags, empty and flag-shaped values). `xingbiao`'s two commits are
  likewise unswept. Observation source: `git log 23dbee5..release/0.4.0` by scope. Current bound: the four
  slices covered 渾儀 seam identity, 圭表's value namespace and module resolution, and 漏刻's origin and
  labels — the shell was never a slice. Risk: MEDIUM — this is the surface where a partial write or a
  mis-resolved symlink corrupts a recorded baseline, and its failure mode is silent. Promotion trigger: none
  needed. Sweep **against the enumerated surface** rather than against invented shapes, per `PROJECT.md`'s
  audit-cycle decision; the bound register does not enumerate this surface, so the first task is to find or
  build the enumeration rather than to start guessing. Version class: PATCH for what it finds unless a
  closure invalidates a recorded baseline. Authority: `PROJECT.md`'s audit-cycle decision, and the atomic
  baseline-write requirements in `violation-baseline`.

- **Five declared bounds have no pinning test.** Class: READY-PATCH (writing a test; no public API, wire
  format, or baseline identity moves). Observed pressure: the observation-bound register's own reaction
  reports them, so this is measured rather than suspected — five of the 41 declared bounds, two each in
  `external-crate-confinement` (cfg-blindness, the lib-and-bin conventional-path conflation)
  and `runtime-origin-assertion` (the target-subtree corpus, a production probe behind a non-production
  cfg), and one in `semantic-dyn-trait-boundary` (an unrenderable sub-node). Three more closed in this
  window once probed — the macro-generated dyn, the glob-imported type in an impl position, and the
  conservative-rank over-reaction, the last of which needed its own test because the nearest candidate
  pinned a different ceiling. Observation source:
  `scripts/check_bound_register.sh`, and the five `UNPINNED` citations that name this entry. Current
  bound: each is declared and believed true, and nothing would react if the behaviour changed — a bound
  reads as permission, so an undefended one is the shape that lets a real escape be dismissed as governed
  policy. Risk: LOW individually and worth naming collectively, since five of 41 declared bounds are
  undefended. Promotion trigger: none needed — each is a fixture and an assertion. They are debt rather
  than done for a reason of scope, not difficulty: six bounds in this window were probed and pinned first,
  and the five that remain need a fixture shape the cheap ones did not.
  **Probe before pinning**: two bounds enumerated in this window described behaviour that had changed and
  were retired rather than pinned, so a test written to match the spec without measuring first would give
  a false claim a green guard. Version class: PATCH. Authority: the five `UNPINNED` citations, and
  `PROJECT.md`'s drift law against a claim with no reaction.

- ~~**The 渾儀 seam-identity and owner-qualification surface has not been swept against the bound index.**~~
  **CLOSED** in the 0.4.1 window, swept and found defended — by a structurally enforced enumeration rather
  than by diligence, which is why the result is worth recording instead of just noting "no findings".

  `every_public_seam_shape_is_named_and_identity_injective` carries the enumeration and forces it to stay
  complete: `seam_kind` matches all eleven `PublicSeam` variants with no wildcard, so a new variant cannot
  compile without a representative; `SeamKind::ALL` is asserted duplicate-free; the observed shape set must
  **equal** it; the shape-to-label mapping is a bijection checked in **both** directions, so a new variant
  folded into an existing shape cannot read as covered; each seam's field schema is asserted exactly; and
  `keys.len() == seams.len()` asserts the injectivity the name claims. Thirty-eight further
  distinctness tests cover the owner-qualification half across the family.

  **ACCEPTED DEBT, stated by that test itself**: the *content* of each representative stays hand-maintained.
  A representative whose field values do not actually differ from a sibling's in the distinguishing field
  would satisfy the set equality and the bijection while proving nothing about that field. The test names
  this as a judgment no structure can force, and it is accepted rather than closed because the alternative —
  generating representatives — would need a model of which field distinguishes which shape, which is the
  judgment itself. It is not a bound and does not belong in the register: it limits a test's completeness,
  not an observation.

  Lesson kept, because it recurred four times in this window: a gap suspected from a partial view dissolves
  on the full one. The injectivity assertion here was read as absent from a fifteen-line excerpt and sits on
  the sixteenth. Read the whole construct before reporting it, and diff what a change actually moves rather
  than reasoning about what it should.


- ~~**An inherited observation bound is RESTATED by each capability that inherits it, so one behaviour
  change leaves several specs stale at once.**~~ **CLOSED** in the 0.4.1 window. Closed by a reaction plus
  the repair it forced, not by choosing one of the three candidates the entry listed: they were framed as
  alternatives and are not — a reaction detects the restatement and the repair resolves it, and only the
  reaction stops the next one accumulating silently.

  The register made it measurable on its first projection: two behaviours were declared as bounds in three
  capabilities each, all six declarations citing one test. `check_bound_register.sh` now fails when a test
  is cited by declared bounds in more than one capability, keyed on the shared citation rather than on
  statement text — two declarations of one behaviour do not have identical prose, so text similarity would
  be a heuristic where a shared citation is a fact. Repetition within one capability is not a restatement
  and does not fire it.

  The repair sank the declaration to `semantic-signature-coupling`, which its own spec already identifies as
  the owner by stating the anchor-and-item property on every single-module-anchored capability's behalf, and
  the other two now reference it. **Raising a shared surface into its own capability was rejected**:
  ownership already existed, so a new capability would have added a name without adding an observation.

  A settled decision was reversed deliberately and the reversal recorded: the register originally declared a
  shared bound once per capability, because declaring it once would leave the other specs silent about a
  bound they have. The reference form, which did not exist when that was settled, keeps the bound visible
  everywhere it applies while leaving one declaration to maintain, so the property the old rule protected is
  no longer bought at the price of restatement.


### WATCH / ACCEPTED / DECLINED / BUILT

- **WATCH:**
  - Token/Lexer extraction (requires cross-scanner false negative or 3rd scanner).
  - `qianyi` generator & LSP/editor integration.
  - ~~A `#[cfg_attr(pred, path=…)]` remap on an **inline** `mod name { … }`~~ **CLOSED** in the 0.4.0
    window. Reproduced against the real entry point, as this entry's own trigger required, with the
    unconditional `#[path]` form as the control — and the reproduction **refuted the recorded risk
    class**. The entry predicted a false negative ("the arm's real children may never be scanned");
    the actual behaviour was a *constitution error* (exit 2, "source file could not be located"),
    so it was fail-loud, never a silent pass. What it really was is narrower and worse for adoption
    than for correctness: 圭表 refused to judge a crate that compiles cleanly, so an adopter using the
    idiom could not run `check` at all. The root cause was one line — `walk.rs` read only
    `direct_path_eq` and never `conditional_path_eqs` — and the fix follows every present candidate
    base, cfg-blind, exactly as 漏刻's spec already required for the identical shape. The lesson kept:
    a single-lens hypothesis can be right that something is broken and wrong about **how**, and the
    risk class is what decides urgency — so reproduce before promoting, which is what this entry's
    trigger said and why it was worth keeping.
  - ~~`xingbiao::crate_root_file` may collapse a multi-root package~~ **RETIRED — superseded by its
    own promotion.** Its trigger was "confirm `cargo metadata` actually emits multiple root files for
    one package before treating this as more than speculative". That was confirmed in the 0.4.0
    window (`[[bin]]` targets are reported alongside the `lib`, each with its own `src_path`), which
    promoted the hypothesis into the DESIGN-BREAKING entry above — now itself CLOSED by the
    per-target corpus. The WATCH line survived its own promotion and is struck here rather than
    deleted, since a reader of an older release may still be looking for it. Lesson kept: when a
    WATCH item is promoted, retire the WATCH line in the same change, or the index carries the
    question and its answer at once.
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
  - Macro/configuration coverage bounds. **One** named residual now that all three dimensions read
    `cfg_if!` arms: transparency covers **item position** only — an invocation inside an
    `impl`/`trait` body holds impl items, needing a parallel flattening across ~10 body walkers
    (measured, pinned by `a_cfg_if_inside_an_impl_body_is_a_stated_bound`). 漏刻's own lack of
    transparency, listed here as the second residual, is CLOSED: its byte scanner reads arm contents
    as real code in both passes, and `cfg_if_transparency_conformance.rs` now pins all three
    dimensions on the one fixture rather than two of three — its module doc already says so, and this
    entry was the site left behind.
  - ~~File-granular un-auditable-probe identity.~~ **RETIRED** — superseded, not accepted: the 0.4.0
    line qualifies an un-auditable-probe fact by its owner-qualified enclosing item and the offending
    expression's own text alongside its file, so the identity is no longer file-granular and this bound
    no longer exists. Kept struck rather than deleted because a reader of an older release may still be
    looking for it.
  - **漏刻's legacy directory corpus does not descend a symlinked subdirectory.** Measured on one
    fixture rather than hypothesised: a module reached through `#[path]` into a symlinked directory,
    holding a declared seam's only probe, is seen when the audit is given the **target root file**
    (clean — the module graph reaches it and reading a file follows symlinks) and unseen when it is
    given the **directory** (the seam reads as unprobed). Deliberate: the directory walk classifies
    entries with `file_type()`, which does not follow symlinks, so a symlinked directory is not
    recognized as one — which is what keeps a cyclic symlink from becoming an unbounded walk. Accepted
    because the input that matters has no gap: the 天衡 shell passes root files, and the directory input
    exists for source compatibility. Stated in `runtime-origin-assertion` and pinned in both directions
    by `a_symlinked_subdirectory_is_descended_from_a_root_file_and_not_from_a_directory`. This replaces
    a WATCH entry that guessed a different mechanism — "`try_visit`'s cycle guard possibly bypassed by a
    second, weaker guard" — which is refuted: no guard is bypassed, and the entry did not distinguish
    the two corpora, which is where the whole answer lives.
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
    guarantee (the file flush) is not in this bound: it is covered by the `baseline_cli`
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

### Closed in the 0.4.0 window — reproduction records

These are **not** open work. Each was a live item in this index when the window opened and is now closed;
the original entry is kept verbatim beneath its closing note because the reproduction record —
what was observed, by which lens, and why the trigger was believed narrower than it was — is the part
that stops the same defect being re-found from scratch. The present-tense `Class:` / `Risk:` /
`Promotion trigger:` lines inside each retained entry describe the state **at the time it was written**.

They live here rather than under their own class heading because an index that carries a question and its
answer at once is a reader trap — the same reason a stale WATCH line was retired in `68e183b`, applied to
the larger entries it left in place. Neither the class nor the number is restated collectively: each
retained entry carries its own `Class:` line, and a count here would go stale the next time an item
closes — which is how the previous two sentences came to say "DESIGN-BREAKING" and "six" about a section
that also holds a closed READY-PATCH record.


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

- ~~**An absolute `#[path]` literal's identity still disagrees across checkouts when its target
  coincidentally lies under one checkout's own anchor.**~~ **CLOSED** in the 0.4.0 window — the last
  open identity gap. Closed by stopping the relativization rather than by threading provenance through
  a labeling decision: a file reached through an absolute literal keeps the path the literal wrote, in
  every checkout, and the flag is inherited by the files that target reaches in turn. The recorded
  promotion trigger described the fix as threading "was this file reached via an absolute `#[path]`
  literal" through four functions, which read as a broad refactor and is why it sat; it is not, because
  `Path::join` discards its receiver EXACTLY when the joinee is absolute, so the fact is knowable at the
  single line that resolves the literal and only has to ride alongside the `(file, base)` pair the walk
  already carries. The test that pinned the gap failed with EQUAL identities the moment the flag landed
  — its own doc had said that is what closure would look like — and is replaced by one asserting it;
  the test that pinned "relativized when inside the anchor" is inverted, that behaviour having been the
  gap's mechanism. `runtime-origin-assertion` gains the rule and drops the two scenarios describing the
  old behaviour. The original entry is kept below for its reproduction record.

- **An absolute `#[path]` literal's identity still disagrees across checkouts when its
  target coincidentally lies under one checkout's own anchor.** Class: DESIGN-BREAKING.
  Observed pressure: found during round-2 adversarial review of
  `change/louke-unauditable-probe-relative-identity` (PR #157), which closed the
  general relative-path case but left this one already-non-portable construct
  unresolved. Observation source: pinned regression test
  `a_nested_absolute_path_literal_still_disagrees_across_checkouts_a_known_residual_gap`, which failed
  with EQUAL identities the moment the fix landed and is replaced by
  `a_nested_absolute_path_literal_now_agrees_across_checkouts`
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
  body. The bound was also stated in `crates/louke/src/finding.rs`/`audit.rs`'s own doc comments and in
  `openspec/specs/runtime-origin-assertion/spec.md`; all three now state the CLOSED rule instead (an
  absolute literal is never relativized), so this record's description of the old behaviour is history,
  not a description of any current surface.

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

- ~~**`OriginEntry`'s public constructor lets any code in the process assert an arbitrary runtime
  origin, defeating origin-based fail-closed confinement.**~~ **CLOSED** in the 0.4.0 window, by the
  `louke-origin-derived-from-type` change. Closed by removing the caller's input rather than by
  guarding it: the expansion target is now generic over the registered type and takes **no
  arguments**, so an entry's whole content is a function of that type and an origin the type does not
  have is unrepresentable. The original entry is kept below for its reproduction record and for the two
  dead ends it cost, both of which were recorded as viable before being tested.

- **`OriginEntry`'s public constructor lets any code in the process assert an arbitrary runtime
  origin, defeating origin-based fail-closed confinement.** (Spelled `OriginEntry::new` when this
  entry was written; renamed `__from_register_origin` and made argument-free in the 0.4.0 window.)
  Class: DESIGN-BREAKING.
  Observed pressure: verified real during 0.3.1 sweep cleanup (2026-08-02/03) — a
  hand-built `OriginEntry::new(TypeId::of::<RogueAdapter>(), "loukehot::good", "RogueAdapter")`
  passed to `install` alongside genuine `register_origin!` entries produces zero
  reaction for a seam declared `.only_origins(["loukehot::good"])`, even though
  `RogueAdapter` never legitimately registered that origin. Observation source: direct
  reproduction against the real `louke::install`/`assert_boundary!` public API, and later an in-tree
  test (`a_forged_origin_silently_passes_a_seam_its_type_may_not_cross`, since removed with the
  gap it pinned) that reproduced the same
  silent pass through the registry the way `install` builds it, then stopped **compiling** when the
  closure landed — that transition being the evidence, not a passing assertion.
  Risk: HIGH — it defeated the crate's core stated
  guarantee outright for any code sharing the process, not merely a narrow idiom;
  unlike the other entries here, it was a capability gap in the trust boundary
  itself, not an identity-collision edge case.
  **Two recorded promotion triggers turned out not to work, and both were recorded before being
  tested** — the lasting lesson of this entry. (a) `pub` → `pub(crate)` on the constructor breaks the
  legitimate `register_origin!` path, since `macro_rules!` visibility is checked at the expansion site.
  (b) A `#[track_caller]`/`std::panic::Location` redesign yields a *file path* where an origin's whole
  vocabulary is a module path. (c) A **proc-macro** does not help either: it is expanded into its
  caller's crate and resolved there exactly as a `macro_rules!` is, so a private constructor fails with
  `error[E0603]` at the adopter's own call (three-crate probe). All three share one shape — hunting for
  a macro that can pass something hand-written code cannot. No such macro exists, which is why the
  closure had to stop taking the origin from the caller at all. A backlog entry naming a fix that
  cannot be built is worse than one naming none: the next reader spends the budget before finding out.
  Authority: `openspec/specs/runtime-origin-assertion/spec.md`; this entry's own reproduction record.


- ~~**Only the ONE resolved crate root of a package is governed; its other compiled roots are not
  observed at all.**~~ **CLOSED** in the 0.4.0 window. Every compiled root is now its own corpus in both
  static dimensions, and an observation carries the compilation unit it came from as an identity role, so
  the same violation in two roots stays two facts rather than one that masks the other.
  Three measurements changed the shape of the work and are worth keeping: 漏刻 ALREADY governed every root
  (`member_root_files`, in production), so this was two dimensions diverging from a third rather than a
  family-wide bound — correcting this entry's own earlier claim that the scope question was shared because
  渾儀 uses the same single-root function, which was true of 渾儀 and never checked against 漏刻; a spike over
  the existing machinery passed 316 of 圭表's 317 tests, so the corpus model tolerated N roots and the
  remaining work was identity, not a rewrite; and a target's NAME turned out not to be unique within a
  package (this repository builds a `lib` and a `bin` both named `tianheng`), so the role is the root's
  path relative to the package directory.
  Two things the work found that the entry had not predicted: a sibling conventional root leaks into every
  other root's walk (a top-level `lib.rs`/`main.rs` is `crate` in each), which without exclusion reported
  one violation once per root — a duplicate worse than the false negative being closed; and a per-root
  loop that deferred ANY error swallowed genuine scan failures whenever a sibling root happened to be
  governable, which is a silent pass. Both are fixed and pinned. The original entry is kept below for its
  reproduction record.

- **Only the ONE resolved crate root of a package is governed; its other compiled roots are not
  observed at all.** Class: DESIGN-BREAKING. Observed pressure: promoted in the 0.4.0 window from a
  single-lens WATCH hypothesis and a one-line `ACCEPTED DEBT` mention ("multi-target conventional-path
  conflation"), both of which understated it. Measured through three independent lenses:
  (1) **mechanism** — `xingbiao::crate_root_file` returns the first library-kind target, else the first
  `bin`, one root by construction, and 圭表's `package_src_dir` plus reachability walk both start there;
  (2) **minimal shape** — a package of exactly `src/lib.rs` + `src/main.rs` with the identical offending
  construct in both files and a boundary on `crate` reports ONE violation, in `lib.rs`;
  (3) **broader shape** — `src/main.rs`, `src/bin/conventional.rs`, a `[[bin]] path` inside `src/`, and
  a `[[bin]] path` outside it are all unobserved when a library root exists. Observation source: those
  measurements, pinned in both directions (the resolved root reacts; the others are silent; a
  library-less package governs its first `bin`) by what is now
  `crates/guibiao/tests/per_target_corpus.rs` — renamed and inverted when this entry was closed, since
  the transition those assertions existed to detect is what happened.
  Current bound: stated in `module-boundary`'s single-governed-root requirement, replacing a
  requirement whose premise — "both crate roots (`lib.rs` and `main.rs`) resolve to `crate`" — was
  factually wrong, and whose recorded limitation (a cross-root same-named submodule) was a narrower
  story than the truth. Risk: HIGH by class, and it lands on the most ordinary Rust package shape — a
  library beside its binary. A real violation written in `main.rs` passes silently, which is the one bug
  class the Core Contract forbids; what limits it in practice is that the governed root is usually where
  the architecture lives, and that a boundary an adopter declares on `crate` still reacts to everything
  the library reaches. Promotion trigger: **per-target module graphs**. Two roots of one package both
  denote module path `crate`, so observing both raises a violation-identity question the current model
  does not answer (which `crate` a finding names) — the same reason the requirement this replaces
  already named it an amendment beyond the conventional-path scanner. Note also that 渾儀 resolves roots
  through the same `crate_root_file`, so the scope question is shared rather than 圭表-local. Version
  class: DESIGN-BREAKING (a per-target corpus changes which facts exist, and plausibly their target
  role). Authority: `openspec/specs/module-boundary/spec.md`'s single-governed-root requirement and its
  two scenarios; the pinned test above.

- ~~**An inbound rule's target match is namespace-blind, so a name declared in two namespaces at once is
  observed only as its module.**~~ **CLOSED** in the 0.4.0 window. Closed by observing the value
  namespace, not by unioning both readings — the entry's own "deliberately NOT" still holds, and that is
  what keeps `shallow_inbound_rules_protect_only_the_exact_module` passing unchanged: an import reacts
  only when the anchored module really declares a `fn`/`const`/`static` of that final segment, so an
  ordinary `use m::child;` naming only a module stays silent.

  The promotion trigger asked for "a value-namespace item observation for the anchored module" as though
  it had to be built. **It already existed**, one module away: `symbol_scan`'s definition collector reads
  every module's own top-level `fn`/`const`/`static` from declaration-cleaned source, with the
  true-inline-module keying and module-top-level-only disciplines already worked out for the
  strict-external local-precedence ladder. The trigger was a missing *connection*, not a missing
  capability — the same misjudged cost as the absolute-`#[path]` entry earlier in this window, whose
  trigger read as a four-function refactor and turned out to be one line. Worth recording as a pattern:
  a promotion trigger written from inside the code that lacks the observation tends to describe building
  it rather than looking for it.

  The residual is now the observation rather than the resolution, and is narrower than the entry's own
  note: a value declared inside a macro body or arriving through a re-export is unobserved, matching
  every other declaration reader in the dimension, and directs the reaction toward the module reading —
  the pre-existing behaviour, not a new gap. `rule-model-surface` states it with two scenarios, and the
  pinned test is inverted rather than deleted (`shallow_inbound_target_match_observes_the_value_namespace`).
  The original entry is kept below for its reproduction record.


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

## Version horizons

The version follows SemVer honesty (`AGENTS.md`), not milestone size: **non-breaking →
patch, breaking → minor**, and never a vanity minor bump. `AGENTS.md`'s *Versioning* section owns what
counts as breaking — any change the adopter has to act on, a stale recorded baseline included — so read
it before assigning a horizon here; the entries below are horizons, not a second definition.

- **0.2.x (shipped)** — additive depth on an existing observation source, false-negative closures. A
  historical record, not a precedent: those closures were classified as patch-class before the 0.4.0
  window settled that a change requiring adopter action earns a minor. The same work today is
  minor-class.
- **0.3.0 (shipped)** — stable rule identity (`RuleKey`), `StructuredFactIdentity`, unsafe-site decomposition, async seam identity.
- **0.4.0 (shipped)** — every compiled root governed, identity-coordinate completeness, the `cfg_if!`
  and conditional-remap conformance across all three dimensions.
- **0.4.1 (open)** — patch-class only, per the definition above: packaging and hygiene, prose and
  specs, opt-in depth, performance, and diagnostics whose exit code and emitted documents do not move.
  A false-negative closure belongs in the next minor instead, however small its diff.
- **Next breaking window (if earned)** — requires real adopter or correctness pressure.

## Explicitly not on the roadmap

- Active code-shaping / generation.
- Prescriptive framework you build inside.
- Lints (opinionated style checks rather than declared intent).
- Universal graph API (whole-graph analysis rather than declared per-target boundaries).
- Supply-chain policy engine (cargo-deny's lane).
- DSL macro consolidation (repetitive builders are designed-to-be-imitated for 潛移 gravity; leave explicit).
