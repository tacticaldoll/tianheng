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

## 4. Documentation

- [x] 4.1 Added a CHANGELOG `[Unreleased] ### Fixed` entry. No **BREAKING** marker — false negatives
      closing, not an identity shape; no existing baseline is invalidated.
- [x] 4.2 Added `MODIFIED Requirements` deltas to `semantic-unsafe-confinement`,
      `semantic-trait-impl-locality`, `semantic-forbidden-marker`, `semantic-signature-coupling`,
      `semantic-dyn-trait-operand-boundary`, and `semantic-impl-trait-operand-boundary` — each
      replaces its stated "`cfg_attr`-wrapped `#[path]` is an unfollowed bound" language (crate-wide
      walk only) with the union-observation rule, while leaving each spec's SEPARATE
      single-module-anchor bound (where stated) untouched.

## 5. Definition of Done

- [x] 5.1 Run the full local gate list from `AGENTS.md` (build, three clippy passes, fmt, full test
      suite, both doc passes, `cargo deny check`, release-coherence scripts, `test_examples.sh`).
- [ ] 5.2 Adversarial apply-stage review: confirm the declared reaction still bites, not a taste call
      — including whether any further `scan_crate` consumer or edge case was missed.
