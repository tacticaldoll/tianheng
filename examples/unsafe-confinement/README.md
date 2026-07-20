# unsafe-confinement — confine `unsafe` to one auditable subtree

A standalone adoption of 渾儀 (hunyi)'s **unsafe-confinement** capability: `unsafe` may appear
**only under** a declared subtree (`crate::ffi`), never elsewhere.

This is the one capability that **cannot** be demonstrated inside the Tianheng family: every family
crate is `unsafe`-free and says so with `#![forbid(unsafe_code)]` (the strongest, compile-time,
unbypassable statement). `UnsafeBoundary` governs *where* `unsafe` lives — the architectural intent
`#![forbid]` cannot express — so it needs a crate that legitimately *contains* confined `unsafe`.

## The law

```rust
UnsafeBoundary::in_crate("unsafe_confinement")
    .only_under(["crate::ffi"])
    .because("unsafe lives only behind the ffi module — everywhere else is safe by contract")
```

## The code

- `src/ffi.rs` — real, **confined** `unsafe` (a raw-pointer read behind a safe wrapper). Allowed.
- `src/net.rs` — a **stray** `unsafe` block outside `crate::ffi`. The deliberate violation → exit 1.

`unsafe` governs *where*, not *whether*: confinement-only. An empty or crate-root `only_under` is a
**constitution error** (exit 2) that points you at `#![forbid(unsafe_code)]` — the compiler's
stronger job — never a silent no-op.

## Run it

```
cargo test        # asserts the reaction (see tests/reaction.rs)
cargo run --bin demo   # renders it, exits 1
```

(CI runs this in isolation via `scripts/test_examples.sh`, patching `hunyi = "0.2"` to local source.)
