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
module-boundary rules**: `must_not_import`, `restrict_imports_to([…])`, `must_not_be_imported_by`.

By default a crate rule observes the normal `[dependencies]` table; `.dependency_kind(DependencyKind::Dev)`
(or `Build`) targets `[dev-dependencies]` / `[build-dependencies]` instead — a boundary governs
exactly one table, so govern two by declaring two. Dev/build findings carry a ` (dev)` / ` (build)`
suffix so the same dependency governed in two tables stays a distinct finding (a normal-table
finding keeps the bare name, so existing baselines do not churn).

**Stated partial coverage** (never silently passed): the hand-rolled `use` scanner does not
see bare path expressions, macro-generated imports, or `#[path]`-remapped modules — closing
those would require an AST, an amendment, not a silent trade.

Most adopters consume the static dimension through the [`tianheng`](https://crates.io/crates/tianheng)
shell (CLI, arg parsing, the composed reaction), which re-exports this crate's surface.

## License

Licensed under either of Apache-2.0 or MIT, at your option.
