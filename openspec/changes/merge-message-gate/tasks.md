## 1. The judgement

- [ ] 1.1 `crates/tianheng/tests/support/merge_message_gate.rs` — `judge(subject, body, title)` returning the
      shared `Refusal`, refusals ordered most-specific first
- [ ] 1.2 Hold: subject equals the title; no trailing `(#N)`; conventional shape with an allowed lowercase
      type; `!` requires a `BREAKING CHANGE:` footer; no agent attribution; a non-empty body; not GitHub's
      concatenated commit list
- [ ] 1.3 An unreadable or empty title is a **cannot-judge**, not a disagreement

## 2. The reaction and its matrix

- [ ] 2.1 `crates/tianheng/tests/merge_message.rs` — one direction per refusal, each asserting the kind and its
      own message, so no two sites can stand in for each other
- [ ] 2.2 The accepted shape passes, so every refusal is about the thing it names
- [ ] 2.3 The gate over env-supplied inputs, gated by `TIANHENG_MERGE_MESSAGE`
- [ ] 2.4 Declare the residual bound (`a_merge_made_outside_the_wrapper_is_not_observed`) in the spec and in
      `crates/tianheng/src/bounds.rs`, with a pinning test
- [ ] 2.5 `refusal_bites` green: every new site reached and distinguished, or declared

## 3. The wrapper

- [ ] 3.1 `scripts/merge-pr.sh` — read the PR title, run the reaction, then `gh pr merge --squash`
- [ ] 3.2 Refuse before the judgement anything that would move it off the PR being merged, as `publish.sh` does
      for `--manifest-path`
- [ ] 3.3 See it refuse: run it against a subject carrying `(#N)` and against one that differs from the title

## 4. Records and closure

- [ ] 4.1 `AGENTS.md` names the wrapper where the merge is decided
- [ ] 4.2 `CHANGELOG.md` `[Unreleased]` under `### Self-governance`, no version bump
- [ ] 4.3 Full Definition of Done including MSRV 1.85 and all gated suites
- [ ] 4.4 Sync the delta, archive, and merge **through the new wrapper** — its first use is this change
