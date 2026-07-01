## 1. DSL (hunyi)

- [ ] 1.1 Add `AsyncExposureBoundary { crate_package, module, reason, severity }` with `in_crate → ImplTrait-style drafts → must_not_expose_async_fn() → warn/because` and accessors — parallel to `ImplTraitBoundary`.
- [ ] 1.2 Add a `SemanticBoundaries.async_exposure: Vec<AsyncExposureBoundary>` slot; update `SemanticBoundaries::is_empty()`.

## 2. Observation (hunyi)

- [ ] 2.1 Add the owner-qualified finding renderer: `async fn <module>::name(params) -> ret` (free fn), `async fn <SelfTy>::name(...)` (inherent method), `async fn trait <module>::<Trait>::name(...)` (trait method). Render params via `type_to_string` (receiver as `self`/`&self`/`&mut self`), return via `type_to_string`; unrenderable → `_`. Owner kind + owner + name guarantee distinct-owner async fns never collide.
- [ ] 2.2 Add `async_exposure_module_findings(src_dir, root_file, module, crate_package)`: resolve the module's items; for each **public** `Item::Fn`, inherent-`Item::Impl` public method, and public `Item::Trait` method declaration with `sig.asyncness.is_some()`, push the owner-qualified finding. Exclude trait-impl methods (`trait_.is_some()`) and private items. Sort + dedup. Shape-only — no resolver.

## 3. Reaction (hunyi)

- [ ] 3.1 Add `check_async_exposure_boundary` (mirror `check_impl_trait_boundary`): resolve crate/module (unresolvable → constitution error, exit 2), push a `Violation { BoundaryKind::Semantic, target = module, rule = "must not expose async fn", finding, reason, severity }` per finding. Add the `async_exposure` loop to `check_all`; add a standalone `check_async_exposure` entry.

## 4. Shell projection (tianheng)

- [ ] 4.1 Add `Constitution::async_exposure_boundary(AsyncExposureBoundary)` + slot; re-export `AsyncExposureBoundary` (+ drafts) at the crate root and prelude, parallel to `ImplTraitBoundary`.
- [ ] 4.2 Add `async_exposure_boundary_json`, a `list` document key + markdown section ("Async-exposure boundaries"), and a text projection. Add a projection test asserting the section and rule label appear.

## 5. Tests (hunyi)

- [ ] 5.1 Detection (pure, synthesized module sources): a public async free fn flags; a public inherent async method flags; a public trait async method declaration flags; a **trait-impl** async method does **not** double-flag; a **private** async fn does not flag; a non-async fn does not flag.
- [ ] 5.2 **Finding injectivity (the crux)**: `impl A { async fn run }` + `impl B { async fn run }` yield TWO distinct findings; `trait T { async fn run }` + `trait U { async fn run }` yield TWO distinct findings; a free `run` and an inherent `<A>::run` do not collide.
- [ ] 5.3 Severity + baseline parity; unresolvable crate/module is a constitution error (exit 2); builder carries anchor + severity.

## 6. Self-governance & quality gates

- [ ] 6.1 `hunyi` keeps its `syn` quarantine (no new dependency); self-governance (`tianheng_governs_itself`, `self_law_projection_is_fresh`) green.
- [ ] 6.2 `cargo test --all-features --workspace`, `cargo clippy --all-features --all-targets`, `cargo doc --all-features`, `cargo fmt --all --check`, default-feature `build`+`clippy` — all clean.

## 7. Docs

- [ ] 7.1 `hunyi` README: add the async-exposure boundary beside impl-trait, with an example and the stated bounds (public-surface, three item kinds; trait-impl/private excluded; complementary to written `-> impl Future`; declarative = implicit-existential-at-a-seam, not "no async").
- [ ] 7.2 Root README + `crates/tianheng/README.md` 渾儀 rows: add `async fn` (implicit existential) exposure (v0.1.2).
- [ ] 7.3 `BACKLOG.md`: mark async-exposure **BUILT (v0.1.2)** (remove from forward depths); note a possible future `must_not_expose_existential` unifier. `PROJECT.md`: terse decision recording async-exposure as the implicit-existential complement of impl-trait, its owner-qualified finding identity (why bare-name/shape would mask), and the weaker-but-holding declarative gate (implicit existential at a declared seam, anchor-scoped).
