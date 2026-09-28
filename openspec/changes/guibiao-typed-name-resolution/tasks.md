# Implementation plan

These tasks are for a later implementation after review accepts the proposal. Keep the proposal branch source-free.

## Ordered steps

1. **Freeze the contract and add one owned probe corpus.** Add the R1 private-import probe, retain all five currently unpinned `per_target_corpus.rs` tests, add explicit source fixtures for R2 baseline identity and R3/R4/R6, and move the `md5x_dep` fixture under `RootProbe.dir`.
   - Acceptance: each probe builds with its declared edition; every current and target result in `measurements.md` has a named test. Run the new fixture test twice concurrently and verify each owned root is removed.
   - Negative run: temporarily mutate the existing engine's `resolve_head` to leave every call head unresolved, then run the existing `inline_resolves_a_type_alias` test; it must lose its finding and fail. This changes engine code, not the fixture. The cleanup acceptance itself runs two concurrent probes and checks their owned roots disappear.

2. **Introduce the typed scope graph without changing matching policy.** Add the per-module scope, item namespace, edition, named binding, glob edge, and external-root types. Populate direct items and explicit `use`/`pub use` entries from the single use-tree walker. Remove duplicate walker implementations only after all consumers can read the shared records.
   - Acceptance: existing inline alias, type alias, public re-export, and sibling-isolation probes keep their expected findings; import reports remain unchanged.
   - Negative run: mutate binding lookup to omit the nearest module scope; the enclosing-module test must fail while the sibling test remains clean.

3. **Resolve glob edges and inherited scope to a fixed point.** Carry visibility through glob edges and re-export closure, preserve candidate sets, and return cannot-judge for distinct ambiguous candidates. Record package/external-prelude roots, including dependency aliases and sysroot names.
   - Acceptance: R1, the existing glob-hazard cases, and `super::super::*` probes resolve with the expected rule findings. An ambiguous-use fixture must produce exit 2; the corresponding Cargo-invalid use documents why it is not silently selected.
   - Negative run: mutate the worklist to skip one parent glob edge; R1 must fail. Mutate candidate coalescing to select the first distinct target; the ambiguity test must fail.

4. **Route path occurrence resolution and prefix validation through the graph.** Record edition and path-root context for each occurrence, distinguish local binding from extern-prelude resolution, canonicalize raw and equivalent root spellings before matching, then apply default versus strict-external policy.
   - Acceptance: R2's normalized baseline suppresses across spellings; R3 is clean/violation by mode for both forms; R4 stays clean because the local item wins; R6 stays clean in both modes.
   - Negative run: mutate `RuleKey` construction to preserve a leading colon and require the cross-spelling baseline case to fail. Mutate precedence so extern-prelude wins over the local item and require R4 to fail. Mutate path-start classification so `>` means global root and require R6 to fail.

5. **Own edition keywords in one table.** Separate boundary-head validation from macro negation, define strict/reserved/weak/underscore categories by edition, normalize raw identifiers, and add a source-anchored corpus generated into Cargo probes.
   - Acceptance: all measured R5 rows and additional raw-form edition probes build and react as declared; invalid prefixes return exit 2 with a specific `r#head` suggestion when legal.
   - Negative run: mutate `gen` to reserved in 2021 and require the 2021 scenario to fail; mutate `_` to a valid head and require the invalid-head scenario to fail; mutate raw normalization to retain `r#` and require the raw baseline/identity assertion to fail.

6. **Amend the active capability spec and retire stale prose in the same change.** Move the accepted draft requirement/scenarios into `openspec/specs/inline-symbol-path-confinement/spec.md`; add PINNED-BY references for the five existing tests and the new R1/public-re-export guards; name remaining bounds and their pins. Rewrite docs for `resolve_head`, `use_trees_with_modules`, and `use_statements`, and remove `_current_module`.
   - Acceptance: every scenario has a registered observation source; all cited tests exist; one use-tree walker remains; generated docs and the law reaction remain current.
   - Negative run: mutate one requirement-representing engine branch (for example, make explicit local bindings lose precedence) and require its pinned scenario to fail. A deliberately dangling PINNED-BY reference must be rejected by the repository's spec pin check.

7. **Run the complete matrix and record adopter migration.** Re-run all 29 Cargo probes against the target implementation, default and strict, with serial Cargo execution and `CARGO_BUILD_JOBS=4`; rerun R2 baseline cross-spelling. Update the measurement table from actual output, add `CHANGELOG.md` BREAKING/Migration notes, and run Tianheng's required reaction for the eventual code change.
   - Acceptance: the target column in `measurements.md` matches executed output; every command retains raw output and exit code; no `syn` or `hunyi` dependency is added.
   - Negative run: mutate the resolver engine to skip the entire `super::*` fixed-point edge and rerun R1; the complete verification must fail. Do not create a negative by changing only a probe fixture.

## Estimate

Estimated implementation effort: **8–12 engineer-days**, excluding review wait time. The high-risk work is the shared scope/fixed-point model and preserving correct behavior across the existing consumers; doc/spec migration and probe ownership are smaller, independently reviewable steps.

## Fallback if 0.8.0 cannot carry the implementation

If the resolver cannot be implemented and verified within the 0.8.0 release window, revert the preceding guibiao resolution commit and add a `BACKLOG.md` item for the inline use/type false-negative class, with the R1 observation, the risk, the compatibility class, and the trigger for revisiting the typed-scope design.

That fallback gives up the prior commit's inline use/type-alias and pub-use resolution gains, its new prefix and glob behavior, and the associated five tests; it also postpones the shared scope table, edition-aware validation, baseline normalization, and context-aware `::` handling. It returns to the measured pre-commit behavior, including its stricter rejection of the global prefix spelling. It does not claim the current R1 false negative is fixed; it makes the architectural correction deferred and visible in the backlog instead of shipping the current mixed resolver.
