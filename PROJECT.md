# Project Contract — 天衡 (Tianheng)

Tianheng's orientation layer for humans and AI agents. Keep it short and concrete.

## Purpose

Tianheng is a Rust-native **reactive architectural-governance** framework. It does not
run your app and it does not instruct your agent; developers and agents propose change,
and Tianheng uses compiler/CI and runtime **reactions** to keep
architectural shape from drifting. The source of truth is Rust code; TOML, Markdown, and
reports are projections of it.

It is the successor to **modou** (墨斗): modou proved the static dimension as a single
focused crate; Tianheng keeps that proven core and grows it into a **crate family** of
observation dimensions — without becoming a god crate.

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

## Core Contract

A **declared boundary reacts.** A boundary declared in Rust must produce a real,
non-bypassable reaction when violated — for the CI dimensions, a CI failure with a non-zero exit and
an explanatory report. The reaction MUST never silently pass, and MUST distinguish a
boundary violation (exit 1) from a constitution error / misconfiguration (exit 2). The
one forbidden bug is a **false negative** (a real violation that silently passes).

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
  whole stack turns on: `Severity`, `Violation`, `Report`, `Baseline`, `Outcome`, with the
  JSON serialization intrinsic to those types. `serde_json`-only; carries no observation
  engine, and depends on no workspace member — every dimension sits above it.
- **`guibiao` (圭表) — the static observation core.** The gnomon: it reads the cast
  shadow — imports and dependencies. The dependency-light static engine, derived from
  modou: declare crate- and module-import boundaries, observe from `cargo metadata` and
  source `use` scans, compare, react. Pure functional core — no shell. Depends on `xuanji`
  (the reaction model) and `serde_json` only; the report/constitution *assembly* (which
  folds in the static `Coverage`) lives here, not in the model.
- **`tianheng` (天衡) — the shell.** The celestial balance that weighs declared against
  observed: the imperative shell + facade — CLI (arg parsing, filesystem, stdout/stderr),
  the `run` reaction that composes every dimension into one, and the re-exported public
  surface. Depends on every dimension it composes (`guibiao` + `hunyi` + `louke`).

**Functional core ⊥ imperative shell, at crate granularity.** `guibiao` must not depend
on `tianheng`. This is the crate-level upgrade of modou's module-level `engine ⊥ runner`,
and Tianheng enforces it on itself (`crates/tianheng/tests/self_governance.rs`) — eating
its own dog food, now across crate boundaries.

**A dimension is a crate born when built** (drift law at crate granularity), and the user
selects governance by depending on the dimensions they want:
- **`hunyi` (渾儀)** — AST/semantic observation (`syn`). **Built (v0.1.0):**
  signature-coupling (a module's public API must not *expose* a forbidden type), plus
  trait-impl locality, visibility, and forbidden-marker boundaries. The heavy `syn`
  dependency is quarantined here, never in the core.
- **`louke` (漏刻)** — runtime observation. **Built (v0.1.0):** origin-assertion (a
  declared seam's `only_origins` allowlist), in two faces — the prod probe
  (`assert_boundary!`, fail-closed, a structured event by default, panic opt-in) and the
  `audit_probe_coverage` CI face, composed into `tianheng check`. Ships into the production
  binary; hot path is std-only, depends on 璇璣 only. (Design gate resolved — see Decisions.)

**The observatory vocabulary (manifested in governance).** The three observation
dimensions — 圭表 (static), 渾儀 (semantic), 漏刻 (runtime) — are the **三儀** (the three
instruments): *what* Tianheng measures; each is a crate born when built, each adds a new
drift type. The governance & observability layer — 垂象 (the reaction surface), 實錄 (the
baseline), 校讎 (the amendment flow) — are the **三司** (the three offices): *how* a reaction
is surfaced, recorded, and amended (see `BACKLOG.md`). 儀 measures, 司 administers — the
三儀 add what is observed, the 三司 wrap the reaction. Both are crate-or-convention as their
nature dictates, never named before their reaction exists.

## Naming — narrative, with meaning in the SSOT

Crate and concept names are **coined / narrative** (圭表, 渾儀, 漏刻), in the celery/kombu
tradition: a name is a stable handle, not a self-description. Meaning lives in the
authoritative **metadata SSOT** (each crate's `description` + docs) — fitting for a tool
whose own thesis is "the source is the SSOT; names are projections." The brand `tianheng`
(天衡) and the bedrock `xuanji` (璇璣) split the one master instrument, 璿璣玉衡: 璣 → 璇璣,
the jade pivot every measurement aligns to; 衡 → 天衡, the balance that weighs declared
against observed. The brand is a star (玉衡, in the Dipper's handle), not an instrument — so
it sits cleanly above the 三儀 it wields, sharing no name with any of them.

## Decisions

Record significant decisions here (the *why*; specs and code carry the *what*).

- **Reborn from modou as a crate family.** modou is frozen/complete at its own `0.1.1`;
  Tianheng starts fresh (clean git history, clean SemVer from `0.1.0`) rather than
  expanding modou's single crate into a god crate. The runtime dimension *must* be a
  separate crate (it ships into production and must stay light), so a family is the
  destiny — but members are born only when built.
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
  its partial coverage — bare path expressions, macro-generated imports, and
  `#[path = "…"]`-remapped modules are out of scope — is acceptable because the drift law
  only enforces what is observed. (A `#[path]` attribute moves a `mod name;` to a
  non-conventional file; the token scanner maps modules by their conventional path, so a
  remapped module's imports are not observed and the module is not governable — the same
  stated partial-coverage bound as inline and macro-generated items. Closing it would
  require reading attributes, an AST-class amendment, not a silent trade.) Comments and
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
  Adopting a real parser (`syn`) would resolve all of this for free but would break the
  dependency-light core (the `serde_json`-only self-law); that is an amendment, not a
  silent trade. A boundary's governed *target* is file-based: an inline `mod name { … }`
  is reachable for import attribution but owns no file, so it cannot be a target — a
  boundary on one fails loud with a self-describing constitution error (exit 2), distinct
  from an unknown-module typo, never a silent pass. Governing inline modules as targets is
  a deliberate non-goal here; if ever wanted it is a separate amendment.
- **`xuanji` is an internal refactor, not a spec'd capability.** When the second
  dimension (渾儀) is built it needs the shared reaction DSL — `Severity`, `Baseline`,
  `Violation`, `Report`, `Outcome` — without `guibiao`'s static engine, so those leaf
  types extract into a `xuanji` crate, re-exported from `guibiao` to preserve its
  public API. The crate is **`serde_json`-only**: `Baseline` *is* a JSON snapshot and the
  per-type renderings (a `Violation`/`Outcome` → JSON value, baseline (de)serialization) are
  intrinsic to those types, so they move with them; `serde_json` is the family's one
  sanctioned dependency. But the **report/constitution assembly stays in `guibiao`/the
  shell**: `report_json` folds in `Coverage` (`workspace_crates`/`uncovered`) and stale
  baseline entries, and `constitution_json` walks `Boundary`/`Rule`/`DependencyKind` — all
  static-dimension concepts that must not leak into the dimension-agnostic model. So the
  split is *per-type serialization in the model, document assembly out of it*. This is a
  behavior-preserving prep step that changes no requirement, so — following modou's line
  that adopter-facing reactions live in specs while Tianheng's own architecture lives here
  and in `self_governance.rs` — it is **not** an OpenSpec capability change. Its invariants
  (model `serde_json`-only and below every dimension; dependency points model ← engine ←
  shell, never the reverse) are enforced as `cargo test` self-governance boundaries, the
  same way the `engine ⊥ shell` law already is. `guibiao`'s *external*-dependency bound
  stays `{serde_json}`; its self-law uses the stricter `restrict_dependencies_to` (which
  governs internal paths too), so it was amended — by deliberate, human-reviewed change to
  `self_governance.rs` — to `["serde_json", "xuanji"]`, naming the one internal path the
  family split requires. 璇璣's own boundary `restrict_dependencies_to(["serde_json"])`
  keeps it beneath every dimension (no workspace member below it).
- **The semantic capability-admission test (the gate against lints).** `syn` makes
  opinionated checks trivial to write ("no `unwrap`", "fns ≤ 50 lines"), every one forbidden
  by the not-a-lint contract. So a semantic capability is admissible as a 渾儀 reaction
  **iff all three hold**: (1) **declarative, not lint** — the constraint has *no universal
  right answer* (two sane projects could declare the opposite and both be correct), so it is
  the developer's intent, not the tool's opinion; (2) **no *essential* gap** — its full
  observation surface is reachable from the local-crate AST, tolerating only *incidental*
  gaps defined by their nature (everything resolvable always reacts; the unresolved
  remainder is a stated bound, never silently passed) and never an *essential* one
  (downstream crates, inferred auto-traits, the transitive call closure), which would make
  it a false-negative engine; (3) **anchorable** — the target is a `syn`-resolvable element,
  and an unresolvable anchor is a constitution error (exit 2), never a silent pass. The
  first admitted capability is **signature-coupling** (a module's public API must not
  *expose* a forbidden type — the complement of import-governance, the case that provably
  earns the AST). Also admitted and now built (v0.1.0):
  local trait-impl surface (`only_implemented_in`), visibility (`must_not_declare_pub`), and
  forbidden-marker boundaries — each born when built. **Rejected**, as explicit non-goals with their reason:
  `Send`/`Sync` constraints (auto-traits are inferred, never written), external trait
  sealing (downstream crates are outside the scan), and transitive effect-purity ("no I/O
  anywhere reachable") — each has an *essential* gap. This test is the standing gate: a new
  semantic capability passes all three in writing, or it is a lint and does not belong.
- **Name resolution is a 渾儀-internal shared layer (`hunyi::resolve`), not 璇璣's and not
  cross-dimension.** Resolution maps a path as written to a canonical item — it is
  *observation*, so it belongs to a dimension, never to 璇璣 (the `serde_json`-only,
  dimension-agnostic reaction model that "holds no observation engine"). It is also not a
  cross-dimension facility: 圭表 resolves with a `syn`-free token scanner (to keep the
  dependency-light core) while 渾儀 resolves with the AST, so a shared resolver would force
  `syn` into the core — the very law that quarantines it. The two resolvers are therefore
  intentionally separate; within 渾儀 the resolver is shared by every semantic capability
  (signature-coupling and trait-impl-locality), resolving via in-scope `use`s,
  `crate`/`self`/`super`, an opt-in bare-name-against-current-module fallback (impl-locality
  needs it; exposure declines it), and local `pub use` re-export chains. This sharing closed
  a re-export false negative in signature-coupling (the contract's one forbidden bug) when
  the second capability was built. Findings are rendered by hand-rolled path/type
  stringification, never `quote`/`syn`'s `printing` feature, so the 渾儀 dependency allowlist
  (`{serde_json, syn, xuanji}`) is untouched.
- **漏刻 (runtime) is identity-coherent; its design gate is resolved on identity, pending
  only an overhead spike.** A design-gate exploration found that the runtime dimension does
  *not* require amending the project's principles — two apparent amendments dissolve:
  (1) **Instrumentation is entailed, not a departure.** The runtime dimension ships into the
  user's binary (the very reason for the modou→tianheng rebirth into a crate family), and the
  concrete type behind a `dyn Trait` is *physically* unobservable from outside — so runtime
  governance needs in-binary probes + `TypeId` origin opt-in. The "source is SSOT, observe
  don't instrument" stance is properly read as *don't instrument **gratuitously***: the
  semantic dimension's rejection of `#[sealed]` was right because outside-observation was
  available there; for runtime it is not, so minimal instrumentation is the consistent
  application, not a reversal.
  (2) **璇璣 is the shared *measure* (`Violation`/`Severity`), and `Outcome`/exit-code is one
  *projection* of it — not the measure itself.** The CI dimensions (圭表, 渾儀) project the
  measure as a process exit code; 漏刻 projects the *same* `Violation` as a structured runtime
  event (`Violation::to_json` — an audit-log record *is* JSON, so 漏刻's dependence on the
  `serde_json`-bearing 璇璣 is justified on the **cold** path, while the **hot** path —
  `TypeId`→origin lookup — stays dependency-light). So 璇璣 remains "the pivot every dimension
  aligns to"; the name's narrative holds, and 漏刻 (clepsydra — the flow/time instrument)
  completes the 三儀 rather than straining it.
  Consequently 漏刻 has **two faces**: a *prod face* (probes emit `Violation` events; default
  emit/audit, `panic` opt-in — never crash prod on a false positive) and a *CI face* (a static
  scan that every declared `RuntimeBoundary` has at least one probe, projected as an exit code
  through the 天衡 shell — closing the "declared but never enforced" coverage gap). The
  **runtime admission test** mirrors the semantic one, with a sharpened criterion (2): a
  runtime capability is admissible iff (1) declarative-not-lint (an origin allowlist is
  intent), (2) **fail-closed — an unknown/unregistered origin reacts, never silently passes**
  (else it is the rejected false-negative engine), (3) anchorable to a declared, named seam
  (an undeclared seam name is a constitution error). First capability when built:
  **origin-assertion** (a declared seam's `only_origins` allowlist). Still **rejected**:
  runtime capability/effect drift ("no I/O reachable") — a runtime policy engine, an explicit
  non-goal. What remains before a propose is **engineering, not identity**: a hot-path
  **overhead spike** (`TypeId`/registry/`dyn` downcast cost). `louke` is still born only when
  built — no stub, no names before their reaction. **(Built in v0.1.0** as origin-assertion,
  both faces — prod probe + the `audit_probe_coverage` CI scan that closes the otherwise-
  *essential* unprobed-seam gap.**)** Two honesties this dimension forces, recorded here:
  (a) runtime observation is **minimally prescriptive** — the user instruments trait/type/seam
  surfaces (`: louke::Tracked`, `register_origin!`, `assert_boundary!`), which the CI dimensions
  never do; "not a framework you build inside" holds for 圭表/渾儀 but relaxes to its irreducible
  minimum for 漏刻 (the cost of observing what is invisible from outside), and origin is **observed**
  (`register_origin!` captures `module_path!()`), not a self-asserted label. (b) The bright line
  against the rejected runtime-policy-engine: louke's registries hold **static label allowlists
  only, never predicates** — no dynamic conditions, effects, or I/O-reachability.
- **The shell takes the 渾儀 dimension as one `SemanticBoundaries` unit, not one parameter per
  capability.** As the semantic dimension grew (signature-coupling, trait-impl-locality,
  visibility), the shell `run`/`dispatch` accumulated one `&[…]` slice per capability — by the
  fourth, an unwieldy positional list where empty slices are easy to misorder. The semantic
  boundaries are gathered into a `hunyi::SemanticBoundaries` container (one field per
  capability), and the shell composes them via `hunyi::check_all` with a single `cargo metadata`
  read. `run(constitution, &SemanticBoundaries, args)` is then stable as further semantic
  capabilities are added (they extend the container, not the signature). This is a
  **behavior-preserving facade refactor** — the same dimensions compose into the same reaction,
  no capability's requirements change — so, like the 璇璣 extraction, it is **not** an OpenSpec
  capability change; it is recorded here and kept honest by the existing tests and
  `self_governance` (the container lives in 渾儀, owns only its own sub-capabilities, and does
  not couple across the 三儀). The per-capability `check_*` entries remain for direct use.
- **漏刻's CI face is behind the non-default `audit` feature, not a 5th crate.** The runtime
  dimension ships into production and must stay light, but its CI face (`audit_probe_coverage` +
  the hand-rolled source scanner) is build-time-only code. Rather than a separate `louke-ci` crate
  — which would be a crate *born without a new reaction* (against the drift law) and would couple a
  detached scanner to `louke`'s own `assert_boundary!` spelling — the CI face is gated behind a
  non-default `audit` Cargo feature. A production binary depending on `louke` for the hot path then
  compiles **zero** scanner code (guaranteed by `cfg`, not left to dead-code elimination), while the
  天衡 shell enables `louke/audit` to run the audit inside `check`. Two honesties: (a) the gated
  scanner's tests compile only with the feature on, so CI runs `test`/`clippy`/`doc` with
  `--all-features` (a plain `--workspace` would silently `cfg` them out — strictly worse than no
  gate, since they guard the *one forbidden bug*), and keeps a default-feature `build`+`clippy` to
  guard the prod-light config has no unused items; (b) `tianheng` is a CI/dev-only tool —
  co-depending on it from a production binary unifies the `audit` feature back ON, voiding the
  zero-scanner-in-prod property. This is a packaging/architecture refactor (no capability's
  requirements change beyond the CI face being feature-scoped) — like the 璇璣 extraction it is
  recorded here, not an OpenSpec change, and kept honest by the tests and `self_governance`. Two
  scanner false negatives were fixed in the same pass: nested block comments (Rust comments nest;
  a probe inside one is commented out and must not count) and non-`()` macro delimiters
  (`{}`/`[]`); both are now reacted to, with the lexical-scan `cfg`-blindness recorded as a stated
  bound in `audit_probe_coverage`'s doc and the spec.
