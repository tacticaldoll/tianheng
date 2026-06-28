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

**Stated partial coverage** (never silently passed): the hand-rolled `use` scanner does not
see bare path expressions, macro-generated imports, or `#[path]`-remapped modules — closing
those would require an AST, an amendment, not a silent trade.

Most adopters consume the static dimension through the [`tianheng`](https://crates.io/crates/tianheng)
shell (CLI, arg parsing, the composed reaction), which re-exports this crate's surface.

## License

Licensed under either of Apache-2.0 or MIT, at your option.
