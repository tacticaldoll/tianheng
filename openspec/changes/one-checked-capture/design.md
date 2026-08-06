## Context

Eight producers were migrated to one checked capture across two gates. Nothing stops the ninth from being written
`done < <(git ls-files)`, and `BACKLOG.md` records nine prior repairs of exactly that.

## Goals / Non-Goals

**Goals.** Make the migrated shape unwritable rather than repaired again. Keep the declared bound honest about what
is still unobserved.

**Non-Goals.** Observing *every* unchecked read. A command substitution whose status nobody inspects, and a pipeline
whose non-final stage fails, are both still invisible — and that is what the narrowed bound now says.

## Decisions

### D1 — The property is about `< <(`, not about "every read"

`< <(` is the construct all eight measured sites used and the one whose status the parent structurally cannot see.
A property claiming to observe every unchecked read would be the overclaim this capability exists to refuse: a
`$(cmd)` without `||` is a different shape, and detecting it needs to know whether the caller inspects `$?`
afterwards — which is control flow, not text.

Narrow and true beats broad and false. The rest stays a declared bound.

### D2 — A builtin over memory is permitted, and the permission is by producer rather than by count

The two sites left after the migration are `done < <(printf '%s\n' "${b//|/$'\n'}")` — a builtin re-splitting a
variable already held. It has no I/O to fail at, so requiring a temporary file for it would be ceremony that makes
the gate longer without making it safer.

Permitted by naming the builtins rather than by allowlisting the two lines: an allowlist of line numbers rots on the
next edit, and the property would then be about where the code is rather than what it does.

### D3 — The bound narrows, and its heading does not move

The heading's slug **is** the bound's id, so renaming it would change the id — breaking the `PINNED-BY` citation and
the typed declaration in `tianheng::observation_bounds()` in one edit, and moving a row in two projections for a
reason that has nothing to do with the bound's content.

So the heading stays and the body narrows. This is the first time in this repository a bound has been *narrowed*
rather than declared or retired, and the mechanics are worth recording: same heading, same id, same pin, changed
WHEN/THEN, changed rationale in the typed declaration. The extents projection re-renders; the register's figures do
not move.

### D4 — Landing green is a requirement, not an aspiration

The migration landed first (`795b808`, `1df4e88`) precisely so this property is 6 of 6 on arrival. The capability's
own D4 forbids a reaction landing with known-failing units — it would establish that its exemptions are negotiable.

## Risks / Trade-offs

**A gate could evade the property by spelling the read differently.** `while read … done < "$(mktemp_output)"` or an
`eval` would not match. The property is aimed at an author who reaches for the familiar construct, which is what all
nine recorded recurrences were; an author deliberately evading it is outside what any text reaction reaches, and the
narrowed bound covers the honest remainder.

**Eleven properties invites a twelfth.** The guard is the same as for the tenth: each must be a class this repository
*observed*. This one has ten recorded instances and two measured failure directions, which is the strongest warrant
of any property in the set.
