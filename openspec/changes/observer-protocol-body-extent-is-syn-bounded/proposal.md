## Why

`observer-protocol`'s bounds-method reader (`crates/kanhe/tests/observer_protocol.rs`) finds a `bounds()`
method's body in two steps: an anchor step decides which occurrence of the signature is the definition, and an
extent step reads that definition's body. The extent step counted braces over a mask that blanked `{`/`}` inside
line-comment tails only. A `{` or `}` written inside a block comment or a string literal was counted as a real
brace, so the read extent could stop short of — or run past — the method's real closing brace.

That gap was already declared —
`observer-protocol/a-brace-inside-a-block-comment-or-a-string-literal-moves-the-read-body-extent-a-stated-bound`
— and typed `Reached::OverReacts`, safe for its one consumer: `bounds_body`'s exact one-statement comparison
cannot be satisfied by a moved extent, so the bug only ever produced a false *refusal* of a conforming
`bounds()` method, never a false *acceptance* of a divergent one. The bound's own prose recorded that a second
reader, once written over the same recognizer with a weaker comparison, turned the identical false refusal into
a false acceptance — and was retired for exactly that reason.

`crates/kanhe/Cargo.toml` gained `syn`/`proc-macro2` (`span-locations`) as dev-dependencies in the commit
immediately before this one, replacing kanhe's hand-rolled Rust-text scanners with a real parser one recognizer
at a time. This change is that replacement for the extent step above: `syn` tokenizes a comment or a string
literal as what it is, so a brace inside either is never available to be counted as one that opens or closes a
body, closing the bound rather than merely re-describing it.

## What Changes

- The extent step (`function_body`'s brace-counting loop, and `mask_line_comment_braces`, the helper that
  approximated comment-awareness for it) is replaced by `syn_body_span`: it parses the source with
  `syn::parse_str::<syn::File>`, walks it with a `syn::visit::Visit` implementation collecting every
  body-carrying function-like item's `fn`-keyword byte offset and its block's exact `{ … }` byte range (via
  `proc-macro2`'s `span-locations`), and returns the range for the item whose `fn` keyword begins exactly where
  the anchor step said the definition starts.
- The anchor step (`anchor`, `begins_a_line`, and every scenario describing what it does and does not find) is
  **not** touched. The still-open bound over it —
  `observer-protocol/a-whole-line-occurrence-that-is-not-the-definition-anchors-the-read-a-stated-bound` —
  remains exactly as declared.
- **Safety is preserved by construction, not by re-deriving the old comparison's argument.** If the source does
  not parse as a Rust file, or parses without a function-like item beginning exactly at the anchored byte
  offset, `syn_body_span` returns `None` and the read declines — the same refuse-rather-than-guess direction the
  closed bound's pin required, restated as this implementation's own failure mode rather than inherited as an
  argument about brace-counting that no longer applies.
- The differential proof: the new extent step was added alongside the old one, both were run against every row
  of the existing `AnchorCase` table and against the real corpus (`crates/guibiao/src/observer.rs`,
  `crates/hunyi/src/observer.rs`, `crates/louke/src/observer.rs`, `crates/tianheng/src/runner.rs`), and two new
  fixtures — a conforming `bounds()` body carrying a `}` inside a block comment, and one carrying a `}` inside a
  string literal — were added to show the new reader now reads each to its real closing brace, where the old one
  stopped short. Only then was the old brace-counting internals deleted.
- `a_brace_in_a_block_comment_moves_the_body_extent` pinned the old, wrong behaviour. It is rewritten (and
  renamed `a_brace_in_a_block_comment_or_a_string_literal_no_longer_moves_the_body_extent`) to assert the extent
  directly, rather than through `bounds_body`'s stricter comparison — a block comment on its own line is still
  not stripped by `kanhe::region`'s `Executed` (an unrelated, already-declared residue), so `bounds_body` still
  refuses that specific fixture, now for that separate, correct reason instead of for a truncated extent.
- **A second-order consequence of fixing the extent step, not a fix to the anchor step**: the `AnchorCase` rows
  "a whole-line copy in a block comment, the definition moved out of the file" and "…in a string literal…" used
  to demonstrate the anchor bound by `Verdict::ReadsTheWrongBody`, because the old extent step happily
  brace-counted through the decoy's own text. A real parser cannot do that — there is no function-like item
  inside a comment or a string literal for it to find — so both rows now correctly read `Verdict::Declines`. The
  anchor bug itself is untouched and still anchors on that occurrence; only its consequence changed, from a
  silent misread to a refusal. Because the requirement's own text says this bound "SHALL be shown rather than
  described" by at least one row that reads a wrong body, a new row was added — "a same-named method on an
  unrelated impl anchors the read, the intended definition absent" — using the literal "or in any other position
  the reader does not distinguish from executed text" clause the still-open scenario already carries. This is
  the one place the intended scope (the extent step alone) could not be held perfectly cleanly: the fix to the
  extent step necessarily changes which concrete fixtures demonstrate the still-open anchor bound, even though
  the anchor rule itself is unedited.
- The spec scenario `A brace inside a block comment or a string literal moves the read body extent — a stated
  bound` is rewritten to drop the `— a stated bound` framing and its `PINNED-BY` retarget: the bound it declared
  is closed. Every other scenario under the same requirement — including the still-open whole-line-occurrence
  bound — is carried over verbatim.
- The bound's typed declaration in `crates/kanhe/src/bounds.rs` is deleted, and `docs/observation-bounds.md` /
  `docs/observation-bound-extents.md` are regenerated to match.

Not **BREAKING**. `kanhe` ships in no package (`publish = false`), so nothing here reaches an adopter's
compiled surface or recorded baseline. The bound identity removed from `crates/kanhe/src/bounds.rs` is
repository-governance vocabulary, not `tianheng::observation_bounds()`.

## Capabilities

### New Capabilities

None.

### Modified Capabilities

- `observer-protocol`: one requirement changes — *The built-in path SHALL keep its behaviour, and the two paths
  SHALL be held equal* — narrowly, at the one scenario naming the now-closed extent bound. Every other scenario
  under that requirement, including the still-open anchor bound, is unchanged.

## Impact

- **Code**: `crates/kanhe/tests/observer_protocol.rs` (the extent step, the rewritten/renamed pin, the
  `AnchorCase` table), `crates/kanhe/src/bounds.rs` (the closed bound's declaration removed).
- **Specs**: `openspec/specs/observer-protocol/spec.md` — the one scenario, at sync.
- **Docs**: `docs/observation-bounds.md`, `docs/observation-bound-extents.md` — regenerated, never hand-edited.
- **Dependencies**: none added; `syn`/`proc-macro2` were already added to `crates/kanhe`'s dev-dependencies in
  the immediately preceding commit.
