# 天衡 / Tianheng

**懸衡以待,失衡即應。** — *Hold the balance ready; react the moment it tips.*

> 天衡 (the celestial balance) weighs the shape you *declared* against the shape the
> code *is*; the moment they no longer balance, it reacts. **Govern by reaction, not
> instruction.**

Tianheng is a Rust-native **reactive architectural-governance** framework whose static core
(圭表) is derived from [`modou`](https://github.com/tacticaldoll/modou) — a still-living,
independently-developed sibling project, not one Tianheng supersedes. It does not run your app
and it does not instruct your agent. Developers and agents propose change; Tianheng uses compiler/CI
and runtime *reactions* to keep architectural shape from drifting.

> **Status: experimental — pre-1.0.** The public faces are still settling; the family is held at
> `0.1.x` deliberately (see [`CHANGELOG.md`](CHANGELOG.md)) until real adoption pressure says which
> become long-term contracts. Within `0.1.x` no release intentionally breaks the adopter-written
> builder (`Constitution` / the boundary DSL / `run`).

## Why reaction, not instruction

Architectural intent — "the core must not depend on adapters" — used to live in human
review. An AI agent writes fluent, locally-plausible code without holding that intent, so
it erodes the shape it does not understand, and *instructing* it cannot bind an agent that
has no understanding. Tianheng crystallizes the human's intent into a **non-bypassable
reaction**: neither the agent nor Tianheng needs to understand for the law to hold.

## Start from the intent

Choose the architectural fact you need to keep true, then select the instrument that can actually
observe it. Tianheng supplies no built-in "best practice" policy: every boundary names *your*
target, rule, and forward-looking reason.

| Intent | Declare | What reacts | Deliberate bound |
|---|---|---|---|
| the domain depends inward, never on infrastructure | `ModuleBoundary::must_not_import` | a written internal `use` edge | not a runtime call graph or macro-generated import |
| only the facade may reach an internal module | `ModuleBoundary::must_only_be_imported_by` | an importer outside the closed allowlist | file/module imports, not reflective access |
| a public seam must not leak an implementation type | `SemanticBoundary::must_not_expose` | the forbidden type in an observed public type position | not a function-body call site |
| an external crate belongs only under an adapter/FFI subtree | `ModuleBoundary::confine_external_crate` | its written `use` outside that subtree | not dynamic loading or resolved supply-chain provenance |
| a core subtree stays synchronous and clock-free | `Constitution::sans_io_pure` | inline clock reads and exposed `async fn`s | only the explicit prefixes/verbs you provide; nothing baked in |
| only registered adapter origins cross a runtime seam | `RuntimeBoundary::only_origins` | a live concrete type crossing the probed seam | not general effect or I/O reachability |

The focused, copyable forms live in [`COOKBOOK.md`](COOKBOOK.md). Read each rule's observation
bound with its recipe: a clean result means no violation was found **within that declared
perimeter**, never that Tianheng understood surfaces it does not observe.

## A declared boundary

**Phase 0 — lock one seam.** The smallest useful law is a single boundary an agent copies by
reflex: a leaf crate that must not couple to its siblings, enforced in CI from the first commit.

```rust
use tianheng::prelude::*;

fn constitution() -> Constitution {
    Constitution::new("my-project").boundary(
        CrateBoundary::crate_("my-core")
            .forbid_all_workspace_dependencies()
            .because("my-core is a leaf; it must not couple to a sibling crate"),
    )
}

fn main() -> std::process::ExitCode {
    tianheng::run(&constitution(), std::env::args())
}
```

Wire it into CI once — emit SARIF for code-scanning; the redirect preserves the reaction's own
exit code, so a violation still fails the job (only a `| jq`-style pipeline would swallow it, the
one forbidden false negative — see the annotation recipe below):

```sh
your-binary check --manifest-path Cargo.toml --format sarif > tianheng.sarif
```

That is the whole on-ramp: one declared seam, enforced. Grow it by adding boundaries to the same
`Constitution` — the full shape carries every dimension from the one source of truth:

```rust
use tianheng::prelude::*;

// One declared constitution carries every dimension — the single source of truth.
fn constitution() -> Constitution {
    Constitution::new("my-project")
        // 圭表 (static): imports & dependencies
        .boundary(
            CrateBoundary::crate_("my-core")
                .deny_external_dependencies()
                .because("my-core must stay dependency-light"),
        )
        .boundary(
            ModuleBoundary::in_crate("my-app")
                .module("crate::kernel")
                .must_not_import("crate::projection")
                .because("the kernel must not depend on a projection"),
        )
        // 渾儀 (semantic): folded in via typed adders (none in this minimal example)
        // .signature_boundary(SemanticBoundary::in_crate("my-app")…)
        // 漏刻 (runtime): a declared seam, audited for probe coverage at CI
        // .runtime(RuntimeBoundary::at("domain-entry").only_origins(["my_app::domain"]).because(…))
}

fn main() -> std::process::ExitCode {
    // One declaration in; the 三儀 compose into the one reaction.
    tianheng::run(&constitution(), std::env::args())
}
```

`your-binary check --manifest-path path/to/Cargo.toml` reacts against *your* constitution:
exit `0` (clean / warn-only / fully baselined), `1` (enforced violation), `2`
(constitution/scan error). `list` projects the declared constitution and never reacts.

### Adopt without a red wall

Severity and baseline are two independent entry paths, not three mandatory stages in one sequence:

- **Greenfield or observation period — warn first.** Add `.warn()` before `.because(...)`; the
  boundary reports what it sees but exits `0`. Remove `.warn()` when the team is ready to gate.
- **Existing codebase — enforce new drift now.** Keep the boundary enforced, snapshot only the
  violations that already exist, commit that generated baseline, and gate every subsequent check:

```sh
# Recording is observation, not judgment: writes the current identities and exits 0.
your-binary check --manifest-path Cargo.toml \
  --write-baseline .tianheng-baseline.json

# CI: known identities stay green; a new enforce violation exits 1.
your-binary check --manifest-path Cargo.toml \
  --baseline .tianheng-baseline.json
```

When a violation is fixed, gate mode reports its baseline entry as stale; regenerate the snapshot
with `--write-baseline` to remove resolved entries. Rewriting preserves hand-added `owner` and
`tracker` metadata for identities that still exist. New baselines are version 2: a violation's
identity is `(target, rule, finding_key)`, while the human `finding`, `file`, `anchor`, reason, and
severity do not churn it. Version-1 text baselines remain readable and match by their old exact
`(target, rule, finding)` triple until the next write upgrades them.

**CI / agent visibility.** `check --format json` is the machine contract; `check --format sarif`
emits a vendor-neutral SARIF 2.1.0 document that GitHub code-scanning (and other tools) inline
onto a PR. In every machine projection a violation's **`reason` is the repair direction** an agent
fixes toward — the declared intent the boundary protects — *not* the `rule` label (the rule names
what tripped; the reason says why, and so where to go). Repair toward the reason; never weaken the
boundary to pass. Each violation also carries a structured **`polarity`** — the repair *kind*:
`deny_breach` (remove the offending edge) or `allowlist_gap` (remove it, or widen/declare the
intent) — and, when the boundary declares one, an **`anchor`** pointing at the durable decision it
protects; a baseline entry may likewise carry `owner` / `tracker` metadata, preserved across
re-baselining. There is deliberately no GitHub-specific `--format`: turning the reaction into one CI
vendor's annotations is a harness step, not a tool format. For GitHub `::error::` inline
annotations without SARIF upload, convert the JSON report in a CI step — but **preserve the
reaction's exit code** (a naïve `check | jq` pipeline would exit with `jq`'s status and let a
violation pass green, the one forbidden false negative). Capture, annotate, then re-exit with
Tianheng's own code:

```sh
# `|| status=$?` keeps the capture from aborting under GitHub Actions' default
# `bash -eo pipefail`: a violation (check exit 1) must still print annotations, then fail the step.
status=0
report=$(your-binary check --manifest-path Cargo.toml --format json) || status=$?
printf '%s\n' "$report" \
  | jq -r '.violations[] | "::error::\(.reason) (rule: \(.rule), found: \(.finding))"'
exit "$status"   # 0 clean · 1 enforced violation · 2 constitution/scan error
```

> The published `tianheng` binary is a *demo* bound to a sample constitution (it governs a
> crate named `example-core`). Tianheng is consumed as a **library**: declare your own
> constitution and expose your own binary, as above.

To put your declared law into an AI agent's context, project it to Markdown and write it
where the agent will read it — `constitution_markdown` is the library primitive behind
`list --format markdown` (its layout is human/agent-readable and may evolve; use the JSON
projection for a stable machine contract):

```rust
let md = tianheng::constitution_markdown(&constitution());
std::fs::write("AGENTS.my-project-law.md", md)?;
```

**Recommended agent context** (orientation, not a reaction — keep it short): point your agent
at, in order,

1. your `AGENTS.md` (the working agreement),
2. the generated `AGENTS.<project>-law.md` (your enforced law, in imitable form),
3. the relevant OpenSpec specs (the capability being touched),
4. the code.

The law's per-boundary detail lives in the generated projection, requirement detail in the specs —
so the entry docs stay short and the agent reads the law where it is densest and most imitable.

**Gate the projection so it cannot drift.** A written-out projection is a *copy* of the law;
if it is hand-maintained (or generated once and forgotten) it silently rots as the constitution
changes — and a stale law in the agent's context pulls it the wrong way (潛移 in reverse). Close
that with a `cargo test` that regenerates the projection and byte-compares it to the committed
file, so CI fails the moment they diverge — the same staleness gate Tianheng runs on its own
`AGENTS.self-law.md`:

```rust
#[test]
fn agent_law_projection_is_fresh() {
    let fresh = tianheng::constitution_markdown(&constitution());
    // The caller reads `BLESS`; the helper is a pure function of its arguments.
    // `BLESS=1 cargo test` regenerates after a deliberate, reviewed change to the law.
    tianheng::projection_gate(
        &fresh,
        std::path::Path::new("AGENTS.my-project-law.md"),
        "BLESS=1 cargo test",
        std::env::var_os("BLESS").is_some(),
    )
    .unwrap();
}
```

`projection_gate` is the same byte-check reaction Tianheng runs on its own `AGENTS.self-law.md`:
under bless it rewrites the artifact (creating parent dirs); otherwise it fails — naming the path
and the regenerate command — when the file drifts, is missing, or is unreadable. This makes the
projection a **byte-checked** artifact rather than prose you must remember to update: the reaction
(a failed test) keeps the imitable surface honest, so an adopter's own agent-context enjoys the same
non-bypassable freshness as its declared boundaries. (The **full adopter-facing generator** and a
`list-self`-style **CLI** both stay deferred — see `BACKLOG.md`, 潛移 — because the projection
primitive plus this reusable gate already close the drift; a generator/CLI would only add
adopter-workflow weight and tangle the demo-vs-self-law story.)

## The instruments (三儀) — observation dimensions

Tianheng is a **crate family**. You select governance by depending on the dimensions you
want; each is real drift (declared vs. observed), never a style lint. The three are
measuring instruments — each reads a different surface of the code.

| 儀 Instrument | Crate | Observes | Observation source | Status |
|---|---|---|---|---|
| 圭表 gnomon (static) | `guibiao` | the cast shadow: imports, dependencies & their declared source kind, and inline symbol-path calls | `cargo metadata` + source `use` / symbol scan | **v0.1.x:** static core, declared dependency-source boundaries, module-source hardening, inbound allowlist depth, external-crate (FFI/platform-vocabulary) confinement, inline-symbol-path (clock-free) confinement |
| 渾儀 armillary (semantic) | `hunyi` | type exposure (incl. public `pub use` re-exports and the opt-in trait-impl surface), impl locality, a `pub`-visibility ceiling, forbidden markers, `unsafe`-confinement, `dyn` & `impl Trait` (existential) exposure (each shape-only & named-operand) & `async fn` (implicit existential) exposure (opt-in whole-subtree) | AST (`syn`) | **v0.1.x:** semantic boundary family plus external-crate/re-export/alias hardening |
| 漏刻 clepsydra (runtime) | `louke` | flow: the concrete type behind a `dyn Trait` crossing a seam | runtime `TypeId` / observed origin | **v0.1.x:** origin-assertion, CI probe coverage, escaped-literal and macro-body audit hardening |

**What the instruments do NOT see — avoid over-inferring.** Each reads one surface, and the gaps
between them are deliberate, not oversights. 圭表 reads *declared* dependencies and *written*
import/call paths — never the resolved dependency graph (cargo-deny's lane) or a runtime value. 渾儀
reads the *AST*: an exposure rule observes the **types and traits named on a signature**, a
forbidden-marker rule the **derives/impls on a type** — never a **call site**, so it cannot see
"this function *reads* the clock" (that is
圭表's `must_not_call_inline`). 漏刻 sees a concrete type crossing a seam **at runtime**, which no
static pass can. A capability exists only where a dimension observes it: where no instrument reads a
surface, Tianheng makes **no claim** about it — a stated bound, never a silent pass. Read a rule's
name as *what it observes*, not what it might intuitively imply.

**漏刻's two faces, one declared source.** The runtime boundaries you declare in the
constitution are projected two ways. At CI, `tianheng check` audits that every declared seam
has an `assert_boundary!` probe (and every probe a declared seam) — the **CI face**. In your
binary, the **prod face** reacts fail-closed at the seam. Both read the *same* declared
objects: at startup you install them straight from the constitution, so the two faces cannot
drift apart —

```rust
// prod startup, in your binary (louke is a direct dependency — its macros live there):
louke::install(
    constitution().runtime_boundaries().iter().cloned(),
    [louke::register_origin!(MyType) /* … */],
);
// then at each seam: louke::assert_boundary!("domain-entry", obj);
```

**渾儀's depth stair — start shape-only, tighten to a named operand.** A semantic boundary is
declared at the same seam in two rungs: forbid the *shape* first, then narrow to a *named* trait
once the intent is precise. The same stair applies to `impl Trait` (`must_not_expose_impl_trait`
→ `must_not_expose_impl_trait_of`) and, for the implicit existential, `must_not_expose_async_fn`.

```rust
Constitution::new("my-project")
    // Rung 1 — shape-only: the core seam must not leak ANY dyn (no dynamic dispatch at the seam).
    .dyn_trait_boundary(
        DynTraitBoundary::in_crate("my-core")
            .module("crate::core")
            .must_not_expose_dyn()
            .because("the core's public seam is statically dispatched"),
    )
    // Rung 2 — operand-scoped: allow a std `dyn Error`, but never a `dyn` of our own Port.
    .dyn_trait_boundary(
        DynTraitBoundary::in_crate("my-core")
            .module("crate::adapters")
            .must_not_expose_dyn_of(["crate::ports::Port"])
            .because("adapters may surface std errors but must not leak a dyn Port"),
    )
```

An empty operand set degenerates to shape-only (any `dyn`) — a loud over-reaction, never a
silent no-op — so a mis-declared narrowing can never become a false negative.

**More semantic depths, and a shell-composed profile.** Beyond the exposure stair: a **visibility
ceiling** (`max_visibility(Crate|Super|Module)` — an item declared more visible than the ceiling
reacts) and **`unsafe`-confinement** (`UnsafeBoundary::only_under(["crate::ffi"])` — confining
`unsafe` to a declared subtree, governing *where* it lives, the architectural intent
`#![forbid(unsafe_code)]` cannot express). Because a profile can span dimensions, the 天衡 shell
composes them — **`sans_io_pure`** folds a clock-free (圭表 `must_not_call_inline`) and a
synchronous-API (渾儀 `must_not_expose_async_fn`, whole-subtree) boundary into one declaration
(三儀 ⊥ 三儀: a dimension never composes its sibling, only the shell does).

Beneath the dimensions sits **`xuanji` (璇璣) — the 底**: the dimension-agnostic
**reaction model** (`Severity`, `BoundaryKind`, `Polarity`, `Violation`, `Report`, `Baseline`,
`Outcome`) every dimension reacts in. It is `serde_json`-only, renders **no verdict** — it holds the
*measure* but never the react itself (comparing declared against observed lives in the
dimensions and the shell) — and depends on no workspace member, so a new dimension reuses the
reaction vocabulary without dragging in another dimension's engine.

Beside it sits **`xingbiao` (星表)** — the shared *declared-workspace-data* substrate: it reads
`cargo metadata` (`serde_json`-only) so the static and semantic dimensions observe the workspace
through **one** reader, not two hand-copied twins that drift apart. Like `xuanji` it is below the
dimensions and depends on no other workspace member; **unlike** `xuanji` it *observes* (does IO) —
so it is a substrate, not the measure-only model. A dimension depending on either shared base is a
downward edge, never a cross-dimension one.

A dimension's crate is **born when it is built** — never pre-created empty. The heavy
dependencies (AST, runtime) are quarantined to their own crates; the `guibiao` core's only
*external* dependency stays `serde_json` (internally it depends only on the shared `serde_json`-only
bases `xuanji` and `xingbiao`). See
[`BACKLOG.md`](BACKLOG.md) for the deferred phases (their observation sources and open
design questions) and the governance/observability layer.

Tianheng governs **itself** with its own reaction: the live self-law is declared in
`crates/tianheng/tests/self_governance.rs`, projected into `AGENTS.self-law.md`, and enforced as
a `cargo test` gate.

## Adoption & stability

**A ladder, not a wall.** Adopt one instrument as an on-ramp and graduate to the composed
constitution — a single 儀 → the suite is the funnel, not a dilution. Onboarding an existing
codebase never starts red: declare a boundary at `.warn()` (reported, never gating — exit `0`),
`Baseline::of(&report)` to grandfather the violations already there (they stay green while *new*
drift reacts), then tighten to `enforce`. Two axes — severity and baseline — either of which lands
the law before you land the fixes. See the runnable [`examples/`](examples/): 圭表 and 渾儀
standalone, the composed all-three funnel, plus focused demos of `unsafe`-confinement and the
`sans_io_pure` profile — and [`COOKBOOK.md`](COOKBOOK.md) for common governance intents translated
into boundaries.

**What stays stable across the pre-1.0 line.**

- **The wire contract is `xuanji`.** Every dimension reacts in one shared model, and its JSON — the
  `--format json` report and the `Baseline` snapshot — is the versioned, machine-facing contract (a
  `Baseline` *is* a JSON snapshot). Presentation changes — a coloured terminal render, the SARIF
  projection — never change the verdict or the exit code.
- **A violation's identity is `(target, rule, finding_key)`.** The key is a dimension-owned fact
  code plus named observed values; the human `finding`, `file`, `anchor`, and `polarity` are
  *presentation/metadata*, not identity. Relocating code, attaching an anchor, or improving finding
  wording therefore does not turn a version-2 baselined violation new.
- **The adopter-written builder does not break in `0.1.x`.** `Constitution`, the boundary DSL, and
  `run` are the surface you write against; the pre-1.0 churn is quarantined to internal faces.

## Non-goals

Not active code-shaping/generation, not a prescriptive framework you build inside, not a
schema crate, not a lint, not a universal graph API, not a supply-chain policy engine. No
TOML/Markdown for the constitution. Each dimension keeps its own observation source; nothing
is named before its reaction exists.

**Relationship to cargo-deny.** cargo-deny owns resolved, whole-graph supply-chain policy.
Tianheng owns the complementary declared, per-target architectural layer; see `PROJECT.md` for the
dependency-source split.

## License

Licensed under either of [Apache-2.0](LICENSE-APACHE) or [MIT](LICENSE-MIT), at your option.
