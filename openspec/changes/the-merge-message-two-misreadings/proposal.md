## Why

The squash-message gate refuses two shapes it should not, and both are over-reactions at the one moment
nothing can be undone — a wrapper that refuses a legal message blocks a merge as surely as a defect.

**Any `!` in a summary is read as a breaking marker.** The check is `subject.contains('!')`, so
`fix(tianheng): preserve bang! in summaries` is required to carry a `BREAKING CHANGE:` footer it has no reason
to. The ability to read the Conventional Commit head is already in the same file, five lines above: the
shape check strips a trailing `!` from the head before matching the type.

**A body written entirely as bullets is read as GitHub's commit list.** The check refuses any body whose
non-blank lines are all bullets, so a self-contained body of `- Why: …` / `- Contract: …` is refused for its
formatting rather than its content.

## What Changes

- The breaking marker is read from the Conventional Commit **head** — the text before `": "` — rather than
  from anywhere in the subject.
- A bare commit list is recognised by **what its bullets say**, not by their shape: the wrapper supplies the
  pull request's own commit subjects, and a body is a bare list when every bullet is one of them.

Tightening the detector instead — requiring bullets to look like Conventional Commits — was considered and
rejected: every commit in this repository is conventional, so it would refuse a hand-written body of
`- fix: …` bullets while a branch carrying one non-conventional subject would slip through. The exact
question is *are these the commits*, and the wrapper can answer it.

The subjects come from `git log`, not from the API: measured, GitHub's `messageHeadline` truncates at 69
characters with an ellipsis, and this repository's commit subjects run longer, so comparing against it would
never match.

## Capabilities

### Modified Capabilities

- `rust-repository-reactions`: the squash-message requirement gains what distinguishes a bare commit list
  from a terse self-contained body, and states that the breaking marker is the head's.

`release-coherence` claims `CHANGELOG.md`, which records this; its requirements do not change.

## Impact

- `crates/kanhe/src/merge_message_gate.rs`, `crates/kanhe/tests/merge_message.rs`, `scripts/merge-pr.sh`.
- The merge wrapper reads the pull request's commit subjects, adding one `git log` to the sanctioned path.
- No version change.
