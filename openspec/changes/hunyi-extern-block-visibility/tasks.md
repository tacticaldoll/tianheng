# tasks: Hunyi Extern-Block Visibility Observation Implementation Plan

- [x] Reproduce the gap directly against the real public API (`hunyi::check_visibility`) with a
      control/treatment probe before touching code: the `unsafe extern "C"` block reacts under
      `SemanticBoundary::must_not_expose` (`ed19dce`) but not under
      `VisibilityBoundary::must_not_declare_pub`/`max_visibility(Module)` — confirming the gap is
      not a ceiling-rank artifact. <!-- id: 0 -->
- [x] Grep for every existing caller of `item_observation`/`item_observation_parts` in
      `crates/hunyi/src` before widening their signature, confirming `visibility_findings` is the
      only consumer. <!-- id: 1 -->
- [x] Widen `item_observation_parts`/`item_observation` (`crates/hunyi/src/syn_util.rs`) from
      `Option`-returning to `Vec`-returning, converting every existing arm's `Some(...)` to
      `vec![...]` with no behavior change, and add the `syn::Item::ForeignMod` arm covering
      `ForeignItem::Fn`/`Static`/`Type` (Decision 3), reusing `VisibleItemKind::Fn`/`Static`/`Type`
      verbatim with no new kind (Decision 2). <!-- id: 2 -->
- [x] Update `visibility_findings`'s one call site (`crates/hunyi/src/visibility.rs`) from
      `filter_map` to `flat_map` to consume the widened `Vec`. <!-- id: 3 -->
- [x] Add regression coverage in `crates/hunyi/src/tests.rs`: multiple `pub` foreign items in one
      block (`Fn`+`Static`+`Type`, exercising the `Option`→`Vec` widening itself), the identical
      shape in the plain 2021-edition `extern "C"` form, an all-non-pub control in the same block
      shape, and a restricted-visibility (`pub(crate)`/`pub(super)`) foreign item under a `Super`
      ceiling. <!-- id: 4 -->
- [x] Verify non-vacuously: revert the `syn_util.rs`/`visibility.rs` fix, confirm every new
      positive test fails and the non-pub control still passes, then restore the fix and confirm
      the full suite is green again. <!-- id: 5 -->
- [x] Add the adopter-facing `CHANGELOG.md` `[Unreleased]` → `### Fixed` entry beside `ed19dce`'s
      own entry, naming the sibling-capability scope precisely (what `ed19dce` fixed vs. what this
      closes) and the `Type` scope decision. <!-- id: 6 -->
- [x] Modify `semantic-visibility-boundary`'s "Bare-pub item observation" requirement (delta under
      `specs/semantic-visibility-boundary/spec.md` in this change) to name the extern-block surface
      in the observed item-kind list, phrased consistently with `ed19dce`'s own
      `semantic-signature-coupling` delta wording. <!-- id: 7 -->
- [x] Run the full Definition of Done from the workspace root (build, four clippy passes including
      `--workspace` and `-p louke`, fmt --check, `TIANHENG_WORKSPACE_TESTS=1` test --workspace
      --all-features, `RUSTDOCFLAGS="-D warnings" cargo doc --workspace --no-deps --all-features`)
      and report actual output. <!-- id: 8 -->
