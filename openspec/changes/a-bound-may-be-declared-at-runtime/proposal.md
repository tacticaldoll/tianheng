## Why

Found in the pre-release review of the `0.5.0` window, on the axis that asks what a third party can actually do
with the surface this window publishes.

`Observer::bounds` has **no default body** — that is the protocol's whole point, and `observer-protocol`'s spec says
so: a participant that says nothing about its limits cannot be written. So every implementor, family or third
party, must construct `BoundDecl`s. And every string on that path is `&'static str`:

```rust
BoundId::new(id: &'static str)
BoundDecl::new(BoundId, shape: &'static str, Extent, pinned_by: &'static str)
Extent::OutOfReach { because: &'static str }
Owner::Inherited { from: &'static str }
```

**A bound whose id, shape or rationale is not a compile-time literal cannot be expressed at all.** An observer over
a plugin set, a set of scanned roots, or anything else whose members are discovered rather than written, is
mandated to declare its limits and given no way to name them.

The constraint has an honest defence — a bound is a property of the *reaction*, and a reaction knows its own limits
at compile time — and if that defence is the decision, it belongs in the documentation. Measured: it is written
**nowhere**. `grep static` across `crates/xuanji/src/bound.rs` and `observer.rs` finds it only in the signatures
themselves. A third-party implementor meets it as a compiler error on their first `format!`, with no statement of
intent to read.

**Why now, and why this is cheap.** Measured before proposing: `BoundDecl`, `BoundId`, `Extent`, `Reached`, `Owner`,
`FactGranularity`, `Demonstrates` and `Observer` are mentioned **zero** times in `v0.4.0`'s `crates/xuanji/src/lib.rs`.
The entire surface is new in this unreleased window. So this is not a breaking change and owes no migration note —
it is a refinement of an API that has never shipped, and the only moment it is free.

## What Changes

**Every string a bound declaration carries becomes `Cow<'static, str>`**, so a literal stays borrowed and allocates
nothing while a computed value is expressible.

- `BoundId::new`, `BoundDecl::new` and the extent rationales accept `impl Into<Cow<'static, str>>`, so a call site
  passing a literal is unchanged.
- The struct-variant fields (`because`, `from`) become `Cow<'static, str>`; a literal at those sites gains `.into()`,
  because struct-literal syntax performs no conversion. That is the churn this change pays: 55 `BoundDecl::new`
  sites and 61 rationale fields across the family's own declarations.
- The accessors return `&str` borrowed from the declaration rather than `&'static str`. Nothing in the family held
  one beyond its declaration's life, and a borrow is what a `Cow` can honestly promise.
- `const fn` is dropped where `Cow` prevents it. Nothing constructs a `BoundDecl` in a `const` item — every family
  declaration is built inside `fn observation_bounds()` — so no call site loses anything.

**And what the model now claims is stated.** A bound may be declared at runtime; what a bound *is* does not change,
and neither does the reaction that holds every declared bound in a bijection with its spec scenario.

## Also in this change: three CHANGELOG corrections from the same review

Found on the same axis and touching the same entries, so they land together rather than in a second pass over one
paragraph:

- The `Observer` entry says "圭表, 渾儀 and 漏刻 each implement it" and **names none of the three types** an adopter
  would construct: `StaticObserver`, `SemanticObserver`, `RuntimeObserver`. The entry's actionable half was missing.
- `louke::RuntimeObserver` is behind the **`audit` feature**, and the entry does not say so. An adopter depending on
  `louke` directly without that feature finds nothing. Through `tianheng` it is always present, because the shell
  enables it — which is exactly why the omission is easy to miss.
- `tianheng::testing::assert_projection_matches` is a new public helper in the adopter-facing test harness and is
  mentioned nowhere.

## Capabilities

### Modified Capabilities

- `observation-bound-model`: a declaration's strings are owned-or-borrowed, and a bound may be declared at runtime.

## Impact

- **Modified**: `crates/xuanji/src/bound.rs`, and every declaration site in `crates/{guibiao,hunyi,louke,tianheng}`.
- **Modified**: `CHANGELOG.md` — the two entries above, corrected.
- **Not breaking**: the whole surface is unreleased. No adopter has this API, no baseline changes, no reaction moves.
  Version class **MINOR**, unchanged from what the window already is.
