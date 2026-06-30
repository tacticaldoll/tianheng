## 1. Declaration DSL (hunyi)

- [x] 1.1 Add `DynTraitBoundary` with a fluent builder mirroring `SemanticBoundary`: `DynTraitBoundary::in_crate(pkg).module(path).must_not_expose_dyn().because(reason)`, plus `.warn()` for severity; accessors for `crate_package`, `module`, `reason`, `severity`. No trait operand (shape-only).
- [x] 1.2 Extend `hunyi::SemanticBoundaries` with a `dyn_trait` field (one `Vec<DynTraitBoundary>`), keeping the container the single unit the shell composes; update `is_empty`. `check_all` gains a loop, no arity change.

## 2. Reaction (hunyi)

- [x] 2.1 **Implemented as a parallel `collect_item_dyn_exposures` walk, NOT a refactor of `collect_item_exposures`** — discovered during apply: sharing the walk would change signature-coupling's behavior (it pushes *bare* supertrait/assoc-bound paths a shared visitor would descend into), and the dyn walk needs a position signature-coupling does not cover (associated-type *defaults*). signature-coupling's engine is left untouched; its full existing suite stayed green (59 tests) as the regression gate. (design.md Decision 1 updated to match.)
- [x] 2.2 New trait-object leaf `DynCollector` (resolve.rs) overrides `visit_type_trait_object`, recording a `dyn` node at any depth; driven over the governed positions by `collect_item_dyn_exposures`. `impl Trait` reacts iff it nests a `dyn` node. Resolver unchanged (no `type`-alias expansion).
- [x] 2.3 React at the public type-alias item (`Item::Type` target) when its target writes `dyn`; a public item that merely names the alias gets no extra reaction (Decision 3) — covered by a unit test.
- [x] 2.4 Render the finding via a new `resolve::trait_object_to_string` (`dyn crate::Port`, `dyn Port + Send`). Never `quote`/`syn` `printing`. Finding identity is the rendered shape; rule string `must not expose dyn`. **Renderer made injective (two adversarial-review rounds):** `path_to_string` renders the `Fn(…) -> …` family and every angle-bracketed argument kind that carries observable payload — associated-type/const bindings (`Item = T`), lifetimes, simple const generics, constraints; `type_to_string` renders nested trait-objects, `*ptr`/`!`, macro *names* (`bar!`), and fn-pointers. So `dyn Fn` vs `dyn FnMut`, `dyn Iterator<Item=u8>` vs `<Item=u16>`, and macro/fn-pointer args all stay distinct findings — no collision under the `(target, rule, finding)` baseline identity. The irreducible residual (complex const *expressions*, same-named macros with different args, `verbatim`) is a **stated rendering bound** in the spec + `trait_object_to_string` doc (the same bound trait-impl-locality's `(impl for <self_ty>)` carries), never a silent pass. No existing trait-impl test changes (none exercise these self-types); signature-coupling never uses these renderers.
- [x] 2.5 Anchor resolution → constitution error (exit 2) via the shared `find_package`/`resolve_module_items`; findings fold into the shared report/outcome with severity + `Baseline` parity under identity `(target, rule, finding)` (same `Violation::new` path as every capability).
- [x] 2.6 Add `check_dyn_trait` parallel to the other per-capability `check_*`; include the `dyn_trait` loop in `hunyi::check_all`.

## 3. Shell (tianheng)

- [x] 3.1 Add `Constitution::dyn_trait_boundary(boundary)` (pushes onto `SemanticBoundaries.dyn_trait`); re-export `DynTraitBoundary` (+ drafts) from the crate root and `prelude`.
- [x] 3.2 Confirmed `run(constitution: &Constitution, args)` and `check_all(constitution.semantic_boundaries(), …)` need no signature change — the new field is composed inside `check_all`.
- [x] 3.3 Project the boundary in all three `list` formats — `dyn_trait_text` (text), `dyn_trait_boundary_json` (json, no `forbidden` set — shape-only), and the `Dyn-trait boundaries` markdown section, reason foregrounded by the generic `boundary_markdown`.

## 4. Tests

- [x] 4.1 **Covered by `hunyi` pure-heart unit tests (temp-file workspaces), the house pattern for semantic capabilities — the `fixtures/{clean,violating}` workspaces are the static dimension's, not used by 渾儀.** Scenarios exercised: dyn in return/param/`pub` field; `pub` const/static; trait method return; trait associated-type default; public-item `where`-clause; nested dyn (`Vec<Box<dyn>>`, `Option<&dyn>`); `impl Iterator<Item = Box<dyn>>` reacts; `impl P` clean; public type-alias target reacts; named-alias non-expansion; private alias stated bound; internal-only dyn clean.
- [x] 4.2 Per-scenario unit tests (`dyn_module_findings`) + the unresolvable-anchor constitution error (`dyn_unknown_module_is_a_constitution_error`). Exit-code 0/1/2 aggregation and the constitution-error-supersedes path are the shared `check_all`/`merge_outcomes` already under test for every capability; the dyn boundary flows through them unchanged.
- [x] 4.3 `warn`-severity carried by the builder is unit-tested; baseline + severity reaction is the shared `Violation`/`apply_baseline` path (identity `(target, rule, finding)`), identical to signature-coupling — no dyn-specific divergence to retest.
- [x] 4.4 Macro-generated dyn (stated bound) is the universal 渾儀 macro-expansion bound — documented in the spec, not separately tested (no observation to assert).

## 5. Self-governance & quality gates

- [x] 5.1 Self-governance green (`tianheng_governs_itself`): `hunyi` allowlist unchanged `{serde_json, syn, xuanji}` (only `syn` + std used), `guibiao` gains no `syn`, `hunyi` does not depend on `tianheng`.
- [x] 5.2 `cargo test --all-features --workspace`, `cargo clippy --all-features --all-targets`, `cargo doc --all-features`, and default-feature `build`+`clippy` — all clean.
- [x] 5.3 `tianheng check`'s default constitution is the **demo** (`example-core`, absent here) — expected; the binding self-reaction is the `self_governance` `cargo test` gate, which is green.

## 6. Docs & governance projections

- [x] 6.1 `BACKLOG.md`: dyn-trait-boundary marked BUILT (v0.1.2) as the first **depth** addition under 渾儀; `must_not_expose_dyn_of` listed as the next named depth.
- [ ] 6.2 **DEFERRED to the separate doc-fix (option A):** the `PROJECT.md` capability-admission decision entry is held back to keep the governance contract file out of this change's diff — same pass as the stale `run(&SemanticBoundaries)` signature correction.
- [x] 6.3 Self-law projection unaffected — `self_law_projection_is_fresh` passes (no dependency boundary moved), so no `BLESS=1` refresh needed.
- [x] 6.4 `hunyi` README lists the dyn-trait capability beside signature-coupling, with an example; the `DynTraitBoundary` type carries its own docs (`#![deny(missing_docs)]`).
