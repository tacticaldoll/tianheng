# tasks: Hunyi Observes an Impl Block Nested in a const/fn Body Implementation Plan

Carried from the investigation (see `design.md` for the measured reproduction table, so none of this
needs re-deriving):

- Extraction: `Item::Const` whose initializer is a bare `Expr::Block` (not paren/group-wrapped), or
  `Item::Fn`, contribute their block's direct `Stmt::Item(Item::Impl(_))` statements — one level
  deep, nothing further. No brace/token scanning needed: `syn`'s own AST already gives item context,
  the same reason `cfg_if!` transparency needed no `MacroScope`-style machine.
- Splice points: `scan.rs::flatten_for_walk` (crate-wide walk) and
  `module_resolve.rs::resolve_module_items_with_files` /
  `resolve_module_items_with_cfg_tags` (per-module resolver). Composes for free with `cfg_if!`
  transparency (runs on the already-arm-flattened list) — confirm this rather than assume it.
- Re-verify the consumer set before wiring: grep every reader of `scan.rs`'s `ImplSite`
  (`trait_impl.rs`, `forbidden_marker.rs`) and every reader of `module_resolve.rs`'s per-module
  resolvers (`exposure.rs`, `async_exposure.rs`, `dyn_trait.rs`/`shape_scan.rs`, `impl_trait.rs`,
  `visibility.rs`) — confirm `visibility.rs`'s own item matcher has no `Item::Impl` arm before
  concluding it is unaffected, rather than assuming it from the mechanism alone.

- [x] Add `body_nested_impls` beside `flatten_transparent_macros`/`transparent_macro_arms` in `crates/hunyi/src/syn_util.rs`, with the three scope bounds documented as load-bearing (impl-only, one level, const/fn-only) and widen `FlatItem::plain` to `pub(crate)` for the one new external caller. <!-- id: 0 -->
- [x] Wire the primitive into `scan.rs::flatten_for_walk`, verifying `scan.impls` now carries a const/fn-body-nested TRAIT impl (feeds `trait_impl.rs` and, independently, `forbidden_marker.rs`'s hand-impl form — verify both, not just the one the finding named). <!-- id: 1 -->
- [x] Wire the primitive into both `module_resolve.rs::resolve_module_items_with_files` and `resolve_module_items_with_cfg_tags`, verifying each of signature-coupling, async-exposure, dyn-trait, and impl-trait independently reacts on a const/fn-body-nested INHERENT impl's method — run each capability's own entrypoint directly, not inferred from shared mechanism. <!-- id: 2 -->
- [x] Add regression coverage in `crates/hunyi/src/tests.rs`: both wrapping forms (`const _: () = { … };` and fn-body) crossed with inherent-impl-exposing capabilities (signature-coupling, async-exposure, dyn-trait, impl-trait) and trait-impl-exposing capabilities (trait-impl-locality, forbidden-marker's hand-impl form) — 12 reaction tests. <!-- id: 3 -->
- [x] Add the control test without which an `is_empty`/positive-count assertion could pass vacuously: the identical unwrapped impl still reacts (signature-coupling). <!-- id: 4 -->
- [x] Add the three scope-bound pinning tests: a plain body-nested `fn` (no enclosing `impl`) stays unobserved, an `impl` nested one level further stays unobserved, a `static`-wrapped `impl` stays unobserved. <!-- id: 5 -->
- [x] Confirm the full existing `cargo test -p hunyi --lib` suite still passes unchanged, in particular `async_subtree_does_not_observe_a_body_nested_module` / `impl_trait_subtree_does_not_observe_a_body_nested_module` (the existing body-nested-`mod` bound must not regress). <!-- id: 6 -->
- [ ] Write the spec deltas: a full requirement in `semantic-trait-impl-locality` and `semantic-forbidden-marker` (distinct `scan.rs` mechanism), a canonical requirement in `semantic-signature-coupling` (per-module-resolver mechanism, matching this spec's existing role stating a property shared across the single-module-anchored capabilities), and a cross-referencing requirement plus its own scenario in each of `semantic-async-exposure-boundary`, `semantic-dyn-trait-boundary`, `semantic-impl-trait-boundary`. State all three scope bounds explicitly in each delta that carries the property. <!-- id: 7 -->
- [ ] Add the adopter-facing `CHANGELOG.md` `[Unreleased]` → `### Fixed` entry, naming the measured false negative, the six affected capabilities, and that new violations are baseline-absorbable. <!-- id: 8 -->
- [ ] Run the full Definition of Done from the workspace root and report actual output. <!-- id: 9 -->
