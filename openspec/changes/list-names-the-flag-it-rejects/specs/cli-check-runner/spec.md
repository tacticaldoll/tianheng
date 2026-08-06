## MODIFIED Requirements

### Requirement: List rejects flags that only apply to check

The `list` command observes no workspace and performs no reaction; it SHALL accept only `--format`. Supplying a
flag that `check` recognizes and `list` does not honor SHALL be a usage error that exits `2`, never silently
ignored. This complements the unrecognized-argument rule: a flag that is recognized by `check` but inapplicable to
`list` SHALL be rejected rather than accepted as a silent no-op.

The rejected set SHALL be **derived** from the flags `check` recognizes rather than enumerated here. A prose list
of it has already gone stale once — it named four flags while the runner rejected five, because `--disallow-stale`
was added to the runner and not to this requirement — and a requirement that under-describes a correct
implementation is a requirement a later reader will implement to.

The refusal SHALL **name the flags the invocation supplied**, and SHALL name all of them rather than the first. This
is the same obligation the requirement governing conflicts *within* `check` states while citing this one as the rule
it extends; before this, the two disagreed and each implementation satisfied its own, which is why no test caught
it. Naming matters most here because `--manifest-path` is in the rejected set and is the flag a user types by habit:
told only that "`list` takes only `--format`", a reader who passed both `--manifest-path` and `--format` is being
shown the flag they got right.

The order of the named flags SHALL be a function of the set and not of the command line, so two invocations
supplying the same flags in different order receive the same diagnostic.

#### Scenario: A check-only flag supplied to list is a usage error naming it

- **WHEN** the runner is invoked as `list --baseline <file>` (or with `--write-baseline`, `--manifest-path`,
  `--warn-uncovered`, or `--disallow-stale`)
- **THEN** it exits `2`, prints usage guidance, and names the flag that was supplied — never silently ignoring it,
  and never reporting only that some inapplicable flag was present

#### Scenario: Several check-only flags are all named

- **WHEN** the runner is invoked as `list` with more than one check-only flag
- **THEN** every one of them is named, because reporting the first would send the reader back for a second round

#### Scenario: List still accepts the format flag

- **WHEN** the runner is invoked as `list --format json` (or `list --format text`, or `list` with no flag)
- **THEN** it prints the projection and exits `0`, because `--format` is the one flag `list` honors
