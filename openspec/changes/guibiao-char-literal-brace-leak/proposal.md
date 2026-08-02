## Why

`strip_comments_and_strings_tracked` (`crates/guibiao/src/module_scan/lexer.rs`) silently drops
every top-level `mod` declared after a non-ASCII char literal that sits immediately adjacent to a
`'{'` char literal (e.g. `['«','{']`, no space) — `guibiao::check` returns exit 0 Clean on source
that genuinely imports a forbidden type. Reproduced directly: a boundary anchored above the affected
module sees no violation at all; the same boundary anchored *at* the dropped module instead fails
loud (exit 2, "module not found among the reachable modules").

Root cause, traced by reading the code (not assumed from the audit doc): the "simple char literal"
branch assumed every char literal's payload is exactly one byte (`i + 2 < bytes.len() &&
bytes[i + 2] == '\''`), which holds for `'x'` but not for a multi-byte UTF-8 scalar (`'«'` is 2
bytes, `'未'` is 3). For a non-ASCII literal, that check fails, so the branch falls through to
treating the opening `'` as a lone stray quote — the scalar's raw bytes then leak into the cleaned
text as ordinary code. If a *second* char literal follows closely enough (as in `['«','{']`, where
only a comma separates them), the misread literal's real closing quote, the comma, and the next
literal's real opening quote can coincidentally match the old 3-byte assumption exactly (`',''`),
so that whole triplet gets swallowed as a fake char literal — including the next literal's genuine
opening quote. That literal's payload (here, `{`) then leaks into the cleaned text unprotected, and
the reachability walker misreads it as a real structural brace, throwing off its brace-depth
tracking for everything that follows.

## What Changes

- The "simple char literal" branch now measures the scalar's real UTF-8 byte length (1–4, from its
  lead byte) and looks for the closing `'` at the correct offset, instead of assuming exactly one
  byte.
- No new lookup or observation source — the fix only changes how many bytes a single already-
  visited branch consumes.

## Capabilities

### New Capabilities

(none)

### Modified Capabilities

(none) — this closes a defect against `module-boundary`'s already-stated contract (import detection
observed from source), not a requirement whose text needs to change. No literal field, exit-code
outcome, or DSL surface is added; the fix makes existing, already-specified behavior (a real
forbidden import reacts) hold for a source shape it previously mishandled.

## Impact

- Affected code: `crates/guibiao/src/module_scan/lexer.rs` only.
- No public API/DSL/builder change, no baseline format change (fixes a false negative, not an
  identity shape — an adopter's existing baseline is unaffected either way).
- Out of scope: any other lexer defect not reproduced here.
