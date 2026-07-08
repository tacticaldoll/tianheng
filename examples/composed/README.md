# Example — composed (天衡 as the funnel target)

One small hexagonal app, governed by **all 三儀** through the [`tianheng`](https://crates.io/crates/tianheng)
shell. This is where a single-instrument adopter graduates to the composed constitution — and
where the runtime dimension (漏刻) lives, because it reacts *at runtime in your binary*, not at CI
time.

```toml
[dependencies]
tianheng = "0.1"   # the CI shell — composes 圭表 + 渾儀 + 漏刻 into one `check`
louke    = "0.1"   # the runtime dimension's prod face — you wire it into your binary
```

## The funnel, made literal

`src/governance.rs` grows the constitution by one line per instrument:

```rust
Constitution::new("composed_app")
    .boundary(ModuleBoundary::in_crate("composed_app")           // 圭表 (static)
        .module("crate::domain").must_not_import("crate::infra").because(…))
    .signature_boundary(SemanticBoundary::in_crate("composed_app") // 渾儀 (semantic)
        .module("crate::api").must_not_expose("crate::infra::DbPool").because(…))
    .runtime(RuntimeBoundary::at("adapter-seam")                 // 漏刻 (runtime)
        .only_origins(["composed_app::adapters::blessed"]).because(…))
```

## Two modes, because the 三儀 react in two places

**check-mode** (CI time, against source) — static + semantic faults react with one exit code:

```sh
cargo run --bin check -- check --manifest-path Cargo.toml
cargo run --bin check -- check --manifest-path Cargo.toml --format json    # same exit code, different presentation
```

**run-mode** (runtime, in the binary) — the concrete type behind a `dyn Adapter` crossing the
port seam is checked against the allowlist, fail-closed:

```sh
cargo run --bin runtime_demo    # the blessed adapter crosses cleanly; the rogue one reacts
```

`tests/funnel.rs` asserts the two CI-time instruments (圭表 static, 渾儀 semantic) each react to
their fault; `tests/runtime.rs` asserts the runtime instrument (漏刻) — the rogue origin reacts and
the blessed one does not.

> Experimental / pre-1.0: public faces may change until real adoption pressure settles them.
