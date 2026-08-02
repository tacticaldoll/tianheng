## 1. Fix

- [x] 1.1 In `strip_comments_and_strings_tracked`'s block-comment branch
      (`crates/guibiao/src/module_scan/lexer.rs`), after the inner `while` loop, consume through EOF
      (`i = bytes.len()`) when `depth > 0` (the comment never closed) — rather than leaving a possible
      dangling trailing byte to be re-scanned as code.

## 2. Regression

- [x] 2.1 Added `an_unterminated_block_comment_swallowing_a_multibyte_char_does_not_panic`
      (`crates/guibiao/src/tests.rs`), reproducing the exact audit trigger, asserting the actual
      violation (not just `result.is_ok()`).
- [x] 2.2 Added `an_unterminated_block_comment_at_end_of_file_with_no_trailing_newline_does_not_panic`,
      a second independently-positioned trigger of the same defect.
- [x] 2.3 Non-vacuous verification: stashed the `lexer.rs` fix, reran both tests, confirmed both panic
      exactly as before (`index out of bounds: the len is 17/36 but the index is 17/36`), restored,
      confirmed both green again.

## 3. Documentation

- [x] 3.1 No CHANGELOG **BREAKING** marker — a crash becoming a correct reaction is a pure bug fix.
      Added a `[Unreleased] ### Fixed` entry describing the crash and its fix.
- [x] 3.2 Confirmed the `module-boundary` spec delta (new "Lexical hygiene never panics on malformed
      source" requirement) reads correctly against the landed code.

## 4. Definition of Done

- [ ] 4.1 Run the full local gate list from `AGENTS.md` (build, three clippy passes, fmt, full test
      suite, both doc passes, `cargo deny check`, release-coherence scripts, `test_examples.sh`).
- [ ] 4.2 Adversarial apply-stage review: confirm the declared reaction still bites, not a taste call.
