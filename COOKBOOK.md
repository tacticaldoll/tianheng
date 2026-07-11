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

> Experimental / pre-1.0. The boundary DSL (the surface these recipes use) does not break within
> `0.1.x`; see [`CHANGELOG.md`](CHANGELOG.md).

---

## 圭表 (static) — imports & dependencies

### Keep a layer pure (hexagonal / onion)

*Intent: the domain depends on ports, never on infrastructure.*

```rust
.boundary(
    ModuleBoundary::in_crate("my-app")
        .module("crate::domain")
        .must_not_import("crate::infra")
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

*Intent: land the law now, fix the pre-existing violations over time.* Two axes:

- **Severity** — declare a boundary at `.warn()`: it is reported but does not gate (exit `0`).
- **Baseline** — `Baseline::of(&report)` snapshots the violations already there; a fully-baselined
  report exits `0`, while any *new*, un-baselined violation still reacts. Refactoring the offending
  code to another file does **not** churn the baseline (a violation's identity is
  `(target, rule, finding)`; `file` is metadata).

Start at `warn`, or baseline the existing drift, then tighten to `enforce`.

### Put your declared law into an AI agent's context (kept fresh)

*Intent: the agent reads the law in imitable form, and the copy cannot silently rot.*

```rust
let md = tianheng::constitution_markdown(&constitution());
std::fs::write("AGENTS.my-project-law.md", md)?;
```

Gate it with `tianheng::projection_gate(...)` in a `cargo test` (see the root `README.md`) so CI
fails the moment the written law drifts from the declared one — the same staleness reaction
Tianheng runs on its own `AGENTS.self-law.md`.
