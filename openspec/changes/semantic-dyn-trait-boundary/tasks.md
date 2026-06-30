## 1. Declaration DSL (hunyi)

- [ ] 1.1 Add `DynTraitBoundary` with a fluent builder mirroring `SemanticBoundary`: `DynTraitBoundary::in_crate(pkg).module(path).must_not_expose_dyn().because(reason)`, plus `.warn()` for severity; accessors for `crate_package`, `module`, `reason`, `severity`. No trait operand (shape-only).
- [ ] 1.2 Extend `hunyi::SemanticBoundaries` with a `dyn_trait` field (one `Vec<DynTraitBoundary>`), keeping the container the single unit the shell composes; do not change `run`/`check_all`'s arity beyond adding the field.

## 2. Reaction (hunyi)

- [ ] 2.1 Refactor signature-coupling's surface walk (`collect_item_exposures`) to feed two leaf observations without changing its path-based findings — `PathCollector` keeps yielding `Vec<syn::Path>`; signature-coupling's **full existing fixture suite is the regression gate** (this edits its engine, not its spec).
- [ ] 2.2 Add the new trait-object leaf: a visitor overriding `visit_type`/`visit_type_trait_object` to record `syn::Type::TraitObject` presence over the same governed positions; react on a `dyn` node at any depth; `impl Trait` reacts iff it nests a `dyn` node (Decision 2). Reuse `hunyi::resolve` unchanged (no `type`-alias expansion).
- [ ] 2.3 React at the public type-alias item (`Item::Type` target) when its target writes `dyn`; emit no extra reaction for a public item that merely names the alias (Decision 3).
- [ ] 2.4 Render the finding via the existing `resolve::type_to_string` ("`<site>` exposes `dyn <trait>`"); settle the alias-item wording (Open Question); never use `quote`/`syn` `printing`. Fall back to a location-only finding identity where `type_to_string` returns `None`.
- [ ] 2.5 Wire anchor resolution → constitution error (exit 2), and fold findings into the shared report/outcome with severity + `Baseline` parity under identity `(target, rule, finding)`.
- [ ] 2.6 Add a `check_dyn_trait` entry parallel to the other per-capability `check_*`, and include it in `hunyi::check_all`.

## 3. Shell (tianheng)

- [ ] 3.1 Add a `Constitution::dyn_trait_boundary(boundary)` adder mirroring `signature_boundary`/`trait_impl_boundary`/etc. (pushes onto `SemanticBoundaries.dyn_trait`), and re-export `DynTraitBoundary` (+ its drafts) from the `prelude`. Without this the capability is unreachable from the builder.
- [ ] 3.2 Confirm `run(constitution: &Constitution, args)` and `hunyi::check_all(constitution.semantic_boundaries(), …)` need no signature change — the new field is composed inside `check_all`.
- [ ] 3.3 Project the new boundary in `list --format markdown`/`json` (constitution projection), leading with its `reason` (潛移 / reason-foregrounding parity).

## 4. Fixtures & tests

- [ ] 4.1 Add fixture(s) under the `violating`/`clean` workspaces exercising: dyn in return/param/`pub` field; **`pub` const/static type; public trait method return; trait associated-type default; public-item `where`-clause**; nested dyn (`Vec<Box<dyn>>`, `Option<&dyn>`); `impl Iterator<Item = Box<dyn>>` reacts; `impl P` clean; public `type` alias target reacts; named-alias non-expansion; private alias stated bound; internal-only dyn clean; macro-generated dyn stated bound.
- [ ] 4.2 Unit/integration tests asserting each spec scenario, including exit codes (0/1/2) and the unresolvable-anchor constitution error superseding a violation.
- [ ] 4.3 Baseline + `warn`-severity tests at parity with signature-coupling.

## 5. Self-governance & quality gates

- [ ] 5.1 Confirm self-governance stays green: `hunyi` allowlist still `{serde_json, syn, xuanji}` (no new dep), `guibiao` gains no `syn`, `hunyi` does not depend on `tianheng`.
- [ ] 5.2 Run `cargo test --all-features`, `cargo clippy --all-features`, `cargo doc`, plus the default-feature `build`+`clippy` (prod-light config) — all clean.
- [ ] 5.3 Run `tianheng check` against the workspace and confirm exit 0 (or expected baselined state).

## 6. Docs & governance projections

- [ ] 6.1 Update `BACKLOG.md`: mark dyn-trait-boundary BUILT under 渾儀; keep `must_not_expose_dyn_of` listed as the next named depth (non-goal here).
- [ ] 6.2 Update `PROJECT.md` Decisions only if the capability-admission narrative needs the new entry (it passed all three gates — record it tersely, do not inflate).
- [ ] 6.3 Refresh the self-law projection if affected: `BLESS=1 cargo test -p tianheng self_law_projection_is_fresh` (only if `self_governance.rs` changed — it should not, since no dependency boundary moves).
- [ ] 6.4 Update `hunyi`'s README/crate docs to list the new capability beside signature-coupling.
