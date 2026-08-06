## Context

`dispatch_list` refuses a check-only flag with a single `usage(...)` call behind an `||` over five conditions, so
the message cannot say which condition fired. The `check`-internal conflict rule, written later, both cites that
requirement as "the same rule" and requires the flag to be named — so the two requirements disagree while each
implementation matches its own.

## Goals / Non-Goals

**Goals.** Make the two requirements say one thing. Make `list`'s refusal name what the invocation did, like every
other cell of this surface. Remove the prose enumeration that already went stale once.

**Non-Goals.** Changing any exit code. Changing `--format`'s value refusal, which is already precise and explains
*why* `sarif` is not a `list` format. Rejecting a repeated boolean flag — `cli-check-runner` already decides that
deliberately, and says why: the second occurrence asks for what the first set, so nothing is dropped and there is
no ambiguity to report.

## Decisions

### D1 — The rejected set is derived, not enumerated in prose

The requirement named four flags; the runner checks five. `--disallow-stale` was added later and only to the code.
Rather than adding it to the prose — which repairs today's instance and leaves the mechanism — the requirement now
says *every flag `check` recognizes that `list` does not honor*, so a sixth flag is covered the moment it exists.

The trade is that the requirement no longer lets a reader see the set without reading the parser. That is the right
side of the trade here: a reader who needs the set can run `tianheng list --manifest-path x` and be told, which is
precisely the capability this change adds.

### D2 — The message lists the flags given, in the order the parser recognizes them

Not "the first offending flag": an invocation may supply several, and reporting one would send a reader back for a
second round. Order comes from the check itself rather than from the command line, so the message is a function of
what was supplied and not of how it was typed — two invocations with the same flags in different order get the same
diagnostic, which is what makes it assertable.

### D3 — This is a spec correction, not a bug fix, and the difference matters

The implementation satisfies the requirement it was written against. Filing it as a bug would record that someone
mis-implemented a clear rule, when what actually happened is that a later requirement described an earlier one as
"the same rule" while strengthening it. Recording that honestly is the point: the defect is in the pair, and a fix
that only touched the code would leave the two requirements still disagreeing.

### D4 — The guard is per flag, driven from the set

One test asserting "some flag is named" would pass while four of the five went unnamed. Each of the five is driven
individually and asserted to appear in the message, which is the same reason `gate-shape-contract`'s per-property
test exists: a reaction that fails only in aggregate cannot be trusted to have five reasons.

## Risks / Trade-offs

**A message change is a compatibility surface for anyone grepping it.** Bounded and stated: the exit code does not
change, the `usage:` banner does not change, and the sentence that changes is a refusal a caller cannot be relying
on for anything but its exit status. `CHANGELOG.md` records it as a fixed diagnostic rather than as a behaviour
change, because no invocation's verdict moves.

**The sweep's enumeration is hand-made, and this change does not make it live.** The CLI surface has no reaction
enumerating it; the measurement in `proposal.md` is a snapshot taken against a named revision, and it will rot the
way the requirement's own list did. That is recorded in `BACKLOG.md` with a trigger — a second defect found in this
surface, or a flag added without a naming test — rather than solved here, because a fifth register over six flags
would be ceremony the finding does not justify.
