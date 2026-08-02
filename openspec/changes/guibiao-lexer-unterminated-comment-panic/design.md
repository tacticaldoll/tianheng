## Context

Reproduced directly before designing the fix (not assumed from the audit doc): a temp fixture whose
`lib.rs` ends `/* 未完` (an unterminated block comment containing two 3-byte CJK characters, no
closing `*/`, no trailing newline) panics at `lexer.rs:66:28` — `index out of bounds: the len is 36
but the index is 36` — inside `strip_macro_bodies_tracked`'s default branch, indexing
`input_positions[i]` where `input_positions` is `strip_comments_and_strings_tracked`'s own
`positions` output.

Traced the desync to its source: the block-comment loop
(`while i + 1 < bytes.len() && depth > 0`) requires two bytes of lookahead to test for `*/`, so it
stops once fewer than two bytes remain — even if `depth` never reached 0 (genuinely unterminated).
The code then unconditionally pushed one separator space and left `i` exactly where the loop
stopped, without consuming a possible single trailing byte. When that byte is a UTF-8 continuation
byte (the tail of a multi-byte character whose lead byte(s) were consumed — and dropped — inside the
comment), the outer loop's next iteration matches none of the special cases and falls to the default
branch, pushing that lone invalid byte into `out` on its own. `String::from_utf8_lossy(&out)` then
replaces that single invalid byte with the 3-byte U+FFFD replacement character — `stripped.len() >
out.len() == positions.len()` — and the next stage's `input_positions[i]` indexing runs past the
map's end once its own `i` walks past the point where the string "grew."

## Goals / Non-Goals

**Goals:**
- An unterminated block comment (any trailing content, any encoding) never leaves a byte dangling
  outside the comment's dropped range — the scanner consumes through EOF once it has determined the
  comment cannot close.
- Two independent trigger shapes (the CJK-swallowing case, and a plain unterminated comment at EOF
  with no trailing newline) both react 0/1/2, never panic.

**Non-Goals:**
- Any other lexer crash class not reproduced here (e.g. `strip_macro_bodies_tracked`'s own balanced-
  delimiter matching for an unterminated macro invocation) — not fixed here without its own
  reproduction; speculative widening is exactly what this project's minimalism rule warns against.
- Any change to how a *terminated* comment (nested or not) is stripped — untouched.

## Decisions

- **Fix locus: consume through EOF only when `depth > 0` after the loop.** The loop's own exit
  condition already distinguishes "closed" (`depth == 0`, `i` correctly past the closing `*/`) from
  "ran out of room to check" (`depth > 0`, `i` may still be short of `bytes.len()` by one byte) — so
  `if depth > 0 { i = bytes.len(); }` is the minimal correct fix, touching only the truly-unterminated
  path and leaving the terminated path's byte accounting untouched.
- **`positions.push(i)` after the EOF jump is still correct.** The pushed value becomes one-past-end
  (`bytes.len()`), used only as a position *bound* by downstream consumers (`read_path_string`'s own
  doc comment already treats "the end of the file" as a safe bound value for the identical position-
  tracking scheme) — never dereferenced as `original_bytes[that_value]` directly.

## Risks / Trade-offs

- **[Risk] A different lexer crash class exists but wasn't reproduced here.** → **Mitigation**:
  explicitly scoped out (see Non-Goals); the audit's remaining unverified findings are each their
  own change, not silently absorbed into this one.
- **[Risk] The fix is silent (no panic) but produces a wrong result instead of a crash.** →
  **Mitigation**: both regression tests assert the actual violation set, not just `result.is_ok()` —
  a `Clean` outcome (no violation at all) would fail the assertions just as loudly as a panic would.

## Migration Plan

1. Land the one-line fix plus its explanatory comment in `lexer.rs`.
2. Add two regression tests, each with a different absolute position for the swallowed trailing
   byte, verified non-vacuous by reverting the fix and confirming both panic exactly as before.
3. No CHANGELOG **BREAKING** marker needed — a crash becoming a correct reaction is a pure bug fix,
   not a behavior an adopter could have depended on.

## Open Questions

None outstanding.
