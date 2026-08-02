# tasks: Hunyi Shared Path-Operand Validation Implementation Plan

- [x] Verify all four existing forbidden-operand call sites (`exposure.rs`, `forbidden_marker.rs`,
      `shape_scan.rs`, `impl_trait.rs`) are genuinely byte-identical in guard shape (type, predicate,
      error) before extracting, rather than assuming it from the prompt. <!-- id: 0 -->
- [x] Add `resolve::validate_path_operands` to `crates/hunyi/src/resolve/mod.rs`, beside
      `has_empty_path_segment`, and swap all four call sites to use it instead of the inline
      3-line guard — no behavior change at these four sites. <!-- id: 1 -->
- [x] Confirm the four Part-1 refactor sites' own existing tests still pass unmodified (no assertion
      touched), isolating the refactor from the Part-2 behavior change, and add the matching
      `CHANGELOG.md` `[Unreleased]` → `### Changed` entry. <!-- id: 2 -->
- [x] Reproduce the `BACKLOG.md`-recorded `allowed_locations` gap directly against
      `trait_impl_findings` and `unsafe_findings` (the pure hearts) BEFORE fixing it, confirming
      the actual failure direction (a spurious violation on a genuinely-in-place site, never a
      silent pass) rather than trusting the backlog text's characterization alone. <!-- id: 3 -->
- [x] Wire `validate_path_operands` into `trait_impl.rs::trait_impl_findings` (over `allowed`,
      before canonicalization and scanning) and `unsafe_confinement.rs::unsafe_findings` (over its
      already-canonicalized `allowed`, alongside the existing empty/crate-root guards). <!-- id: 4 -->
- [x] Add regression tests in `crates/hunyi/src/tests.rs`: a malformed-`allowed_locations`
      constitution-error test for each of `trait_impl.rs` and `unsafe_confinement.rs`, each paired
      with a well-formed control proving the identical genuinely-in-place site still passes clean.
      <!-- id: 5 -->
- [x] Close the `BACKLOG.md` `ACCEPTED DEBT` entry for this gap (move to `BUILT / HISTORY`), having
      confirmed both named call sites are fixed, and add the matching `CHANGELOG.md` `[Unreleased]`
      → `### Fixed` entry. <!-- id: 6 -->
- [x] Full-file grep `openspec/specs/semantic-trait-impl-locality/spec.md` and
      `openspec/specs/semantic-unsafe-confinement/spec.md` for "malformed"/"tolerate"/"empty segment"
      before drafting deltas, confirming neither carries a stale claim to correct (pure `ADDED`,
      no `MODIFIED`). <!-- id: 7 -->
- [x] Run the full Definition of Done from the workspace root (build, four clippy passes including
      `--workspace` and `-p louke`, fmt, `TIANHENG_WORKSPACE_TESTS=1` test, doc, `cargo deny`, both
      release-coherence scripts, `test_examples.sh`) and report actual output. <!-- id: 8 -->
