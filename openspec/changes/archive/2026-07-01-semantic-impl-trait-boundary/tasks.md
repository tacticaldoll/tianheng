## 1. DSL (hunyi)

- [x] 1.1 Add `ImplTraitBoundary { crate_package, module, reason, severity }` with `in_crate(pkg) -> ImplTraitCrateDraft`, `.module(m) -> ImplTraitModuleDraft`, `.must_not_expose_impl_trait() -> ImplTraitBoundaryDraft`, `.warn()`, `.because(reason)`, and accessors — parallel to `DynTraitBoundary`.
- [x] 1.2 Add a `SemanticBoundaries.impl_trait: Vec<ImplTraitBoundary>` slot; update `SemanticBoundaries::is_empty()` to include it.

## 2. Observation (hunyi)

- [x] 2.1 Add an `impl Trait` shape renderer reusing the `dyn` bound machinery: an `syn::TypeImplTrait` has `bounds: Punctuated<TypeParamBound>`, so render `impl {bounds}` via the existing `bound_to_string` (factor a shared helper if cleaner). Stable/injective, same as `trait_object_to_string`.
- [x] 2.2 Add a **return-position** collector: for each public item, collect `impl Trait` nodes appearing at any depth in a function/method **return type** only — `Item::Fn` → `sig.output`; inherent `Item::Impl` (`trait_.is_none()`) public methods → `sig.output`; public `Item::Trait` methods → `sig.output`. Do NOT visit `sig.inputs` (APIT is not governed), generics, fields, consts, statics, or type aliases (no stable written `impl Trait` position there). Exclude trait-`impl` methods (return dictated by the trait).
- [x] 2.3 `impl_trait_module_findings(src_dir, root_file, module, crate_package) -> Result<Vec<String>>`: resolve the module's items, run the return-position collector, return sorted+deduped rendered `impl …` shapes. Shape-only — no `use`-map / re-export closure needed.

## 3. Reaction (hunyi)

- [x] 3.1 Add `check_impl_trait_boundary` (mirror `check_dyn_trait_boundary`): resolve crate/module (unresolvable → constitution error, exit 2), push a `Violation { BoundaryKind::Semantic, target = module, rule = "must not expose impl trait", finding = shape, reason, severity }` per finding. Add the `impl_trait` loop to `check_all`; add a standalone `check_impl_trait` entry mirroring `check_dyn_trait`.

## 4. Shell projection (tianheng)

- [x] 4.1 Add `Constitution::impl_trait_boundary(ImplTraitBoundary)` + slot push; re-export `ImplTraitBoundary` (+ drafts) at the crate root and in the prelude, parallel to `DynTraitBoundary`.
- [x] 4.2 Add `impl_trait_boundary_json` and a `list` document key + markdown section ("Impl-trait boundaries"), parallel to dyn-trait. Add a projection test asserting the section and rule label appear.

## 5. Tests (hunyi)

- [x] 5.1 Return-position matching (pure, against synthesized module sources as the dyn tests do): a `pub fn -> impl crate::Port` flags `impl crate::Port`; a **nested** `-> Option<impl crate::Port>` flags; a public **trait method** `fn m() -> impl crate::Port;` flags; an **argument** `impl Trait` does **not** flag; an **`async fn`** does **not** flag (stated bound); a private fn's RPIT does not flag; a trait-**impl** method's return does not double-flag.
- [x] 5.2 Rendering: `impl Iterator<Item = u8>` / `impl Fn(i32) -> i32` render distinctly (reuse the dyn renderer's injectivity).
- [x] 5.3 Severity + baseline parity; unresolvable crate/module is a constitution error (exit 2). Builder carries anchor + severity.

## 6. Self-governance & quality gates

- [x] 6.1 `hunyi` keeps its `syn` quarantine (no new dependency); self-governance (`tianheng_governs_itself`, `self_law_projection_is_fresh`) green; if the self-law projection enumerates dimensions/kinds, refresh with `BLESS=1` only if the enumeration legitimately grew.
- [x] 6.2 `cargo test --all-features --workspace`, `cargo clippy --all-features --all-targets`, `cargo doc --all-features`, `cargo fmt --all --check`, default-feature `build`+`clippy` — all clean.

## 7. Docs

- [x] 7.1 `hunyi` README: add the impl-trait boundary beside dyn-trait, with an example and the stated bounds (return-position only; APIT and `async fn` out of scope; shape-only).
- [x] 7.2 Root README + `crates/tianheng/README.md` 渾儀 rows: add `impl Trait` (existential) exposure to what 渾儀 observes (v0.1.2).
- [x] 7.3 `BACKLOG.md`: mark impl-trait **BUILT (v0.1.2)** as a 渾儀 depth (the existential complement of dyn-trait); record **async-exposure** as a named, deferred forward sibling. `PROJECT.md`: terse decision recording impl-trait as the existential shape sibling of dyn-trait, governing return-position `impl Trait`, with APIT / `async fn` as stated out-of-scope bounds.
