# Cookbook — governance intents as declared boundaries

Tianheng has no policy-import format on purpose: you don't translate a foreign rule language into
Tianheng, you **declare the intent directly** as a boundary in Rust. That declaration *is* the
imitable surface (潛移) — an agent reads it and generates code that fits. This cookbook is a set of
common intents and the boundary that expresses each; copy a recipe and change the paths and the
`.because(...)`.

Every recipe folds into one `Constitution` (the single source of truth), handed to the shell:

```rust
use tianheng::prelude::*;

fn constitution() -> Constitution {
    Constitution::new("my-project")
        // …recipes below…
}

fn main() -> std::process::ExitCode {
    tianheng::run(&constitution(), std::env::args())
}
```

Exit codes are the contract: `0` clean / warn-only / fully baselined · `1` enforced violation ·
`2` constitution or scan error. Read a violation's **`reason`** first — it is the repair direction.

> Experimental / pre-1.0. The boundary DSL (the surface these recipes use) is patch-compatible
> within `0.2.x`; a breaking change must earn a new minor. See [`CHANGELOG.md`](CHANGELOG.md).

---

## Find the boundary from the intent

| Intent | Recipe |
|---|---|
| keep domain/core pointing inward | [Keep a layer pure](#keep-a-layer-pure-hexagonal--onion) |
| admit access only through a facade | [Funnel access through one module](#funnel-access-through-one-module-closed-inbound-allowlist) |
| keep a core dependency-light | [Keep a core dependency-light](#keep-a-core-dependency-light) |
| prevent implementation types from escaping through public API | [Don't leak an internal type](#dont-leak-an-internal-type-on-the-public-api) |
| confine FFI/platform vocabulary to an adapter subtree | [Confine platform/FFI vocabulary](#confine-a-platform--ffi-vocabulary-to-one-module) |
| keep a subtree synchronous and free of ambient reads | [Compose a sans-I/O pure core](#compose-a-sans-io-pure-core-clock-free--synchronous-one-declaration) |
| restrict live objects crossing a port seam | [Govern which adapter crosses a seam](#govern-which-adapter-crosses-a-dyn-seam-at-runtime) |
| introduce governance without making an existing project red | [Adopt on a dirty codebase](#adopt-on-a-dirty-codebase-without-a-red-wall) |
| prove a declared boundary still has teeth | [Test that a boundary reacts](#test-that-a-boundary-actually-reacts) |
| publish an imitable Agent Law for your codebase | [Publish an imitable Agent Law](#publish-an-imitable-agent-law-for-your-codebase-three-layer-agent-law) |

The table routes by *observable architectural fact*, not by architecture-fashion label. A recipe's
text immediately below its declaration states what the selected instrument observes; it makes no
claim outside that perimeter.

---

## 圭表 (static) — imports & dependencies

### Keep a layer pure (hexagonal / onion)

*Intent: the domain depends on ports, never on infrastructure.*

```rust
.boundary(
    ModuleBoundary::in_crate("my-app")
        .module("crate::domain")
        .must_not_import("crate::infra")
        .depth(ScanDepth::Subtree)
        .because("the domain stays pure — it never depends on infrastructure"),
)
```

Reacts when any module under `crate::domain` has a `use crate::infra::…`.

### Funnel access through one module (closed inbound allowlist)

*Intent: only the facade may reach the internals.*

```rust
.boundary(
    ModuleBoundary::in_crate("my-app")
        .module("crate::internal")
        .must_only_be_imported_by(["crate::facade"])
        .because("internal is reached only through the facade"),
)
```

Reacts when any importer *other than* `crate::facade` imports `crate::internal`.

### Keep a core dependency-light

*Intent: the core crate pulls no external dependencies.*

```rust
.boundary(
    CrateBoundary::crate_("my-core")
        .deny_external_dependencies()
        .because("my-core is a domain-free core and must stay dependency-light"),
)
```

Reacts when `my-core` declares any external (non-workspace) dependency.

### Forbid a dependency's risky feature (or require `default-features = false`)

*Intent: a crate rule admits a dependency whole; govern which of its **features** you may declare —
the feature surface a plain dependency allowlist cannot see.*

```rust
.boundary(
    CrateBoundary::crate_("app")
        .forbid_feature("some-lib", "unstable")
        .because("production must not opt into some-lib's unstable API surface"),
)
```

`forbid_feature(C, "default")` is the way to require `default-features = false`, and
`restrict_features_of` pins a closed set (an empty allowlist forbids every feature, `default`
included):

```rust
.boundary(
    CrateBoundary::crate_("app")
        .restrict_features_of("some-lib", ["std"])
        .because("some-lib may be declared only with its std feature"),
)
```

It observes the feature request you **author** in `Cargo.toml` — the declared `features` list plus
the `default` pseudo-feature — matched by package name, unioned across a crate's edges. A feature a
dependency enables *transitively*, or one a sibling crate turns on via Cargo feature unification, is
out of scope by design (declared-not-resolved, like every other crate rule).

### Stay publishable to crates.io (declared source hygiene)

*Intent: a crate prepared for crates.io declares only registry/path dependencies — no git.*

```rust
.boundary(
    CrateBoundary::crate_("infra")
        .restrict_dependency_sources_to([SourceKind::Registry, SourceKind::Path])
        .because("infra must publish to crates.io, so it declares no git dependencies"),
)
```

Reacts when `infra` declares a `git = …` dependency. (Hygiene on the *declared* source, not a
`cargo publish` oracle; the resolved-source lane is cargo-deny's.)

### Confine a heavy dependency to the dev table

*Intent: a test-only crate must not leak into the normal dependency table.*

```rust
.boundary(
    CrateBoundary::crate_("my-app")
        .restrict_dependencies_to(["serde", "serde_json"])
        .dependency_kind(DependencyKind::Dev)
        .because("only serde is a dev dependency; nothing else in [dev-dependencies]"),
)
```

A boundary governs exactly one table; declare two to govern `[dependencies]` and
`[dev-dependencies]` both.

### Confine a platform / FFI vocabulary to one module

*Intent: the raw `libc` surface may be used only behind the `ffi` module — nowhere else in the
crate.*

```rust
.boundary(
    ModuleBoundary::in_crate("my-app")
        .module("crate::ffi")
        .confine_external_crate("libc")
        .because("the raw libc surface stays behind the ffi module; the rest of the crate uses the safe wrapper"),
)
```

Reacts when any module *outside* `crate::ffi`'s own subtree declares `use libc::…`. Unlike a
crate-dependency rule (which governs *whether* the crate may depend on `libc`, via `cargo
metadata`), this governs *where within the crate* `libc` may be used — source-observed, so it sees
the importing module. Deliberately not a `cargo publish` / dependency-table check: confining a crate
the app never imports is simply clean. Because it is cfg-blind, it flags a `#[cfg(windows)] use
windows::…` outside its module even when building on Linux — correct, since confinement is a
source-location property independent of the active platform.

### Keep a core clock-free (inject time, don't read it)

*Intent: `crate::core` must not read the ambient wall clock — time is injected. It may still
receive and name `std::time` types; it just must not call into them to read the clock.*

```rust
// Precise (recommended): forbid the READ calls, allow receiving/naming injected time.
.boundary(
    ModuleBoundary::in_crate("my-app")
        .module("crate::core")
        .must_not_call_inline("std::time")
        .ending_with(["now"])
        .because("core reads no wall clock — time is injected, not read"),
)
```

Reacts when `crate::core` (or a submodule) makes an inline **call** whose path resolves under
`std::time` and ends with a declared read verb — `std::time::SystemTime::now()`,
`Instant::now()`, a renamed / bare / `type`-aliased / locally re-exported spelling, or such a call
hidden in a macro body. It does **not** react on a type annotation (`fn tick(now: std::time::Instant)`)
or a constant — the core may *receive* injected time. Two knobs:

- **Bare** `.must_not_call_inline("std::time")` forbids *every* call under the prefix (including
  `Duration::from_secs`) — the safe, no-heuristic default; narrow with `.ending_with([…])` (you own
  any read verb you omit) to a precise read list.
- **`.strict_prefix_only()`** escalates to forbid *any* mention of `std::time` (type annotations and
  constants too) — for a core that must not even name it.

Stated bounds (declared non-observations, never silent passes): a receiver-method or UFCS read
whose type is not in a plain path (`instant.elapsed()`, `<T as Tr>::now()`), an alias defined inside
a macro body, a symbol assembled by a proc-macro, an external-crate re-export, and a path taken as a
value under the default. A glob that could smuggle the surface in (`use std::time::*`, or a module
that re-exports it, globbed) reacts fail-closed.

### Also catch a fully-qualified external-crate call (opt-in `.strict_external()`)

*Intent: the same clock-free core must not read the clock via an external crate either — e.g. a
`chrono::Utc::now()` written in full, with no `use chrono` in scope.*

```rust
.boundary(
    ModuleBoundary::in_crate("my-app")
        .module("crate::core")
        .must_not_call_inline("chrono::Utc")
        .strict_external()
        .because("core reads no wall clock — not even a fully-qualified chrono call"),
)
```

By default `must_not_call_inline` catches a sysroot head (`std::time::…`) but resolves a
fully-qualified *external* head (`chrono::…` with no `use`) as a local path and lets it pass — a
stated bound. `.strict_external()` closes it: a bare head matching a **declared dependency** is
resolved as that crate, so the fully-qualified call reacts (and an external-crate glob
`use chrono::*;` reacts fail-closed). It composes with `.ending_with([…])` / `.strict_prefix_only()`,
and the default (flag off) is byte-identical — opt-in, no baseline churn. Stated bounds under the
flag (**any** prefix): an `extern crate dep as alias;` rename (it catches external calls by the
crate's *real* name, not a local alias — so `chr::Utc::now()` via `extern crate chrono as chr;` is
not observed), a glob-brought name *except via the glob-hazard reaction*, and a `mod` token inside a
macro body. One additional **over-reaction** bound applies **only** under a single-segment bare-crate
prefix (`"rand"`, never a multi-segment `chrono::Utc`): a local `let`/parameter/closure binding, or
the definition site of an associated/nested `fn` named like the crate, may false-positive.

---

## 渾儀 (semantic) — public-API exposure

### Don't leak an internal type on the public API

*Intent: the public API must not expose the database pool.*

```rust
.signature_boundary(
    SemanticBoundary::in_crate("my-app")
        .module("crate::api")
        .must_not_expose("crate::infra::DbPool")
        .because("the public API must not leak the internal database pool"),
)
```

Reacts when a `pub` signature under `crate::api` names `infra::DbPool` — including a
fully-qualified path a token scanner would miss, and a `pub use` re-export.

> **Why deny-shaped, not a "may only expose" allowlist?** Exposure rules name what must **not**
> appear, never a closed set of what may. A positive allowlist would have to enumerate the authored
> public set and diff it against the compiler-derived *reachable closure* — the auto-trait,
> blanket-impl, and multi-hop re-export reachability the AST scan cannot fully see — so its
> "complete" list would drift into false positives on legitimate API. Deny-shape reacts to a
> concrete named leak, which the AST *can* observe. (Import/dependency rules **do** offer closed
> allowlists — `restrict_imports_to`, `must_only_be_imported_by`, `restrict_dependencies_to` —
> because there the observed set *is* the declared set, with no hidden closure.)

### Keep a seam statically dispatched (no `dyn`)

*Intent: the core's public seam leaks no dynamic dispatch; start shape-only, tighten to an operand.*

```rust
.dyn_trait_boundary(
    DynTraitBoundary::in_crate("my-core")
        .module("crate::core")
        .must_not_expose_dyn()                       // rung 1: any exposed dyn reacts
        .because("the core's public seam is statically dispatched"),
)
.dyn_trait_boundary(
    DynTraitBoundary::in_crate("my-core")
        .module("crate::adapters")
        .must_not_expose_dyn_of(["crate::ports::Port"])   // rung 2: only a dyn of our Port
        .because("adapters may surface std errors but must not leak a dyn Port"),
)
```

An empty operand set degenerates to shape-only (a loud over-reaction), never a silent no-op.

### Register a trait's impls in one place (impl locality)

*Intent: every `Command` impl lives under `crate::commands`.*

```rust
.trait_impl_boundary(
    TraitImplBoundary::in_crate("my-app")
        .trait_("crate::Command")
        .only_implemented_in("crate::commands")
        .because("commands are registered in one place"),
)
```

### Keep a module crate-private / free of a marker

```rust
.visibility_boundary(
    VisibilityBoundary::in_crate("my-app")
        .module("crate::internal")
        .must_not_declare_pub()
        .because("internal is crate-private by contract"),
)
.forbidden_marker_boundary(
    ForbiddenMarkerBoundary::in_crate("my-app")
        .module("crate::domain")
        .must_not_acquire("serde::Serialize")
        .because("domain types must not be wire-coupled"),
)
```

`must_not_declare_pub()` is the `max_visibility(VisibilityCeiling::Crate)` sugar — react on bare
`pub`, allow `pub(crate)` and below. For a more tightly sealed layer, name a lower ceiling:

```rust
.visibility_boundary(
    VisibilityBoundary::in_crate("my-app")
        .module("crate::deep")
        .max_visibility(VisibilityCeiling::Super)   // also react on pub(crate); allow pub(super)/below
        .because("this submodule is sealed to its parent"),
)
```

`Super` reacts on anything more visible than `pub(super)`; `Module` reacts on any `pub`-family
keyword (a fully module-private layer). It governs the **declared** keyword, not crate-reachability —
the compiler accepts widening a `pub(crate)` to `pub`; this catches that drift.

### Confine `unsafe` to one auditable subtree

*Intent: all `unsafe` (blocks, `unsafe fn`/`impl`/`trait`, `unsafe extern`) lives under `crate::ffi`;
a site elsewhere reacts, so a reviewer knows where to look.*

```rust
.unsafe_boundary(
    UnsafeBoundary::in_crate("my-app")
        .only_under(["crate::ffi"])
        .because("unsafe lives only behind the ffi module — everywhere else is safe"),
)
```

Reacts on any `unsafe` site outside `crate::ffi` (and beneath it); `.only_under(["crate::ffi",
"crate::simd"])` allows several subtrees. **Confinement only:** for a crate-wide ban use the
compiler's `#![forbid(unsafe_code)]` (stronger — compile-time, unbypassable); an empty or crate-root
`only_under` is a constitution error that says so. It observes the `unsafe` **keyword** — a
`#[unsafe(...)]` attribute, a bare `unsafe fn` pointer type, a plain `extern "C" {}` block (no
keyword; its call sites still react), and macro-generated `unsafe` are stated bounds.

---

## 漏刻 (runtime) — origin governance

### Govern which adapter crosses a `dyn` seam at runtime

*Intent: only the blessed adapter's origin may cross the port seam — enforced against the live
object, which static/semantic analysis cannot see.*

```rust
// in the constitution:
.runtime(
    RuntimeBoundary::at("adapter-seam")
        .only_origins(["my_app::adapters::blessed"])
        .because("only the blessed adapter may cross the port seam"),
)
```

```rust
// in your binary (louke is a direct dependency — its macros live there):
//   trait Adapter: louke::Tracked {}
louke::install(
    constitution().runtime_boundaries().iter().cloned(),
    [louke::register_origin!(BlessedAdapter) /* registered inside its own module */],
);
// at the seam:
//   louke::assert_boundary!("adapter-seam", obj);   // unknown/disallowed origin reacts fail-closed
```

At CI, `tianheng check` audits that every declared seam has a probe (the CI face); in prod the
probe reacts as an **event** by default (`panic` is opt-in). The runtime dimension is a *depth* you
reach through the composed shell, not a standalone on-ramp.

---

## Cross-cutting

### Compose a sans-I/O pure core (clock-free + synchronous, one declaration)

*Intent: `crate::kernel` reads no ambient clock **and** exposes no `async fn` — the two
source-observable axes of a sans-I/O kernel, declared together.*

```rust
.sans_io_pure(
    SansIoPure::in_crate("my-app")
        .module("crate::kernel")
        .reading_clock_via("std::time", ["now"])
        .because("the kernel stays sans-I/O: time is injected, and async lives at the edges"),
)
```

Expands to the 圭表 `must_not_call_inline("std::time").ending_with(["now"])` and 渾儀
`must_not_expose_async_fn().including_submodules()` boundaries on `crate::kernel` — a shell-composed
convenience for the two you would otherwise write by hand (a dimension never composes its sibling; the
天衡 shell does). Add `.warn()` before `.because(...)` to make both advisory (the adoption rung).

**Both halves reach the kernel's whole subtree.** A pure kernel is sans-I/O *throughout*, so the
async half opts into subtree scope: a public `async fn` in **any** module under `crate::kernel`
reacts, not only one at the kernel's own seam. (The clock half is inherently subtree-wide.)

**Scoped honestly to clock + async only.** A core that must also avoid ambient `fs`/`net`/`env` adds
those explicitly (`must_not_call_inline("std::fs")`, `confine_external_crate(...)`); nothing is baked
in — you supply the time prefix and read verbs, so it governs exactly what you declare and no more.

### Forbid `async fn` across a whole subtree (not just one module's seam)

*Intent: no public `async fn` anywhere under `crate::core` — a sync-core/async-edges layering.*

```rust
.async_exposure_boundary(
    AsyncExposureBoundary::in_crate("my-app")
        .module("crate::core")
        .must_not_expose_async_fn()
        .including_submodules()          // descend the whole subtree, not just crate::core's own items
        .because("the core is synchronous throughout; async lives at the edges"),
)
```

By default `must_not_expose_async_fn()` governs the anchored module's **own** seam (one declared
module). `.including_submodules()` descends the anchored module's whole subtree, so an `async fn` in
any descendant reacts too — anchor at `crate` to govern the whole crate. The opt-in is projected in
`list` (a `(including submodules)` marker) only when set; a `#[path]`-remapped module is a stated
bound (the walk descends the declared mod-tree).

### Adopt on a dirty codebase without a red wall

*Intent: land the law now, fix the pre-existing violations over time.* There are two independent
entry paths, chosen by project state — baseline is not a mandatory stage between warn and enforce:

- **Greenfield / observation period — severity.** Declare a boundary at `.warn()`: it is reported
  but does not gate (exit `0`). Remove `.warn()` to promote the same declared law to enforce.
- **Existing dirty codebase — baseline.** Keep the boundary enforced and snapshot the violations
  already there; a fully-baselined report exits `0`, while any *new*, un-baselined enforce
  violation still reacts.

Through the composed `tianheng::run` shell, the baseline path is executable end to end:

```sh
# First adoption: record current violation identities. Recording exits 0.
your-binary check --manifest-path Cargo.toml \
  --write-baseline .tianheng-baseline.json

# Commit the generated file, then make this the CI gate.
your-binary check --manifest-path Cargo.toml \
  --baseline .tianheng-baseline.json
```

Gate mode reports a baseline identity that no longer matches as **stale**; regenerate with
`--write-baseline` after reviewing the fix to ratchet the snapshot down. Rewriting preserves an
existing entry's hand-added `owner` / `tracker` metadata when its identity still exists, adds new
identities without metadata, and drops resolved identities. A missing baseline is an error in gate
mode (exit `2`), never an empty baseline that silently passes.

Refactoring the offending code or improving finding wording does **not** churn a baseline: identity
is `(target, rule_key, fact)`; the human `finding`, `file`, `anchor`, reason, and severity are
presentation/metadata. The baseline document itself is unversioned — it carries a semantic `format`
string, never a numeric `version`. A baseline written before this shape (any file still carrying a
numeric `version`) is unsupported: `--write-baseline` refuses to overwrite it and `--baseline` refuses
to gate against it, both exiting `2` rather than silently upgrading or reinterpreting it. Preserve any
desired `owner`/`tracker` annotations by hand, move or delete the unsupported file, then regenerate
with `--write-baseline`. The runnable
`examples/guibiao-standalone/tests/reaction.rs` proves identity stability and the new-violation
gate; `scripts/test_examples.sh` drives the real CLI write/gate path.

### Test that a boundary actually reacts

*Intent: prove in `cargo test` that a boundary fires on the code it should — and stays clean on the
code it shouldn't — so your governance cannot silently rot.*

Each check entry point takes a manifest path and returns an `Outcome` whose `.exit_code()` is the
contract (`0`/`1`/`2`). Point it at a small **governed fixture crate** committed in your repo — its
own manifest, resolved relative to `CARGO_MANIFEST_DIR`:

```rust
use tianheng::check;   // 圭表 (static); use `check_all` for the 渾儀 (semantic) boundaries

fn manifest() -> std::path::PathBuf {
    std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("Cargo.toml")
}

#[test]
fn the_boundary_reacts() {
    // the fixture crate deliberately violates the law → exit 1
    assert_eq!(check(constitution().static_boundaries(), &manifest()).exit_code(), 1);
}

#[test]
fn a_boundary_that_should_not_fire_stays_clean() {
    // precision: a boundary naming something the fixture does NOT do → exit 0
    let clean = vec![/* a boundary that shouldn't react on this fixture */];
    assert_eq!(check(&clean, &manifest()).exit_code(), 0);
}
```

The **semantic teeth are unit-testable the same way** — `check_all(constitution().semantic_boundaries(), &manifest())`. Both entry points read a manifest **on disk**, so the fixture is a small
committed crate, not an inline string. Tianheng's own `examples/*/tests/reaction.rs` follow this
pattern — one governed fixture crate, asserting the reacting case (exit 1) alongside a green case (a
boundary that shouldn't fire, or `.warn()` / a baseline). Copy one as your starting point.

### Gate coverage in CI — assert every crate is governed

*Intent: workspace coverage is a **pure projection** — it names which workspace members are targeted
by no boundary, but never touches the exit code. Make "every workspace crate carries a boundary" a
hard gate that **you** own.*

`guibiao::check_and_cover` (on `guibiao` — add it as a direct dependency; it is not re-exported from
`tianheng`) returns the `Outcome` plus an `Option<Coverage>` (`Some` whenever the workspace metadata
was read). `Coverage` names the `total` workspace members and the `uncovered` ones. Assert
`uncovered` empty, guarding against a vacuous pass:

```rust
let (_outcome, coverage) = guibiao::check_and_cover(constitution().static_boundaries(), &manifest());
let coverage = coverage.expect("workspace metadata is readable in-repo");
assert!(coverage.total > 0, "coverage read no crates — the gate would pass vacuously");
assert!(coverage.uncovered.is_empty(), "ungoverned workspace crates: {:?}", coverage.uncovered);
```

Coverage changes no exit code on its own, so nothing gates unless *you* assert on it — the wall
against a new, ungoverned crate slipping in is one you build deliberately. This is exactly how
Tianheng gates its own coverage in `crates/tianheng/tests/self_governance.rs`.

### Publish an imitable Agent Law for your codebase (Three-Layer Agent Law)

*Intent: publish your declared governance as a 3-layer imitable Agent Law (`AGENTS.md` / `AGENTS.self-law.md`) so AI agents imitate your architecture by gravity (潛移) — and gate it in `cargo test` so the projection never rots.*

The **Three-Layer Agent Law** consists of:
1. **Layer 1: Universal Preamble** — meta-instructions and vocabulary only (`SELF_LAW_PREAMBLE` discipline; never crate-specific or un-reacted architectural claims).
2. **Layer 2: Projection Body** — rendered directly from your declared `Constitution` via `tianheng::constitution_markdown(&constitution())`.
3. **Layer 3: Law Source** — the Rust `constitution()` code, protected by `.github/CODEOWNERS` and verified by `cargo test`.

```rust
use tianheng::prelude::*;

const PREAMBLE: &str = r#"# AGENTS.self-law.md — My Project Law

Working agreement for AI agents and humans. Read this before changing code.

1. **Before changing code — read the declared law.** Read the boundaries below to understand the shape you must not drift.
2. **After changing code — react.** Run `cargo test` or `your-binary check`.
3. **On a violation — repair toward the declared reason.** Read `reason` first (it is the repair direction), then fix the code so the reason holds again. Never weaken a boundary to pass CI.
"#;

#[test]
fn test_agent_law_freshness() {
    GovernanceTest::for_constitution(constitution())
        .assert_clean()                            // Asserts 0 violations returned
        .assert_all_workspace_members_covered()    // Asserts 100% of workspace members are governed
        .assert_projection_fresh_with_preamble("AGENTS.self-law.md", PREAMBLE); // Staleness test + BLESS=1 support
}
```

> **Preamble Discipline (Important):** Keep Layer 1 strictly universal. A preamble MUST NOT contain crate-specific or un-reacted architectural rules (e.g., "all handlers must be in `src/handlers`"). Crate-specific architectural claims belong ONLY in Layer 2 (generated from `constitution()`), where every boundary carries a real, non-bypassable reaction. Un-reacted claims in preambles lead to rotted agent instructions over time.

**Workflow (`BLESS=1` Regeneration):**
Run `BLESS=1 cargo test` to automatically overwrite or update `AGENTS.self-law.md` when you update your `Constitution`. In CI, `cargo test` runs without `BLESS` and fails loudly if the checked-in Markdown file drifts from `constitution()`.
