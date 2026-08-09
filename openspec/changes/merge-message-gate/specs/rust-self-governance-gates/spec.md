## ADDED Requirements

### Requirement: A squash message SHALL be judged before the merge that records it

A proposed squash subject and body SHALL be judged by a reaction before `gh pr merge` runs, and the sanctioned
path to that merge SHALL be a wrapper that cannot be reached without the judgement.

The subject SHALL equal the pull request's title exactly, and SHALL NOT carry a trailing `(#N)`. The rule is
already written; what is new is that something holds it. Measured over this repository's history, **9**
subjects carry that serial, the most recent on the commit that landed a reaction for a requirement enforced by
nothing.

The judgement SHALL be a Rust reaction returning the shared kinded refusal, so that a title that could not be
read is separated from a subject that disagrees, and so that its own construction sites are swept like every
other. Only the wrapper SHALL be shell, and it SHALL carry no verdict.

The refusals SHALL be ordered from most specific to least: a subject carrying a serial also differs from its
title and is also still conventional-shaped, and reporting the general fact for the specific one sends a reader
to compare two strings that differ by exactly the thing the rule names.

#### Scenario: A squash subject carries the pull request's number

- **WHEN** a proposed subject ends in `(#N)`
- **THEN** the merge is refused, naming the serial rather than the fact that the subject differs from the title

#### Scenario: A squash subject is not the pull request's title

- **WHEN** a proposed subject differs from the title in any other way
- **THEN** the merge is refused; the title is what review saw, and a subject that says something else makes the
  record disagree with what was approved

#### Scenario: The pull request's title cannot be read

- **WHEN** the title is unavailable
- **THEN** the reaction refuses as a cannot-judge rather than as a disagreement, because an unread title is not
  a wrong subject

#### Scenario: A hook is proposed for this rule — a stated bound

- **WHEN** someone reaches for a `commit-msg` hook, or for the repository's squash-title setting
- **THEN** neither holds it: a squash merge runs on GitHub's servers so no local commit exists and no hook
  runs, and both values of that setting append the serial. Nor can a merge made in the browser be reached by a
  wrapper. The compliance point is one string passed at merge time, and this reaction guards the sanctioned
  path to it rather than every path
- **PINNED-BY** `a_merge_made_outside_the_wrapper_is_not_observed`
