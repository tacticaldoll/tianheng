## Why

`strip_comments_and_strings_tracked` (`crates/guibiao/src/module_scan/lexer.rs`) panics — `index out
of bounds` — on any governed source file ending in an unterminated block comment that swallows a
multi-byte UTF-8 character. A crash is none of PROJECT.md's Core Contract outcomes (0 clean / 1
violation / 2 constitution error), and it aborts the whole process rather than reacting.

Root cause, traced directly (not assumed from the audit doc's paraphrase): the block-comment loop
stops peeking once fewer than two bytes remain, which — for a comment that is genuinely unterminated
(runs to EOF) — can leave exactly one trailing byte unconsumed. When that byte is the orphaned tail
of a multi-byte character whose lead byte(s) were already dropped inside the (still-open) comment,
the outer loop re-scans it as ordinary code and pushes it into `out` alone: an invalid, standalone
UTF-8 continuation byte. `String::from_utf8_lossy` then replaces that one invalid byte with the
3-byte U+FFFD replacement character, *lengthening* the string relative to the position map built
alongside `out` — the next pipeline stage's `input_positions[i]` lookup then indexes past the map's
end.

## What Changes

- When a block comment's closing `*/` is never found before EOF, the scanner now consumes through
  EOF as part of that (still-open) comment, instead of leaving a dangling trailing byte to be
  re-scanned as code.
- No new lookup or observation source — the fix corrects how many bytes the existing loop consumes
  in an already-detected (`depth > 0` after the loop) unterminated state.

## Capabilities

### New Capabilities

(none)

### Modified Capabilities

- `module-boundary`: adds a requirement that lexical hygiene never panics on malformed source
  (specifically: an unterminated block comment swallowing a multi-byte character), reacting 0/1/2
  like any other input instead.

## Impact

- Affected code: `crates/guibiao/src/module_scan/lexer.rs` only.
- No public API/DSL/builder change. No baseline format change — this fixes a crash, not an identity
  shape, so no existing baseline is invalidated.
- Out of scope: any other malformed-input crash class (this proposal fixes the one reported and
  reproduced; a different trigger would be its own finding, not folded in speculatively).
