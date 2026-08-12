## MODIFIED Requirements

### Requirement: A squash message SHALL be judged before the merge that records it

A proposed squash subject and body SHALL be judged by a check before `gh pr merge` runs, and the sanctioned
path to that merge SHALL be a wrapper that cannot be reached without the judgement.

The subject SHALL equal the pull request's title exactly, and SHALL NOT carry a trailing `(#N)`. The rule is
already written; what is new is that something holds it. Measured over this repository's history, **9**
subjects carry that serial, the most recent on the commit that landed a check for a requirement enforced by
nothing.

The judgement SHALL be a Rust check returning the shared kinded refusal, so that a title that could not be
read is separated from a subject that disagrees, and so that its own construction sites are swept like every
other. Only the wrapper SHALL be shell, and it SHALL carry no verdict.

The refusals SHALL be ordered from most specific to least: a subject carrying a serial also differs from its
title and is also still conventional-shaped, and reporting the general fact for the specific one sends a reader
to compare two strings that differ by exactly the thing the rule names.

**What the gate judged SHALL be what the act records, for every judged input and not for the message alone.**
An input the gate received as a **value** SHALL reach `gh pr merge` as that value, never as a path or other
reference the tool re-resolves after the gate has run. The interval between the two holds a whole `cargo test`
run, and what lands cannot be amended — a squash commit's hash is cited by the pull request's merge record, so
correcting the commit afterwards decouples the two. This is the local half of the pin the head requirement
makes remotely: a pull request that moved is refused, and an input that moved on disk is never read a second
time.

The obligation is stated over the whole set because three of the four inputs already satisfied it while the
fourth did not, and nothing said they were one set: the subject travelled as a value, the repository was
resolved once and named on every call, the head was captured before the commit set and supplied as
`--match-head-commit`, and the live commit subjects were pinned through that head — while the body was handed
over as the path it had been read from. The wrapper's own allowlist already refuses a **caller's** body flag
in every spelling, naming this same split as its reason, which is what makes the wrapper composing one itself
a defect rather than a gap.

#### Scenario: A squash subject carries the pull request's number

- **WHEN** a proposed subject ends in `(#N)`
- **THEN** the merge is refused, naming the serial rather than the fact that the subject differs from the title

#### Scenario: A squash subject is not the pull request's title

- **WHEN** a proposed subject differs from the title in any other way
- **THEN** the merge is refused; the title is what review saw, and a subject that says something else makes the
  record disagree with what was approved

#### Scenario: The pull request's title cannot be read

- **WHEN** the title is unavailable
- **THEN** the check refuses as a cannot-judge rather than as a disagreement, because an unread title is not
  a wrong subject

#### Scenario: The body file changes between the gate and the merge

- **WHEN** the file a body was read from is modified after the gate judged that body and before the merge runs
- **THEN** the merge records the value the gate judged, because the wrapper hands the tool that value and never
  the path it came from

#### Scenario: A hook is proposed for this rule — a stated bound

- **WHEN** someone reaches for a `commit-msg` hook, or for the repository's squash-title setting
- **THEN** neither holds it: a squash merge runs on GitHub's servers so no local commit exists and no hook
  runs, and both values of that setting append the serial. Nor can a merge made in the browser be reached by a
  wrapper. The compliance point is one string passed at merge time, and this check guards the sanctioned
  path to it rather than every path
- **PINNED-BY** `a_merge_made_outside_the_wrapper_is_not_observed`
