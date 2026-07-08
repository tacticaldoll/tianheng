# 漏刻 / louke

**刻漏無聲,越界即覺。** — *The clepsydra is silent; a crossing is sensed at once.*

**The runtime observation dimension of [Tianheng](https://github.com/tacticaldoll/tianheng) — the clepsydra.**

漏刻 (the clepsydra, the instrument of flow) observes what static and semantic analysis
**structurally cannot**: the *concrete* type behind a `dyn Trait` as it crosses an
architectural seam at runtime. You declare which concrete-type **origins** may cross a named
seam; a probe reads the live object's observed origin and reacts **fail-closed** (an unknown
origin reacts, never silently passes).

It **ships into your production binary**, so the hot path is `std`-only and near-zero
overhead: a write-once registry read with no lock and a non-SipHash `TypeId` map. `serde_json`
(via [`xuanji`](https://crates.io/crates/xuanji)) is used only on the cold path — emitting an
event — never the hot path.

**Two faces, one declared source.** The same declared `RuntimeBoundary` objects project two
ways:

- **Prod face** — at each seam, `assert_boundary!` reads the crossing object's origin and
  reacts: a structured `Violation` event by default; `panic` is opt-in (a governance tool
  must not crash production on a false positive).
- **CI face** (behind the non-default `audit` feature — the shell enables it; a prod dependency
  on louke compiles none of it) — `audit_probe_coverage` verifies at build/CI time that every
  declared seam has a probe and every probe references a declared seam (closing the "declared
  but never enforced" gap). This face is composed into `tianheng check`.

```rust
// 1. declare the seam (part of your constitution)
let boundary = louke::RuntimeBoundary::at("domain-entry")
    .only_origins(["my_app::domain"])
    .because("only the domain layer may cross into the kernel");

// 2. at startup, install boundaries + register concrete-type origins
louke::install(
    [boundary],
    [louke::register_origin!(MyDomainType) /* … */],
);

// 3. the governed trait carries the Tracked supertrait:
//    trait DomainPort: louke::Tracked {}

// 4. at each seam, probe a crossing object:
//    louke::assert_boundary!("domain-entry", obj); // obj: &dyn DomainPort
```

Origin is **observed** (`register_origin!` captures `module_path!()`), not a self-asserted
label. Explicitly **rejected** as a non-goal: runtime capability/effect drift ("no I/O
reachable") — a runtime policy engine. The registry holds static label allowlists only, never
predicates.

## Adoption & status

**Experimental — pre-1.0.** Public faces may change until adoption settles them; within `0.1.x` no
release intentionally breaks the adopter-written builder.

漏刻 is usually reached **top-down**, through the composed
[`tianheng`](https://crates.io/crates/tianheng) constitution — runtime origin governance is a
*depth* you add once the static and semantic instruments are in place, not a standalone on-ramp.
The prod reaction is an **event by default** (`panic` is opt-in), so it never crashes production on
a false positive. See the runtime mode of the `composed` example under the workspace `examples/`.

## License

Licensed under either of Apache-2.0 or MIT, at your option.
