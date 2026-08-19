# Project Contract — 天衡 (Tianheng)

Tianheng's orientation layer for humans and AI agents. Keep it short and concrete.

## Purpose

Tianheng is a Rust-native **reactive architectural-governance** framework. It does not
run your app and it does not instruct your agent; developers and agents propose change,
and Tianheng uses compiler/CI and runtime **reactions** to keep
architectural shape from drifting. The source of truth is Rust code; TOML, Markdown, and
reports are projections of it.

It grows from **modou** (墨斗): modou proved the static dimension as a single focused crate,
and 圭表 is derived from it — but modou lives on as an independently-developed sibling project,
not a line Tianheng supersedes. Tianheng keeps that proven core and grows it into a **crate
family** of observation dimensions — without becoming a god crate.

## What Tianheng is — and is not

- It **is** reactive governance across **observation dimensions** (static, semantic, and
  runtime — all three 儀 built), each a separate crate the user selects by depending on it.
- It is **not active shaping**: it observes and reacts; it does not generate or prescribe
  structure. (Active shaping is a different axis, deliberately deferred — adopting it would
  be a conscious amendment, not drift.)
- It is **not a framework** in the prescriptive sense: you do not build *inside* it. It is
  a CLI + library.
- It is **not a lint**: every dimension must be real drift — declared intent vs. observed
  reality — never an opinionated style check.
- It is **not a supply-chain policy engine**: resolved, whole-graph dependency policy —
  advisories, dependency licenses, bans / duplicates, resolved source allowlists — is
  cargo-deny's lane (run in this repo's `supply-chain` CI job). Tianheng governs the
  *declared, per-target, architectural* layer instead (deps / imports by name, declared
  dependency-source kind for manifest hygiene, type exposure, impl locality, visibility,
  runtime seams). The two are complementary, not overlapping — the reason resolved
  build-provenance is cargo-deny's, not a Tianheng capability (see the 圭表 depth decision).

## Core Contract

A **declared boundary reacts.** A boundary declared in Rust must produce a real,
non-bypassable reaction when violated — for the CI dimensions, a CI failure with a non-zero exit and
an explanatory report. The reaction MUST never silently pass, and MUST distinguish a
boundary violation (exit 1) from a constitution error / misconfiguration (exit 2). The
one forbidden bug is a **false negative** (a real violation that silently passes).

**Non-bypassable, precisely.** The governed code's own *shape* cannot make a declared boundary stop
reacting: no spelling, alias, re-export, `cfg` arm, or macro form escapes observation, and where an
observation genuinely cannot decide, the reaction is exit 2 rather than a pass. What an observation
must never accept is a value the observed code chose for itself — a declaration wearing an
observation's name. 漏刻's runtime origin is the worked example: it is **derived from the type** (the
module the concrete type is defined in), never taken from the registering call, so no code in the
process can present a type under an origin it does not have. `runtime-origin-assertion` states it in
full. A governance tool must claim exactly the guarantee it has — neither more, which is a lie, nor
less, which invites the workaround.

## 潛移 (Qiányí) — govern by gravity, too: the idiom is imitated, the reaction is the backstop

The reaction binds, but an autoregressive agent is first an **imitation engine** — it
continues whatever idiom sits in its context. So compliance has a second source,
complementary to the non-bypassable reaction: **潛移 (qiányí) — gravity, the quiet pull of an
idiom** (潛移默化: it is assimilated without being told). The more the declared law and the
governed code read *as one strong, distinctive idiom in the agent's context*, the more its
continuations stay in-shape by default — and invocation stops being an act the agent must
remember and becomes an **emergent property of imitation**. This is neither instruction
(dictating what to write) nor bare reaction (catching it after); the agent is *pulled*, not
pushed or told — still consistent with "we do not instruct your agent."

Hence a standing design principle: **every Tianheng-facing semantic surface — the
declaration DSL, the `because`/`reason` prose, and the law's projections (`list --format
markdown` foremost) — is designed to be *imitated*, not merely *read* or *parsed*.**
Legibility serves a human; imitability serves the continuation engine, and the two optimize
differently — density, distinctiveness, and reason-as-first-class-prose over exhaustive
enumeration. But imitability is **bounded by the drift law**: an imitable surface states the
forward shape its boundary *observes and reacts to*, never structural guidance beyond that
perimeter. A reason that pulls the agent toward a shape Tianheng cannot react to is prose
prescription — the open loop this project exists to close, smuggled back in as text.

Gravity does **not** replace the reaction; it relocates it. Imitation transports *surface
form*, never *invariants* — a strong idiom still admits a locally-plausible violation. So
gravity lowers the base rate of drift (the frictionless primary) and the non-bypassable
reaction forecloses what gravity misses (the backstop). The one forbidden bug is still a
false negative, and only the reaction can foreclose it.

## Inherited laws (from modou — non-negotiable)

- **Drift law** — *No drift type without an observation source. No target type or name
  without a reaction.* Names are not claimed for reactions that do not yet exist; this
  holds at module, crate, and **dimension** granularity (we do not pre-create empty
  `semantic`/`runtime` crates).
- **Minimalism bound** — fail loud only on *observable misconfiguration*; no defensive
  over-foolproofing of impossible states.
- **SemVer honesty** — pre-1.0, a release that breaks no public API is a **patch**, never
  a vanity minor bump. (modou's hard-won lesson.)

## Architecture — a crate family, not a god crate

- **`xuanji` (璇璣) — the 底 (bedrock).** The dimension-agnostic **reaction model** the
  whole stack turns on: `Severity`, `BoundaryKind`, `Violation`, `Report`, `Baseline`,
  `Outcome`, with the JSON serialization intrinsic to those types. `serde_json`-only; carries no observation
  engine, and depends on no workspace member — every dimension sits above it.
- **`xingbiao` (星表) — the workspace-data substrate.** The star-table: the shared,
  `serde_json`-only reader of `cargo metadata` (`cargo_metadata` / `find_package` /
  `crate_root_file`), sitting below every dimension like 璇璣 and depending on no workspace
  member. It is *not* 璇璣 — it does IO (it spawns cargo) and observes — but a substrate beneath
  the dimensions, so the static and semantic dimensions read the workspace through **one** source
  of truth, not two hand-copied twins that drift apart (the v0.1.6 SSOT extraction — see Decisions).
- **`guibiao` (圭表) — the static observation core.** The gnomon: it reads the cast
  shadow — imports, dependencies, and inline symbol-path calls (the clock-free
  `must_not_call_inline` confinement). The dependency-light static engine, derived from
  modou: declare crate- and module-import boundaries, observe from `cargo metadata` (read
  through 星表) and source `use` / symbol scans, compare, react. Pure functional core — no shell; its dependency
  allowlist is declared in the law and rendered in [`AGENTS.self-law.md`](AGENTS.self-law.md), not restated
  here. The report/constitution *assembly* (which folds in the static `Coverage`) lives here, not in the model.
- **`tianheng` (天衡) — the shell.** The celestial balance that weighs declared against
  observed: the imperative shell + facade — CLI (arg parsing, filesystem, stdout/stderr),
  the `run` reaction that composes every dimension into one, and the re-exported public
  surface. Depends on every dimension it composes (`guibiao` + `hunyi` + `louke`). It is also where
  cross-cutting **composed profiles** live (e.g. `sans_io_pure`, folding a 圭表 clock-free and a 渾儀
  synchronous-API boundary into one declaration) — a dimension never composes its sibling, only the
  shell does (三儀 ⊥ 三儀).

**Functional core ⊥ imperative shell, at crate granularity.** `guibiao` must not depend
on `tianheng`. This is the crate-level upgrade of modou's module-level `engine ⊥ runner`,
and Tianheng enforces it on itself (`crates/shengmo/src/law.rs`) — eating
its own dog food, now across crate boundaries.

**A dimension is a crate born when built** (drift law at crate granularity), and the user
selects governance by depending on the dimensions they want:
- **`hunyi` (渾儀)** — AST/semantic observation (`syn`). **Built (v0.1.0):**
  signature-coupling (a module's public API must not *expose* a forbidden type), plus
  trait-impl locality, visibility, and forbidden-marker boundaries; **(v0.1.2):** a
  type-shape/existential **depth stair** on the same `syn` source — dyn-trait and impl-trait
  exposure (each shape-only *and* named-operand-scoped) and async-fn exposure — the type-shape
  and existential complements of signature-coupling; **(v0.1.3):** two further same-source depth
  additions to that flagship exposure surface — **re-export exposure** (a named public `pub use`
  of a forbidden type is itself an exposure, default-on — an API-compatible but behavior-changing
  false-negative closure) and **trait-impl exposure** (the opt-in `.including_trait_impls()`
  depth, surfacing a trait impl's impl-site-authored positions); **(v0.1.8):** a **visibility
  ceiling** (`max_visibility(Crate|Super|Module)` — the binary `must_not_declare_pub` generalized to
  a rank), **`unsafe`-confinement** (`UnsafeBoundary::only_under([…])` — `unsafe` confined to a
  declared subtree, the non-compiler-expressible complement of `#![forbid(unsafe_code)]`), and an
  opt-in **whole-subtree scope** for async-exposure (`including_submodules`) — all detailed in the
  Decisions section. The heavy `syn` dependency is quarantined here, never in the core.
- **`louke` (漏刻)** — runtime observation. **Built (v0.1.0):** origin-assertion (a
  declared seam's `only_origins` allowlist), in two faces — the prod probe
  (`assert_boundary!`, fail-closed, a structured event by default, panic opt-in) and the
  `audit_probe_coverage` CI face, composed into `tianheng check`. Ships into the production
  binary; hot path is std-only, depends on 璇璣 only — 星表 is an additive, `audit`-feature-gated
  exception that never reaches the production hot path (0.2.3). (Design gate resolved — see
  Decisions.)

**The observatory vocabulary (manifested in governance).** The three observation
dimensions — 圭表 (static), 渾儀 (semantic), 漏刻 (runtime) — are the **三儀** (the three
instruments): *what* Tianheng measures; each is a crate born when built, each adds a new
drift type. Beside them the governance & observability layer names three surfaces by what
each does: **垂象** surfaces a reaction — `crates/guibiao/src/projection.rs` assembles the
report and constitution documents, `crates/tianheng/src/runner/render.rs` renders text and
SARIF, `crates/xuanji` serializes a `Violation`. **實錄** records one — `crates/xuanji/src/baseline.rs`
holds the model every crate above it folds its verdicts into. **校讎** amends one —
`.github/CODEOWNERS` routes a change to the law to the steward, `AGENTS.md` owns the OpenSpec
lifecycle, and `crates/tianheng/src/constitution.rs` names that routing in shipped source.
儀 measures; each of these three administers (司) rather than measuring.

**None of them is or becomes a crate**, and the reason is boundaries rather than importance.
三儀 are orthogonal — a dimension must never learn from a sibling — so each needs a boundary
the self-law can react to, and every dimension's `restrict_dependencies_to` naming no sibling
**is** that reaction. A governance surface has no boundary to be: each crosses every crate it
touches, and one lives outside `crates/` altogether. A crate there would enclose nothing, so
the name would mark nothing — the drift law's own prohibition rather than a stylistic call.
This says nothing about `xuanji` or `tianheng`, which are crates and are not instruments; it
says a surface with no boundary earns no crate.

**Two names are crates and are neither 儀 nor 司.** 繩墨 (the inked line) holds the law this repository
declares over itself and the dogfood gates that run the delivered product's reactions against this workspace;
勘合 (the split tally) holds the repository checks that fit its record against itself. Both ship in **zero** packages, which is what
separates governance from product here, and both are deliberately outside the observatory vocabulary: they
measure nothing and administer nothing. 勘合 was first drafted as 校讎 — a word already spent above, on the
amendment flow — and taking a name that already has a referent is the misnaming these crates exist to end.

This paragraph replaced one reading *crate-or-convention as their nature dictates*, which
answered nothing and was consulted three times. The three answers written for it were all
withdrawn, each restating a law that was then only half reacted to. None of the three is a
crate name, and none reaches any published crate's public surface: measured, zero occurrences
across the six publishable crates' `///` and `//!` documentation, and one `//` comment inside
`tianheng`'s private `render` module. Where they do appear — that comment, `kanhe`'s manifest
comments, `shengmo`'s source, and this repository's own governance documents and changelog — it
is commentary about this repository's vocabulary rather than a name an adopter uses. So the
naming was never a product question.

## Naming — narrative, with meaning in the SSOT

Crate and concept names are **coined / narrative** (圭表, 渾儀, 漏刻), in the celery/kombu
tradition: a name is a stable handle, not a self-description. Meaning lives in the
authoritative **metadata SSOT** (each crate's `description` + docs) — fitting for a tool
whose own thesis is "the source is the SSOT; names are projections." The brand `tianheng`
(天衡) and the bedrock `xuanji` (璇璣) split the one master instrument, 璿璣玉衡: 璣 → 璇璣,
the jade pivot every measurement aligns to; 衡 → 天衡, the balance that weighs declared
against observed. The brand is a star (玉衡, in the Dipper's handle), not an instrument — so
it sits cleanly above the 三儀 it wields, sharing no name with any of them.

潛移 (the gravity thesis above) deliberately breaks the celestial pattern: it names neither
an instrument (儀) nor an office (司) but a **mode of governance** — compliance by imitation —
so it is drawn from the idiom 潛移默化 (change that assimilates without the subject's
awareness), not from 璿璣玉衡. It is a handle for *how* the declared law spreads, parallel to
govern-by-reaction, never a thing the tool wields.

## Decisions

Record significant decisions here (the *why*; specs and code carry the *what*).

- **Accepted: 繩墨 and 勘合 are formations of the law, not a relocation of it.** `09c29d0` added two crate
  boundaries to `shengmo::law::constitution()` under a commit body reading *the law itself did not change:
  the regenerated projection differs by exactly three lines, all of them the preamble's own self-reference*.
  The projection gained nineteen lines, fourteen of them these two entries, with their own targets, rules and
  severities. What was recorded as a relocation was a formation, and a formation carries its own acceptance.

  Accepted as declared: **繩墨 may depend on 天衡 and serde_json only; 勘合 on 繩墨, 天衡 and serde_json
  only.** Both at `enforce`. Both crates ship in no package, so no adopter is reached either way. The cost
  accepted with them is the ordinary one: a later edge from either to a dimension is an amendment, not a
  commit.

  The acceptance rests on evidence rather than on the record's word: four perturbations, one command, each
  perturbed run exiting `101` with the boundary's own `reason` carried into the violation, and the clean run
  at `0` — reproduced and written out in the `[0.5.0]` CHANGELOG entry. *(Accepted 2026-08-19.)*

- **The amendment requirement is a reaction now, because as prose it never fired.**
  `.github/CODEOWNERS` says the review requirement *is* the reaction and that a merge cannot relax the law
  without a human accepting it — then says designation alone only auto-requests review. Measured, `main`'s
  protection answers `require_code_owner_reviews: false` and `required_approving_review_count: 0`; and
  enabling it would not close this, since GitHub does not let a pull request's author approve their own, so
  for a single-steward repository the rule cannot fire at all. That is a prose prescription with no backstop
  standing on the law itself, which is the shape this project's reason rule refuses everywhere else.

  Every boundary the law declares — its heading, reason, rule and severity — is declared in
  `crates/kanhe/tests/self_law_amendment.rs` and held against the projection in both directions. A boundary
  added or removed, an allowlist widened, a severity lowered from `enforce` to `warn`, or a reason rewritten
  all move it. Measured before building it: with 璇璣's allowlist quietly widened to permit 圭表 and the
  projection re-blessed, all nine existing self-governance assertions pass and only this one fails.

  **What it establishes is naming, not acceptance, and the difference is recorded rather than blurred.** It
  forces a structural change to the law to produce a *second explicit artifact*, in a file CODEOWNERS routes
  to the steward, so a delta cannot arrive inside a regenerated projection unremarked. One actor can still
  change the law, re-bless the projection and edit the declaration in a single commit, and everything passes:
  that shows the amendment was named, not that anyone accepted it. Acceptance rests on a steward decision,
  and a single-steward repository has no mechanical second party to carry it — a pull request's author cannot
  approve their own. That is a **judgement boundary**, recorded here as one; renaming a same-author constant
  edit as acceptance would be the overclaim this project's own reason rule refuses.
  *(Decided 2026-08-19; the naming/acceptance distinction stated 2026-08-19 after review.)*

- **Reborn from modou as a crate family.** modou was taken as frozen/complete at its own
  `0.1.1` when Tianheng was reborn; Tianheng started fresh (clean git history, clean SemVer
  from `0.1.0`) rather than expanding modou's single crate into a god crate. The runtime
  dimension *must* be a separate crate (it ships into production and must stay light), so a
  family is the destiny — but members are born only when built. *(Amended 2026-07: modou is
  unfrozen and now develops independently in its own repo — a living sibling, not a superseded
  ancestor. Tianheng retains all three dimensions including the static 圭表, does not reroute
  static-only users to modou, and 圭表 stays derived-from-modou by lineage. How modou evolves —
  including whether it depends on `guibiao`/`tianheng` — is out of Tianheng's scope; the two do
  not share a workspace, so no shared-shell / born-when-built commitment is pulled forward.)*
- **The static core is `guibiao`, not `tianheng-core`.** Named by its stable identity
  (the gnomon, the static instrument, modou's derivative), not by a temporary role ("the
  whole core back when it was the only dimension").
- **Cross-crate visibility is the price of the split.** Items modou kept `pub(crate)`
  (baseline, coverage, projection, `check_and_cover`) are `pub` in `guibiao` because the
  shell consumes them across the crate boundary. This widens the engine's public API
  beyond modou's minimal `check` — acceptable, and refinable pre-1.0.
- **Baseline is a generated snapshot, not policy.** A baseline records accepted
  violations so a dirty project can adopt a boundary and gate only on *new* drift; it is
  a projection of the report, never the constitution.
- **Module imports are observed by scanning source `use` declarations**, not by parsing
  a full AST. A hand-rolled scanner keeps the 圭表 core dependency-light and macro-free;
  its partial coverage — bare path expressions and macro-generated imports are out of scope —
  is acceptable because the drift law only enforces what is observed. An unconditional, direct
  `#[path = "…"]` remap is **followed** to its target (0.2.2), matching 渾儀/漏刻, so all three
  observation dimensions agree on what rustc compiles; a `cfg_attr`-wrapped `path = "…"` is
  **union-scanned** (0.3.x) — every candidate that physically exists on disk is followed, never
  either silently preferred over the conventional file or dropped for being cfg-conditional. A
  candidate that resolves is also treated as legitimate grounds for the conventional file's own
  absence — the same "might legitimately be absent on this build" signal a bare `#[cfg]` or a
  `cfg_if!` arm already carries — so a module backed only by one or more `cfg_attr(path)` remaps
  (e.g. two stacked, jointly-exhaustive per-platform attributes, neither a plain file nor a direct
  `#[path]`) is governed rather than hard-errored, matching 渾儀/漏刻's identical rule for the same
  shape. Only when every candidate is absent, with no other cfg-conditional gate, does it fail
  loud rather than governing a same-named orphan. Comments and
  string literals (normal, byte, and raw) are stripped so their text is never mistaken
  for a `use`. A module's identity is derived in three places — its file path, its `mod`
  declaration, and a `use` path that names it — and these MUST stay in lockstep, since a
  divergence both fails to govern a real module and silently hides its imports (a false
  negative, the one thing the core contract forbids). Two consequences stay token-level,
  not parser-level, to keep the hand-rolled scanner: a raw identifier is canonicalized
  (`mod r#type;` compiles to `type.rs`, so `r#type` and `type` are one module), and a
  `use` is attributed to the inline `mod { … }` that encloses it (so `self`/`super`
  resolve correctly); macro bodies are stripped before scanning for `mod` declarations
  too, not just `use`s, so the out-of-scope rule for macro-generated items is symmetric.
  One macro is carved out of that stripping in **all three** dimensions: `cfg_if!`, whose
  arms wrap human-authored items without transforming their identities, so its contents
  are real code (圭表 0.2.3, 渾儀 and 漏刻 0.4.0 — each hand-written, 三儀 ⊥ 三儀, with
  `cfg_if_transparency_conformance.rs` as the drift check). Gating that carve-out on the
  macro **name** is soundness, not caution: an arbitrary macro's nested blocks are not arms,
  and reading them as such invents items the macro may never emit.
  Adopting a real parser (`syn`) would resolve all of this for free but would break the
  dependency-light core (the `serde_json`-only self-law); that is an amendment, not a
  silent trade. A boundary's governed *target* is file-based: an inline `mod name { … }`
  is reachable for import attribution but owns no file, so it cannot be a target — a
  boundary on one fails loud with a self-describing constitution error (exit 2), distinct
  from an unknown-module typo, never a silent pass. Governing inline modules as targets is
  a deliberate non-goal here; if ever wanted it is a separate amendment.
- **Module Resolution & Safety Key Disambiguation.** Keyed identity for *governance* (what to report a violation under) and keyed identity for *safety/resolution bookkeeping* (what counts as "the same thing open", or where a file's own children live) are separate keys. A fix must target the underlying shared model rather than a single reported instance (the 0.2.2 module resolution lesson).
- **圭表's source concern is the declared layer; the resolved layer is cargo-deny's.** Tianheng governs the declared per-target layer (manifest hygiene, declared imports); resolved whole-graph build-provenance belongs to `cargo-deny`.
- **`xuanji` is an internal refactor (reaction model), `serde_json`-only.** Holds dimension-agnostic types (`Violation`, `Report`, `Baseline`, `Outcome`) beneath all dimensions without observation engines.
- **`xingbiao` is the shared workspace-data substrate.** Cargo metadata reading logic is consolidated into `xingbiao` below the 三儀 to prevent twin-drift.
- **The semantic capability-admission test (the gate against lints).** A semantic capability is admissible in 渾儀 iff: (1) declarative-not-lint; (2) no essential gap on local-crate AST; (3) anchorable to a `syn`-resolvable element.
- **Name resolution is a 渾儀-internal shared layer (`hunyi::resolve`).** `guibiao` (syn-free scanner) and `hunyi` (`syn` AST) retain separate resolution engines to maintain the syn quarantine.
- **漏刻 (runtime) is identity-coherent.** Prod face (`assert_boundary!`) is std-light and fail-closed; CI face (`audit_probe_coverage`) is feature-gated behind `audit` (`xingbiao` dependency). The shipped default sink never panics on a broken stderr write, and never silently loses that failure either: it counts it (`dropped_sink_events`), a single infallible atomic add, so an adopter who never calls `set_sink` can still detect the loss from outside the process.
- **Violation identity is a structured observed fact, not presentation.** 璇璣 carries the vocabulary-neutral `ViolationId`, constructed from a governed target, a validated `RuleKey`, and a `StructuredFactIdentity` — `(target, rule_key, fact)` — with each dimension owning its own fact schemas. Diagnostic text and file paths stay out of baseline identity.
- **Unsafe-site identity is structurally decomposed, not compressed into one label.** 渾儀's unsafe-confinement fact carries owner, owner kind, trait, and name as separate structured fields across nine site shapes (free fn, inherent/trait/trait-impl method, inherent/trait impl, trait, extern block, block), so two distinct sites sharing a name never collapse under one baseline entry.
- **Async identity is the seam, not the rendered signature.** An async-exposure fact's identity is its module, owner kind, canonical owner, and item name; the full parameter/return signature is human diagnostic presentation only and never enters identity — a signature-only change preserves the baseline, while a different owner or name is a new fact.
- **Rule construction is builder-owned; inspection stays open-ended.** Data-carrying `Rule`/`ModuleRule` variants are `#[non_exhaustive]`.
- **Repository-check diagnostics are behavior-tested, not constructor-governed.** Kanhe keeps one pure typed
  result separating a disagreement from an input it cannot judge, and focused matrices assert the kind and
  actionable message at operator-facing boundaries. Constructor locations are implementation detail: they do
  not carry runtime mutation, reach recording, exemptions, or product-visible observation bounds.
- **Repository observation-bound catalogs stay with their unpublished check owner.** Kanhe returns bounds
  for its record and coherence checks, Shengmo returns bounds for its self-law dogfood, and the repository model
  composes both with the product dimensions. Tianheng exposes no repository catalog: the shell composes product
  observers rather than claiming unpublished governance as product capability.
- **Governance is driven by an audit cycle against an enumerated surface, not by invented hypotheses.**
  A sweep that invents its own scope cannot be dry, only tired: it reports the shapes someone happened to
  think of, and "the audit was thorough" is then unfalsifiable. So the standing direction is to enumerate a
  claim surface, check it, and audit against that enumeration. `observation-bound-register` is the
  first instance and the proof of the shape — assembling it retired two bounds that had outlived their
  behaviour and added six tests for bounds nothing defended, after twenty pull requests, ten review rounds
  and a three-slice hypothesis-driven sweep had missed all eight. The same three-part test applies to any
  future instance: the surface must be enumerable, the enumeration must be **generated and
  staleness-checked** rather than hand-maintained, and the projection must state what it does not claim.
  A register that implied completeness would mislead exactly where it is most trusted. `observation-bound-model`
  is the second instance: it enumerates the same surface the register does and checks what each bound's
  stop actually *is*, which the register never governed — a slot that had accumulated many phrasings, one of
  them used by two capabilities for bounds on opposite sides of the false-negative line. `gate-shape-contract`
  was the third, and the first to turn the cycle on **this repository's own check surface** rather than on the
  specs: the `check_*` gates and their twins, enumerated from tracked content, with the properties that cannot be
  mechanically checked declared as bounds. It supplied the cycle's sharpest evidence that a
  hand-written table is not an enumeration — the backlog entry proposing it described the surface as uniform, and
  applying it found two properties that were not, one of which no twin held at all once the assertion was
  actually observed rather than counted. **It was retired by `5abda1f` in the same window**, once the shell
  gates it enumerated were deleted and its subject no longer existed; `repository-checks` holds what replaced
  them. The lesson is what the cycle keeps — the capability was the instrument, not the finding.
  `projection-register` is the fourth and turns the cycle on the
  repository's own **generated documents**: the set of them was prose in `AGENTS.md`, so the mechanism whose whole
  purpose is to stop documents drifting was described by a document that drifted. Its lesson is narrower and
  sharper than the others' — when a check's subject is *text*, the check's own text is part of the corpus,
  and it hit that three times in one apply (its specification quoting the marker it requires, its own source
  naming the signature it excludes, and its projection having to be blessed twice to include itself). The fix each
  time was to recognize by position or shape and never by the bare string. Larger surfaces are
  named but not committed to — the requirement and scenario surface across every capability is a
  multi-window program, and promoting one is a `BACKLOG.md` decision with its own trigger, never an
  assumption inherited from this first success. Its size is deliberately **not** written here. This decision
  once named two such figures, and by 2026-08-05 both had drifted by dozens — measured, not noticed — so a
  promotion estimate read off them would have been drawn from a surface that had already moved. That is
  stated in the past tense on purpose: a count is only ever true of the moment it was taken, so it belongs
  to the measurement made when a surface is promoted, beside the check that will keep it honest, never to
  prose. Writing one here would be the claim class this very decision exists to end.
- **An observation participant declares its limits by construction.** `xuanji::Observer` has two methods and
  no default body on either, so a participant cannot be composed into a run without declaring what it does not
  observe — the family's central promise becomes a property of the type rather than a convention it keeps about
  itself. Adding a stage is a breaking change on purpose: that is the enforcement working. The fold is a
  **fan-out with a cannot-judge short-circuit, never a pipe** — 三儀 ⊥ 三儀 forbids one dimension consuming
  another's output, so the shell composes and the dimensions do not — and it is **eager**, which is why no trait
  object exists anywhere: the heterogeneous set never forms. Assembly order is part of the contract, because it
  decides which cannot-judge is reported.
- **A published vocabulary value is admissible only if a third party owns one.** The question to ask of any
  enum reaching the public surface is whether an outsider occupies one of its values or must borrow one of this
  family's. `Owner` (`Engine` / `Inherited` / `Adopter`) names **roles**, so a third party's own engine is its
  `Engine`; `BoundaryKind` (`Crate` / `Module` / `Semantic` / `Runtime`) names **三儀's own dimensions**, so an
  outside `Observer` must label its findings with a dimension that did not produce them. Identity-bound versus
  role-relative is the discriminator, and it is applied before publication because that is the only moment the
  answer is free: a shipped variant cannot be unshipped, and an adopter who has already labelled findings has
  already been made to lie. It is not a general preference for abstraction — `BoundaryKind`'s four names are
  right for the dimensions they name, and the gap is that nothing else can be said.
- **The composed adopter surface is compile-reacted.** `tianheng::prelude::*` is the entrypoint. `check_constitution(&Constitution, &Path) -> Outcome` unifies CLI and library testing evaluation.
- **OpenSpec is adopted in its `specs` mode, not its `changes` mode.** OpenSpec offers two halves: `specs/` as
  the per-capability requirement truth, and `changes/` as a proposal workflow — a change directory carrying
  `proposal.md` / `design.md` / `tasks.md`, synced into the specs and then archived. This project uses the
  first and not the second. A requirement change is written directly onto its spec, in the same branch and
  pull request as the reaction that answers it. *(Recorded 2026-08: the mode had been chosen in practice from
  the beginning and stated nowhere — measured, zero change directories have ever existed, while seven whole
  capabilities were specified in the 0.5.0 window alone, four of them under `feat:`. `AGENTS.md` meanwhile
  described the `changes` workflow in the present indicative, and the cost was observed rather than
  hypothesised: an agent reading it first, as that file instructs, planned work through a mode this project
  does not use, three times in one session, before anyone measured.)* **What would change it:** a change too
  large to land as one reviewable pull request, where the value of a written proposal exceeds the cost of a
  second place for requirements to live. That has not happened; the discriminator is reviewability, not size
  of intent. Nothing enforces the mode, deliberately — a reaction that failed when a change directory
  appeared would prevent adopting the other half, which is the freezing this decision exists to avoid.
- **繩墨 and 勘合 are two crates because they answer to different subjects, and the dependency law states only
  what it can observe.** 繩墨 holds the law 天衡 declares over itself and the dogfood gates running the
  delivered product's reactions against this workspace; 勘合 holds this repository's record against itself and
  reaches no product contract. Keeping them apart is what stops a claim about the law being read as a claim
  about document hygiene, and it is why 繩墨 is an adopter of the shell rather than a member of the family it
  governs — exercising exactly the surface an adopter has. **None of that is observable by a dependency
  rule**, so it lives here: the two `restrict_dependencies_to` boundaries state only their allowlists and the
  dimension edges those forbid, which is their whole observable perimeter. The roles are the reason the
  boundaries were formed; the boundaries react to the edges.
