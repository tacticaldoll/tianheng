# 天衡 / tianheng

**懸衡以待,失衡即應。** — *Hold the balance ready; react the moment it tips.*

The **shell** of the [Tianheng](https://github.com/tacticaldoll/tianheng) crate family:
reactive architectural governance that weighs the shape you *declared* against the shape the
code *is*, and reacts the moment they no longer balance. **Govern by reaction, not
instruction.**

天衡 (the celestial balance) is the imperative shell + facade — CLI (arg parsing, filesystem,
stdout/stderr) and the `run` reaction that composes every dimension into one. You declare a
single `Constitution` carrying all 三儀 (the three instruments) and call `run`:

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
        // 渾儀 (semantic): folded in via typed adders, e.g.
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
`--format json` projects the reaction as JSON.

## The instruments (三儀)

`tianheng` composes the three observation dimensions, each its own crate, each real drift
(declared vs. observed) — never a style lint:

| 儀 | Crate | Observes |
|---|---|---|
| 圭表 (static) | [`guibiao`](https://crates.io/crates/guibiao) | imports & dependencies (`cargo metadata` + `use` scan) |
| 渾儀 (semantic) | [`hunyi`](https://crates.io/crates/hunyi) | type exposure, impl locality, visibility, forbidden markers (AST/`syn`) |
| 漏刻 (runtime) | [`louke`](https://crates.io/crates/louke) | the concrete type behind a `dyn Trait` crossing a seam (runtime `TypeId`) |

Beneath them sits [`xuanji`](https://crates.io/crates/xuanji) — the dimension-agnostic
reaction model. The 漏刻 prod face is wired in your own binary
(`louke::install(constitution().runtime_boundaries()…)`); its CI probe-coverage face is run
by `tianheng check`.

> The published `tianheng` binary is a **demo** bound to a sample constitution (it governs a
> crate named `example-core`). Tianheng is consumed as a **library**: declare your own
> constitution and expose your own binary, as above.

Tianheng governs **itself** with its own reaction (`crates/tianheng/tests/self_governance.rs`):
the core must not depend on the shell, `syn` is quarantined to `hunyi`, `xuanji` stays beneath
every dimension.

## License

Licensed under either of Apache-2.0 or MIT, at your option.
