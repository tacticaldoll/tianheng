# 天衡 / Tianheng

**懸衡以待,失衡即應。** — *Hold the balance ready; react the moment it tips.*

> 天衡 (the celestial balance) weighs the shape you *declared* against the shape the
> code *is*; the moment they no longer balance, it reacts. **Govern by reaction, not
> instruction.**

Tianheng is a Rust-native **reactive architectural-governance** framework — the successor
to [`modou`](https://github.com/tacticaldoll/modou). It does not run your app and it does
not instruct your agent. Developers and agents propose change; Tianheng uses compiler/CI
and runtime *reactions* to keep architectural shape from drifting.

## Why reaction, not instruction

Architectural intent — "the core must not depend on adapters" — used to live in human
review. An AI agent writes fluent, locally-plausible code without holding that intent, so
it erodes the shape it does not understand, and *instructing* it cannot bind an agent that
has no understanding. Tianheng crystallizes the human's intent into a **non-bypassable
reaction**: neither the agent nor Tianheng needs to understand for the law to hold.

## A declared boundary (v0.1.0)

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

> The published `tianheng` binary is a *demo* bound to a sample constitution (it governs a
> crate named `example-core`). Tianheng is consumed as a **library**: declare your own
> constitution and expose your own binary, as above.

## The instruments (三儀) — observation dimensions

Tianheng is a **crate family**. You select governance by depending on the dimensions you
want; each is real drift (declared vs. observed), never a style lint. The three are
measuring instruments — each reads a different surface of the code.

| 儀 Instrument | Crate | Observes | Observation source | Status |
|---|---|---|---|---|
| 圭表 gnomon (static) | `guibiao` | the cast shadow: imports & dependencies | `cargo metadata` + source `use` scan | **v0.1.0** (static core, from modou) |
| 渾儀 armillary (semantic) | `hunyi` | type exposure, impl locality, visibility & forbidden markers | AST (`syn`) | **v0.1.0** (signature-coupling, trait-impl-locality, visibility, forbidden-marker) |
| 漏刻 clepsydra (runtime) | `louke` | flow: the concrete type behind a `dyn Trait` crossing a seam | runtime `TypeId` / observed origin | **v0.1.0** (origin-assertion; CI probe-coverage face composed into `tianheng check`) |

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

Beneath the dimensions sits **`xuanji` (璇璣) — the 底**: the dimension-agnostic
**reaction model** (`Severity`, `Violation`, `Report`, `Baseline`, `Outcome`) every
dimension reacts in. It is `serde_json`-only, carries no observation engine, and depends
on no workspace member — so a new dimension reuses the reaction vocabulary without dragging
in another dimension's engine.

A dimension's crate is **born when it is built** — never pre-created empty. The heavy
dependencies (AST, runtime) are quarantined to their own crates; the `guibiao` core's only
*external* dependency stays `serde_json` (it depends internally on `xuanji`). See
[`BACKLOG.md`](BACKLOG.md) for the deferred phases (their observation sources and open
design questions) and the governance/observability layer.

Tianheng governs **itself** with its own reaction: the core (`guibiao`) must not depend on
the shell (`tianheng`), `syn` is quarantined to `hunyi`, `xuanji` stays beneath every
dimension, and the core stays dependency-light — enforced as a `cargo test` gate
(`crates/tianheng/tests/self_governance.rs`).

## Non-goals

Not active code-shaping/generation, not a prescriptive framework you build inside, not a
schema crate, not a lint, not a universal graph API. No TOML/Markdown for the constitution.
Each dimension keeps its own observation source; nothing is named before its reaction
exists.

## License

Licensed under either of Apache-2.0 or MIT, at your option.
