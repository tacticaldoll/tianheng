## Why

`AGENTS.md` states the squash subject rule exactly — *set the squash subject exactly to the PR title with no
auto-appended `(#N)`* — and nothing holds it. Measured over the whole history: **9 of the subjects on this
repository's default branch carry a trailing `(#N)`**, the most recent being the commit that landed a reaction
for a requirement enforced by nothing.

The rule cannot be held where rules are usually held here. A squash merge happens on GitHub's servers, so no
local commit exists and no git hook runs; and neither value of `squash_merge_commit_title` suppresses the
append, so the repository setting cannot fix it either. The only way to obtain a subject without the serial is
to pass one explicitly — which makes the compliance point **one string typed at merge time**, at the moment a
record lands on a release branch and stops being repairable.

That is the same shape `scripts/publish.sh` already exists for: a rule that was written, then missed, at the
one moment nothing can be undone.

## What Changes

- **A new reaction, `crates/tianheng/tests/merge_message.rs`**, judging a proposed squash subject and body
  against the PR title, with a failure matrix. The judgement returns the shared kinded `Refusal`, so its
  construction sites are swept by `refusal_bites` like every other.
- **A new wrapper, `scripts/merge-pr.sh`**, which reads the PR title, runs the judgement, and only then calls
  `gh pr merge --squash`. Behind it the check stops being a step to remember: it is the only sanctioned way to
  reach the merge.
- **`AGENTS.md`** names the wrapper where the merge is decided, as it does for the publish path.

## Capabilities

### New Capabilities

None.

### Modified Capabilities

- `rust-self-governance-gates`: the squash-message rule gains a reaction, and the requirement records why a
  hook cannot hold it.

## Impact

- **New**: `crates/tianheng/tests/support/merge_message_gate.rs`, `crates/tianheng/tests/merge_message.rs`,
  `scripts/merge-pr.sh`.
- **Amended**: `AGENTS.md`, `CHANGELOG.md`.
- **Not repaired**: the nine subjects already in history. A commit message is a measurement of the moment it
  was taken; rewriting one to satisfy a rule written afterwards would falsify it, and amending a merged squash
  would decouple it from the pull request whose merge record cites its hash.
