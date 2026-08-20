## 1. Add the syn-based extent step alongside the old one

- [x] 1.1 Add `FnBodies` (a `syn::visit::Visit` implementation collecting every body-carrying `ItemFn` /
  `ImplItemFn` / `TraitItemFn`'s `(fn-keyword byte offset, block byte range)`) and `syn_body_span`, which parses
  the source, runs the visitor, and returns the block range whose `fn`-keyword offset matches the anchor.
- [x] 1.2 Leave `mask_line_comment_braces` and `function_body`'s brace-counting loop in place for now, so both
  extent readers exist side by side.

## 2. Observe the old reader's documented failure, and prove the new one right on it

- [x] 2.1 Confirm the existing `AnchorCase` table and the real corpus
  (`crates/guibiao/src/observer.rs`, `crates/hunyi/src/observer.rs`, `crates/louke/src/observer.rs`,
  `crates/tianheng/src/runner.rs`) already exercise the ordinary and comment-tail shapes without regression.
- [x] 2.2 Add two fixtures the old reader is documented to get wrong: a conforming `bounds()` body carrying a
  `}` inside a block comment, and one carrying a `}` inside a string literal. Run `syn_body_span` against both
  and confirm it now reaches the real closing brace in each — where the old brace-counter stopped short.

## 3. Cut over

- [x] 3.1 Rewrite `function_body` to call `anchor` (unchanged) and then `syn_body_span`, deleting the
  brace-counting loop.
- [x] 3.2 Delete `mask_line_comment_braces` — dead once `function_body` no longer calls it.
- [x] 3.3 Rewrite `a_brace_in_a_block_comment_moves_the_body_extent` — renamed
  `a_brace_in_a_block_comment_or_a_string_literal_no_longer_moves_the_body_extent` — to assert the corrected
  extent directly via `function_body`, plus the still-refusing (for a separate, correct reason) control through
  `bounds_body`.
- [x] 3.4 Update the two `AnchorCase` rows ("a whole-line copy in a block comment…" / "…in a string
  literal…") from `Verdict::ReadsTheWrongBody` to `Verdict::Declines`, matching the reader's real new behaviour.
- [x] 3.5 Add the new `AnchorCase` row ("a same-named method on an unrelated impl anchors the read, the intended
  definition absent") so the still-open anchor bound's "shown rather than described" requirement keeps at least
  one `Verdict::ReadsTheWrongBody` row to satisfy it.
- [x] 3.6 Run `TIANHENG_WORKSPACE_TESTS=1 cargo test -p kanhe --test observer_protocol` and confirm every
  existing `#[test]` name (besides the one renamed in 3.3) is unchanged and green.

## 4. Close the bound in the spec and in code

- [x] 4.1 In `openspec/specs/observer-protocol/spec.md`, rewrite only the scenario "A brace inside a block
  comment or a string literal moves the read body extent — a stated bound": drop the `— a stated bound` framing
  and its `PINNED-BY`, and state the new implementation's own failure mode (refuses to verify rather than
  passing, on a parse failure or a missing match at the anchor). Every other scenario under the same requirement
  is carried over verbatim, including the still-open whole-line-occurrence bound.
- [x] 4.2 Delete the bound's typed declaration
  (`observer-protocol/a-brace-inside-a-block-comment-or-a-string-literal-moves-the-read-body-extent-a-stated-bound`)
  from `crates/kanhe/src/bounds.rs`.
- [x] 4.3 Sync the delta into `openspec/specs/observer-protocol/spec.md`.
- [x] 4.4 Regenerate `docs/observation-bounds.md`
  (`BLESS=1 TIANHENG_WORKSPACE_TESTS=1 cargo test -p kanhe --test bound_register`) and
  `docs/observation-bound-extents.md`
  (`BLESS=1 TIANHENG_WORKSPACE_TESTS=1 cargo test -p kanhe --test observation_bound_model`), and confirm both
  gates pass again without `BLESS`.

## 5. Prove the change against the full gate list

- [x] 5.1 `cargo build --workspace`
- [x] 5.2 `cargo clippy --all-targets --all-features -- -D warnings`
- [x] 5.3 `cargo fmt --all --check`
- [x] 5.4 `TIANHENG_WORKSPACE_TESTS=1 cargo test --workspace --all-features`
- [x] 5.5 `npx --no-install openspec validate --specs --strict`
- [x] 5.6 `TIANHENG_WORKSPACE_TESTS=1 cargo test -p kanhe --test reference_integrity`
