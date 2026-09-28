# Implementation plan

These tasks are for a later implementation after review accepts the proposal. Keep the proposal branch source-free. Every Cargo command runs serially in the foreground with `CARGO_BUILD_JOBS=4`. A negative run mutates **engine** code, never only a fixture, and its record is pasted from the run.

## Ordered steps

1. **Freeze the answers that do not change, and own the md5x fixture.** (1.5 days) Add `per_target_corpus.rs` cases for every measured row whose current release/0.8.0 answer already equals its target — R2 `std::time`, R3 bare, R4, A/A2, B, G, C1–C4, E1–E5, and the accepting K/N heads — so the refactor in steps 3–5 is held to them. A row whose answer changes gets its test in the step that changes it, landing with its reaction; no test is added `#[ignore]`d. Move `md5x_dep` under `RootProbe.dir`.
   - Acceptance: every probe builds under its declared edition (K rows at 2018/2021/2024); by the end of step 5 every row in `measurements.md` names one test; two concurrent runs of the md5x test leave neither owned root behind.
   - Negative run: mutate `resolve_head` to leave every call head unresolved and run `inline_resolves_a_type_alias`; it must lose its finding and fail. Remove the root cleanup from `RootProbe::drop` and run the concurrent md5x pair; the absence assertion must fail.

2. **Prefix heads by the finite rule.** (0.5–1 day) In `path_vocab.rs`, replace `is_disallowed_symbol_head`'s call to `lexer::is_rust_keyword` with the set in design e (`_`, `r#_`, `r#crate`, `r#self`, `r#super`, `r#Self`, bare `self`/`super`/`Self`, `::crate`), keeping bare `crate`. `lexer::is_rust_keyword` then has only its macro-negation caller. Rename `a_prefix_starting_with_a_keyword_is_refused` to `a_prefix_head_naming_no_crate_or_module_is_refused` and extend it to the N rows; add `a_keyword_prefix_head_is_accepted_bare_or_raw` over the K rows. Rewrite the `[Unreleased]` spelling entry and the `path_vocab.rs` docs to the same set. Independent of the scope graph, so it lands first.
   - Acceptance: all K rows 1/1 bare and raw with one `RuleKey` per head; `_`, `_::clock` and every N refusal 2/2; `crate::clock` 1/1; `git grep -n is_rust_keyword crates/guibiao/src` shows only the lexer.
   - Negative run: drop `_` from the set — the refusal test fails on `_::clock`. Put `async` in the set — the acceptance test fails. Stop stripping `r#` before building the key — the one-identity assertion fails.

3. **Typed scope graph with module and block scopes.** (2.5–3 days) Add `scope_graph.rs`: `Scope` (module or block), items per scope and namespace, `NamedBinding` recorded in its innermost enclosing scope, `GlobEdge`, `ExternPrelude` with edition. Extend the brace walk `path_occurrences` already does so every `{` opens a scope; keep the extern-block brace transparent. Populate from the one use-tree walker; retire the duplicate walkers only once every consumer reads the shared records.
   - Acceptance: existing inline alias, type-alias, public re-export and sibling-isolation tests keep their findings; import reports are byte-identical; B3/B4 and F reach their target answers, H1 and H3 stay 1/1, and G stays 1/1, with their tests landing here.
   - Negative run: record every `use` in its module scope instead of its innermost block — B3 under `crate::a` and F fail. Drop the block scope on its closing brace late (at end of file) — B3 under `crate::b` fails. Let an `impl` body open a scope that records its members — H1 fails.

4. **Resolve globs, inherited scope, candidate sets and the 2015 root.** (2–2.5 days) Fixed-point glob resolution with visibility; candidate sets whose occurrence reacts when any candidate resolves under the prefix; in an edition-2015 package, `use` paths and `::`-leading paths resolve from the crate root scope, including its `extern crate` items.
   - Acceptance: R1, `super::super::*`, the existing glob-hazard cases, E1–E6 and the D rows reach their target answers. No resolver path returns exit 2.
   - Negative run: skip one parent glob edge in the worklist — R1 fails. Keep only the last candidate — E6 under `crate::a` fails. Resolve 2015 `::x` from the current module — D `crate::clock` fails.

5. **Path root form, canonical identity and policy.** (1.5 days) Record each occurrence's root form (plain, `::`-rooted, `<`-qualified) from its syntactic position; resolve plain and `::` heads through the graph; leave `<`-qualified paths unresolved under the receiver-method bound; canonicalize before `RuleKey`; apply default versus strict-external.
   - Acceptance: R2's cross-spelling baseline suppresses; R3 and D `md5x`/`::md5x` are 0/1 for both spellings; R4 stays clean; R6 and C1–C4 are 0/0 in both modes.
   - Negative run: keep the leading colon in `RuleKey` — the cross-spelling case fails. Let the extern prelude win over the local item — R4 fails. Read `>` before `::` as a global root — R6 fails. Resolve `<T>::f` as a plain path — `inline_qualified_path_is_the_type_directed_bound` fails.

6. **Amend the active spec, pins and stale prose in the same change.** (1 day) Move the accepted draft into `openspec/specs/inline-symbol-path-confinement/spec.md`; add every `PINNED-BY` in the draft; add the mutation records for the receiver-method bound's new pin and for the generic-parameter bound's pin (H2), and commit them before running `pin_bites`, which reads it from `HEAD`; rewrite `resolve_head`, `use_trees_with_modules` and `use_statements` docs and remove `_current_module`; sweep every tracked live file for the retired claim (`keyword head`, `not starting with a keyword`, the old test name) including test names and `PINNED-BY`s.
   - Acceptance: every scenario has a registered observation source; every cited test exists; one use-tree walker remains; `observation_bound_model` and the generated projections are fresh.
   - Negative run: make explicit local bindings lose precedence — its pinned scenario fails. Revert the new pin's mutation target — `pin_bites` reports the mutation surviving. A deliberately dangling `PINNED-BY` is rejected by the spec pin check.

7. **Re-run the matrix and record the migration.** (1 day) Re-run both probe scripts against the implementation, default and strict; update the target columns from executed output; write the `CHANGELOG.md` BREAKING and Migration notes from design i; run the full Definition of Done.
   - Acceptance: every target cell matches executed output with its raw record; no `syn` or `hunyi` dependency; Definition of Done run in full.
   - Negative run: disable the `super::*` fixed-point edge and re-run R1 through the script; the recorded cell must change.

8. **Strip `openspec/changes/guibiao-typed-name-resolution/` before the squash.** The requirement's one home is then the active spec; the measurements and their raw logs remain reachable in the branch's commits and the pull request.

## Estimate

**10–12 engineer-days**, excluding review wait; the step estimates sum to 10–11.5. Against the previous 8–12 plan, the edition keyword table, its Reference snapshot, generator and parity check are gone; block scopes and the 2015 root rule are added in steps 3–4; and step 1 now costs freezing the unchanged answers before the refactor. The scope walk and the fixed point (steps 3–4) remain the risk; steps 2, 6 and 8 are small and independently reviewable.

## Fallback if 0.8.0 cannot carry the implementation

If the resolver cannot be implemented and verified within the 0.8.0 release window, revert the preceding guibiao resolution commit and add a `BACKLOG.md` item for the inline use/type false-negative class, with the R1 and B3/B4 observations, the risk, the compatibility class, and the trigger for revisiting the typed-scope design. The `_` refusal in step 2 can still land alone, since it does not depend on the graph.

That fallback gives up the prior commit's inline use/type-alias and pub-use resolution gains, its new prefix and glob behavior, and the associated five tests; it also postpones the shared scope table, block scopes, baseline normalization, and context-aware `::` handling. It returns to the measured pre-commit behavior, including its stricter rejection of the global prefix spelling. It does not claim the R1 or block-scope false negatives are fixed; it makes the architectural correction deferred and visible in the backlog instead of shipping the current mixed resolver.
