## 1. Comment-tolerant mod-name skip

- [x] 1.1 Added `skip_space_and_comments` (`crates/louke/src/audit/scan.rs`), mirroring
      `skip_ascii_space` but also stepping over any interleaved comment via the existing
      `skip_literal_or_comment`.
- [x] 1.2 Used it at both positions between the `mod` keyword and its terminator (before AND after
      the name), replacing `skip_ascii_space`.

## 2. Block-scoped path-mod descent

- [x] 2.1 Generalized `collect_scope_modules`'s catch-all `{…}` handler to recurse into any
      unrecognized brace scope (fn/const/static body, bare block, match arm, …) via
      `collect_scope_modules` itself, instead of skipping it as one opaque unit — the enclosing
      bases thread through unchanged (no new directory component, unlike a NAMED inline `mod x { …
      }`), and arm membership is inherited.

## 3. cfg_attr(path) union

- [x] 3.1 Added `cfg_attr_paths: Vec<String>` to `ModPreambleAttrs`.
- [x] 3.2 Added `paren_group_end` (mirroring `attr_group_end`'s `[]`-balance tracking, for a
      `cfg_attr`'s own `(...)` argument list) and `find_path_meta_value` (an identifier-precise scan
      for a `path = "…"` meta within that span, reusing `read_path_string`).
- [x] 3.3 Added a `b"cfg_attr"` match arm in `mod_preamble_attrs`'s attribute-matching pass,
      populating `cfg_attr_paths` from every SEPARATE `cfg_attr`-wrapped `#[path]` attribute on the
      declaration (stacked, not nested).
- [x] 3.4 Updated the consumer (`collect_scope_modules`'s `;` branch): when no unconditional `#[path]`
      is present, union every `cfg_attr_paths` candidate that resolves with the conventional file;
      absence tolerated only when NEITHER resolves anywhere AND no other cfg-conditional gate
      (`attrs.cfg || in_transparent_arm`) applies.
- [x] 3.5 Nested `#[cfg_attr(a, cfg_attr(b, path = "…"))]` is an explicit, documented non-goal — not
      attempted by this hand-rolled scanner.

## 4. Regression

- [x] 4.1 `a_comment_between_mod_and_its_name_does_not_drop_the_module` +
      `a_comment_between_the_mod_name_and_its_terminator_does_not_drop_the_module` — the first audit
      finding, both comment positions.
- [x] 4.2 `a_path_mod_inside_a_function_body_reacts` +
      `a_path_mod_inside_a_nested_bare_block_reacts` — the second audit finding, both a direct fn
      body and one bare block deeper.
- [x] 4.3 `two_cfg_attr_path_declarations_covering_every_platform_are_scanned_not_erred` +
      `..._are_clean_when_probes_match` — the third audit finding (the false-positive
      `ConstitutionError` on always-compiling source), both the reacting-typo and genuinely-Clean
      cases.
- [x] 4.4 `a_missing_cfg_attr_path_target_is_tolerated_when_the_conventional_file_backs_the_module` —
      confirms the union's absence tolerance is additive, not a broadening that masks a real error.
- [x] 4.5 Non-vacuous verification: each of the three fixes independently reverted (comment-skip →
      plain `skip_ascii_space`; brace-descent → the old opaque skip; `cfg_attr_paths` union →
      conventional-file-only), confirming its own regression tests fail in the predicted way, then
      restored. Full suite green after each restore (116 tests, up from 109).

## 5. Documentation

- [x] 5.1 Added a CHANGELOG `[Unreleased] ### Fixed` entry. No **BREAKING** marker — closes false
      negatives and one false positive, not an identity shape; no baseline is invalidated.
- [x] 5.2 Updated the stale doc comments describing the unimplemented "cfg_attr reads as cfg" claim:
      `ModPreambleAttrs`'s own field docs, `mod_preamble_attrs`'s function doc, and
      `audit_probe_coverage_with_markers`'s public doc in `audit.rs`.
- [x] 5.3 Added a `MODIFIED Requirements` delta to `runtime-origin-assertion`'s "Root-aware audit
      excludes unreachable source files" requirement, correcting its own stated `cfg_attr(path)`
      claim and adding scenarios for all three fixes.

## 6. Definition of Done

- [x] 6.1 Run the full local gate list from `AGENTS.md` (build, three clippy passes, fmt, full test
      suite, both doc passes, `cargo deny check`, release-coherence scripts, `test_examples.sh`).
- [ ] 6.2 Adversarial apply-stage review: confirm the declared reaction still bites, not a taste
      call.
