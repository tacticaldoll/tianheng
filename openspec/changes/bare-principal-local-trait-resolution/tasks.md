## 1. Carry the branch's local type namespace

- [ ] 1.1 Add `local_types: HashSet<String>` to `FileExternScope` with a doc line naming it the observation
      source for a bare principal, and build it once in `file_extern_scope` so `externs_type`'s difference
      reuses the same set instead of discarding it.

## 2. Gate and canonicalize the bare fallback

- [ ] 2.1 In `resolve_principal`, fire the fallback only when `file_scope.local_types` contains the
      canonical name; canonicalize the segment with `strip_raw`; drop the redundant
      `path.leading_colon.is_none()` conjunct.

## 3. Make the bound tests observe the drop

- [ ] 3.1 Re-point `dyn_operand_genuinely_unresolvable_bare_principal_is_a_bound` to forbid
      `["crate::m::Frobnicate"]`, and record that it fails against the unfixed resolver.
- [ ] 3.2 Re-point `impl_trait_operand_genuinely_unresolvable_bare_principal_is_a_bound` the same way,
      and record the same negative run.

## 4. Pin the two reacting directions, per capability

- [ ] 4.1 Keep `same_module_bare_trait_resolves_without_use` as the dyn reacting control and add
      `dyn_operand_bare_raw_identifier_local_trait_resolves_canonically`; record its failure without the
      `strip_raw` half.
- [ ] 4.2 Add the impl-trait twins — `impl_trait_operand_same_module_bare_trait_resolves_without_use` and
      `impl_trait_operand_bare_raw_identifier_local_trait_resolves_canonically` — each its own test, since
      三儀 ⊥ 三儀 and the register refuses one test cited by two capabilities.

## 5. Bring the declared law and its projections into agreement

- [ ] 5.1 Apply both delta specs to `openspec/specs/*` at sync, including the corrected auto-trait
      mechanism sentence and the impl-trait `Iterator` parenthetical.
- [ ] 5.2 Regenerate `docs/observation-bounds.md` (`BLESS=1 bash scripts/check_bound_register.sh`) and
      confirm the register reports clean afterwards.
- [ ] 5.3 Add the `**BREAKING**` `[Unreleased]` CHANGELOG entry stating both reaction directions and the
      baseline consequence.

## 6. Verify

- [ ] 6.1 `TIANHENG_WORKSPACE_TESTS=1 cargo test --workspace --all-features`, `cargo clippy` (all four
      passes), `cargo fmt --all --check`, `cargo doc` with `-D warnings`.
- [ ] 6.2 `bash scripts/check_bound_register.sh`, `bash scripts/test_bound_register.sh`,
      `bash scripts/check_reference_integrity.sh`, `bash scripts/check_whitespace_hygiene.sh`,
      `bash scripts/check_release_coherence.sh`.
