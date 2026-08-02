## 1. Crate-wide walk fix

- [x] 1.1 Added `cfg_attr_path_value` (`crates/hunyi/src/syn_util.rs`), extracting the target path
      from a `cfg_attr`-wrapped `#[path]` (including arbitrarily nested `cfg_attr`), mirroring
      `direct_path_value`'s existing NameValue-matching pattern.
- [x] 1.2 Replaced `resolve_child_modules`'s blanket `if has_path_attr(...) { continue; }` skip
      (`crates/hunyi/src/scan.rs`): an inline module's body is now always descended regardless of any
      `cfg_attr`-wrapped `#[path]` (irrelevant to an inline module's content); a file module's
      conventional file AND its `cfg_attr` target (if it exists on disk) are both read as separate
      unioned sources, deduped via the existing `seen_files` guard.
- [x] 1.3 Neither candidate existing, with no other cfg-conditional gate, remains a genuine scan
      error (exit 2) — `has_backing_source` is additive to the existing `cfg_conditional` tolerance,
      not a broadening of it.
- [x] 1.4 Updated the stale doc comment at the unconditional-`#[path]`-absent branch that referenced
      the old `has_path_attr` skip "below" (it no longer exists in that form).

## 2. Verified every direct `scan_crate` consumer

`scan_crate`'s crate-wide maps (re-exports, aliases, trait-impls, type-defs) back FIVE capabilities,
not only the two the original audit findings measured against (unsafe-confinement, via
`check_unsafe_confinement`). Each independently reproduced before being counted as fixed:

- [x] 2.1 `unsafe-confinement` (`unsafe_confinement.rs` → `scan_unsafe_sites`) — the audit's own
      measurement channel.
- [x] 2.2 `trait-impl-locality` (`trait_impl.rs` → `scan_crate`) — the `cfg_attr`-remapped-module
      test that previously asserted "out of scope" now asserts the impl reacts (both the single and
      nested-`cfg_attr` forms).
- [x] 2.3 `forbidden-marker` (`forbidden_marker.rs` → `scan_crate`) — covered transitively by the
      same crate-wide fix; no capability-specific reproduction needed beyond the shared mechanism
      already verified for 2.1/2.2/2.4/2.5 (identical code path).
- [x] 2.4 `signature-coupling` (`exposure.rs` → `scan_crate`, its OWN alias/re-export closure,
      distinct from `module_resolve.rs`'s separate single-module-anchor resolution) — reproduced a
      `cfg_attr`-path-hidden module's `pub use` re-export missing from the exposure query.
- [x] 2.5 `dyn-trait`/`impl-trait`'s **operand-scoped** boundaries (`crate_scope.rs::extern_resolution`
      → `scan_crate`, feeding `resolve_principal`) — reproduced the identical missing-re-export gap
      through `dyn_operand_module_findings`.

## 3. Regression

- [x] 3.1 `cfg_attr_wrapped_path_on_an_inline_module_is_still_observed` — the first audit finding
      (inline body dropped).
- [x] 3.2 `cfg_attr_wrapped_path_conventional_file_is_read_when_the_predicate_is_always_false` — the
      second audit finding (conventional file dropped under an always-false predicate).
- [x] 3.3 `cfg_attr_wrapped_path_target_is_read_when_the_conventional_file_is_absent` — the symmetric
      case (the `cfg_attr` target is what a build actually compiles).
- [x] 3.4 `cfg_attr_wrapped_path_with_neither_candidate_present_fails_loud` — confirms the fix did
      not turn a genuinely broken module into a silent pass.
- [x] 3.5 `a_cfg_attr_remapped_module_target_is_followed_when_the_conventional_file_is_absent` (plain
      and nested `cfg_attr` forms) — trait-impl-locality's own pre-existing test, updated from
      asserting the old "out of scope" behavior to the new correct reaction.
- [x] 3.6 `signature_coupling_reacts_through_a_cfg_attr_path_hidden_reexport` — signature-coupling's
      own crate-wide alias/re-export closure.
- [x] 3.7 `dyn_trait_operand_resolution_reacts_through_a_cfg_attr_path_hidden_reexport` — the shared
      `resolve_principal` mechanism dyn-trait's and impl-trait's operand-scoped boundaries both use.
- [x] 3.8 Non-vacuous verification: reverted `scan.rs` wholesale to the pre-fix `has_path_attr` skip,
      confirmed every one of 3.1–3.7 fails in the predicted way (including the two pre-existing tests
      at 3.5), restored. Full suite green after restore.

## 3a. Round-2 consumers (adversarial apply-stage review)

An independent review declined to accept round 1's "all five consumers, `module_resolve.rs`
correctly out of scope" narrative on its own terms and found `walk_subtree_modules` (async-exposure's
and impl-trait's SUBTREE-scope opt-in) shares the identical `resolve_child_modules` mechanism —
undercounted and misattributed to `module_resolve.rs` in round 1's own commit message:

- [x] 3a.1 `async_subtree_reacts_through_a_cfg_attr_wrapped_path_submodule` —
      `async_exposure_subtree_findings`'s own subtree walk, independently reproduced and fixed.
- [x] 3a.2 `impl_trait_subtree_reacts_through_a_cfg_attr_wrapped_path_submodule` — the identical
      shape at `impl_trait_subtree_findings`'s own subtree walk.
- [x] 3a.3 Non-vacuous verification for 3a.1 (reverted `scan.rs`/`syn_util.rs` to pre-fix, confirmed
      failure, restored); 3a.2 relies on the identical, already-verified shared function
      (`resolve_child_modules` via `walk_subtree_modules` → `collect_subtree`), not independently
      re-verified by revert given it is the same code path already isolated at 3a.1 and in section 3.
- [x] 3a.4 Fixed two stale doc comments describing the pre-fix "skip" behavior: `scan.rs`'s
      `walk_subtree_modules` doc, and `syn_util.rs`'s `has_path_attr`/`direct_path_value` docs
      (narrowed to correctly describe `module_resolve.rs`'s descent as the only remaining skip-bound
      caller).
- [x] 3a.5 Six additional counter-examples constructed by the review found no further bug: union of
      two different existing files; conventional/target resolving to the identical canonical file
      (deduped); deeply nested `cfg_attr(cfg_attr(path))`; an inline module's own nested file-children
      under a cfg_attr-wrapped path; `has_backing_source` combined with a co-occurring bare `#[cfg]`.

## 3b. Round-3: `module_resolve.rs`'s own false-negative (adversarial review)

A third independent review re-examined this change's OWN "already correct, fails loud" claim about
`module_resolve.rs::descend` (restated across rounds 1 and 2) and disproved it: a `cfg_attr`-wrapped
`#[path]` declaration silently absorbs into a resolving sibling's success (exit 0, not exit 2), and
even a LONE such declaration with an existing target file was never followed at all before this
round's fix.

- [x] 3b.1 Extended `descend()` (`crates/hunyi/src/module_resolve.rs`) with the identical union
      `resolve_child_modules` already applies: the `cfg_attr` target and the conventional file are
      both read when they exist on disk; `has_backing_source` gates the absence-tolerance the same
      way it does in `scan.rs`.
- [x] 3b.2 Deleted the now-fully-dead `has_path_attr`, `is_path_remap`, `applied_metas_remap`, and
      `meta_is_path_remap` (`syn_util.rs`) — `descend()` was their only remaining caller, now migrated
      to `cfg_attr_path_value`. Updated the surrounding doc comments (`direct_path_value`,
      `has_cfg_attr`) that referenced the deleted functions.
- [x] 3b.3 `cfg_attr_wrapped_path_resolves_through_its_own_target_with_no_sibling_at_all` — the LONE
      case (no sibling), now resolves rather than failing loud.
- [x] 3b.4 `cfg_attr_wrapped_path_sibling_reacts_through_its_own_file_not_absorbed_by_a_sibling` +
      `cfg_attr_wrapped_path_sibling_reacts_through_a_cfg_if_arm` — the sibling-absorption bug, both a
      bare-`#[cfg]` pair and a `cfg_if!` arm pair.
- [x] 3b.5 `cfg_attr_wrapped_path_with_no_sibling_and_no_backing_file_still_fails_loud` — confirms the
      genuinely-unbacked case still fails loud (exit 2), never silently passing.
- [x] 3b.6 Non-vacuous verification: reverted `module_resolve.rs`/`syn_util.rs` to the pre-3b state,
      confirmed 3b.3/3b.4 fail in the predicted way, restored. Full suite green after restore.
- [x] 3b.7 Added `MODIFIED Requirements` deltas closing the newly-corrected claim in
      `semantic-signature-coupling` (both its "Anchor resolution" requirement — the shared
      single-module-anchor property declaration — and its "Name resolution scope" requirement),
      `semantic-visibility-boundary`, `semantic-dyn-trait-boundary` (shape-only, module-scoped
      resolution), and `semantic-trait-impl-exposure` (shares signature-coupling's resolver). Swept
      every remaining hunyi-owned spec mentioning `cfg_attr` to confirm no other stale claim survived
      (guibiao's and louke's own separate scanners' specs are out of scope for this hunyi-only
      change and were left untouched).

## 4. Documentation

- [x] 4.1 Added a CHANGELOG `[Unreleased] ### Fixed` entry. No **BREAKING** marker — false negatives
      closing, not an identity shape; no existing baseline is invalidated.
- [x] 4.2 Added `MODIFIED Requirements` deltas to `semantic-unsafe-confinement`,
      `semantic-trait-impl-locality`, `semantic-forbidden-marker`, `semantic-signature-coupling`,
      `semantic-dyn-trait-operand-boundary`, `semantic-impl-trait-operand-boundary`,
      `semantic-async-exposure-boundary`, `semantic-impl-trait-boundary` (round 2), and — after round
      3's `module_resolve.rs` fix — `semantic-visibility-boundary`, `semantic-dyn-trait-boundary`, and
      `semantic-trait-impl-exposure`. Each replaces its stated "`cfg_attr`-wrapped `#[path]` is an
      unfollowed/fail-loud bound" language with the union-observation rule.

## 5. Definition of Done

- [x] 5.1 Run the full local gate list from `AGENTS.md` (build, three clippy passes, fmt, full test
      suite, both doc passes, `cargo deny check`, release-coherence scripts, `test_examples.sh`) —
      re-run after round 3's `module_resolve.rs` fix.
- [x] 5.2 Adversarial apply-stage review (rounds 2 and 3): round 2 confirmed the declared reaction
      still bites and closed the `walk_subtree_modules` undercounting plus two stale doc comments;
      round 3 found and closed a real, pre-existing gap in `module_resolve.rs` this change's own
      commits had twice misdescribed as "already correct."
