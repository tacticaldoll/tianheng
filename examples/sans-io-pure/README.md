# sans-io-pure — a clock-free, synchronous kernel in one declaration

A standalone adoption of the 天衡 (tianheng) shell's **`sans_io_pure`** composed profile: it folds
the two source-observable axes of a sans-I/O kernel into a single declaration —

```rust
Constitution::new("sans_io_kernel").sans_io_pure(
    SansIoPure::in_crate("sans_io_kernel")
        .module("crate::kernel")
        .reading_clock_via("std::time", ["now"])
        .because("the kernel stays sans-I/O: time is injected, and async lives at the edges"),
)
```

It depends on `tianheng` (not one dimension) because `sans_io_pure` is a **shell** profile: it
composes a 圭表 clock boundary and a 渾儀 async-exposure boundary — and a dimension never composes
its sibling; only the shell does (三儀 ⊥ 三儀).

## The two faults

- `src/kernel.rs` — `stamp()` reads `std::time::SystemTime::now()` inline (the **clock** axis, 圭表).
- `src/kernel/inner.rs` — a `pub async fn` in a **submodule** (the **async** axis, 渾儀).

The async fault sits one module *below* the anchor `crate::kernel`, so it reacts only because
`sans_io_pure`'s async half is **subtree-scoped** (`including_submodules`). `tests/reaction.rs`
includes the discriminator: a seam-only async boundary would miss it.

## Run it

```
cargo test              # asserts both axes react (see tests/reaction.rs)
cargo run --bin check -- check --manifest-path .   # folds both into one exit code (1)
```

(CI runs this in isolation via `cargo test -p tianheng --test examples_suite`, patching the manifest requirement to local source.)
