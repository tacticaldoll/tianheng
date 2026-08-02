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

- [x] 4.1 Ran the full local gate list from `AGENTS.md` — all green: `cargo build --workspace
      --all-targets`; the three clippy passes; `cargo fmt --all --check`;
      `TIANHENG_WORKSPACE_TESTS=1 cargo test --workspace --all-features` (every suite `ok`, 0
      failed); both `cargo doc` passes; `cargo deny check`
      (`advisories ok, bans ok, licenses ok, sources ok`); `scripts/test_release_coherence.sh` and
      `check_release_coherence.sh` (`ok release coherence (development: 0.3.0)`);
      `scripts/test_examples.sh` (`all examples reacted as declared`).
- [x] 4.2 Independent apply-stage adversarial review performed (not self-assessment): confirmed
      seam-identity matches an ordinary item's exactly (by direct code comparison, not the doc
      comment alone), probed 3 further edge cases (foreign `Type`/macro items, multiple ABI
      strings, restricted-visibility items) via temporary tests — none broke the fix — and
      independently redid the non-vacuous revert-and-confirm. One real finding: the proposal's
      Non-Goals section understated that the identical gap also exists in **this same file**
      (`collect_item_async_exposures`, `collect_item_return_impl_traits`, and
      `collect_item_dyn_exposures`), not only in the separately-cited `syn_util.rs:439`. Corrected
      in `proposal.md`/`design.md` — named explicitly as follow-up candidates, not fixed here
      (none has its own audit reproduction or regression test yet). PASS verdict overall.
