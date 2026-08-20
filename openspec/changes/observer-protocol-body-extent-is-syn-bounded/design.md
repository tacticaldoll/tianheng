## Context

`crates/kanhe/tests/observer_protocol.rs` reads a dimension observer's `bounds()` method in two separate steps:

- `anchor` decides **which** occurrence of the signature is the definition: it requires the signature to occur
  exactly once in the source, and to begin a (trimmed) line. This step is unrelated to the bug this change
  closes, and stays untouched.
- `function_body` then reads the definition's **extent**: from the anchor's opening brace to its matching
  closing brace, counted over `mask_line_comment_braces`'s output — the source text with `{`/`}` inside a
  line-comment tail replaced by a space, so a stray brace in a `// …` comment cannot prematurely close the
  count.

The mask never handled block comments (`/* … */`) or string literals, and its own doc said so: "a brace inside a
string, a character literal, or a block comment is counted as code." A `bounds()` body carrying a `}` inside a
block comment or a string literal therefore had its extent moved — closed too early, or (depending on brace
balance) too late.

This was declared, not silently present:
`observer-protocol/a-brace-inside-a-block-comment-or-a-string-literal-moves-the-read-body-extent-a-stated-bound`,
typed `Reached::OverReacts`, pinned by `a_brace_in_a_block_comment_moves_the_body_extent`. The pin's own
rationale is why it was safe to leave declared rather than close immediately: its one consumer,
`bounds_body`'s exact one-statement equality (`body == vec!["observation_bounds()"]`), cannot be satisfied by
*any* moved extent — a body that lost or gained a brace-worth of text never happens to look like exactly one
statement again — so the bug's only observable effect on that comparison is refusing a conforming method,
never accepting a divergent one. The same spec paragraph records that a second reader, once written over this
identical recognizer with a weaker "count and containment" comparison, took the identical moved extent and
turned it into a false *acceptance* of a divergent body — and was retired for it. That history is why the safety
property is written into this change's acceptance criteria rather than assumed: whatever replaces the extent
step must not merely happen to be safe for today's one comparison, it must refuse whenever it cannot verify the
body, so a future second comparison cannot repeat that retirement.

`crates/kanhe/Cargo.toml` added `syn` (workspace features `full`, `visit`, plus `extra-traits` locally) and
`proc-macro2` (`span-locations`) as dev-dependencies in the commit immediately before this one, for exactly this
purpose: kanhe's hand-rolled Rust-text scanners have produced a steady stream of lexing-edge-case bugs, and this
is the first of them replaced.

A spike (informal, not committed) confirmed `proc-macro2`'s `Span::byte_range()` is exact outside a proc-macro
context: parsing `"fn f() { a(); b(); }"` with `syn::parse_str::<syn::File>` and reading the block's
`brace_token.span.join().byte_range()` returned `7..20`, exactly `"{ a(); b(); }"` in the original string. This
is documented in `proc-macro2`'s own source (`fallback.rs`): the byte-offset map is built per parsed source
string, not against some global concatenation, so offsets read back are relative to the exact string handed to
`parse_str`.

## Goals / Non-Goals

**Goals:**

- Replace brace-counting with a real parse for the extent step alone. A comment or a string literal can no
  longer move the read extent, because `syn` never presents its contents as tokens to count.
- Preserve the safety direction the closed bound's pin required: where the new step cannot cleanly attribute a
  body to the anchored `fn`, it refuses — it never falls back to a guess.
- Leave the anchor step, and every scenario describing its behaviour (including the still-open
  whole-line-occurrence bound), untouched.
- Prove the fix differentially: the new step runs alongside the old one against the full `AnchorCase` table and
  the real corpus before the old internals are deleted, and a fixture the old step is documented to get wrong is
  added and shown to be read correctly by the new one.

**Non-Goals:**

- Fixing the anchor step. A whole-line occurrence of the signature inside a comment or a string literal still
  anchors the read exactly as a real definition would — that is a different, larger, cross-cutting bound shared
  with `kanhe::region`'s own `Executed` residue, and closing it needs the same instrument applied to a different
  step. `observer-protocol/a-whole-line-occurrence-that-is-not-the-definition-anchors-the-read-a-stated-bound`
  stays declared, unedited, in the spec.
- Making `bounds_body`'s comparison, or the region-discipline residue it inherits from `kanhe::region::Executed`
  (a block comment on its own line is not stripped as a comment there), any more permissive. A body that
  delegates exactly but carries a block comment on its own line still fails that comparison after this change —
  correctly, now, rather than as a side effect of a truncated extent.
- Touching `crates/guibiao/src/observer.rs`, `crates/hunyi/src/observer.rs`, `crates/louke/src/observer.rs`, or
  `crates/tianheng/src/runner.rs`. They are the reaction's subject, not its target, and this change reads them
  unmodified as part of the differential proof.

## Decisions

**Key on byte offset, not on name-search.** `syn_body_span` collects every body-carrying function-like item's
`(fn-keyword byte offset, block byte range)` via one `syn::visit::Visit` pass, then looks up the anchor's exact
byte offset in that collection. Matching by position rather than by "find a function of this name somewhere in
the file" keeps the extent step from silently widening what the anchor step already decided: a same-named
method belonging to an unrelated `impl` elsewhere in the file is real, parseable Rust, and a name-only search
would read its body as if it were the anchored one. Position-matching cannot do that, because the position is
fixed by the (untouched) anchor step before `syn` is ever consulted. Two distinct AST items cannot share one
`fn`-keyword byte offset in one source string, so this lookup is unambiguous by construction — it never needs a
tie-breaking rule.

**Cover `ItemFn`, `ImplItemFn`, and `TraitItemFn`.** The real corpus only exercises `ImplItemFn` (each
dimension's `bounds()` lives inside `impl Observer for … { … }`) and a free `ItemFn`
(`evaluate_constitution` in `crates/tianheng/src/runner.rs`), but a trait's own default-bodied method is the
same shape one level removed, and costs one more `Visit` override to cover.

**Decline on parse failure or missing match, uniformly.** `syn::parse_str::<syn::File>(text).ok()?` and
`.find(|(start, _)| *start == at)` are both `?`/`Option`-shaped, so "the file does not parse" and "nothing
matches the anchor" collapse into the same `None` the anchor step's own decline path already returns through.
No separate error message distinguishes them; both mean the same thing to every caller — there is nothing here
this reader can stand behind.

**Test the extent directly, not through `bounds_body`.** The rewritten pin
(`a_brace_in_a_block_comment_or_a_string_literal_no_longer_moves_the_body_extent`) asserts on
`function_body(...).whole()` — the raw extracted text — rather than on `bounds_body`'s parsed one-statement
list. Measured: a fixture with the brace-carrying construct on its own line still fails `bounds_body`'s exact
equality after the fix, because `kanhe::region`'s `Executed::rust()` does not strip a block comment written on
its own line (a separate, pre-existing, already-declared residue of that region, not of this reader). Asserting
through `bounds_body` would therefore still show `assert_ne!` holding — true both before and after the fix, for
two different reasons — and would not demonstrate that anything changed. Asserting on the raw extent shows the
real, intended difference: the old reader stopped at the comment's own brace, the new one reaches the real
closing brace.

**Update the `AnchorCase` rows the fix necessarily changes, and add one to keep the still-open bound
demonstrable.** This was the one place the intended scope (the extent step) could not be held perfectly
cleanly. `anchor()` is unedited, so it still returns `Anchor::At` for a whole-line signature copy sitting inside
a block comment or a string literal — the still-open bound's own trigger condition. What changed is only what
happens next: the old brace-counter would happily read a body out of the decoy's own text, while `syn` finds no
function-like item starting inside a comment or a string literal's token, because there is none — a
mathematical consequence of the anchor's own uniqueness requirement (if a second, real `fn` existed elsewhere to
be misread, the anchor would have counted two occurrences and declined already). So the two rows demonstrating
that bound by `Verdict::ReadsTheWrongBody` necessarily become `Verdict::Declines`. Left there, the requirement's
own text — "This bound SHALL be shown rather than described … the rows where it reads a body that is not the
method's are this bound" — would have no row left to satisfy it, even though the bound itself (and its
spec text) is untouched and still open. A new row, "a same-named method on an unrelated impl anchors the read,
the intended definition absent," keeps the demonstration alive using a shape the still-open scenario's own text
already covers ("or in any other position the reader does not distinguish from executed text"): a `bounds`
method on some unrelated `impl` is real, parseable Rust, and the anchor step cannot distinguish it from the
intended `Observer::bounds` by name alone. No spec text changes for this — the scenario already described the
class widely enough to include it.

**Delete `mask_line_comment_braces` rather than keep it dead.** Nothing else in this file uses it once
`function_body` no longer does.

## Risks / Trade-offs

- **The scope boundary between the extent step and the anchor step is not perfectly clean**, as the previous
  decision states. Mitigated by not editing `anchor`, `begins_a_line`, or the spec scenario that declares the
  anchor bound at all — only the `AnchorCase` table's recorded *consequence* of that bug changes, because the
  table is required to state the reader's real behaviour ("shown rather than described") and the real behaviour
  did change for two of its rows.
- **A future reviewer could read the new `AnchorCase` row as scope creep.** It is not a new bug or a new
  capability; it is the minimum needed to keep an unrelated, still-open, already-declared bound's own
  "shown rather than described" requirement satisfiable after this fix, using language the scenario already
  carries.
- **`syn::parse_str` failing on a legitimately unusual body** (heavy macro use, an `async` fn, etc.) would make
  the reader refuse rather than judge. Accepted: the real corpus is three ordinary `impl Observer` blocks and one
  ordinary free function, none of which are unusual, and a refusal is the stated safe direction rather than a
  silent pass either way.

## Migration Plan

None required. `kanhe` ships in no package; nothing here is adopter-visible. The bound identity removed from
`crates/kanhe/src/bounds.rs` is repository-governance vocabulary that has never been part of
`tianheng::observation_bounds()`.

## Open Questions

None outstanding; `observation_bound_model`'s bijection (`every_declared_bound_is_classified_and_every_classification_names_one`)
verifies directly that removing the bound from both the spec (by dropping its "— a stated bound" heading) and
`crates/kanhe/src/bounds.rs` keeps the two sides equal.
