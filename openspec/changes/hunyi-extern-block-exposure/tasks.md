## 1. Fix

- [x] 1.1 Add a `syn::Item::ForeignMod` arm to `collect_item_exposures`
      (`crates/hunyi/src/collect.rs`), walking each `syn::ForeignItem::Fn`/`::Static` whose
      visibility is public through the existing `fn_seam`/`item_seam(ItemKind::Static, …)` +
      `paths_in_signature`/`paths_in_type` collectors — verbatim reuse, no new seam kind.

## 2. Regression

- [x] 2.1 Added `a_forbidden_type_in_an_extern_block_pub_fn_signature_is_observed`.
- [x] 2.2 Added `a_forbidden_type_in_an_extern_block_pub_static_is_observed`.
- [x] 2.3 Added `a_non_pub_extern_block_item_is_not_observed` — a non-`pub` foreign fn/static must
      NOT react, matching every other item kind's own-visibility rule.
- [x] 2.4 Non-vacuous verification: stashed the `collect.rs` fix, reran all three. The two positive
      tests failed exactly as predicted (empty findings where a violation was expected); the
      non-`pub` control test correctly kept passing. Restored, confirmed all three green.

## 3. Documentation

- [x] 3.1 Added a CHANGELOG `[Unreleased] ### Fixed` entry. No **BREAKING** marker — fixes a false
      negative, not an identity shape; no existing baseline is invalidated.
- [x] 3.2 Amended `semantic-signature-coupling`'s "Public-signature observation governs exposure"
      requirement to name `extern` block items as part of the observed surface, and added three
      scenarios (pub fn, pub static, non-pub control) — a real textual gap, not merely an
      implementation bug against fully-stated behavior (same pattern as the char-literal fix).

## 4. Definition of Done

- [ ] 4.1 Run the full local gate list from `AGENTS.md` (build, three clippy passes, fmt, full test
      suite, both doc passes, `cargo deny check`, release-coherence scripts, `test_examples.sh`).
- [ ] 4.2 Adversarial apply-stage review: confirm the declared reaction still bites, not a taste call.
