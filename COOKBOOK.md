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
