## 1. DSL (hunyi)

- [ ] 1.1 Add `forbidden_operands: Vec<String>` to `ImplTraitBoundary` (empty ⇒ shape-only / any returned impl Trait; non-empty ⇒ operand-scoped). `must_not_expose_impl_trait()` constructs it empty; add a getter mirroring `DynTraitBoundary::forbidden_operands`.
- [ ] 1.2 Add `ImplTraitModuleDraft::must_not_expose_impl_trait_of<I, S>(self, operands)` returning the existing `ImplTraitBoundaryDraft` (so `.warn()` / `.because()` compose unchanged), carrying the operand set. Doc states: empty ⇒ any (loud, safe-by-direction); module-prefix operands honored; auto-trait markers never operands; bare/std principals unresolved (stated bound); return-position scoping inherited.

## 2. Observation (hunyi)

- [ ] 2.1 Factor a shared `principal_trait_path(bounds: &Punctuated<TypeParamBound, _>) -> Option<syn::Path>` (the first `TypeParamBound::Trait`), used by both `trait_object_principal_path` and the impl-trait collector — grammar guarantees the base trait is first.
- [ ] 2.2 Extend `ImplTraitCollector` to record, per returned `impl Trait`, both the rendered `impl …` shape and its principal trait path (mirror the `DynExposure`/`DynCollector` shape). The shape-only `impl_trait_module_findings` maps to shapes (unchanged output).
- [ ] 2.3 Add `impl_trait_operand_module_findings(src_dir, root_file, module, forbidden, crate_package)`: resolve the module's `use` map + re-export closure, keep a returned-impl-Trait finding iff `forbidden.is_empty() || matches_forbidden(canonicalize(resolve_path(principal)), &forbidden)`. Same pipeline as `dyn_operand_module_findings`. The return-position walk is unchanged (still `sig.output` only).

## 3. Reaction (hunyi)

- [ ] 3.1 In `check_impl_trait_boundary`, route an empty operand set to the shape-only `impl_trait_module_findings` (resolution-free); a non-empty set to `impl_trait_operand_module_findings`. Same `Violation` shape (finding = rendered `impl …`, rule = "must not expose impl trait"), severity, and constitution-error behavior.

## 4. Shell projection (tianheng)

- [ ] 4.1 Add the operand set to `impl_trait_boundary_json` as a `forbidden` param **when non-empty**; confirm it flows through the generic markdown projection. Add a projection test asserting the operand set appears and that a shape-only boundary projects unchanged (no `forbidden` param).

## 5. Tests (hunyi)

- [ ] 5.1 Operand matching (pure, against synthesized module sources): a returned `impl` of a listed trait flags; a returned `impl` of an unlisted trait passes; a **module-prefix** operand flags; a **re-exported/aliased** operand matches its defining path; **auto-trait markers not operands**; a returned `impl` **nested** (`-> Option<impl crate::Port>`) still matched by principal.
- [ ] 5.2 **Empty operand degeneracy**: `must_not_expose_impl_trait_of([])` reacts to any returned `impl Trait` (identical to shape-only), never a silent no-op.
- [ ] 5.3 Shape-only unchanged: `must_not_expose_impl_trait()` still reacts to any returned `impl Trait` (regression guard). Return-position scoping still holds (APIT / `async fn` not flagged) under both variants.
- [ ] 5.4 Severity + baseline parity; unresolvable crate/module is a constitution error (exit 2); builder carries operands + severity.

## 6. Self-governance & quality gates

- [ ] 6.1 `hunyi` keeps its `syn` quarantine (no new dependency); self-governance (`tianheng_governs_itself`, `self_law_projection_is_fresh`) green.
- [ ] 6.2 `cargo test --all-features --workspace`, `cargo clippy --all-features --all-targets`, `cargo doc --all-features`, `cargo fmt --all --check`, default-feature `build`+`clippy` — all clean.

## 7. Docs

- [ ] 7.1 `hunyi` README: add the operand-scoped impl-trait beside the shape-only one, with an example (`must_not_expose_impl_trait_of([...])`) and the stated bounds (empty ⇒ any; auto-trait markers not operands; bare/std principal resolver bound; return-position inherited).
- [ ] 7.2 Root README + `crates/tianheng/README.md` 渾儀 rows: note `impl Trait` exposure is now shape-only **and** named-operand (v0.1.2).
- [ ] 7.3 `BACKLOG.md`: mark operand-scoped impl-trait **BUILT (v0.1.2)** under 渾儀's built depths (remove it from forward depths). `PROJECT.md`: extend the impl-trait decision noting the named-operand depth (the same stair as operand-scoped dyn).
