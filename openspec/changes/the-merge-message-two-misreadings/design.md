## Context

Both defects are over-reactions in the gate standing in front of a record that cannot be amended. A wrapper
that refuses a legal message blocks a merge as surely as one that misses a defect.

## Goals / Non-Goals

**Goals:** the breaking marker is the head's; a bare commit list is recognised by content.
**Non-Goals:** any change to what the gate refuses about a serial, a title, or an attribution.

## Decisions

### The exact question is *are these the commits*, and the wrapper can answer it

`scripts/merge-pr.sh` knows the pull request. Passing its commit subjects makes the judgement exact rather
than a shape heuristic, at the cost of one more input and one more cannot-judge.

**Read from `git log`, not from the API.** Measured: `gh pr view --json commits --jq '.commits[].messageHeadline'`
truncates at 69 characters with an ellipsis, while this repository's subjects run to 75 — comparing against it
would never match, and a rule that never matches is the miss the gate was written against.

### Tightening the recogniser was rejected

Requiring a bullet to look like a Conventional Commit would refuse a hand-written `- fix: …` body while a
branch carrying one non-conventional subject slipped through. Tightening a detector while leaving the
requirement is how a floor opens.

### An unread subject list refuses rather than falling back

Falling back to the shape would restore the over-reaction being removed, in exactly the case where nobody is
watching.

## Risks / Trade-offs

- **[One more input to the sanctioned path]** → it is read where the wrapper already reads the title, and a
  read that fails refuses.
- **[A body legitimately quoting one commit subject as a bullet]** → refused, since every bullet being a
  commit subject is the test and a one-bullet body of a commit subject is GitHub's default for a one-commit
  branch. Stated rather than left to be discovered.
