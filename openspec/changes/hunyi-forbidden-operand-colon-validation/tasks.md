# tasks: Hunyi Rejects a Malformed `::`-Path Operand Implementation Plan

Carries the measured reproduction and the four affected call sites from `design.md`, so none of
this needs re-deriving.

- [x] Add `has_empty_path_segment` beside `canonical_path_str` in `crates/hunyi/src/resolve/mod.rs`, and `malformed_path_operand_error` beside `unsafe_empty_allowed_error` in `crates/hunyi/src/errors.rs`. <!-- id: 0 -->
- [x] Wire the check into `exposure::module_findings` (the audited call site), guarding the `forbidden` list before its `canonical_path_str` mapping, mirroring `unsafe_findings`'s own guard placement. <!-- id: 1 -->
- [x] Wire the check into `shape_scan::operand_module_findings` (covers both `dyn_operand_module_findings` and `impl_trait_operand_module_findings`) and separately into `impl_trait::impl_trait_operand_subtree_findings`, which canonicalizes its own copy of the forbidden list. <!-- id: 2 -->
- [x] Wire the check into `forbidden_marker::forbidden_marker_findings` for `must_not_acquire`/`and_not_acquire`'s forbidden set. <!-- id: 3 -->
- [x] Add regression coverage in `crates/hunyi/src/tests.rs` for all four call sites: leading `::`, trailing `::`, doubled `::`, each asserting the constitution error (not a silent empty result), plus the bare-string control still reacting. <!-- id: 4 -->
- [x] Write the new requirement into `openspec/changes/hunyi-forbidden-operand-colon-validation/specs/semantic-signature-coupling/spec.md`, keeping it visibly distinct from the existing source-spelling requirement it sits beside. <!-- id: 5 -->
- [x] Write the one-line inherited-validation cross-reference into the dyn-trait-operand-boundary and impl-trait-operand-boundary spec deltas. <!-- id: 6 -->
- [x] Write the narrower (trailing-`::`-only) requirement into the forbidden-marker spec delta, since leaf matching is a different mechanism and needs its own scenario rather than a copy of signature-coupling's. <!-- id: 7 -->
- [x] Add the adopter-facing `CHANGELOG.md` `[Unreleased]` → `### Fixed` entry, naming the four affected DSL methods and that the change is a non-breaking tightening (a whole-repo grep found no existing usage of the malformed spelling). <!-- id: 8 -->
- [x] Run the full Definition of Done from the workspace root and report actual output. <!-- id: 9 -->
