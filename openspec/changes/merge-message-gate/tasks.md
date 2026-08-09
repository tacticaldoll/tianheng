## 1. The judgement

- [x] 1.1 `crates/tianheng/tests/support/merge_message_gate.rs` — `judge(subject, body, title)` returning the
      shared `Refusal`, refusals ordered most-specific first
- [x] 1.2 Hold: subject equals the title; no trailing `(#N)`; conventional shape with an allowed lowercase
      type; `!` requires a `BREAKING CHANGE:` footer; no agent attribution; a non-empty body; not GitHub's
      concatenated commit list
- [x] 1.3 An unreadable or empty title is a **cannot-judge**, not a disagreement

## 2. The reaction and its matrix

- [x] 2.1 `crates/tianheng/tests/merge_message.rs` — one direction per refusal, each asserting the kind and its
      own message, so no two sites can stand in for each other
- [x] 2.2 The accepted shape passes, so every refusal is about the thing it names
- [x] 2.3 The gate over env-supplied inputs, gated by `TIANHENG_MERGE_MESSAGE`
- [x] 2.4 Declare the residual bound (`a_merge_made_outside_the_wrapper_is_not_observed`) in the spec and in
      `crates/tianheng/src/bounds.rs`, with a pinning test
- [x] 2.5 `refusal_bites` green: every new site reached and distinguished, or declared

## 3. The wrapper

- [x] 3.1 `scripts/merge-pr.sh` — read the PR title, run the reaction, then `gh pr merge --squash`
- [x] 3.2 Refuse before the judgement anything that would move it off the PR being merged, as `publish.sh` does
      for `--manifest-path`
- [x] 3.3 See it refuse: run it against a subject carrying `(#N)` and against one that differs from the title

## 4. Records and closure

- [x] 4.1 `AGENTS.md` names the wrapper where the merge is decided
- [x] 4.2 `CHANGELOG.md` `[Unreleased]` under `### Self-governance`, no version bump
- [x] 4.3 Full Definition of Done including MSRV 1.85 and all gated suites
- [x] 4.4 Sync the delta, archive, and merge **through the new wrapper** — its first use is this change
