## 1. Make the reaction drivable by fixtures

- [x] 1.1 Extract the judgement inside `the_shell_delegates_semantic_emptiness_to_the_public_entry_point` into a
  helper over `&Source` returning a verdict that distinguishes delegating, divergent, and cannot-judge. The test
  keeps reading the tracked `crates/tianheng/src/runner.rs` and calls the helper.
- [x] 1.2 Verify behaviour is preserved: `cargo test -p tianheng --test observer_protocol` passes with the test
  otherwise unchanged.

## 2. Observe the false negative before closing it

- [x] 2.1 Add a fixture holding the delegation, `if "https://host".is_empty() {` with its closing brace on a
  later line, and a further `constitution.semantic_boundaries()` access below — asserting the helper does not
  report delegation.
- [x] 2.2 Add the same shape with a bare `}` inside a string literal, no `//` present.
- [x] 2.3 Add the same shape with a bare `}` inside a block comment, and one with `let c = '}';`.
- [x] 2.4 Run all four and **record the observed failure output** for the PR's `## Verification`. They must fail
  now — each currently reports the divergent body as delegating.
- [x] 2.5 Add the control fixture: the same body with no delimiter, second access present, which must already be
  reported as divergent. Without it the four above could pass by the helper refusing everything.

## 3. Close it with a refusal

- [x] 3.1 In the helper, scan the read extent's executed lines (`body.rust()`) for `"`, `'`, `/*`, or `*/` and
  return cannot-judge when any is present, naming the delimiter found and stating that the extent may not be the
  function's body.
- [x] 3.2 Confirm the four fixtures from group 2 now refuse, and the control from 2.5 still reports divergent.
- [x] 3.3 Add `the tracked composition body is still judged` — the positive control asserting the real
  `evaluate_constitution` yields a verdict, not a refusal, so a refuse-everything implementation fails.

## 4. Re-declare the bound across every site it lives in

- [x] 4.1 In `crates/tianheng/src/bounds.rs`, narrow the existing declaration to the bounds-method comparison,
  keeping `Reached::OverReacts` and its pin `a_brace_in_a_block_comment_moves_the_body_extent`.
- [x] 4.2 Add the second declaration for the delegation reaction, typed `Reached::RefusesToJudge`, pinned by
  `an_ambiguous_delegation_extent_is_refused_rather_than_judged`.
- [x] 4.3 Confirm the pin test names in both declarations match the test function names exactly, since the
  register resolves citations through the harness enumeration.
- [x] 4.4 Regenerate the projections: `BLESS=1 bash scripts/check_bound_register.sh`, then
  `BLESS=1 TIANHENG_WORKSPACE_TESTS=1 cargo test -p tianheng --test observation_bound_model`.
- [x] 4.5 Re-run both without `BLESS` and confirm green — `bash scripts/check_bound_register.sh` and
  `TIANHENG_WORKSPACE_TESTS=1 cargo test -p tianheng --test observation_bound_model`.

## 5. Sweep the retired claim out of every file that states it

- [x] 5.1 Update the rustdoc above `a_brace_in_a_block_comment_moves_the_body_extent` so its safe-direction
  claim is scoped to the one-statement comparison.
- [x] 5.2 Update `mask_line_comment_braces`' doc comment, which asserts the same direction for all readers of
  the extent.
- [x] 5.3 Grep the whole of `crates/tianheng/tests/observer_protocol.rs`, `crates/tianheng/src/bounds.rs`, and
  the delta spec for the retired wording — the vocabulary sweep is whole-file, not diff-only.

## 6. Prove the change against the full gate list

- [x] 6.1 Run the Definition of Done from `AGENTS.md` in its stated order, including
  `bash scripts/check_reference_integrity.sh` and `bash scripts/check_whitespace_hygiene.sh`.
- [x] 6.2 Add the `[Unreleased]` entry to `CHANGELOG.md` stating the closure and the re-classification, then
  `bash scripts/check_release_coherence.sh`.
- [ ] 6.3 Assemble the PR body: `## Why`, `## What changed`, `## Adversarial review`, `## Verification` naming
  the recorded failures from 2.4, and `## Compatibility` stating that no adopter action follows because the
  recognizer ships in no crate and the bound identities are absent at `v0.4.0`.
