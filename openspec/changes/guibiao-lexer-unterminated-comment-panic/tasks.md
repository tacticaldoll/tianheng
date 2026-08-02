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
      Added a `[Unreleased] ### Fixed` entry describing the crash and its fix. (An independent
      propose-stage adversarial review caught this task checked off before the entry actually
      existed — corrected: the entry is now genuinely present in `CHANGELOG.md`.)
- [x] 3.2 Confirmed the `module-boundary` spec delta (new "Lexical hygiene never panics on malformed
      source" requirement) reads correctly against the landed code.

## 4. Definition of Done

- [x] 4.1 Ran the full local gate list from `AGENTS.md` — all green: `cargo build --workspace
      --all-targets`; the three clippy passes; `cargo fmt --all --check`;
      `TIANHENG_WORKSPACE_TESTS=1 cargo test --workspace --all-features` (every suite `ok`, 0
      failed); both `cargo doc` passes; `cargo deny check`
      (`advisories ok, bans ok, licenses ok, sources ok`); `scripts/test_release_coherence.sh` and
      `check_release_coherence.sh` (`ok release coherence (development: 0.3.0)`);
      `scripts/test_examples.sh` (`all examples reacted as declared`).
- [x] 4.2 Adversarial apply-stage review performed independently (not self-assessment): verified the
      root-cause trace with a standalone `from_utf8_lossy` length-growth check, probed ~11 additional
      edge-case fixtures (empty comment, lone `*`, 4-byte emoji, nested depth-2 unterminated, NUL
      byte, etc.) looking for a case that still panics with the fix applied — found none — and
      independently reverted the fix to confirm both regression tests fail with the exact panic
      before restoring it. One real finding: this task list had 3.1 checked off before the CHANGELOG
      entry actually existed — corrected in 3.1 above.
