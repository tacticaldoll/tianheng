# 圭表 / guibiao

**立表見影,依賴無遁。** — *Set the gnomon and the shadow shows — no dependency escapes.*

**The static observation dimension of [Tianheng](https://github.com/tacticaldoll/tianheng) — the gnomon.**

圭表 (the gnomon, reading the cast shadow) is the **dependency-light static core**, derived
from [`modou`](https://github.com/tacticaldoll/modou). It reads the shadow the code casts —
its **imports and dependencies** — from `cargo metadata` and a source `use` scan, compares
against boundaries you declare in Rust, and reacts.

It is a **pure functional core**: no CLI, no filesystem shell. Its only *external* dependency
is `serde_json` (it depends internally on [`xuanji`](https://crates.io/crates/xuanji), the
reaction model).

```rust
use guibiao::{Constitution, CrateBoundary, ModuleBoundary, check};

let constitution = Constitution::new("my-project")
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
    );

// `check` is the pure entry: observe a workspace, return an Outcome.
let outcome = check(&constitution, std::path::Path::new("path/to/Cargo.toml"));
```

Beyond *which* crates a target may depend on (by name, or the external/internal split), a
crate boundary can also restrict the **declared source kind** of its dependencies — the
git-vs-registry-vs-path distinction `cargo metadata` records:

```rust
use guibiao::{CrateBoundary, SourceKind};

// A crate prepared for crates.io declares no git source: its manifest must name only
// registry and path dependencies (an *optional* git dependency blocks publishing too).
let boundary = CrateBoundary::crate_("infra")
    .restrict_dependency_sources_to([SourceKind::Registry, SourceKind::Path])
    .because("infra must publish to crates.io, so its manifest declares no git dependencies");
```

Two **stated bounds** (deliberate, never silently overreached):
- It governs the **declared** source, not the *resolved* one. A registry dependency that
  `[patch]` or `[source] replace-with` redirects to git reads as `registry+…` and does **not**
  violate — correct for manifest hygiene, since `[patch]` is workspace-local and never blocks
  `cargo publish`. Observing the resolved source is cargo-deny's `[sources]` lane, not a
  Tianheng capability.
- It is source-kind **hygiene**, not a `cargo publish` oracle. A `{ git = "…", version = "…" }`
  dependency declares a git source and is flagged even though it would publish successfully;
  the rule classifies by the declared source and does not parse the `version` key.

**The crate-boundary rules** (each declared in Rust, each carrying a `.because(…)` reason and an
optional `.warn()` severity): `deny_external_dependencies` (allow a named exception list),
`forbid_dependency_on([…])`, `restrict_dependencies_to([…])` (a closed allowlist),
`restrict_workspace_dependencies_to([…])` / `forbid_all_workspace_dependencies` (the
crate-to-crate layering surface), and `restrict_dependency_sources_to([…])` (above). **The
module-boundary rules**: `must_not_import`, `restrict_imports_to([…])`, `must_not_be_imported_by`,
`must_only_be_imported_by([…])` (the closed inbound allowlist), and `confine_external_crate` (confine
an external crate's `use` imports to one module subtree — FFI / platform-vocabulary confinement).

By default a crate rule observes the normal `[dependencies]` table; `.dependency_kind(DependencyKind::Dev)`
(or `Build`) targets `[dev-dependencies]` / `[build-dependencies]` instead — a boundary governs
exactly one table, so govern two by declaring two. Dev/build findings carry a ` (dev)` / ` (build)`
suffix so the same dependency governed in two tables stays a distinct finding (a normal-table
finding keeps the bare name, so existing baselines do not churn).

**Stated partial coverage** (never silently passed): the hand-rolled `use` scanner does not
see bare path expressions, macro-generated imports, or `#[path]`-remapped modules — closing
those would require an AST, an amendment, not a silent trade.

The scanner anchors source discovery to Cargo's observed target `src_path` (the lib target,
else a bin target), so custom `[lib] path = "lib.rs"` and bin-only crates are scanned from
the compiled source root rather than the `manifest_dir/src` shortcut. A `#[path]`-remapped
module remains outside this token scanner's coverage and is not governed through a same-named
conventional orphan file.

## Adoption & status

**Experimental — pre-1.0.** Public faces may change until adoption settles them; within `0.1.x` no
release intentionally breaks the adopter-written builder.

Adopt 圭表 on its own — the footprint is just `guibiao` (+ `serde_json`), no `syn` — or graduate to
the composed constitution through the [`tianheng`](https://crates.io/crates/tianheng) shell (which
re-exports this crate's surface): a single 儀 is an on-ramp, the suite is the destination. Onboard
without a red wall — declare at `.warn()`, `Baseline::of(...)` to grandfather an existing codebase,
then enforce. A runnable `guibiao`-standalone example lives under the workspace `examples/`.

## License

Licensed under either of Apache-2.0 or MIT, at your option.
