## 1. Fix

- [x] 1.1 Add `utf8_scalar_len(lead: u8) -> Option<usize>` and
      `simple_char_literal_scalar_len(bytes: &[u8], i: usize) -> Option<usize>` helpers
      (`crates/guibiao/src/module_scan/lexer.rs`).
- [x] 1.2 Replace the "simple char literal" branch's fixed 3-byte assumption
      (`i + 2 < bytes.len() && bytes[i + 2] == b'\''`) with `simple_char_literal_scalar_len`, and
      skip `1 + len + 1` bytes (opening quote, scalar, closing quote) instead of the hardcoded `3`.

## 2. Regression

- [x] 2.1 Added `a_non_ascii_char_literal_adjacent_to_a_brace_literal_does_not_leak_a_spurious_brace`
      — the exact audit trigger (`['«','{']`, no space), boundary anchored above the affected
      module, asserting the real violation.
- [x] 2.2 Added `a_boundary_anchored_directly_at_the_previously_dropped_module_resolves` — the
      identical fixture with the boundary anchored *at* `crate::hidden` directly (used to fail loud
      with a constitution error instead of silently passing).
- [x] 2.3 Added `a_non_ascii_char_literal_adjacent_to_a_brace_literal_in_a_match_arm_does_not_leak`
      — the audit's cited "everyday form" (`match c { 'é'|'{' => … }`, no space around `|` — a
      spaced pipe was confirmed, while writing this test, to pass even with the bug present, so the
      exact no-space spelling matters).
- [x] 2.4 Added `the_spaced_spelling_of_the_same_array_literal_already_reacts_and_keeps_reacting` —
      a control locking in that the already-correct spaced spelling (`['«', '{']`) keeps working.
- [x] 2.5 Non-vacuous verification: stashed the `lexer.rs` fix, reran all four tests. The three
      defect-exercising tests failed exactly as predicted (silent pass / wrong constitution error);
      the spaced-spelling control test correctly kept passing (confirming it tests a genuinely
      different, already-working case, not a vacuous duplicate). Restored, confirmed all four green.

## 3. Documentation

- [x] 3.1 Added a CHANGELOG `[Unreleased] ### Fixed` entry. No **BREAKING** marker — this fixes a
      false negative (a real import silently passing), not an identity shape; no existing baseline
      is invalidated either way.
- [x] 3.2 No spec-text change — `module-boundary`'s existing import-detection requirement already
      promises this behavior; this closes an implementation gap against it, not a requirement whose
      text needs to grow. Archived with `--skip-specs`.

## 4. Definition of Done

- [ ] 4.1 Run the full local gate list from `AGENTS.md` (build, three clippy passes, fmt, full test
      suite, both doc passes, `cargo deny check`, release-coherence scripts, `test_examples.sh`).
- [ ] 4.2 Adversarial apply-stage review: confirm the declared reaction still bites, not a taste call.
