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
- [x] 2.6 An independent apply-stage review (see section 4.2) suggested two further cases: a 4-byte
      scalar (emoji) and "3+ chained literals." Added
      `a_four_byte_scalar_char_literal_adjacent_to_a_brace_literal_does_not_leak` — verified
      non-vacuous immediately. For the chained case, two constructions were tried and found
      **vacuous** first: three literals with no separator at all never lands the old check's
      coincidental match, and `['«','{','}']` (a matched brace pair) leaks two structural characters
      that net to zero depth change and corrupt nothing — both confirmed to pass identically with
      and without the fix, so neither was kept. `two_unmatched_braces_cascading_from_chained_char_
      literals_do_not_leak` (`['«','{','{']`, two unmatched opens) is what actually cascades; verified
      non-vacuous the same way as the others.
- [x] 2.7 Full non-vacuous sweep across all 6 final tests: reverted the fix once more, ran the whole
      set — the 5 defect-exercising tests failed exactly as predicted, the spaced-spelling control
      correctly still passed. Restored; all 313 guibiao tests green.

## 3. Documentation

- [x] 3.1 Added a CHANGELOG `[Unreleased] ### Fixed` entry. No **BREAKING** marker — this fixes a
      false negative (a real import silently passing), not an identity shape; no existing baseline
      is invalidated either way.
- [x] 3.2 Spec text **does** need a small amendment — corrected after review found `module-boundary`'s
      existing requirement enumerates comments and string literals as stripped-before-scanning but
      never mentions char literals. Added a `specs/module-boundary/spec.md` MODIFIED delta (one
      sentence amended, one new scenario) rather than leaving the requirement textually silent on a
      literal category that demonstrably needs the same hygiene. Archived normally (not
      `--skip-specs`, reversing the original propose-stage plan).

## 4. Definition of Done

- [x] 4.1 Ran the full local gate list from `AGENTS.md` — all green: `cargo build --workspace
      --all-targets`; the three clippy passes; `cargo fmt --all --check`;
      `TIANHENG_WORKSPACE_TESTS=1 cargo test --workspace --all-features` (every suite `ok`, 0
      failed); both `cargo doc` passes; `cargo deny check`
      (`advisories ok, bans ok, licenses ok, sources ok`); `scripts/test_release_coherence.sh` and
      `check_release_coherence.sh` (`ok release coherence (development: 0.3.0)`);
      `scripts/test_examples.sh` (`all examples reacted as declared`).
- [x] 4.2 Independent apply-stage adversarial review performed (not self-assessment): re-derived the
      root-cause trace from code, constructed and ran three additional edge-case fixtures (4-byte
      emoji, invalid lead byte, chained literals) directly via `cargo test`, independently redid the
      non-vacuous revert-and-confirm, and checked the module-boundary spec and CHANGELOG for
      accuracy. Found the fix correct and generalizing properly, flagged the spec-text gap (3.2) and
      an imprecise design.md risk claim (corrected — the "invalid lead byte" fallback is unreachable
      dead code given the `&str` invariant, not a live risk). PASS verdict.
