# proposal: Hunyi Observes an Impl Block Nested in a const/fn Body

## Why

Two "not yet verified" findings from `docs/audit/0.3.1-adversarial-sweep.md` share one root cause.
`exposure.rs:173` reported that an inherent impl wrapped in `const _: () = { impl Svc { … } };` (the
"const-eval trick" idiom, commonly used for a compile-time trait assertion or a doctest/dogfooding
scratch impl) hides its methods from every capability that reads a module's public API — async
exposure, signature coupling, `dyn`, and `impl Trait`. `trait_impl.rs:91` reported the identical
wrapping hiding a **trait** impl from impl-locality. Both were investigated together and confirmed:
渾儀 reads a module's items from `&[syn::Item]` (via `scan.rs`'s crate-wide walk and
`module_resolve.rs`'s per-module resolver), and neither ever looks inside a `const`'s initializer or
a `fn`'s body for a nested `Stmt::Item` — the same posture that correctly treats a body-nested `mod`
as unreachable, applied somewhere it does not hold.

That posture is unsound for exactly one item kind. A `fn`/`struct`/`mod` written directly in a body
genuinely is scoped to that body and unreachable as `crate::…` — the dimension already states this
bound for a body-nested `mod` (`semantic-async-exposure-boundary`'s "A body-nested module is a stated
bound" scenario), and it is correct. An `impl` is different: Rust binds it to its self type's own
coherence set regardless of where it is lexically written, so `impl Svc { pub fn leak(&self) -> … }`
inside ANY body still makes `Svc::leak` real, externally callable public API the instant `Svc` itself
is module-level. Reproduced directly (`cargo test -p hunyi`, five capabilities, both the `const _`
and fn-body forms, plus an unwrapped control proving each fixture is otherwise sound): the identical
method that reacts at module top level produces zero findings the moment its enclosing `impl` moves
into a `const`/`fn` body, on ordinary, compilable source. That is the one false negative the core
contract forbids, on both the derived-permission axis (signature/async/dyn/impl-trait exposure) and
the impl-locality axis.

Grepped every consumer of `scan.rs`'s crate-wide impl collection and every consumer of
`module_resolve.rs`'s per-module item resolver (not assumed complete from the earlier read): a sixth
capability shares the trait-impl-locality mechanism and closes for free — `forbidden_marker.rs`'s
hand-`impl T for X` acquisition form also reads `scan.impls`, so it has the identical gap and is
covered by the identical fix. `visibility.rs` also consumes the per-module resolver, but its own item
matcher (`syn_util::item_observation_parts`) never matches `Item::Impl` at all, so it is structurally
unaffected — confirmed, not assumed.

Grepped `PROJECT.md`'s Decisions and every `openspec/specs/semantic-*` for a stated bound covering
this shape: none exists. `semantic-trait-impl-locality`'s own requirement text ("The system SHALL
observe **every** `impl <Trait> for <Type>` block...") was flat-out false for this shape. Neither
finding is refuted.

## What Changes

- One new shared primitive, `syn_util::body_nested_impls`, sibling to
  `flatten_transparent_macros`/`transparent_macro_arms` and held to the identical soundness
  discipline that macro-name-gates `cfg_if!` transparency rather than generalizing it: it recovers
  **only** an `impl` block that is a direct statement of the outermost body of a `const` initializer
  (written as a bare `{ … }` block expression) or a `fn`'s own body — one level deep, `const`/`fn`
  only. Both audited trigger shapes are exactly this depth; nothing further is claimed.
- Applied at the two places items already enter observation: `scan.rs::flatten_for_walk` (feeding the
  crate-wide `ImplSite` collection that backs trait-impl-locality AND forbidden-marker's hand-impl
  form) and `module_resolve.rs`'s `resolve_module_items_with_files` /
  `resolve_module_items_with_cfg_tags` (feeding the four `collect.rs` collectors behind
  signature-coupling, async-exposure, `dyn`-trait, and `impl Trait`). No downstream matcher changes:
  each already handles a top-level `Item::Impl` correctly, and now also sees the extracted one.
- Three residual bounds stated in the spec deltas rather than left silent, mirroring the `cfg_if!`
  precedent's own discipline: (1) only `impl` is extracted — a plain item nested the same way stays
  exactly as unobserved as it already was; (2) only one level deep — an `impl` nested further inside
  the body (inside an `if`/`loop`/closure/nested `fn`) is not recovered; (3) `const`/`fn` only, not
  `static` — the const-eval trick is specifically about `const`, and no audited idiom uses `static`
  for it.

## Capabilities

### Modified Capabilities

- `semantic-trait-impl-locality`: gains the crate-wide `impl` observation property for a
  `const`/fn-body-nested trait impl (distinct mechanism from the four below — `scan.rs`'s own
  crate-wide walk, not the per-module resolver).
- `semantic-forbidden-marker`: gains the identical property for its hand-`impl T for X` acquisition
  form, sharing `scan.rs`'s `ImplSite` collection with trait-impl-locality — an affected capability
  the original two findings did not name, found by re-verifying every `scan.impls` consumer.
- `semantic-signature-coupling`: carries the canonical requirement text for the per-module-resolver
  mechanism (matching this spec's existing role of stating a property "shared by every
  single-module-anchored semantic capability" on their behalf), plus its own inherent-impl scenario.
- `semantic-async-exposure-boundary`, `semantic-dyn-trait-boundary`, `semantic-impl-trait-boundary`:
  each cross-references the shared property and states its own capability-specific scenario (an
  `async fn`, a `Box<dyn Trait>` return, an `impl Trait` return, respectively, nested the same way).

## Impact

- `crates/hunyi/src/syn_util.rs`: the new `body_nested_impls` primitive; `FlatItem::plain` widened
  from private to `pub(crate)` so `module_resolve.rs` can wrap a synthetic extracted impl without a
  fabricated arm-membership claim.
- `crates/hunyi/src/scan.rs`, `crates/hunyi/src/module_resolve.rs`: the two call sites wired to the
  new primitive.
- `crates/hunyi/src/tests.rs`: regression coverage for both wrapping forms across all six affected
  capabilities, an unwrapped control, and three tests pinning the stated scope bounds (a plain
  body-nested fn stays unobserved, an impl nested one level further stays unobserved, a
  `static`-wrapped impl stays unobserved) so the scope cannot silently widen later.
- `CHANGELOG.md`: `[Unreleased]` → `### Fixed`.
- Non-breaking: no public API, DSL, or wire-format change. An adopter using the const-eval-trick
  idiom (or its fn-body sibling) for a real impl may see new violations — a false-negative closure,
  the same class 0.2.2/0.2.3/`hunyi-cfg-if-transparency` shipped as patches, absorbable by baseline.
