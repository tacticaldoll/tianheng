## ADDED Requirements

### Requirement: The squash-message gate SHALL refuse a shape by what it is, not by what it resembles

The gate SHALL refuse a message for what it is rather than for what it resembles: a refusal at the merge is an
over-reaction as costly as a miss, blocking a legal record at the one moment nothing can be undone. Two of its
checks read a resemblance rather than the thing.

The **breaking marker** SHALL be read from the Conventional Commit head — the text before `": "` — never from
anywhere in the subject. `fix(tianheng): preserve bang! in summaries` announces no migration, and the ability
to read the head already sits in the same judgement, which strips a trailing `!` before matching the type.

A **bare commit list** SHALL be recognised by what its bullets say. The pull request's own commit subjects
SHALL be supplied to the judgement, and a body SHALL be a bare list when every bullet is one of them. A body
of `- Why: …` / `- Contract: …` is self-contained and its shape is not the question.

Tightening the recogniser instead — requiring a bullet to look like a Conventional Commit — SHALL NOT be used:
every commit in this repository is conventional, so it would refuse a hand-written body of `- fix: …` bullets
while a branch carrying one non-conventional subject slipped through.

Where the commit subjects cannot be read, the judgement SHALL refuse as a cannot-judge rather than fall back
to the shape, because falling back is the over-reaction being removed.

#### Scenario: A summary containing an exclamation mark

- **WHEN** a subject carries `!` after the `": "` and its head does not end in one
- **THEN** the message is accepted without a `BREAKING CHANGE:` footer, while a head ending in `!` still
  requires one

#### Scenario: A terse body written entirely as bullets

- **WHEN** every non-blank line of the body is a bullet and none of them is one of the pull request's commit
  subjects
- **THEN** the message is accepted: the body is self-contained, and its formatting is not what the rule is
  about

#### Scenario: GitHub's default body

- **WHEN** the body's bullets are the pull request's commit subjects
- **THEN** the merge is refused, which is the default this rule exists to replace

#### Scenario: The commit subjects cannot be read

- **WHEN** the wrapper supplies no commit subjects
- **THEN** the judgement refuses as a cannot-judge naming what it could not read, rather than falling back to
  the shape it was refusing before
