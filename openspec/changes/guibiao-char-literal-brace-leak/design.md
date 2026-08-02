## Context

Reproduced directly before designing the fix: a fixture with `lib.rs = "pub mod forbidden;\nconst
Q: [char; 2] = ['«','{'];\npub mod hidden;\n"` (where `hidden.rs` imports the forbidden type) returns
`Ok(())` with zero violations — a real forbidden import silently passes.

Traced the mechanism in `strip_comments_and_strings_tracked`'s char-literal branch: the "simple char
literal" check `i + 2 < bytes.len() && bytes[i + 2] == b'\''` assumes the literal's payload is
exactly one byte. For `'«'` (2-byte UTF-8: `0xC2 0xAB`), `bytes[i+2]` is the *second byte of «
itself*, not a quote, so the check fails and the branch falls to "stray quote": only the opening `'`
is pushed to `out`, and «'s two raw bytes are left to be re-scanned by the outer loop's default
branch as ordinary code (never re-entering the char-literal branch, since neither byte is `'`).

The outer loop reaches «'s **real** closing `'` next and re-enters the char-literal branch there,
now misreading it as a **new** opening quote. In `['«','{']`, the three bytes immediately following
that quote are `,`, `'` (the next literal's real opening quote) — exactly matching the buggy check's
assumption (quote, one byte, quote), so the branch swallows all three as a fake `','`-literal,
including the *real* opening quote of `'{'`. `{` is now unprotected and gets pushed to `out` as
plain code by the default branch on the next iteration — the reachability walker (which counts
braces to find top-level items) reads it as a genuine structural brace and mistracks nesting for
everything after it.

## Goals / Non-Goals

**Goals:**
- A char literal's real byte length (1–4, from its UTF-8 lead byte) is used to locate its closing
  quote, so a multi-byte scalar is recognized and consumed correctly regardless of what follows it.
- Four independently-verified shapes react correctly: the exact audit trigger (no space), the
  identical array with a space (already worked — must keep working), a boundary anchored directly
  at the previously-dropped module (used to fail loud instead of silently passing — now resolves),
  and the "everyday" `match` arm form with no space around `|`.

**Non-Goals:**
- Any other lexer defect not reproduced here.
- Changing how an *escaped* char literal (`'\n'`, `'\''`, `'\u{…}'`) is handled — untouched; that
  branch already scans forward to the real closing quote rather than assuming a fixed length.

## Decisions

- **Measure the scalar's byte length from its lead byte** (`utf8_scalar_len`), rather than, say,
  decoding the full `char` via `std::str`/`char::from_str` — this file is deliberately "pure byte
  processing... only `std`" per its own module doc, and locating a scalar's end only needs its
  length class (1/2/3/4), not full validation; the source is already real Rust `rustc` would accept,
  so a governed file's char literals are never actually malformed UTF-8 in practice.
- **No spec-text change.** `module-boundary`'s existing "Module imports observed from source use
  declarations" requirement already promises a real import reacts; this fixes an implementation
  bug against that already-stated behavior, not a requirement whose text needs to grow (unlike the
  unterminated-comment panic, which genuinely lacked any stated "never crashes" requirement).

## Risks / Trade-offs

- **[Risk] The fix could itself mis-measure a scalar for some byte value.** → **Mitigation**:
  `utf8_scalar_len` only classifies the lead byte into 1/2/3/4 (or `None` for a byte that cannot
  start a scalar) — a continuation byte or invalid lead byte correctly yields `None`, falling back
  to the pre-existing "stray quote" handling exactly as before for anything not a real char literal.

## Migration Plan

1. Land the fix plus `utf8_scalar_len`/`simple_char_literal_scalar_len` helpers.
2. Add four regression tests (the exact trigger, the anchored-at-dropped-module case, the everyday
   `match`-arm form, and a spaced-spelling control locking in already-correct behavior), each
   asserting the real violation/outcome, not just "did not panic."
3. Verify non-vacuous: revert the fix, confirm the three defect-exercising tests fail exactly as
   predicted (the control test must NOT fail, confirming it tests something different), restore.

## Open Questions

None outstanding.
